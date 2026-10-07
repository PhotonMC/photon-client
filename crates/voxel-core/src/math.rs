//! The numeric operations currently covered by the Java behavior corpus.
//! Arithmetic stays at the reference's precision and evaluation order.

use crate::aabb::Aabb;
use crate::binary::{BlockPos, Uuid};
use crate::direction::{Axis, Direction};
use crate::random::RandomSource;

/// Java `Mth` public scalar constants, stored with their reference f32 bits.
pub const MTH_PI: f32 = f32::from_bits(0x4049_0fdb);
pub const MTH_HALF_PI: f32 = f32::from_bits(0x3fc9_0fdb);
pub const MTH_TWO_PI: f32 = f32::from_bits(0x40c9_0fdb);
pub const MTH_DEG_TO_RAD: f32 = f32::from_bits(0x3c8e_fa35);
pub const MTH_RAD_TO_DEG: f32 = f32::from_bits(0x4265_2ee0);
pub const MTH_EPSILON: f32 = f32::from_bits(0x3727_c5ac);
pub const MTH_SQRT_OF_TWO: f32 = f32::from_bits(0x3fb5_04f3);

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
impl Vec2 {
    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const ONE: Self = Self::new(1.0, 1.0);
    pub const UNIT_X: Self = Self::new(1.0, 0.0);
    pub const NEG_UNIT_X: Self = Self::new(-1.0, 0.0);
    pub const UNIT_Y: Self = Self::new(0.0, 1.0);
    pub const NEG_UNIT_Y: Self = Self::new(0.0, -1.0);
    pub const MAX: Self = Self::new(f32::MAX, f32::MAX);
    // Java Float.MIN_VALUE is the smallest positive subnormal, not the
    // smallest positive normal represented by Rust's f32::MIN_POSITIVE.
    pub const MIN: Self = Self::new(f32::from_bits(1), f32::from_bits(1));

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn scale(self, factor: f32) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }

    pub fn dot(self, other: Self) -> f32 {
        // Keep both products rounded to f32 before the source-ordered sum.
        java_f32_add(self.x * other.x, self.y * other.y)
    }

    pub fn add_vec2(self, other: Self) -> Self {
        Self::new(java_f32_add(self.x, other.x), java_f32_add(self.y, other.y))
    }

    pub fn add_scalar(self, value: f32) -> Self {
        Self::new(java_f32_add(self.x, value), java_f32_add(self.y, value))
    }

    pub fn distance_to_sqr(self, other: Self) -> f32 {
        let xd = other.x - self.x;
        let yd = other.y - self.y;
        java_f32_add(xd * xd, yd * yd)
    }

    pub fn negated(self) -> Self {
        Self::new(-self.x, -self.y)
    }

    pub fn equals_java(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }

    pub fn hash_code_java(self) -> i32 {
        let mut result = 1_i32;
        result = result
            .wrapping_mul(31)
            .wrapping_add(java_float_hash(self.x));
        result
            .wrapping_mul(31)
            .wrapping_add(java_float_hash(self.y))
    }

    pub fn length_squared(self) -> f32 {
        // Do not fuse the multiplication and addition or substitute hypot:
        // intermediate f32 rounding/overflow is observable in the reference.
        java_f32_add(self.x * self.x, self.y * self.y)
    }

    pub fn length(self) -> f32 {
        f64::from(self.length_squared()).sqrt() as f32
    }

    pub fn normalized(self) -> Self {
        let length = self.length();
        if length < 0.0001 {
            Self::ZERO
        } else {
            // Dividing each component preserves the reference rounding;
            // multiplication by a shared reciprocal is not interchangeable.
            Self::new(self.x / length, self.y / length)
        }
    }

    /// Rotate using the Java `Mth` sine/cosine lookup contract.
    pub fn rotate(self, angle_radians: f64) -> Self {
        let cosine = mth_cos(angle_radians);
        let sine = mth_sin(angle_radians);
        Self::new(
            self.x * cosine - self.y * sine,
            self.y * cosine + self.x * sine,
        )
    }
}

/// Three-component f32 value returned by the Java-compatible Vec3 conversion.
#[derive(Debug, Clone, Copy)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3f {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// Java `Mth` public unit-axis constants (`Vector3fc` values).
pub const MTH_Y_AXIS: Vec3f = Vec3f::new(0.0, 1.0, 0.0);
pub const MTH_X_AXIS: Vec3f = Vec3f::new(1.0, 0.0, 0.0);
pub const MTH_Z_AXIS: Vec3f = Vec3f::new(0.0, 0.0, 1.0);

