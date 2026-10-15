//! The `mth/algorithms` cases.

mod random;
mod ray_and_inv_sqrt;
mod roots_color_seed;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [random::cases(), ray_and_inv_sqrt::cases(), roots_color_seed::cases()].concat()
}
