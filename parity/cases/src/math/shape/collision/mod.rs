//! The `shape/collision` cases.

mod block_collisions;
mod collide_movement;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [collide_movement::cases(), block_collisions::cases()].concat()
}
