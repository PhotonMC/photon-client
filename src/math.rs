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

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    pub const X_AXIS: Self = Self::new(1.0, 0.0, 0.0);
    pub const Y_AXIS: Self = Self::new(0.0, 1.0, 0.0);
    pub const Z_AXIS: Self = Self::new(0.0, 0.0, 1.0);

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Decode the fixed three-element list used by Java `Vec3.CODEC`.
    pub fn codec_decode(values: &[f64]) -> Result<Self, Vec3CodecError> {
        match values {
            [x, y, z] => Ok(Self::new(*x, *y, *z)),
            values => Err(Vec3CodecError::WrongLength {
                actual: values.len(),
                partial: (values.len() >= 3).then(|| Self::new(values[0], values[1], values[2])),
            }),
        }
    }

    /// Encode a Vec3 in the x, y, z order used by Java `Vec3.CODEC`.
    pub const fn codec_encode(self) -> [f64; 3] {
        [self.x, self.y, self.z]
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

    /// Apply local coordinates using the Java pitch/yaw basis construction.
    pub fn apply_local_coordinates_to_rotation(rotation: Vec2, direction: Self) -> Self {
        // Keep angle arithmetic and basis products in f32 until each basis
        // component is widened by the Java Vec3 constructor.
        const DEGREES_TO_RADIANS_F32: f32 = f32::from_bits(0x3c8efa35);
        let y_angle = (rotation.y + 90.0_f32) * DEGREES_TO_RADIANS_F32;
        let y_cos = mth_cos(f64::from(y_angle));
        let y_sin = mth_sin(f64::from(y_angle));
        let x_angle = -rotation.x * DEGREES_TO_RADIANS_F32;
        let x_cos = mth_cos(f64::from(x_angle));
        let x_sin = mth_sin(f64::from(x_angle));
        let x_up_angle = (-rotation.x + 90.0_f32) * DEGREES_TO_RADIANS_F32;
        let x_cos_up = mth_cos(f64::from(x_up_angle));
        let x_sin_up = mth_sin(f64::from(x_up_angle));
        let forwards = Self::new(
            f64::from(y_cos * x_cos),
            f64::from(x_sin),
            f64::from(y_sin * x_cos),
        );
        let up = Self::new(
            f64::from(y_cos * x_cos_up),
            f64::from(x_sin_up),
            f64::from(y_sin * x_cos_up),
        );
        let left = forwards.cross(up).scale(-1.0);
        Self::new(
            forwards.x * direction.z + up.x * direction.y + left.x * direction.x,
            forwards.y * direction.z + up.y * direction.y + left.y * direction.x,
            forwards.z * direction.z + up.z * direction.y + left.z * direction.x,
        )
    }

    /// Apply local coordinates using this vector's Java rotation.
    pub fn add_local_coordinates(self, direction: Self) -> Self {
        Self::apply_local_coordinates_to_rotation(self.rotation(), direction)
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

/// Return Java `Math.max(Math.abs(a), Math.abs(b))` for signed i32 values.
pub fn abs_max(a: i32, b: i32) -> i32 {
    a.wrapping_abs().max(b.wrapping_abs())
}

/// Return the Java `Mth.chessboardDistance` result with i32 wrapping.
pub fn chessboard_distance(x0: i32, z0: i32, x1: i32, z1: i32) -> i32 {
    abs_max(x1.wrapping_sub(x0), z1.wrapping_sub(z0))
}

/// Return Java `Mth.abs(int)`, including `Integer.MIN_VALUE` wrapping.
pub fn abs_i32(value: i32) -> i32 {
    value.wrapping_abs()
}

/// Return Java `Mth.clamp(int, int, int)`.
pub fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
    value.max(min).min(max)
}

/// Return Java `Mth.clamp(long, long, long)`.
pub fn clamp_i64(value: i64, min: i64, max: i64) -> i64 {
    value.max(min).min(max)
}

/// Return Java `Mth.isMultipleOf(int, int)`.
pub fn is_multiple_of_i32(dividend: i32, divisor: i32) -> bool {
    if divisor == 0 {
        panic!("division by zero");
    }
    if dividend == i32::MIN && divisor == -1 {
        return true;
    }
    dividend % divisor == 0
}

/// Return Java `Mth.wrapDegrees(int)`.
pub fn wrap_degrees_i32(angle: i32) -> i32 {
    let mut normalized_angle = angle % 360;
    if normalized_angle >= 180 {
        normalized_angle -= 360;
    }
    if normalized_angle < -180 {
        normalized_angle += 360;
    }
    normalized_angle
}

/// Return Java `Mth.wrapDegrees(long)`, including the final f32 narrowing.
pub fn wrap_degrees_i64(angle: i64) -> f32 {
    let mut normalized_angle = (angle % 360) as f32;
    if normalized_angle >= 180.0 {
        normalized_angle -= 360.0;
    }
    if normalized_angle < -180.0 {
        normalized_angle += 360.0;
    }
    normalized_angle
}

/// Return Java `Mth.packDegrees(float)` with byte narrowing semantics.
pub fn pack_degrees_f32(angle: f32) -> i8 {
    floor_f32(angle * 256.0 / 360.0) as i8
}

/// Return Java `Mth.unpackDegrees(byte)`.
pub fn unpack_degrees_i8(rotation: i8) -> f32 {
    (i32::from(rotation) * 360) as f32 / 256.0
}

/// Return Java `Mth.ceil(float)` with Java's floating-point-to-int narrowing.
pub fn ceil_f32(value: f32) -> i32 {
    f64::from(value).ceil() as i32
}

/// Return Java `Mth.ceil(double)` with Java's floating-point-to-int narrowing.
pub fn ceil_f64(value: f64) -> i32 {
    value.ceil() as i32
}

/// Return Java `Mth.lfloor(double)` with Java's floating-point-to-long narrowing.
pub fn lfloor_f64(value: f64) -> i64 {
    value.floor() as i64
}

/// Return Java `Mth.ceilLong(double)` with Java's floating-point-to-long narrowing.
pub fn ceil_long_f64(value: f64) -> i64 {
    value.ceil() as i64
}

/// Return Java `Mth.positiveModulo(float)` without changing operation order.
pub fn positive_modulo_f32(input: f32, modulus: f32) -> f32 {
    let result = (input % modulus + modulus) % modulus;
    if result.is_nan() {
        if input.is_finite() && modulus.is_infinite() {
            f32::from_bits(0xffc0_0000)
        } else {
            f32::from_bits(0x7fc0_0000)
        }
    } else {
        result
    }
}

/// Return Java `Mth.positiveModulo(double)` without changing operation order.
pub fn positive_modulo_f64(input: f64, modulus: f64) -> f64 {
    let result = (input % modulus + modulus) % modulus;
    if result.is_nan() {
        if input.is_finite() && modulus.is_infinite() {
            f64::from_bits(0xfff8_0000_0000_0000)
        } else {
            f64::from_bits(0x7ff8_0000_0000_0000)
        }
    } else {
        result
    }
}

/// Return Java `Mth.absMax(float)` with Java NaN and absolute-value bits.
pub fn abs_max_f32(first: f32, second: f32) -> f32 {
    let first = f32::from_bits(first.to_bits() & 0x7fff_ffff);
    let second = f32::from_bits(second.to_bits() & 0x7fff_ffff);
    if first.is_nan() {
        first
    } else if second.is_nan() {
        second
    } else {
        first.max(second)
    }
}

/// Return Java `Mth.absMax(double)` with Java NaN and absolute-value bits.
pub fn abs_max_f64(first: f64, second: f64) -> f64 {
    let first = f64::from_bits(first.to_bits() & 0x7fff_ffff_ffff_ffff);
    let second = f64::from_bits(second.to_bits() & 0x7fff_ffff_ffff_ffff);
    if first.is_nan() {
        first
    } else if second.is_nan() {
        second
    } else {
        first.max(second)
    }
}

/// Return Java `Mth.equal(float, float)` with its strict f32 threshold.
pub fn equal_f32(first: f32, second: f32) -> bool {
    (second - first).abs() < 1.0e-5_f32
}

/// Return Java `Mth.equal(double, double)` with its promoted f32 threshold.
pub fn equal_f64(first: f64, second: f64) -> bool {
    (second - first).abs() < f64::from(1.0e-5_f32)
}

/// Return Java `Mth.clampedLerp(float, float, float)` in source order.
pub fn clamped_lerp_f32(factor: f32, minimum: f32, maximum: f32) -> f32 {
    if factor < 0.0 {
        minimum
    } else if factor > 1.0 {
        maximum
    } else {
        minimum + factor * (maximum - minimum)
    }
}

/// Return Java `Mth.clampedLerp(double, double, double)` in source order.
pub fn clamped_lerp_f64(factor: f64, minimum: f64, maximum: f64) -> f64 {
    if factor < 0.0 {
        minimum
    } else if factor > 1.0 {
        maximum
    } else {
        minimum + factor * (maximum - minimum)
    }
}

/// Return Java `Mth.inverseLerp(float, float, float)` in source order.
pub fn mth_inverse_lerp_f32(value: f32, minimum: f32, maximum: f32) -> f32 {
    let numerator = value - minimum;
    let denominator = maximum - minimum;
    numerator / denominator
}

/// Return Java `Mth.inverseLerp(double, double, double)` in source order.
pub fn mth_inverse_lerp_f64(value: f64, minimum: f64, maximum: f64) -> f64 {
    let numerator = value - minimum;
    let denominator = maximum - minimum;
    numerator / denominator
}

/// Return Java `Mth.clampedMap(double, double, double, double, double)` in source order.
pub fn mth_clamped_map_f64(
    value: f64,
    from_minimum: f64,
    from_maximum: f64,
    to_minimum: f64,
    to_maximum: f64,
) -> f64 {
    let factor = mth_inverse_lerp_f64(value, from_minimum, from_maximum);
    clamped_lerp_f64(factor, to_minimum, to_maximum)
}

/// Return Java `Mth.clampedMap(float, float, float, float, float)` in source order.
pub fn mth_clamped_map_f32(
    value: f32,
    from_minimum: f32,
    from_maximum: f32,
    to_minimum: f32,
    to_maximum: f32,
) -> f32 {
    let factor = mth_inverse_lerp_f32(value, from_minimum, from_maximum);
    clamped_lerp_f32(factor, to_minimum, to_maximum)
}

/// Return Java `Mth.map(double, double, double, double, double)` in source order.
pub fn mth_map_f64(
    value: f64,
    from_minimum: f64,
    from_maximum: f64,
    to_minimum: f64,
    to_maximum: f64,
) -> f64 {
    let factor = mth_inverse_lerp_f64(value, from_minimum, from_maximum);
    mth_lerp_f64(factor, to_minimum, to_maximum)
}

/// Return Java `Mth.lerp(float, float, float)` in source order.
pub fn mth_lerp_f32(alpha: f32, start: f32, end: f32) -> f32 {
    start + alpha * (end - start)
}

/// Return Java `Mth.lerp(double, double, double)` in source order.
pub fn mth_lerp_f64(alpha: f64, start: f64, end: f64) -> f64 {
    start + alpha * (end - start)
}

/// Return Java `Mth.lerpInt(float, int, int)` with i32 wrapping arithmetic.
pub fn mth_lerp_int(alpha: f32, start: i32, end: i32) -> i32 {
    let delta = end.wrapping_sub(start);
    start.wrapping_add(floor_f32(alpha * delta as f32))
}

/// Return Java `Mth.lerpDiscrete(float, int, int)` with i32 wrapping arithmetic.
pub fn mth_lerp_discrete(alpha: f32, start: i32, end: i32) -> i32 {
    let delta = end.wrapping_sub(start);
    let offset = floor_f32(alpha * delta.wrapping_sub(1) as f32);
    let positive_alpha = if alpha > 0.0 { 1 } else { 0 };
    start.wrapping_add(offset).wrapping_add(positive_alpha)
}

/// Return Java `Mth.smoothstep(double)` in source order.
pub fn mth_smoothstep(x: f64) -> f64 {
    x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
}

/// Return Java `Mth.smoothstepDerivative(double)` in source order.
pub fn mth_smoothstep_derivative(x: f64) -> f64 {
    30.0 * x * x * (x - 1.0) * (x - 1.0)
}

/// Return Java `Mth.sign(double)` with its NaN branch preserved.
pub fn mth_sign(number: f64) -> i32 {
    if number == 0.0 {
        0
    } else if number > 0.0 {
        1
    } else {
        -1
    }
}

/// Return Java `Mth.rotLerp(float, float, float)` in source order.
pub fn mth_rot_lerp_f32(alpha: f32, from: f32, to: f32) -> f32 {
    from + alpha * wrap_degrees_f32(to - from)
}

/// Return Java `Mth.rotLerp(double, double, double)` in source order.
pub fn mth_rot_lerp_f64(alpha: f64, from: f64, to: f64) -> f64 {
    from + alpha * wrap_degrees_f64(to - from)
}

/// Return Java `Mth.rotLerpRad(float, float, float)` with float loop constants.
pub fn mth_rot_lerp_rad(alpha: f32, from: f32, to: f32) -> f32 {
    let mut diff = to - from;
    let pi = f32::from_bits(0x4049_0fdb);
    let two_pi = f32::from_bits(0x40c9_0fdb);

    while diff < -pi {
        diff += two_pi;
    }
    while diff >= pi {
        diff -= two_pi;
    }

    from + alpha * diff
}

/// Return Java `Mth.triangleWave(float, float)` in source order.
pub fn mth_triangle_wave(index: f32, period: f32) -> f32 {
    ((index % period - period * 0.5_f32).abs() - period * 0.25_f32) / (period * 0.25_f32)
}

/// Return Java `Mth.square(float)` in source order.
pub fn mth_square_f32(x: f32) -> f32 {
    x * x
}

/// Return Java `Mth.cube(float)` in source order.
pub fn mth_cube_f32(x: f32) -> f32 {
    x * x * x
}

/// Return Java `Mth.square(double)` in source order.
pub fn mth_square_f64(x: f64) -> f64 {
    x * x
}

/// Return Java `Mth.square(int)` with Java's wrapping i32 multiplication.
pub fn mth_square_i32(x: i32) -> i32 {
    x.wrapping_mul(x)
}

/// Return Java `Mth.square(long)` with Java's wrapping i64 multiplication.
pub fn mth_square_i64(x: i64) -> i64 {
    x.wrapping_mul(x)
}

/// Return Java `Mth.nextInt(RandomSource, int, int)`, including its
/// min-greater-than-or-equal-to-max short circuit.
pub fn mth_next_int(
    random: &mut RandomSource,
    minimum: i32,
    maximum: i32,
) -> Result<i32, crate::random::RandomError> {
    if minimum >= maximum {
        Ok(minimum)
    } else {
        let bound = maximum.wrapping_sub(minimum).wrapping_add(1);
        random
            .next_int_bound(bound)
            .map(|value| value.wrapping_add(minimum))
    }
}

/// Return Java `Mth.nextFloat(RandomSource, float, float)` in source order.
pub fn mth_next_float(random: &mut RandomSource, minimum: f32, maximum: f32) -> f32 {
    if minimum >= maximum {
        minimum
    } else {
        random.next_float() * (maximum - minimum) + minimum
    }
}

/// Return Java `Mth.nextDouble(RandomSource, double, double)` in source order.
pub fn mth_next_double(random: &mut RandomSource, minimum: f64, maximum: f64) -> f64 {
    if minimum >= maximum {
        minimum
    } else {
        random.next_double() * (maximum - minimum) + minimum
    }
}

/// Return Java `Mth.randomBetweenInclusive(RandomSource, int, int)`.
pub fn mth_random_between_inclusive(
    random: &mut RandomSource,
    minimum: i32,
    maximum: i32,
) -> Result<i32, crate::random::RandomError> {
    let bound = maximum.wrapping_sub(minimum).wrapping_add(1);
    random
        .next_int_bound(bound)
        .map(|value| value.wrapping_add(minimum))
}

/// Return Java `Mth.randomBetween(RandomSource, float, float)` in source order.
pub fn mth_random_between(random: &mut RandomSource, minimum: f32, maximum: f32) -> f32 {
    random.next_float() * (maximum - minimum) + minimum
}

/// Return Java `Mth.normal(RandomSource, float, float)` in source order.
pub fn mth_normal(random: &mut RandomSource, mean: f32, deviation: f32) -> f32 {
    mean + random.next_gaussian() as f32 * deviation
}

/// Return Java `Mth.lengthSquared(double, double)` in source order.
pub fn mth_length_squared_f64_2(x: f64, y: f64) -> f64 {
    let x_squared = x * x;
    let y_squared = y * y;
    x_squared + y_squared
}

/// Return Java `Mth.length(double, double)` in source order.
pub fn mth_length_f64_2(x: f64, y: f64) -> f64 {
    mth_length_squared_f64_2(x, y).sqrt()
}

/// Return Java `Mth.length(float, float)`, including its double helper call.
pub fn mth_length_f32_2(x: f32, y: f32) -> f32 {
    mth_length_squared_f64_2(f64::from(x), f64::from(y)).sqrt() as f32
}

/// Return Java `Mth.lengthSquared(double, double, double)` in source order.
pub fn mth_length_squared_f64_3(x: f64, y: f64, z: f64) -> f64 {
    let x_squared = x * x;
    let y_squared = y * y;
    let z_squared = z * z;
    x_squared + y_squared + z_squared
}

/// Return Java `Mth.length(double, double, double)` in source order.
pub fn mth_length_f64_3(x: f64, y: f64, z: f64) -> f64 {
    mth_length_squared_f64_3(x, y, z).sqrt()
}

/// Return Java `Mth.lengthSquared(float, float, float)` with f32 rounding.
pub fn mth_length_squared_f32_3(x: f32, y: f32, z: f32) -> f32 {
    java_f32_add(java_f32_add(x * x, y * y), z * z)
}

/// Return Java `Mth.quantize(double, int)`, including Java i32 overflow.
pub fn mth_quantize(value: f64, resolution: i32) -> i32 {
    let floored = (value / f64::from(resolution)).floor() as i32;
    floored.wrapping_mul(resolution)
}

/// Return Java `Mth.outFromOrigin(int, int, int, int)` as an ordered list.
pub fn mth_out_from_origin(
    origin: i32,
    lower_bound: i32,
    upper_bound: i32,
    step_size: i32,
) -> Result<Vec<i32>, String> {
    if lower_bound > upper_bound {
        return Err(format!(
            "upperBound {upper_bound} expected to be > lowerBound {lower_bound}"
        ));
    }
    if step_size < 1 {
        return Err(format!("step size expected to be >= 1, was {step_size}"));
    }

    let clamped_origin = clamp_i32(origin, lower_bound, upper_bound);
    let mut cursor = clamped_origin;
    let mut values = Vec::new();
    loop {
        let current_distance = clamped_origin.wrapping_sub(cursor).wrapping_abs();
        let has_next = clamped_origin.wrapping_sub(current_distance) >= lower_bound
            || clamped_origin.wrapping_add(current_distance) <= upper_bound;
        if !has_next {
            break;
        }
        values.push(cursor);

        let previous_was_negative = cursor <= clamped_origin;
        let can_move_positive = clamped_origin
            .wrapping_add(current_distance)
            .wrapping_add(step_size)
            <= upper_bound;
        if !previous_was_negative || !can_move_positive {
            let attempted_step = clamped_origin
                .wrapping_sub(current_distance)
                .wrapping_sub(if previous_was_negative { step_size } else { 0 });
            if attempted_step >= lower_bound {
                cursor = attempted_step;
                continue;
            }
        }

        cursor = clamped_origin
            .wrapping_add(current_distance)
            .wrapping_add(step_size);
    }
    Ok(values)
}

/// Return Java `Mth.mulAndTruncate(Fraction, int)` for a Fraction's fields.
pub fn mth_mul_and_truncate(numerator: i32, denominator: i32, factor: i32) -> i32 {
    let product = numerator.wrapping_mul(factor);
    if product == i32::MIN && denominator == -1 {
        i32::MIN
    } else {
        product / denominator
    }
}

/// Return Java `Mth.rotationAroundAxis(Vector3fc, Quaternionf, Quaternionf)`.
pub fn mth_rotation_around_axis(axis: Vec3f, rotation: Quaternionf) -> Quaternionf {
    let projected_length = axis
        .x
        .mul_add(rotation.x, axis.y.mul_add(rotation.y, axis.z * rotation.z));
    Quaternionf::new(
        axis.x * projected_length,
        axis.y * projected_length,
        axis.z * projected_length,
        rotation.w,
    )
    .normalized()
}

/// Return Java `Math.floorDiv(int, int)`, including Java's signed overflow.
pub fn floor_div_i32(dividend: i32, divisor: i32) -> i32 {
    if divisor == 0 {
        panic!("division by zero");
    }
    // Java's integer division defines MIN_VALUE / -1 as MIN_VALUE; Rust's
    // checked debug division would otherwise panic here.
    if dividend == i32::MIN && divisor == -1 {
        return i32::MIN;
    }
    let quotient = dividend / divisor;
    let remainder = dividend % divisor;
    if remainder != 0 && (dividend < 0) != (divisor < 0) {
        quotient.wrapping_sub(1)
    } else {
        quotient
    }
}

/// Return Java `Math.floorMod(int, int)` as used by `Mth.positiveModulo`.
pub fn positive_modulo_i32(input: i32, modulus: i32) -> i32 {
    let quotient = floor_div_i32(input, modulus);
    input.wrapping_sub(quotient.wrapping_mul(modulus))
}

/// Return `Mth.positiveCeilDiv(int, int)` with Java negation semantics.
pub fn positive_ceil_div_i32(input: i32, divisor: i32) -> i32 {
    floor_div_i32(input.wrapping_neg(), divisor).wrapping_neg()
}

/// Return `Mth.positiveCeilDiv(long, long)` with Java negation semantics.
pub fn positive_ceil_div_i64(input: i64, divisor: i64) -> i64 {
    floor_div_i64(input.wrapping_neg(), divisor).wrapping_neg()
}

/// Return `Mth.roundToward(int, int)` with Java multiplication semantics.
pub fn round_toward_i32(input: i32, multiple: i32) -> i32 {
    positive_ceil_div_i32(input, multiple).wrapping_mul(multiple)
}

/// Return `Mth.roundToward(long, long)` with Java multiplication semantics.
pub fn round_toward_i64(input: i64, multiple: i64) -> i64 {
    positive_ceil_div_i64(input, multiple).wrapping_mul(multiple)
}

fn floor_div_i64(dividend: i64, divisor: i64) -> i64 {
    if divisor == 0 {
        panic!("division by zero");
    }
    if dividend == i64::MIN && divisor == -1 {
        return i64::MIN;
    }
    let quotient = dividend / divisor;
    let remainder = dividend % divisor;
    if remainder != 0 && (dividend < 0) != (divisor < 0) {
        quotient.wrapping_sub(1)
    } else {
        quotient
    }
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

fn java_float_hash(value: f32) -> i32 {
    let bits = if value.is_nan() {
        0x7fc0_0000_u32
    } else {
        value.to_bits()
    };
    bits as i32
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
