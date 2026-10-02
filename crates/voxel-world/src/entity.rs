//! `ClientWorld` entity layer split out of the former single-file module.

use super::*;

use voxel_core::aabb::Aabb;
use voxel_core::math::{
    MTH_DEG_TO_RAD, Vec3, clamp_f32, mth_atan2, mth_cos_f64, mth_lerp_f32, mth_lerp_f64,
    mth_rot_lerp_f32, mth_rot_lerp_f64, mth_sin_f64, projectile_lerp_rotation_f32,
    wrap_degrees_f32,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ClientEntityPose {
    #[default]
    Standing,
    FallFlying,
    Sleeping,
    Swimming,
    SpinAttack,
    Crouching,
    LongJumping,
    Dying,
    Croaking,
    UsingTongue,
    Sitting,
    Roaring,
    Sniffing,
    Emerging,
    Digging,
    Sliding,
    Shooting,
    Inhaling,
}

impl ClientEntityPose {
    /// The reference's `Pose` constant name.
    pub const fn java_name(self) -> &'static str {
        match self {
            Self::Standing => "STANDING",
            Self::FallFlying => "FALL_FLYING",
            Self::Sleeping => "SLEEPING",
            Self::Swimming => "SWIMMING",
            Self::SpinAttack => "SPIN_ATTACK",
            Self::Crouching => "CROUCHING",
            Self::LongJumping => "LONG_JUMPING",
            Self::Dying => "DYING",
            Self::Croaking => "CROAKING",
            Self::UsingTongue => "USING_TONGUE",
            Self::Sitting => "SITTING",
            Self::Roaring => "ROARING",
            Self::Sniffing => "SNIFFING",
            Self::Emerging => "EMERGING",
            Self::Digging => "DIGGING",
            Self::Sliding => "SLIDING",
            Self::Shooting => "SHOOTING",
            Self::Inhaling => "INHALING",
        }
    }

    pub const fn id(self) -> u8 {
        match self {
            Self::Standing => 0,
            Self::FallFlying => 1,
            Self::Sleeping => 2,
            Self::Swimming => 3,
            Self::SpinAttack => 4,
            Self::Crouching => 5,
            Self::LongJumping => 6,
            Self::Dying => 7,
            Self::Croaking => 8,
            Self::UsingTongue => 9,
            Self::Sitting => 10,
            Self::Roaring => 11,
            Self::Sniffing => 12,
            Self::Emerging => 13,
            Self::Digging => 14,
            Self::Sliding => 15,
            Self::Shooting => 16,
            Self::Inhaling => 17,
        }
    }

    pub(crate) fn from_wire_id(id: i32) -> Self {
        match id {
            1 => Self::FallFlying,
            2 => Self::Sleeping,
            3 => Self::Swimming,
            4 => Self::SpinAttack,
            5 => Self::Crouching,
            6 => Self::LongJumping,
            7 => Self::Dying,
            8 => Self::Croaking,
            9 => Self::UsingTongue,
            10 => Self::Sitting,
            11 => Self::Roaring,
            12 => Self::Sniffing,
            13 => Self::Emerging,
            14 => Self::Digging,
            15 => Self::Sliding,
            16 => Self::Shooting,
            17 => Self::Inhaling,
            _ => Self::Standing,
        }
    }

    /// Resolve a wire pose ID with Java's out-of-range standing fallback.
    pub fn from_id(id: i32) -> Self {
        Self::from_wire_id(id)
    }

    /// Return the default Avatar dimensions for this pose at scale one.
    pub fn player_dimensions(self) -> ClientEntityDimensions {
        match self {
            Self::Standing => ClientEntityDimensions::scalable(0.6, 1.8).with_eye_height(1.62),
            Self::Sleeping => ClientEntityDimensions::fixed(0.2, 0.2).with_eye_height(0.2),
            Self::FallFlying | Self::Swimming | Self::SpinAttack => {
                ClientEntityDimensions::scalable(0.6, 0.6).with_eye_height(0.4)
            }
            Self::Crouching => ClientEntityDimensions::scalable(0.6, 1.5).with_eye_height(1.27),
            Self::Dying => ClientEntityDimensions::fixed(0.2, 0.2).with_eye_height(1.62),
            _ => Self::Standing.player_dimensions(),
        }
    }
}

/// Entity dimensions used to derive Java's feet-origin bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientEntityDimensions {
    pub(crate) width_bits: u32,
    pub(crate) height_bits: u32,
    pub(crate) eye_height_bits: u32,
    pub(crate) fixed: bool,
}

impl ClientEntityDimensions {
    /// Create scalable dimensions with Java's default eye-height multiplier.
    pub fn scalable(width: f32, height: f32) -> Self {
        Self::from_values(width, height, height * 0.85_f32, false)
    }

    /// Create dimensions that ignore later scale changes.
    pub fn fixed(width: f32, height: f32) -> Self {
        Self::from_values(width, height, height * 0.85_f32, true)
    }

    pub(crate) fn from_values(width: f32, height: f32, eye_height: f32, fixed: bool) -> Self {
        Self {
            width_bits: width.to_bits(),
            height_bits: height.to_bits(),
            eye_height_bits: eye_height.to_bits(),
            fixed,
        }
    }

    pub fn width(self) -> f32 {
        f32::from_bits(self.width_bits)
    }

    pub fn height(self) -> f32 {
        f32::from_bits(self.height_bits)
    }

    pub fn eye_height(self) -> f32 {
        f32::from_bits(self.eye_height_bits)
    }

    pub fn is_fixed(self) -> bool {
        self.fixed
    }

    /// Apply Java's width/height scale path, including the fixed early return.
    pub fn scale(self, width_scale: f32, height_scale: f32) -> Self {
        if !self.fixed && (width_scale != 1.0_f32 || height_scale != 1.0_f32) {
            Self::from_values(
                self.width() * width_scale,
                self.height() * height_scale,
                self.eye_height() * height_scale,
                false,
            )
        } else {
            self
        }
    }

    pub fn with_eye_height(self, eye_height: f32) -> Self {
        Self::from_values(self.width(), self.height(), eye_height, self.fixed)
    }

    /// Make the feet-origin AABB used by `EntityDimensions.makeBoundingBox`.
    pub fn make_bounding_box(self, position: Vec3) -> Aabb {
        let half_width = f64::from(self.width() / 2.0_f32);
        let height = f64::from(self.height());
        Aabb::new(
            position.x - half_width,
            position.y,
            position.z - half_width,
            position.x + half_width,
            position.y + height,
            position.z + half_width,
        )
    }
}

