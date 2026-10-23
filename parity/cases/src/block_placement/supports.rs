//! Blocks with a small foot: what they stand on or hang from.

use super::*;

/// What a block with a small foot may be put on or hung from: full blocks, partial ones whose
/// face still covers the centre or the rim, and ones that do not.
const SUPPORTS: &[(&str, &[(&str, &str)])] = &[
    ("stone", &[]),
    ("oak_fence", &[]),
    ("oak_slab", &[("type", "bottom")]),
    ("oak_slab", &[("type", "top")]),
    ("oak_stairs", &[]),
    ("glass", &[]),
    ("glass_pane", &[]),
    ("iron_bars", &[]),
    ("iron_chain", &[]),
    ("hopper", &[]),
    ("cauldron", &[]),
    ("composter", &[]),
    ("soul_sand", &[]),
    ("snow", &[("layers", "4")]),
    ("snow", &[("layers", "8")]),
    ("oak_leaves", &[]),
    ("ice", &[]),
    ("oak_trapdoor", &[("half", "top"), ("open", "false")]),
    ("oak_trapdoor", &[("half", "bottom"), ("open", "false")]),
    ("cobblestone_wall", &[]),
    ("oak_fence_gate", &[]),
    ("lantern", &[]),
    ("stone_pressure_plate", &[]),
    ("white_carpet", &[]),
    ("farmland", &[]),
    ("dirt_path", &[]),
    ("flower_pot", &[]),
    ("bell", &[]),
    ("lightning_rod", &[]),
    ("end_rod", &[]),
];

/// The support-name label of an entry.
fn support_label(name: &str, properties: &[(&str, &str)]) -> String {
    let suffix: Vec<String> = properties.iter().map(|(_, value)| (*value).to_owned()).collect();
    if suffix.is_empty() { name.to_owned() } else { format!("{name}-{}", suffix.join("-")) }
}

/// Blocks with a small foot: put on top of each support, hung from the underside of each, and
/// put against the side of one.
fn foot_rows(label: &str, blocks: &[&str], top: bool, ceiling: bool) -> Vec<Value> {
    let mut rows = Vec::new();
    for (name, properties) in SUPPORTS {
        let tag = support_label(name, properties);
        if top {
            let world = vec![block([0, 63, 0], name, properties)];
            rows.push(row(&format!("{label}-on-{tag}"), blocks, world, &ON_TOP));
        }
        if ceiling {
            let world = vec![block([0, 65, 0], name, properties)];
            let under = click([0, 65, 0], "down", [0.5, 65.0, 0.5], 0.0, 0.0);
            rows.push(row(&format!("{label}-under-{tag}"), blocks, world, &under));
        }
    }
    rows
}

/// Pressure plates stand on a rigid or a centred support.
pub(super) fn plate_rows() -> Vec<Value> {
    let mut plates: Vec<String> = WOODS.iter().map(|wood| format!("{wood}_pressure_plate")).collect();
    plates.extend(
        ["stone", "polished_blackstone", "light_weighted", "heavy_weighted"]
            .map(|kind| format!("{kind}_pressure_plate")),
    );
    foot_rows("plate", &names(&plates), true, false)
}

/// Lanterns stand or hang by the look, bells hang from the face they are put on.
pub(super) fn lantern_bell_rows() -> Vec<Value> {
    let blocks = ["lantern", "soul_lantern", "copper_lantern", "bell", "spore_blossom"];
    let mut rows = foot_rows("hung", &blocks, true, true);
    for pitch in [0.0, 60.0, -60.0] {
        for yaw in [0.0, 90.0, 225.0] {
            let looking = format!("yaw{yaw}-pitch{pitch}");
            let wall = vec![block([0, 64, -1], "stone", &[])];
            rows.push(row(
                &format!("wall-{looking}"),
                &blocks,
                wall,
                &click([0, 64, -1], "south", [0.5, 64.5, 0.0], yaw, pitch),
            ));
            let both = vec![block([-1, 64, 0], "stone", &[]), block([1, 64, 0], "stone", &[])];
            rows.push(row(
                &format!("between-{looking}"),
                &blocks,
                both,
                &click([-1, 64, 0], "east", [0.0, 64.5, 0.5], yaw, pitch),
            ));
            let north_south = vec![block([0, 64, -1], "stone", &[]), block([0, 64, 1], "stone", &[])];
            rows.push(row(
                &format!("between-z-{looking}"),
                &blocks,
                north_south,
                &click([0, 64, -1], "south", [0.5, 64.5, 0.0], yaw, pitch),
            ));
            let floor = vec![block([0, 63, 0], "stone", &[]), block([0, 64, -1], "oak_fence", &[])];
            rows.push(row(
                &format!("fence-wall-{looking}"),
                &blocks,
                floor,
                &click([0, 64, -1], "south", [0.5, 64.5, 0.0], yaw, pitch),
            ));
        }
    }
    let waterlogged = vec![block([0, 63, 0], "stone", &[]), block([0, 64, 0], "water", &[("level", "0")])];
    rows.push(row("into-water", &blocks, waterlogged, &click([0, 64, 0], "up", [0.5, 65.0, 0.5], 0.0, 0.0)));
    rows
}

/// Repeaters and comparators face the player; a diode beside a repeater may lock it.
pub(super) fn diode_rows() -> Vec<Value> {
    let blocks = ["repeater", "comparator"];
    let mut rows = foot_rows("diode", &blocks, true, false);
    for yaw in [0.0, 90.0, 180.0, -90.0] {
        let world = vec![block([0, 63, 0], "stone", &[])];
        rows.push(row(&format!("yaw{yaw}"), &blocks, world, &click([0, 63, 0], "up", [0.5, 64.0, 0.5], yaw, 0.0)));
    }
    let beside =
        vec![block([0, 63, 0], "stone", &[]), block([1, 64, 0], "stone", &[]), block([-1, 64, 0], "dirt", &[])];
    rows.push(row("beside-solid", &blocks, beside, &ON_TOP));
    rows
}

/// A cactus flower stands on a centred support or one of its override tag.
pub(super) fn cactus_flower_rows() -> Vec<Value> {
    let mut rows = foot_rows("cactus-flower", &["cactus_flower"], true, false);
    for ground in ["cactus", "farmland", "sand", "red_sand"] {
        rows.push(row(
            &format!("cactus-flower-on-{ground}"),
            &["cactus_flower"],
            vec![block([0, 63, 0], ground, &[])],
            &ON_TOP,
        ));
    }
    rows
}
