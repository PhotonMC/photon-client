use super::*;

/// An entity as it is drawn: where it is, and which way it and its head look
/// (degrees), between two ticks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EntitySnapshot {
    pub position: (f64, f64, f64),
    pub yaw: f32,
    pub pitch: f32,
    pub head_yaw: f32,
    /// Which way the body faces.
    pub body_yaw: f32,
    /// The limbs' walk animation, as the model reads it.
    pub walk_position: f32,
    pub walk_speed: f32,
    pub swim_amount: f32,
    pub pose: ClientEntityPose,
    /// `LivingEntityRenderState.hasRedOverlay`: hurt or dying.
    pub red_overlay: bool,
    /// `LivingEntityRenderState.deathTime`: the turns of falling over, with
    /// the part of a tick, or zero while it lives.
    pub death_time: f32,
    /// `LivingEntityRenderState.attackTime`: how far through its swing.
    pub attack_time: f32,
    /// Whether the arm that swings is the left (`attackArm`).
    pub attack_arm_left: bool,
    /// `EntityRenderState.ageInTicks`.
    pub age_in_ticks: f32,
    /// `LivingEntityRenderer.isShaking`.
    pub shaking: bool,
}

/// The entities a tick looks up in the world: what a guardian's beam aims at and
/// what a wither's side heads look at.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct EntityTargets {
    pub guardian: Option<crate::mobs::guardian::GuardianTarget>,
    pub wither: [Option<crate::mobs::guardian::GuardianTarget>; 2],
    pub dragon: crate::mobs::ender_dragon::DragonCrystals,
}

pub struct EntityInput {
    pub entity_id: EntityId,
    pub uuid: photon_core::binary::Uuid,
    pub type_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub movement: (f64, f64, f64),
    pub rotation: (f32, f32, f32),
    pub data: i32,
}

pub struct EntityMotionInput {
    pub entity_id: EntityId,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub struct EntityRotationInput {
    pub entity_id: EntityId,
    pub y_rot: Option<f32>,
    pub x_rot: Option<f32>,
    pub y_head_rot: Option<f32>,
    pub relative_y_rot: bool,
    pub relative_x_rot: bool,
}

pub struct EntityPositionInput {
    pub entity_id: EntityId,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub relative_flags: i32,
}

pub struct EntityTeleportInput {
    pub entity_id: EntityId,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub y_rot: f32,
    pub x_rot: f32,
    pub relative_flags: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityPositionUpdateMode {
    PositionSync,
    Teleport,
}

pub struct EntityMoveInput {
    pub entity_id: EntityId,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub has_position: bool,
    pub y_rot: Option<f32>,
    pub x_rot: Option<f32>,
}
