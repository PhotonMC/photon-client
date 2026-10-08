//! Keyboard-driven player input for the pinned Java client.

use crate::java_math;
use std::cmp::Ordering;

use photon_core::math::{MTH_DEG_TO_RAD, Vec2, Vec3, clamp_f32, mth_cos_f64, mth_sin_f64};

mod fluids;
pub use fluids::*;
mod friction;
pub use friction::*;
mod keyboard;
pub use keyboard::*;
mod sprint;
pub use sprint::*;
mod travel;
pub use travel::*;
