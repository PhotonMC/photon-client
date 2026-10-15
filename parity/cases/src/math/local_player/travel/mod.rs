//! The `local_player/travel` cases.

mod by_medium;
mod fall_flying_dispatch;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [by_medium::cases(), fall_flying_dispatch::cases()].concat()
}
