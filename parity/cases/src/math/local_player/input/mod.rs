//! The `local_player/input` cases.

mod fluid_and_climbing;
mod modifiers;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [modifiers::cases(), fluid_and_climbing::cases()].concat()
}
