//! A display entity (`DisplayRenderer`): the matrix of its transformation, and where its block or
//! item is drawn.

use voxel_world::display::{Billboard, Transformation};

use crate::joml::{Matrix4, Pose, Quaternion};

/// `Transformation`'s matrix (`compose`): the translation, the left turn, the scale and the right
/// turn, in that order.
pub fn transformation_matrix_of(transformation: &Transformation) -> Matrix4 {
    let quaternion = |[x, y, z, w]: [f32; 4]| Quaternion { x, y, z, w };
    let [tx, ty, tz] = transformation.translation;
    let [sx, sy, sz] = transformation.scale;
    let mut matrix = Matrix4::translation(tx, ty, tz);
    matrix.rotate(&quaternion(transformation.left_rotation));
    matrix.scale(sx, sy, sz);
    matrix.rotate(&quaternion(transformation.right_rotation));
    matrix
}

/// The matrix as the 16 numbers `Matrix4f.get(float[])` gives (column by column).
pub fn transformation_matrix(transformation: &Transformation) -> [f32; 16] {
    let matrix = transformation_matrix_of(transformation);
    let mut out = [0.0; 16];
    for (index, value) in matrix.m.iter().flatten().enumerate() {
        out[index] = *value;
    }
    out
}

/// The turns the camera and the entity have, in degrees: yaw and pitch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisplayTurns {
    pub entity: (f32, f32),
    pub camera: (f32, f32),
}

/// `DisplayRenderer.calculateOrientation`: how the display faces, from the entity's turn or the
/// camera's as its billboard constraint says.
pub fn display_orientation(billboard: Billboard, turns: &DisplayTurns) -> Quaternion {
    let to_radians = (std::f64::consts::PI / 180.0) as f32;
    let negative = (-std::f64::consts::PI / 180.0) as f32;
    let (entity_y, entity_x) = turns.entity;
    let (camera_y, camera_x) = turns.camera;
    // `transformYRot` and `transformXRot`.
    let camera_y = camera_y - 180.0;
    let camera_x = -camera_x;
    let (yaw, pitch) = match billboard {
        Billboard::Fixed => (entity_y, entity_x),
        Billboard::Horizontal => (entity_y, camera_x),
        Billboard::Vertical => (camera_y, entity_x),
        Billboard::Center => (camera_y, camera_x),
    };
    Quaternion::rotation_yxz(negative * yaw, to_radians * pitch, 0.0)
}

/// What `DisplayRenderer.submit` leaves the pose at for `submitInner`: oriented, then moved by
/// the transformation at this progress. `base` is where the dispatcher puts the entity.
pub fn display_pose(
    base: &Pose,
    (billboard, transformation): (Billboard, &Transformation),
    turns: &DisplayTurns,
) -> Pose {
    let mut pose = *base;
    pose.rotate(&display_orientation(billboard, turns));
    pose.mul_matrix(&transformation_matrix_of(transformation));
    pose
}

/// `ItemDisplayRenderer.submitInner`: the item is turned to face the other way.
pub fn item_display_pose(pose: &Pose) -> Pose {
    let mut pose = *pose;
    pose.rotate(&Quaternion::rotation_y(std::f32::consts::PI));
    pose
}
