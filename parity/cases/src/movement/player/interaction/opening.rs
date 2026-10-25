//! The use key on doors, trap doors, fence gates and buttons with an empty or full hand: the
//! block the click sets and the sound the server leaves the user out of.

use serde_json::Value;

use super::placing::uses;
use super::{aim_at, using};
use crate::movement::world::{put, put_with};

fn door(block: &str, half: &str, open: bool, y: i64) -> Value {
    put_with(
        0,
        y,
        2,
        block,
        &[
            ("facing", "south"),
            ("half", half),
            ("hinge", "left"),
            ("open", if open { "true" } else { "false" }),
            ("powered", "false"),
        ],
    )
}

fn door_world(block: &str, open: bool) -> Vec<Value> {
    vec![door(block, "lower", open, 64), door(block, "upper", open, 65)]
}

fn trapdoor(block: &str, open: bool) -> Vec<Value> {
    vec![put_with(
        0,
        64,
        2,
        block,
        &[
            ("facing", "north"),
            ("half", "bottom"),
            ("open", if open { "true" } else { "false" }),
            ("powered", "false"),
            ("waterlogged", "false"),
        ],
    )]
}

fn gate(block: &str, facing: &str, open: bool) -> Vec<Value> {
    vec![put_with(
        0,
        64,
        2,
        block,
        &[
            ("facing", facing),
            ("in_wall", "false"),
            ("open", if open { "true" } else { "false" }),
            ("powered", "false"),
        ],
    )]
}

fn button(block: &str, powered: bool) -> Vec<Value> {
    vec![put_with(
        0,
        64,
        2,
        block,
        &[("face", "floor"), ("facing", "north"), ("powered", if powered { "true" } else { "false" })],
    )]
}

fn comparator(mode: &str) -> Vec<Value> {
    vec![put_with(0, 64, 2, "minecraft:comparator", &[("facing", "north"), ("mode", mode), ("powered", "false")])]
}

/// A use of the block at (0, 64, 2) with a block in hand: a pass lets the block go down in front
/// of the face, a success or consume does not.
fn hand_row(label: &str, world: Vec<Value>, aim: &Value) -> super::Interaction {
    let around = [(0, 64, 1), (0, 64, 2), (0, 65, 2), (0, 64, 3)];
    let ticks = [uses(1, Some(aim), &around).clicks(1).go(), uses(3, Some(aim), &around).go()].concat();
    using(label, world, ticks).main_hand("minecraft:stone").count(4)
}

fn north() -> Value {
    aim_at((0, 64, 2), "north", (0.5, 64.5, 2.0))
}

fn bed_and_plants() -> Vec<Value> {
    let bed = |part: &str, z: i64| {
        put_with(0, 64, z, "minecraft:red_bed", &[("facing", "south"), ("occupied", "false"), ("part", part)])
    };
    let vines = |block: &str, props: &[(&str, &str)]| vec![put_with(0, 64, 2, block, props)];
    let top = aim_at((0, 64, 2), "up", (0.5, 65.0, 2.5));
    vec![
        hand_row(
            "bed-head-server-answers",
            vec![bed("foot", 1), bed("head", 2)],
            &aim_at((0, 64, 2), "up", (0.5, 64.5625, 2.5)),
        )
        .row(),
        hand_row("fence-passes", vec![put(0, 64, 2, "minecraft:oak_fence")], &north()).row(),
        hand_row("light-consumes", vines("minecraft:light", &[("level", "7"), ("waterlogged", "false")]), &north())
            .row(),
        hand_row(
            "cave-vines-with-berries",
            vines("minecraft:cave_vines", &[("age", "3"), ("berries", "true")]),
            &north(),
        )
        .row(),
        hand_row(
            "cave-vines-without-berries",
            vines("minecraft:cave_vines", &[("age", "3"), ("berries", "false")]),
            &north(),
        )
        .row(),
        hand_row(
            "cave-vines-plant-with-berries",
            vines("minecraft:cave_vines_plant", &[("berries", "true")]),
            &north(),
        )
        .row(),
        hand_row("daylight-detector-builds", vec![put(0, 64, 2, "minecraft:daylight_detector")], &top).row(),
        hand_row("daylight-detector-adventure", vec![put(0, 64, 2, "minecraft:daylight_detector")], &top)
            .game_mode("adventure")
            .row(),
    ]
}

fn repeaters() -> Vec<Value> {
    let repeater = |delay: &str| {
        vec![put_with(
            0,
            64,
            2,
            "minecraft:repeater",
            &[("delay", delay), ("facing", "north"), ("locked", "false"), ("powered", "false")],
        )]
    };
    let on_top = aim_at((0, 64, 2), "up", (0.5, 64.125, 2.5));
    vec![
        hand_row("repeater-delay-1", repeater("1"), &on_top).row(),
        hand_row("repeater-delay-4", repeater("4"), &on_top).row(),
        hand_row("repeater-adventure", repeater("2"), &on_top).game_mode("adventure").row(),
    ]
}

