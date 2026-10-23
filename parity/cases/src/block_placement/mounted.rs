//! Torches, ladders, signs, banners, heads and coral fans: the item tries the standing block and
//! the wall block in the order the look reaches them.

use super::*;

const TORCHES: &[&str] = &[
    "torch",
    "soul_torch",
    "copper_torch",
    "redstone_torch",
    "ladder",
    "tube_coral_fan",
    "dead_brain_coral_fan",
    "fire_coral_fan",
];

const HEADS: &[&str] = &[
    "skeleton_skull",
    "wither_skeleton_skull",
    "zombie_head",
    "creeper_head",
    "dragon_head",
    "piglin_head",
    "player_head",
    "white_banner",
    "red_banner",
    "black_banner",
];

const SUPPORTS: &[&str] = &["stone", "oak_fence", "glass", "oak_slab", "ice", "oak_leaves", "hopper", "soul_sand"];

/// Clicks on the top, a side and the underside of a block, from every way of looking.
fn look_rows(prefix: &str, blocks: &[&str]) -> Vec<Value> {
    let mut rows = Vec::new();
    for yaw in [0.0, 90.0, 180.0, -90.0, 45.0, 135.0] {
        for pitch in [0.0, 60.0, -60.0, 30.0, -30.0] {
            let looking = format!("yaw{yaw}-pitch{pitch}");
            let floor = vec![block([0, 63, 0], "stone", &[])];
            let top = click([0, 63, 0], "up", [0.5, 64.0, 0.5], yaw, pitch);
            rows.push(row(&format!("{prefix}-top-{looking}"), blocks, floor, &top));
            let wall = vec![block([0, 63, 0], "stone", &[]), block([0, 64, -1], "stone", &[])];
            let side = click([0, 64, -1], "south", [0.5, 64.5, 0.0], yaw, pitch);
            rows.push(row(&format!("{prefix}-wall-{looking}"), blocks, wall, &side));
            let ceiling = vec![block([0, 65, 0], "stone", &[])];
            let under = click([0, 65, 0], "down", [0.5, 65.0, 0.5], yaw, pitch);
            rows.push(row(&format!("{prefix}-ceiling-{looking}"), blocks, ceiling, &under));
        }
    }
    rows
}

/// What is clicked, in each of the three places, decides what the block can hang from.
fn support_rows(prefix: &str, blocks: &[&str]) -> Vec<Value> {
    let mut rows = Vec::new();
    for support in SUPPORTS {
        let world = vec![block([0, 64, -1], support, &[])];
        let side = click([0, 64, -1], "south", [0.5, 64.5, 0.0], 0.0, 0.0);
        rows.push(row(&format!("{prefix}-wall-of-{support}"), blocks, world, &side));
        let world = vec![block([0, 63, 0], support, &[])];
        let top = click([0, 63, 0], "up", [0.5, 64.0, 0.5], 0.0, 0.0);
        rows.push(row(&format!("{prefix}-top-of-{support}"), blocks, world, &top));
    }
    // Into what gives way, and into water, where the look decides and the block floods.
    for pitch in [0.0, 70.0, -70.0] {
        let world = vec![block([0, 63, 0], "stone", &[]), block([0, 64, 0], "short_grass", &[])];
        let into = click([0, 64, 0], "up", [0.5, 65.0, 0.5], 0.0, pitch);
        rows.push(row(&format!("{prefix}-into-grass-pitch{pitch}"), blocks, world, &into));
    }
    for level in ["0", "3", "8"] {
        let world = vec![block([0, 63, 0], "stone", &[]), block([0, 64, 0], "water", &[("level", level)])];
        let into = click([0, 63, 0], "up", [0.5, 64.0, 0.5], 90.0, 0.0);
        rows.push(row(&format!("{prefix}-into-water-level{level}"), blocks, world, &into));
    }
    let both = vec![block([1, 64, 0], "stone", &[]), block([-1, 64, 0], "stone", &[])];
    rows.push(row(&format!("{prefix}-between"), blocks, both, &click([1, 64, 0], "west", [1.0, 64.5, 0.5], 90.0, 0.0)));
    rows
}

