//! Cardinal directions for the pinned reference (P01.c).
//!
//! Covers names, data values, axes, steps, opposites and axial rotations.
//! Entity-lookahead, quaternions and codecs belong to later slices.

use crate::math::{Vec3, Vec3f};
use crate::random::RandomSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionYRotError {
    Vertical(Direction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionYRotationError {
    ClockwiseVertical(Direction),
    CounterClockwiseVertical(Direction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisDirection {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plane {
    Horizontal,
    Vertical,
}

const DIRECTIONS: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

fn shuffled_copy<T: Copy>(items: &[T], random: &mut RandomSource) -> Vec<T> {
    let mut copy = items.to_vec();
    for index in (1..copy.len()).rev() {
        let swap_to = random
            .next_int_bound(index as i32 + 1)
            .expect("shuffle bound is positive") as usize;
        copy.swap(index, swap_to);
    }
    copy
}

impl Direction {
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
        const DEGREES_TO_RADIANS_F32: f32 = f32::from_bits(0x3c8e_fa35);
        let radians = y_angle * DEGREES_TO_RADIANS_F32;
        let dx = -crate::math::mth_sin_f64(f64::from(radians));
        let dz = crate::math::mth_cos_f64(f64::from(radians));
        let (normal_x, _, normal_z) = self.step();
        normal_x as f32 * dx + normal_z as f32 * dz > 0.0
    }

    /// Return Java `Direction.getApproximateNearest(double, double, double)`.
    pub fn approximate_nearest(dx: f64, dy: f64, dz: f64) -> Self {
        let dx = dx as f32;
        let dy = dy as f32;
        let dz = dz as f32;
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

    fn clockwise_y_axis(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
            other => other,
        }
    }

    fn counter_clockwise_y_axis(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
            other => other,
        }
    }

    fn clockwise_x(self) -> Self {
        match self {
            Self::Down => Self::South,
            Self::Up => Self::North,
            Self::North => Self::Down,
            Self::South => Self::Up,
            other => other,
        }
    }

    fn counter_clockwise_x(self) -> Self {
        match self {
            Self::Down => Self::North,
            Self::Up => Self::South,
            Self::North => Self::Up,
            Self::South => Self::Down,
            other => other,
        }
    }

    fn clockwise_z(self) -> Self {
        match self {
            Self::Down => Self::West,
            Self::Up => Self::East,
            Self::West => Self::Up,
            Self::East => Self::Down,
            other => other,
        }
    }

    fn counter_clockwise_z(self) -> Self {
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

impl Axis {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "x" => Some(Self::X),
            "y" => Some(Self::Y),
            "z" => Some(Self::Z),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::X => "x",
            Self::Y => "y",
            Self::Z => "z",
        }
    }

    /// Return Java `Direction.Axis.isVertical()`.
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Y)
    }

    /// Return Java `Direction.Axis.isHorizontal()`.
    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::X | Self::Z)
    }

    /// Return Java `Direction.Axis.getPositive()`.
    pub fn get_positive(self) -> Direction {
        match self {
            Self::X => Direction::East,
            Self::Y => Direction::Up,
            Self::Z => Direction::South,
        }
    }

    /// Return Java `Direction.Axis.getNegative()`.
    pub fn get_negative(self) -> Direction {
        match self {
            Self::X => Direction::West,
            Self::Y => Direction::Down,
            Self::Z => Direction::North,
        }
    }

    /// Return Java `Direction.Axis.getDirections()` in positive/negative order.
    pub fn get_directions(self) -> [Direction; 2] {
        [self.get_positive(), self.get_negative()]
    }

    /// Return Java `Direction.Axis.getRandom(RandomSource)` using enum order.
    pub fn get_random(random: &mut RandomSource) -> Self {
        const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];
        AXES[random
            .next_int_bound(AXES.len() as i32)
            .expect("axis random bound is positive") as usize]
    }

    /// Return Java `Direction.Axis.choose(int, int, int)`.
    pub fn choose_i32(self, x: i32, y: i32, z: i32) -> i32 {
        match self {
            Self::X => x,
            Self::Y => y,
            Self::Z => z,
        }
    }

    /// Return Java `Direction.Axis.choose(double, double, double)`.
    pub fn choose_f64(self, x: f64, y: f64, z: f64) -> f64 {
        match self {
            Self::X => x,
            Self::Y => y,
            Self::Z => z,
        }
    }

    /// Return Java `Direction.Axis.choose(boolean, boolean, boolean)`.
    pub fn choose_bool(self, x: bool, y: bool, z: bool) -> bool {
        match self {
            Self::X => x,
            Self::Y => y,
            Self::Z => z,
        }
    }

    /// Return Java `Direction.Axis.test(Direction)` with nullable input.
    pub fn test(self, input: Option<Direction>) -> bool {
        input.is_some_and(|direction| direction.axis() == self)
    }

    /// Return Java `Direction.Axis.getPlane()`.
    pub fn get_plane(self) -> Plane {
        match self {
            Self::X | Self::Z => Plane::Horizontal,
            Self::Y => Plane::Vertical,
        }
    }
}

impl AxisDirection {
    pub fn name(self) -> &'static str {
        match self {
            Self::Positive => "Towards positive",
            Self::Negative => "Towards negative",
        }
    }

    /// Return Java `Direction.AxisDirection.getStep()`.
    pub fn get_step(self) -> i32 {
        match self {
            Self::Positive => 1,
            Self::Negative => -1,
        }
    }

    /// Return Java `Direction.AxisDirection.opposite()`.
    pub fn opposite(self) -> Self {
        match self {
            Self::Positive => Self::Negative,
            Self::Negative => Self::Positive,
        }
    }
}

impl Plane {
    /// Return Java `Direction.Plane.getRandomDirection(RandomSource)`.
    pub fn get_random_direction(self, random: &mut RandomSource) -> Direction {
        let directions = self.directions();
        directions[random
            .next_int_bound(directions.len() as i32)
            .expect("plane direction random bound is positive") as usize]
    }

    /// Return Java `Direction.Plane.getRandomAxis(RandomSource)`.
    pub fn get_random_axis(self, random: &mut RandomSource) -> Axis {
        let axes = self.axes();
        axes[random
            .next_int_bound(axes.len() as i32)
            .expect("plane axis random bound is positive") as usize]
    }

    /// Return Java `Direction.Plane.shuffledCopy(RandomSource)`.
    pub fn shuffled_copy(self, random: &mut RandomSource) -> Vec<Direction> {
        shuffled_copy(self.directions(), random)
    }

    /// Return Java `Direction.Plane.test(Direction)` with nullable input.
    pub fn test(self, input: Option<Direction>) -> bool {
        input.is_some_and(|direction| direction.axis().get_plane() == self)
    }

    /// Return the Java Plane face order used by iteration and streams.
    pub fn directions(self) -> &'static [Direction] {
        match self {
            Self::Horizontal => &[
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ],
            Self::Vertical => &[Direction::Up, Direction::Down],
        }
    }

    /// Return the Java Plane axis order used by random-axis selection.
    pub fn axes(self) -> &'static [Axis] {
        match self {
            Self::Horizontal => &[Axis::X, Axis::Z],
            Self::Vertical => &[Axis::Y],
        }
    }

    /// Return Java `Direction.Plane.length()`.
    pub fn length(self) -> usize {
        self.directions().len()
    }
}
