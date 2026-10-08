//! A boat (`AbstractBoatRenderer`): where its model is drawn, from its turn,
//! the rocking of a hit and the shaking of a bubble column.

use photon_core::math::mth_sin_f64;

use crate::entity::mob_model::MobModel;
use crate::entity::textures::{TextureRect, uv};
use crate::joml::{Pose, Quaternion};
use crate::scene::Vertex;

/// What a boat's model is posed and placed from (`BoatRenderState`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BoatRenderState {
    pub y_rot: f32,
    pub hurt_dir: i32,
    pub hurt_time: f32,
    pub damage_time: f32,
    pub bubble_angle: f32,
    pub is_under_water: bool,
    pub rowing_time_left: f32,
    pub rowing_time_right: f32,
}

/// `AbstractBoatRenderer.submit` up to the model: from the pose the entity's
/// place leaves, the boat lifted a little, turned to face, rocked by a hit and
/// shaken by a bubble column, then flipped over (the models are upside down)
/// and turned a quarter.
pub fn boat_pose(base: &Pose, state: &BoatRenderState) -> Pose {
    let mut pose = *base;
    let degrees = Quaternion::degrees_to_radians;
    pose.translate(0.0, 0.375, 0.0);
    pose.rotate(&Quaternion::rotation_y(degrees(180.0 - state.y_rot)));
    let hurt = state.hurt_time;
    if hurt > 0.0 {
        pose.rotate(&Quaternion::rotation_x(degrees(
            mth_sin_f64(f64::from(hurt)) * hurt * state.damage_time / 10.0 * state.hurt_dir as f32,
        )));
    }
    // `Mth.equal`: closer than a hundred thousandth.
    if !state.is_under_water && (state.bubble_angle - 0.0).abs() >= 1.0E-5 {
        pose.rotate(&Quaternion::angle_axis(
            state.bubble_angle * (std::f64::consts::PI / 180.0) as f32,
            1.0,
            0.0,
            1.0,
        ));
    }
    pose.scale(-1.0, -1.0, 1.0);
    pose.rotate(&Quaternion::rotation_y(degrees(90.0)));
    pose
}

/// The posed boat model's triangles from a pose, lit as the boat is, with no
/// overlay and each vertex carrying its normal for the entity shader.
pub fn boat_vertices(
    model: &MobModel,
    pose: &Pose,
    light: i32,
    texture: TextureRect,
) -> Vec<Vertex> {
    let light = crate::block::light::light_uv(light);
    let mut vertices = Vec::new();
    for corners in model.quads(pose) {
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(Vertex {
                position: corner.position,
                uv: uv(texture, corner.uv),
                shade: 0.0,
                tint: [corner.normal[0], corner.normal[1], corner.normal[2], 0.0],
                light,
            });
        }
    }
    vertices
}
