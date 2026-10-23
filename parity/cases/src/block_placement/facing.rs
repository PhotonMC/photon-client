//! Leaves, chests, fence gates, observers, pistons, hoppers, chains, lightning rods, amethyst
//! buds, shulker boxes and end rods: the facing, pairing or waterlogging a click gives them.

use super::*;

const FACES: [&str; 6] = ["down", "up", "north", "south", "west", "east"];

/// The unit step of a face.
fn step(face: &str) -> [i32; 3] {
    match face {
        "down" => [0, -1, 0],
        "up" => [0, 1, 0],
        "north" => [0, 0, -1],
        "south" => [0, 0, 1],
        "west" => [-1, 0, 0],
        _ => [1, 0, 0],
    }
}

fn opposite(face: &str) -> &'static str {
    match face {
        "down" => "up",
        "up" => "down",
        "north" => "south",
        "south" => "north",
        "west" => "east",
        _ => "west",
    }
}

fn face_name(face: &str) -> &'static str {
    FACES.iter().copied().find(|name| *name == face).expect("a face")
}

/// A click on the middle of `face` of the block at `hit`.
fn face_click(hit: [i32; 3], face: &str, yaw: f32, pitch: f32) -> Click {
    let offset = step(face);
    let location = [0, 1, 2].map(|axis| f64::from(hit[axis]) + 0.5 + f64::from(offset[axis]) * 0.5);
    click(hit, face_name(face), location, yaw, pitch)
}

/// Every face of a stone block at the origin's height, from the four ways of looking.
fn face_rows(prefix: &str, blocks: &[&str]) -> Vec<Value> {
    let mut rows = Vec::new();
    for face in FACES {
        for yaw in [0.0, 90.0, 180.0, -90.0] {
            let world = vec![block([0, 64, 0], "stone", &[])];
            let at = face_click([0, 64, 0], face, yaw, 10.0);
            rows.push(row(&format!("{prefix}-{face}-yaw{yaw}"), blocks, world, &at));
        }
    }
    // Into water and into a flowing one.
    for (label, level) in [("water", "0"), ("flowing", "3")] {
        let world = vec![block([0, 64, 0], "water", &[("level", level)])];
        rows.push(row(&format!("{prefix}-in-{label}"), blocks, world, &face_click([0, 64, 0], "up", 0.0, 0.0)));
    }
    rows
}

/// The way the player looks, from every direction, onto the top of the ground.
fn look_rows(prefix: &str, blocks: &[&str]) -> Vec<Value> {
    let mut rows = Vec::new();
    for yaw in [0.0, 45.0, 90.0, 135.0, 180.0, -90.0, -135.0] {
        for pitch in [0.0, 30.0, 60.0, 89.0, -30.0, -60.0, -89.0] {
            let world = vec![block([0, 63, 0], "stone", &[])];
            let at = click([0, 63, 0], "up", [0.5, 64.0, 0.5], yaw, pitch);
            rows.push(row(&format!("{prefix}-yaw{yaw}-pitch{pitch}"), blocks, world, &at));
        }
    }
    rows
}

fn leaves() -> Vec<String> {
    let mut leaves: Vec<String> = [
        "oak",
        "spruce",
        "birch",
        "jungle",
        "acacia",
        "dark_oak",
        "pale_oak",
        "mangrove",
        "cherry",
        "azalea",
        "flowering_azalea",
    ]
    .iter()
    .map(|tree| format!("{tree}_leaves"))
    .collect();
    leaves.sort();
    leaves
}

/// Leaves are persistent and take the distance their neighbours give.
pub fn leaves_rows() -> Vec<Value> {
    let leaves = leaves();
    let blocks = names(&leaves);
    let ground = block([0, 63, 0], "stone", &[]);
    let mut rows = Vec::new();
    let mut with = |label: &str, extra: Vec<Value>| {
        let mut world = vec![ground.clone()];
        world.extend(extra);
        rows.push(row(&format!("leaves-{label}"), &blocks, world, &ON_TOP));
    };
    with("alone", vec![]);
    with("log-east", vec![block([1, 64, 0], "oak_log", &[("axis", "y")])]);
    with("log-above", vec![block([0, 65, 0], "birch_log", &[("axis", "y")])]);
    with("wood-north", vec![block([0, 64, -1], "oak_wood", &[("axis", "y")])]);
    with("stripped-south", vec![block([0, 64, 1], "stripped_oak_log", &[("axis", "x")])]);
    for distance in 1..=7 {
        let text = distance.to_string();
        let props = [("distance", text.as_str()), ("persistent", "true")];
        with(&format!("leaves-{distance}-west"), vec![block([-1, 64, 0], "oak_leaves", &props)]);
    }
    with(
        "two-leaves",
        vec![
            block([-1, 64, 0], "oak_leaves", &[("distance", "5")]),
            block([1, 64, 0], "oak_leaves", &[("distance", "2")]),
        ],
    );
    with("azalea-leaves-3", vec![block([0, 65, 0], "azalea_leaves", &[("distance", "3")])]);
    with("stone-around", vec![block([1, 64, 0], "stone", &[]), block([-1, 64, 0], "stone", &[])]);
    with("scaffolding", vec![block([1, 64, 0], "scaffolding", &[("distance", "0")])]);
    with("in-water", vec![block([0, 64, 0], "water", &[("level", "0")])]);
    with("in-flowing", vec![block([0, 64, 0], "water", &[("level", "2")])]);
    rows
}

