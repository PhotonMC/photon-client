//! The `positions/sectionpos` cases.

mod offsets_and_ranges;
mod packed_accessors;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [packed_accessors::cases(), offsets_and_ranges::cases()].concat()
}
