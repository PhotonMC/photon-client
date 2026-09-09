//! The numeric operations currently covered by the Java behavior corpus.
//! Arithmetic stays at the reference's precision and evaluation order.

use crate::direction::Axis;
use crate::random::RandomSource;

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
impl Vec2 {
    pub const ZERO: Self = Self::new(0.0, 0.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn scale(self, factor: f32) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }

    pub fn length_squared(self) -> f32 {
        // Do not fuse the multiplication and addition or substitute hypot:
        // intermediate f32 rounding/overflow is observable in the reference.
        self.x * self.x + self.y * self.y
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
}

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn add_components(self, x: f64, y: f64, z: f64) -> Self {
        Self::new(self.x + x, self.y + y, self.z + z)
    }

    pub fn add_scalar(self, value: f64) -> Self {
        self.add_components(value, value, value)
    }

    pub fn add_vec3(self, other: Self) -> Self {
        self.add_components(other.x, other.y, other.z)
    }

    pub fn subtract_components(self, x: f64, y: f64, z: f64) -> Self {
        self.add_components(java_negate(x), java_negate(y), java_negate(z))
    }

    pub fn subtract_scalar(self, value: f64) -> Self {
        self.subtract_components(value, value, value)
    }

    pub fn subtract_vec3(self, other: Self) -> Self {
        self.subtract_components(other.x, other.y, other.z)
    }

    pub fn closer_than_position(self, other: Self, distance: f64) -> bool {
        self.distance_to_sqr_components(other.x, other.y, other.z) < distance * distance
    }

    pub fn closer_than_xz_y(self, other: Self, distance_xz: f64, distance_y: f64) -> bool {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dz = other.z - self.z;
        dx * dx + dz * dz < distance_xz * distance_xz && dy.abs() < distance_y
    }

    pub fn vector_to(self, other: Self) -> Self {
        Self::new(other.x - self.x, other.y - self.y, other.z - self.z)
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn distance_to_sqr(self, other: Self) -> f64 {
        self.distance_to_sqr_components(other.x, other.y, other.z)
    }

    pub fn distance_to(self, other: Self) -> f64 {
        let xd = other.x - self.x;
        let yd = other.y - self.y;
        let zd = other.z - self.z;
        (xd * xd + yd * yd + zd * zd).sqrt()
    }

    pub fn distance_to_sqr_components(self, x: f64, y: f64, z: f64) -> f64 {
        let xd = x - self.x;
        let yd = y - self.y;
        let zd = z - self.z;
        xd * xd + yd * yd + zd * zd
    }

    pub fn scale(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor, self.z * factor)
    }

    pub fn reverse(self) -> Self {
        self.scale(-1.0)
    }

    pub fn multiply_vec3(self, scale: Self) -> Self {
        self.multiply_components(scale.x, scale.y, scale.z)
    }

    pub fn multiply_components(self, x_scale: f64, y_scale: f64, z_scale: f64) -> Self {
        Self::new(self.x * x_scale, self.y * y_scale, self.z * z_scale)
    }

    pub fn horizontal(self) -> Self {
        Self::new(self.x, 0.0, self.z)
    }

    pub fn horizontal_distance(self) -> f64 {
        (self.x * self.x + self.z * self.z).sqrt()
    }

    pub fn horizontal_distance_squared(self) -> f64 {
        self.x * self.x + self.z * self.z
    }

    pub fn offset_random(self, random: &mut RandomSource, offset: f32) -> Self {
        let x = f64::from((random.next_float() - 0.5_f32) * offset);
        let y = f64::from((random.next_float() - 0.5_f32) * offset);
        let z = f64::from((random.next_float() - 0.5_f32) * offset);
        self.add_components(x, y, z)
    }

    pub fn offset_random_xz(self, random: &mut RandomSource, offset: f32) -> Self {
        let x = f64::from((random.next_float() - 0.5_f32) * offset);
        let z = f64::from((random.next_float() - 0.5_f32) * offset);
        self.add_components(x, 0.0, z)
    }

    pub fn lerp(self, other: Self, alpha: f64) -> Self {
        Self::new(
            self.x + alpha * (other.x - self.x),
            self.y + alpha * (other.y - self.y),
            self.z + alpha * (other.z - self.z),
        )
    }

    pub fn rotate_clockwise_90(self) -> Self {
        Self::new(java_negate(self.z), self.y, self.x)
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    pub fn get(self, axis: Axis) -> f64 {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    pub fn with(self, axis: Axis, value: f64) -> Self {
        match axis {
            Axis::X => Self::new(value, self.y, self.z),
            Axis::Y => Self::new(self.x, value, self.z),
            Axis::Z => Self::new(self.x, self.y, value),
        }
    }

    pub fn length_squared(self) -> f64 {
        // Keep the reference evaluation order; do not fuse or substitute hypot.
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn normalized(self) -> Self {
        let length = self.length();
        // The reference compares against the float literal widened to double.
        if length < f64::from(1e-5_f32) {
            Self::ZERO
        } else {
            Self::new(self.x / length, self.y / length, self.z / length)
        }
    }
}

/// Block-to-section coordinate conversion, flooring toward negative infinity.
pub fn block_to_section_coord(block_coord: i32) -> i32 {
    block_coord >> 4
}

/// Section-to-block coordinate conversion with Java int wrapping.
pub fn section_to_block_coord(section_coord: i32) -> i32 {
    section_coord << 4
}

/// Position inside its section, always in `0..16`.
pub fn section_relative(block_coord: i32) -> i32 {
    block_coord & 15
}

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

#[inline(never)]
fn java_negate(value: f64) -> f64 {
    f64::from_bits(value.to_bits() ^ (1_u64 << 63))
}
