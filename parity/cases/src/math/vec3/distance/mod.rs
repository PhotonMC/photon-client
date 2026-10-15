//! The `vec3/distance` cases.

mod dot_and_distance;
mod to_string;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [dot_and_distance::cases(), to_string::cases()].concat()
}
