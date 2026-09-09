//! Cardinal directions for the pinned reference (P01.c).
//!
//! Covers names, data values, axes, steps, opposites and axial rotations.
//! Entity-lookahead, quaternions and codecs belong to later slices.

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

    /// Choose the direction with the greatest Java float dot product.
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

    fn clockwise_y(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
            other => other,
        }
    }

    fn counter_clockwise_y(self) -> Self {
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
                    self.clockwise_y()
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
                    self.counter_clockwise_y()
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
}

impl AxisDirection {
    pub fn name(self) -> &'static str {
        match self {
            Self::Positive => "Towards positive",
            Self::Negative => "Towards negative",
        }
    }
}
