//! Sugar cane, cactus, bamboo, kelp and the vines, vines and lichen, scaffolding, rails and
//! redstone wire: what they stand on and the state a click gives them.

use super::facing::{FACES, face_click, step};
use super::*;

/// The four ways of looking along the ground.
const YAWS: [f32; 4] = [0.0, 90.0, 180.0, -90.0];

const HORIZONTAL: [&str; 4] = ["north", "east", "south", "west"];

/// What a stalk may stand on, and some it may not.
const STALK_GROUNDS: &[&str] = &[
    "sand",
    "red_sand",
    "dirt",
    "grass_block",
    "podzol",
    "mud",
    "gravel",
    "stone",
    "coarse_dirt",
    "suspicious_sand",
    "terracotta",
    "snow_block",
    "oak_slab",
    "glass",
];

pub fn cane_cactus_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for ground in STALK_GROUNDS {
        for (label, beside) in [
            ("none", None),
            ("water", Some(("water", "0"))),
            ("flowing", Some(("water", "3"))),
            ("lava", Some(("lava", "0"))),
            ("ice", Some(("frosted_ice", "0"))),
            ("stone", Some(("stone", "0"))),
        ] {
            let mut world = vec![block([0, 63, 0], ground, &[])];
            if let Some((name, level)) = beside {
                let properties: &[(&str, &str)] =
                    if name == "stone" || name == "frosted_ice" { &[] } else { &[("level", level)] };
                world.push(block([1, 63, 0], name, properties));
            }
            rows.push(row(&format!("cane-{ground}-{label}"), &["sugar_cane", "cactus"], world, &ON_TOP));
        }
    }
    // On a stalk of the same kind.
    for name in ["sugar_cane", "cactus"] {
        let world = vec![block([0, 63, 0], name, &[])];
        rows.push(row(&format!("{name}-on-itself"), &["sugar_cane", "cactus"], world, &ON_TOP));
    }
    // A cactus with something beside it, or above.
    for (label, name, pos) in [
        ("solid-side", "stone", [1, 64, 0]),
        ("lava-side", "lava", [0, 64, 1]),
        ("water-side", "water", [-1, 64, 0]),
        ("fence-side", "oak_fence", [0, 64, -1]),
        ("water-above", "water", [0, 65, 0]),
        ("stone-above", "stone", [0, 65, 0]),
    ] {
        let properties: &[(&str, &str)] = if name == "lava" || name == "water" { &[("level", "0")] } else { &[] };
        let world = vec![block([0, 63, 0], "sand", &[]), block(pos, name, properties)];
        rows.push(row(&format!("cactus-{label}"), &["cactus"], world, &ON_TOP));
    }
    rows
}

pub fn bamboo_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for ground in STALK_GROUNDS.iter().chain(&["bamboo_sapling", "bamboo", "moss_block", "farmland"]) {
        let properties: &[(&str, &str)] = if *ground == "bamboo" { &[("age", "1")] } else { &[] };
        let world = vec![block([0, 63, 0], ground, properties)];
        rows.push(row(&format!("bamboo-{ground}"), &["bamboo"], world, &ON_TOP));
    }
    for age in ["0", "1"] {
        let world = vec![block([0, 63, 0], "bamboo", &[("age", age)])];
        rows.push(row(&format!("bamboo-on-age{age}"), &["bamboo"], world, &ON_TOP));
        let world = vec![block([0, 63, 0], "dirt", &[]), block([0, 65, 0], "bamboo", &[("age", age)])];
        rows.push(row(&format!("bamboo-under-age{age}"), &["bamboo"], world, &ON_TOP));
    }
    for (label, name, level) in [("water", "water", "0"), ("flowing", "water", "3"), ("lava", "lava", "0")] {
        let world = vec![block([0, 63, 0], "dirt", &[]), block([0, 64, 0], name, &[("level", level)])];
        rows.push(row(&format!("bamboo-in-{label}"), &["bamboo"], world, &face_click([0, 63, 0], "up", 0.0, 0.0)));
    }
    rows
}

