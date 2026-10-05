//! Where the projectiles' models are drawn from (`ArrowRenderer` and the
//! renderers like it): from the pose the entity's place leaves, turned after
//! the way it flies.

use crate::entity_textures::{TextureRect, uv};
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

/// `ThrownTridentRenderer.submit` up to the model: turned about the vertical to point where it
/// goes, then tipped to stand along its flight (the model stands upright, an arrow's lies down).
pub fn trident_pose(base: &Pose, y_rot: f32, x_rot: f32) -> Pose {
    let mut pose = *base;
    pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
        y_rot - 90.0,
    )));
    pose.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
        x_rot + 90.0,
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

/// The quad a dragon's fireball is drawn as (`DragonFireballRenderer.submit`): placed, with the
/// texture coordinate each corner has and the one normal they share.
#[derive(Debug, Clone, PartialEq)]
pub struct FireballQuad {
    pub corners: [([f32; 3], [f32; 2]); 4],
    /// `Pose.transformNormal(0, 1, 0)`.
    pub normal: [f32; 3],
}

/// `DragonFireballRenderer.submit`: twice the size, turned to face the camera, a quad about
/// its middle (a quarter lower than the entity's position, as the reference builds it).
pub fn dragon_fireball_quad(base: &Pose, orientation: &Quaternion) -> FireballQuad {
    let mut pose = *base;
    pose.scale(2.0, 2.0, 2.0);
    pose.rotate(orientation);
    let corner = |x: f32, y: i32, u: i32, v: i32| {
        (
            pose.pose.transform_position(x - 0.5, y as f32 - 0.25, 0.0),
            [u as f32, v as f32],
        )
    };
    FireballQuad {
        corners: [
            corner(0.0, 0, 0, 1),
            corner(1.0, 0, 1, 1),
            corner(1.0, 1, 1, 0),
            corner(0.0, 1, 0, 0),
        ],
        normal: pose.transform_normal([0.0, 1.0, 0.0]),
    }
}

/// The fireball's quad as triangles for the entity shader (`entityCutout`: lit by the world and
/// by its normal, whichever face shows, no overlay).
pub fn dragon_fireball_triangles(
    quad: &FireballQuad,
    texture: TextureRect,
    light: [f32; 2],
) -> Vec<Vertex> {
    [0, 1, 2, 0, 2, 3]
        .into_iter()
        .map(|index| {
            let (position, coordinate) = quad.corners[index];
            Vertex {
                position,
                uv: uv(texture, coordinate),
                shade: 0.0,
                tint: [quad.normal[0], quad.normal[1], quad.normal[2], 0.0],
                light,
            }
        })
        .collect()
}
