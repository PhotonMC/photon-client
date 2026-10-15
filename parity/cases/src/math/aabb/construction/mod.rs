//! The `aabb/construction` cases.

mod builder_and_expand;
mod intersect_minmax;
mod move_and_endpoints;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [builder_and_expand::cases(), intersect_minmax::cases(), move_and_endpoints::cases()].concat()
}
