//! A mount the local player rides: the saddled horse family, as a client that is its authority
//! ticks it (`LivingEntity.aiStep` into `travelRidden`: the rider's keys steer it, and its own
//! attributes decide how fast and how high it goes). The movement is `LivingEntity`'s; the
//! player's own port of it is `LocalPlayer`'s.

use photon_core::aabb::Aabb;
use photon_core::binary::BlockPos;
use photon_core::java_float::{JavaF32, JavaF64};
use photon_core::math::{Vec3, ceil_f64, equal_f64, floor_f64, mth_cos_f64, mth_sin_f64};
use photon_data::block::definition::Fluid;

use crate::ClientWorld;
use crate::collision::{collide_entity_movement, select_step_up_movement};
use crate::entity::{ClientEntityDimensions, Support};
use crate::fluid::{self, FluidTracker, LAVA_FLOW_SCALE, Pushed, WATER_FLOW_SCALE};
use crate::java_math;
use crate::local_player::{
    InsideBody, InsideVisit, Level, LivingEffects, Movement, Unsupported,
    compute_modified_friction, flag, living_inside_effect, property, walk_inside_blocks,
};
pub(crate) use body::*;
pub(crate) use camel::*;

mod body;
mod camel;
mod riding;
mod state;
pub use state::*;
mod travel;
mod world;
