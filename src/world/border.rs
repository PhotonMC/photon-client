//! `ClientWorld` border layer split out of the former single-file module.


use super::*;


use crate::math::{
    clamp_f64, mth_lerp_f64,
};
use crate::shape::VoxelShape;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientWorldBorderStatus {
    Stationary,
    Shrinking,
    Growing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClientWorldBorderExtent {
    Static {
        size_bits: u64,
    },
    Moving {
        from_bits: u64,
        to_bits: u64,
        duration: i64,
        progress: i64,
        size_bits: u64,
        previous_size_bits: u64,
    },
}

/// Model the client-visible Java `WorldBorder` extent state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientWorldBorder {
    pub(crate) center_x_bits: u64,
    pub(crate) center_z_bits: u64,
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
            center_x_bits: center_x.to_bits(),
            center_z_bits: center_z.to_bits(),
            absolute_max_size,
            warning_blocks: 5,
            warning_time: 15,
            extent: ClientWorldBorderExtent::Static {
                size_bits: size.to_bits(),
            },
        }
    }

    pub fn center_x(&self) -> f64 {
        f64::from_bits(self.center_x_bits)
    }

    pub fn center_z(&self) -> f64 {
        f64::from_bits(self.center_z_bits)
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
        self.center_x_bits = center_x.to_bits();
        self.center_z_bits = center_z.to_bits();
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
            size_bits: size.to_bits(),
        };
    }

    pub fn lerp_size_between(&mut self, from: f64, to: f64, duration: i64) {
        self.extent = if from == to {
            ClientWorldBorderExtent::Static {
                size_bits: to.to_bits(),
            }
        } else {
            ClientWorldBorderExtent::Moving {
                from_bits: from.to_bits(),
                to_bits: to.to_bits(),
                duration,
                progress: duration,
                size_bits: from.to_bits(),
                previous_size_bits: from.to_bits(),
            }
        };
    }

    pub fn size(&self) -> f64 {
        match self.extent {
            ClientWorldBorderExtent::Static { size_bits }
            | ClientWorldBorderExtent::Moving { size_bits, .. } => f64::from_bits(size_bits),
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
            ClientWorldBorderExtent::Static { size_bits } => f64::from_bits(size_bits),
            ClientWorldBorderExtent::Moving { to_bits, .. } => f64::from_bits(to_bits),
        }
    }

    pub fn status(&self) -> ClientWorldBorderStatus {
        match self.extent {
            ClientWorldBorderExtent::Static { .. } => ClientWorldBorderStatus::Stationary,
            ClientWorldBorderExtent::Moving {
                from_bits, to_bits, ..
            } => {
                if f64::from_bits(to_bits) < f64::from_bits(from_bits) {
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
            from_bits,
            to_bits,
            duration,
            progress,
            size_bits,
            ..
        } = self.extent
        else {
            return;
        };
        let progress = progress - 1;
        let previous_size_bits = size_bits;
        let size = calculate_world_border_size(
            f64::from_bits(from_bits),
            f64::from_bits(to_bits),
            duration,
            progress,
        );
        self.extent = if progress <= 0 {
            ClientWorldBorderExtent::Static { size_bits: to_bits }
        } else {
            ClientWorldBorderExtent::Moving {
                from_bits,
                to_bits,
                duration,
                progress,
                size_bits: size.to_bits(),
                previous_size_bits,
            }
        };
    }

    pub(crate) fn effective_size(&self, partial_tick: f32) -> f64 {
        match self.extent {
            ClientWorldBorderExtent::Static { size_bits } => f64::from_bits(size_bits),
            ClientWorldBorderExtent::Moving {
                size_bits,
                previous_size_bits,
                ..
            } => mth_lerp_f64(
                f64::from(partial_tick),
                f64::from_bits(previous_size_bits),
                f64::from_bits(size_bits),
            ),
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
