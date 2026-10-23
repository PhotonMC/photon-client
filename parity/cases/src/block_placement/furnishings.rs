//! Blocks that face the player or wait in water: shelves, statues, grates, furnaces, anvils,
//! pumpkins, vaults, looms, lecterns, campfires, pots, sculk sensors, lamps and the like.

use super::facing::{face_rows, look_rows};
use super::*;

fn weathered(suffix: &str) -> Vec<String> {
    let mut names = Vec::new();
    for weathering in ["", "exposed_", "weathered_", "oxidized_"] {
        names.push(format!("{weathering}{suffix}"));
        names.push(format!("waxed_{weathering}{suffix}"));
    }
    names
}

/// Blocks put on the ground from every side and every way of looking, and into water.
pub fn facing_rows() -> Vec<Value> {
    let shelves: Vec<String> = WOODS.iter().map(|wood| format!("{wood}_shelf")).collect();
    let mut blocks: Vec<String> = shelves;
    blocks.extend(weathered("copper_golem_statue"));
    blocks.extend(weathered("copper_grate"));
    blocks.extend(
        [
            "furnace",
            "smoker",
            "blast_furnace",
            "anvil",
            "chipped_anvil",
            "damaged_anvil",
            "carved_pumpkin",
            "jack_o_lantern",
            "vault",
            "lectern",
            "loom",
            "stonecutter",
            "beehive",
            "bee_nest",
            "chiseled_bookshelf",
            "end_portal_frame",
            "decorated_pot",
            "dried_ghast",
            "campfire",
            "soul_campfire",
            "grindstone",
            "heavy_core",
            "sculk_sensor",
            "calibrated_sculk_sensor",
            "sculk_shrieker",
            "mangrove_roots",
            "barrier",
            "infested_deepslate",
        ]
        .map(String::from),
    );
    let blocks = names(&blocks);
    face_rows("face", &blocks)
}

/// Blocks that face away from the look, from every direction of looking.
pub fn looking_rows() -> Vec<Value> {
    let blocks = ["dispenser", "dropper", "barrel", "command_block", "crafter"];
    let mut rows = look_rows("look", &blocks);
    rows.extend(face_rows("face", &blocks));
    rows
}

/// Blocks a signal beside them changes.
pub fn signal_rows() -> Vec<Value> {
    let blocks = ["oak_shelf", "redstone_lamp", "crafter"];
    let mut rows = Vec::new();
    for (label, extra) in [
        ("alone", vec![]),
        ("redstone-block", vec![block([1, 64, 0], "redstone_block", &[])]),
        ("redstone-below", vec![block([0, 63, 0], "redstone_block", &[])]),
        ("lever-on", vec![block([-1, 64, 0], "lever", &[("powered", "true"), ("face", "floor")])]),
        ("lever-off", vec![block([-1, 64, 0], "lever", &[("powered", "false"), ("face", "floor")])]),
        ("stone", vec![block([0, 64, -1], "stone", &[])]),
    ] {
        for yaw in [0.0, 90.0] {
            let mut world = vec![block([0, 63, 0], "stone", &[])];
            world.extend(extra.clone());
            if label == "redstone-below" {
                world.remove(0);
                world.push(block([0, 63, 0], "redstone_block", &[]));
            }
            let at = click([0, 63, 0], "up", [0.5, 64.0, 0.5], yaw, 30.0);
            rows.push(row(&format!("signal-{label}-yaw{yaw}"), &blocks, world, &at));
        }
    }
    rows
}

/// A campfire over hay or on other ground, and in water.
pub fn campfire_rows() -> Vec<Value> {
    let blocks = ["campfire", "soul_campfire", "conduit"];
    let mut rows = Vec::new();
    for ground in ["stone", "hay_block", "grass_block", "oak_slab"] {
        rows.push(row(&format!("campfire-on-{ground}"), &blocks, vec![block([0, 63, 0], ground, &[])], &ON_TOP));
    }
    for (label, level) in [("source", "0"), ("flowing", "3"), ("falling", "8")] {
        let world = vec![block([0, 63, 0], "stone", &[]), block([0, 64, 0], "water", &[("level", level)])];
        rows.push(row(
            &format!("campfire-in-{label}"),
            &blocks,
            world,
            &click([0, 64, 0], "up", [0.5, 65.0, 0.5], 0.0, 0.0),
        ));
    }
    rows
}

/// A giant mushroom shows the faces that have no mushroom of its own kind against them.
pub fn huge_mushroom_rows() -> Vec<Value> {
    let blocks = ["red_mushroom_block", "brown_mushroom_block", "mushroom_stem"];
    let mut rows = Vec::new();
    let sides = [("down", [0, 63, 0]), ("up", [0, 65, 0]), ("east", [1, 64, 0]), ("west", [-1, 64, 0])];
    for (label, at) in sides {
        for neighbour in ["red_mushroom_block", "brown_mushroom_block", "mushroom_stem", "stone"] {
            let mut world = vec![block([0, 63, 0], "stone", &[])];
            world.push(block(at, neighbour, &[]));
            rows.push(row(&format!("mushroom-{label}-{neighbour}"), &blocks, world, &ON_TOP));
        }
    }
    rows
}

/// Blocks that care what lies above or below them: cake, paths, farmland, seagrass.
pub fn ground_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for ground in ["stone", "glass", "oak_slab", "grass_block", "water", "air"] {
        let world = if ground == "air" { vec![] } else { vec![block([0, 63, 0], ground, &[])] };
        rows.push(row(&format!("cake-on-{ground}"), &["cake"], world, &ON_TOP));
    }
    for over in ["stone", "glass", "oak_fence_gate", "oak_slab", "wheat", "farmland", "air"] {
        let world = vec![block([0, 63, 0], "stone", &[]), block([0, 65, 0], over, &[])];
        rows.push(row(&format!("soil-under-{over}"), &["dirt_path", "farmland"], world, &ON_TOP));
    }
    for ground in ["stone", "sand", "ice", "blue_ice", "packed_ice", "magma_block", "oak_slab", "glass"] {
        for (label, level) in [("source", "0"), ("flowing", "3"), ("falling", "8")] {
            let world = vec![block([0, 63, 0], ground, &[]), block([0, 64, 0], "water", &[("level", level)])];
            let at = click([0, 64, 0], "up", [0.5, 65.0, 0.5], 0.0, 0.0);
            rows.push(row(&format!("seagrass-on-{ground}-{label}"), &["seagrass"], world, &at));
        }
    }
    rows
}
