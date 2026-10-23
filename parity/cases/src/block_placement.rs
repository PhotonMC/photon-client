//! The `placement_rules` operation of the `block` family: blocks used on small scripted worlds.
//!
//! A row is a world of a few blocks, one click on it and the blocks to place there. The oracle
//! answers with what the game's block classes decide: whether the clicked block gives way,
//! whether the block may go there, the state it takes and whether it may stand.

use serde_json::{Value, json};

/// Plants that stand on what `VegetationBlock.mayPlaceOn` and its overrides accept.
const PLANTS: &[&str] = &[
    "oak_sapling",
    "spruce_sapling",
    "birch_sapling",
    "jungle_sapling",
    "acacia_sapling",
    "cherry_sapling",
    "dark_oak_sapling",
    "pale_oak_sapling",
    "short_grass",
    "fern",
    "bush",
    "azalea",
    "flowering_azalea",
    "dead_bush",
    "firefly_bush",
    "short_dry_grass",
    "tall_dry_grass",
    "dandelion",
    "golden_dandelion",
    "open_eyeblossom",
    "closed_eyeblossom",
    "poppy",
    "blue_orchid",
    "allium",
    "azure_bluet",
    "red_tulip",
    "orange_tulip",
    "white_tulip",
    "pink_tulip",
    "oxeye_daisy",
    "cornflower",
    "lily_of_the_valley",
    "wither_rose",
    "torchflower",
    "crimson_fungus",
    "warped_fungus",
    "crimson_roots",
    "warped_roots",
    "nether_sprouts",
    "pumpkin_stem",
    "melon_stem",
    "nether_wart",
    "sweet_berry_bush",
];

/// What a plant is put on.
const GROUNDS: &[&str] = &[
    "grass_block",
    "dirt",
    "coarse_dirt",
    "podzol",
    "mycelium",
    "rooted_dirt",
    "moss_block",
    "mud",
    "muddy_mangrove_roots",
    "farmland",
    "dirt_path",
    "stone",
    "sand",
    "red_sand",
    "terracotta",
    "crimson_nylium",
    "warped_nylium",
    "soul_sand",
    "soul_soil",
    "netherrack",
    "clay",
    "gravel",
    "snow_block",
    "ice",
    "glass",
    "oak_slab",
];

fn minecraft(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| format!("minecraft:{name}")).collect()
}

