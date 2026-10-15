//! The `client_world/input` cases.

mod bindings;
mod sprint_window;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [bindings::cases(), sprint_window::cases()].concat()
}
