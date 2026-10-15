//! The `shape/world_collision` cases.

mod collider_sources;
mod stair_and_player;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [collider_sources::cases(), stair_and_player::cases()].concat()
}
