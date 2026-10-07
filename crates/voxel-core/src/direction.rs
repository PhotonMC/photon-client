//! Cardinal directions for the pinned reference (P01.c).
//!
//! Covers names, data values, axes, steps, opposites, rotations and quaternion
//! orientation values. Entity-lookahead and codecs belong to later slices.

use crate::math::{Quaternionf, Vec3, Vec3f};
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

impl Axis {
    /// Return Java `Enum.name()` for this axis.
    pub fn enum_name(self) -> &'static str {
        match self {
            Self::X => "X",
            Self::Y => "Y",
            Self::Z => "Z",
        }
    }

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
    /// Return Java `Enum.name()` for this axis direction.
    pub fn enum_name(self) -> &'static str {
        match self {
            Self::Positive => "POSITIVE",
            Self::Negative => "NEGATIVE",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Positive => "Towards positive",
            Self::Negative => "Towards negative",
        }
    }

    /// Return Java `AxisDirection.toString()`.
    pub fn to_string_java(self) -> &'static str {
        self.name()
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
    /// Return Java `Enum.name()` for this plane.
    pub fn enum_name(self) -> &'static str {
        match self {
            Self::Horizontal => "HORIZONTAL",
            Self::Vertical => "VERTICAL",
        }
    }

    /// Return Java `Plane.toString()`, which inherits `Enum.toString()`.
    pub fn to_string_java(self) -> &'static str {
        self.enum_name()
    }

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

mod direction_clockwise;
