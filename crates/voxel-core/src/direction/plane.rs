use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plane {
    Horizontal,
    Vertical,
}

pub(super) fn shuffled_copy<T: Copy>(items: &[T], random: &mut RandomSource) -> Vec<T> {
    let mut copy = items.to_vec();
    for index in (1..copy.len()).rev() {
        let swap_to = random
            .next_int_bound(index as i32 + 1)
            .expect("shuffle bound is positive") as usize;
        copy.swap(index, swap_to);
    }
    copy
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
