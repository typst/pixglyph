use iai::{main, Iai};

use pixglyph::Glyph;
use ttf_parser::Face;

const ROBOTO: &[u8] = include_bytes!("../fonts/Roboto-Regular.ttf");
const SOURCE_SANS: &[u8] = include_bytes!("../fonts/SourceSans3-Regular.otf");
const SIZE: f32 = 14.0;

main!(
    bench_load_simple,
    bench_load_complex,
    bench_rasterize_simple,
    bench_rasterize_complex,
    bench_rasterize_cubic,
);

fn bench_load_simple(iai: &mut Iai) {
    let face = Face::parse(ROBOTO, 0).unwrap();
    let id = face.glyph_index('A').unwrap();
    iai.run(|| Glyph::load(&face, id));
}

fn bench_load_complex(iai: &mut Iai) {
    let face = Face::parse(ROBOTO, 0).unwrap();
    let id = face.glyph_index('g').unwrap();
    iai.run(|| Glyph::load(&face, id));
}

fn bench_rasterize_simple(iai: &mut Iai) {
    let face = Face::parse(ROBOTO, 0).unwrap();
    let id = face.glyph_index('A').unwrap();
    let glyph = Glyph::load(&face, id).unwrap();
    iai.run(|| glyph.rasterize(0.0, 0.0, SIZE));
}

fn bench_rasterize_complex(iai: &mut Iai) {
    let face = Face::parse(ROBOTO, 0).unwrap();
    let id = face.glyph_index('g').unwrap();
    let glyph = Glyph::load(&face, id).unwrap();
    iai.run(|| glyph.rasterize(0.0, 0.0, SIZE));
}

fn bench_rasterize_cubic(iai: &mut Iai) {
    let face = Face::parse(SOURCE_SANS, 0).unwrap();
    let id = face.glyph_index('g').unwrap();
    let glyph = Glyph::load(&face, id).unwrap();
    iai.run(|| glyph.rasterize(0.0, 0.0, SIZE));
}
