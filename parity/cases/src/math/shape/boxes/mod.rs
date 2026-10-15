//! The `shape/boxes` cases.

mod basic_shapes;
mod encompassing_equality;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [basic_shapes::cases(), encompassing_equality::cases()].concat()
}
