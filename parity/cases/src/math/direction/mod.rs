//! The `direction` cases.

mod nearest_and_axes;
mod properties_rotation;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [properties_rotation::cases(), nearest_and_axes::cases()].concat()
}