const CHESTS: &[&str] = &[
    "chest",
    "trapped_chest",
    "ender_chest",
    "copper_chest",
    "exposed_copper_chest",
    "weathered_copper_chest",
    "oxidized_copper_chest",
    "waxed_copper_chest",
    "waxed_exposed_copper_chest",
    "waxed_weathered_copper_chest",
    "waxed_oxidized_copper_chest",
];

/// Chests face the player, wherever they are put.
pub fn chest_alone_rows() -> Vec<Value> {
    let mut rows = face_rows("chest", CHESTS);
    for (label, other, props) in [
        ("stone", "stone", vec![]),
        ("chest-west", "chest", vec![("facing", "north")]),
        ("trapped-west", "trapped_chest", vec![("facing", "north")]),
    ] {
        let world = vec![block([0, 63, 0], "stone", &[]), block([-1, 64, 0], other, &props)];
        rows.push(row(&format!("chest-beside-{label}"), CHESTS, world, &ON_TOP));
    }
    rows.extend(look_rows("chest", CHESTS).into_iter().step_by(11));
    rows
}

const DIRECTIONS: [&str; 4] = ["north", "south", "west", "east"];

fn walk_or_sneak(label: &str, sneak: bool) -> String {
    format!("{label}-{}", if sneak { "sneak" } else { "walk" })
}

/// Every facing with every yaw, once walking and once sneaking.
fn looks() -> impl Iterator<Item = (&'static str, f32, bool)> {
    DIRECTIONS.into_iter().flat_map(|facing| {
        [0.0, 90.0, 180.0, -90.0].into_iter().flat_map(move |yaw| [false, true].map(|sneak| (facing, yaw, sneak)))
    })
}

/// Put on the ground beside a single chest, from every way of looking, sneaking or not.
fn beside_partner_rows(side: &str, kinds: &[&str]) -> Vec<Value> {
    let offset = step(side);
    let partner = [offset[0], 64, offset[2]];
    looks()
        .map(|(facing, yaw, sneak)| {
            let world = vec![
                block([0, 63, 0], "stone", &[]),
                block(partner, "chest", &[("facing", facing), ("type", "single")]),
            ];
            let at = click([0, 63, 0], "up", [0.5, 64.0, 0.5], yaw, 0.0);
            let label = walk_or_sneak(&format!("beside-{side}-facing-{facing}-yaw{yaw}"), sneak);
            let placed = row(&label, kinds, world, &at);
            if sneak { sneaking(placed) } else { placed }
        })
        .collect()
}

/// Click the side of a single chest, from every way of looking, sneaking or not.
fn side_of_partner_rows(side: &str, kinds: &[&str]) -> Vec<Value> {
    let offset = step(side);
    let partner = [offset[0], 64, offset[2]];
    looks()
        .map(|(facing, yaw, sneak)| {
            let world = vec![block(partner, "chest", &[("facing", facing), ("type", "single")])];
            let at = face_click(partner, opposite(side), yaw, 0.0);
            let label = walk_or_sneak(&format!("side-{side}-facing-{facing}-yaw{yaw}"), sneak);
            let placed = row(&label, kinds, world, &at);
            if sneak { sneaking(placed) } else { placed }
        })
        .collect()
}

/// A single chest beside the place, facing every way, with the player facing every way and
/// sneaking or not; and the player sneaking against a chest's side.
pub fn chest_pairing_rows() -> Vec<Value> {
    let kinds = ["chest", "trapped_chest", "ender_chest"];
    let mut rows = Vec::new();
    for side in DIRECTIONS {
        rows.extend(beside_partner_rows(side, &kinds));
        rows.extend(side_of_partner_rows(side, &kinds));
    }
    // Partners that do not count: a double half, another kind of chest.
    let single = vec![("facing", "north"), ("type", "single")];
    for (label, other, props) in [
        ("left-half", "chest", vec![("facing", "north"), ("type", "left")]),
        ("right-half", "chest", vec![("facing", "north"), ("type", "right")]),
        ("trapped", "trapped_chest", single),
        ("ender", "ender_chest", vec![("facing", "north")]),
    ] {
        let world = vec![block([0, 63, 0], "stone", &[]), block([-1, 64, 0], other, &props)];
        let at = click([0, 63, 0], "up", [0.5, 64.0, 0.5], 0.0, 0.0);
        rows.push(row(&format!("other-{label}"), &["chest"], world, &at));
    }
    rows
}

