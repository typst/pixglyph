//! OpenType glyph rendering.
//!
//! - Render glyph outlines into coverage bitmaps.
//! - Place glyphs at subpixel offsets and scale them to subpixel sizes. This is
//!   important if you plan to render more than a single glyph since inter-glyph
//!   spacing will look off if every glyph origin must be pixel-aligned.
//! - No font data structure you have to store somewhere. Just owned glyphs
//!   which you can load individually from a font, cache if you care about
//!   performance, and then render at any size.
//! - No unsafe code.
//!
//! _Note on text:_  This library does not provide any capabilities to map
//! text/characters to glyph ids. Instead, you should use a proper shaping
//! library (like [`rustybuzz`]) to do this step. This will take care of proper
//! glyph positioning, ligatures and more.
//!
//! _Note on emojis:_ This library only supports normal outlines. How to best
//! render bitmap, SVG and colored glyphs depends very much on your rendering
//! environment.
//!
//! [`rustybuzz`]: https://github.com/RazrFalcon/rustybuzz

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::fmt::{self, Debug, Formatter};
use std::ops::{Add, Div, Mul, Sub};

use ttf_parser::{Face, GlyphId, OutlineBuilder, Rect};

/// A loaded glyph that is ready for rendering.
#[derive(Debug, Clone)]
pub struct Glyph {
    /// The number of font design units per em unit.
    units_per_em: u16,
    /// The glyph bounding box.
    bbox: Rect,
    /// The path segments.
    segments: Vec<Segment>,
}

/// A path segment.
#[derive(Debug, Copy, Clone)]
enum Segment {
    /// A straight line.
    Line(Point, Point),
    /// A quadratic bezier curve.
    Quad(Point, Point, Point),
    /// A cubic bezier curve.
    Cubic(Point, Point, Point, Point),
}

impl Glyph {
    /// Load the glyph with the given `glyph_id` from the face.
    ///
    /// This method takes a `ttf-parser` font face. If you don't already use
    /// `ttf-parser`, you can [create a face](ttf_parser::Face::from_slice) from
    /// raw OpenType font bytes with very little overhead.
    ///
    /// Returns `None` if the glyph does not exist or the outline is malformed.
    pub fn load(face: &Face, glyph_id: GlyphId) -> Option<Self> {
        let mut builder = Builder::default();
        Some(Self {
            units_per_em: face.units_per_em().unwrap_or(1000),
            bbox: face.outline_glyph(glyph_id, &mut builder)?,
            segments: builder.segments,
        })
    }

    /// Rasterize the glyph.
    ///
    /// # Placing & scaling
    /// The values of `x` and `y` determine the subpixel positions at which the
    /// glyph origin should reside in some larger pixel raster (i.e. a canvas
    /// which you render text into). This is important when you're rendering the
    /// resulting bitmap into a larger pixel buffer and the glyph origin is not
    /// pixel-aligned in that raster.
    ///
    /// For example, if you want to render a glyph into your own canvas with its
    /// origin at `(3.5, 4.6)` (in pixels) you would use these exact values for
    /// `x` and `y`.
    ///
    /// The `size` defines how many pixels should correspond to `1em`
    /// horizontally and vertically. So, if you wanted to want to render your
    /// text at a size of `12px`, then `size` should be `12.0`.
    ///
    /// # Rendering into a larger canvas
    /// The result of rasterization is a coverage bitmap along with position and
    /// sizing data for it. Each individual coverage value defines how much one
    /// pixel is covered by the text. So if you have an RGB text color, you can
    /// directly use the coverage values as alpha values to form RGBA pixels.
    /// The returned `left` and `top` values define on top of which pixels in
    /// your canvas you should blend each of these new pixels.
    ///
    /// In our example, we have `glyph.rasterize(3.5, 4.6, 12.0, 12.0)`. Now,
    /// let's say the returned values are `left: 3`, `top: 1`, `width: 6` and
    /// `height: 9`. Then you need to apply the coverage values to your canvas
    /// starting at `(3, 1)` and going to `(9, 10)` row-by-row.
    pub fn rasterize(&self, x: f32, y: f32, size: f32) -> Bitmap {
        // Scale is in pixel per em, but curve data is in font design units, so
        // we have to divide by units per em.
        let s = size / self.units_per_em as f32;

        // Determine the pixel-aligned bounding box of the glyph in the larger
        // pixel raster. For y, we flip and sign and min/max because Y-up. We
        // add a bit of horizontal slack to prevent floating problems when the
        // curve is directly at the border (only needed horizontally due to
        // row-by-row data layout).
        let slack = 0.01;
        let left = (x + s * self.bbox.x_min as f32 - slack).floor() as i32;
        let right = (x + s * self.bbox.x_max as f32 + slack).ceil() as i32;
        let top = (y - s * self.bbox.y_max as f32).floor() as i32;
        let bottom = (y - s * self.bbox.y_min as f32).ceil() as i32;
        let width = (right - left) as u32;
        let height = (bottom - top) as u32;

        // Create function to transform individual points.
        let dx = x - left as f32;
        let dy = y - top as f32;
        let t = |p: Point| point(dx + p.x * s, dy - p.y * s);

        // Draw!
        let mut canvas = Canvas::new(width, height);
        for &segment in &self.segments {
            match segment {
                Segment::Line(p0, p1) => canvas.line(t(p0), t(p1)),
                Segment::Quad(p0, p1, p2) => canvas.quad(t(p0), t(p1), t(p2)),
                Segment::Cubic(p0, p1, p2, p3) => {
                    canvas.cubic(t(p0), t(p1), t(p2), t(p3))
                }
            }
        }

        Bitmap {
            left,
            top,
            width,
            height,
            coverage: canvas.accumulate(),
        }
    }
}

