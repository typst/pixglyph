use std::io::Write;

use pixglyph::Glyph;
use skrifa::instance::{Location, LocationRef};
use skrifa::raw::TableProvider;
use skrifa::{FontRef, GlyphId, MetadataProvider};

const ROBOTO: &[u8] = include_bytes!("../fonts/Roboto-Regular.ttf");
const SOURCE_SANS: &[u8] = include_bytes!("../fonts/SourceSans3-Regular.otf");
const IBM_PLEX: &[u8] = include_bytes!("../fonts/IBMPlexSans-Bold.ttf");
const LIBERTINUS: &[u8] = include_bytes!("../fonts/LibertinusSerif-Regular.otf");
const ASANA_MATH_SUBSET: &[u8] = include_bytes!("../fonts/Asana-Math-Subset.otf");
const AOTS: &[u8] = include_bytes!("../fonts/gpos1_2_font1.otf");
const ROBOTO_VF: &[u8] = include_bytes!("../fonts/Roboto-VariableFont_wdth,wght.ttf");
const NOTO_SERIF_CJK_SUBSET: &[u8] =
    include_bytes!("../fonts/NotoSerifCJKsc-VF-Subset.otf");

#[test]
fn test_load_all() {
    let font = FontRef::new(SOURCE_SANS).unwrap();
    for i in 0..font.maxp().unwrap().num_glyphs() {
        Glyph::load(&font, LocationRef::default(), GlyphId::new(i as u32));
    }
}

#[test]
fn test_rasterize() {
    let mut ok = true;
    ok &= raster_letter(ROBOTO, None, 'A', 0.0, 0.0, 100.0);
    ok &= raster_letter(ROBOTO, None, '≤', 0.0, 0.0, 25.0);
    ok &= raster_letter(ROBOTO, None, 'χ', 0.0, 0.0, 33.0);
    ok &= raster_letter(ROBOTO, None, '≥', 0.0, 0.0, 128.0);
    ok &= raster_letter(SOURCE_SANS, None, 'g', 0.0, 0.0, 400.0);
    ok &= raster_letter(IBM_PLEX, None, 'l', 138.48, 95.84, 80.0);
    ok &= raster_letter(LIBERTINUS, None, '(', 114.09056, 34.47, 22.0);
    ok &= raster_letter(ASANA_MATH_SUBSET, None, 'µ', 0.0, 0.0, 40.0);
    ok &= raster_letter(AOTS, None, '1', 0.0, 0.0, 50.0);
    ok &= raster_letter(NOTO_SERIF_CJK_SUBSET, None, 'J', 0.0, 0.0, 30.0);
    ok &= raster_letter(ROBOTO_VF, Some(("wght", 550.0)), 'p', 0.0, 0.0, 60.0);
    ok &= raster_letter(ROBOTO_VF, Some(("wdth", 90.0)), 'Ç', 0.0, 0.0, 47.5);
    ok &=
        raster_letter(NOTO_SERIF_CJK_SUBSET, Some(("wght", 400.0)), 'K', 0.0, 0.0, 50.0);
    if !ok {
        panic!();
    }
}

fn raster_letter(
    data: &[u8],
    variation: Option<(&str, f32)>,
    letter: char,
    x: f32,
    y: f32,
    s: f32,
) -> bool {
    let out_path = format!("target/{}.ppm", letter);
    let ref_path = format!("tests/{}.ppm", letter);

    let font = FontRef::new(data).unwrap();
    let location = if let Some(variation) = variation {
        font.axes().location([variation])
    } else {
        Location::default()
    };
    let id = font.charmap().map(letter).unwrap();
    let glyph = Glyph::load(&font, &location, id).unwrap();
    let bitmap = glyph.rasterize(x, y, s);

    let mut ppm = vec![];
    write!(
        ppm,
        "P6\n# left {} top {}\n{} {}\n255\n",
        bitmap.left, bitmap.top, bitmap.width, bitmap.height
    )
    .unwrap();

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
