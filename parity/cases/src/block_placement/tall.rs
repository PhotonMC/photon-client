//! Doors, tall plants and beds: the part that is put down, and the part the placed block adds.

use super::*;

fn doors() -> Vec<String> {
    let mut doors: Vec<String> = WOODS.iter().map(|wood| format!("{wood}_door")).collect();
    doors.push("iron_door".to_owned());
    for weathering in ["", "exposed_", "weathered_", "oxidized_"] {
        doors.push(format!("{weathering}copper_door"));
        doors.push(format!("waxed_{weathering}copper_door"));
    }
    doors
}

/// Full blocks beside the door, above it, and doors already put up.
fn door_surroundings() -> Vec<(&'static str, Vec<Value>)> {
    vec![
        ("stone-west", vec![block([-1, 64, 0], "stone", &[])]),
        ("stone-east", vec![block([1, 64, 0], "stone", &[])]),
        ("stone-north", vec![block([0, 64, -1], "stone", &[])]),
        ("stone-south", vec![block([0, 64, 1], "stone", &[])]),
        ("upper-east", vec![block([1, 65, 0], "stone", &[])]),
        ("upper-north", vec![block([0, 65, -1], "stone", &[])]),
        ("both-sides", vec![block([-1, 64, 0], "stone", &[]), block([1, 64, 0], "stone", &[])]),
        ("slab-west", vec![block([-1, 64, 0], "oak_slab", &[("type", "double")])]),
        ("fence-west", vec![block([-1, 64, 0], "oak_fence", &[])]),
        ("door-west", vec![block([-1, 64, 0], "oak_door", &[("half", "lower")])]),
        ("door-east", vec![block([1, 64, 0], "oak_door", &[("half", "lower")])]),
        ("door-north", vec![block([0, 64, -1], "oak_door", &[("half", "lower")])]),
        ("upper-door-east", vec![block([1, 64, 0], "oak_door", &[("half", "upper")])]),
        ("door-and-stone", vec![block([-1, 64, 0], "oak_door", &[("half", "lower")]), block([1, 64, 0], "stone", &[])]),
    ]
}

/// The hinge follows what stands on either side and, failing that, where the click was.
pub fn door_rows() -> Vec<Value> {
    let doors = doors();
    let blocks = names(&doors);
    let ground = block([0, 63, 0], "stone", &[]);
    let mut rows = Vec::new();
    for yaw in [0.0, 90.0, 180.0, -90.0] {
        for (x, z) in [(0.2, 0.2), (0.8, 0.2), (0.2, 0.8), (0.8, 0.8), (0.5, 0.5)] {
            let at = click([0, 63, 0], "up", [x, 64.0, z], yaw, 0.0);
            rows.push(row(&format!("door-yaw{yaw}-at{x}-{z}"), &blocks, vec![ground.clone()], &at));
        }
        for (label, beside) in door_surroundings() {
            let mut world = vec![ground.clone()];
            world.extend(beside);
            let at = click([0, 63, 0], "up", [0.3, 64.0, 0.6], yaw, 0.0);
            rows.push(row(&format!("door-yaw{yaw}-{label}"), &blocks, world, &at));
        }
    }
    // Room above, the height limit, water, and signals at either half.
    for (label, extra) in [
        ("blocked-above", vec![block([0, 65, 0], "stone", &[])]),
        ("grass-above", vec![block([0, 65, 0], "short_grass", &[])]),
        ("snow-above", vec![block([0, 65, 0], "snow", &[("layers", "1")])]),
        ("slab-above", vec![block([0, 65, 0], "oak_slab", &[("type", "bottom")])]),
        ("water-above", vec![block([0, 65, 0], "water", &[("level", "0")])]),
        ("redstone-beside", vec![block([1, 64, 0], "redstone_block", &[])]),
        ("redstone-beside-upper", vec![block([1, 65, 0], "redstone_block", &[])]),
        ("lever-beside", vec![block([1, 64, 0], "lever", &[("powered", "true"), ("face", "floor")])]),
    ] {
        let mut world = vec![ground.clone()];
        world.extend(extra);
        rows.push(row(&format!("door-{label}"), &blocks, world, &ON_TOP));
    }
    let top = vec![block([0, 318, 0], "stone", &[])];
    rows.push(row("door-at-limit", &blocks, top, &click([0, 318, 0], "up", [0.5, 319.0, 0.5], 0.0, 0.0)));
    let below = vec![block([0, 317, 0], "stone", &[])];
    rows.push(row("door-below-limit", &blocks, below, &click([0, 317, 0], "up", [0.5, 318.0, 0.5], 0.0, 0.0)));
    for ground in
        ["dirt", "oak_slab", "glass", "oak_fence", "snow", "grass_block", "ice", "water", "hopper", "oak_leaves"]
    {
        rows.push(row(&format!("door-on-{ground}"), &blocks, vec![block([0, 63, 0], ground, &[])], &ON_TOP));
    }
    rows.push(row(
        "door-into-grass",
        &blocks,
        vec![ground.clone(), block([0, 64, 0], "short_grass", &[])],
        &click([0, 64, 0], "up", [0.5, 65.0, 0.5], 0.0, 0.0),
    ));
    rows
}

