//! Chorus plants and flowers, coral blocks and creaking hearts, which look at their neighbours.

use super::*;

/// Chorus plants and flowers over every kind of ground, beside and under others and other blocks.
pub fn chorus_rows() -> Vec<Value> {
    let blocks = ["chorus_plant", "chorus_flower"];
    let mut rows = Vec::new();
    for ground in ["end_stone", "stone", "chorus_plant", "chorus_flower", "dirt", "glass", "air"] {
        let world = if ground == "air" { vec![] } else { vec![block([0, 63, 0], ground, &[])] };
        rows.push(row(&format!("chorus-on-{ground}"), &blocks, world, &ON_TOP));
    }
    let sides = [("north", [0, 64, -1]), ("east", [1, 64, 0]), ("south", [0, 64, 1]), ("west", [-1, 64, 0])];
    for (label, at) in sides {
        for beside in ["chorus_plant", "chorus_flower", "stone", "oak_slab"] {
            for (ground_label, ground) in [("air", "air"), ("stone", "stone")] {
                let mut world = vec![block(at, beside, &[])];
                if ground != "air" {
                    world.push(block([0, 63, 0], ground, &[]));
                }
                if beside == "chorus_plant" {
                    world.push(block([at[0], 63, at[2]], "end_stone", &[]));
                }
                rows.push(row(&format!("chorus-{label}-{beside}-over-{ground_label}"), &blocks, world, &ON_TOP));
            }
        }
    }
    for above in ["air", "stone", "chorus_plant", "chorus_flower"] {
        let world = vec![
            block([0, 63, 0], "end_stone", &[]),
            block([0, 65, 0], above, &[]),
            block([1, 64, 0], "chorus_plant", &[]),
            block([1, 63, 0], "stone", &[]),
        ];
        rows.push(row(&format!("chorus-above-{above}"), &blocks, world, &ON_TOP));
    }
    let two = vec![
        block([1, 64, 0], "chorus_plant", &[]),
        block([-1, 64, 0], "chorus_plant", &[]),
        block([1, 63, 0], "end_stone", &[]),
        block([-1, 63, 0], "end_stone", &[]),
    ];
    rows.push(row("chorus-two-beside", &blocks, two, &ON_TOP));
    let both = vec![
        block([0, 63, 0], "stone", &[]),
        block([0, 65, 0], "stone", &[]),
        block([1, 64, 0], "chorus_plant", &[]),
        block([1, 63, 0], "end_stone", &[]),
    ];
    rows.push(row("chorus-between-blocks", &blocks, both, &ON_TOP));
    rows
}

/// Coral blocks beside water or without, and creaking hearts beside pale oak logs. A heart
/// between two logs asks the time of day, which the oracle's level cannot answer.
pub fn lone_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    let coral = ["tube_coral_block", "brain_coral_block", "bubble_coral_block", "fire_coral_block", "horn_coral_block"];
    for (label, world) in [
        ("dry", vec![block([0, 63, 0], "stone", &[])]),
        ("wet", vec![block([0, 63, 0], "stone", &[]), block([1, 64, 0], "water", &[("level", "0")])]),
    ] {
        rows.push(row(&format!("coral-block-{label}"), &coral, world, &ON_TOP));
    }
    let heart = ["creaking_heart"];
    for face in [("up", [0.5, 64.0, 0.5], [0, 63, 0]), ("north", [0.5, 64.5, 0.0], [0, 64, 0])] {
        for (label, logs) in [
            ("alone", vec![]),
            ("one-log", vec![block([0, 65, 0], "pale_oak_log", &[("axis", "y")])]),
            (
                "wrong-axis",
                vec![
                    block([0, 65, 0], "pale_oak_log", &[("axis", "x")]),
                    block([0, 63, 0], "pale_oak_log", &[("axis", "y")]),
                ],
            ),
        ] {
            let mut world = vec![block([0, 63, 0], "stone", &[])];
            world.extend(logs.into_iter().filter(|log| log["pos"] != json!([0, 63, 0]) || face.0 == "up"));
            let at = click(face.2, face.0, face.1, 0.0, 0.0);
            rows.push(row(&format!("creaking-heart-{}-{label}", face.0), &heart, world, &at));
        }
    }
    rows
}
