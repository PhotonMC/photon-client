//! The pictures armor trims are drawn with: a synthetic picture recoloured
//! by a palette (a colour of the key palette replaced by the permutation's,
//! its alpha scaled by the replacement's; other opaque colours kept; clear
//! pixels kept), by the real mapping; and the name of the picture each
//! pattern in each material takes on each armor's layer.

use serde_json::{Value, json};

use super::png;
use crate::case::case;

type Pixel = [u8; 4];

const KEYS: [Pixel; 6] = [
    [200, 10, 10, 255],
    [10, 200, 10, 255],
    [10, 10, 200, 255],
    [90, 90, 90, 255],
    [250, 250, 250, 255],
    [1, 2, 3, 0],
];
const VALUES: [Pixel; 6] = [
    [255, 0, 0, 255],
    [0, 255, 0, 128],
    [0, 0, 255, 255],
    [12, 34, 56, 200],
    [255, 255, 255, 0],
    [9, 9, 9, 255],
];
const BASE: [Pixel; 16] = [
    KEYS[0],
    KEYS[1],
    KEYS[2],
    KEYS[3],
    KEYS[4],
    [1, 2, 3, 255],
    [77, 77, 77, 255],
    [0, 0, 0, 0],
    [200, 10, 10, 100],
    [10, 200, 10, 255],
    [10, 10, 200, 1],
    [90, 90, 90, 254],
    [250, 250, 250, 255],
    [5, 5, 5, 255],
    [0, 0, 0, 0],
    [200, 10, 10, 255],
];
const OTHER: [Pixel; 8] = [
    [0, 0, 0, 255],
    [255, 128, 0, 255],
    [10, 200, 10, 255],
    [7, 7, 7, 40],
    [200, 10, 10, 255],
    [90, 90, 90, 90],
    [1, 1, 1, 255],
    [2, 2, 2, 255],
];

const PATTERNS: &[&str] = &[
    "sentry",
    "dune",
    "coast",
    "wild",
    "ward",
    "eye",
    "vex",
    "tide",
    "snout",
    "rib",
    "spire",
    "wayfinder",
    "shaper",
    "silence",
    "raiser",
    "host",
    "flow",
    "bolt",
];
const MATERIALS: &[&str] = &[
    "quartz",
    "iron",
    "netherite",
    "redstone",
    "copper",
    "gold",
    "emerald",
    "diamond",
    "lapis",
    "amethyst",
    "resin",
];
const EQUIPMENT: &[&str] = &[
    "iron",
    "diamond",
    "gold",
    "netherite",
    "copper",
    "chainmail",
    "leather",
    "turtle_scute",
];

fn picture(pixels: &[Pixel], width: usize) -> String {
    let rows: Vec<Vec<u8>> = pixels.chunks(width).map(|row| row.concat()).collect();
    png::rgba(width as u32, &rows)
}

pub fn palette_permutation() -> Value {
    let (keys, values) = (picture(&KEYS, 6), picture(&VALUES, 6));
    let rows = json!([
        {"label": "mixed", "png": picture(&BASE, 4), "keys": keys, "values": values},
        {"label": "other-pixels", "png": picture(&OTHER, 4), "keys": keys, "values": values},
        {
            "label": "one-colour", "png": picture(&KEYS[..1], 1),
            "keys": picture(&KEYS[..1], 1), "values": picture(&VALUES[..1], 1),
        },
    ]);
    case(
        "image/palette-permutation",
        "image",
        "palette_permutation",
        json!({ "rows": rows }),
    )
}

pub fn trim_sprite_id() -> Value {
    let mut rows = Vec::new();
    for (index, pattern) in PATTERNS.iter().enumerate() {
        let layer = ["humanoid", "humanoid_leggings"][index % 2];
        let equipment = EQUIPMENT[index % EQUIPMENT.len()];
        for material in MATERIALS {
            rows.push(json!({
                "label": format!("{pattern}-{material}-{layer}"),
                "pattern": format!("minecraft:{pattern}"), "material": format!("minecraft:{material}"),
                "equipment": format!("minecraft:{equipment}"), "layer": layer,
            }));
        }
    }
    for asset in EQUIPMENT {
        for material in ["iron", "gold", "copper", "diamond", "netherite", "quartz"] {
            rows.push(json!({
                "label": format!("over-{asset}-{material}"), "pattern": "minecraft:sentry",
                "material": format!("minecraft:{material}"), "equipment": format!("minecraft:{asset}"),
                "layer": "humanoid",
            }));
        }
    }
    case(
        "image/trim-sprite-id",
        "image",
        "trim_sprite_id",
        json!({ "rows": rows }),
    )
}
