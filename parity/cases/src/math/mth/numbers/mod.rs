//! The `mth/numbers` cases.

mod constants_and_clamps;
mod rounding_and_signs;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [constants_and_clamps::cases(), rounding_and_signs::cases()].concat()
}
