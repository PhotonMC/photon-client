//! The jump key in the air with a glider worn: the glide starts on the press, the packet that
//! says so goes out, and the motion follows the glide until the player is told otherwise.

use serde_json::Value;

use super::breaking;
use crate::movement::player::scene::keys;

/// A press of the jump key, then `rest` ticks without it.
fn press_then(rest: usize) -> Vec<Value> {
    [keys(1, &["jump"]), keys(rest, &[])].concat()
}

pub fn rows() -> Vec<Value> {
    let glide = |label: &str, ticks: Vec<Value>| breaking(label, vec![], ticks).start(0.5, 100.0, 0.5);
    vec![
        glide("glide-level", press_then(40)).chest("minecraft:elytra").x_rot(0.0).row(),
        glide("glide-dive", press_then(40)).chest("minecraft:elytra").x_rot(45.0).row(),
        glide("glide-climb", press_then(40)).chest("minecraft:elytra").x_rot(-40.0).row(),
        glide("glide-turned", [keys(1, &["jump"]), keys(30, &["forward"])].concat())
            .chest("minecraft:elytra")
            .y_rot(135.0)
            .x_rot(10.0)
            .row(),
        glide("no-glider-falls", press_then(20)).x_rot(0.0).row(),
        // Pressing again in the air after a jump from the ground starts the glide there.
        breaking(
            "glide-after-jump",
            vec![],
            [keys(3, &[]), keys(1, &["jump"]), keys(2, &[]), keys(1, &["jump"]), keys(30, &[])].concat(),
        )
        .chest("minecraft:elytra")
        .start(0.5, 64.0, 0.5)
        .x_rot(5.0)
        .row(),
        // A rocket that flies with a glider pushes it along the look direction.
        glide("glide-rocket", press_then(20)).chest("minecraft:elytra").rockets(1).x_rot(20.0).row(),
        glide("glide-two-rockets", press_then(20)).chest("minecraft:elytra").rockets(2).y_rot(60.0).x_rot(-10.0).row(),
        // Without a glide the rocket only follows.
        glide("no-glider-rocket", press_then(10)).rockets(1).x_rot(0.0).row(),
        // Holding the key does not start a second glide: the press is an edge.
        glide("glide-held-jump", keys(30, &["jump"])).chest("minecraft:elytra").x_rot(20.0).row(),
    ]
}
