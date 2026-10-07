use super::*;

/// Integer-aligned bounds used by structure and world-generation geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox {
    pub min_x: i32,
    pub min_y: i32,
    pub min_z: i32,
    pub max_x: i32,
    pub max_y: i32,
    pub max_z: i32,
}

/// A block hit produced by AABB or VoxelShape clipping.
#[derive(Debug, Clone, Copy)]
pub struct BlockHitResult {
    pub location: Vec3,
    pub direction: Direction,
    pub block_pos: BlockPos,
    pub inside: bool,
    pub world_border_hit: bool,
}

impl BoundingBox {
    /// Construct bounds, ordering each axis as Java's BoundingBox does.
    pub fn new(min_x: i32, min_y: i32, min_z: i32, max_x: i32, max_y: i32, max_z: i32) -> Self {
        Self {
            min_x: min_x.min(max_x),
            min_y: min_y.min(max_y),
            min_z: min_z.min(max_z),
            max_x: min_x.max(max_x),
            max_y: min_y.max(max_y),
            max_z: min_z.max(max_z),
        }
    }
}

impl BlockHitResult {
    pub(crate) fn new(
        location: Vec3,
        direction: Direction,
        block_pos: BlockPos,
        inside: bool,
    ) -> Self {
        Self {
            location,
            direction,
            block_pos,
            inside,
            world_border_hit: false,
        }
    }
}
