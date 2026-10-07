//! Voxel shapes for the first P01.d geometry slice.
//!
//! A shape stores coordinate planes and the occupied cells between them. This
//! is enough to preserve Java's box, move, supported BooleanOp joins, single
//! shape collision, VoxelShape clip hits, closest points, face/occlusion, rotations, axis variants, bounds,
//! box iteration, object identity, and text formatting behavior
//! without choosing a broader collision representation yet.

use crate::aabb::{Aabb, BlockHitResult};
use crate::binary::BlockPos;
use crate::direction::{Axis, AxisDirection, Direction};
use crate::math::{Vec3, clamp_f64};
use std::sync::{Arc, OnceLock};

impl VoxelShape {
    pub(super) fn new(xs: Vec<f64>, ys: Vec<f64>, zs: Vec<f64>, cells: Vec<bool>) -> Self {
        debug_assert_eq!(cells.len(), Grid::from_coords(&xs, &ys, &zs).cell_count());
        Self {
            xs,
            ys,
            zs,
            cells,
            identity: Arc::new(()),
        }
    }

    /// A number no other live shape shares, like Java's identity hash: the
    /// key for caches the reference keeps per shape object.
    pub fn identity(&self) -> usize {
        Arc::as_ptr(&self.identity) as usize
    }

    /// Compare object identity like Java `VoxelShape.equals(Object)`.
    pub fn equals_java(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity)
    }

    /// Format the shape like Java `VoxelShape.toString()`.
    pub fn to_string_java(&self) -> String {
        if self.is_empty() {
            return "EMPTY".to_owned();
        }
        let bounds = self.bounds().expect("nonempty shape has bounds");
        format!("VoxelShape[{}]", bounds.to_string_java())
    }

    /// Return whether no voxel cell is occupied.
    pub fn is_empty(&self) -> bool {
        !self.cells.iter().any(|cell| *cell)
    }

    /// Move every coordinate plane by the supplied delta.
    pub fn move_by(&self, dx: f64, dy: f64, dz: f64) -> Self {
        if self.is_empty() {
            return Shapes::empty();
        }
        Self::new(
            self.xs.iter().map(|value| value + dx).collect(),
            self.ys.iter().map(|value| value + dy).collect(),
            self.zs.iter().map(|value| value + dz).collect(),
            self.cells.clone(),
        )
    }

    /// Move by a double-precision vector.
    pub fn move_by_vec3(&self, delta: Vec3) -> Self {
        self.move_by(delta.x, delta.y, delta.z)
    }

    /// Move by integer block coordinates converted to double precision.
    pub fn move_by_block_pos(&self, delta: BlockPos) -> Self {
        self.move_by(f64::from(delta.x), f64::from(delta.y), f64::from(delta.z))
    }

    /// Return the coordinate planes for an axis in ascending order.
    pub fn coords(&self, axis: Axis) -> &[f64] {
        self.axis_coords(axis)
    }

    /// Rebuild the shape from its merged boxes using Java's optimize order.
    pub fn optimize(&self) -> Self {
        let mut optimized = Shapes::empty();
        for bounds in self.to_aabbs() {
            let box_shape = Shapes::box_shape(
                bounds.min_x,
                bounds.min_y,
                bounds.min_z,
                bounds.max_x,
                bounds.max_y,
                bounds.max_z,
            )
            .expect("merged shape boxes have ordered bounds");
            optimized = if optimized.is_empty() {
                box_shape
            } else {
                Shapes::join_unoptimized(&optimized, &box_shape, BooleanOp::Or)
            };
        }
        optimized
    }
}

/// A voxelized region described by coordinate planes and occupied cells.
#[derive(Debug, Clone, PartialEq)]
pub struct VoxelShape {
    pub(super) xs: Vec<f64>,
    pub(super) ys: Vec<f64>,
    pub(super) zs: Vec<f64>,
    pub(super) cells: Vec<bool>,
    pub(super) identity: Arc<()>,
}

mod boolean_op;
pub use boolean_op::*;
mod edges;
mod grid;
use grid::*;
mod queries;
mod rotation;
pub use rotation::*;
mod shapes;
pub use shapes::*;
