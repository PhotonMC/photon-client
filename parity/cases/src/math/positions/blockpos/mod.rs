//! The `positions/blockpos` cases.

mod geometry_helpers;
mod mutable_helpers;
mod packed_and_relative;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [packed_and_relative::cases(), geometry_helpers::cases(), mutable_helpers::cases()].concat()
}
