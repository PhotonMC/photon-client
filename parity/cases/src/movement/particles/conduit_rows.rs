//! The particle engine's rows of a conduit: its shell worked out every two seconds from the water
//! and the frame around it, its count of ticks and turns, and the specks its frame sends to it.

use serde_json::{Value, json};

use super::engine::*;
use crate::bits::f64_bits;
use crate::movement::world::{put, put_with};

/// The places of a conduit's frame, as offsets from it: two blocks out along some axis and in line
/// with the conduit along another.
fn frame() -> Vec<(i64, i64, i64)> {
    let range = -2..=2_i64;
    let cube = range.clone().flat_map(|x| range.clone().flat_map(move |y| (-2..=2_i64).map(move |z| (x, y, z))));
    cube.filter(|(x, y, z)| [x, y, z].contains(&&0) && [x, y, z].iter().any(|n| n.abs() == 2)).collect()
}

/// The conduits of a row: each wholly in water, with some of its frame, to try one thing.
fn columns() -> (Vec<Value>, Vec<Value>) {
    let mut blocks: Vec<Value> = Vec::new();
    let mut places = Vec::new();
    let kinds = ["prismarine", "prismarine_bricks", "sea_lantern", "dark_prismarine"];
    let mut conduit = |x: i64, water: bool, frame_blocks: usize, spoil: Option<&str>| {
        if water {
            for index in 0..27 {
                let (dx, dy, dz) = (index % 3 - 1, index / 3 % 3 - 1, index / 9 - 1);
                blocks.push(put(x + dx, 64 + dy, dz, "minecraft:water"));
            }
        }
        for (index, (dx, dy, dz)) in frame().into_iter().take(frame_blocks).enumerate() {
            let name = format!("minecraft:{}", kinds[index % kinds.len()]);
            blocks.push(put(x + dx, 64 + dy, dz, &name));
        }
        match spoil {
            Some("air") => blocks.push(put(x + 1, 64, 0, "minecraft:air")),
            Some("stone") => blocks.push(put(x + 2, 64, 0, "minecraft:stone")),
            _ => {}
        }
        blocks.push(put_with(x, 64, 0, "minecraft:conduit", &[("waterlogged", "true")]));
        places.push(json!({"x": x, "y": 64, "z": 0, "type": "minecraft:conduit"}));
    };
    conduit(-12, true, 42, None);
    conduit(-5, true, 20, None);
    conduit(2, true, 15, None);
    conduit(9, true, 42, Some("air"));
    conduit(16, false, 0, None);
    conduit(23, true, 42, Some("stone"));

    (blocks, places)
}

fn conduit_row(label: &str, game_time: u64) -> Value {
    let (blocks, places) = columns();
    let watch: Vec<Value> = places.iter().map(|place| json!({"x": place["x"], "y": place["y"], "z": 0})).collect();
    engine_row(label, vec![])
        .blocks(blocks)
        .set("block_entities", json!(places))
        .set("watch", json!(watch))
        // Two players, and the conduits told to attack them: a hunting one, an idle one with a frame
        // too small, and one told to attack a player the level does not have.
        .set(
            "players",
            json!([
                {"uuid": [1, 2, 3, 4], "x": f64_bits(-11.5), "y": f64_bits(66.0), "z": f64_bits(3.5)},
                {"uuid": [5, 6, 7, 8], "x": f64_bits(23.5), "y": f64_bits(66.0), "z": f64_bits(4.5)},
            ]),
        )
        .set(
            "conduit_targets",
            json!([
                {"x": -12, "y": 64, "z": 0, "uuid": [1, 2, 3, 4]},
                {"x": 23, "y": 64, "z": 0, "uuid": [5, 6, 7, 8]},
                {"x": 2, "y": 64, "z": 0, "uuid": [9, 9, 9, 9]},
            ]),
        )
        .set("game_time", json!(game_time))
        .set("tick_block_entities", json!(true))
        .ticks(100)
        .seeds(1500)
        .describe(6)
        .row()
}

pub fn rows() -> Vec<Value> {
    vec![conduit_row("conduit-shells", 160), conduit_row("conduit-shells-off-beat", 161)]
}
