//! Cyclic axis permutations used by the pinned Java core helpers.

use crate::direction::Axis;

/// Permute coordinates around the X/Y/Z axis cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxisCycle {
    None,
    Forward,
    Backward,
}

impl AxisCycle {
    /// Java `AxisCycle.VALUES` order.
    pub const VALUES: [Self; 3] = [Self::None, Self::Forward, Self::Backward];

    /// Return the Java enum-style lowercase name used by behavior adapters.
    pub const fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Forward => "forward",
            Self::Backward => "backward",
        }
    }

    /// Cycle integer components for one selected output axis.
    pub fn cycle_i32(self, x: i32, y: i32, z: i32, axis: Axis) -> i32 {
        match self {
            Self::None => axis.choose_i32(x, y, z),
            Self::Forward => axis.choose_i32(z, x, y),
            Self::Backward => axis.choose_i32(y, z, x),
        }
    }

    /// Cycle double components for one selected output axis.
    pub fn cycle_f64(self, x: f64, y: f64, z: f64, axis: Axis) -> f64 {
        match self {
            Self::None => axis.choose_f64(x, y, z),
            Self::Forward => axis.choose_f64(z, x, y),
            Self::Backward => axis.choose_f64(y, z, x),
        }
    }

    /// Cycle one axis label in the X/Y/Z order.
    pub const fn cycle_axis(self, axis: Axis) -> Axis {
        match self {
            Self::None => axis,
            Self::Forward => match axis {
                Axis::X => Axis::Y,
                Axis::Y => Axis::Z,
                Axis::Z => Axis::X,
            },
            Self::Backward => match axis {
                Axis::X => Axis::Z,
                Axis::Y => Axis::X,
                Axis::Z => Axis::Y,
            },
        }
    }

    /// Return the inverse coordinate cycle.
    pub const fn inverse(self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }

    /// Return the cycle that maps a source axis to a target axis.
    pub const fn between(from: Axis, to: Axis) -> Self {
        let difference = (axis_ordinal(to) - axis_ordinal(from)).rem_euclid(3);
        match difference {
            0 => Self::None,
            1 => Self::Forward,
            2 => Self::Backward,
            _ => unreachable!(),
        }
    }
}

const fn axis_ordinal(axis: Axis) -> i32 {
    match axis {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Z => 2,
    }
}
