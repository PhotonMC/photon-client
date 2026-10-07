//! The overworld sky: what colour and where everything in it is, and the
//! geometry that draws it.
//!
//! The reference does not hard-code its sky. Every value the sky reads is an
//! environment attribute: a default, then the dimension's value, then the
//! biomes' values blended around the camera, then each timeline the dimension
//! runs, sampled at its clock. This module reads those same data files from
//! the local vanilla data pack and evaluates them in that order, once per
//! tick, and blends the last two ticks by the partial tick as the camera's
//! attribute probe does.
//!
//! The geometry is the sky renderer's: a disc above in the sky colour faded
//! into the fog colour with distance, the sunrise fan, the sun and the moon
//! phase on quads a hundred blocks out, the seeded star field, and the dark
//! disc below the horizon.

pub mod clouds;
pub mod weather;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use voxel_core::json::{JsonObject, JsonValue};
use voxel_core::math::{mth_cos_f64, mth_sin_f64};
use voxel_core::random::RandomSource;

use crate::block::tint::{argb, multiply};

mod attribute;
pub use attribute::*;
mod environment;
pub use environment::*;
mod frame;
pub use frame::*;
mod lerp;
pub use lerp::*;
mod lightmap;
pub use lightmap::*;
mod matrix;
use matrix::*;
mod parse;
use parse::*;
mod stars;
pub use stars::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn track(attribute: Attribute, period: Option<i64>, text: &str) -> Track {
        let JsonValue::Object(object) = voxel_core::json::parse(text).expect("json") else {
            panic!("an object");
        };
        Track::parse(attribute, period, &object).expect("a track")
    }

    #[test]
    fn the_sun_track_wraps_its_period_and_is_overhead_at_noon() {
        let sun = track(
            Attribute::SunAngle,
            Some(24000),
            r#"{"ease": {"cubic_bezier": [0.362, 0.241, 0.638, 0.759]},
                "keyframes": [{"ticks": 6000, "value": 360.0}, {"ticks": 6000, "value": 0.0}]}"#,
        );
        assert_eq!(sun.sample(6000), Value::Float(0.0));
        assert_eq!(sun.sample(30000), Value::Float(0.0));
        // Halfway round the clock the eased curve is at its midpoint.
        assert_eq!(sun.sample(18000), Value::Float(180.0));
        let before_noon = sun.sample(5999).float();
        assert!(before_noon > 359.0 && before_noon < 360.0);
    }

    #[test]
    fn a_colour_multiply_keeps_white_as_identity() {
        assert_eq!(multiply(0xFFFF_FFFF, 0xFF78_A7FF), 0xFF78_A7FF);
        assert_eq!(multiply(0xFF78_A7FF, 0xFF00_0000), 0xFF00_0000);
        assert_eq!(srgb_lerp(0.5, 0xFF00_0000, 0xFFFF_FFFF), 0xFF7F_7F7F);
    }

    #[test]
    fn a_constant_track_holds_until_the_next_keyframe() {
        let phase = track(
            Attribute::MoonPhase,
            Some(192000),
            r#"{"keyframes": [{"ticks": 0, "value": "full_moon"}, {"ticks": 24000, "value": "waning_gibbous"}]}"#,
        );
        assert_eq!(phase.sample(23999), Value::Phase(0));
        assert_eq!(phase.sample(24000), Value::Phase(1));
    }
}
