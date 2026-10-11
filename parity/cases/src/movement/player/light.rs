//! Light: the client level lights itself from nothing, then relights as
//! block updates arrive. Each row reports a box's block and sky light after
//! the start and after every update.

use serde_json::{Value, json};

use crate::case::case;
use crate::movement::world::{cat, put, put_with, slab};

/// One row: the blocks, the updates and the box reported.
fn lit(label: &str, blocks: Vec<Value>, events: Vec<Value>, size: [i64; 6]) -> Value {
    json!({
        "label": label, "blocks": blocks, "chunk_radius": 1, "events": events,
        "box": {"x": size[0], "y": size[1], "z": size[2], "size_x": size[3], "size_y": size[4], "size_z": size[5]},
    })
}

fn ground() -> Vec<Value> {
    slab((-8, 8), (-8, 8), (61, 63))
}

/// Digging a shaft to the void, and a closed room with glowstone in it.
fn room_rows() -> Vec<Value> {
    let walls: Vec<Value> = slab((-3, 3), (-3, 3), (64, 67))
        .into_iter()
        .filter(|block| {
            let get = |axis: &str| block[axis].as_i64().unwrap_or(0);
            get("x").abs() == 3 || get("z").abs() == 3 || get("y") == 67
        })
        .collect();
    vec![
        lit(
            "dig-to-the-void",
            ground(),
            vec![
                put(0, 63, 0, "minecraft:air"),
                put(0, 62, 0, "minecraft:air"),
                put(0, 61, 0, "minecraft:air"),
                put(0, 61, 0, "minecraft:stone"),
            ],
            [-3, 40, -3, 7, 28, 7],
        ),
        lit(
            "closed-room-glowstone",
            cat![ground(), walls],
            vec![
                put(0, 64, 0, "minecraft:glowstone"),
                put(0, 64, 0, "minecraft:air"),
                put(0, 67, 0, "minecraft:air"),
                put(0, 67, 0, "minecraft:stone"),
                put(2, 65, 2, "minecraft:glowstone"),
            ],
            [-4, 60, -4, 9, 10, 9],
        ),
    ]
}

/// Blocks of each material under the open sky, and a chunk edge.
fn open_rows() -> Vec<Value> {
    vec![
        lit(
            "materials-under-sky",
            ground(),
            vec![
                put(0, 64, 0, "minecraft:torch"),
                put_with(2, 64, 0, "minecraft:oak_slab", &[("type", "bottom")]),
                put_with(2, 65, 0, "minecraft:oak_slab", &[("type", "top")]),
                put_with(4, 64, 0, "minecraft:oak_leaves", &[("persistent", "true")]),
                put_with(4, 65, 0, "minecraft:oak_leaves", &[("persistent", "true")]),
                put(-2, 64, 0, "minecraft:water"),
                put(-4, 64, 0, "minecraft:glass"),
                put_with(-4, 65, 0, "minecraft:stone_stairs", &[("facing", "north"), ("half", "bottom")]),
                put(0, 66, 2, "minecraft:stone"),
                put(0, 64, 0, "minecraft:air"),
            ],
            [-6, 60, -3, 13, 9, 7],
        ),
        lit(
            "chunk-edge",
            slab((8, 24), (-4, 4), (61, 63)),
            vec![
                put(15, 63, 0, "minecraft:air"),
                put(16, 63, 0, "minecraft:air"),
                put(16, 62, 0, "minecraft:glowstone"),
                put(16, 62, 0, "minecraft:air"),
            ],
            [12, 58, -2, 8, 9, 5],
        ),
    ]
}

pub fn light_changes() -> Value {
    let rows = [room_rows(), open_rows()].concat();
    case("movement/light-changes", "movement", "light_changes", json!({"rows": rows}))
}