/// The result of rasterizing a glyph.
pub struct Bitmap {
    /// Horizontal pixel position (from the left) at which the bitmap should be
    /// placed in the larger raster.
    pub left: i32,
    /// Vertical pixel position (from the top) at which the bitmap should be
    /// placed in the larger raster.
    pub top: i32,
    /// The width of the coverage bitmap in pixels.
    pub width: u32,
    /// The height of the coverage bitmap in pixels.
    pub height: u32,
    /// How much each pixel should be covered, `0` means 0% coverage and `255`
    /// means 100% coverage.
    ///
    /// The length of this vector is `width * height`, with the values being
    /// stored row-by-row.
    pub coverage: Vec<u8>,
}

impl Debug for Bitmap {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Bitmap")
            .field("left", &self.left)
            .field("top", &self.top)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

/// Builds the glyph outline.
#[derive(Default)]
struct Builder {
    segments: Vec<Segment>,
    start: Option<Point>,
    last: Point,
}

impl OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.start = Some(point(x, y));
        self.last = point(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.segments.push(Segment::Line(self.last, point(x, y)));
        self.last = point(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.segments
            .push(Segment::Quad(self.last, point(x1, y1), point(x2, y2)));
        self.last = point(x2, y2);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        self.segments.push(Segment::Cubic(
            self.last,
            point(x1, y1),
            point(x2, y2),
            point(x3, y3),
        ));
        self.last = point(x3, y3);
    }

    fn close(&mut self) {
        if let Some(start) = self.start.take() {
            self.segments.push(Segment::Line(self.last, start));
            self.last = start;
        }
    }
}

// Accumulation, line and quad drawing taken from here:
// https://github.com/raphlinus/font-rs
//
// Copyright 2015 Google Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// The internal rendering buffer.
struct Canvas {
    w: usize,
    h: usize,
    a: Vec<f32>,
}

impl Canvas {
    /// Create a completely uncovered canvas.
    fn new(w: u32, h: u32) -> Self {
        Self {
            w: w as usize,
            h: h as usize,
            a: vec![0.0; (w * h + 4) as usize],
        }
    }

    /// Return the accumulated coverage values.
    fn accumulate(self) -> Vec<u8> {
        let mut acc = 0.0;
        self.a[.. self.w * self.h]
            .iter()
            .map(|c| {
                acc += c;
                (255.0 * acc.abs().min(1.0)) as u8
            })
            .collect()
    }

    /// Add to a value in the accumulation buffer.
    fn add(&mut self, index: usize, delta: f32) {
        if let Some(a) = self.a.get_mut(index) {
            *a += delta;
        }
    }

    /// Draw a straight line.
    fn line(&mut self, p0: Point, p1: Point) {
        if (p0.y - p1.y).abs() <= core::f32::EPSILON {
            return;
        }
        let (dir, p0, p1) = if p0.y < p1.y { (1.0, p0, p1) } else { (-1.0, p1, p0) };
        let dxdy = (p1.x - p0.x) / (p1.y - p0.y);
        let mut x = p0.x;
        let y0 = p0.y as usize;
        if p0.y < 0.0 {
            x -= p0.y * dxdy;
        }
        for y in y0 .. self.h.min(p1.y.ceil() as usize) {
            let linestart = y * self.w;
            let dy = ((y + 1) as f32).min(p1.y) - (y as f32).max(p0.y);
            let xnext = x + dxdy * dy;
            let d = dy * dir;
            let (x0, x1) = if x < xnext { (x, xnext) } else { (xnext, x) };
            let x0floor = x0.floor();
            let x0i = x0floor as i32;
            let x1ceil = x1.ceil();
            let x1i = x1ceil as i32;
            if x1i <= x0i + 1 {
                let xmf = 0.5 * (x + xnext) - x0floor;
                self.add(linestart + x0i as usize, d - d * xmf);
                self.add(linestart + (x0i + 1) as usize, d * xmf);
            } else {
                let s = (x1 - x0).recip();
                let x0f = x0 - x0floor;
                let a0 = 0.5 * s * (1.0 - x0f) * (1.0 - x0f);
                let x1f = x1 - x1ceil + 1.0;
                let am = 0.5 * s * x1f * x1f;
                self.add(linestart + x0i as usize, d * a0);
                if x1i == x0i + 2 {
                    self.add(linestart + (x0i + 1) as usize, d * (1.0 - a0 - am));
                } else {
                    let a1 = s * (1.5 - x0f);
                    self.add(linestart + (x0i + 1) as usize, d * (a1 - a0));
                    for xi in x0i + 2 .. x1i - 1 {
                        self.add(linestart + xi as usize, d * s);
                    }
                    let a2 = a1 + (x1i - x0i - 3) as f32 * s;
                    self.add(linestart + (x1i - 1) as usize, d * (1.0 - a2 - am));
                }
                self.add(linestart + x1i as usize, d * am);
            }
            x = xnext;
        }
    }

