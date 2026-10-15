//! The `shape/dimensions` cases.

mod entity_fit;
mod player_pose;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [entity_fit::cases(), player_pose::cases()].concat()
}
