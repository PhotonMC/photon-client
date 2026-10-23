//! Blocks placed beside others that reshape to them (`updateShape` of the neighbour).

use super::*;

const PLACED: [&str; 9] = [
    "stone",
    "oak_fence",
    "glass_pane",
    "cobblestone_wall",
    "oak_fence_gate",
    "snow_block",
    "chorus_plant",
    "tripwire",
    "big_dripleaf",
];

/// A neighbour row: `block` stands at `at` before the stone below the click is built on.
fn neighbour_row(label: &str, world: Vec<Value>, blocks: &[&str]) -> Value {
    let mut row = row(label, blocks, world, &ON_TOP);
    row["neighbours"] = Value::Bool(true);
    row
}

const SIDES: [(&str, [i32; 3]); 4] =
    [("north", [0, 64, -1]), ("east", [1, 64, 0]), ("south", [0, 64, 1]), ("west", [-1, 64, 0])];

/// One kind of neighbour on each side of the block placed on the stone.
fn beside(name: &str, kind: &str, props: &[(&str, &str)]) -> Vec<Value> {
    SIDES
        .iter()
        .map(|(side, at)| {
            let world = vec![
                block([0, 63, 0], "stone", &[]),
                block(*at, kind, props),
                block([at[0], 63, at[2]], "end_stone", &[]),
            ];
            neighbour_row(&format!("shape-{name}-{side}"), world, &PLACED)
        })
        .collect()
}

/// Fences, whose sides join what is placed beside them.
pub fn fence_rows() -> Vec<Value> {
    let mut rows = beside("fence", "oak_fence", &[]);
    rows.extend(beside("nether-fence", "nether_brick_fence", &[]));
    rows
}

/// Iron bars and panes.
pub fn bars_rows() -> Vec<Value> {
    let mut rows = beside("bars", "iron_bars", &[]);
    rows.extend(beside("pane", "glass_pane", &[]));
    rows
}

/// Walls, whose sides and post follow what is placed beside and above them.
pub fn wall_rows() -> Vec<Value> {
    let mut rows = beside("wall", "cobblestone_wall", &[]);
    let line = [("north", "low"), ("south", "low"), ("up", "false")];
    let under = vec![block([0, 63, 0], "cobblestone_wall", &line)];
    rows.push(neighbour_row("shape-wall-under", under, &PLACED));
    rows
}

/// Fence gates, which lower into a wall they stand against.
pub fn gate_rows() -> Vec<Value> {
    let mut rows = beside("gate", "oak_fence_gate", &[("facing", "north")]);
    rows.extend(beside("gate-east", "oak_fence_gate", &[("facing", "east")]));
    for (label, kind) in [("walls", "cobblestone_wall"), ("fences", "oak_fence")] {
        let world = vec![
            block([0, 63, 0], "stone", &[]),
            block([1, 64, 0], "oak_fence_gate", &[("facing", "north")]),
            block([1, 64, 1], kind, &[]),
        ];
        rows.push(neighbour_row(&format!("shape-gate-beside-{label}"), world, &PLACED));
    }
    rows
}

/// Grass-like blocks that turn snowy under snow.
pub fn snowy_rows() -> Vec<Value> {
    ["grass_block", "podzol", "mycelium", "dirt"]
        .iter()
        .map(|ground| neighbour_row(&format!("shape-snowy-{ground}"), vec![block([0, 63, 0], ground, &[])], &PLACED))
        .collect()
}

/// Chorus plants, which join what is placed beside them.
pub fn chorus_rows() -> Vec<Value> {
    beside("chorus", "chorus_plant", &[])
}

/// Tripwires, which join what is placed beside them.
pub fn tripwire_rows() -> Vec<Value> {
    beside("tripwire", "tripwire", &[])
}

/// A big dripleaf with a block placed on top of it.
pub fn dripleaf_rows() -> Vec<Value> {
    ["north", "south"]
        .iter()
        .map(|facing| {
            let world = vec![block([0, 62, 0], "clay", &[]), block([0, 63, 0], "big_dripleaf", &[("facing", *facing)])];
            neighbour_row(&format!("shape-dripleaf-over-{facing}"), world, &PLACED)
        })
        .collect()
}

/// A neighbour's label, block and state.
type Kind = (&'static str, &'static str, &'static [(&'static str, &'static str)]);

/// Pointed dripstone and sulfur spikes between every kind of neighbour above and below, clicked
/// from either side, looking up and down, walking and sneaking.
pub fn speleothem_rows() -> Vec<Value> {
    let kinds: [Kind; 8] = [
        ("air", "air", &[]),
        ("stone", "stone", &[]),
        ("sand", "soul_sand", &[]),
        ("down-tip", "pointed_dripstone", &[("vertical_direction", "down"), ("thickness", "tip")]),
        ("down-frustum", "pointed_dripstone", &[("vertical_direction", "down"), ("thickness", "frustum")]),
        ("up-tip", "pointed_dripstone", &[("vertical_direction", "up"), ("thickness", "tip")]),
        ("up-merge", "pointed_dripstone", &[("vertical_direction", "up"), ("thickness", "tip_merge")]),
        ("sulfur-down", "sulfur_spike", &[("vertical_direction", "down"), ("thickness", "tip")]),
    ];
    let mut rows = Vec::new();
    for (below_label, below, below_props) in kinds {
        for (above_label, above, above_props) in kinds {
            for (side, clicked_below) in [("floor", true), ("ceiling", false)] {
                let clicked = if clicked_below { below } else { above };
                if clicked == "air" {
                    continue;
                }
                let world = vec![block([0, 63, 0], below, below_props), block([0, 65, 0], above, above_props)];
                let aim = if clicked_below {
                    ON_TOP
                } else {
                    Click { hit: [0, 65, 0], face: "down", location: [0.5, 65.0, 0.5], ..ON_TOP }
                };
                for pitch in [-45.0, 45.0] {
                    let label = format!("speleothem-{below_label}-{above_label}-{side}-{pitch}");
                    let mut row = neighbour_row(&label, world.clone(), &["pointed_dripstone", "sulfur_spike"]);
                    row["hit"] = json!(aim.hit);
                    row["face"] = json!(aim.face);
                    row["location"] = json!(aim.location.map(bits));
                    row["pitch"] = json!(bits32(pitch));
                    rows.push(row.clone());
                    rows.push(sneaking(row));
                }
            }
        }
    }
    for (label, level) in [("source", "0"), ("flowing", "3")] {
        let world = vec![block([0, 63, 0], "stone", &[]), block([0, 64, 0], "water", &[("level", level)])];
        let blocks = ["pointed_dripstone", "sulfur_spike"];
        rows.push(neighbour_row(&format!("speleothem-in-{label}"), world, &blocks));
    }
    rows
}