fn bits(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn bits32(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

/// A block of a scripted world: its position, name and the properties it differs in.
fn block(pos: [i32; 3], name: &str, properties: &[(&str, &str)]) -> Value {
    let properties: serde_json::Map<String, Value> =
        properties.iter().map(|(key, value)| ((*key).to_owned(), json!(value))).collect();
    json!({"pos": pos, "block": format!("minecraft:{name}"), "properties": properties})
}

/// One click: the aimed block, its face, where on it, and the way the player looks.
struct Click {
    hit: [i32; 3],
    face: &'static str,
    location: [f64; 3],
    yaw: f32,
    pitch: f32,
}

/// A click on the top of the block at the origin's foot, from the south, looking level.
const ON_TOP: Click = Click { hit: [0, 63, 0], face: "up", location: [0.5, 64.0, 0.5], yaw: 0.0, pitch: 0.0 };

fn row(label: &str, blocks: &[&str], world: Vec<Value>, click: &Click) -> Value {
    json!({
        "label": label,
        "blocks": minecraft(blocks),
        "world": world,
        "hit": click.hit,
        "face": click.face,
        "location": click.location.map(bits),
        "yaw": bits32(click.yaw),
        "pitch": bits32(click.pitch),
    })
}

/// The plants on every ground, beside a plant of their own kind, in a snow layer and over
/// nothing.
fn plant_rows() -> Vec<Value> {
    let mut rows: Vec<Value> = GROUNDS
        .iter()
        .map(|ground| row(&format!("plant-on-{ground}"), PLANTS, vec![block([0, 63, 0], ground, &[])], &ON_TOP))
        .collect();
    // Replacing what is clicked: grass gives way to a flower but not to grass itself.
    rows.push(row(
        "plant-into-short-grass",
        PLANTS,
        vec![block([0, 63, 0], "grass_block", &[]), block([0, 64, 0], "short_grass", &[])],
        &Click { hit: [0, 64, 0], location: [0.5, 65.0, 0.5], ..ON_TOP },
    ));
    rows.push(row(
        "plant-into-snow-layer",
        PLANTS,
        vec![block([0, 63, 0], "grass_block", &[]), block([0, 64, 0], "snow", &[("layers", "1")])],
        &Click { hit: [0, 64, 0], location: [0.5, 64.125, 0.5], ..ON_TOP },
    ));
    // Against the side of a wall, with nothing below.
    rows.push(row(
        "plant-on-air",
        PLANTS,
        vec![block([0, 64, -1], "stone", &[])],
        &Click { hit: [0, 64, -1], face: "south", location: [0.5, 64.5, 0.0], ..ON_TOP },
    ));
    rows
}

const COLORS: &[&str] = &[
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];

const WOODS: &[&str] = &[
    "oak", "spruce", "birch", "jungle", "acacia", "cherry", "dark_oak", "pale_oak", "mangrove", "bamboo", "crimson",
    "warped",
];

fn colored(suffix: &str) -> Vec<String> {
    COLORS.iter().map(|color| format!("{color}_{suffix}")).collect()
}

fn names(list: &[String]) -> Vec<&str> {
    list.iter().map(String::as_str).collect()
}

/// A click on the face of the block at `hit`, ground below the origin.
fn click(hit: [i32; 3], face: &'static str, location: [f64; 3], yaw: f32, pitch: f32) -> Click {
    Click { hit, face, location, yaw, pitch }
}

/// Carpets stand on anything but air, whatever it is.
fn carpet_rows() -> Vec<Value> {
    let carpets = colored("carpet");
    let blocks = names(&carpets);
    let mut rows = Vec::new();
    for ground in ["stone", "water", "oak_fence", "snow", "grass_block"] {
        rows.push(row(&format!("carpet-on-{ground}"), &blocks, vec![block([0, 63, 0], ground, &[])], &ON_TOP));
    }
    rows.push(row(
        "carpet-on-air",
        &blocks,
        vec![block([0, 64, -1], "stone", &[])],
        &click([0, 64, -1], "south", [0.5, 64.5, 0.0], 0.0, 0.0),
    ));
    rows.push(row(
        "carpet-on-carpet",
        &blocks,
        vec![block([0, 63, 0], "stone", &[]), block([0, 64, 0], "white_carpet", &[])],
        &click([0, 64, 0], "up", [0.5, 64.0625, 0.5], 0.0, 0.0),
    ));
    rows
}

/// Buttons and the lever hang from the face they are put on, or from another one when the way
/// the player looks says so and that one is full.
fn attached_rows() -> Vec<Value> {
    let mut wood: Vec<String> = WOODS.iter().map(|wood| format!("{wood}_button")).collect();
    wood.extend(["stone_button", "polished_blackstone_button", "lever"].map(String::from));
    let blocks = names(&wood);
    let mut rows = Vec::new();
    for yaw in [0.0, 90.0, 180.0, -90.0, 45.0, 135.0] {
        for pitch in [0.0, 60.0, -60.0, 30.0, -30.0, 89.0] {
            let looking = format!("yaw{yaw}-pitch{pitch}");
            let floor = vec![block([0, 63, 0], "stone", &[])];
            let top = click([0, 63, 0], "up", [0.5, 64.0, 0.5], yaw, pitch);
            rows.push(row(&format!("top-{looking}"), &blocks, floor.clone(), &top));
            let wall = vec![block([0, 63, 0], "stone", &[]), block([0, 64, -1], "stone", &[])];
            let side = click([0, 64, -1], "south", [0.5, 64.5, 0.0], yaw, pitch);
            rows.push(row(&format!("wall-{looking}"), &blocks, wall, &side));
            let ceiling = vec![block([0, 65, 0], "stone", &[])];
            let under = click([0, 65, 0], "down", [0.5, 65.0, 0.5], yaw, pitch);
            rows.push(row(&format!("ceiling-{looking}"), &blocks, ceiling, &under));
        }
    }
    // What is clicked decides whether it can be hung from.
    for support in ["oak_fence", "oak_slab", "oak_stairs", "glass", "iron_chain", "ice", "oak_leaves", "hopper"] {
        let world = vec![block([0, 64, -1], support, &[])];
        let side = click([0, 64, -1], "south", [0.5, 64.5, 0.0], 0.0, 0.0);
        rows.push(row(&format!("wall-of-{support}"), &blocks, world, &side));
        let world = vec![block([0, 63, 0], support, &[])];
        rows.push(row(
            &format!("top-of-{support}"),
            &blocks,
            world,
            &click([0, 63, 0], "up", [0.5, 64.0, 0.5], 0.0, 0.0),
        ));
    }
    // Into what gives way, the player's own look decides.
    for pitch in [0.0, 70.0, -70.0] {
        let world = vec![block([0, 63, 0], "stone", &[]), block([0, 64, 0], "short_grass", &[])];
        rows.push(row(
            &format!("into-grass-pitch{pitch}"),
            &blocks,
            world,
            &click([0, 64, 0], "up", [0.5, 65.0, 0.5], 0.0, pitch),
        ));
    }
    rows
}

/// Trap doors take their side and half from the face and the look, and open when powered.
fn trapdoor_rows() -> Vec<Value> {
    let mut trapdoors: Vec<String> = WOODS.iter().map(|wood| format!("{wood}_trapdoor")).collect();
    trapdoors.push("iron_trapdoor".to_owned());
    for weathering in ["", "exposed_", "weathered_", "oxidized_"] {
        trapdoors.push(format!("{weathering}copper_trapdoor"));
        trapdoors.push(format!("waxed_{weathering}copper_trapdoor"));
    }
    let blocks = names(&trapdoors);
    let mut rows = Vec::new();
    for yaw in [0.0, 90.0, 180.0, -90.0] {
        let ground = vec![block([0, 63, 0], "stone", &[])];
        let top = click([0, 63, 0], "up", [0.5, 64.0, 0.5], yaw, 0.0);
        rows.push(row(&format!("top-yaw{yaw}"), &blocks, ground, &top));
        let ceiling = vec![block([0, 65, 0], "stone", &[])];
        let under = click([0, 65, 0], "down", [0.5, 65.0, 0.5], yaw, 0.0);
        rows.push(row(&format!("under-yaw{yaw}"), &blocks, ceiling, &under));
        let grass = vec![block([0, 63, 0], "stone", &[]), block([0, 64, 0], "short_grass", &[])];
        let into = click([0, 64, 0], "up", [0.5, 65.0, 0.5], yaw, 0.0);
        rows.push(row(&format!("into-grass-yaw{yaw}"), &blocks, grass, &into));
    }
    for face in ["north", "south", "east", "west"] {
        for height in [0.25, 0.5, 0.75] {
            let (x, z) = match face {
                "north" => (0.5, 0.0),
                "south" => (0.5, 1.0),
                "east" => (1.0, 0.5),
                _ => (0.0, 0.5),
            };
            let wall = vec![block([0, 64, 0], "stone", &[])];
            rows.push(row(
                &format!("side-{face}-{height}"),
                &blocks,
                wall,
                &click([0, 64, 0], face, [x, 64.0 + height, z], 0.0, 0.0),
            ));
        }
    }
    // Water that fills the place makes it waterlogged, flowing water does not.
    for (label, level) in [("source", "0"), ("flowing", "3"), ("falling", "8")] {
        let world = vec![block([0, 63, 0], "stone", &[]), block([0, 64, 0], "water", &[("level", level)])];
        let into = click([0, 64, 0], "up", [0.5, 65.0, 0.5], 0.0, 0.0);
        rows.push(row(&format!("into-water-{label}"), &blocks, world, &into));
    }
    // Blocks that carry no signal beside the place do not power it.
    let beside = vec![block([0, 63, 0], "stone", &[]), block([1, 64, 0], "stone", &[]), block([0, 65, 0], "dirt", &[])];
    rows.push(row("beside-solid-blocks", &blocks, beside, &ON_TOP));
    rows
}

/// Concrete powder turns to concrete where it meets water, even by the side.
fn concrete_rows() -> Vec<Value> {
    let powders = colored("concrete_powder");
    let blocks = names(&powders);
    let ground = block([0, 63, 0], "stone", &[]);
    let mut rows = vec![row("dry", &blocks, vec![ground.clone()], &ON_TOP)];
    for (label, at) in [
        ("east", [1, 64, 0]),
        ("west", [-1, 64, 0]),
        ("north", [0, 64, -1]),
        ("south", [0, 64, 1]),
        ("above", [0, 65, 0]),
        ("diagonal", [1, 64, 1]),
    ] {
        let world = vec![ground.clone(), block(at, "water", &[("level", "0")])];
        rows.push(row(&format!("water-{label}"), &blocks, world, &ON_TOP));
    }
    for level in ["0", "1", "7", "8"] {
        let world = vec![ground.clone(), block([0, 64, 0], "water", &[("level", level)])];
        let into = click([0, 63, 0], "up", [0.5, 64.0, 0.5], 0.0, 0.0);
        rows.push(row(&format!("into-water-level{level}"), &blocks, world, &into));
    }
    // Water under the place is only read when the place itself holds water.
    let below = vec![block([0, 62, 0], "water", &[("level", "0")]), block([1, 63, 0], "stone", &[])];
    rows.push(row("water-below", &blocks, below, &click([1, 63, 0], "west", [1.0, 63.5, 0.5], 0.0, 0.0)));
    let wall = vec![ground.clone(), block([1, 64, 0], "water", &[("level", "0")]), block([0, 64, -1], "stone", &[])];
    rows.push(row("water-east-wall-north", &blocks, wall, &click([0, 64, -1], "south", [0.5, 64.5, 0.0], 0.0, 0.0)));
    rows.push(row("lava-east", &blocks, vec![ground.clone(), block([1, 64, 0], "lava", &[("level", "0")])], &ON_TOP));
    rows.push(row("sturdy-east", &blocks, vec![ground, block([1, 64, 0], "oak_slab", &[("type", "double")])], &ON_TOP));
    rows
}

/// Glazed terracotta faces the player.
fn glazed_rows() -> Vec<Value> {
    let glazed = colored("glazed_terracotta");
    let blocks = names(&glazed);
    [0.0, 90.0, 180.0, -90.0, 44.0, 46.0, 360.0, -721.5]
        .iter()
        .map(|yaw| {
            let ground = vec![block([0, 63, 0], "stone", &[])];
            row(&format!("yaw{yaw}"), &blocks, ground, &click([0, 63, 0], "up", [0.5, 64.0, 0.5], *yaw, 20.0))
        })
        .collect()
}

mod signals;
mod supports;

pub fn cases() -> Vec<Value> {
    let case = |label: &str, rows: Vec<Value>| {
        json!({
            "case_id": format!("block/placement-{label}"), "family": "block",
            "operation": "placement_rules", "rows": rows,
        })
    };
    vec![
        case("plants", plant_rows()),
        case("carpets", carpet_rows()),
        case("attached", attached_rows()),
        case("trapdoors", trapdoor_rows()),
        case("concrete-powder", concrete_rows()),
        case("glazed-terracotta", glazed_rows()),
        case("pressure-plates", supports::plate_rows()),
        case("lanterns-bells", supports::lantern_bell_rows()),
        case("diodes", supports::diode_rows()),
        case("trapdoor-signals", signals::signal_rows()),
        case("cactus-flower", supports::cactus_flower_rows()),
    ]
}
