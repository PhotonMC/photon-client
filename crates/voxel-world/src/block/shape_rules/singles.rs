//! The shapes of blocks that take one of a few fixed shapes by a property or
//! two, or by the way they face.

use voxel_core::direction::{Axis, Direction};
use voxel_core::math::Vec3;
use voxel_core::shape::{BooleanOp, ShapeRotation, Shapes, VoxelShape};
use voxel_data::block::classes::BlockClass;
use voxel_data::block::registry::Block;

use super::super::shapes::{block_cube, box_z, box_z_sized, box_z_xy, column, column_xz, cube};
use super::{arm, facing, flag, number, prop};

type Horizontal = [(Direction, VoxelShape); 4];
type All = [(Direction, VoxelShape); 6];

struct Scaffolding {
    stable: VoxelShape,
    unstable: VoxelShape,
}

mod wire;
use wire::*;
mod outline;
pub use outline::*;
mod scaffolding;
pub use scaffolding::*;
