use std::io::Write;

use pixglyph::Glyph;
use skrifa::{FontRef, MetadataProvider};

const ROBOTO: &[u8] = include_bytes!("../fonts/Roboto-Regular.ttf");
const SOURCE_SANS: &[u8] = include_bytes!("../fonts/SourceSans3-Regular.otf");
const IBM_PLEX: &[u8] = include_bytes!("../fonts/IBMPlexSans-Bold.ttf");
const LIBERTINUS: &[u8] = include_bytes!("../fonts/LibertinusSerif-Regular.otf");

#[test]
fn test_load_all() {
    let font = FontRef::new(SOURCE_SANS).unwrap();
    font.charmap().mappings().for_each(|(_, id)| {
        Glyph::load(&font, id);
    })
}

#[test]
fn test_rasterize() {
    let mut ok = true;
    ok &= raster_letter(ROBOTO, 'A', 0.0, 0.0, 100.0);
    ok &= raster_letter(SOURCE_SANS, 'g', 0.0, 0.0, 400.0);
    ok &= raster_letter(IBM_PLEX, 'l', 138.48, 95.84, 80.0);
    ok &= raster_letter(LIBERTINUS, '(', 114.09056, 34.47, 22.0);
    if !ok {
        panic!();
    }
}

fn raster_letter(data: &[u8], letter: char, x: f32, y: f32, s: f32) -> bool {
    let out_path = format!("target/{}.ppm", letter);
    let ref_path = format!("tests/{}.ppm", letter);

    let font = FontRef::new(data).unwrap();
    let id = font.charmap().map(letter).unwrap();
    let glyph = Glyph::load(&font, id).unwrap();
    let bitmap = glyph.rasterize(x, y, s);

    let mut ppm = vec![];
    write!(ppm, "P6\n{} {}\n255\n", bitmap.width, bitmap.height).unwrap();

    for &c in &bitmap.coverage {
        ppm.extend([255 - c; 3]);
    }

    std::fs::write(out_path, &ppm).unwrap();

    let reference = std::fs::read(ref_path).ok();

    let ok = Some(ppm) == reference;
    if !ok {
        eprintln!("Letter {letter:?} differs ❌");
    }

    ok
}
