//! Cardinal directions for the pinned reference (P01.c).
//!
//! Covers names, data values, axes, steps, opposites, rotations and quaternion
//! orientation values. Entity-lookahead and codecs belong to later slices.

use crate::math::{Quaternionf, Vec3, Vec3f};
use crate::random::RandomSource;

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

mod axis;
pub use axis::*;
mod plane;
pub use plane::*;
mod rotation;
pub use rotation::*;
