//! Blocks that stack (candles, sea pickles, turtle eggs, petals, leaf litter) and the blocks that
//! decide for themselves whether they give way.

use super::*;

/// Each stacking block with the property that counts it.
fn stackers() -> Vec<(String, &'static str)> {
    let mut list: Vec<(String, &'static str)> = vec![("candle".to_owned(), "candles")];
    list.extend(["white", "red"].map(|color| (format!("{color}_candle"), "candles")));
    list.push(("sea_pickle".to_owned(), "pickles"));
    list.push(("turtle_egg".to_owned(), "eggs"));
    list.push(("pink_petals".to_owned(), "flower_amount"));
    list.push(("wildflowers".to_owned(), "flower_amount"));
    list.push(("leaf_litter".to_owned(), "segment_amount"));
    list
}

/// A click on the block at the origin's foot, on its top at the given height.
fn on_top_at(height: f64, yaw: f32) -> Click {
    click([0, 64, 0], "up", [0.5, 64.0 + height, 0.5], yaw, 0.0)
}

/// Putting one on a stack of itself, on another kind, and on the ground.
pub(super) fn stacking_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    let ground = block([0, 63, 0], "stone", &[]);
    for (name, stacked) in stackers() {
        let blocks = [name.as_str()];
        for amount in ["1", "2", "3", "4"] {
            for (side, face, at) in [("top", "up", [0.5, 64.5, 0.5]), ("side", "north", [0.5, 64.1, 0.0])] {
                for yaw in [0.0, 90.0, 180.0] {
                    let world = vec![ground.clone(), block([0, 64, 0], &name, &[(stacked, amount)])];
                    rows.push(row(
                        &format!("{name}-onto-{amount}-{side}-yaw{yaw}"),
                        &blocks,
                        world,
                        &click([0, 64, 0], face, at, yaw, 0.0),
                    ));
                }
            }
        }
        rows.extend(onto_rows(&name, &blocks, &ground));
        let waterlogged = vec![ground.clone(), block([0, 64, 0], "water", &[("level", "0")])];
        rows.push(row(&format!("{name}-into-water"), &blocks, waterlogged, &on_top_at(1.0, 0.0)));
    }
    rows
}

/// A block that decides for itself whether it gives way, clicked with blocks it holds no item of.
pub(super) fn replaced_rows() -> Vec<Value> {
    let blocks = ["stone", "oak_planks", "dirt"];
    let ground = block([0, 63, 0], "stone", &[]);
    let mut rows = Vec::new();
    let clicked: Vec<(&str, Vec<(&str, &str)>)> = vec![
        ("vine", vec![("up", "true")]),
        ("vine", vec![("north", "true"), ("east", "true"), ("south", "true"), ("west", "true"), ("up", "true")]),
        ("glow_lichen", vec![("north", "true")]),
        (
            "glow_lichen",
            vec![
                ("north", "true"),
                ("south", "true"),
                ("east", "true"),
                ("west", "true"),
                ("up", "true"),
                ("down", "true"),
            ],
        ),
        ("sculk_vein", vec![("up", "true")]),
        ("scaffolding", vec![]),
        ("pitcher_crop", vec![("half", "lower")]),
        ("kelp_plant", vec![]),
        ("weeping_vines_plant", vec![]),
        ("twisting_vines_plant", vec![]),
        ("cave_vines_plant", vec![]),
        ("candle", vec![("candles", "4")]),
        ("sea_pickle", vec![("pickles", "2")]),
        ("turtle_egg", vec![("eggs", "3")]),
        ("pink_petals", vec![("flower_amount", "4")]),
        ("leaf_litter", vec![("segment_amount", "1")]),
    ];
    for (name, properties) in clicked {
        let tag: Vec<&str> = properties.iter().map(|(_, value)| *value).collect();
        for (side, face, at) in [("top", "up", [0.5, 65.0, 0.5]), ("side", "north", [0.5, 64.5, 0.0])] {
            let world = vec![ground.clone(), block([0, 64, 0], name, &properties)];
            rows.push(row(
                &format!("replaced-{name}-{}-{side}", tag.join("-")),
                &blocks,
                world,
                &click([0, 64, 0], face, at, 0.0, 0.0),
            ));
        }
    }
    rows
}

/// Putting one onto other kinds of block, and onto what carries it or does not.
fn onto_rows(name: &str, blocks: &[&str], ground: &Value) -> Vec<Value> {
    let mut rows = Vec::new();
    // Onto a stack of another kind, or something that gives way.
    for other in ["pink_petals", "candle", "sea_pickle", "short_grass", "snow", "water", "stone", "oak_slab"] {
        let properties: Vec<(&str, &str)> = match other {
            "snow" => vec![("layers", "1")],
            "water" => vec![("level", "0")],
            "oak_slab" => vec![("type", "bottom")],
            _ => vec![],
        };
        let world = vec![ground.clone(), block([0, 64, 0], other, &properties)];
        rows.push(row(&format!("{name}-onto-{other}"), blocks, world, &on_top_at(1.0, 0.0)));
    }
    // On what carries it and what does not.
    for support in [
        "stone",
        "oak_slab",
        "oak_fence",
        "glass",
        "ice",
        "oak_leaves",
        "hopper",
        "cauldron",
        "oak_trapdoor",
        "sand",
        "water",
        "snow",
    ] {
        let properties: Vec<(&str, &str)> = match support {
            "oak_slab" => vec![("type", "top")],
            "snow" => vec![("layers", "5")],
            "water" => vec![("level", "0")],
            "oak_trapdoor" => vec![("open", "false"), ("half", "top")],
            _ => vec![],
        };
        let world = vec![block([0, 63, 0], support, &properties)];
        rows.push(row(&format!("{name}-on-{support}"), blocks, world, &ON_TOP));
    }
    rows
}
