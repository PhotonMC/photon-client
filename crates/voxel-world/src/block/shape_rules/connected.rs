//! The shapes of blocks that grow an arm towards each neighbour they
//! connect to: walls, fences and panes.

use std::sync::OnceLock;

use voxel_core::direction::Direction;
use voxel_core::shape::{Shapes, VoxelShape};
use voxel_data::block::properties::{
    EAST, EAST_WALL, NORTH, NORTH_WALL, SOUTH, SOUTH_WALL, UP, WEST, WEST_WALL,
};
use voxel_data::block::registry::Block;

use super::super::shapes::{box_z, column};
use super::arm;

/// `WallBlock.makeShapes`: the post and the low and tall arm of each side.
struct Wall {
    post: VoxelShape,
    low: [(Direction, VoxelShape); 4],
    tall: [(Direction, VoxelShape); 4],
}

impl Wall {
    fn new(post_height: f64, wall_top: f64) -> Self {
        Self {
            post: column(8.0, 8.0, 0.0, post_height),
            low: Shapes::rotate_horizontal(&box_z(6.0, 0.0, wall_top, 0.0, 11.0)),
            tall: Shapes::rotate_horizontal(&box_z(6.0, 0.0, post_height, 0.0, 11.0)),
        }
    }

    fn shape(&self, block: &Block, offset: u32) -> VoxelShape {
        let mut shape = if block.flag(offset, &UP) {
            self.post.clone()
        } else {
            Shapes::empty()
        };
        for (direction, property) in [
            (Direction::North, &NORTH_WALL),
            (Direction::East, &EAST_WALL),
            (Direction::South, &SOUTH_WALL),
            (Direction::West, &WEST_WALL),
        ] {
            let arms = match block.value(offset, property) {
                Some("low") => &self.low,
                Some("tall") => &self.tall,
                _ => continue,
            };
            shape = Shapes::or(&shape, arm(arms, direction));
        }
        shape
    }
}

/// `WallBlock.getShape`.
pub fn wall_outline(block: &Block, offset: u32) -> VoxelShape {
    static WALL: OnceLock<Wall> = OnceLock::new();
    WALL.get_or_init(|| Wall::new(16.0, 14.0))
        .shape(block, offset)
}

/// `WallBlock.getCollisionShape`.
pub fn wall_collision(block: &Block, offset: u32) -> VoxelShape {
    static WALL: OnceLock<Wall> = OnceLock::new();
    WALL.get_or_init(|| Wall::new(24.0, 24.0))
        .shape(block, offset)
}

/// `CrossCollisionBlock.makeShapes`: the post and an arm towards each
/// connected side.
struct Cross {
    post: VoxelShape,
    arms: [(Direction, VoxelShape); 4],
}

impl Cross {
    fn new(post_width: f64, post_height: f64, wall_width: f64, wall_top: f64) -> Self {
        Self {
            post: column(post_width, post_width, 0.0, post_height),
            arms: Shapes::rotate_horizontal(&box_z(wall_width, 0.0, wall_top, 0.0, 8.0)),
        }
    }

    fn shape(&self, block: &Block, offset: u32) -> VoxelShape {
        let mut shape = self.post.clone();
        for (direction, property) in [
            (Direction::North, &NORTH),
            (Direction::East, &EAST),
            (Direction::South, &SOUTH),
            (Direction::West, &WEST),
        ] {
            if block.flag(offset, property) {
                shape = Shapes::or(&shape, arm(&self.arms, direction));
            }
        }
        shape
    }
}

/// The two shapes a `CrossCollisionBlock` builds in its constructor: the
/// outline and the collision one.
struct CrossShapes {
    outline: Cross,
    collision: Cross,
}

/// `new CrossCollisionBlock(postWidth, postHeight, wallWidth, wallHeight,
/// collisionHeight)` of a fence or of a pane.
fn cross_shapes(fence: bool) -> &'static CrossShapes {
    static FENCE: OnceLock<CrossShapes> = OnceLock::new();
    static PANE: OnceLock<CrossShapes> = OnceLock::new();
    let (cell, post_width, post_height, wall_width, wall_height, collision) = if fence {
        (&FENCE, 4.0, 16.0, 4.0, 16.0, 24.0)
    } else {
        (&PANE, 2.0, 16.0, 2.0, 16.0, 16.0)
    };
    cell.get_or_init(|| CrossShapes {
        outline: Cross::new(post_width, post_height, wall_width, wall_height),
        collision: Cross::new(post_width, collision, wall_width, collision),
    })
}

/// `CrossCollisionBlock.getShape`.
pub fn cross_outline(block: &Block, offset: u32, fence: bool) -> VoxelShape {
    cross_shapes(fence).outline.shape(block, offset)
}

/// `CrossCollisionBlock.getCollisionShape`.
pub fn cross_collision(block: &Block, offset: u32, fence: bool) -> VoxelShape {
    cross_shapes(fence).collision.shape(block, offset)
}