/// Four-component f32 value used by Java-compatible quaternion helpers.
#[derive(Debug, Clone, Copy)]
pub struct Quaternionf {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternionf {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// Return the JOML `Quaternionf.rotationX(float)` construction used by
    /// Direction's fixed rotations.
    pub(crate) fn rotation_x(angle: f32) -> Self {
        let half_angle = angle * 0.5;
        let sine = joml_sin_f32(half_angle);
        let cosine = joml_cos_from_sin(sine, half_angle);
        Self::new(sine, 0.0, 0.0, cosine)
    }

    /// Return the JOML `Quaternionf.rotationXYZ(float, float, float)`
    /// construction used by Direction's fixed rotations.
    pub(crate) fn rotation_xyz(x_angle: f32, y_angle: f32, z_angle: f32) -> Self {
        let x_half = x_angle * 0.5;
        let x_sin = joml_sin_f32(x_half);
        let x_cos = joml_cos_from_sin(x_sin, x_half);
        let y_half = y_angle * 0.5;
        let y_sin = joml_sin_f32(y_half);
        let y_cos = joml_cos_from_sin(y_sin, y_half);
        let z_half = z_angle * 0.5;
        let z_sin = joml_sin_f32(z_half);
        let z_cos = joml_cos_from_sin(z_sin, z_half);

        let cos_y_cos_z = y_cos * z_cos;
        let sin_y_sin_z = y_sin * z_sin;
        let sin_y_cos_z = y_sin * z_cos;
        let cos_y_sin_z = y_cos * z_sin;
        Self::new(
            x_sin * cos_y_cos_z + x_cos * sin_y_sin_z,
            x_cos * sin_y_cos_z - x_sin * cos_y_sin_z,
            x_cos * cos_y_sin_z + x_sin * sin_y_cos_z,
            x_cos * cos_y_cos_z - x_sin * sin_y_sin_z,
        )
    }

    fn normalized(self) -> Self {
        // JOML accumulates the quaternion norm with a right-associated f32
        // FMA chain, then narrows Math.sqrt's f64 result before dividing.
        let length_squared = self.x.mul_add(
            self.x,
            self.y
                .mul_add(self.y, self.z.mul_add(self.z, self.w * self.w)),
        );
        let square_root = f64::from(length_squared).sqrt() as f32;
        let inverse = 1.0_f32 / square_root;
        Self::new(
            self.x * inverse,
            self.y * inverse,
            self.z * inverse,
            self.w * inverse,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum Vec3CodecError {
    WrongLength {
        actual: usize,
        partial: Option<Vec3>,
    },
}

const MULTIPLY_DE_BRUIJN_BIT_POSITION: [i32; 32] = [
    0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8, 31, 27, 13, 23, 21, 19, 16, 7, 26,
    12, 18, 6, 11, 5, 10, 9,
];

// The same contract at two precisions, without a public numeric trait hierarchy.
macro_rules! scalar_operations {
    ($ty:ty, $floor:ident, $wrap:ident, $clamp:ident) => {
        /// Round downward and convert to i32, saturating at the signed bounds.
        /// NaN converts to zero, as with the reference's Java narrowing cast.
        pub fn $floor(value: $ty) -> i32 {
            f64::from(value).floor() as i32
        }

        /// Wrap degrees to [-180, 180), retaining the sign of zero.
        pub fn $wrap(value: $ty) -> $ty {
            let remainder = value % 360.0;
            if remainder >= 180.0 {
                remainder - 360.0
            } else if remainder < -180.0 {
                remainder + 360.0
            } else {
                remainder
            }
        }

        /// Reference clamp semantics, including NaN and signed-zero behavior.
        /// Unlike a general clamp API, this does not assert ordered bounds.
        pub fn $clamp(value: $ty, min: $ty, max: $ty) -> $ty {
            if value < min {
                min
            } else if value.is_nan() {
                value
            } else if max.is_nan() {
                max
            } else if value == 0.0 && max == 0.0 {
                // Java Math.min chooses negative zero when either zero is negative.
                <$ty>::from_bits(value.to_bits() | max.to_bits())
            } else if value <= max {
                value
            } else {
                max
            }
        }
    };
}

scalar_operations!(f32, floor_f32, wrap_degrees_f32, clamp_f32);
scalar_operations!(f64, floor_f64, wrap_degrees_f64, clamp_f64);

const MTH_SIN_SCALE: f64 = 10430.378350470453;

mod mth;
pub use mth::*;
mod mth_2;
pub use mth_2::*;
mod mth_3;
pub use mth_3::*;

mod vec3_components;
