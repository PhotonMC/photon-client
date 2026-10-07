//! The shulker box block entity (`ShulkerBoxRenderer`): the texture of its colour, the pose its
//! facing turns the model into, and the lid that slides up and turns as it opens.

use voxel_core::direction::Direction;

use crate::joml::{Matrix4, Pose, Quaternion};

/// The model layer of every shulker box.
pub const LAYER: &str = "minecraft:shulker_box#main";

/// The texture under `textures/` of a box of a colour (`Sheets.getShulkerBoxSprite`), or of the
/// undyed one.
pub fn texture(color: Option<&str>) -> String {
    match color {
        Some(color) => format!("entity/shulker/shulker_{color}"),
        None => "entity/shulker/shulker".to_owned(),
    }
}

/// `Direction.getRotation`.
fn rotation(facing: Direction) -> Quaternion {
    use std::f32::consts::PI;
    let half = (std::f64::consts::PI / 2.0) as f32;
    match facing {
        Direction::Down => Quaternion::rotation_x(PI),
        Direction::Up => Quaternion::identity(),
        Direction::North => Quaternion::rotation_xyz(half, 0.0, PI),
        Direction::South => Quaternion::rotation_x(half),
        Direction::West => Quaternion::rotation_xyz(half, 0.0, half),
        Direction::East => Quaternion::rotation_xyz(half, 0.0, -half),
    }
}

/// `ShulkerBoxRenderer.createModelTransform`: the box turned from "up" to its facing, a hair
/// smaller than the block.
pub fn model_transformation(facing: Direction) -> Matrix4 {
    let mut matrix = Matrix4::translation(0.5, 0.5, 0.5);
    matrix.scale(0.9995, 0.9995, 0.9995);
    matrix.rotate(&rotation(facing));
    matrix.scale(1.0, -1.0, -1.0);
    matrix.translate(0.0, -1.0, 0.0);
    // `MatrixUtil.isPureTranslation` works the flags of the transformation's matrix out the first
    // time it is used.
    matrix.determine_properties();
    matrix
}

/// The pose a box is drawn from, with `base` the pose of the block's corner.
pub fn box_pose(base: &Pose, facing: Direction) -> Pose {
    let mut pose = *base;
    pose.mul_matrix(&model_transformation(facing));
    pose
}