pub fn growing_plant_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    let plants = ["kelp", "weeping_vines", "twisting_vines", "cave_vines"];
    for ground in ["stone", "dirt", "sand", "magma_block", "ice", "glass", "oak_slab", "kelp", "kelp_plant"] {
        for (label, face, hit, fluid) in [
            ("up", "up", [0, 63, 0], Some("0")),
            ("up-dry", "up", [0, 63, 0], None),
            ("down", "down", [0, 65, 0], Some("0")),
            ("down-dry", "down", [0, 65, 0], None),
        ] {
            let mut world = vec![block(hit, ground, &[])];
            if let Some(level) = fluid {
                let at = [0, 64, 0];
                world.push(block(at, "water", &[("level", level)]));
            }
            rows.push(row(&format!("plants-{ground}-{label}"), &plants, world, &face_click(hit, face, 0.0, 0.0)));
        }
    }
    // In flowing water, and continuing a plant of the same kind.
    let world = vec![block([0, 63, 0], "stone", &[]), block([0, 64, 0], "water", &[("level", "3")])];
    rows.push(row("plants-flowing", &plants, world, &face_click([0, 63, 0], "up", 0.0, 0.0)));
    for (name, body, stem_at, ahead_at, face) in [
        ("kelp", "kelp_plant", [0, 63, 0], [0, 65, 0], "up"),
        ("weeping_vines", "weeping_vines_plant", [0, 65, 0], [0, 63, 0], "down"),
        ("twisting_vines", "twisting_vines_plant", [0, 63, 0], [0, 65, 0], "up"),
        ("cave_vines", "cave_vines_plant", [0, 65, 0], [0, 63, 0], "down"),
    ] {
        for ahead in [name, body] {
            let world = vec![
                block(stem_at, "stone", &[]),
                block([0, 64, 0], "water", &[("level", "0")]),
                block(ahead_at, ahead, &[]),
            ];
            rows.push(row(&format!("{name}-ahead-{ahead}"), &[name], world, &face_click(stem_at, face, 0.0, 0.0)));
        }
    }
    rows
}

/// Vines and lichen on every face of a stone block, from the four ways of looking.
pub fn multiface_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    let blocks = ["vine", "glow_lichen", "sculk_vein"];
    for face in FACES {
        for yaw in YAWS {
            for pitch in [-60.0, 0.0, 60.0] {
                let world = vec![block([0, 64, 0], "stone", &[])];
                let at = face_click([0, 64, 0], face, yaw, pitch);
                rows.push(row(&format!("multiface-{face}-yaw{yaw}-pitch{pitch}"), &blocks, world, &at));
            }
        }
    }
    // Against water, slabs, glass and a leaf block, the supports that are not full cubes.
    for support in ["glass", "oak_slab", "oak_leaves", "water", "ice", "oak_fence", "soul_sand"] {
        let properties: &[(&str, &str)] = if support == "water" { &[("level", "0")] } else { &[] };
        for face in ["up", "north"] {
            let world = vec![block([0, 64, 0], support, properties)];
            let at = face_click([0, 64, 0], face, 0.0, 0.0);
            rows.push(row(&format!("multiface-on-{support}-{face}"), &blocks, world, &at));
        }
    }
    // Adding another face to the one clicked, and clicking one with every face.
    let faces_of = |faces: &[&str]| -> Vec<(&'static str, &'static str)> {
        ["up", "north", "east", "south", "west"]
            .into_iter()
            .map(|face| (face, if faces.contains(&face) { "true" } else { "false" }))
            .collect()
    };
    for (label, faces) in [
        ("north", vec!["north"]),
        ("north-east", vec!["north", "east"]),
        ("all", vec!["up", "north", "east", "south", "west"]),
    ] {
        let world = vec![
            block([0, 64, 0], "stone", &[]),
            block([1, 65, 0], "stone", &[]),
            block([0, 65, 0], "vine", &faces_of(&faces)),
        ];
        for yaw in YAWS {
            let at = face_click([0, 64, 0], "up", yaw, 20.0);
            rows.push(row(&format!("vine-clicked-{label}-yaw{yaw}"), &["vine"], world.clone(), &at));
        }
    }
    // Hanging from a vine above, and in water.
    let world = vec![
        block([1, 66, 0], "stone", &[]),
        block([0, 66, 0], "vine", &[("east", "true")]),
        block([1, 65, 0], "stone", &[]),
    ];
    rows.push(row("vine-under-vine", &["vine"], world, &face_click([1, 65, 0], "west", 90.0, 0.0)));
    let world = vec![block([0, 64, 0], "stone", &[]), block([0, 65, 0], "water", &[("level", "0")])];
    rows.push(row("multiface-in-water", &blocks, world, &face_click([0, 64, 0], "up", 0.0, 0.0)));
    rows
}

