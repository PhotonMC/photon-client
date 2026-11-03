//! Item frames and paintings (`HangingEntityItem.useOn`): the spot must be inside the build height,
//! a painting needs a side face, and a frame is hung only where it survives (a solid block or a
//! repeater behind it, nothing in its box).

use serde_json::Value;

use super::aim_at;
use super::item_on::{north, row, row_around};
use crate::movement::world::{put, put_with};

fn frame(label: &str, item: &'static str, world: Vec<Value>, aim: &Value) -> super::Interaction {
    row(label, world, aim).main_hand(item).count(3)
}

pub(super) fn rows() -> Vec<Value> {
    let stone = || vec![put(0, 64, 2, "minecraft:stone")];
    let up = aim_at((0, 64, 2), "up", (0.5, 65.0, 2.5));
    let down = aim_at((0, 64, 2), "down", (0.5, 64.0, 2.5));
    let east = aim_at((0, 64, 2), "east", (1.0, 64.5, 2.5));
    let mut rows = Vec::new();
    for item in ["item_frame", "glow_item_frame"] {
        let held: &'static str = Box::leak(format!("minecraft:{item}").into_boxed_str());
        let name = |what: &str| format!("{item}-{what}");
        rows.push(frame(&name("on-wall"), held, stone(), &north()).row());
        rows.push(frame(&name("on-floor"), held, stone(), &up).row());
        rows.push(frame(&name("on-ceiling"), held, stone(), &down).row());
        rows.push(frame(&name("on-wall-creative"), held, stone(), &north()).game_mode("creative").row());
        rows.push(frame(&name("on-wall-adventure"), held, stone(), &north()).game_mode("adventure").row());
        // Behind the frame is a block that does not hold it.
        let carpet = vec![put(0, 64, 2, "minecraft:white_carpet")];
        let carpet_side = aim_at((0, 64, 2), "north", (0.5, 64.03, 2.0));
        rows.push(frame(&name("on-carpet-side"), held, carpet, &carpet_side).row());
        // A repeater holds a frame on a side face only.
        let repeater = |world: Vec<Value>, aim: &Value, label: &str| {
            row_around(label, world, aim, &[(0, 64, 1), (0, 65, 2), (1, 64, 2)]).main_hand(held).count(3).row()
        };
        let diode = vec![put_with(
            0,
            64,
            2,
            "minecraft:repeater",
            &[("delay", "1"), ("facing", "north"), ("locked", "false"), ("powered", "false")],
        )];
        let diode_side = aim_at((0, 64, 2), "north", (0.5, 64.1, 2.0));
        rows.push(repeater(diode.clone(), &diode_side, &name("on-repeater-side")));
        rows.push(repeater(diode, &aim_at((0, 64, 2), "up", (0.5, 64.125, 2.5)), &name("on-repeater-top")));
        // A block in the frame's box.
        let mut crowded = stone();
        crowded.push(put(0, 64, 1, "minecraft:stone_slab"));
        rows.push(frame(&name("in-slab"), held, crowded, &north()).row());
        rows.push(frame(&name("on-side-east"), held, stone(), &east).row());
    }
    for (what, aim) in [("wall", north()), ("floor", up.clone()), ("ceiling", down.clone())] {
        let label = format!("painting-on-{what}");
        rows.push(frame(&label, "minecraft:painting", stone(), &aim).row());
    }
    rows.push(
        frame("painting-on-wall-adventure", "minecraft:painting", stone(), &north()).game_mode("adventure").row(),
    );
    rows
}