fn bells() -> Vec<Value> {
    let bell = |attachment: &str| {
        vec![put_with(
            0,
            64,
            2,
            "minecraft:bell",
            &[("attachment", attachment), ("facing", "north"), ("powered", "false")],
        )]
    };
    let east = aim_at((0, 64, 2), "east", (1.0, 64.5, 2.5));
    let high = aim_at((0, 64, 2), "north", (0.5, 64.9, 2.0));
    vec![
        hand_row("bell-floor-front", bell("floor"), &north()).row(),
        hand_row("bell-floor-side", bell("floor"), &east).row(),
        hand_row("bell-floor-high", bell("floor"), &high).row(),
        hand_row("bell-wall-front", bell("single_wall"), &north()).row(),
        hand_row("bell-wall-side", bell("single_wall"), &east).row(),
        hand_row("bell-ceiling-side", bell("ceiling"), &east).row(),
    ]
}

/// These open a screen or act on the server; the client's use ends in a success.
fn server_work() -> Vec<Value> {
    [
        "chest",
        "ender_chest",
        "crafting_table",
        "furnace",
        "anvil",
        "hopper",
        "barrel",
        "shulker_box",
        "dragon_egg",
        "beacon",
        "brewing_stand",
        "stonecutter",
        "dispenser",
        "enchanting_table",
        "loom",
        "grindstone",
        "smithing_table",
        "cartography_table",
        "crafter",
        "lever",
    ]
    .iter()
    .map(|block| {
        let world = vec![put(0, 64, 2, &format!("minecraft:{block}"))];
        hand_row(&format!("server-work-{block}"), world, &north()).row()
    })
    .collect()
}

fn hand_rows() -> Vec<Value> {
    [bed_and_plants(), repeaters(), bells(), server_work()].concat()
}

pub fn rows() -> Vec<Value> {
    let around = [(0, 64, 2), (0, 65, 2)];
    let at_door = aim_at((0, 64, 2), "north", (0.5, 64.5, 2.0));
    let on_top = aim_at((0, 64, 2), "up", (0.5, 64.125, 2.5));
    let at_upper = aim_at((0, 65, 2), "north", (0.5, 65.5, 2.0));
    let run = |aim: &Value| [uses(1, Some(aim), &around).clicks(1).go(), uses(3, Some(aim), &around).go()].concat();
    let row = |label: &str, world: Vec<Value>, aim: &Value| using(label, world, run(aim));
    let mut rows = vec![
        row("door-oak-opens", door_world("minecraft:oak_door", false), &at_door).row(),
        row("door-oak-closes", door_world("minecraft:oak_door", true), &at_door).row(),
        row("door-oak-upper-half", door_world("minecraft:oak_door", false), &at_upper).row(),
        row("door-iron-passes", door_world("minecraft:iron_door", false), &at_door).row(),
        row("door-copper-opens", door_world("minecraft:copper_door", false), &at_door).row(),
        row("door-waxed-weathered-copper", door_world("minecraft:waxed_weathered_copper_door", false), &at_door).row(),
        row("door-bamboo-opens", door_world("minecraft:bamboo_door", false), &at_door).row(),
        row("door-cherry-closes", door_world("minecraft:cherry_door", true), &at_door).row(),
        row("door-crimson-opens", door_world("minecraft:crimson_door", false), &at_door).row(),
        row("door-opens-with-block-held", door_world("minecraft:oak_door", false), &at_door)
            .main_hand("minecraft:stone")
            .count(4)
            .row(),
        row("door-opens-in-adventure", door_world("minecraft:oak_door", false), &at_door).game_mode("adventure").row(),
        row("trapdoor-oak-opens", trapdoor("minecraft:oak_trapdoor", false), &at_door).row(),
        row("trapdoor-oak-closes", trapdoor("minecraft:oak_trapdoor", true), &at_door).row(),
        row("trapdoor-iron-passes", trapdoor("minecraft:iron_trapdoor", false), &at_door).row(),
        row("trapdoor-copper-opens", trapdoor("minecraft:copper_trapdoor", false), &at_door).row(),
        row("trapdoor-nether-opens", trapdoor("minecraft:warped_trapdoor", false), &at_door).row(),
        row("gate-oak-opens-turned", gate("minecraft:oak_fence_gate", "north", false), &at_door).row(),
        row("gate-oak-opens-kept", gate("minecraft:oak_fence_gate", "east", false), &at_door).row(),
        row("gate-oak-closes", gate("minecraft:oak_fence_gate", "north", true), &at_door).row(),
        row("gate-bamboo-opens", gate("minecraft:bamboo_fence_gate", "south", false), &at_door).row(),
        row("gate-cherry-opens", gate("minecraft:cherry_fence_gate", "south", false), &at_door).row(),
        row("gate-crimson-closes", gate("minecraft:crimson_fence_gate", "south", true), &at_door).row(),
        row("button-oak-presses", button("minecraft:oak_button", false), &at_door).row(),
        row("button-stone-presses", button("minecraft:stone_button", false), &at_door).row(),
        row("button-polished-blackstone", button("minecraft:polished_blackstone_button", false), &at_door).row(),
        row("button-bamboo-presses", button("minecraft:bamboo_button", false), &at_door).row(),
        row("button-cherry-presses", button("minecraft:cherry_button", false), &at_door).row(),
        row("button-warped-presses", button("minecraft:warped_button", false), &at_door).row(),
        row("comparator-to-subtract", comparator("compare"), &on_top).row(),
        row("comparator-to-compare", comparator("subtract"), &on_top).row(),
        row("comparator-adventure-passes", comparator("compare"), &on_top).game_mode("adventure").row(),
        row("button-powered-consumes", button("minecraft:oak_button", true), &at_door).row(),
    ];
    rows.extend(hand_rows());
    rows
}
