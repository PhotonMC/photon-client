//! The `aabb/queries` cases.

mod contains_predicates;
mod distance_and_clip;
mod equality_and_sizes;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [contains_predicates::cases(), equality_and_sizes::cases(), distance_and_clip::cases()].concat()
}
