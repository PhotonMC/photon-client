//! Where the projectiles' models are drawn from (`ArrowRenderer` and the
//! renderers like it): from the pose the entity's place leaves, turned after
//! the way it flies.

use crate::joml::{Pose, Quaternion};
use crate::scene::Vertex;

/// `ArrowRenderer.submit` up to the model: turned about the vertical to point where it
/// goes (the model lies along its x), then tipped by the pitch.
pub fn arrow_pose(base: &Pose, y_rot: f32, x_rot: f32) -> Pose {
    let mut pose = *base;
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        y_rot - 90.0,
    )));
    pose.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
        x_rot,
    )));
    pose
}

/// The triangles a pipeline that culls back faces (`entityCutoutCull`) draws: those whose
/// face looks toward the camera. `vertices` are the six triangle corners of each quad that
/// the models make, each carrying its normal in its tint; a model of flat planes (an arrow's)
/// has both faces of each at the same place, and only the one toward the eye is drawn.
pub fn cull_back_faces(vertices: Vec<Vertex>, camera: [f32; 3]) -> Vec<Vertex> {
    vertices
        .chunks_exact(6)
        .filter(|quad| {
            let [nx, ny, nz, _] = quad[0].tint;
            let at = quad[0].position;
            let to_eye = [camera[0] - at[0], camera[1] - at[1], camera[2] - at[2]];
            nx * to_eye[0] + ny * to_eye[1] + nz * to_eye[2] > 0.0
        })
        .flatten()
        .copied()
        .collect()
}

/// `ThrownItemRenderer.submit` up to the item: scaled, then turned to face the camera
/// (`camera.orientation`, so the item's flat side looks at the eye).
pub fn thrown_item_pose(base: &Pose, scale: f32, orientation: &Quaternion) -> Pose {
    let mut pose = *base;
    pose.scale(scale, scale, scale);
    pose.rotate(orientation);
    pose
}

/// `FireworkEntityRenderer.submit` up to the item: turned to face the camera, and a rocket
/// shot at an angle (from a crossbow) laid on its side the way it flies.
pub fn firework_pose(base: &Pose, orientation: &Quaternion, shot_at_angle: bool) -> Pose {
    let mut pose = *base;
    pose.rotate(orientation);
    if shot_at_angle {
        let degrees = Quaternion::degrees_to_radians;
        pose.rotate(&Quaternion::rotation_z(degrees(180.0)));
        pose.rotate(&Quaternion::rotation_y(degrees(180.0)));
        pose.rotate(&Quaternion::rotation_x(degrees(90.0)));
    }
    pose
}
