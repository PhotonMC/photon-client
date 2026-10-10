//! `ClientWorld` border layer split out of the former single-file module.

use super::*;

use photon_core::java_float::JavaF64;
use photon_core::math::{clamp_f64, mth_lerp_f64};
use photon_core::shape::VoxelShape;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientWorldBorderStatus {
    Stationary,
    Shrinking,
    Growing,
}

impl ClientWorldBorderStatus {
    /// `BorderStatus.getColor()`.
    pub fn color(self) -> i32 {
        match self {
            Self::Growing => 4_259_712,
            Self::Shrinking => 16_724_016,
            Self::Stationary => 2_138_367,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClientWorldBorderExtent {
    Static {
        size: JavaF64,
    },
    Moving {
        from: JavaF64,
        to: JavaF64,
        duration: i64,
        progress: i64,
        size: JavaF64,
        previous_size: JavaF64,
    },
}

/// Model the client-visible Java `WorldBorder` extent state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientWorldBorder {
    pub(crate) center_x: JavaF64,
    pub(crate) center_z: JavaF64,
    pub(crate) absolute_max_size: i32,
    pub(crate) warning_blocks: i32,
    pub(crate) warning_time: i32,
    pub(crate) extent: ClientWorldBorderExtent,
}

impl Default for ClientWorldBorder {
    fn default() -> Self {
        Self::new(0.0, 0.0, 29_999_984, f64::from(5.999997E7_f32))
    }
}

impl ClientWorldBorder {
    pub fn new(center_x: f64, center_z: f64, absolute_max_size: i32, size: f64) -> Self {
        Self {
            center_x: JavaF64::new(center_x),
            center_z: JavaF64::new(center_z),
            absolute_max_size,
            warning_blocks: 5,
            warning_time: 15,
            extent: ClientWorldBorderExtent::Static {
                size: JavaF64::new(size),
            },
        }
    }

    pub fn center_x(&self) -> f64 {
        self.center_x.get()
    }

    pub fn center_z(&self) -> f64 {
        self.center_z.get()
    }

    pub fn absolute_max_size(&self) -> i32 {
        self.absolute_max_size
    }

    pub fn warning_blocks(&self) -> i32 {
        self.warning_blocks
    }

    pub fn warning_time(&self) -> i32 {
        self.warning_time
    }

    pub fn set_center(&mut self, center_x: f64, center_z: f64) {
        self.center_x = JavaF64::new(center_x);
        self.center_z = JavaF64::new(center_z);
    }

    pub fn set_absolute_max_size(&mut self, absolute_max_size: i32) {
        self.absolute_max_size = absolute_max_size;
    }

    pub fn set_warning_blocks(&mut self, warning_blocks: i32) {
        self.warning_blocks = warning_blocks;
    }

    pub fn set_warning_time(&mut self, warning_time: i32) {
        self.warning_time = warning_time;
    }

    pub fn set_size(&mut self, size: f64) {
        self.extent = ClientWorldBorderExtent::Static {
            size: JavaF64::new(size),
        };
    }

    pub fn lerp_size_between(&mut self, from: f64, to: f64, duration: i64) {
        self.extent = if from == to {
            ClientWorldBorderExtent::Static {
                size: JavaF64::new(to),
            }
        } else {
            ClientWorldBorderExtent::Moving {
                from: JavaF64::new(from),
                to: JavaF64::new(to),
                duration,
                progress: duration,
                size: JavaF64::new(from),
                previous_size: JavaF64::new(from),
            }
        };
    }

    pub fn size(&self) -> f64 {
        match self.extent {
            ClientWorldBorderExtent::Static { size }
            | ClientWorldBorderExtent::Moving { size, .. } => size.get(),
        }
    }

    pub fn lerp_time(&self) -> i64 {
        match self.extent {
            ClientWorldBorderExtent::Static { .. } => 0,
            ClientWorldBorderExtent::Moving { progress, .. } => progress,
        }
    }

    pub fn lerp_target(&self) -> f64 {
        match self.extent {
            ClientWorldBorderExtent::Static { size } => size.get(),
            ClientWorldBorderExtent::Moving { to, .. } => to.get(),
        }
    }

    pub fn status(&self) -> ClientWorldBorderStatus {
        match self.extent {
            ClientWorldBorderExtent::Static { .. } => ClientWorldBorderStatus::Stationary,
            ClientWorldBorderExtent::Moving { from, to, .. } => {
                if to.get() < from.get() {
                    ClientWorldBorderStatus::Shrinking
                } else {
                    ClientWorldBorderStatus::Growing
                }
            }
        }
    }

    pub fn min_x(&self, partial_tick: f32) -> f64 {
        self.edge(self.center_x(), self.effective_size(partial_tick), false)
    }

    pub fn max_x(&self, partial_tick: f32) -> f64 {
        self.edge(self.center_x(), self.effective_size(partial_tick), true)
    }

    pub fn min_z(&self, partial_tick: f32) -> f64 {
        self.edge(self.center_z(), self.effective_size(partial_tick), false)
    }

    pub fn max_z(&self, partial_tick: f32) -> f64 {
        self.edge(self.center_z(), self.effective_size(partial_tick), true)
    }

    /// `WorldBorder.getDistanceToBorder`: the distance to the nearest edge,
    /// measured against the edges of this tick (no partial-tick lerp).
    pub fn distance_to_border(&self, x: f64, z: f64) -> f64 {
        let from_west = x - self.min_x(0.0);
        let from_east = self.max_x(0.0) - x;
        let from_north = z - self.min_z(0.0);
        let from_south = self.max_z(0.0) - z;
        let min = crate::java_math::min;
        min(min(min(from_west, from_east), from_north), from_south)
    }

    pub fn collision_shape(&self) -> VoxelShape {
        world_border_collision_shape(
            self.center_x(),
            self.center_z(),
            self.effective_size(0.0),
            self.absolute_max_size,
        )
    }

    pub fn tick(&mut self) {
        let ClientWorldBorderExtent::Moving {
            from,
            to,
            duration,
            progress,
            size: previous_size,
            ..
        } = self.extent
        else {
            return;
        };
        let progress = progress - 1;
        let size = calculate_world_border_size(from.get(), to.get(), duration, progress);
        self.extent = if progress <= 0 {
            ClientWorldBorderExtent::Static { size: to }
        } else {
            ClientWorldBorderExtent::Moving {
                from,
                to,
                duration,
                progress,
                size: JavaF64::new(size),
                previous_size,
            }
        };
    }

    pub(crate) fn effective_size(&self, partial_tick: f32) -> f64 {
        match self.extent {
            ClientWorldBorderExtent::Static { size } => size.get(),
            ClientWorldBorderExtent::Moving {
                size,
                previous_size,
                ..
            } => mth_lerp_f64(f64::from(partial_tick), previous_size.get(), size.get()),
        }
    }

    pub(crate) fn edge(&self, center: f64, size: f64, maximum: bool) -> f64 {
        let absolute_max_size = f64::from(self.absolute_max_size);
        let half_size = size / 2.0;
        let value = if maximum {
            center + half_size
        } else {
            center - half_size
        };
        clamp_f64(value, -absolute_max_size, absolute_max_size)
    }
}

/// Fields carried by Java's initialize-border packet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientWorldBorderInitialization {
    pub center_x: f64,
    pub center_z: f64,
    pub old_size: f64,
    pub new_size: f64,
    pub lerp_time: i64,
    pub absolute_max_size: i32,
    pub warning_blocks: i32,
    pub warning_time: i32,
}

pub(crate) fn calculate_world_border_size(from: f64, to: f64, duration: i64, progress: i64) -> f64 {
    let progress = (duration as f64 - progress as f64) / duration as f64;
    if progress < 1.0 {
        mth_lerp_f64(progress, from, to)
    } else {
        to
    }
}
