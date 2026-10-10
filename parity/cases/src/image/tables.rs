//! Sprite metadata, UVs, biome colour maps and tint arithmetic.

use serde_json::{Value, json};

use super::png;
use crate::bits::{f32_bits, f64_bits};
use crate::case::case;

/// The texture section of a sprite's .mcmeta. Every field is optional, so
/// what an absent field falls back to is the substance here.
const METADATA: &[(&str, &str)] = &[
    ("empty", "{}"),
    ("blur", r#"{"blur": true}"#),
    ("clamp", r#"{"clamp": true}"#),
    ("strategy-mean", r#"{"mipmap_strategy": "mean"}"#),
    ("strategy-cutout", r#"{"mipmap_strategy": "cutout"}"#),
    ("strategy-strict", r#"{"mipmap_strategy": "strict_cutout"}"#),
    ("strategy-dark", r#"{"mipmap_strategy": "dark_cutout"}"#),
    ("strategy-auto", r#"{"mipmap_strategy": "auto"}"#),
    ("bias", r#"{"alpha_cutoff_bias": 0.25}"#),
    ("bias-negative", r#"{"alpha_cutoff_bias": -0.5}"#),
    ("bias-integer", r#"{"alpha_cutoff_bias": 1}"#),
    (
        "all",
        r#"{"blur": true, "clamp": true, "mipmap_strategy": "dark_cutout", "alpha_cutoff_bias": 0.125}"#,
    ),
    // Names the enum does not carry, and shapes the codec does not take.
    ("strategy-unknown", r#"{"mipmap_strategy": "smooth"}"#),
    ("strategy-uppercase", r#"{"mipmap_strategy": "MEAN"}"#),
    ("strategy-number", r#"{"mipmap_strategy": 1}"#),
    ("blur-string", r#"{"blur": "true"}"#),
    ("bias-string", r#"{"alpha_cutoff_bias": "0.25"}"#),
    ("unknown-field", r#"{"unexpected": 1}"#),
    ("array", "[]"),
    ("string", r#""texture""#),
];

/// Label, sprite size, atlas size, slot origin and padding.
type SpriteUv = (&'static str, u32, u32, u32, u32, u32, u32, u32);

/// Where a stitched sprite's texels land. The packer hands back the padded
/// slot's origin, not the sprite's own, so the padding has to be added back
/// before anything is written or any UV is taken.
const SPRITE_UVS: &[SpriteUv] = &[
    ("origin", 16, 16, 256, 256, 0, 0, 0),
    ("padded", 16, 16, 256, 256, 0, 0, 1),
    ("offset", 16, 16, 256, 256, 18, 36, 1),
    ("mip-padding", 16, 16, 256, 256, 32, 64, 4),
    ("anisotropic-padding", 16, 16, 512, 512, 64, 128, 16),
    ("oblong", 8, 32, 128, 512, 10, 40, 2),
    // An atlas whose side is not a power of two still divides exactly.
    ("odd-atlas", 16, 16, 300, 200, 20, 30, 1),
];

const COLOUR_MAP: &[(&str, &str, f64, f64)] = &[
    ("grass/default", "grass", 0.5, 1.0),
    ("grass/cold-wet", "grass", 0.0, 1.0),
    ("grass/hot-dry", "grass", 1.0, 0.0),
    ("grass/hot-wet", "grass", 1.0, 1.0),
    ("grass/mid", "grass", 0.8, 0.4),
    ("foliage/default", "foliage", 0.5, 1.0),
    ("foliage/plains", "foliage", 0.8, 0.4),
    ("dry-foliage/default", "dry_foliage", 0.5, 1.0),
    // Rain is scaled by temperature first, so the same rain lands elsewhere
    // at a different temperature.
    ("grass/rain-scaling", "grass", 0.25, 1.0),
    // The truncation is toward zero, so a fraction of a step is dropped.
    ("grass/just-under-a-step", "grass", 0.5 + 1.0 / 512.0, 1.0),
    // The lookup guards the upper bound but not the lower one.
    ("grass/above-one", "grass", 1.5, 1.0),
    ("grass/below-zero", "grass", -0.5, 1.0),
    ("grass/negative-rain", "grass", 0.5, -1.0),
];

/// How a tint folds into a quad's colour: per channel, with an integer
/// division, and with white on either side short-circuiting.
const MULTIPLY: &[(&str, &str, &str)] = &[
    ("white-left", "ffffffff", "ff48b518"),
    ("white-right", "ff48b518", "ffffffff"),
    ("both-white", "ffffffff", "ffffffff"),
    ("foliage-on-grey", "ff48b518", "ff808080"),
    ("halves", "80808080", "80808080"),
    ("black", "ff000000", "ff48b518"),
    ("one-each", "01010101", "01010101"),
    // The top byte is unsigned, so a full alpha survives the divide.
    ("full-alpha", "ff010203", "ff040506"),
    ("max", "fefefefe", "fefefefe"),
];

pub fn texture_metadata() -> Value {
    let rows: Vec<Value> = METADATA
        .iter()
        .map(|(label, text)| json!({"label": label, "json": text}))
        .collect();
    case(
        "image/texture-metadata",
        "image",
        "texture_metadata",
        json!({ "rows": rows }),
    )
}

pub fn sprite_uv() -> Value {
    let offsets: Vec<String> = [0.0, 0.25, 0.5, 1.0, -0.5, 1.5]
        .into_iter()
        .map(f32_bits)
        .collect();
    let rows: Vec<Value> = SPRITE_UVS
        .iter()
        .map(
            |(label, width, height, atlas_width, atlas_height, x, y, padding)| {
                json!({
                    "label": label, "width": width, "height": height,
                    "atlas_width": atlas_width, "atlas_height": atlas_height,
                    "x": x, "y": y, "padding": padding, "offsets": offsets,
                })
            },
        )
        .collect();
    case(
        "image/sprite-uv",
        "image",
        "sprite_uv",
        json!({ "rows": rows }),
    )
}

/// The biome tints index a 256x256 colour map by temperature and rainfall.
/// A synthetic map keeps the case self-contained; what is being pinned is
/// the indexing, not the game's own artwork.
pub fn colour_map() -> Value {
    let texels: Vec<Vec<u8>> = (0..=255u8)
        .map(|y| {
            (0..=255u8)
                .flat_map(|x| [x, y, x.wrapping_mul(y), 255])
                .collect()
        })
        .collect();
    let rows: Vec<Value> = COLOUR_MAP
        .iter()
        .map(|(label, kind, temperature, rain)| {
            json!({
                "label": label, "kind": kind,
                "temperature": f64_bits(*temperature), "rain": f64_bits(*rain),
            })
        })
        .collect();
    let fields = json!({"colour_map": png::rgba(256, &texels), "rows": rows});
    case("image/colour-map", "image", "colour_map", fields)
}

pub fn argb_multiply() -> Value {
    let rows: Vec<Value> = MULTIPLY
        .iter()
        .map(|(label, lhs, rhs)| json!({"label": label, "lhs": lhs, "rhs": rhs}))
        .collect();
    case(
        "image/argb-multiply",
        "image",
        "argb_multiply",
        json!({ "rows": rows }),
    )
}
