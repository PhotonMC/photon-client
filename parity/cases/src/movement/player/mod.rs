//! The movement family: the player and the world.

mod containers;
mod interaction;
mod light;
mod rides;
mod scene;
mod ticks;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [
        containers::cases(),
        interaction::cases(),
        vec![light::light_changes()],
        rides::cases(),
        vec![ticks::local_player_ticks()],
    ]
    .concat()
}
