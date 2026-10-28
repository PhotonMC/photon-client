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
    /// `Entity.isOnFire`, or a type whose renderer answers 15 (`getBlockLightLevel`): it is lit
    /// at full block light.
    pub full_block_light: bool,
    /// An invisible entity the local player sees as a ghost because of a team.
    pub friendly_ghost: bool,
    /// A mob a spawner shows: where it is put and how it turns (`SpawnerRenderer`).
    pub spawner_display: Option<SpawnerDisplayPose>,
    /// `LivingEntityRenderState.isUpsideDown`: the height of the bounding box of a living entity
    /// named Dinnerbone or Grumm, whose pitch and head turn are already mirrored here.
    pub upside_down: Option<f32>,
    /// `Pose.SLEEPING`: the step in x and z of the direction of the bed it lies in (none without
    /// one) and its eye height standing.
    pub sleeping: Option<(Option<(i32, i32)>, f32)>,
    /// `Entity.getLightProbePosition`: where the light is sampled for it.
    pub light_probe: (f64, f64, f64),
}

impl EntitySnapshot {
    /// `extractRenderState` for an upside-down entity: it looks the other way up and aside.
    pub fn turn_upside_down(&mut self, height: f32) {
        self.upside_down = Some(height);
        self.pitch = -self.pitch;
        self.head_yaw =
            self.body_yaw - photon_core::math::wrap_degrees_f32(self.head_yaw - self.body_yaw);
    }
}

/// How a spawner shows its mob: the block's corner, the turn in degrees (the lerped spin times
/// ten) and the scale that fits the mob in the cage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpawnerDisplayPose {
    pub block: (i32, i32, i32),
    pub spin: f32,
    pub scale: f32,
}

impl SpawnerDisplayPose {
    /// `TrialSpawnerRenderer.extractSpawnerData`: the spin between two ticks, ten times as far
    /// round, and the scale that fits a mob of a box in the cage.
    pub fn of(
        block: (i32, i32, i32),
        (previous, current): (f64, f64),
        partial_tick: f32,
        (width, height): (f32, f32),
    ) -> Self {
        let spin = (previous + f64::from(partial_tick) * (current - previous)) as f32 * 10.0;
        let longest = width.max(height);
        let mut scale = 0.53125_f32;
        if longest > 1.0 {
            scale /= longest;
        }
        Self { block, spin, scale }
    }
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
    pub type_id: EntityTypeId,
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
