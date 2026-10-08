//! A boat the local player rides: `AbstractBoat.tick` on a client that is its authority. The
//! rider's input steers it (`controlBoat`), the water, the land or the air it is in decides how it
//! floats and slows (`getStatus`, `floatBoat`), and `Entity.move` carries it through the blocks.
//! What the boat does to other entities it meets (pushing them) is not carried.

use photon_core::aabb::Aabb;
use photon_core::binary::BlockPos;
use photon_core::java_float::{JavaF32, JavaF64};
use photon_core::math::{Vec3, ceil_f64, floor_f64, mth_cos_f64, mth_sin_f64};
use photon_core::shape::{BooleanOp, Shapes, VoxelShape};
use photon_data::block::definition::Fluid;

use crate::ClientWorld;
use crate::collision::collide_entity_movement;
use crate::entity::{BoatInput, ClientEntityDimensions, Support};
use crate::fluid::{self, LAVA_FLOW_SCALE, Pushed, WATER_FLOW_SCALE};
use crate::java_math;
use crate::local_player::{
    InsideBody, InsideVisit, Level, Movement, Unsupported, flag, walk_inside_blocks,
};
pub(crate) use body::*;

mod body;
mod physics;
mod state;
pub use state::*;
mod world;
