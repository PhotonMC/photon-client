//! The `vec3/basic` cases.

mod codec_and_arithmetic;
mod vector_to_cross;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [codec_and_arithmetic::cases(), vector_to_cross::cases()].concat()
}
