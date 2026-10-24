use super::*;

impl Vec3 {
    pub fn lerp(self, other: Self, alpha: f64) -> Self {
        Self::new(
            self.x + alpha * (other.x - self.x),
            self.y + alpha * (other.y - self.y),
            self.z + alpha * (other.z - self.z),
        )
    }

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
        pub(super) const PI_F32: f32 = f32::from_bits(0x40490fdb);
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

    /// `Entity.calculateViewVector`: where an entity looks, from its pitch and yaw in degrees. It
    /// rounds differently from `directionFromRotation`, so the two are not interchangeable.
    pub fn view_vector(x_rot: f32, y_rot: f32) -> Self {
        const DEGREES_TO_RADIANS_F32: f32 = f32::from_bits(0x3c8efa35);
        let real_x_rot = x_rot * DEGREES_TO_RADIANS_F32;
        let real_y_rot = -y_rot * DEGREES_TO_RADIANS_F32;
        let y_cos = mth_cos(f64::from(real_y_rot));
        let y_sin = mth_sin(f64::from(real_y_rot));
        let x_cos = mth_cos(f64::from(real_x_rot));
        let x_sin = mth_sin(f64::from(real_x_rot));
        Self::new(
            f64::from(y_sin * x_cos),
            f64::from(-x_sin),
            f64::from(y_cos * x_cos),
        )
    }

    /// Apply local coordinates using the Java pitch/yaw basis construction.
    pub fn apply_local_coordinates_to_rotation(rotation: Vec2, direction: Self) -> Self {
        // Keep angle arithmetic and basis products in f32 until each basis
        // component is widened by the Java Vec3 constructor.
        pub(super) const DEGREES_TO_RADIANS_F32: f32 = f32::from_bits(0x3c8efa35);
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
        pub(super) const DEGREES_PER_RADIAN_F32: f32 = f32::from_bits(0x42652ee0);
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
