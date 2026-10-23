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
    // Clicked face, the block clicked, where, the block past the place, and the axis between.
    let sides = [
        ("up", [0, 63, 0], [0.5, 64.0, 0.5], [0, 65, 0], "y"),
        ("south", [0, 64, 0], [0.5, 64.5, 1.0], [0, 64, 2], "z"),
        ("east", [0, 64, 0], [1.0, 64.5, 0.5], [2, 64, 0], "x"),
    ];
    for (face, hit, location, far, axis) in sides {
        let wrong = if axis == "y" { "x" } else { "y" };
        let log = |at: [i32; 3], axis: &str| block(at, "pale_oak_log", &[("axis", axis)]);
        let worlds = [
            ("alone", vec![block(hit, "stone", &[])]),
            ("one-log", vec![log(hit, axis)]),
            ("one-wrong-log", vec![log(hit, wrong)]),
            ("far-log", vec![block(hit, "stone", &[]), log(far, axis)]),
            ("far-wrong-log", vec![log(hit, axis), log(far, wrong)]),
            ("near-wrong-log", vec![log(hit, wrong), log(far, axis)]),
            ("both-wrong-logs", vec![log(hit, wrong), log(far, wrong)]),
            ("far-other-wood", vec![log(hit, axis), block(far, "oak_log", &[("axis", axis)])]),
        ];
        for (label, world) in worlds {
            let at = click(hit, face, location, 0.0, 0.0);
            rows.push(row(&format!("creaking-heart-{face}-{label}"), &heart, world, &at));
        }
    }
    rows
}

/// A tripwire beside tripwires and hooks that face it, away from it or are other blocks.
pub fn tripwire_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    let sides = [("north", [0, 64, -1]), ("east", [1, 64, 0]), ("south", [0, 64, 1]), ("west", [-1, 64, 0])];
    let faces = ["north", "east", "south", "west"];
    for (label, at) in sides {
        for beside in ["tripwire", "stone", "oak_slab"] {
            let world = vec![block([0, 63, 0], "stone", &[]), block(at, beside, &[])];
            rows.push(row(&format!("tripwire-{label}-{beside}"), &["tripwire"], world, &ON_TOP));
        }
        for facing in faces {
            let hook = block(at, "tripwire_hook", &[("facing", facing)]);
            let world = vec![block([0, 63, 0], "stone", &[]), hook];
            rows.push(row(&format!("tripwire-{label}-hook-{facing}"), &["tripwire"], world, &ON_TOP));
        }
    }
    rows
}

/// Big and small dripleaves on every kind of ground, in water, under blocks and facing every way.
pub fn dripleaf_rows() -> Vec<Value> {
    let blocks = ["big_dripleaf", "small_dripleaf"];
    let mut rows = Vec::new();
    for ground in ["clay", "moss_block", "dirt", "grass_block", "stone", "mud", "sand", "farmland", "glass"] {
        rows.push(row(&format!("dripleaf-on-{ground}"), &blocks, vec![block([0, 63, 0], ground, &[])], &ON_TOP));
    }
    for facing in ["north", "east", "south", "west"] {
        for part in ["big_dripleaf", "big_dripleaf_stem"] {
            let world = vec![block([0, 63, 0], part, &[("facing", facing)])];
            rows.push(row(&format!("dripleaf-on-{part}-{facing}"), &blocks, world, &ON_TOP));
        }
    }
    for yaw in [0.0, 90.0, 180.0, 270.0] {
        let at = click([0, 63, 0], "up", [0.5, 64.0, 0.5], yaw, 0.0);
        rows.push(row(&format!("dripleaf-yaw{yaw}"), &blocks, vec![block([0, 63, 0], "clay", &[])], &at));
    }
    for ground in ["clay", "dirt", "stone", "sand"] {
        for (label, level) in [("source", "0"), ("flowing", "3"), ("falling", "8")] {
            let world = vec![block([0, 63, 0], ground, &[]), block([0, 64, 0], "water", &[("level", level)])];
            let at = click([0, 64, 0], "up", [0.5, 65.0, 0.5], 0.0, 0.0);
            rows.push(row(&format!("dripleaf-in-{label}-over-{ground}"), &blocks, world, &at));
            let world = vec![block([0, 63, 0], ground, &[]), block([0, 64, 0], "water", &[("level", level)])];
            rows.push(row(&format!("dripleaf-beside-{label}-over-{ground}"), &blocks, world, &ON_TOP));
        }
    }
    for above in ["stone", "oak_slab", "water", "grass_block", "small_dripleaf"] {
        let world = vec![block([0, 63, 0], "clay", &[]), block([0, 65, 0], above, &[])];
        rows.push(row(&format!("dripleaf-under-{above}"), &blocks, world, &ON_TOP));
    }
    rows
}

/// A pitcher pod on crop soil or not, in the light and the dark, with room above or not.
pub fn pitcher_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for ground in ["farmland", "grass_block", "dirt", "stone", "podzol", "sand", "glass"] {
        for light in [0, 7, 8, 15] {
            let world = vec![block([0, 63, 0], ground, &[])];
            let mut row = row(&format!("pitcher-{ground}-{light}"), &["pitcher_crop"], world, &ON_TOP);
            row["raw_light"] = json!(light);
            rows.push(row);
        }
    }
    for above in ["stone", "oak_slab", "water"] {
        let world = vec![block([0, 63, 0], "farmland", &[]), block([0, 65, 0], above, &[])];
        let mut under = row(&format!("pitcher-under-{above}"), &["pitcher_crop"], world, &ON_TOP);
        under["raw_light"] = json!(15);
        rows.push(under);
    }
    rows
}
