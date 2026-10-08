//! The shapes of blocks that turn on a hinge or a facing: stairs, doors and
//! trapdoors.

use std::sync::OnceLock;

use photon_core::direction::Direction;
use photon_core::shape::{ShapeRotation, Shapes, VoxelShape};
use photon_data::block::properties::{DOOR_HINGE, HALF, OPEN, STAIRS_SHAPE};
use photon_data::block::registry::Block;

use super::super::shapes::{box_z_xy, column_xz, cube};
use super::{arm, facing};

type Ring = [(Direction, VoxelShape); 4];

/// `StairBlock`'s outer, straight and inner shapes, for each facing, upright
/// and upside down.
struct Stairs {
    bottom: [Ring; 3],
    top: [Ring; 3],
}

impl Stairs {
    fn new() -> Self {
        let outer = Shapes::or(
            &column_xz(16.0, 0.0, 8.0),
            &cube(0.0, 8.0, 0.0, 8.0, 16.0, 8.0),
        );
        let straight = Shapes::or(&outer, &Shapes::rotate(&outer, ShapeRotation::Rot90YNeg));
        let inner = Shapes::or(
            &straight,
            &Shapes::rotate(&straight, ShapeRotation::Rot90YNeg),
        );
        let rings = |initial| {
            [&straight, &outer, &inner]
                .map(|shape| Shapes::rotate_horizontal_with_initial(shape, initial))
        };
        Self {
            bottom: rings(ShapeRotation::Identity),
            top: rings(ShapeRotation::InvertY),
        }
    }
}

/// `StairBlock.getShape`.
pub fn stairs_outline(block: &Block, offset: u32) -> VoxelShape {
    static STAIRS: OnceLock<Stairs> = OnceLock::new();
    let stairs = STAIRS.get_or_init(Stairs::new);
    let facing = facing(block, offset, "facing");
    let rings = if block.value(offset, &HALF) == Some("bottom") {
        &stairs.bottom
    } else {
        &stairs.top
    };
    let shape = block.value(offset, &STAIRS_SHAPE).unwrap_or("straight");
    let ring = match shape {
        "straight" => &rings[0],
        "outer_left" | "outer_right" => &rings[1],
        _ => &rings[2],
    };
    let direction = match shape {
        "inner_left" => facing.counter_clockwise_y(),
        "outer_right" => facing.clockwise_y(),
        _ => Ok(facing),
    }
    .expect("a horizontal facing turns about y");
    arm(ring, direction).clone()
}

/// `DoorBlock.getShape`.
pub fn door_outline(block: &Block, offset: u32) -> VoxelShape {
    static SHAPES: OnceLock<Ring> = OnceLock::new();
    let shapes = SHAPES.get_or_init(|| Shapes::rotate_horizontal(&box_z_xy(16.0, 13.0, 16.0)));
    let facing = facing(block, offset, "facing");
    let door = if block.flag(offset, &OPEN) {
        if block.value(offset, &DOOR_HINGE) == Some("right") {
            facing.counter_clockwise_y()
        } else {
            facing.clockwise_y()
        }
        .expect("a horizontal facing turns about y")
    } else {
        facing
    };
    arm(shapes, door).clone()
}

/// `TrapDoorBlock.getShape`.
pub fn trapdoor_outline(block: &Block, offset: u32) -> VoxelShape {
    static SHAPES: OnceLock<[(Direction, VoxelShape); 6]> = OnceLock::new();
    let shapes = SHAPES.get_or_init(|| Shapes::rotate_all(&box_z_xy(16.0, 13.0, 16.0)));
    let direction = if block.flag(offset, &OPEN) {
        facing(block, offset, "facing")
    } else if block.value(offset, &HALF) == Some("top") {
        Direction::Down
    } else {
        Direction::Up
    };
    arm(shapes, direction).clone()
}
