//! The numeric operations currently covered by the Java behavior corpus.
//! Arithmetic stays at the reference's precision and evaluation order.

use crate::binary::BlockPos;
use crate::direction::{Axis, Direction};
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

    pub fn dot(self, other: Self) -> f32 {
        // Keep both products rounded to f32 before the source-ordered sum.
        java_f32_add(self.x * other.x, self.y * other.y)
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

    pub fn at_lower_corner_of(pos: BlockPos) -> Self {
        Self::new(f64::from(pos.x), f64::from(pos.y), f64::from(pos.z))
    }

    pub fn at_lower_corner_with_offset(pos: BlockPos, x: f64, y: f64, z: f64) -> Self {
        Self::new(
            f64::from(pos.x) + x,
            f64::from(pos.y) + y,
            f64::from(pos.z) + z,
        )
    }

    pub fn at_center_of(pos: BlockPos) -> Self {
        Self::at_lower_corner_with_offset(pos, 0.5, 0.5, 0.5)
    }

    pub fn at_bottom_center_of(pos: BlockPos) -> Self {
        Self::at_lower_corner_with_offset(pos, 0.5, 0.0, 0.5)
    }

    pub fn up_from_bottom_center_of(pos: BlockPos, y_offset: f64) -> Self {
        Self::at_lower_corner_with_offset(pos, 0.5, y_offset, 0.5)
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

    pub fn relative(self, direction: Direction, distance: f64) -> Self {
        let (step_x, step_y, step_z) = direction.step();
        Self::new(
            self.x + distance * f64::from(step_x),
            self.y + distance * f64::from(step_y),
            self.z + distance * f64::from(step_z),
        )
    }

    pub fn align(self, axes: &[Axis]) -> Self {
        let x = if axes.contains(&Axis::X) {
            floor_f64(self.x) as f64
        } else {
            self.x
        };
        let y = if axes.contains(&Axis::Y) {
            floor_f64(self.y) as f64
        } else {
            self.y
        };
        let z = if axes.contains(&Axis::Z) {
            floor_f64(self.z) as f64
        } else {
            self.z
        };
        Self::new(x, y, z)
    }

    pub fn projected_on(self, onto: Self) -> Self {
        if onto.length_squared() == 0.0 {
            onto
        } else {
            onto.scale(self.dot(onto))
                .scale(1.0 / onto.length_squared())
        }
    }

    pub fn to_vector3f(self) -> Vec3f {
        Vec3f::new(self.x as f32, self.y as f32, self.z as f32)
    }

    /// Compare components with Java `Double.compare` semantics.
    pub fn equals_java(self, other: Self) -> bool {
        java_double_compare(self.x, other.x) == 0
            && java_double_compare(self.y, other.y) == 0
            && java_double_compare(self.z, other.z) == 0
    }

    /// Compute the Java `Vec3.hashCode` value for the three components.
    pub fn hash_code_java(self) -> i32 {
        let mut result = java_double_hash(self.x);
        result = result
            .wrapping_mul(31)
            .wrapping_add(java_double_hash(self.y));
        result
            .wrapping_mul(31)
            .wrapping_add(java_double_hash(self.z))
    }

    /// Format the vector like Java `Vec3.toString()`.
    pub fn to_string_java(self) -> String {
        format!(
            "({}, {}, {})",
            java_double_to_string(self.x),
            java_double_to_string(self.y),
            java_double_to_string(self.z),
        )
    }

    /// Rotate around X using the Java `Mth` sine/cosine lookup contract.
    pub fn x_rot(self, radians: f32) -> Self {
        let cos = f64::from(mth_cos(f64::from(radians)));
        let sin = f64::from(mth_sin(f64::from(radians)));
        let xx = self.x;
        let yy = self.y * cos + self.z * sin;
        let zz = self.z * cos - self.y * sin;
        Self::new(xx, yy, zz)
    }

    /// Rotate around Y using the Java `Mth` sine/cosine lookup contract.
    pub fn y_rot(self, radians: f32) -> Self {
        let cos = f64::from(mth_cos(f64::from(radians)));
        let sin = f64::from(mth_sin(f64::from(radians)));
        let xx = self.x * cos + self.z * sin;
        let yy = self.y;
        let zz = self.z * cos - self.x * sin;
        Self::new(xx, yy, zz)
    }

    /// Rotate around Z using the Java `Mth` sine/cosine lookup contract.
    pub fn z_rot(self, radians: f32) -> Self {
        let cos = f64::from(mth_cos(f64::from(radians)));
        let sin = f64::from(mth_sin(f64::from(radians)));
        let xx = self.x * cos + self.y * sin;
        let yy = self.y * cos - self.x * sin;
        let zz = self.z;
        Self::new(xx, yy, zz)
    }

    /// Convert pitch and yaw in degrees to a Java-compatible direction vector.
    pub fn direction_from_rotation(rotation: Vec2) -> Self {
        Self::direction_from_rotation_components(rotation.x, rotation.y)
    }

    /// Convert pitch and yaw components in degrees to a Java-compatible direction vector.
    pub fn direction_from_rotation_components(rot_x: f32, rot_y: f32) -> Self {
        // These constants are the f32 results of Java's `(float)Math.PI` and
        // `(float)(Math.PI / 180.0)` casts. Keep all angle and vector products
        // in f32 until the Vec3 constructor widens the results to f64.
        const PI_F32: f32 = f32::from_bits(0x40490fdb);
        const DEGREES_TO_RADIANS_F32: f32 = f32::from_bits(0x3c8efa35);
        let y_angle = -rot_y * DEGREES_TO_RADIANS_F32 - PI_F32;
        let y_cos = mth_cos(f64::from(y_angle));
        let y_sin = mth_sin(f64::from(y_angle));
        let x_angle = -rot_x * DEGREES_TO_RADIANS_F32;
        let x_cos = -mth_cos(f64::from(x_angle));
        let x_sin = mth_sin(f64::from(x_angle));
        Self::new(
            f64::from(y_sin * x_cos),
            f64::from(x_sin),
            f64::from(y_cos * x_cos),
        )
    }

    /// Extract pitch and yaw angles using the Java `rotation` calculation.
    pub fn rotation(self) -> Vec2 {
        // Java evaluates atan2/asin/sqrt in f64, narrows each angle to f32,
        // then multiplies by the f32 degrees-per-radian constant.
        const DEGREES_PER_RADIAN_F32: f32 = f32::from_bits(0x42652ee0);
        let yaw = ((-self.x).atan2(self.z) as f32) * DEGREES_PER_RADIAN_F32;
        let length_squared = self.x * self.x + self.y * self.y + self.z * self.z;
        let pitch = (java_asin(-self.y / length_squared.sqrt()) as f32) * DEGREES_PER_RADIAN_F32;
        Vec2::new(pitch, yaw)
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

fn java_asin(value: f64) -> f64 {
    if value.abs() > 1.0 {
        // Java Math.asin returns the negative canonical NaN for every
        // out-of-domain finite or infinite argument. NaN inputs themselves
        // retain their payload and sign through the platform implementation.
        f64::from_bits(0xfff8_0000_0000_0000)
    } else {
        value.asin()
    }
}

fn java_f32_add(left: f32, right: f32) -> f32 {
    if right.is_nan() {
        right
    } else if left.is_nan() {
        left
    } else {
        left + right
    }
}

fn java_double_compare(left: f64, right: f64) -> i32 {
    if left < right {
        -1
    } else if left > right {
        1
    } else {
        let left_bits = java_double_to_long_bits(left) as i64;
        let right_bits = java_double_to_long_bits(right) as i64;
        if left_bits < right_bits {
            -1
        } else if left_bits > right_bits {
            1
        } else {
            0
        }
    }
}

fn java_double_to_long_bits(value: f64) -> u64 {
    if value.is_nan() {
        0x7ff8_0000_0000_0000
    } else {
        value.to_bits()
    }
}

fn java_double_hash(value: f64) -> i32 {
    let bits = java_double_to_long_bits(value);
    (bits ^ (bits >> 32)) as i32
}

const MTH_SIN_SCALE: f64 = 10430.378350470453;

fn mth_sin(value: f64) -> f32 {
    mth_sin_table_value(mth_lookup_index(value * MTH_SIN_SCALE))
}

fn mth_cos(value: f64) -> f32 {
    mth_sin_table_value(mth_lookup_index(value * MTH_SIN_SCALE + 16384.0))
}

fn mth_lookup_index(scaled: f64) -> usize {
    // Java's double-to-long narrowing conversion maps NaN to zero and clamps
    // out-of-range values; Rust's float-to-int cast has the same contract.
    ((scaled as i64) as u64 & 0xffff) as usize
}

fn mth_sin_table_value(index: usize) -> f32 {
    (index as f64 / MTH_SIN_SCALE).sin() as f32
}

pub(crate) fn java_double_to_string(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value == f64::INFINITY {
        return "Infinity".to_owned();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".to_owned();
    }

    let raw = format!("{value:?}");
    if value == 0.0 || (value.abs() >= 1.0e-3 && value.abs() < 1.0e7) {
        return raw;
    }

    if let Some((mantissa, exponent)) = raw.split_once('e') {
        return java_scientific_string(
            mantissa,
            exponent.parse().expect("Rust exponent is numeric"),
        );
    }
    java_decimal_to_scientific(&raw)
}

fn java_scientific_string(mantissa: &str, exponent: i32) -> String {
    let mantissa = if mantissa.contains('.') {
        mantissa.to_owned()
    } else {
        format!("{mantissa}.0")
    };
    format!("{mantissa}E{exponent}")
}

fn java_decimal_to_scientific(raw: &str) -> String {
    let (sign, magnitude) = match raw.strip_prefix('-') {
        Some(magnitude) => ("-", magnitude),
        None => ("", raw),
    };
    let decimal = magnitude.find('.').unwrap_or(magnitude.len());
    let digits = magnitude.replace('.', "");
    let digits = digits.trim_end_matches('0');
    let first = digits
        .bytes()
        .position(|digit| digit != b'0')
        .expect("nonzero finite value has a significant digit");
    let exponent = decimal as i32 - first as i32 - 1;
    let mut mantissa = String::new();
    mantissa.push(digits.as_bytes()[first] as char);
    mantissa.push('.');
    if first + 1 < digits.len() {
        mantissa.push_str(&digits[first + 1..]);
    } else {
        mantissa.push('0');
    }
    format!("{sign}{mantissa}E{exponent}")
}
