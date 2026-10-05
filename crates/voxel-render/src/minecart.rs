//! A minecart (`AbstractMinecartRenderer`, the old behaviour's way): where its
//! model and the block it carries are drawn, from the rail under it, the turn it
//! was last sent, and the rocking of a hit.

use voxel_core::math::mth_sin_f64;

use crate::joml::{Pose, Quaternion};

/// What a minecart is placed from (`MinecartRenderState`), when its behaviour is the old one.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MinecartRenderState {
    /// Where the entity is between two ticks (`EntityRenderState.x, y, z`).
    pub position: (f64, f64, f64),
    pub x_rot: f32,
    pub y_rot: f32,
    pub offset_seed: i64,
    pub hurt_dir: i32,
    pub hurt_time: f32,
    pub damage_time: f32,
    pub display_offset: i32,
    /// Where the rail puts it, and the rail's two ends a little ahead and behind it,
    /// when it is on one.
    pub rail: Option<RailPlace>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RailPlace {
    pub pos_on_rail: (f64, f64, f64),
    pub front: (f64, f64, f64),
    pub back: (f64, f64, f64),
}

/// `MinecartRenderState.offsetSeed`: the minecart's own tiny shift, from its id.
pub fn offset_seed(entity_id: i32) -> i64 {
    let seed = i64::from(entity_id).wrapping_mul(493_286_711);
    seed.wrapping_mul(seed)
        .wrapping_mul(4_392_167_121)
        .wrapping_add(seed.wrapping_mul(98_761))
}

/// The poses `AbstractMinecartRenderer.submit` draws with, from the pose the entity's
/// place leaves: the block it carries (when it carries one) and the model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinecartPoses {
    pub block: Pose,
    pub model: Pose,
}

pub fn minecart_poses(base: &Pose, state: &MinecartRenderState) -> MinecartPoses {
    let mut pose = *base;
    let degrees = Quaternion::degrees_to_radians;
    // The shift that keeps two minecarts' faces from fighting: a tenth of a pixel or so.
    let shift = |bits: u32| ((((state.offset_seed >> bits) & 7) as f32 + 0.5) / 8.0 - 0.5) * 0.004;
    pose.translate(shift(16), shift(20), shift(24));
    // `oldRender`.
    let (entity_x, entity_y, entity_z) = state.position;
    let mut x_rot = state.x_rot;
    let mut rotation = state.y_rot;
    if let Some(rail) = state.rail {
        let (front, back) = (rail.front, rail.back);
        pose.translate(
            (rail.pos_on_rail.0 - entity_x) as f32,
            ((front.1 + back.1) / 2.0 - entity_y) as f32,
            (rail.pos_on_rail.2 - entity_z) as f32,
        );
        let direction = (back.0 + -front.0, back.1 + -front.1, back.2 + -front.2);
        let length =
            (direction.0 * direction.0 + direction.1 * direction.1 + direction.2 * direction.2)
                .sqrt();
        if length != 0.0 {
            // `Vec3.normalize`.
            let (dx, dy, dz) = (
                direction.0 / length,
                direction.1 / length,
                direction.2 / length,
            );
            rotation = (dz.atan2(dx) * 180.0 / std::f64::consts::PI) as f32;
            x_rot = (dy.atan() * 73.0) as f32;
        }
    }
    pose.translate(0.0, 0.375, 0.0);
    pose.rotate(&Quaternion::rotation_y(degrees(180.0 - rotation)));
    pose.rotate(&Quaternion::rotation_z(degrees(-x_rot)));
    let hurt = state.hurt_time;
    if hurt > 0.0 {
        pose.rotate(&Quaternion::rotation_x(degrees(
            mth_sin_f64(f64::from(hurt)) * hurt * state.damage_time / 10.0 * state.hurt_dir as f32,
        )));
    }
    // The carried block: three quarters size, lowered, a quarter turn.
    let mut block = pose;
    block.scale(0.75, 0.75, 0.75);
    block.translate(-0.5, (state.display_offset - 8) as f32 / 16.0, 0.5);
    block.rotate(&Quaternion::rotation_y(degrees(90.0)));
    pose.scale(-1.0, -1.0, 1.0);
    MinecartPoses { block, model: pose }
}
