# pixglyph
[![Crates.io](https://img.shields.io/crates/v/pixglyph.svg)](https://crates.io/crates/pixglyph)
[![Documentation](https://docs.rs/pixglyph/badge.svg)](https://docs.rs/pixglyph)

OpenType glyph rendering.

```toml
[dependencies]
pixglyph = "0.3"
```

## Features
- Render glyph outlines into coverage bitmaps.
- Place glyphs at subpixel offsets and scale them to subpixel sizes. This is
  important if you plan to render more than a single glyph since inter-glyph
  spacing will look off if every glyph origin must be pixel-aligned.
- No font data structure you have to store somewhere. Just owned glyphs
  which you can load individually from a font, cache if you care about
  performance, and then render at any size.
- No unsafe code.

## License
This crate is licensed under the terms of the Apache 2.0 license.