/// Resolve Java `LivingEntity.getDimensions` from default dimensions and scales.
pub fn living_entity_dimensions(
    pose: ClientEntityPose,
    default_dimensions: ClientEntityDimensions,
    age_scale: f32,
    entity_scale: f32,
) -> ClientEntityDimensions {
    if pose == ClientEntityPose::Sleeping {
        ClientEntityDimensions::fixed(0.2, 0.2).with_eye_height(0.2)
    } else {
        default_dimensions
            .scale(age_scale, age_scale)
            .scale(entity_scale, entity_scale)
    }
}

/// State produced by Java Entity.refreshDimensions before optional server
/// position correction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientEntityDimensionsRefresh {
    pub dimensions: ClientEntityDimensions,
    pub bounding_box: Aabb,
    pub should_fudge_position: bool,
}

/// Refresh dimensions at the current position and evaluate Java's resize gate.
pub fn refresh_entity_dimensions(
    old_dimensions: ClientEntityDimensions,
    new_dimensions: ClientEntityDimensions,
    position: Vec3,
    level_is_client_side: bool,
    first_tick: bool,
    no_physics: bool,
    is_player: bool,
) -> ClientEntityDimensionsRefresh {
    let is_small = new_dimensions.width() <= 4.0 && new_dimensions.height() <= 4.0;
    let grew = new_dimensions.width() > old_dimensions.width()
        || new_dimensions.height() > old_dimensions.height();
    ClientEntityDimensionsRefresh {
        dimensions: new_dimensions,
        bounding_box: new_dimensions.make_bounding_box(position),
        should_fudge_position: !level_is_client_side
            && !first_tick
            && !no_physics
            && is_small
            && grew
            && !is_player,
    }
}

/// The four movement flags copied by Java's `AbstractBoat.setInput`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BoatInput {
    pub left: bool,
    pub right: bool,
    pub forward: bool,
    pub backward: bool,
}

impl BoatInput {
    pub const EMPTY: Self = Self::new(false, false, false, false);

    pub const fn new(left: bool, right: bool, forward: bool, backward: bool) -> Self {
        Self {
            left,
            right,
            forward,
            backward,
        }
    }
}

pub(crate) fn shared_flag_is_set(flags: u8, flag: u8) -> bool {
    flags & (1_u8 << flag) != 0
}

pub(crate) fn advance_head_rotation(current_bits: &mut u32, target_bits: u32, steps: &mut u8) {
    if *steps == 0 {
        return;
    }
    let step_count = f64::from(*steps);
    let current = f64::from(f32::from_bits(*current_bits));
    let target = f64::from(f32::from_bits(target_bits));
    *current_bits = (mth_rot_lerp_f64(1.0 / step_count, current, target) as f32).to_bits();
    *steps -= 1;
}

pub(crate) const VANILLA_BOAT_PASSENGER_MAX_ROTATION: f32 = 105.0_f32;

pub(crate) fn vanilla_boat_passenger_rotation_clamped_values(
    passenger_y_rot: f32,
    passenger_old_y_rot: f32,
    vehicle_y_rot: f32,
) -> (f32, f32) {
    let delta = wrap_degrees_f32(passenger_y_rot - vehicle_y_rot);
    let target_delta = clamp_f32(
        delta,
        -VANILLA_BOAT_PASSENGER_MAX_ROTATION,
        VANILLA_BOAT_PASSENGER_MAX_ROTATION,
    );
    (
        passenger_old_y_rot + target_delta - delta,
        passenger_y_rot + target_delta - delta,
    )
}

pub(crate) fn vanilla_passenger_can_turn_in_boat(passenger_type_id: i32) -> bool {
    passenger_type_id == VANILLA_BREEZE_ENTITY_TYPE_ID
}

pub(crate) fn vanilla_passenger_is_animal(passenger_type_id: i32) -> bool {
    VANILLA_ANIMAL_ENTITY_TYPE_IDS.contains(&passenger_type_id)
}

pub(crate) fn vanilla_passenger_is_living(passenger_type_id: i32) -> bool {
    VANILLA_LIVING_ENTITY_TYPE_IDS.contains(&passenger_type_id)
}

pub(crate) fn vanilla_passenger_vehicle_attachment_y(passenger_type_id: i32, is_baby: bool) -> f64 {
    match passenger_type_id {
        VANILLA_PLAYER_ENTITY_TYPE_ID => VANILLA_PLAYER_VEHICLE_ATTACHMENT_Y,
        VANILLA_SPIDER_ENTITY_TYPE_ID => VANILLA_SPIDER_VEHICLE_ATTACHMENT_Y,
        VANILLA_ZOMBIE_ENTITY_TYPE_ID if is_baby => 0.1875,
        _ => 0.0,
    }
}

pub(crate) fn vanilla_boat_max_passengers(vehicle_type_id: i32) -> Option<usize> {
    VANILLA_BOAT_PASSENGER_GEOMETRY
        .iter()
        .any(|&(type_id, _, _)| type_id == vehicle_type_id)
        .then(|| {
            if VANILLA_CHEST_BOAT_ENTITY_TYPE_IDS.contains(&vehicle_type_id) {
                1
            } else {
                2
            }
        })
}

pub(crate) fn is_vanilla_boat_entity_type(entity_type_id: i32) -> bool {
    VANILLA_BOAT_PASSENGER_GEOMETRY
        .iter()
        .any(|&(type_id, _, _)| type_id == entity_type_id)
}

pub(crate) fn is_vanilla_shulker_entity_type(entity_type_id: i32) -> bool {
    entity_type_id == VANILLA_SHULKER_ENTITY_TYPE_ID
}

pub(crate) fn is_vanilla_minecart_entity_type(entity_type_id: i32) -> bool {
    VANILLA_MINECART_ENTITY_TYPE_IDS.contains(&entity_type_id)
}

pub(crate) fn is_vanilla_happy_ghast_entity_type(entity_type_id: i32) -> bool {
    entity_type_id == VANILLA_HAPPY_GHAST_ENTITY_TYPE_ID
}

pub(crate) fn is_vanilla_player_entity_type(entity_type_id: i32) -> bool {
    entity_type_id == VANILLA_PLAYER_ENTITY_TYPE_ID
}

pub(crate) fn is_vanilla_pushable_entity_type(entity_type_id: i32) -> bool {
    is_vanilla_boat_entity_type(entity_type_id)
        || is_vanilla_shulker_entity_type(entity_type_id)
        || is_vanilla_minecart_entity_type(entity_type_id)
        || is_vanilla_player_entity_type(entity_type_id)
}

