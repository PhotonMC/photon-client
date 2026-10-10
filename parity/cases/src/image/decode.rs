//! `NativeImage.read`: every PNG colour type reduced to RGBA8.
//!
//! The reference hands the bytes to stb with four components requested, so
//! grayscale, palette and RGB all come back as RGBA and 16-bit samples are
//! cut to their high byte. The inputs cover each colour type, both bit
//! depths, all five row filters and the transparency chunk.

use serde_json::{Value, json};

use super::png::Png;
use super::samples::{self, gray, gray_alpha, indexed, packed, rgb, rgba, rgba16};
use crate::bits::hex;

const OPAQUE_INDEXES: &[u8] = &[0, 80, 170, 255];
const SOME: &[u8] = &[0, 1, 2, 4];

/// One decoded image: its rows and how they are stored.
struct Decode {
    label: &'static str,
    width: u32,
    colour_type: u8,
    bit_depth: u8,
    rows: Vec<Vec<u8>>,
    paletted: bool,
    transparency: Option<&'static [u8]>,
    filters: &'static [u8],
}

fn image(
    label: &'static str,
    width: u32,
    (colour_type, bit_depth): (u8, u8),
    rows: Vec<Vec<u8>>,
    filters: &'static [u8],
) -> Decode {
    Decode { label, width, colour_type, bit_depth, rows, paletted: colour_type == 3, transparency: None, filters }
}

impl Decode {
    fn keyed(self, transparency: &'static [u8]) -> Self {
        Self { transparency: Some(transparency), ..self }
    }
}

const RGBA8: (u8, u8) = (6, 8);

fn direct_rows() -> Vec<Decode> {
    vec![
        image("rgba8-filter-none", 4, RGBA8, rgba(4, 4), &[0, 0, 0, 0]),
        image("rgba8-filter-sub", 4, RGBA8, rgba(4, 4), &[1, 1, 1, 1]),
        image("rgba8-filter-up", 4, RGBA8, rgba(4, 4), &[0, 2, 2, 2]),
        image("rgba8-filter-average", 4, RGBA8, rgba(4, 4), &[0, 3, 3, 3]),
        image("rgba8-filter-paeth", 4, RGBA8, rgba(4, 4), &[0, 4, 4, 4]),
        image("rgba8-mixed-filters", 6, RGBA8, rgba(6, 5), &[0, 1, 2, 3, 4]),
        image("rgb8", 4, (2, 8), rgb(4, 4), SOME),
        image("gray8", 4, (0, 8), gray(4, 4), SOME),
        image("gray-alpha8", 4, (4, 8), gray_alpha(4, 4), SOME),
        image("indexed", 4, (3, 8), indexed(4, 4, 4), SOME),
        image("indexed-trns", 4, (3, 8), indexed(4, 4, 4), &[0; 4]).keyed(OPAQUE_INDEXES),
    ]
}

fn packed_rows() -> Vec<Decode> {
    let sum2 = |x: i64, y: i64| (x + y) % 2;
    let sum4 = |x: i64, y: i64| (x + y) % 4;
    vec![
        image("gray1", 8, (0, 1), packed(8, 4, 1, sum2), SOME),
        image("gray2", 8, (0, 2), packed(8, 4, 2, sum4), SOME),
        image("gray4", 8, (0, 4), packed(8, 4, 4, |x, y| (x * 3 + y) % 16), SOME),
        image("indexed1", 8, (3, 1), packed(8, 4, 1, sum2), SOME),
        image("indexed2", 8, (3, 2), packed(8, 4, 2, sum4), SOME),
        image("indexed4", 8, (3, 4), packed(8, 4, 4, sum4), SOME),
        image("indexed4-trns", 8, (3, 4), packed(8, 4, 4, sum4), SOME).keyed(OPAQUE_INDEXES),
        image("gray1-odd-width", 5, (0, 1), packed(5, 3, 1, |x, y| (x * y) % 2), &[0, 1, 4]),
        image("indexed2-odd-width", 3, (3, 2), packed(3, 3, 2, |x, y| (x + 2 * y) % 4), &[0, 1, 4]),
    ]
}

fn keyed_rows() -> Vec<Decode> {
    let hit = vec![vec![0, 37, 74, 111]; 4];
    vec![
        image("gray8-trns", 4, (0, 8), gray(4, 4), SOME).keyed(&[0, 0]),
        image("gray8-trns-hit", 4, (0, 8), hit, &[0; 4]).keyed(&[0, 0]),
        image("gray2-trns", 8, (0, 2), packed(8, 4, 2, |x, y| (x + y) % 4), SOME).keyed(&[0, 0]),
        image("gray1-trns", 8, (0, 1), packed(8, 4, 1, |x, y| (x + y) % 2), SOME).keyed(&[0, 1]),
        image("rgb8-trns", 4, (2, 8), rgb(4, 4), SOME).keyed(&[0; 6]),
        image("rgba16", 3, (6, 16), rgba16(3, 3), &[0, 1, 4]),
        image("single-pixel", 1, RGBA8, rgba(1, 1), &[0]),
        image("tall", 2, RGBA8, rgba(2, 8), &[0, 1, 2, 3, 4, 1, 2, 3]),
        image("wide", 8, RGBA8, rgba(8, 2), &[4, 3]),
    ]
}

pub fn png_decode() -> Value {
    let palette = samples::palette();
    let rows: Vec<Value> = [direct_rows(), packed_rows(), keyed_rows()]
        .into_iter()
        .flatten()
        .map(|image| {
            let png = Png {
                width: image.width,
                colour_type: image.colour_type,
                bit_depth: image.bit_depth,
                rows: &image.rows,
                palette: image.paletted.then_some(palette.as_slice()),
                transparency: image.transparency,
                filters: image.filters,
            };
            json!({"label": image.label, "png": hex(&png.encode())})
        })
        .collect();
    crate::case::case("image/png-decode", "image", "png_decode", json!({ "rows": rows }))
}