/// Ladders do not go on a ladder that already faces them.
fn ladder_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for facing in ["north", "south", "east", "west"] {
        let world = vec![
            block([0, 64, -1], "stone", &[]),
            block([0, 64, 0], "ladder", &[("facing", facing)]),
            block([0, 63, 0], "stone", &[]),
        ];
        for face in ["north", "south", "east", "west"] {
            let at = click([0, 64, 0], face, [0.5, 64.5, 0.5], 0.0, 0.0);
            rows.push(row(&format!("ladder-{facing}-clicked-{face}"), &["ladder"], world.clone(), &at));
        }
    }
    rows
}

pub fn torch_rows() -> Vec<Value> {
    let mut rows = look_rows("torches", TORCHES);
    rows.extend(support_rows("torches", TORCHES));
    rows.extend(ladder_rows());
    rows
}

pub fn head_rows() -> Vec<Value> {
    let mut rows = look_rows("heads", HEADS);
    rows.extend(support_rows("heads", HEADS));
    // A head's wall form only needs something beside it that does not give way.
    for beside in ["short_grass", "water", "snow", "oak_slab", "glass"] {
        let world = vec![block([0, 63, 0], "stone", &[]), block([0, 64, -1], beside, &[])];
        rows.push(row(
            &format!("heads-beside-{beside}"),
            HEADS,
            world,
            &click([0, 63, 0], "up", [0.5, 64.0, 0.5], 0.0, -60.0),
        ));
    }
    // Next to a redstone block, a head is powered.
    let powered = vec![block([0, 63, 0], "stone", &[]), block([1, 64, 0], "redstone_block", &[])];
    rows.push(row("heads-powered", HEADS, powered, &ON_TOP));
    rows
}

pub fn sign_rows() -> Vec<Value> {
    let signs: Vec<String> = WOODS.iter().map(|wood| format!("{wood}_sign")).collect();
    let hanging: Vec<String> = WOODS.iter().map(|wood| format!("{wood}_hanging_sign")).collect();
    let mut rows = look_rows("signs", &names(&signs[..3]));
    rows.extend(support_rows("signs", &names(&signs[..3])));
    rows.extend(look_rows("hanging", &names(&hanging[..3])));
    rows.extend(support_rows("hanging", &names(&hanging[..3])));
    let blocks = names(&hanging[..3]);
    // Chained from another hanging sign, or from a block with a half face.
    for (above, props) in [
        ("oak_hanging_sign", vec![("rotation", "0")]),
        ("oak_hanging_sign", vec![("rotation", "4")]),
        ("oak_hanging_sign", vec![("rotation", "5")]),
        ("oak_wall_hanging_sign", vec![("facing", "north")]),
        ("oak_wall_hanging_sign", vec![("facing", "east")]),
    ] {
        for yaw in [0.0, 90.0, 180.0, -90.0] {
            let world = vec![block([0, 65, 0], above, &props)];
            let under = click([0, 65, 0], "down", [0.5, 65.0, 0.5], yaw, 0.0);
            rows.push(row(&format!("hanging-under-{above}-{yaw}"), &blocks, world, &under));
        }
    }
    // Beside a wall hanging sign on one side only.
    for facing in ["north", "east"] {
        let world = vec![block([1, 64, 0], "oak_wall_hanging_sign", &[("facing", facing)])];
        for yaw in [0.0, 90.0] {
            let at = click([1, 64, 0], "west", [1.0, 64.5, 0.5], yaw, 0.0);
            rows.push(row(&format!("hanging-beside-{facing}-{yaw}"), &blocks, world.clone(), &at));
        }
    }
    rows
}
