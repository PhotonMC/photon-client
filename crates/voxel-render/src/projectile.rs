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
