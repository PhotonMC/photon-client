//! `ClientWorld` entity layer split out of the former single-file module.

use super::*;

use voxel_core::aabb::Aabb;
use voxel_core::java_float::{DVec3, JavaF32, JavaF64};
use voxel_core::math::{
    Vec3, clamp_f32, mth_atan2, mth_cos_f64, mth_lerp_f32, mth_lerp_f64, mth_rot_lerp_f32,
    mth_rot_lerp_f64, mth_sin_f64, projectile_lerp_rotation_f32, wrap_degrees_f32,
};

mod accessors;
mod animation;
mod kinds;
mod state;

mod entity_2;
pub use entity_2::*;

mod dimensions;
pub use dimensions::*;
mod attribute;
pub use attribute::*;