    /// Draw a quadratic bezier curve.
    fn quad(&mut self, p0: Point, p1: Point, p2: Point) {
        // How much does the curve deviate from a straight line?
        let devsq = hypot2(p0 - 2.0 * p1 + p2);

        // Check if the curve is already flat enough.
        if devsq < 0.333 {
            self.line(p0, p2);
            return;
        }

        // Estimate the required number of subdivisions for flattening.
        let tol = 3.0;
        let n = 1.0 + (tol * devsq).sqrt().sqrt().floor();
        let nu = n as usize;
        let step = n.recip();

        // Flatten the curve.
        let mut t = 0.0;
        let mut p = p0;
        for _ in 0 .. nu - 1 {
            t += step;

            // Evaluate the curve at `t` using De Casteljau and draw a line from
            // the last point to the new evaluated point.
            let p01 = lerp(t, p0, p1);
            let p12 = lerp(t, p1, p2);
            let pt = lerp(t, p01, p12);
            self.line(p, pt);

            // Then set the evaluated point as the start point of the new line.
            p = pt;
        }

        // Draw a final line.
        self.line(p, p2);
    }
}

// Cubic to quad conversion adapted from here:
// https://github.com/linebender/kurbo/blob/master/src/cubicbez.rs
//
// Copyright 2018 The kurbo Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

impl Canvas {
    /// Draw a cubic bezier curve.
    fn cubic(&mut self, p0: Point, p1: Point, p2: Point, p3: Point) {
        // How much does the curve deviate?
        let p1x2 = 3.0 * p1 - p0;
        let p2x2 = 3.0 * p2 - p3;
        let err = hypot2(p2x2 - p1x2);

        // Estimate the required number of subdivisions for conversion.
        let tol = 0.333;
        let max = 432.0 * tol * tol;
        let n = (err / max).powf(1.0 / 6.0).ceil().max(1.0);
        let nu = n as usize;
        let step = n.recip();
        let step4 = step / 4.0;

        // Compute the derivative of the cubic.
        let dp0 = 3.0 * (p1 - p0);
        let dp1 = 3.0 * (p2 - p1);
        let dp2 = 3.0 * (p3 - p2);

        // Convert the cubics to quadratics.
        let mut t = 0.0;
        let mut p = p0;
        let mut pd = dp0;
        for _ in 0 .. nu {
            t += step;

            // Evaluate the curve at `t` using De Casteljau.
            let p01 = lerp(t, p0, p1);
            let p12 = lerp(t, p1, p2);
            let p23 = lerp(t, p2, p3);
            let p012 = lerp(t, p01, p12);
            let p123 = lerp(t, p12, p23);
            let pt = lerp(t, p012, p123);

            // Evaluate the derivative of the curve at `t` using De Casteljau.
            let dp01 = lerp(t, dp0, dp1);
            let dp12 = lerp(t, dp1, dp2);
            let pdt = lerp(t, dp01, dp12);

            // Determine the control point of the quadratic.
            let pc = (p + pt) / 2.0 + step4 * (pd - pdt);

            // Draw the quadratic.
            self.quad(p, pc, pt);

            p = pt;
            pd = pdt;
        }
    }
}

/// Create a point.
fn point(x: f32, y: f32) -> Point {
    Point { x, y }
}

/// Linearly interpolate between two points.
fn lerp(t: f32, p1: Point, p2: Point) -> Point {
    Point {
        x: p1.x + t * (p2.x - p1.x),
        y: p1.y + t * (p2.y - p1.y),
    }
}

/// The squared distance of the point from the origin.
fn hypot2(p: Point) -> f32 {
    p.x * p.x + p.y * p.y
}

/// A point in 2D.
#[derive(Debug, Default, Copy, Clone)]
struct Point {
    x: f32,
    y: f32,
}

impl Add for Point {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl Sub for Point {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl Mul<Point> for f32 {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        Point { x: self * rhs.x, y: self * rhs.y }
    }
}

impl Div<f32> for Point {
    type Output = Point;

    fn div(self, rhs: f32) -> Self::Output {
        Point { x: self.x / rhs, y: self.y / rhs }
    }
}
