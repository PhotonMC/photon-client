use super::*;

/// A horizontal facing's shape from a map built once.
pub(super) fn by_facing(block: &Block, offset: u32, ring: &'static Horizontal) -> VoxelShape {
    arm(ring, facing(block, offset, "facing")).clone()
}

pub(super) fn by_horizontal_axis(
    block: &Block,
    offset: u32,
    rings: &'static [(Axis, VoxelShape); 2],
) -> VoxelShape {
    arm(rings, facing(block, offset, "facing").axis()).clone()
}

/// Which way a segmented block's shape is turned: `SegmentableBlock`.
pub(super) fn segments(block: &Block, offset: u32, height: f64, amount: &str) -> VoxelShape {
    let shapes = Shapes::rotate_horizontal(&cube(0.0, 0.0, 0.0, 8.0, height, 8.0));
    let mut direction = facing(block, offset, "facing");
    let mut shape = Shapes::empty();
    for _ in 0..number(block, offset, amount) {
        shape = Shapes::or(&shape, arm(&shapes, direction));
        direction = direction
            .counter_clockwise_y()
            .expect("a horizontal facing turns about y");
    }
    shape.single_encompassing()
}

/// The faces a block of vines, lichen or fire grows on.
pub(super) fn faces(block: &Block, offset: u32) -> impl Iterator<Item = Direction> + '_ {
    [
        (Direction::North, "north"),
        (Direction::East, "east"),
        (Direction::South, "south"),
        (Direction::West, "west"),
        (Direction::Up, "up"),
        (Direction::Down, "down"),
    ]
    .into_iter()
    .filter(move |(_, name)| flag(block, offset, name))
    .map(|(direction, _)| direction)
}

pub(super) fn plate(block: &Block, offset: u32) -> VoxelShape {
    let pressed = flag(block, offset, "powered")
        || prop(block, offset, "power").is_some_and(|power| power != "0");
    if pressed {
        cached!(VoxelShape, column_xz(14.0, 0.0, 0.5)).clone()
    } else {
        cached!(VoxelShape, column_xz(14.0, 0.0, 1.0)).clone()
    }
}

pub(super) fn redstone_wire(block: &Block, offset: u32) -> VoxelShape {
    struct Wire {
        dot: VoxelShape,
        floor: Horizontal,
        up: Horizontal,
    }
    let wire = cached!(
        Wire,
        Wire {
            dot: column_xz(10.0, 0.0, 1.0),
            floor: Shapes::rotate_horizontal(&box_z(10.0, 0.0, 1.0, 0.0, 8.0)),
            up: Shapes::rotate_horizontal(&box_z_sized(10.0, 16.0, 0.0, 1.0)),
        }
    );
    let mut shape = wire.dot.clone();
    for (direction, name) in [
        (Direction::North, "north"),
        (Direction::East, "east"),
        (Direction::South, "south"),
        (Direction::West, "west"),
    ] {
        match prop(block, offset, name) {
            Some("up") => {
                shape = Shapes::or_many(
                    &shape,
                    &[
                        arm(&wire.floor, direction).clone(),
                        arm(&wire.up, direction).clone(),
                    ],
                );
            }
            Some("side") => shape = Shapes::or(&shape, arm(&wire.floor, direction)),
            _ => {}
        }
    }
    shape
}

pub(super) fn attach_face(
    block: &Block,
    offset: u32,
    rings: &'static [(photon_core::shape::AttachFace, Horizontal); 3],
) -> VoxelShape {
    let face = match prop(block, offset, "face") {
        Some("floor") => photon_core::shape::AttachFace::Floor,
        Some("wall") => photon_core::shape::AttachFace::Wall,
        _ => photon_core::shape::AttachFace::Ceiling,
    };
    let ring = &rings
        .iter()
        .find(|(own, _)| *own == face)
        .expect("every attach face is listed")
        .1;
    arm(ring, facing(block, offset, "facing")).clone()
}
