//! The cubes of the end portal and the end gateway (`AbstractEndPortalRenderer`): the faces that
//! show, as the four corners of each, from which the surface is drawn.

use crate::block::geometry::face_vertices;
use crate::joml::Pose;
use photon_core::direction::Direction;

/// `AbstractEndPortalRenderer.submitCube`: the corners of each of `faces`, of the unit cube, moved
/// by `pose`. The faces are a set of the enum, so they are written in the enum's order whatever
/// order they are given in.
pub fn cube_corners(pose: &Pose, faces: &[Direction]) -> Vec<[f32; 3]> {
    Direction::stream()
        .filter(|face| faces.contains(face))
        .flat_map(|face| face_vertices([0.0; 3], [16.0; 3], face))
        .map(|[x, y, z]| pose.pose.transform_position(x, y, z))
        .collect()
}

/// `TheEndPortalRenderer.TRANSFORMATION`: the portal is a flat slab, three eighths up and three
/// eighths tall, so its top is lower than the block's.
pub fn portal_pose(base: &Pose) -> Pose {
    let mut pose = *base;
    pose.translate(0.0, 0.375, 0.0);
    pose.scale(1.0, 0.375, 1.0);
    pose
}
