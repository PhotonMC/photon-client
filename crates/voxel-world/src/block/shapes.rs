//! Shapes blocks build in code: `Block.box`, `Block.column` and
//! `Block.boxZ`, which measure in sixteenths of a block, and the class
//! constants the state cache reads (occlusion and support shapes).

use std::sync::OnceLock;

use voxel_core::direction::{Axis, Direction};
use voxel_core::shape::{Shapes, VoxelShape};

/// `Block.box`: a box in sixteenths.
pub fn cube(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> VoxelShape {
    Shapes::box_shape(
        min_x / 16.0,
        min_y / 16.0,
        min_z / 16.0,
        max_x / 16.0,
        max_y / 16.0,
        max_z / 16.0,
    )
    .expect("a block box has its corners in order")
}

/// `Block.column`: a box centred on the block's vertical axis.
pub fn column(size_x: f64, size_z: f64, min_y: f64, max_y: f64) -> VoxelShape {
    let half_x = size_x / 2.0;
    let half_z = size_z / 2.0;
    cube(
        8.0 - half_x,
        min_y,
        8.0 - half_z,
        8.0 + half_x,
        max_y,
        8.0 + half_z,
    )
}

/// `Block.boxZ`: a box centred across x, running along z.
pub fn box_z(size_x: f64, min_y: f64, max_y: f64, min_z: f64, max_z: f64) -> VoxelShape {
    let half_x = size_x / 2.0;
    cube(8.0 - half_x, min_y, min_z, 8.0 + half_x, max_y, max_z)
}

fn by_axis(shapes: &[(Axis, VoxelShape); 2], axis: Axis) -> &VoxelShape {
    &shapes
        .iter()
        .find(|(own, _)| *own == axis)
        .expect("both horizontal axes are listed")
        .1
}

/// `FenceBlock.occlusionShapes`: `makeShapes(4, 16, 2, 6, 15)`, the post and
/// an arm towards each connected side.
pub fn fence_occlusion(connected: impl Fn(Direction) -> bool) -> VoxelShape {
    static PARTS: OnceLock<(VoxelShape, [(Direction, VoxelShape); 4])> = OnceLock::new();
    let (post, arms) = PARTS.get_or_init(|| {
        (
            column(4.0, 4.0, 0.0, 16.0),
            Shapes::rotate_horizontal(&box_z(2.0, 6.0, 15.0, 0.0, 8.0)),
        )
    });
    arms.iter()
        .filter(|(direction, _)| connected(*direction))
        .fold(post.clone(), |shape, (_, arm)| Shapes::or(&shape, arm))
}

/// `FenceGateBlock.SHAPE_OCCLUSION` and `SHAPE_OCCLUSION_WALL`.
pub fn fence_gate_occlusion(axis: Axis, in_wall: bool) -> VoxelShape {
    static SHAPES: OnceLock<[[(Axis, VoxelShape); 2]; 2]> = OnceLock::new();
    let [free, wall] = SHAPES.get_or_init(|| {
        let free = Shapes::rotate_horizontal_axis(&Shapes::or(
            &cube(0.0, 5.0, 7.0, 2.0, 16.0, 9.0),
            &cube(14.0, 5.0, 7.0, 16.0, 16.0, 9.0),
        ));
        let wall = free
            .clone()
            .map(|(axis, shape)| (axis, shape.move_by(0.0, -0.1875, 0.0).optimize()));
        [free, wall]
    });
    by_axis(if in_wall { wall } else { free }, axis).clone()
}

/// `FenceGateBlock.SHAPE_SUPPORT`.
pub fn fence_gate_support(axis: Axis) -> VoxelShape {
    static SHAPES: OnceLock<[(Axis, VoxelShape); 2]> = OnceLock::new();
    let shapes =
        SHAPES.get_or_init(|| Shapes::rotate_horizontal_axis(&column(16.0, 4.0, 5.0, 24.0)));
    by_axis(shapes, axis).clone()
}

/// `ChorusFlowerBlock.SHAPE_BLOCK_SUPPORT`.
pub fn chorus_flower_support() -> VoxelShape {
    static SHAPE: OnceLock<VoxelShape> = OnceLock::new();
    SHAPE.get_or_init(|| column(14.0, 14.0, 0.0, 15.0)).clone()
}
