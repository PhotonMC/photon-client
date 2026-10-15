//! The `mth/interpolation` cases.

mod lerp3_catmullrom;
mod lerp_family;
mod map_and_wobble;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [lerp_family::cases(), map_and_wobble::cases(), lerp3_catmullrom::cases()].concat()
}
