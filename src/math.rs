//! The numeric operations currently covered by the Java behavior corpus.
//! Arithmetic stays at the reference's precision and evaluation order.

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

    pub fn scale(self, factor: f64) -> Self {
        Self::new(self.x * factor, self.y * factor, self.z * factor)
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
