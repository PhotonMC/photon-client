//! The `shape/rotation` cases.

mod rotate;
mod rotate_all;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [rotate::cases(), rotate_all::cases()].concat()
}
