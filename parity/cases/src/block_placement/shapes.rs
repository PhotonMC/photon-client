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

/// Chests of every kind beside a chest of every half, with a chest placed against them (pairing
/// when walking, single when sneaking).
pub fn chest_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for (side, at) in SIDES {
        for facing in ["north", "east"] {
            for half in ["single", "left", "right"] {
                let world =
                    vec![block([0, 63, 0], "stone", &[]), block(at, "chest", &[("facing", facing), ("type", half)])];
                let label = format!("shape-chest-{side}-{facing}-{half}");
                let placed = ["chest", "trapped_chest", "ender_chest", "stone"];
                rows.push(neighbour_row(&label, world.clone(), &placed));
                rows.push(sneaking(neighbour_row(&format!("{label}-sneak"), world, &placed)));
            }
        }
    }
    rows
}

/// Copper chests beside copper chests of every weathering and half, with one placed single.
pub fn copper_chest_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for (side, at) in SIDES {
        for kind in ["copper_chest", "weathered_copper_chest", "waxed_copper_chest"] {
            for half in ["left", "right"] {
                let world =
                    vec![block([0, 63, 0], "stone", &[]), block(at, kind, &[("facing", "north"), ("type", half)])];
                let placed = ["copper_chest", "exposed_copper_chest", "stone"];
                rows.push(sneaking(neighbour_row(&format!("shape-{kind}-{side}-{half}"), world, &placed)));
            }
        }
    }
    rows
}

/// Vines beside, over and under the block placed: the face toward it holds only against a full
/// face, a face over air goes, and a vine above can hold a face up.
pub fn vine_rows() -> Vec<Value> {
    let placed = ["stone", "oak_fence", "glass", "oak_leaves", "oak_slab"];
    let mut rows = Vec::new();
    for (side, at, toward, across) in [
        ("east", [1, 64, 0], "west", "north"),
        ("west", [-1, 64, 0], "east", "south"),
        ("north", [0, 64, -1], "south", "east"),
        ("south", [0, 64, 1], "north", "west"),
    ] {
        let hold = |axis: &str| match axis {
            "north" => [at[0], at[1], at[2] - 1],
            "south" => [at[0], at[1], at[2] + 1],
            "east" => [at[0] + 1, at[1], at[2]],
            _ => [at[0] - 1, at[1], at[2]],
        };
        let base = || vec![block([0, 63, 0], "stone", &[]), block(hold(across), "stone", &[])];
        let faces = [(across, "true"), (toward, "true")];
        let mut world = base();
        world.push(block(at, "vine", &faces));
        rows.push(neighbour_row(&format!("shape-vine-{side}"), world, &placed));
        let mut world = base();
        world.push(block(at, "vine", &[(toward, "true")]));
        rows.push(neighbour_row(&format!("shape-vine-{side}-only"), world, &placed));
        let mut world = base();
        world.push(block(at, "vine", &[(across, "true"), (side, "true")]));
        rows.push(neighbour_row(&format!("shape-vine-{side}-over-air"), world, &placed));
        let mut world = base();
        world.push(block(at, "vine", &[(across, "true"), (side, "true")]));
        world.push(block([at[0], at[1] + 1, at[2]], "vine", &[(side, "true")]));
        rows.push(neighbour_row(&format!("shape-vine-{side}-held-above"), world, &placed));
        let mut world = base();
        world.push(block(at, "vine", &[(across, "true"), ("up", "true")]));
        rows.push(neighbour_row(&format!("shape-vine-{side}-up-over-air"), world, &placed));
    }
    // The direction to a vine above is down, which leaves it as it is, unsupported face and all.
    let under = vec![
        block([0, 63, 0], "stone", &[]),
        block([0, 65, -1], "stone", &[]),
        block([0, 65, 0], "vine", &[("east", "true"), ("north", "true")]),
    ];
    rows.push(neighbour_row("shape-vine-over-placed", under, &placed));
    rows
}

const INERT: [&str; 64] = [
    "oak_sapling",
    "dandelion",
    "torch",
    "wall_torch",
    "white_carpet",
    "moss_carpet",
    "sugar_cane",
    "brain_coral",
    "brain_coral_fan",
    "brain_coral_wall_fan",
    "bubble_column",
    "spore_blossom",
    "frogspawn",
    "comparator",
    "barrier",
    "soul_fire",
    "sand",
    "suspicious_sand",
    "mangrove_roots",
    "creaking_heart",
    "chorus_flower",
    "oak_leaves",
    "redstone_wall_torch",
    "amethyst_cluster",
    "white_banner",
    "white_wall_banner",
    "dead_brain_coral",
    "dead_brain_coral_wall_fan",
    "stone_pressure_plate",
    "rail",
    "big_dripleaf_stem",
    "cactus",
    "cake",
    "candle",
    "white_candle_cake",
    "oak_hanging_sign",
    "oak_wall_hanging_sign",
    "oak_sign",
    "oak_wall_sign",
    "iron_chain",
    "cocoa",
    "conduit",
    "decorated_pot",
    "dirt_path",
    "dried_ghast",
    "farmland",
    "flower_pot",
    "hanging_roots",
    "heavy_core",
    "ladder",
    "light",
    "lightning_rod",
    "mangrove_propagule",
    "observer",
    "repeater",
    "scaffolding",
    "sculk_sensor",
    "sculk_shrieker",
    "sea_pickle",
    "seagrass",
    "oak_slab",
    "snow",
    "oak_trapdoor",
    "tinted_glass",
];

