//! Item classes that go beyond `BlockItem`: scaffolding moved along its stack, lily pad and
//! frogspawn on water, and the blocks only an operator in creative mode places.

use super::facing::{FACES, face_click};
use super::*;

fn flagged(mut row: Value, key: &str) -> Value {
    row[key] = json!(true);
    row
}

/// Scaffolding used on a stack of scaffolding: from every face, looking every way, sneaking or
/// not, from inside the block or not; and along runs that end, are blocked or run too long.
pub fn scaffolding_stack_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for face in FACES {
        for yaw in [0.0, 90.0, 180.0, -90.0] {
            for (sneak, inside) in [(false, false), (true, false), (true, true), (false, true)] {
                let world =
                    vec![block([0, 63, 0], "stone", &[]), block([0, 64, 0], "scaffolding", &[("distance", "0")])];
                let mut placed = row(
                    &format!("stack-{face}-yaw{yaw}-sneak{sneak}-inside{inside}"),
                    &["scaffolding"],
                    world,
                    &face_click([0, 64, 0], face, yaw, 0.0),
                );
                if sneak {
                    placed = sneaking(placed);
                }
                if inside {
                    placed = flagged(placed, "inside");
                }
                rows.push(placed);
            }
        }
    }
    // A run along the ground of `length` blocks, extended from its start looking along it.
    for length in 1..=9 {
        let mut world = vec![block([0, 63, 0], "stone", &[])];
        for index in 0..length {
            let distance = index.min(7).to_string();
            world.push(block([index, 64, 0], "scaffolding", &[("distance", &distance)]));
        }
        rows.push(row(
            &format!("run-{length}"),
            &["scaffolding"],
            world,
            &click([0, 64, 0], "up", [0.5, 65.0, 0.5], -90.0, 0.0),
        ));
    }
    // A column, with something above it that gives way or does not.
    for (label, above) in [("air", "air"), ("stone", "stone"), ("grass", "short_grass"), ("water", "water")] {
        let properties: &[(&str, &str)] = if above == "water" { &[("level", "0")] } else { &[] };
        let world = vec![
            block([0, 63, 0], "stone", &[]),
            block([0, 64, 0], "scaffolding", &[("distance", "0")]),
            block([0, 65, 0], "scaffolding", &[("distance", "0")]),
            block([0, 66, 0], above, properties),
        ];
        rows.push(row(
            &format!("column-below-{label}"),
            &["scaffolding"],
            world,
            &face_click([0, 64, 0], "north", 0.0, 0.0),
        ));
    }
    // Scaffolding on a block with no support within reach of the stack.
    let world = vec![block([0, 60, 0], "stone", &[])];
    rows.push(row("hanging", &["scaffolding"], world, &face_click([0, 60, 0], "north", 0.0, 0.0)));
    rows
}

/// Lily pads and frogspawn are put on the block above the water the look reaches.
pub fn water_item_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for (label, below, level) in [
        ("source", "water", "0"),
        ("flowing", "water", "3"),
        ("falling", "water", "8"),
        ("lava", "lava", "0"),
        ("ice", "ice", ""),
        ("blue-ice", "blue_ice", ""),
        ("frosted-ice", "frosted_ice", ""),
        ("stone", "stone", ""),
        ("air", "air", ""),
    ] {
        let properties: &[(&str, &str)] = if level.is_empty() { &[] } else { &[("level", level)] };
        let world = vec![block([0, 63, 0], below, properties)];
        rows.push(row(
            &format!("on-{label}"),
            &["lily_pad", "frogspawn"],
            world,
            &click([0, 64, 0], "up", [0.5, 64.0, 0.5], 0.0, 90.0),
        ));
    }
    // Into something that gives way, and something that does not.
    let world = vec![block([0, 63, 0], "water", &[("level", "0")]), block([0, 64, 0], "short_grass", &[])];
    rows.push(row(
        "into-grass",
        &["lily_pad", "frogspawn"],
        world,
        &click([0, 64, 0], "up", [0.5, 64.0, 0.5], 0.0, 90.0),
    ));
    let world = vec![block([0, 63, 0], "water", &[("level", "0")]), block([0, 64, 0], "oak_slab", &[])];
    rows.push(row(
        "into-slab",
        &["lily_pad", "frogspawn"],
        world,
        &click([0, 64, 0], "up", [0.5, 64.0, 0.5], 0.0, 90.0),
    ));
    rows
}

/// Command, structure, jigsaw and test blocks, with and without the operator's rights.
pub fn game_master_rows() -> Vec<Value> {
    let blocks = [
        "command_block",
        "repeating_command_block",
        "chain_command_block",
        "structure_block",
        "jigsaw",
        "test_instance_block",
    ];
    let mut rows = Vec::new();
    for operator in [false, true] {
        for yaw in [0.0, 90.0] {
            let world = vec![block([0, 63, 0], "stone", &[])];
            let mut placed = row(
                &format!("operator-{operator}-yaw{yaw}"),
                &blocks,
                world,
                &click([0, 63, 0], "up", [0.5, 64.0, 0.5], yaw, 30.0),
            );
            if operator {
                placed = flagged(placed, "game_master");
            }
            rows.push(placed);
        }
    }
    rows
}
