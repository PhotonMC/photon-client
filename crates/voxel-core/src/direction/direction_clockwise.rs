use super::*;

impl Direction {
    /// Return Java `Enum.name()` for this direction.
    pub fn enum_name(self) -> &'static str {
        match self {
            Self::Down => "DOWN",
            Self::Up => "UP",
            Self::North => "NORTH",
            Self::South => "SOUTH",
            Self::West => "WEST",
            Self::East => "EAST",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "down" => Some(Self::Down),
            "up" => Some(Self::Up),
            "north" => Some(Self::North),
            "south" => Some(Self::South),
            "west" => Some(Self::West),
            "east" => Some(Self::East),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Down => "down",
            Self::Up => "up",
            Self::North => "north",
            Self::South => "south",
            Self::West => "west",
            Self::East => "east",
        }
    }

    pub fn data3d(self) -> i32 {
        match self {
            Self::Down => 0,
            Self::Up => 1,
            Self::North => 2,
            Self::South => 3,
            Self::West => 4,
            Self::East => 5,
        }
    }

    pub fn data2d(self) -> i32 {
        match self {
            Self::Down | Self::Up => -1,
            Self::North => 2,
            Self::South => 0,
            Self::West => 1,
            Self::East => 3,
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }

    pub fn axis(self) -> Axis {
        match self {
            Self::Down | Self::Up => Axis::Y,
            Self::North | Self::South => Axis::Z,
            Self::West | Self::East => Axis::X,
        }
    }

    pub fn axis_direction(self) -> AxisDirection {
        match self {
            Self::Down | Self::North | Self::West => AxisDirection::Negative,
            Self::Up | Self::South | Self::East => AxisDirection::Positive,
        }
    }

    pub fn step(self) -> (i32, i32, i32) {
        match self {
            Self::Down => (0, -1, 0),
            Self::Up => (0, 1, 0),
            Self::North => (0, 0, -1),
            Self::South => (0, 0, 1),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
        }
    }

    /// Return Java `Direction.getStepX()`.
    pub fn get_step_x(self) -> i32 {
        self.step().0
    }

    /// Return Java `Direction.getStepY()`.
    pub fn get_step_y(self) -> i32 {
        self.step().1
    }

    /// Return Java `Direction.getStepZ()`.
    pub fn get_step_z(self) -> i32 {
        self.step().2
    }

    /// Return Java `Direction.step()` as a f32 vector.
    pub fn step_vec3f(self) -> Vec3f {
        self.unit_vec3f()
    }

    /// Return Java `Direction.getUnitVec3()` as a f64 vector.
    pub fn unit_vec3(self) -> Vec3 {
        let (x, y, z) = self.step();
        Vec3::new(f64::from(x), f64::from(y), f64::from(z))
    }

    /// Return Java `Direction.getUnitVec3f()` as a f32 vector.
    pub fn unit_vec3f(self) -> Vec3f {
        let (x, y, z) = self.step();
        Vec3f::new(x as f32, y as f32, z as f32)
    }

    /// Return Java `Direction.from3DDataValue(int)` with its abs-remainder index.
    pub fn from_3d_data_value(value: i32) -> Self {
        match value.wrapping_rem(6).wrapping_abs() {
            0 => Self::Down,
            1 => Self::Up,
            2 => Self::North,
            3 => Self::South,
            4 => Self::West,
            5 => Self::East,
            _ => unreachable!("remainder is bounded by the direction table length"),
        }
    }

    /// Return Java `Direction.from2DDataValue(int)` with its abs-remainder index.
    pub fn from_2d_data_value(value: i32) -> Self {
        match value.wrapping_rem(4).wrapping_abs() {
            0 => Self::South,
            1 => Self::West,
            2 => Self::North,
            3 => Self::East,
            _ => unreachable!("remainder is bounded by the horizontal direction table length"),
        }
    }

    /// Return Java `Direction.fromYRot(double)` with its floor, mask, and 2D lookup order.
    pub fn from_y_rot(y_rot: f64) -> Self {
        let data = crate::math::floor_f64(y_rot / 90.0 + 0.5) & 3;
        Self::from_2d_data_value(data)
    }

    /// Return Java `Direction.fromAxisAndDirection(Axis, AxisDirection)`.
    pub fn from_axis_and_direction(axis: Axis, direction: AxisDirection) -> Self {
        match (axis, direction) {
            (Axis::X, AxisDirection::Positive) => Self::East,
            (Axis::X, AxisDirection::Negative) => Self::West,
            (Axis::Y, AxisDirection::Positive) => Self::Up,
            (Axis::Y, AxisDirection::Negative) => Self::Down,
            (Axis::Z, AxisDirection::Positive) => Self::South,
            (Axis::Z, AxisDirection::Negative) => Self::North,
        }
    }

    /// Return Java `Direction.get(AxisDirection, Axis)`.
    pub fn get(axis_direction: AxisDirection, axis: Axis) -> Self {
        Self::from_axis_and_direction(axis, axis_direction)
    }

    /// Return Java `Direction.toYRot()` as a float after its integer data mask.
    pub fn to_y_rot(self) -> f32 {
        ((self.data2d() & 3) * 90) as f32
    }

    /// Return Java `Direction.getYRot(Direction)`, including its vertical error path.
    pub fn get_y_rot(direction: Self) -> Result<f32, DirectionYRotError> {
        match direction {
            Self::North => Ok(180.0),
            Self::South => Ok(0.0),
            Self::West => Ok(90.0),
            Self::East => Ok(-90.0),
            Self::Down | Self::Up => Err(DirectionYRotError::Vertical(direction)),
        }
    }

    /// Return Java `Direction.getRotation()` using the pinned JOML formulas.
    pub fn get_rotation(self) -> Quaternionf {
        let half_pi = std::f32::consts::FRAC_PI_2;
        let pi = std::f32::consts::PI;
        match self {
            Self::Down => Quaternionf::rotation_x(pi),
            Self::Up => Quaternionf::new(0.0, 0.0, 0.0, 1.0),
            Self::North => Quaternionf::rotation_xyz(half_pi, 0.0, pi),
            Self::South => Quaternionf::rotation_x(half_pi),
            Self::West => Quaternionf::rotation_xyz(half_pi, 0.0, half_pi),
            Self::East => Quaternionf::rotation_xyz(half_pi, 0.0, -half_pi),
        }
    }

    /// Return Java `Direction.getNearest(int, int, int, Direction)` with nullable fallback.
    pub fn get_nearest(x: i32, y: i32, z: i32, or_else: Option<Self>) -> Option<Self> {
        let abs_x = x.wrapping_abs();
        let abs_y = y.wrapping_abs();
        let abs_z = z.wrapping_abs();
        if abs_x > abs_z && abs_x > abs_y {
            Some(if x < 0 { Self::West } else { Self::East })
        } else if abs_z > abs_x && abs_z > abs_y {
            Some(if z < 0 { Self::North } else { Self::South })
        } else if abs_y > abs_x && abs_y > abs_z {
            Some(if y < 0 { Self::Down } else { Self::Up })
        } else {
            or_else
        }
    }

    /// Return Java `Direction.axisStepOrder(Vec3)` using the strict X/Z magnitude check.
    pub fn axis_step_order(x: f64, z: f64) -> [Axis; 3] {
        if x.abs() < z.abs() {
            [Axis::Y, Axis::Z, Axis::X]
        } else {
            [Axis::Y, Axis::X, Axis::Z]
        }
    }

    /// Return Java `Direction.getRandom(RandomSource)` using enum declaration order.
    pub fn get_random(random: &mut RandomSource) -> Self {
        DIRECTIONS[random
            .next_int_bound(DIRECTIONS.len() as i32)
            .expect("direction random bound is positive") as usize]
    }

    /// Return Java `Direction.stream()` in enum declaration order.
    pub fn stream() -> impl Iterator<Item = Self> {
        DIRECTIONS.iter().copied()
    }

    /// Return Java `Direction.allShuffled(RandomSource)` in enum declaration order.
    pub fn all_shuffled(random: &mut RandomSource) -> Vec<Self> {
        shuffled_copy(&DIRECTIONS, random)
    }

    /// Return Java `Direction.isFacingAngle(float)` using the Mth lookup table.
    pub fn is_facing_angle(self, y_angle: f32) -> bool {
        // Java evaluates `(float) (Math.PI / 180.0)` before the f32 product.
        pub(super) const DEGREES_TO_RADIANS_F32: f32 = f32::from_bits(0x3c8e_fa35);
        let radians = y_angle * DEGREES_TO_RADIANS_F32;
        let dx = -crate::math::mth_sin_f64(f64::from(radians));
        let dz = crate::math::mth_cos_f64(f64::from(radians));
        let (normal_x, _, normal_z) = self.step();
        normal_x as f32 * dx + normal_z as f32 * dz > 0.0
    }

    /// Return Java `Direction.getApproximateNearest(double, double, double)`.
    pub fn approximate_nearest(dx: f64, dy: f64, dz: f64) -> Self {
        Self::approximate_nearest_f32(dx as f32, dy as f32, dz as f32)
    }

    /// Return Java `Direction.getApproximateNearest(float, float, float)`.
    pub fn approximate_nearest_f32(dx: f32, dy: f32, dz: f32) -> Self {
        let mut result = Self::North;
        let mut highest_dot = f32::from_bits(1);

        for direction in [
            Self::Down,
            Self::Up,
            Self::North,
            Self::South,
            Self::West,
            Self::East,
        ] {
            let (normal_x, normal_y, normal_z) = direction.step();
            let dot = dx * normal_x as f32 + dy * normal_y as f32 + dz * normal_z as f32;
            if dot > highest_dot {
                highest_dot = dot;
                result = direction;
            }
        }

        result
    }

    /// Return Java `Direction.getApproximateNearest(Vec3)`.
    pub fn approximate_nearest_vec3(vector: Vec3) -> Self {
        Self::approximate_nearest(vector.x, vector.y, vector.z)
    }

    pub(super) fn clockwise_y_axis(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
            other => other,
        }
    }

    pub(super) fn counter_clockwise_y_axis(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
            other => other,
        }
    }

    pub(super) fn clockwise_x(self) -> Self {
        match self {
            Self::Down => Self::South,
            Self::Up => Self::North,
            Self::North => Self::Down,
            Self::South => Self::Up,
            other => other,
        }
    }

    pub(super) fn counter_clockwise_x(self) -> Self {
        match self {
            Self::Down => Self::North,
            Self::Up => Self::South,
            Self::North => Self::Up,
            Self::South => Self::Down,
            other => other,
        }
    }

    pub(super) fn clockwise_z(self) -> Self {
        match self {
            Self::Down => Self::West,
            Self::Up => Self::East,
            Self::West => Self::Up,
            Self::East => Self::Down,
            other => other,
        }
    }

    pub(super) fn counter_clockwise_z(self) -> Self {
        match self {
            Self::Down => Self::East,
            Self::Up => Self::West,
            Self::West => Self::Down,
            Self::East => Self::Up,
            other => other,
        }
    }

    /// Return Java `Direction.getClockWise()` with its vertical error path.
    pub fn clockwise_y(self) -> Result<Self, DirectionYRotationError> {
        match self {
            Self::North => Ok(Self::East),
            Self::East => Ok(Self::South),
            Self::South => Ok(Self::West),
            Self::West => Ok(Self::North),
            Self::Down | Self::Up => Err(DirectionYRotationError::ClockwiseVertical(self)),
        }
    }

    /// Return Java `Direction.getCounterClockWise()` with its vertical error path.
    pub fn counter_clockwise_y(self) -> Result<Self, DirectionYRotationError> {
        match self {
            Self::North => Ok(Self::West),
            Self::South => Ok(Self::East),
            Self::West => Ok(Self::South),
            Self::East => Ok(Self::North),
            Self::Down | Self::Up => Err(DirectionYRotationError::CounterClockwiseVertical(self)),
        }
    }

    /// Axial rotation; directions aligned to the axis stay put.
    pub fn clockwise(self, axis: Axis) -> Self {
        match axis {
            Axis::X => {
                if matches!(self, Self::West | Self::East) {
                    self
                } else {
                    self.clockwise_x()
                }
            }
            Axis::Y => {
                if matches!(self, Self::Up | Self::Down) {
                    self
                } else {
                    self.clockwise_y_axis()
                }
            }
            Axis::Z => {
                if matches!(self, Self::North | Self::South) {
                    self
                } else {
                    self.clockwise_z()
                }
            }
        }
    }

    /// Axial counter-rotation; directions aligned to the axis stay put.
    pub fn counter_clockwise(self, axis: Axis) -> Self {
        match axis {
            Axis::X => {
                if matches!(self, Self::West | Self::East) {
                    self
                } else {
                    self.counter_clockwise_x()
                }
            }
            Axis::Y => {
                if matches!(self, Self::Up | Self::Down) {
                    self
                } else {
                    self.counter_clockwise_y_axis()
                }
            }
            Axis::Z => {
                if matches!(self, Self::North | Self::South) {
                    self
                } else {
                    self.counter_clockwise_z()
                }
            }
        }
    }
}