pub fn scaffolding_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    for ground in ["stone", "oak_slab", "glass", "dirt_path", "water"] {
        let properties: &[(&str, &str)] = if ground == "water" { &[("level", "0")] } else { &[] };
        rows.push(row(
            &format!("scaffolding-on-{ground}"),
            &["scaffolding"],
            vec![block([0, 63, 0], ground, properties)],
            &ON_TOP,
        ));
    }
    for distance in 0..8 {
        let distance = distance.to_string();
        let world = vec![block([0, 63, 0], "scaffolding", &[("distance", &distance)])];
        rows.push(row(&format!("scaffolding-on-{distance}"), &["scaffolding"], world, &ON_TOP));
        let world = vec![
            block([0, 63, 0], "air", &[]),
            block([1, 64, 0], "scaffolding", &[("distance", &distance)]),
            block([0, 62, 0], "stone", &[]),
        ];
        rows.push(row(
            &format!("scaffolding-beside-{distance}"),
            &["scaffolding"],
            world,
            &face_click([0, 62, 0], "up", 0.0, 0.0),
        ));
    }
    let world = vec![block([0, 63, 0], "stone", &[]), block([0, 64, 0], "water", &[("level", "0")])];
    rows.push(row("scaffolding-in-water", &["scaffolding"], world, &face_click([0, 63, 0], "up", 0.0, 0.0)));
    rows
}

pub fn rail_rows() -> Vec<Value> {
    let rails = ["rail", "powered_rail", "detector_rail", "activator_rail"];
    let mut rows = Vec::new();
    for ground in ["stone", "oak_slab", "glass", "dirt", "snow", "oak_stairs", "hopper", "air"] {
        for yaw in YAWS {
            let world = vec![block([0, 63, 0], ground, &[])];
            let at = click([0, 63, 0], "up", [0.5, 64.0, 0.5], yaw, 0.0);
            rows.push(row(&format!("rails-on-{ground}-yaw{yaw}"), &rails, world, &at));
        }
    }
    let world = vec![block([0, 63, 0], "stone", &[]), block([0, 64, 0], "water", &[("level", "0")])];
    rows.push(row("rails-in-water", &rails, world, &face_click([0, 63, 0], "up", 0.0, 0.0)));
    rows
}

/// Redstone wire among sources, conductors and rails of every kind.
pub fn wire_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    let neighbours: [(&str, &[(&str, &str)]); 12] = [
        ("redstone_wire", &[]),
        ("stone", &[]),
        ("lever", &[("face", "floor")]),
        ("repeater", &[("facing", "north")]),
        ("repeater", &[("facing", "east")]),
        ("observer", &[("facing", "north")]),
        ("observer", &[("facing", "east")]),
        ("redstone_torch", &[]),
        ("redstone_block", &[]),
        ("oak_slab", &[]),
        ("glass", &[]),
        ("comparator", &[("facing", "north")]),
    ];
    for (name, properties) in neighbours {
        for direction in HORIZONTAL {
            let side = step(direction);
            let at = [side[0], 64, side[2]];
            let world = vec![block([0, 63, 0], "stone", &[]), block(at, name, properties)];
            let label = format!("wire-{name}-{direction}-{}", properties.first().map_or("", |p| p.1));
            rows.push(row(&label, &["redstone_wire"], world, &ON_TOP));
        }
    }
    // A wire climbing a block, or a step down.
    for (label, upper) in [("stone", "stone"), ("glass", "glass"), ("slab", "oak_slab"), ("trapdoor", "oak_trapdoor")] {
        for roof in [false, true] {
            let mut world = vec![
                block([0, 63, 0], "stone", &[]),
                block([1, 64, 0], upper, &[]),
                block([1, 65, 0], "redstone_wire", &[]),
            ];
            if roof {
                world.push(block([0, 65, 0], "stone", &[]));
            }
            rows.push(row(&format!("wire-climb-{label}-{roof}"), &["redstone_wire"], world, &ON_TOP));
        }
    }
    for lower in ["stone", "glass"] {
        let world = vec![
            block([0, 63, 0], "stone", &[]),
            block([1, 64, 0], "air", &[]),
            block([1, 63, 0], lower, &[]),
            block([1, 62, 0], "redstone_wire", &[]),
        ];
        rows.push(row(&format!("wire-step-down-{lower}"), &["redstone_wire"], world, &ON_TOP));
    }
    // The ground it stands on.
    for ground in ["stone", "glass", "oak_slab", "hopper", "soul_sand", "dirt_path", "air", "ice", "sand"] {
        let world = vec![block([0, 63, 0], ground, &[])];
        rows.push(row(&format!("wire-on-{ground}"), &["redstone_wire"], world, &ON_TOP));
    }
    rows
}
