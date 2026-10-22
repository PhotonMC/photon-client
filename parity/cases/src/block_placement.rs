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

pub fn cases() -> Vec<Value> {
    vec![json!({
        "case_id": "block/placement-plants", "family": "block",
        "operation": "placement_rules", "rows": plant_rows(),
    })]
}
