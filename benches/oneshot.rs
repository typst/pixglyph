use iai::{main, Iai};

use pixglyph::Glyph;
use skrifa::{FontRef, MetadataProvider};

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
    let font = FontRef::new(ROBOTO).unwrap();
    let id = font.charmap().map('A').unwrap();
    iai.run(|| Glyph::load(&font, id));
}

fn bench_load_complex(iai: &mut Iai) {
    let font = FontRef::new(ROBOTO).unwrap();
    let id = font.charmap().map('g').unwrap();
    iai.run(|| Glyph::load(&font, id));
}

fn bench_rasterize_simple(iai: &mut Iai) {
    let font = FontRef::new(ROBOTO).unwrap();
    let id = font.charmap().map('A').unwrap();
    let glyph = Glyph::load(&font, id).unwrap();
    iai.run(|| glyph.rasterize(0.0, 0.0, SIZE));
}

fn bench_rasterize_complex(iai: &mut Iai) {
    let font = FontRef::new(ROBOTO).unwrap();
    let id = font.charmap().map('g').unwrap();
    let glyph = Glyph::load(&font, id).unwrap();
    iai.run(|| glyph.rasterize(0.0, 0.0, SIZE));
}

fn bench_rasterize_cubic(iai: &mut Iai) {
    let font = FontRef::new(SOURCE_SANS).unwrap();
    let id = font.charmap().map('g').unwrap();
    let glyph = Glyph::load(&font, id).unwrap();
    iai.run(|| glyph.rasterize(0.0, 0.0, SIZE));
}