const GATES: &[&str] = &[
    "oak_fence_gate",
    "spruce_fence_gate",
    "birch_fence_gate",
    "jungle_fence_gate",
    "acacia_fence_gate",
    "cherry_fence_gate",
    "dark_oak_fence_gate",
    "pale_oak_fence_gate",
    "mangrove_fence_gate",
    "bamboo_fence_gate",
    "crimson_fence_gate",
    "warped_fence_gate",
];

/// Fence gates face the player, open by a signal and sit in a wall that stands beside them.
pub fn gate_rows() -> Vec<Value> {
    let ground = block([0, 63, 0], "stone", &[]);
    let mut rows = Vec::new();
    let beside = [
        ("alone", vec![]),
        ("wall-west", vec![block([-1, 64, 0], "cobblestone_wall", &[])]),
        ("wall-east", vec![block([1, 64, 0], "mossy_cobblestone_wall", &[])]),
        ("wall-north", vec![block([0, 64, -1], "stone_brick_wall", &[])]),
        ("wall-south", vec![block([0, 64, 1], "brick_wall", &[])]),
        ("fence-west", vec![block([-1, 64, 0], "oak_fence", &[])]),
        ("stone-north", vec![block([0, 64, -1], "stone", &[])]),
        ("redstone-block", vec![block([1, 64, 0], "redstone_block", &[])]),
        ("redstone-below", vec![block([0, 63, 0], "redstone_block", &[])]),
        ("lever-on", vec![block([-1, 64, 0], "lever", &[("powered", "true"), ("face", "floor")])]),
        ("lever-off", vec![block([-1, 64, 0], "lever", &[("powered", "false"), ("face", "floor")])]),
        ("both-walls", vec![block([-1, 64, 0], "cobblestone_wall", &[]), block([0, 64, -1], "cobblestone_wall", &[])]),
    ];
    for yaw in [0.0, 90.0, 180.0, -90.0, 45.0, 135.0] {
        for (label, extra) in &beside {
            let mut world = vec![ground.clone()];
            world.extend(extra.clone());
            let at = click([0, 63, 0], "up", [0.5, 64.0, 0.5], yaw, 0.0);
            rows.push(row(&format!("gate-yaw{yaw}-{label}"), GATES, world, &at));
        }
    }
    rows
}

/// Observers face the player's look, pistons away from it.
pub fn observer_piston_rows() -> Vec<Value> {
    let blocks = ["observer", "piston", "sticky_piston"];
    let mut rows = look_rows("look", &blocks);
    rows.extend(face_rows("face", &blocks));
    rows
}

/// A hopper points away from the side clicked, down for the top and the bottom.
pub fn hopper_rows() -> Vec<Value> {
    face_rows("hopper", &["hopper"])
}

fn chains() -> Vec<&'static str> {
    vec![
        "iron_chain",
        "copper_chain",
        "exposed_copper_chain",
        "weathered_copper_chain",
        "oxidized_copper_chain",
        "waxed_copper_chain",
        "waxed_exposed_copper_chain",
        "waxed_weathered_copper_chain",
        "waxed_oxidized_copper_chain",
    ]
}

fn rods() -> Vec<&'static str> {
    vec![
        "lightning_rod",
        "exposed_lightning_rod",
        "weathered_lightning_rod",
        "oxidized_lightning_rod",
        "waxed_lightning_rod",
        "waxed_exposed_lightning_rod",
        "waxed_weathered_lightning_rod",
        "waxed_oxidized_lightning_rod",
    ]
}

const BUDS: &[&str] = &["small_amethyst_bud", "medium_amethyst_bud", "large_amethyst_bud", "amethyst_cluster"];

/// Chains lie along the axis of the face, rods and buds point out of it.
pub fn chain_rod_rows() -> Vec<Value> {
    let mut all = chains();
    all.extend(rods());
    all.extend(BUDS);
    face_rows("faced", &all)
}

/// Buds need a full face to grow from.
pub fn bud_support_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for support in ["stone", "oak_slab", "glass", "oak_fence", "ice", "amethyst_block", "oak_leaves", "hopper", "dirt"]
    {
        for face in FACES {
            let world = vec![block([0, 64, 0], support, &[])];
            let at = face_click([0, 64, 0], face, 0.0, 0.0);
            rows.push(row(&format!("bud-{face}-of-{support}"), BUDS, world, &at));
        }
    }
    rows
}

/// Shulker boxes open out of the face clicked.
pub fn shulker_rows() -> Vec<Value> {
    let mut boxes = colored("shulker_box");
    boxes.push("shulker_box".to_owned());
    face_rows("shulker", &names(&boxes))
}

/// An end rod points out of the face, or back along a rod that points at it.
pub fn end_rod_rows() -> Vec<Value> {
    let mut rows = face_rows("end-rod", &["end_rod"]);
    for face in FACES {
        for rod_facing in FACES {
            let world = vec![block([0, 64, 0], "end_rod", &[("facing", rod_facing)])];
            let at = face_click([0, 64, 0], face, 0.0, 0.0);
            rows.push(row(&format!("end-rod-on-{rod_facing}-rod-{face}"), &["end_rod"], world, &at));
        }
    }
    rows
}
