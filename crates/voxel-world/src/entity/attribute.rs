use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EntityInterpolationState {
    pub(crate) target: DVec3,
    pub(crate) target_y_rot: JavaF32,
    pub(crate) target_x_rot: JavaF32,
    pub(crate) previous: DVec3,
    pub(crate) previous_y_rot: JavaF32,
    pub(crate) previous_x_rot: JavaF32,
    pub(crate) steps: u8,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ResolvedEntityPosition {
    pub(crate) current: (f64, f64, f64),
    pub(crate) target: (f64, f64, f64),
    pub(crate) ticking: bool,
}

impl EntityInterpolationState {
    pub(crate) fn target_position(self) -> (f64, f64, f64) {
        (
            self.target.x.get(),
            self.target.y.get(),
            self.target.z.get(),
        )
    }

    pub(crate) fn target_rotation(self) -> (f32, f32) {
        (self.target_y_rot.get(), self.target_x_rot.get())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientEntity {
    pub(crate) id: i32,
    pub uuid: voxel_core::binary::Uuid,
    pub type_id: i32,
    pub(crate) tick_count: i32,
    pub(crate) first_tick: bool,
    /// `FallingBlockEntity.getStartPos`: the block the entity was added in.
    pub(crate) start_block: (i32, i32, i32),
    pub(crate) position: DVec3,
    pub(crate) old_position: DVec3,
    pub(crate) motion: DVec3,
    pub(crate) motion_lerp_target: DVec3,
    pub(crate) motion_lerp_steps: u8,
    pub(crate) x_rot: JavaF32,
    pub(crate) y_rot: JavaF32,
    pub(crate) y_head_rot: JavaF32,
    pub(crate) y_body_rot: JavaF32,
    pub(crate) old_x_rot: JavaF32,
    pub(crate) old_y_rot: JavaF32,
    pub(crate) old_y_head_rot: JavaF32,
    pub(crate) old_y_body_rot: JavaF32,
    pub(crate) delta_rotation: JavaF32,
    pub(crate) boat_input: BoatInput,
    /// What a boat the local player rides keeps between ticks (status, water level, fall).
    pub(crate) boat_ride: crate::boat_ride::RideState,
    /// The same of a saddled horse the local player rides.
    pub(crate) mount_ride: crate::mount_ride::MountState,
    /// A flying projectile's owner and whether it has left it.
    pub(crate) flight: crate::flight::FlightState,
    pub(crate) paddle_left: bool,
    pub(crate) paddle_right: bool,
    pub(crate) head_rotation_target: JavaF32,
    pub(crate) head_lerp_steps: u8,
    pub(crate) interpolation: Option<EntityInterpolationState>,
    pub(crate) on_ground: bool,
    pub(crate) tracking: Tracking,
    pub(crate) always_ticking: bool,
    pub(crate) ticking: bool,
    /// The limbs' walk animation and the swimming pose's ease (a player's).
    pub(crate) walk: crate::living::WalkAnimation,
    pub(crate) swim_amount: JavaF32,
    pub(crate) old_swim_amount: JavaF32,
    /// A mob's `BodyRotationControl` state: how long its head has been still,
    /// and where it was when it last moved far.
    pub(crate) head_stable_time: i32,
    pub(crate) last_stable_y_head_rot: JavaF32,
    /// What a dropped item holds (`ItemEntity.DATA_ITEM`), and where in its
    /// spin it started (`bobOffs`).
    pub(crate) item_stack: Option<ClientItemStackState>,
    pub(crate) bob_offset: JavaF32,
    /// What it holds and wears, by `EquipmentSlot`.
    pub(crate) equipment: [Option<ClientItemStackState>; EquipmentSlot::COUNT],
    pub data: i32,
    pub(crate) metadata: Option<Vec<u8>>,
    pub(crate) shared_flags: u8,
    pub(crate) living_flags: u8,
    pub(crate) mob_flags: u8,
    pub(crate) air_supply: i32,
    pub(crate) health_bits: Option<u32>,
    pub(crate) living: EntityState,
    pub(crate) arrow_count: i32,
    pub(crate) stinger_count: i32,
    pub(crate) effect_ambience: bool,
    pub(crate) effect_particle_type_ids: Vec<i32>,
    pub(crate) sleeping_pos: Option<voxel_core::binary::BlockPos>,
    pub(crate) custom_name_present: bool,
    pub(crate) custom_name_visible: bool,
    pub(crate) no_gravity: bool,
    pub(crate) silent: bool,
    pub(crate) pose: ClientEntityPose,
    pub(crate) baby: bool,
    pub(crate) alive: bool,
    pub(crate) spectator: bool,
    pub(crate) on_climbable: bool,
    pub(crate) flying: bool,
    pub(crate) happy_ghast_still_timeout: bool,
    pub(crate) happy_ghast_is_vehicle: bool,
    pub(crate) ticks_frozen: i32,
    pub(crate) vehicle_id: Option<i32>,
    pub(crate) passenger_ids: Vec<i32>,
    /// What the server changed of the attributes the type starts with.
    pub(crate) attributes: Vec<ClientAttributeState>,
    /// What kind of entity this is, with the state that kind keeps (a rabbit's hops, a warden's
    /// pulses…), made when something first reaches it.
    pub(crate) kind: crate::entity_kind::EntityKind,
    /// A primed tnt's fuse and block, made when its data or ticks first reach it.
    /// `ItemFrame.getRotation`: the eighths of a turn the item in it is turned.
    pub(crate) item_frame_rotation: i32,
    /// `Leashable.LeashData.delayedLeashHolderId`: the entity holding the lead, zero for none.
    pub(crate) leash_holder_id: i32,
    /// Whether the bolt ran out of flashes at its last tick (`LightningBolt.discard`).
    pub(crate) lightning_done: bool,
    /// `FireworkRocketEntity.isShotAtAngle`: fired from a crossbow, which turns how it is drawn.
    pub(crate) shot_at_angle: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientAttributeModifierState {
    pub(crate) id: String,
    pub(crate) amount_bits: u64,
    pub(crate) operation: u8,
}

impl ClientAttributeModifierState {
    pub fn new(id: String, amount: f64, operation: u8) -> Self {
        Self {
            id,
            amount_bits: amount.to_bits(),
            operation,
        }
    }

    pub fn from_wire(id: String, amount: f64, operation: u8) -> Self {
        Self::new(id, amount, operation)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn amount(&self) -> f64 {
        f64::from_bits(self.amount_bits)
    }

    pub fn operation(&self) -> u8 {
        self.operation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientAttributeState {
    pub(crate) attribute_id: i32,
    pub(crate) base_value_bits: u64,
    pub(crate) modifiers: Vec<ClientAttributeModifierState>,
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientMobEffectState {
    pub(crate) effect_id: i32,
    pub(crate) amplifier: i32,
    pub(crate) duration_ticks: i32,
    pub(crate) flags: u8,
}

impl ClientMobEffectState {
    pub fn effect_id(&self) -> i32 {
        self.effect_id
    }

    pub fn amplifier(&self) -> i32 {
        self.amplifier
    }

    pub fn duration_ticks(&self) -> i32 {
        self.duration_ticks
    }

    pub fn has_remaining_duration(&self) -> bool {
        self.duration_ticks == -1 || self.duration_ticks > 0
    }

    pub fn flags(&self) -> u8 {
        self.flags
    }

    pub(crate) fn tick_client(&mut self) {
        if self.has_remaining_duration() && self.duration_ticks != -1 {
            self.duration_ticks -= 1;
        }
    }
}

impl ClientAttributeState {
    pub fn new(
        attribute_id: i32,
        base_value: f64,
        modifiers: Vec<ClientAttributeModifierState>,
    ) -> Self {
        Self {
            attribute_id,
            base_value_bits: base_value.to_bits(),
            modifiers,
        }
    }

    pub fn from_wire(
        attribute_id: i32,
        base_value: f64,
        modifiers: Vec<ClientAttributeModifierState>,
    ) -> Self {
        Self::new(attribute_id, base_value, modifiers)
    }

    pub fn attribute_id(&self) -> i32 {
        self.attribute_id
    }

    pub fn base_value(&self) -> f64 {
        f64::from_bits(self.base_value_bits)
    }

    pub fn modifiers(&self) -> &[ClientAttributeModifierState] {
        &self.modifiers
    }

    /// Calculate the effective value using Java's AttributeInstance operation order.
    pub fn effective_value(&self, min_value: f64, max_value: f64) -> f64 {
        assert!(min_value <= max_value, "attribute range must be ordered");

        let mut base = self.base_value();
        for modifier in self.modifiers.iter().filter(|modifier| {
            modifier.operation() == 0 || (modifier.operation() != 1 && modifier.operation() != 2)
        }) {
            base += modifier.amount();
        }

        let mut result = base;
        for modifier in self
            .modifiers
            .iter()
            .filter(|modifier| modifier.operation() == 1)
        {
            result += base * modifier.amount();
        }
        for modifier in self
            .modifiers
            .iter()
            .filter(|modifier| modifier.operation() == 2)
        {
            result *= 1.0 + modifier.amount();
        }

        if result.is_nan() {
            min_value
        } else {
            result.clamp(min_value, max_value)
        }
    }
}

/// The entities a tick looks up in the world: what a guardian's beam aims at and
/// what a wither's side heads look at.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct EntityTargets {
    pub guardian: Option<crate::guardian::GuardianTarget>,
    pub wither: [Option<crate::guardian::GuardianTarget>; 2],
    pub dragon: crate::ender_dragon::DragonCrystals,
}

pub struct EntityInput {
    pub entity_id: i32,
    pub uuid: voxel_core::binary::Uuid,
    pub type_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub movement: (f64, f64, f64),
    pub rotation: (f32, f32, f32),
    pub data: i32,
}

pub struct EntityMotionInput {
    pub entity_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub struct EntityRotationInput {
    pub entity_id: i32,
    pub y_rot: Option<f32>,
    pub x_rot: Option<f32>,
    pub y_head_rot: Option<f32>,
    pub relative_y_rot: bool,
    pub relative_x_rot: bool,
}

pub struct EntityPositionInput {
    pub entity_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub relative_flags: i32,
}

pub struct EntityTeleportInput {
    pub entity_id: i32,
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
    pub entity_id: i32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub has_position: bool,
    pub y_rot: Option<f32>,
    pub x_rot: Option<f32>,
}

/// The reference entities whose client tick is a plain one-step ballistic move
/// guarded by an air check. Each variant reproduces its own Java step exactly;
/// the grouping is only over the shared guard, never over the arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VanillaProjectileKind {
    Snowball,
    Arrow,
    ExperienceOrb,
    PrimedTnt,
    FallingBlock,
    Firework,
}

impl VanillaProjectileKind {
    /// Position after one tick, in the reference's own operand order.
    pub(crate) fn next_position(
        self,
        position: (f64, f64, f64),
        motion: (f64, f64, f64),
        no_gravity: bool,
    ) -> (f64, f64, f64) {
        match self {
            Self::Snowball => {
                let air_drag = f64::from(0.99_f32);
                let gravity = if no_gravity { 0.0 } else { SNOWBALL_GRAVITY };
                (
                    position.0 + motion.0 * air_drag,
                    position.1 + (motion.1 - gravity) * air_drag,
                    position.2 + motion.2 * air_drag,
                )
            }
            Self::Arrow => (
                position.0 + motion.0,
                position.1 + motion.1,
                position.2 + motion.2,
            ),
            Self::ExperienceOrb => {
                gravity_step(position, motion, no_gravity, VANILLA_EXPERIENCE_ORB_GRAVITY)
            }
            Self::PrimedTnt => {
                gravity_step(position, motion, no_gravity, VANILLA_PRIMED_TNT_GRAVITY)
            }
            Self::FallingBlock => {
                gravity_step(position, motion, no_gravity, VANILLA_FALLING_BLOCK_GRAVITY)
            }
            Self::Firework => {
                let movement = (
                    motion.0 * VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
                    motion.1 + VANILLA_FIREWORK_ROCKET_VERTICAL_ACCELERATION,
                    motion.2 * VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
                );
                (
                    position.0 + movement.0,
                    position.1 + movement.1,
                    position.2 + movement.2,
                )
            }
        }
    }
}