const TALL: &[&str] = &["tall_grass", "large_fern", "sunflower", "lilac", "rose_bush", "peony"];

/// Tall plants need a ground and room above; the item clears the space above to water or air.
pub fn tall_plant_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for ground in ["grass_block", "dirt", "stone", "sand", "farmland", "podzol", "moss_block", "glass", "water"] {
        rows.push(row(&format!("tall-on-{ground}"), TALL, vec![block([0, 63, 0], ground, &[])], &ON_TOP));
    }
    let grass = block([0, 63, 0], "grass_block", &[]);
    for (label, extra) in [
        ("blocked-above", vec![block([0, 65, 0], "stone", &[])]),
        ("grass-above", vec![block([0, 65, 0], "short_grass", &[])]),
        ("tall-grass-above", vec![block([0, 65, 0], "tall_grass", &[("half", "upper")])]),
        ("snow-above", vec![block([0, 65, 0], "snow", &[("layers", "1")])]),
        ("water-above", vec![block([0, 65, 0], "water", &[("level", "0")])]),
        ("flowing-water-above", vec![block([0, 65, 0], "water", &[("level", "3")])]),
        ("water-in-place", vec![block([0, 64, 0], "water", &[("level", "0")])]),
    ] {
        let mut world = vec![grass.clone()];
        world.extend(extra);
        rows.push(row(&format!("tall-{label}"), TALL, world, &ON_TOP));
    }
    // Into what gives way: the place is the clicked block, and the space above is next to it.
    rows.push(row(
        "tall-into-short-grass",
        TALL,
        vec![grass.clone(), block([0, 64, 0], "short_grass", &[])],
        &click([0, 64, 0], "up", [0.5, 65.0, 0.5], 0.0, 0.0),
    ));
    rows.push(row(
        "tall-into-tall-grass",
        TALL,
        vec![
            grass.clone(),
            block([0, 64, 0], "tall_grass", &[("half", "lower")]),
            block([0, 65, 0], "tall_grass", &[("half", "upper")]),
        ],
        &click([0, 64, 0], "up", [0.5, 65.0, 0.5], 0.0, 0.0),
    ));
    let top = vec![block([0, 318, 0], "grass_block", &[])];
    rows.push(row("tall-at-limit", TALL, top, &click([0, 318, 0], "up", [0.5, 319.0, 0.5], 0.0, 0.0)));
    rows
}

/// A bed lies away from the player, over a head place that must give way.
pub fn bed_rows() -> Vec<Value> {
    let beds = colored("bed");
    let blocks = names(&beds[..3]);
    let ground = block([0, 63, 0], "stone", &[]);
    let mut rows = Vec::new();
    for yaw in [0.0, 90.0, 180.0, -90.0, 44.0, 46.0] {
        rows.push(row(
            &format!("bed-yaw{yaw}"),
            &blocks,
            vec![ground.clone()],
            &click([0, 63, 0], "up", [0.5, 64.0, 0.5], yaw, 0.0),
        ));
        for (label, head) in [
            ("stone", "stone"),
            ("grass", "short_grass"),
            ("water", "water"),
            ("snow", "snow"),
            ("slab", "oak_slab"),
            ("bed", "white_bed"),
        ] {
            // The head place, whichever way the player faces: around the foot.
            for at in [[0, 64, -1], [1, 64, 0], [0, 64, 1], [-1, 64, 0]] {
                let world = vec![ground.clone(), block(at, head, &[])];
                rows.push(row(
                    &format!("bed-yaw{yaw}-{label}-at{}{}", at[0], at[2]),
                    &blocks,
                    world,
                    &click([0, 63, 0], "up", [0.5, 64.0, 0.5], yaw, 0.0),
                ));
            }
        }
    }
    for ground in ["air", "water", "oak_slab", "glass"] {
        rows.push(row(&format!("bed-on-{ground}"), &blocks, vec![block([0, 63, 0], ground, &[])], &ON_TOP));
    }
    // Next to the edge of the world, the head would be outside the border.
    let edge = vec![block([29_999_983, 63, 0], "stone", &[])];
    let east = click([29_999_983, 63, 0], "up", [29_999_983.5, 64.0, 0.5], -90.0, 0.0);
    rows.push(row("bed-at-border-east", &blocks, edge, &east));
    let inside = vec![block([29_999_982, 63, 0], "stone", &[])];
    let east = click([29_999_982, 63, 0], "up", [29_999_982.5, 64.0, 0.5], -90.0, 0.0);
    rows.push(row("bed-inside-border-east", &blocks, inside, &east));
    rows
}
