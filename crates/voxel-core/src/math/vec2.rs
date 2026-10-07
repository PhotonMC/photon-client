use super::*;

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
