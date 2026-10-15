//! The `vec3/rotation` cases.

mod clockwise_and_accessors;
mod rotate_directions;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [clockwise_and_accessors::cases(), rotate_directions::cases()].concat()
}
