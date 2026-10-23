//! Fences, iron bars, panes and walls: which neighbours they connect to when placed.

use super::*;

type Spec = (&'static str, Vec<(&'static str, &'static str)>);

/// What may stand beside a place: blocks that connect, blocks that do not, and the exceptions.
fn neighbours() -> Vec<Spec> {
    let mut list: Vec<Spec> = vec![
        ("stone", vec![]),
        ("oak_fence", vec![]),
        ("nether_brick_fence", vec![]),
        ("oak_leaves", vec![]),
        ("glass_pane", vec![]),
        ("iron_bars", vec![]),
        ("cobblestone_wall", vec![]),
        ("pumpkin", vec![]),
        ("carved_pumpkin", vec![]),
        ("jack_o_lantern", vec![]),
        ("melon", vec![]),
        ("barrier", vec![]),
        ("white_shulker_box", vec![]),
        ("oak_slab", vec![("type", "bottom")]),
        ("oak_slab", vec![("type", "double")]),
        ("oak_stairs", vec![]),
        ("oak_trapdoor", vec![("open", "false"), ("half", "bottom")]),
        ("glass", vec![]),
        ("ice", vec![]),
        ("hopper", vec![]),
        ("water", vec![("level", "0")]),
        ("short_grass", vec![]),
    ];
    for facing in ["north", "east"] {
        for open in ["false", "true"] {
            list.push(("oak_fence_gate", vec![("facing", facing), ("open", open)]));
        }
    }
    list
}

fn label(name: &str, properties: &[(&str, &str)]) -> String {
    let suffix: Vec<&str> = properties.iter().map(|(_, value)| *value).collect();
    if suffix.is_empty() { name.to_owned() } else { format!("{name}-{}", suffix.join("-")) }
}

const SIDES: [(&str, [i32; 3]); 4] =
    [("north", [0, 64, -1]), ("east", [1, 64, 0]), ("south", [0, 64, 1]), ("west", [-1, 64, 0])];

/// Each neighbour on each side of a place, and on all sides at once.
fn around(prefix: &str, blocks: &[&str], extra: &[Value]) -> Vec<Value> {
    let ground = block([0, 63, 0], "stone", &[]);
    let mut rows = Vec::new();
    for (name, properties) in neighbours() {
        let tag = label(name, &properties);
        for (side, at) in SIDES {
            let mut world = vec![ground.clone(), block(at, name, &properties)];
            world.extend(extra.iter().cloned());
            rows.push(row(&format!("{prefix}-{side}-{tag}"), blocks, world, &ON_TOP));
        }
        let mut world = vec![ground.clone()];
        world.extend(SIDES.iter().map(|(_, at)| block(*at, name, &properties)));
        world.extend(extra.iter().cloned());
        rows.push(row(&format!("{prefix}-all-{tag}"), blocks, world, &ON_TOP));
    }
    let mut world = vec![ground.clone(), block([0, 64, 0], "water", &[("level", "0")])];
    world.extend(extra.iter().cloned());
    rows.push(row(
        &format!("{prefix}-into-water"),
        blocks,
        world,
        &click([0, 64, 0], "up", [0.5, 65.0, 0.5], 0.0, 0.0),
    ));
    rows
}

/// Fences connect to their own kind, gates across them and whole faces.
pub(super) fn fence_rows() -> Vec<Value> {
    let mut fences: Vec<String> = WOODS.iter().map(|wood| format!("{wood}_fence")).collect();
    fences.push("nether_brick_fence".to_owned());
    around("fence", &names(&fences), &[])
}

/// Bars and panes connect to each other, walls and whole faces.
pub(super) fn bars_rows() -> Vec<Value> {
    let mut bars: Vec<String> = colored("stained_glass_pane");
    bars.extend(["glass_pane", "iron_bars", "copper_bars", "waxed_oxidized_copper_bars"].map(String::from));
    around("bars", &names(&bars), &[])
}

/// What may stand over a wall, which decides its post and the height of its arms.
fn tops() -> Vec<Spec> {
    vec![
        ("stone", vec![]),
        ("oak_slab", vec![("type", "bottom")]),
        ("oak_slab", vec![("type", "top")]),
        ("oak_slab", vec![("type", "double")]),
        ("cobblestone_wall", vec![("up", "true")]),
        ("cobblestone_wall", vec![("up", "false")]),
        ("oak_fence", vec![]),
        ("oak_fence_gate", vec![("open", "false")]),
        ("iron_bars", vec![]),
        ("lantern", vec![("hanging", "true")]),
        ("torch", vec![]),
        ("oak_pressure_plate", vec![]),
        ("stone_button", vec![("face", "ceiling")]),
        ("white_carpet", vec![]),
        ("snow", vec![("layers", "1")]),
        ("snow", vec![("layers", "8")]),
        ("oak_trapdoor", vec![("open", "false"), ("half", "bottom")]),
        ("oak_trapdoor", vec![("open", "false"), ("half", "top")]),
        ("oak_stairs", vec![("half", "bottom")]),
        ("oak_stairs", vec![("half", "top")]),
        ("iron_chain", vec![]),
        ("hopper", vec![]),
        ("short_grass", vec![]),
        ("end_rod", vec![]),
        ("flower_pot", vec![]),
        ("cobweb", vec![]),
        ("glass", vec![]),
        ("oak_leaves", vec![]),
        ("bell", vec![]),
        ("sea_pickle", vec![]),
    ]
}

/// Walls take a post and a height for each arm from what is beside and above them.
pub(super) fn wall_rows() -> Vec<Value> {
    let mut walls: Vec<String> =
        ["cobblestone", "mossy_cobblestone", "stone_brick", "brick", "sandstone", "deepslate_tile", "tuff"]
            .iter()
            .map(|kind| format!("{kind}_wall"))
            .collect();
    walls.push("blackstone_wall".to_owned());
    let blocks = names(&walls);
    let mut rows = around("wall", &blocks, &[]);
    let ground = block([0, 63, 0], "stone", &[]);
    for (name, properties) in tops() {
        let tag = label(name, &properties);
        for (sides, arms) in [
            ("none", Vec::new()),
            ("north", vec![[0, 64, -1]]),
            ("north-south", vec![[0, 64, -1], [0, 64, 1]]),
            ("east-west", vec![[1, 64, 0], [-1, 64, 0]]),
            ("north-east", vec![[0, 64, -1], [1, 64, 0]]),
            ("all", vec![[0, 64, -1], [0, 64, 1], [1, 64, 0], [-1, 64, 0]]),
        ] {
            let mut world = vec![ground.clone(), block([0, 65, 0], name, &properties)];
            world.extend(arms.iter().map(|at| block(*at, "stone", &[])));
            rows.push(row(&format!("wall-under-{tag}-{sides}"), &blocks, world, &ON_TOP));
        }
    }
    rows
}
