//! The `image` family: PNG decoding, mipmaps, sprite placement, tints and
//! armor trim pictures, all from synthetic images.

mod decode;
mod mips;
mod png;
mod samples;
mod tables;
mod trim;

use serde_json::{Value, json};

use crate::case::case;

const ITEM_TEXTURES: &[&str] = &[
    "diamond_sword",
    "stick",
    "apple",
    "bow",
    "fishing_rod",
    "glass_bottle",
    "leather_helmet",
    "leather_helmet_overlay",
    "potion_overlay",
    "bucket",
    "iron_ingot",
    "coal",
];

fn item_side_faces() -> Value {
    let rows: Vec<Value> = ITEM_TEXTURES
        .iter()
        .map(|name| json!({"label": name, "texture": format!("assets/minecraft/textures/item/{name}.png")}))
        .collect();
    case("image/item-side-faces", "image", "item_side_faces", json!({ "rows": rows }))
}

fn single(label: &str, operation: &str, row: &str) -> Value {
    case(format!("image/{label}"), "image", operation, json!({"rows": [{"label": row}]}))
}

pub fn cases() -> Vec<Value> {
    vec![
        item_side_faces(),
        single("srgb-tables", "srgb_tables", "tables"),
        mips::mip_levels(),
        mips::compute_transparency(),
        tables::texture_metadata(),
        tables::sprite_uv(),
        tables::colour_map(),
        tables::argb_multiply(),
        single("tint-constants", "tint_constants", "constants"),
        decode::png_decode(),
        trim::palette_permutation(),
        trim::trim_sprite_id(),
    ]
}
