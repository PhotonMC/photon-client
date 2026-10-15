//! The `positions/search` cases.

mod corners_and_traversal;
mod ranges_and_columns;
mod spiral_and_closest;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [ranges_and_columns::cases(), spiral_and_closest::cases(), corners_and_traversal::cases()].concat()
}