pub(crate) fn source_accepts_pushable_entities(entity_type_id: i32) -> bool {
    is_vanilla_boat_entity_type(entity_type_id) || is_vanilla_minecart_entity_type(entity_type_id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EntityInterpolationState {
    pub(crate) target_x_bits: u64,
    pub(crate) target_y_bits: u64,
    pub(crate) target_z_bits: u64,
    pub(crate) target_y_rot_bits: u32,
    pub(crate) target_x_rot_bits: u32,
    pub(crate) previous_x_bits: u64,
    pub(crate) previous_y_bits: u64,
    pub(crate) previous_z_bits: u64,
    pub(crate) previous_y_rot_bits: u32,
    pub(crate) previous_x_rot_bits: u32,
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
            f64::from_bits(self.target_x_bits),
            f64::from_bits(self.target_y_bits),
            f64::from_bits(self.target_z_bits),
        )
    }

    pub(crate) fn target_rotation(self) -> (f32, f32) {
        (
            f32::from_bits(self.target_y_rot_bits),
            f32::from_bits(self.target_x_rot_bits),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientEntity {
    pub(crate) id: i32,
    pub uuid: voxel_core::binary::Uuid,
    pub type_id: i32,
    pub(crate) tick_count: i32,
    pub(crate) first_tick: bool,
    pub(crate) x_bits: u64,
    pub(crate) y_bits: u64,
    pub(crate) z_bits: u64,
    pub(crate) old_x_bits: u64,
    pub(crate) old_y_bits: u64,
    pub(crate) old_z_bits: u64,
    pub(crate) motion_x_bits: u64,
    pub(crate) motion_y_bits: u64,
    pub(crate) motion_z_bits: u64,
    pub(crate) motion_lerp_target_x_bits: u64,
    pub(crate) motion_lerp_target_y_bits: u64,
    pub(crate) motion_lerp_target_z_bits: u64,
    pub(crate) motion_lerp_steps: u8,
    pub(crate) x_rot_bits: u32,
    pub(crate) y_rot_bits: u32,
    pub(crate) y_head_rot_bits: u32,
    pub(crate) y_body_rot_bits: u32,
    pub(crate) old_x_rot_bits: u32,
    pub(crate) old_y_rot_bits: u32,
    pub(crate) old_y_head_rot_bits: u32,
    pub(crate) old_y_body_rot_bits: u32,
    pub(crate) delta_rotation_bits: u32,
    pub(crate) boat_input: BoatInput,
    pub(crate) paddle_left: bool,
    pub(crate) paddle_right: bool,
    pub(crate) head_rotation_target_bits: u32,
    pub(crate) head_lerp_steps: u8,
    pub(crate) interpolation: Option<EntityInterpolationState>,
    pub(crate) on_ground: bool,
    pub(crate) always_ticking: bool,
    pub(crate) ticking: bool,
    /// The limbs' walk animation and the swimming pose's ease (a player's).
    pub(crate) walk: crate::living::WalkAnimation,
    pub(crate) swim_amount_bits: u32,
    pub(crate) old_swim_amount_bits: u32,
    pub data: i32,
    pub(crate) metadata: Option<Vec<u8>>,
    pub(crate) shared_flags: u8,
    pub(crate) living_flags: u8,
    pub(crate) mob_flags: u8,
    pub(crate) air_supply: i32,
    pub(crate) health_bits: Option<u32>,
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

impl ClientEntity {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub(crate) fn collision_candidate(
        &self,
        source_entity_id: Option<i32>,
        source_type_id: Option<i32>,
        source_position: Option<Vec3>,
        same_vehicle: bool,
    ) -> Option<ClientEntityCollisionCandidate> {
        let (width, height) = if is_vanilla_boat_entity_type(self.type_id) {
            (
                (VANILLA_BOAT_HALF_WIDTH * 2.0) as f32,
                VANILLA_BOAT_HEIGHT as f32,
            )
        } else if is_vanilla_shulker_entity_type(self.type_id) {
            (1.0, 1.0)
        } else if is_vanilla_minecart_entity_type(self.type_id) {
            (0.98, 0.7)
        } else if is_vanilla_player_entity_type(self.type_id) {
            (0.6, 1.8)
        } else if is_vanilla_happy_ghast_entity_type(self.type_id) {
            (4.0, 4.0)
        } else {
            return None;
        };
        let (x, y, z) = self.position();
        let dimensions = if is_vanilla_happy_ghast_entity_type(self.type_id) && self.baby {
            ClientEntityDimensions::scalable(width, height).scale(0.2375, 0.2375)
        } else {
            ClientEntityDimensions::scalable(width, height)
        };
        let bounding_box = dimensions.make_bounding_box(Vec3::new(x, y, z));
        let can_be_collided_with = if is_vanilla_happy_ghast_entity_type(self.type_id) {
            !self.baby
                && self.alive
                && ((source_type_id == Some(VANILLA_PLAYER_ENTITY_TYPE_ID)
                    && source_position.is_some_and(|position| position.y >= bounding_box.max_y))
                    || (source_type_id == Some(VANILLA_HAPPY_GHAST_ENTITY_TYPE_ID)
                        && self.happy_ghast_is_vehicle)
                    || self.happy_ghast_still_timeout)
        } else {
            is_vanilla_boat_entity_type(self.type_id)
                || is_vanilla_shulker_entity_type(self.type_id)
        };
        let is_pushable = if is_vanilla_player_entity_type(self.type_id) {
            self.alive
                && !self.spectator
                && (self.flying || !self.on_climbable)
                && self
                    .health_bits
                    .is_none_or(|health| f32::from_bits(health) > 0.0)
        } else {
            is_vanilla_pushable_entity_type(self.type_id)
                || (is_vanilla_happy_ghast_entity_type(self.type_id) && self.alive)
        };
        Some(ClientEntityCollisionCandidate {
            bounding_box,
            is_source: source_entity_id == Some(self.id),
            spectator: self.spectator,
            can_be_collided_with,
            is_pushable,
            same_vehicle,
        })
    }

    pub fn vehicle_id(&self) -> Option<i32> {
        self.vehicle_id
    }

    pub fn passenger_ids(&self) -> &[i32] {
        &self.passenger_ids
    }

    pub(crate) fn turn(&mut self, mouse_x: f64, mouse_y: f64) {
        let x_delta = mouse_y as f32 * 0.15_f32;
        let y_delta = mouse_x as f32 * 0.15_f32;
        self.x_rot_bits = (f32::from_bits(self.x_rot_bits) + x_delta).to_bits();
        self.y_rot_bits = (f32::from_bits(self.y_rot_bits) + y_delta).to_bits();
        self.x_rot_bits = clamp_f32(f32::from_bits(self.x_rot_bits), -90.0, 90.0).to_bits();
        self.old_x_rot_bits = (f32::from_bits(self.old_x_rot_bits) + x_delta).to_bits();
        self.old_y_rot_bits = (f32::from_bits(self.old_y_rot_bits) + y_delta).to_bits();
        self.old_x_rot_bits = clamp_f32(f32::from_bits(self.old_x_rot_bits), -90.0, 90.0).to_bits();
    }

    pub(crate) fn capture_old_transform(&mut self) {
        self.old_x_bits = self.x_bits;
        self.old_y_bits = self.y_bits;
        self.old_z_bits = self.z_bits;
        self.old_x_rot_bits = self.x_rot_bits;
        self.old_y_rot_bits = self.y_rot_bits;
        self.old_y_head_rot_bits = self.y_head_rot_bits;
        self.old_y_body_rot_bits = self.y_body_rot_bits;
    }

    pub(crate) fn tick_head_rotation(&mut self) {
        advance_head_rotation(
            &mut self.y_head_rot_bits,
            self.head_rotation_target_bits,
            &mut self.head_lerp_steps,
        );
    }

    /// What a player's drawn pose follows from its tick (`RemotePlayer.tick`
    /// after `LivingEntity.tick`): the swimming pose easing in or out, the
    /// body turning after where the player walks, and the limbs' swing.
    pub(crate) fn tick_player_pose(&mut self) {
        self.old_swim_amount_bits = self.swim_amount_bits;
        self.swim_amount_bits = crate::living::ease_swim_amount(
            f32::from_bits(self.swim_amount_bits),
            self.pose == ClientEntityPose::Swimming,
        )
        .to_bits();
        let moved = (
            f64::from_bits(self.x_bits) - f64::from_bits(self.old_x_bits),
            f64::from_bits(self.z_bits) - f64::from_bits(self.old_z_bits),
        );
        let mut body = f32::from_bits(self.y_body_rot_bits);
        let mut body_old = f32::from_bits(self.old_y_body_rot_bits);
        let mut head_old = f32::from_bits(self.old_y_head_rot_bits);
        crate::living::turn_body(
            (&mut body, &mut body_old),
            (
                f32::from_bits(self.y_rot_bits),
                f32::from_bits(self.y_head_rot_bits),
                &mut head_old,
            ),
            moved,
        );
        self.y_body_rot_bits = body.to_bits();
        self.old_y_body_rot_bits = body_old.to_bits();
        self.old_y_head_rot_bits = head_old.to_bits();
        // `calculateEntityAnimation`: a rider or a dead player stands still.
        if self.vehicle_id.is_none() && self.alive {
            self.walk.follow(moved.0.hypot(moved.1));
        } else {
            self.walk.stop();
        }
    }

    pub(crate) fn interpolation_position(&self) -> (f64, f64, f64) {
        self.interpolation
            .filter(|interpolation| interpolation.steps > 0)
            .map_or_else(
                || self.position(),
                EntityInterpolationState::target_position,
            )
    }

    pub(crate) fn interpolation_rotation(&self) -> (f32, f32) {
        self.interpolation
            .filter(|interpolation| interpolation.steps > 0)
            .map_or_else(
                || self.body_rotation(),
                EntityInterpolationState::target_rotation,
            )
    }

    pub(crate) fn interpolate_to(&mut self, position: (f64, f64, f64), y_rot: f32, x_rot: f32) {
        let target_matches = self.interpolation.is_some_and(|interpolation| {
            interpolation.steps > 0
                && interpolation.target_x_bits == position.0.to_bits()
                && interpolation.target_y_bits == position.1.to_bits()
                && interpolation.target_z_bits == position.2.to_bits()
                && interpolation.target_y_rot_bits == y_rot.to_bits()
                && interpolation.target_x_rot_bits == x_rot.to_bits()
        });
        if target_matches {
            return;
        }

        let current_position = self.position();
        let current_rotation = self.body_rotation();
        self.interpolation = Some(EntityInterpolationState {
            target_x_bits: position.0.to_bits(),
            target_y_bits: position.1.to_bits(),
            target_z_bits: position.2.to_bits(),
            target_y_rot_bits: y_rot.to_bits(),
            target_x_rot_bits: x_rot.to_bits(),
            previous_x_bits: current_position.0.to_bits(),
            previous_y_bits: current_position.1.to_bits(),
            previous_z_bits: current_position.2.to_bits(),
            previous_y_rot_bits: current_rotation.0.to_bits(),
            previous_x_rot_bits: current_rotation.1.to_bits(),
            steps: LIVING_ENTITY_POSITION_LERP_STEPS,
        });
    }

    pub(crate) fn snap_to(&mut self, position: (f64, f64, f64), y_rot: f32, x_rot: f32) {
        self.x_bits = position.0.to_bits();
        self.y_bits = position.1.to_bits();
        self.z_bits = position.2.to_bits();
        self.y_rot_bits = y_rot.to_bits();
        self.x_rot_bits = x_rot.to_bits();
        self.old_x_bits = position.0.to_bits();
        self.old_y_bits = position.1.to_bits();
        self.old_z_bits = position.2.to_bits();
        self.old_y_rot_bits = y_rot.to_bits();
        self.old_x_rot_bits = x_rot.to_bits();
    }

    pub(crate) fn tick_position_interpolation(&mut self) {
        let current_position = self.position();
        let current_rotation = self.body_rotation();
        let Some(interpolation) = self
            .interpolation
            .as_mut()
            .filter(|interpolation| interpolation.steps > 0)
        else {
            return;
        };

        let alpha = 1.0 / f64::from(interpolation.steps);
        let previous_position = (
            f64::from_bits(interpolation.previous_x_bits),
            f64::from_bits(interpolation.previous_y_bits),
            f64::from_bits(interpolation.previous_z_bits),
        );
        let delta_position = (
            current_position.0 - previous_position.0,
            current_position.1 - previous_position.1,
            current_position.2 - previous_position.2,
        );
        interpolation.target_x_bits =
            (f64::from_bits(interpolation.target_x_bits) + delta_position.0).to_bits();
        interpolation.target_y_bits =
            (f64::from_bits(interpolation.target_y_bits) + delta_position.1).to_bits();
        interpolation.target_z_bits =
            (f64::from_bits(interpolation.target_z_bits) + delta_position.2).to_bits();

        let previous_rotation = (
            f32::from_bits(interpolation.previous_y_rot_bits),
            f32::from_bits(interpolation.previous_x_rot_bits),
        );
        let delta_rotation = (
            current_rotation.0 - previous_rotation.0,
            current_rotation.1 - previous_rotation.1,
        );
        interpolation.target_y_rot_bits =
            (f32::from_bits(interpolation.target_y_rot_bits) + delta_rotation.0).to_bits();
        interpolation.target_x_rot_bits =
            (f32::from_bits(interpolation.target_x_rot_bits) + delta_rotation.1).to_bits();

        let target_position = interpolation.target_position();
        let target_rotation = interpolation.target_rotation();
        let new_position = (
            mth_lerp_f64(alpha, current_position.0, target_position.0),
            mth_lerp_f64(alpha, current_position.1, target_position.1),
            mth_lerp_f64(alpha, current_position.2, target_position.2),
        );
        let new_rotation = (
            mth_rot_lerp_f64(
                alpha,
                f64::from(current_rotation.0),
                f64::from(target_rotation.0),
            ) as f32,
            mth_lerp_f64(
                alpha,
                f64::from(current_rotation.1),
                f64::from(target_rotation.1),
            ) as f32,
        );
        self.x_bits = new_position.0.to_bits();
        self.y_bits = new_position.1.to_bits();
        self.z_bits = new_position.2.to_bits();
        self.y_rot_bits = (new_rotation.0 % 360.0).to_bits();
        self.x_rot_bits = (new_rotation.1 % 360.0).clamp(-90.0, 90.0).to_bits();
        interpolation.steps -= 1;
        interpolation.previous_x_bits = new_position.0.to_bits();
        interpolation.previous_y_bits = new_position.1.to_bits();
        interpolation.previous_z_bits = new_position.2.to_bits();
        interpolation.previous_y_rot_bits = new_rotation.0.to_bits();
        interpolation.previous_x_rot_bits = new_rotation.1.to_bits();
    }

    pub(crate) fn tick_motion_interpolation(&mut self) {
        if self.motion_lerp_steps == 0 {
            return;
        }

        let steps = f64::from(self.motion_lerp_steps);
        let current = self.motion();
        let target = (
            f64::from_bits(self.motion_lerp_target_x_bits),
            f64::from_bits(self.motion_lerp_target_y_bits),
            f64::from_bits(self.motion_lerp_target_z_bits),
        );
        self.motion_x_bits = (current.0 + (target.0 - current.0) / steps).to_bits();
        self.motion_y_bits = (current.1 + (target.1 - current.1) / steps).to_bits();
        self.motion_z_bits = (current.2 + (target.2 - current.2) / steps).to_bits();
        self.motion_lerp_steps -= 1;
    }

    pub(crate) fn tick_snowball(&mut self) {
        let air_drag = f64::from(0.99_f32);
        let gravity = if self.no_gravity {
            0.0
        } else {
            SNOWBALL_GRAVITY
        };
        let current_motion = self.motion();
        let motion = (
            current_motion.0 * air_drag,
            (current_motion.1 - gravity) * air_drag,
            current_motion.2 * air_drag,
        );
        self.motion_x_bits = motion.0.to_bits();
        self.motion_y_bits = motion.1.to_bits();
        self.motion_z_bits = motion.2.to_bits();

        let current_position = self.position();
        self.x_bits = (current_position.0 + motion.0).to_bits();
        self.y_bits = (current_position.1 + motion.1).to_bits();
        self.z_bits = (current_position.2 + motion.2).to_bits();

        let horizontal_distance = (motion.0 * motion.0 + motion.2 * motion.2).sqrt();
        let target_x_rot =
            (mth_atan2(motion.1, horizontal_distance) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        let target_y_rot = (mth_atan2(motion.0, motion.2) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        self.x_rot_bits =
            projectile_lerp_rotation_f32(f32::from_bits(self.old_x_rot_bits), target_x_rot)
                .to_bits();
        self.y_rot_bits =
            projectile_lerp_rotation_f32(f32::from_bits(self.old_y_rot_bits), target_y_rot)
                .to_bits();
    }

    pub(crate) fn tick_arrow(&mut self) {
        let current_position = self.position();
        let current_motion = self.motion();
        self.x_bits = (current_position.0 + current_motion.0).to_bits();
        self.y_bits = (current_position.1 + current_motion.1).to_bits();
        self.z_bits = (current_position.2 + current_motion.2).to_bits();

        let horizontal_distance =
            (current_motion.0 * current_motion.0 + current_motion.2 * current_motion.2).sqrt();
        let target_x_rot = (mth_atan2(current_motion.1, horizontal_distance)
            * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        let target_y_rot =
            (mth_atan2(current_motion.0, current_motion.2) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        self.x_rot_bits =
            projectile_lerp_rotation_f32(f32::from_bits(self.x_rot_bits), target_x_rot).to_bits();
        self.y_rot_bits =
            projectile_lerp_rotation_f32(f32::from_bits(self.y_rot_bits), target_y_rot).to_bits();

        let air_friction = f64::from(VANILLA_ARROW_AIR_FRICTION);
        let motion = (
            current_motion.0 * air_friction,
            current_motion.1 * air_friction,
            current_motion.2 * air_friction,
        );
        self.motion_x_bits = motion.0.to_bits();
        self.motion_y_bits = (motion.1
            - if self.no_gravity {
                0.0
            } else {
                VANILLA_ARROW_GRAVITY
            })
        .to_bits();
        self.motion_z_bits = motion.2.to_bits();
    }

    pub(crate) fn tick_experience_orb(&mut self, path_is_open: bool) {
        if self.first_tick {
            self.first_tick = false;
            return;
        }
        if !path_is_open {
            return;
        }

        let current_motion = self.motion();
        let motion = (
            current_motion.0,
            current_motion.1
                - if self.no_gravity {
                    0.0
                } else {
                    VANILLA_EXPERIENCE_ORB_GRAVITY
                },
            current_motion.2,
        );
        let current_position = self.position();
        self.x_bits = (current_position.0 + motion.0).to_bits();
        self.y_bits = (current_position.1 + motion.1).to_bits();
        self.z_bits = (current_position.2 + motion.2).to_bits();

        let air_friction = f64::from(VANILLA_EXPERIENCE_ORB_AIR_FRICTION);
        self.motion_x_bits = (motion.0 * air_friction).to_bits();
        self.motion_y_bits = (motion.1 * air_friction).to_bits();
        self.motion_z_bits = (motion.2 * air_friction).to_bits();
    }

    pub(crate) fn tick_primed_tnt(&mut self) {
        let current_motion = self.motion();
        let motion = (
            current_motion.0,
            current_motion.1
                - if self.no_gravity {
                    0.0
                } else {
                    VANILLA_PRIMED_TNT_GRAVITY
                },
            current_motion.2,
        );
        let current_position = self.position();
        self.x_bits = (current_position.0 + motion.0).to_bits();
        self.y_bits = (current_position.1 + motion.1).to_bits();
        self.z_bits = (current_position.2 + motion.2).to_bits();

        let air_friction = f64::from(VANILLA_PRIMED_TNT_AIR_FRICTION);
        self.motion_x_bits = (motion.0 * air_friction).to_bits();
        self.motion_y_bits = (motion.1 * air_friction).to_bits();
        self.motion_z_bits = (motion.2 * air_friction).to_bits();
    }

    pub(crate) fn tick_falling_block(&mut self) {
        let current_motion = self.motion();
        let motion = (
            current_motion.0,
            current_motion.1
                - if self.no_gravity {
                    0.0
                } else {
                    VANILLA_FALLING_BLOCK_GRAVITY
                },
            current_motion.2,
        );
        let current_position = self.position();
        self.x_bits = (current_position.0 + motion.0).to_bits();
        self.y_bits = (current_position.1 + motion.1).to_bits();
        self.z_bits = (current_position.2 + motion.2).to_bits();

        let air_friction = f64::from(VANILLA_FALLING_BLOCK_AIR_FRICTION);
        self.motion_x_bits = (motion.0 * air_friction).to_bits();
        self.motion_y_bits = (motion.1 * air_friction).to_bits();
        self.motion_z_bits = (motion.2 * air_friction).to_bits();
    }

    pub(crate) fn tick_firework(&mut self) {
        let current_motion = self.motion();
        let movement = (
            current_motion.0 * VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
            current_motion.1 + VANILLA_FIREWORK_ROCKET_VERTICAL_ACCELERATION,
            current_motion.2 * VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
        );
        let current_position = self.position();
        self.x_bits = (current_position.0 + movement.0).to_bits();
        self.y_bits = (current_position.1 + movement.1).to_bits();
        self.z_bits = (current_position.2 + movement.2).to_bits();

        let horizontal_distance = (movement.0 * movement.0 + movement.2 * movement.2).sqrt();
        let target_x_rot =
            (mth_atan2(movement.1, horizontal_distance) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        let target_y_rot =
            (mth_atan2(movement.0, movement.2) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
        self.x_rot_bits =
            projectile_lerp_rotation_f32(f32::from_bits(self.old_x_rot_bits), target_x_rot)
                .to_bits();
        self.y_rot_bits =
            projectile_lerp_rotation_f32(f32::from_bits(self.old_y_rot_bits), target_y_rot)
                .to_bits();
        self.motion_x_bits = movement.0.to_bits();
        self.motion_y_bits = movement.1.to_bits();
        self.motion_z_bits = movement.2.to_bits();
    }

    pub(crate) fn tick_boat_float_in_air(&mut self) {
        let current_motion = self.motion();
        let vertical_speed = if self.no_gravity {
            0.0
        } else {
            -VANILLA_BOAT_GRAVITY
        };
        let motion = (
            current_motion.0 * f64::from(VANILLA_BOAT_AIR_FRICTION),
            current_motion.1 + vertical_speed,
            current_motion.2 * f64::from(VANILLA_BOAT_AIR_FRICTION),
        );
        self.motion_x_bits = motion.0.to_bits();
        self.motion_y_bits = motion.1.to_bits();
        self.motion_z_bits = motion.2.to_bits();
        self.delta_rotation_bits = (f32::from_bits(self.delta_rotation_bits) * 0.9_f32).to_bits();
    }

    pub(crate) fn move_by_motion(&mut self) {
        let current_position = self.position();
        let motion = self.motion();
        self.set_position((
            current_position.0 + motion.0,
            current_position.1 + motion.1,
            current_position.2 + motion.2,
        ));
    }

    pub(crate) fn tick_boat_control(&mut self, input: BoatInput, has_passengers: bool) {
        if !has_passengers {
            return;
        }

        let mut delta_rotation = f32::from_bits(self.delta_rotation_bits);
        if input.left {
            delta_rotation -= 1.0_f32;
        }
        if input.right {
            delta_rotation += 1.0_f32;
        }

        let mut acceleration = 0.0_f32;
        if input.right != input.left && !input.forward && !input.backward {
            acceleration += 0.005_f32;
        }

        let y_rot = f32::from_bits(self.y_rot_bits) + delta_rotation;
        if input.forward {
            acceleration += 0.04_f32;
        }
        if input.backward {
            acceleration -= 0.005_f32;
        }

        let x_acceleration = mth_sin_f64(f64::from(-y_rot * MTH_DEG_TO_RAD)) * acceleration;
        let z_acceleration = mth_cos_f64(f64::from(y_rot * MTH_DEG_TO_RAD)) * acceleration;
        let current_motion = self.motion();
        self.delta_rotation_bits = delta_rotation.to_bits();
        self.y_rot_bits = y_rot.to_bits();
        self.motion_x_bits = (current_motion.0 + f64::from(x_acceleration)).to_bits();
        self.motion_z_bits = (current_motion.2 + f64::from(z_acceleration)).to_bits();
        self.paddle_left = (input.right && !input.left) || input.forward;
        self.paddle_right = (input.left && !input.right) || input.forward;
    }

    pub(crate) fn set_position(&mut self, position: (f64, f64, f64)) {
        self.x_bits = position.0.to_bits();
        self.y_bits = position.1.to_bits();
        self.z_bits = position.2.to_bits();
    }

    pub fn tick_count(&self) -> i32 {
        self.tick_count
    }

    pub fn position(&self) -> (f64, f64, f64) {
        (
            f64::from_bits(self.x_bits),
            f64::from_bits(self.y_bits),
            f64::from_bits(self.z_bits),
        )
    }

    pub fn previous_position(&self) -> (f64, f64, f64) {
        (
            f64::from_bits(self.old_x_bits),
            f64::from_bits(self.old_y_bits),
            f64::from_bits(self.old_z_bits),
        )
    }

    pub fn interpolated_position(&self, partial_tick: f32) -> (f64, f64, f64) {
        let partial_tick = f64::from(partial_tick);
        let current = self.position();
        let previous = self.previous_position();
        (
            mth_lerp_f64(partial_tick, previous.0, current.0),
            mth_lerp_f64(partial_tick, previous.1, current.1),
            mth_lerp_f64(partial_tick, previous.2, current.2),
        )
    }

    pub fn motion(&self) -> (f64, f64, f64) {
        (
            f64::from_bits(self.motion_x_bits),
            f64::from_bits(self.motion_y_bits),
            f64::from_bits(self.motion_z_bits),
        )
    }

    pub fn boat_delta_rotation(&self) -> f32 {
        f32::from_bits(self.delta_rotation_bits)
    }

    pub fn paddle_state(&self) -> (bool, bool) {
        (self.paddle_left, self.paddle_right)
    }

    pub(crate) fn set_motion_if_finite(&mut self, motion: (f64, f64, f64)) {
        if motion.0.is_finite() && motion.1.is_finite() && motion.2.is_finite() {
            self.motion_x_bits = motion.0.to_bits();
            self.motion_y_bits = motion.1.to_bits();
            self.motion_z_bits = motion.2.to_bits();
        }
    }

    pub fn motion_interpolation_target(&self) -> Option<(f64, f64, f64)> {
        (self.motion_lerp_steps > 0).then(|| {
            (
                f64::from_bits(self.motion_lerp_target_x_bits),
                f64::from_bits(self.motion_lerp_target_y_bits),
                f64::from_bits(self.motion_lerp_target_z_bits),
            )
        })
    }

    pub fn motion_interpolation_steps(&self) -> u8 {
        self.motion_lerp_steps
    }

    pub fn rotation(&self) -> (f32, f32, f32) {
        (
            f32::from_bits(self.y_rot_bits),
            f32::from_bits(self.x_rot_bits),
            f32::from_bits(self.y_head_rot_bits),
        )
    }

    pub fn head_rotation_target(&self) -> f32 {
        f32::from_bits(self.head_rotation_target_bits)
    }

    pub fn head_rotation_lerp_steps(&self) -> u8 {
        self.head_lerp_steps
    }

    pub fn body_y_rotation(&self) -> f32 {
        f32::from_bits(self.y_body_rot_bits)
    }

    pub fn previous_body_y_rotation(&self) -> f32 {
        f32::from_bits(self.old_y_body_rot_bits)
    }

    pub(crate) fn clamp_vanilla_boat_passenger_rotation(
        &mut self,
        vehicle_y_rot: f32,
        animal_rotation_offset: Option<f32>,
    ) {
        let (old_y_rot, y_rot) = vanilla_boat_passenger_rotation_clamped_values(
            f32::from_bits(self.y_rot_bits),
            f32::from_bits(self.old_y_rot_bits),
            vehicle_y_rot,
        );
        self.old_y_rot_bits = old_y_rot.to_bits();
        self.y_rot_bits = y_rot.to_bits();
        if vanilla_passenger_is_living(self.type_id) {
            self.y_head_rot_bits = self.y_rot_bits;
            self.y_body_rot_bits = vehicle_y_rot.to_bits();
        }
        if let Some(rotation_offset) = animal_rotation_offset {
            self.y_body_rot_bits =
                (f32::from_bits(self.y_body_rot_bits) + rotation_offset).to_bits();
            self.y_head_rot_bits =
                (f32::from_bits(self.y_head_rot_bits) + rotation_offset).to_bits();
        }
    }

    pub(crate) fn add_vanilla_boat_passenger_rotation(&mut self, delta_rotation: f32) {
        self.y_rot_bits = (f32::from_bits(self.y_rot_bits) + delta_rotation).to_bits();
        if vanilla_passenger_is_living(self.type_id) {
            self.y_head_rot_bits =
                (f32::from_bits(self.y_head_rot_bits) + delta_rotation).to_bits();
        }
    }

    pub(crate) fn body_rotation(&self) -> (f32, f32) {
        (
            f32::from_bits(self.y_rot_bits),
            f32::from_bits(self.x_rot_bits),
        )
    }

    pub fn interpolation_target_position(&self) -> Option<(f64, f64, f64)> {
        self.interpolation
            .filter(|interpolation| interpolation.steps > 0)
            .map(EntityInterpolationState::target_position)
    }

    pub fn interpolation_target_rotation(&self) -> Option<(f32, f32)> {
        self.interpolation
            .filter(|interpolation| interpolation.steps > 0)
            .map(EntityInterpolationState::target_rotation)
    }

    pub fn interpolation_steps(&self) -> u8 {
        self.interpolation
            .filter(|interpolation| interpolation.steps > 0)
            .map_or(0, |interpolation| interpolation.steps)
    }

    pub fn previous_rotation(&self) -> (f32, f32, f32) {
        (
            f32::from_bits(self.old_y_rot_bits),
            f32::from_bits(self.old_x_rot_bits),
            f32::from_bits(self.old_y_head_rot_bits),
        )
    }

    pub fn interpolated_rotation(&self, partial_tick: f32) -> (f32, f32, f32) {
        let current = self.rotation();
        if partial_tick == 1.0 {
            return current;
        }
        let previous = self.previous_rotation();
        (
            mth_rot_lerp_f32(partial_tick, previous.0, current.0),
            mth_lerp_f32(partial_tick, previous.1, current.1),
            mth_rot_lerp_f32(partial_tick, previous.2, current.2),
        )
    }

    /// The limbs' walk animation.
    pub fn walk_animation(&self) -> &crate::living::WalkAnimation {
        &self.walk
    }

    /// `getSwimAmount(partialTick)`.
    pub fn swim_amount(&self, partial_tick: f32) -> f32 {
        let old = f32::from_bits(self.old_swim_amount_bits);
        old + partial_tick * (f32::from_bits(self.swim_amount_bits) - old)
    }

    /// The body's facing between two ticks (`Mth.rotLerp` of `yBodyRotO`
    /// and `yBodyRot`).
    pub fn interpolated_body_y_rotation(&self, partial_tick: f32) -> f32 {
        mth_rot_lerp_f32(
            partial_tick,
            self.previous_body_y_rotation(),
            self.body_y_rotation(),
        )
    }

    pub fn on_ground(&self) -> bool {
        self.on_ground
    }

    pub fn is_ticking(&self) -> bool {
        self.ticking
    }

    pub fn is_always_ticking(&self) -> bool {
        self.always_ticking
    }

    pub fn metadata(&self) -> Option<&[u8]> {
        self.metadata.as_deref()
    }

    pub fn shared_flags(&self) -> u8 {
        self.shared_flags
    }

    pub fn living_flags(&self) -> u8 {
        self.living_flags
    }

    pub fn mob_flags(&self) -> u8 {
        self.mob_flags
    }

    pub fn is_no_ai(&self) -> bool {
        shared_flag_is_set(self.mob_flags, MOB_FLAG_NO_AI)
    }

    pub fn is_left_handed(&self) -> bool {
        shared_flag_is_set(self.mob_flags, MOB_FLAG_LEFT_HANDED)
    }

    pub fn is_aggressive(&self) -> bool {
        shared_flag_is_set(self.mob_flags, MOB_FLAG_AGGRESSIVE)
    }

    pub fn is_using_item(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_USING_ITEM)
    }

    pub fn used_item_is_off_hand(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_OFF_HAND)
    }

    pub fn is_auto_spin_attack(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_SPIN_ATTACK)
    }

    pub fn air_supply(&self) -> i32 {
        self.air_supply
    }

    pub fn health(&self) -> Option<f32> {
        self.health_bits.map(f32::from_bits)
    }

    pub fn arrow_count(&self) -> i32 {
        self.arrow_count
    }

    pub fn stinger_count(&self) -> i32 {
        self.stinger_count
    }

    pub fn is_effect_ambient(&self) -> bool {
        self.effect_ambience
    }

    pub fn effect_particle_type_ids(&self) -> &[i32] {
        &self.effect_particle_type_ids
    }

    pub fn sleeping_pos(&self) -> Option<voxel_core::binary::BlockPos> {
        self.sleeping_pos
    }

    pub fn is_sleeping(&self) -> bool {
        self.sleeping_pos.is_some()
    }

    pub fn has_custom_name(&self) -> bool {
        self.custom_name_present
    }

    pub fn is_custom_name_visible(&self) -> bool {
        self.custom_name_visible
    }

    pub fn should_show_name(&self) -> bool {
        self.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID || self.custom_name_visible
    }

    pub fn is_no_gravity(&self) -> bool {
        self.no_gravity
    }

    pub fn is_silent(&self) -> bool {
        self.silent
    }

    pub fn pose(&self) -> ClientEntityPose {
        self.pose
    }

    /// Return player/Avatar dimensions for a remote player entity.
    pub fn player_dimensions(&self) -> Option<ClientEntityDimensions> {
        (self.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID).then(|| self.pose.player_dimensions())
    }

    pub fn is_baby(&self) -> bool {
        self.baby
    }

    pub fn ticks_frozen(&self) -> i32 {
        self.ticks_frozen
    }

    pub fn percent_frozen(&self) -> f32 {
        (self.ticks_frozen.min(DEFAULT_TICKS_REQUIRED_TO_FREEZE) as f32)
            / DEFAULT_TICKS_REQUIRED_TO_FREEZE as f32
    }

    pub fn is_fully_frozen(&self) -> bool {
        self.ticks_frozen >= DEFAULT_TICKS_REQUIRED_TO_FREEZE
    }

    pub fn is_freezing(&self) -> bool {
        self.ticks_frozen > 0
    }

    pub fn is_on_fire(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_ON_FIRE)
    }

    pub fn is_shift_key_down(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SHIFT_KEY_DOWN)
    }

    pub fn is_sprinting(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SPRINTING)
    }

    pub fn is_swimming(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SWIMMING)
    }

    /// Whether the entity is a player.
    pub fn is_player(&self) -> bool {
        is_vanilla_player_entity_type(self.type_id)
    }

    /// Whether the player only watches (its game mode, as the server told).
    pub fn is_spectator(&self) -> bool {
        self.spectator
    }

    pub fn is_invisible(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_INVISIBLE)
    }

    pub fn is_glowing(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_GLOWING)
    }

    pub fn is_fall_flying(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_FALL_FLYING)
    }
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

pub(crate) fn entity_chunk_key(position: (f64, f64, f64)) -> (i32, i32) {
    (
        voxel_core::math::floor_f64(position.0).div_euclid(16),
        voxel_core::math::floor_f64(position.2).div_euclid(16),
    )
}

pub(crate) fn entity_section_key(position: (f64, f64, f64)) -> (i32, i32, i32) {
    (
        entity_section_coord(position.0),
        entity_section_coord(position.1),
        entity_section_coord(position.2),
    )
}

pub(crate) fn entity_section_coord(position: f64) -> i32 {
    voxel_core::math::floor_f64(position).div_euclid(16)
}

pub(crate) fn decode_entity_delta(base: f64, delta: i16) -> u64 {
    if delta == 0 {
        base.to_bits()
    } else {
        let encoded_base = (base * 4096.0 + 0.5).floor() as i64;
        ((encoded_base + i64::from(delta)) as f64 / 4096.0).to_bits()
    }
}

pub(crate) fn resolve_entity_teleport_motion(
    current_motion: (f64, f64, f64),
    current_rotation: (f32, f32),
    target_rotation: (f32, f32),
    delta_movement: (f64, f64, f64),
    relative_flags: i32,
) -> (f64, f64, f64) {
    let current_motion = if relative_flags & ENTITY_RELATIVE_ROTATE_DELTA_FLAG != 0 {
        rotate_entity_motion(current_motion, current_rotation, target_rotation)
    } else {
        current_motion
    };
    (
        if relative_flags & ENTITY_RELATIVE_DELTA_X_FLAG != 0 {
            current_motion.0 + delta_movement.0
        } else {
            delta_movement.0
        },
        if relative_flags & ENTITY_RELATIVE_DELTA_Y_FLAG != 0 {
            current_motion.1 + delta_movement.1
        } else {
            delta_movement.1
        },
        if relative_flags & ENTITY_RELATIVE_DELTA_Z_FLAG != 0 {
            current_motion.2 + delta_movement.2
        } else {
            delta_movement.2
        },
    )
}

pub(crate) fn rotate_entity_motion(
    current_motion: (f64, f64, f64),
    current_rotation: (f32, f32),
    target_rotation: (f32, f32),
) -> (f64, f64, f64) {
    let diff_y_rot = current_rotation.0 - target_rotation.0;
    let diff_x_rot = current_rotation.1 - target_rotation.1;
    let x_radians = (f64::from(diff_x_rot) * (std::f64::consts::PI / 180.0)) as f32;
    let y_radians = (f64::from(diff_y_rot) * (std::f64::consts::PI / 180.0)) as f32;
    let x_cos = f64::from(mth_cos_f64(f64::from(x_radians)));
    let x_sin = f64::from(mth_sin_f64(f64::from(x_radians)));
    let x_rotated = (
        current_motion.0,
        current_motion.1 * x_cos + current_motion.2 * x_sin,
        current_motion.2 * x_cos - current_motion.1 * x_sin,
    );
    let y_cos = f64::from(mth_cos_f64(f64::from(y_radians)));
    let y_sin = f64::from(mth_sin_f64(f64::from(y_radians)));
    (
        x_rotated.0 * y_cos + x_rotated.2 * y_sin,
        x_rotated.1,
        x_rotated.2 * y_cos - x_rotated.0 * y_sin,
    )
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

/// Shared `position + motion - gravity` step; the operand order matches the
/// reference, so the three entities using it stay bit-identical to Java.
fn gravity_step(
    position: (f64, f64, f64),
    motion: (f64, f64, f64),
    no_gravity: bool,
    gravity: f64,
) -> (f64, f64, f64) {
    (
        position.0 + motion.0,
        position.1 + motion.1 - if no_gravity { 0.0 } else { gravity },
        position.2 + motion.2,
    )
}

/// Map a pinned entity type id onto its ballistic tick, if it has one.
pub(crate) fn vanilla_projectile_kind(type_id: i32) -> Option<VanillaProjectileKind> {
    match type_id {
        VANILLA_SNOWBALL_ENTITY_TYPE_ID => Some(VanillaProjectileKind::Snowball),
        VANILLA_ARROW_ENTITY_TYPE_ID => Some(VanillaProjectileKind::Arrow),
        VANILLA_EXPERIENCE_ORB_ENTITY_TYPE_ID => Some(VanillaProjectileKind::ExperienceOrb),
        VANILLA_PRIMED_TNT_ENTITY_TYPE_ID => Some(VanillaProjectileKind::PrimedTnt),
        VANILLA_FALLING_BLOCK_ENTITY_TYPE_ID => Some(VanillaProjectileKind::FallingBlock),
        VANILLA_FIREWORK_ROCKET_ENTITY_TYPE_ID => Some(VanillaProjectileKind::Firework),
        _ => None,
    }
}