/// Blocks whose `updateShape` only schedules a tick or falls to air, east of the block placed.
pub fn inert_rows() -> Vec<Value> {
    let placed = ["stone", "oak_fence", "glass", "dirt"];
    INERT
        .iter()
        .map(|kind| {
            let world = vec![block([0, 63, 0], "stone", &[]), block([1, 64, 0], kind, &[])];
            neighbour_row(&format!("shape-inert-{kind}"), world, &placed)
        })
        .collect()
}

/// Mushroom blocks whose faces toward another of the kind go, and the moss whose tip follows
/// the moss under it.
pub fn mushroom_rows() -> Vec<Value> {
    let faces =
        [("north", "true"), ("east", "true"), ("south", "true"), ("west", "true"), ("up", "true"), ("down", "true")];
    let mut rows: Vec<Value> = ["red_mushroom_block", "brown_mushroom_block", "mushroom_stem"]
        .iter()
        .map(|kind| {
            let world = vec![block([0, 63, 0], "stone", &[]), block([1, 64, 0], kind, &faces)];
            neighbour_row(
                &format!("shape-mushroom-{kind}"),
                world,
                &["red_mushroom_block", "brown_mushroom_block", "mushroom_stem", "stone"],
            )
        })
        .collect();
    let moss = vec![
        block([0, 63, 0], "stone", &[]),
        block([0, 65, 0], "pale_hanging_moss", &[("tip", "true")]),
        block([0, 66, 0], "stone", &[]),
    ];
    rows.push(neighbour_row("shape-moss-under", moss, &["stone", "pale_hanging_moss", "oak_fence"]));
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

const INSTRUMENT_BLOCKS: [&str; 22] = [
    "stone",
    "oak_planks",
    "sand",
    "glass",
    "gold_block",
    "clay",
    "bone_block",
    "iron_block",
    "soul_sand",
    "pumpkin",
    "emerald_block",
    "hay_block",
    "glowstone",
    "copper_block",
    "white_wool",
    "packed_ice",
    "skeleton_skull",
    "zombie_head",
    "creeper_head",
    "player_head",
    "piglin_head",
    "dragon_head",
];

/// Note blocks below and above every kind of block.
pub fn note_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for kind in INSTRUMENT_BLOCKS {
        let below = vec![block([0, 63, 0], kind, &[])];
        rows.push(row(&format!("note-over-{kind}"), &["note_block"], below, &ON_TOP));
        let above = vec![block([0, 63, 0], "stone", &[]), block([0, 65, 0], kind, &[])];
        rows.push(row(&format!("note-under-{kind}"), &["note_block"], above, &ON_TOP));
        let both = vec![block([0, 63, 0], "stone", &[]), block([0, 65, 0], "skeleton_skull", &[])];
        rows.push(row(&format!("note-between-{kind}"), &["note_block"], both, &ON_TOP));
    }
    rows
}

/// Blocks placed against note blocks.
pub fn note_neighbour_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for kind in ["air", "stone", "skeleton_skull", "zombie_head", "gold_block", "player_head"] {
        let under = vec![block([0, 62, 0], kind, &[]), block([0, 63, 0], "note_block", &[])];
        rows.push(neighbour_row(&format!("note-below-placed-{kind}"), under, &INSTRUMENT_BLOCKS));
        let over =
            vec![block([0, 63, 0], "stone", &[]), block([0, 65, 0], "note_block", &[]), block([0, 66, 0], kind, &[])];
        rows.push(neighbour_row(&format!("note-above-placed-{kind}"), over, &INSTRUMENT_BLOCKS));
    }
    rows
}

/// Potent sulfur under every kind of water and over every kind of ground, and the blocks
/// placed under one that is already there.
pub fn sulfur_rows() -> Vec<Value> {
    let grounds: [Kind; 7] = [
        ("stone", "stone", &[]),
        ("magma", "magma_block", &[]),
        ("lava", "lava", &[("level", "0")]),
        ("flowing-lava", "lava", &[("level", "3")]),
        ("water", "water", &[("level", "0")]),
        ("air", "air", &[]),
        ("dirt", "dirt", &[]),
    ];
    let tops: [Kind; 5] = [
        ("air", "air", &[]),
        ("source", "water", &[("level", "0")]),
        ("flowing", "water", &[("level", "3")]),
        ("falling", "water", &[("level", "8")]),
        ("stone", "stone", &[]),
    ];
    let side = Click { hit: [1, 63, 0], face: "west", location: [1.0, 63.5, 0.5], ..ON_TOP };
    let mut rows = Vec::new();
    for (ground_label, ground, ground_props) in grounds {
        for (top_label, top, top_props) in tops {
            let world = vec![
                block([1, 63, 0], "stone", &[]),
                block([0, 62, 0], ground, ground_props),
                block([0, 64, 0], top, top_props),
            ];
            let label = format!("sulfur-over-{ground_label}-under-{top_label}");
            rows.push(row(&label, &["potent_sulfur"], world, &side));
        }
    }
    for (state, top) in [("wet", "source"), ("dry", "air"), ("erupting", "source"), ("dormant", "source")] {
        let water =
            if top == "air" { block([0, 66, 0], "air", &[]) } else { block([0, 66, 0], "water", &[("level", "0")]) };
        let world = vec![
            block([0, 63, 0], "stone", &[]),
            block([0, 65, 0], "potent_sulfur", &[("potent_sulfur_state", state)]),
            water,
        ];
        rows.push(neighbour_row(&format!("sulfur-{state}-above"), world, &["stone", "magma_block"]));
    }
    rows
}
