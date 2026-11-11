//! The biome noise cases: the precipitation and the grass colour a biome gives at a block.

use serde_json::{Value, json};

use crate::case::case;

/// Where 26.2's double noise and 26.3's float noise decide a threshold the other way, found by
/// searching the two evaluations over the world (the frozen patches at 726,4518; the swamp's
/// ground at three columns). The rows around them are ordinary positions that agree.
const FROZEN_FLIP: (i32, i32) = (726, 4518);
const SWAMP_FLIPS: [(i32, i32); 3] = [(14547, -15489), (-1002, 38972), (-16849, -8459)];

fn row(label: String, biome: &str, (x, y, z): (i32, i32, i32)) -> Value {
    json!({
        "label": label, "blocks": [], "chunk_radius": 0, "biome": biome,
        "x": x, "y": y, "z": z, "sea_level": 63,
        // The swamp biomes give a colour of their own; the others read a colour map the oracle
        // does not load.
        "grass": biome.ends_with("swamp"),
    })
}

/// The precipitation and the grass colour of a spread of biomes and columns.
pub fn biome_noise() -> Value {
    let mut rows = Vec::new();
    for biome in ["plains", "snowy_plains", "desert", "swamp", "mangrove_swamp", "dark_forest"] {
        for (x, z) in [(0, 0), (100, -250), (-3000, 700), (1234, 5678)] {
            rows.push(row(format!("{biome}-{x}-{z}"), &format!("minecraft:{biome}"), (x, 64, z)));
        }
    }
    // The frozen biomes lay cold patches over themselves.
    for biome in ["frozen_ocean", "deep_frozen_ocean"] {
        let (flip_x, flip_z) = FROZEN_FLIP;
        rows.push(row(format!("{biome}-flip"), &format!("minecraft:{biome}"), (flip_x, 64, flip_z)));
        for step in 0..24 {
            let (x, z) = (step * 37 - 400, step * 53 + 200);
            rows.push(row(format!("{biome}-{x}-{z}"), &format!("minecraft:{biome}"), (x, 64, z)));
        }
    }
    for (x, z) in SWAMP_FLIPS {
        rows.push(row(format!("swamp-flip-{x}-{z}"), "minecraft:swamp", (x, 64, z)));
        rows.push(row(format!("mangrove-flip-{x}-{z}"), "minecraft:mangrove_swamp", (x, 64, z)));
    }
    // Above the snow line the air cools by the temperature noise: a tall column of plains.
    for step in 0..24 {
        let (x, z) = (step * 41 - 500, 900 - step * 29);
        for y in [300, 560, 640] {
            rows.push(row(format!("plains-high-{y}-{x}-{z}"), "minecraft:plains", (x, y, z)));
        }
    }
    case("movement/biome-noise", "movement", "biome_noise", json!({"rows": rows}))
}
