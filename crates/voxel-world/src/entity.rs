//! `ClientWorld` entity layer split out of the former single-file module.

use super::*;

use voxel_core::aabb::Aabb;
use voxel_core::math::{
    Vec3, clamp_f32, mth_atan2, mth_cos_f64, mth_lerp_f32, mth_lerp_f64, mth_rot_lerp_f32,
    mth_rot_lerp_f64, mth_sin_f64, projectile_lerp_rotation_f32, wrap_degrees_f32,
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

/// The block a grounded entity stands on, and whether the last look for one
/// found none (`Entity.mainSupportingBlockPos`, `onGroundNoBlocks`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Support {
    pub(crate) main_block: Option<voxel_core::binary::BlockPos>,
    pub(crate) no_blocks: bool,
}

/// What the client keeps of where an entity is, besides the position itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Tracking {
    pub(crate) support: Support,
    /// Where the server's relative position packets count from
    /// (`Entity.packetPositionCodec`): the last position a packet gave, which
    /// is not the entity's own once the client moves it.
    pub(crate) packet_base_bits: [u64; 3],
}

impl Tracking {
    pub(crate) fn at(x: f64, y: f64, z: f64) -> Self {
        Self {
            support: Support::default(),
            packet_base_bits: [x.to_bits(), y.to_bits(), z.to_bits()],
        }
    }
}

/// What an entity's tick and look keep besides its pose:
/// `LivingEntity.hurtTime` and `deathTime`, the registry id of the variant a
/// pig or a cow wears, and the value of an experience orb, once the server
/// has told them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct EntityState {
    pub(crate) hurt_time: i32,
    pub(crate) death_time: i32,
    /// How many ticks the item in use has been used (`getTicksUsingItem`, whole).
    pub(crate) using_item_ticks: i32,
    /// `LivingEntity.lastKineticHitFeedbackTime`: the game time a spear's hit last kicked, if one
    /// has.
    pub(crate) kinetic_hit_time: Option<u64>,
    /// `LivingEntity.elytraAnimationState`.
    pub(crate) elytra: crate::elytra::ElytraState,
    /// `LivingEntity.fallFlyTicks`: how long the entity has glided.
    pub(crate) fall_fly_ticks: i32,
    /// `ClientAvatarState.deltaMovementOnPreviousTick`.
    pub(crate) previous_motion: [u64; 3],
    pub(crate) variant: Option<i32>,
    pub(crate) orb_value: Option<i32>,
    /// A piglin's dancing flag.
    pub(crate) dancing: bool,
    /// A piglin's crossbow-charging flag.
    pub(crate) charging_crossbow: bool,
    /// An avatar's main arm is the left.
    pub(crate) main_arm_left: bool,
    /// A villager's type, profession and level (registry ids), once told.
    pub(crate) villager_data: Option<[i32; 3]>,
    /// `AbstractVillager.getUnhappyCounter`.
    pub(crate) unhappy_counter: i32,
    /// `Raider.isCelebrating`, and whether a spellcaster is casting.
    pub(crate) celebrating: bool,
    pub(crate) casting_spell: bool,
    /// The block state an enderman carries (zero for none), and whether it is
    /// creepy (stared at).
    pub(crate) carried_state: i32,
    pub(crate) creepy: bool,
    /// Whether a snow golem wears its pumpkin; it does until the data says not.
    pub(crate) bare_head: bool,
    /// Which of its horns a goat has lost, until the data says it has none.
    pub(crate) lost_left_horn: bool,
    pub(crate) lost_right_horn: bool,
    /// A mooshroom's variant, zero for red.
    pub(crate) mooshroom_type: i32,
    /// A rabbit's variant, zero for brown.
    pub(crate) rabbit_type: i32,
    /// Whether a creeper is charged.
    pub(crate) powered: bool,
    /// A horse's variant data (the variant in the low byte, the markings in
    /// the next) and whether a chested one carries its chest.
    pub(crate) horse_variant: i32,
    pub(crate) horse_chest: bool,
    /// A bee's flags and the game time its anger ends.
    pub(crate) bee_flags: u8,
    pub(crate) anger_end: i64,
    /// A tamable animal's flags (1 sitting, 4 tame), a wolf's interest, and
    /// the dye of its collar, when the data has said (red otherwise).
    pub(crate) tamable_flags: u8,
    pub(crate) wolf_interested: bool,
    /// A wolf's interest angle and shake.
    pub(crate) wolf: crate::wolf::WolfState,
    pub(crate) collar: Option<i32>,
    /// A cat's data (whether it lies, whether it is relaxed) and its own
    /// client state.
    pub(crate) cat_lying: bool,
    pub(crate) cat_relaxed: bool,
    pub(crate) cat: crate::cat::CatState,
    /// `Strider.isSuffocating`: it is out of lava and cold.
    pub(crate) strider_suffocating: bool,
    /// `Hoglin.isImmuneToZombification`.
    pub(crate) hoglin_immune: bool,
    /// `Entity.isInWater` for the mobs that track it.
    pub(crate) in_water: bool,
    /// `GlowSquid.getDarkTicksRemaining`.
    pub(crate) glow_dark_ticks: i32,
    /// `AbstractCubeMob.getSize` once the data has said (one before), and its squish.
    pub(crate) cube_size: Option<i32>,
    pub(crate) cube: crate::cube_mob::CubeMobState,
    /// A shulker's attach face (a direction's data value, down at first), raw
    /// peek and colour (`None` for the default), and its lid.
    pub(crate) shulker_face: i32,
    pub(crate) shulker_raw_peek: i8,
    pub(crate) shulker_color: Option<u8>,
    pub(crate) shulker: crate::shulker::ShulkerState,
    /// `Parrot.getVariant`'s id.
    pub(crate) parrot_variant: i32,
    /// `Bat.isResting`, `Vex.isCharging` and `Allay.isDancing`.
    pub(crate) bat_resting: bool,
    pub(crate) vex_charging: bool,
    pub(crate) allay_dancing: bool,
    /// `Ghast.isCharging` and `Phantom.getPhantomSize`.
    pub(crate) ghast_charging: bool,
    pub(crate) phantom_size: i32,
    /// `Guardian.isMoving` and the id of `getActiveAttackTarget` (zero for none).
    pub(crate) guardian_moving: bool,
    pub(crate) guardian_target: i32,
    /// `Axolotl.getVariant`'s id and `Axolotl.isPlayingDead`.
    pub(crate) axolotl_variant: i32,
    pub(crate) axolotl_playing_dead: bool,
    /// `Turtle.hasEgg` and `Turtle.isLayingEgg`.
    pub(crate) turtle_has_egg: bool,
    pub(crate) turtle_laying_egg: bool,
    /// A salmon's size, a tropical fish's packed variant or a pufferfish's puff
    /// state, once the data has said.
    pub(crate) fish_variant: Option<i32>,
    /// A copper golem's `CopperGolemState` and its weathering (0 to 3).
    pub(crate) copper_golem_action: crate::copper_golem::GolemAction,
    pub(crate) copper_golem_weather: u8,
    /// A polar bear's standing animation.
    pub(crate) polar_bear: crate::polar_bear::PolarBearState,
    /// A fox's flags, variant and own client state.
    pub(crate) fox: crate::fox::FoxState,
    /// A sheep's wool and eating, a chicken's flap.
    pub(crate) animal: crate::animal::AnimalState,
    /// The arm's swing.
    pub(crate) swing: crate::living::SwingState,
    /// An armor stand's flags and poses.
    pub(crate) armor_stand: crate::armor_stand::ArmorStandState,
}

impl EntityState {
    /// Takes what a cat's data holds (see [`crate::metadata::CatData`]).
    pub(crate) fn apply_cat(&mut self, cat: crate::metadata::CatData) {
        if let Some(flags) = cat.flags {
            self.tamable_flags = flags;
        }
        if let Some(lying) = cat.lying {
            self.cat_lying = lying;
        }
        if let Some(relaxed) = cat.relaxed {
            self.cat_relaxed = relaxed;
        }
        if let Some(dye) = cat.collar {
            self.collar = Some(dye);
        }
    }

    /// Takes what a wolf's data holds (see [`crate::metadata::WolfData`]).
    pub(crate) fn apply_wolf(&mut self, wolf: crate::metadata::WolfData) {
        if let Some(flags) = wolf.flags {
            self.tamable_flags = flags;
        }
        if let Some(interested) = wolf.interested {
            self.wolf_interested = interested;
        }
        if let Some(dye) = wolf.collar {
            self.collar = Some(dye);
        }
        if let Some(end) = wolf.anger_end {
            self.anger_end = end;
        }
    }
}

/// `EquipmentSlot`, in the order the wire counts them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Feet,
    Legs,
    Chest,
    Head,
    Body,
    Saddle,
}

impl EquipmentSlot {
    pub const COUNT: usize = 8;

    pub fn from_ordinal(ordinal: usize) -> Option<Self> {
        Some(match ordinal {
            0 => Self::MainHand,
            1 => Self::OffHand,
            2 => Self::Feet,
            3 => Self::Legs,
            4 => Self::Chest,
            5 => Self::Head,
            6 => Self::Body,
            7 => Self::Saddle,
            _ => return None,
        })
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

/// How a living entity's pose is ticked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LivingKind {
    Player,
    /// A `Mob`: its body follows the head by `BodyRotationControl`.
    Mob,
    /// A living entity that is neither, an armor stand.
    Other,
}

fn armor_stand_type_id() -> i32 {
    voxel_data::entity_type::id("minecraft:armor_stand").unwrap_or(-1)
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

/// `Entity.blockPosition`: the block a position is in.
fn block_of(position: (f64, f64, f64)) -> [i32; 3] {
    [position.0, position.1, position.2].map(|value| value.floor() as i32)
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
    /// `FallingBlockEntity.getStartPos`: the block the entity was added in.
    pub(crate) start_block: (i32, i32, i32),
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
    /// What a boat the local player rides keeps between ticks (status, water level, fall).
    pub(crate) boat_ride: crate::boat_ride::RideState,
    pub(crate) paddle_left: bool,
    pub(crate) paddle_right: bool,
    pub(crate) head_rotation_target_bits: u32,
    pub(crate) head_lerp_steps: u8,
    pub(crate) interpolation: Option<EntityInterpolationState>,
    pub(crate) on_ground: bool,
    pub(crate) tracking: Tracking,
    pub(crate) always_ticking: bool,
    pub(crate) ticking: bool,
    /// The limbs' walk animation and the swimming pose's ease (a player's).
    pub(crate) walk: crate::living::WalkAnimation,
    pub(crate) swim_amount_bits: u32,
    pub(crate) old_swim_amount_bits: u32,
    /// A mob's `BodyRotationControl` state: how long its head has been still,
    /// and where it was when it last moved far.
    pub(crate) head_stable_time: i32,
    pub(crate) last_stable_y_head_rot_bits: u32,
    /// What a dropped item holds (`ItemEntity.DATA_ITEM`), and where in its
    /// spin it started (`bobOffs`).
    pub(crate) item_stack: Option<ClientItemStackState>,
    pub(crate) bob_offset_bits: u32,
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
    /// A rabbit's hop and animation states, made when it first ticks.
    pub(crate) rabbit: Option<Box<crate::rabbit::RabbitState>>,
    /// A copper golem's animation states, made when it first ticks.
    pub(crate) copper_golem: Option<Box<crate::copper_golem::CopperGolemAnimations>>,
    /// A squid's tentacles and body turns, made when it first ticks.
    pub(crate) squid: Option<Box<crate::squid::SquidState>>,
    /// An axolotl's animators and animation states, made when it first ticks.
    pub(crate) axolotl: Option<Box<crate::axolotl::AxolotlState>>,
    /// A boat's hit, bubbles and paddles, made when its data first says something or it ticks.
    pub(crate) boat: Option<Box<crate::boat::BoatState>>,
    /// A bat's animation states and an allay's progress, made when they first tick.
    pub(crate) bat: Option<Box<crate::bat::BatState>>,
    /// A parrot's wing beats, made when it first ticks.
    pub(crate) parrot: Option<Box<crate::parrot::ParrotState>>,
    pub(crate) allay: Option<Box<crate::allay::AllayState>>,
    /// A guardian's animations, made when it first ticks.
    pub(crate) guardian: Option<Box<crate::guardian::GuardianState>>,
    /// A breeze's animation states, made when its pose first changes or it first ticks.
    pub(crate) breeze: Option<Box<crate::breeze::BreezeState>>,
    /// A warden's animation states and pulses, made when its pose, data or events first reach it.
    pub(crate) warden: Option<Box<crate::warden::WardenState>>,
    /// A creaking's animation states and flags, made when its data, events or ticks first reach it.
    pub(crate) creaking: Option<Box<crate::creaking::CreakingState>>,
    /// An armadillo's state and animation states, made when its data, events or ticks first reach it.
    pub(crate) armadillo: Option<crate::armadillo::ArmadilloState>,
    /// A sniffer's state and animation states, made when its data first reaches it.
    pub(crate) sniffer: Option<crate::sniffer::SnifferState>,
    /// A wither's head turns, targets and invulnerable ticks, made when its data first reaches it.
    pub(crate) wither: Option<crate::wither::WitherState>,
    /// An ender dragon's flight history, wing beat and phase, made when its data or ticks first reach it.
    pub(crate) ender_dragon: Option<Box<crate::ender_dragon::EnderDragonState>>,
    /// An end crystal's age and data, made when its data or ticks first reach it.
    pub(crate) end_crystal: Option<crate::end_crystal::EndCrystalState>,
    /// A minecart's hit, carried block and fuel, made when its data or ticks first reach it.
    pub(crate) minecart: Option<crate::minecart::MinecartState>,
    /// An arrow's shake and data, made when its data or ticks first reach it.
    pub(crate) arrow: Option<crate::arrow::ArrowState>,
    /// Evoker fangs' bite, made when the server first says they bite or they first tick.
    pub(crate) evoker_fangs: Option<crate::evoker_fangs::FangsState>,
    /// A primed tnt's fuse and block, made when its data or ticks first reach it.
    pub(crate) primed_tnt: Option<crate::primed_tnt::PrimedTntState>,
    /// `Leashable.LeashData.delayedLeashHolderId`: the entity holding the lead, zero for none.
    pub(crate) leash_holder_id: i32,
    /// A lightning bolt's shape and flashes, made when it first ticks.
    pub(crate) lightning: Option<crate::lightning::LightningState>,
    /// Whether the bolt ran out of flashes at its last tick (`LightningBolt.discard`).
    pub(crate) lightning_done: bool,
    /// `FireworkRocketEntity.isShotAtAngle`: fired from a crossbow, which turns how it is drawn.
    pub(crate) shot_at_angle: bool,
    /// A frog's animation states, made when its pose first changes or it first ticks.
    pub(crate) frog: Option<Box<crate::frog::FrogState>>,
    /// A panda's data and own client state, once it has any.
    pub(crate) panda: Option<Box<crate::panda::PandaState>>,
    /// A ravager's counters, once it has any.
    pub(crate) ravager: Option<Box<crate::ravager::RavagerState>>,
    /// A horse's animations and tail, made when it first ticks, and the flags
    /// the server sent before that.
    pub(crate) horse: Option<Box<crate::horse::HorseState>>,
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
        let bounding_box = if is_vanilla_shulker_entity_type(self.type_id) {
            self.shulker_bounding_box()
        } else {
            dimensions.make_bounding_box(Vec3::new(x, y, z))
        };
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

    /// Whether the type is a `LivingEntity`.
    pub fn is_living_entity(&self) -> bool {
        self.living_kind().is_some()
    }

    /// Which kind of living entity this is, as far as its drawn pose goes.
    pub(crate) fn living_kind(&self) -> Option<LivingKind> {
        if self.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
            Some(LivingKind::Player)
        } else if !vanilla_passenger_is_living(self.type_id) {
            None
        } else if self.type_id == armor_stand_type_id() {
            Some(LivingKind::Other)
        } else {
            Some(LivingKind::Mob)
        }
    }

    /// What a living entity's drawn pose follows from its tick: the limbs'
    /// swing with the distance moved, and the body turning (a player's after
    /// where it walks, a mob's by its body control), and for a player the
    /// swimming pose easing in or out.
    pub(crate) fn tick_living_pose(&mut self, kind: LivingKind) {
        let moved = (
            f64::from_bits(self.x_bits) - f64::from_bits(self.old_x_bits),
            f64::from_bits(self.z_bits) - f64::from_bits(self.old_z_bits),
        );
        if kind == LivingKind::Player {
            self.old_swim_amount_bits = self.swim_amount_bits;
            self.swim_amount_bits = crate::living::ease_swim_amount(
                f32::from_bits(self.swim_amount_bits),
                self.pose == ClientEntityPose::Swimming,
            )
            .to_bits();
        }
        let mut body = f32::from_bits(self.y_body_rot_bits);
        let mut body_old = f32::from_bits(self.old_y_body_rot_bits);
        let mut head = f32::from_bits(self.y_head_rot_bits);
        let mut head_old = f32::from_bits(self.old_y_head_rot_bits);
        if kind == LivingKind::Mob && self.is_shulker() {
            // `Shulker.ShulkerBodyRotationControl`: the body does not follow.
            crate::living::keep_old_angles_near((body, &mut body_old), (head, &mut head_old));
        } else if kind == LivingKind::Mob {
            let mut control = crate::living::MobBodyControl {
                head_stable_time: self.head_stable_time,
                last_stable_y_head_rot: f32::from_bits(self.last_stable_y_head_rot_bits),
            };
            crate::living::tick_mob_body(
                &mut control,
                (&mut body, &mut head),
                f32::from_bits(self.y_rot_bits),
                moved,
                !self.passenger_ids.is_empty(),
            );
            self.head_stable_time = control.head_stable_time;
            self.last_stable_y_head_rot_bits = control.last_stable_y_head_rot.to_bits();
            crate::living::keep_old_angles_near((body, &mut body_old), (head, &mut head_old));
            self.y_head_rot_bits = head.to_bits();
        } else {
            crate::living::turn_body(
                (&mut body, &mut body_old),
                (f32::from_bits(self.y_rot_bits), head, &mut head_old),
                moved,
            );
        }
        self.y_body_rot_bits = body.to_bits();
        self.old_y_body_rot_bits = body_old.to_bits();
        self.old_y_head_rot_bits = head_old.to_bits();
        // `calculateEntityAnimation`: a rider or a dead entity stands still.
        // The dragon's own `aiStep` does not reach `calculateEntityAnimation`.
        if self.vehicle_id.is_none() && self.alive && !self.is_ender_dragon() {
            let camel = matches!(
                voxel_data::entity_type::name(self.type_id),
                Some("minecraft:camel" | "minecraft:camel_husk")
            );
            if camel {
                // `Camel.updateWalkAnimation`: only a standing camel that is
                // not dashing walks, and its stride is quicker to settle.
                let walking = self.pose == ClientEntityPose::Standing
                    && !self
                        .horse
                        .as_ref()
                        .is_some_and(|horse| horse.camel.is_dash_animation_started());
                let target = if walking {
                    ((moved.0.hypot(moved.1)) as f32 * 6.0).min(1.0)
                } else {
                    0.0
                };
                self.walk
                    .update(target, 0.2, if self.baby { 3.0 } else { 1.0 });
            } else if self.is_creaking() {
                // `Creaking.updateWalkAnimation`: a quicker stride, capped at three.
                let target = (moved.0.hypot(moved.1) as f32 * 25.0).min(3.0);
                self.walk.update(target, 0.4, 1.0);
            } else if self.is_frog() {
                let target = self.frog.as_ref().map_or_else(
                    || ((moved.0.hypot(moved.1)) as f32 * 25.0).min(1.0),
                    |frog| frog.walk_target(moved.0.hypot(moved.1)),
                );
                self.walk
                    .update(target, 0.4, if self.baby { 3.0 } else { 1.0 });
            } else {
                self.walk.follow(moved.0.hypot(moved.1), self.baby);
            }
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
        if self.is_shulker() {
            // `Entity.snapTo` sets the old place from the raw one, then
            // `reapplyPosition` runs `Shulker.setPos` from the raw block.
            self.shulker_set_pos(position, block_of(position));
        }
    }

    /// `Entity.moveOrInterpolateTo`: a shulker has no interpolation
    /// (`getInterpolation` is null), so it is put at the place, and turned,
    /// at once; the rest interpolate.
    pub(crate) fn move_or_interpolate_to(
        &mut self,
        position: (f64, f64, f64),
        y_rot: f32,
        x_rot: f32,
    ) {
        if self.is_shulker() {
            self.set_position(position);
            self.y_rot_bits = (y_rot % 360.0).to_bits();
            self.x_rot_bits = (x_rot % 360.0).clamp(-90.0, 90.0).to_bits();
        } else {
            self.interpolate_to(position, y_rot, x_rot);
        }
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
        interpolation.steps -= 1;
        interpolation.previous_x_bits = new_position.0.to_bits();
        interpolation.previous_y_bits = new_position.1.to_bits();
        interpolation.previous_z_bits = new_position.2.to_bits();
        interpolation.previous_y_rot_bits = new_rotation.0.to_bits();
        interpolation.previous_x_rot_bits = new_rotation.1.to_bits();
        self.x_bits = new_position.0.to_bits();
        self.y_bits = new_position.1.to_bits();
        self.z_bits = new_position.2.to_bits();
        self.y_rot_bits = (new_rotation.0 % 360.0).to_bits();
        self.x_rot_bits = (new_rotation.1 % 360.0).clamp(-90.0, 90.0).to_bits();
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

    pub(crate) fn set_position(&mut self, position: (f64, f64, f64)) {
        if self.is_shulker() {
            self.shulker_set_pos(position, block_of(self.position()));
            return;
        }
        self.x_bits = position.0.to_bits();
        self.y_bits = position.1.to_bits();
        self.z_bits = position.2.to_bits();
    }

    /// `Entity.getPositionCodec().getBase()`.
    pub(crate) fn packet_base(&self) -> (f64, f64, f64) {
        let [x, y, z] = self.tracking.packet_base_bits.map(f64::from_bits);
        (x, y, z)
    }

    pub(crate) fn set_packet_base(&mut self, base: (f64, f64, f64)) {
        self.tracking.packet_base_bits = [base.0.to_bits(), base.1.to_bits(), base.2.to_bits()];
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

    /// `xRotO`.
    pub fn previous_x_rotation(&self) -> f32 {
        f32::from_bits(self.old_x_rot_bits)
    }

    /// `yBodyRot`.
    pub fn y_body_rot(&self) -> f32 {
        f32::from_bits(self.y_body_rot_bits)
    }

    /// `getYHeadRot`.
    pub fn y_head_rot(&self) -> f32 {
        f32::from_bits(self.y_head_rot_bits)
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

    /// `Entity.getYRot(partialTick)`: the turn between two ticks, a plain
    /// lerp (a boat's own, which does not wrap).
    pub fn y_rot_at(&self, partial_tick: f32) -> f32 {
        let current = self.rotation().0;
        if partial_tick == 1.0 {
            return current;
        }
        mth_lerp_f32(partial_tick, self.previous_rotation().0, current)
    }

    /// `Entity.getXRot(partialTick)`: the pitch between two ticks, a plain lerp.
    pub fn x_rot_at(&self, partial_tick: f32) -> f32 {
        let current = self.rotation().1;
        if partial_tick == 1.0 {
            return current;
        }
        mth_lerp_f32(partial_tick, self.previous_rotation().1, current)
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

    /// Whether the type is a `Zombie` or one of its subclasses, which share
    /// `DATA_BABY_ID`.
    pub fn is_zombie_type(&self) -> bool {
        voxel_data::entity_type::name(self.type_id).is_some_and(|name| {
            matches!(
                name,
                "minecraft:zombie"
                    | "minecraft:husk"
                    | "minecraft:drowned"
                    | "minecraft:zombie_villager"
                    | "minecraft:zombified_piglin"
            )
        })
    }

    pub fn is_aggressive(&self) -> bool {
        shared_flag_is_set(self.mob_flags, MOB_FLAG_AGGRESSIVE)
    }

    pub fn is_using_item(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_USING_ITEM)
    }

    /// `LivingEntity.getFallFlyingTicks`.
    pub fn fall_flying_ticks(&self) -> i32 {
        self.living.fall_fly_ticks
    }

    /// `ClientAvatarState.deltaMovementOnPreviousTick`.
    pub fn previous_motion(&self) -> (f64, f64, f64) {
        let [x, y, z] = self.living.previous_motion.map(f64::from_bits);
        (x, y, z)
    }

    /// `ElytraAnimationState.getRotX/Y/Z`: how far the wings are spread.
    pub fn elytra_rotations(&self, partial_tick: f32) -> [f32; 3] {
        self.living.elytra.rotations(partial_tick)
    }

    /// `LivingEntity.getTicksUsingItem(partialTick)`: how long the item in use has been used.
    pub fn ticks_using_item(&self, partial_tick: f32) -> f32 {
        if self.is_using_item() {
            self.living.using_item_ticks as f32 + partial_tick
        } else {
            0.0
        }
    }

    /// `LivingEntity.onKineticHit` (entity event 2): the kick of a spear's hit starts, unless one
    /// began in the last ten ticks.
    pub(crate) fn on_kinetic_hit(&mut self, game_time: u64) {
        if self
            .living
            .kinetic_hit_time
            .is_none_or(|last| game_time.saturating_sub(last) > 10)
        {
            self.living.kinetic_hit_time = Some(game_time);
        }
    }

    /// `LivingEntity.getTicksSinceLastKineticHitFeedback`.
    pub fn ticks_since_kinetic_hit(&self, game_time: u64, partial: f32) -> f32 {
        self.living
            .kinetic_hit_time
            .map_or(0.0, |last| game_time.saturating_sub(last) as f32 + partial)
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

    /// `LivingEntity.isDeadOrDying`: its health, as the server told, is spent.
    pub fn is_dead_or_dying(&self) -> bool {
        self.health().is_some_and(|health| health <= 0.0)
    }

    /// The registry id of the variant (`PigVariant`, `CowVariant`) the server
    /// set, or none while it wears the default.
    pub fn variant(&self) -> Option<i32> {
        self.living.variant
    }

    /// `Piglin.isChargingCrossbow`.
    pub fn is_charging_crossbow(&self) -> bool {
        self.living.charging_crossbow
    }

    /// Whether a piglin dances (`Piglin.isDancing`).
    pub fn is_dancing(&self) -> bool {
        self.living.dancing
    }

    /// An experience orb's value (`ExperienceOrb.getValue`), once told.
    pub fn orb_value(&self) -> Option<i32> {
        self.living.orb_value
    }

    pub fn hurt_time(&self) -> i32 {
        self.living.hurt_time
    }

    pub fn death_time(&self) -> i32 {
        self.living.death_time
    }

    /// `LivingEntity.baseTick`'s share: `hurtTime` counts down, and a dying
    /// entity's `deathTime` up (`tickDeath`; the server removes it later) while
    /// the level says deaths tick here (`shouldTickDeath`).
    pub(crate) fn tick_hurt_and_death(&mut self, tick_death: bool) {
        if self.living.hurt_time > 0 {
            self.living.hurt_time -= 1;
        }
        if self.is_dead_or_dying() && tick_death {
            if self.is_ender_dragon() {
                // `EnderDragon.tickDeath` keeps its own count, not `deathTime`, and
                // lifts the dragon and its parts a tenth of a block.
                let lift = f64::from(0.1_f32);
                self.ender_dragon_mut().tick_death(lift);
                self.y_bits = (f64::from_bits(self.y_bits) + lift).to_bits();
            } else {
                self.living.death_time += 1;
            }
        }
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

    /// The stack a dropped item holds.
    pub fn item_stack(&self) -> Option<&ClientItemStackState> {
        self.item_stack.as_ref()
    }

    /// What the entity holds or wears in a slot (`getItemBySlot`); none for
    /// an empty one.
    pub fn equipment(&self, slot: EquipmentSlot) -> Option<&ClientItemStackState> {
        self.equipment[slot as usize].as_ref()
    }

    /// `getMainArm`: a mob's left-handed flag, an avatar's data.
    pub fn main_arm_is_left(&self) -> bool {
        if self.is_player() {
            self.living.main_arm_left
        } else {
            self.is_left_handed()
        }
    }

    /// `Villager.getVillagerData`: the registry ids of its type and profession
    /// and its level; a plains villager without a profession, at level one,
    /// until the server says otherwise (`createDefaultVillagerData`).
    pub fn villager_data(&self) -> [i32; 3] {
        self.living.villager_data.unwrap_or([2, 0, 1])
    }

    /// Whether a villager or a wandering trader shakes its head
    /// (`getUnhappyCounter() > 0`).
    pub fn is_unhappy(&self) -> bool {
        self.living.unhappy_counter > 0
    }

    /// Whether an enderman carries a block (`getCarriedBlock() != null`).
    pub fn is_carrying_block(&self) -> bool {
        self.living.carried_state != 0
    }

    /// The block state an enderman carries (`getCarriedBlock`), by its id.
    pub fn carried_block_state(&self) -> Option<u32> {
        u32::try_from(self.living.carried_state)
            .ok()
            .filter(|state| *state != 0)
    }

    /// A horse's animations and tail, if it has ticked.
    pub fn horse(&self) -> Option<&crate::horse::HorseState> {
        self.horse.as_deref()
    }

    /// `Horse.getVariant` and `getMarkings`: ids, each wrapped into its enum
    /// (seven variants, five markings).
    pub fn horse_look(&self) -> (i32, i32) {
        let data = self.living.horse_variant;
        (
            (data & 0xFF).rem_euclid(7),
            ((data & 0xFF00) >> 8).rem_euclid(5),
        )
    }

    /// `Llama.getVariant`'s id, kept within the four variants (an id past the
    /// last takes the last).
    pub fn llama_variant(&self) -> i32 {
        self.living.horse_variant.clamp(0, 3)
    }

    /// `AbstractChestedHorse.hasChest`.
    pub fn horse_has_chest(&self) -> bool {
        self.living.horse_chest
    }

    /// Fixes the source a horse's tail chance is drawn from (see
    /// [`crate::horse::HorseState::fix_seed`]).
    pub fn fix_horse_seed(&mut self, seed: i64) {
        self.horse
            .get_or_insert_with(|| Box::new(crate::horse::HorseState::new()))
            .fix_seed(seed);
    }

    /// A rabbit's hop and animation states, if it has ticked.
    pub fn rabbit(&self) -> Option<&crate::rabbit::RabbitState> {
        self.rabbit.as_deref()
    }

    /// Fixes the idle head tilt's wait and random source of a rabbit (see
    /// [`crate::rabbit::RabbitState::fix_idle`]).
    pub fn fix_rabbit_idle(&mut self, timeout: i32, seed: i64) {
        self.rabbit
            .get_or_insert_with(|| Box::new(crate::rabbit::RabbitState::new()))
            .fix_idle(timeout, seed);
    }

    /// A squid's tentacles and body turns, if it has ticked.
    pub fn squid(&self) -> Option<&crate::squid::SquidState> {
        self.squid.as_deref()
    }

    pub(crate) fn is_squid(&self) -> bool {
        matches!(
            voxel_data::entity_type::name(self.type_id),
            Some("minecraft:squid" | "minecraft:glow_squid")
        )
    }

    /// Whether the entity's water state is tracked (`Entity.isInWater`): the
    /// mobs that draw differently in water.
    pub(crate) fn tracks_water(&self) -> bool {
        self.is_squid()
            || self.is_fish()
            || matches!(
                voxel_data::entity_type::name(self.type_id),
                Some(
                    "minecraft:turtle"
                        | "minecraft:axolotl"
                        | "minecraft:frog"
                        | "minecraft:tadpole"
                        | "minecraft:guardian"
                        | "minecraft:elder_guardian"
                )
            )
    }

    /// `Bat.isResting`.
    pub fn is_bat_resting(&self) -> bool {
        self.living.bat_resting
    }

    /// `Vex.isCharging`.
    pub fn is_vex_charging(&self) -> bool {
        self.living.vex_charging
    }

    /// `Allay.isDancing`.
    pub fn is_allay_dancing(&self) -> bool {
        self.living.allay_dancing
    }

    /// What a boat the local player rides keeps between ticks.
    pub fn boat_ride(&self) -> crate::boat_ride::RideState {
        self.boat_ride
    }

    /// Whether this is a boat, a chest boat or a raft.
    pub fn is_boat(&self) -> bool {
        vanilla_boat_max_passengers(self.type_id).is_some()
    }

    /// A boat's hit, bubbles and paddles, if it has any.
    pub fn boat(&self) -> Option<&crate::boat::BoatState> {
        self.boat.as_deref()
    }

    /// Whether a boat's paddle on a side (zero left, one right) rows
    /// (`AbstractBoat.getPaddleState`): its data says so and someone rows it.
    pub fn boat_paddle_rows(&self, side: usize) -> bool {
        let flag = if side == 0 {
            self.paddle_left
        } else {
            self.paddle_right
        };
        flag && !self.passenger_ids.is_empty()
    }

    /// `AbstractBoat.tick` on the client, at the start of the tick (`under_water` is
    /// none for an entity that is not a boat).
    pub(crate) fn tick_boat(&mut self, under_water: Option<bool>) {
        let Some(under_water) = under_water else {
            return;
        };
        let paddling = [self.boat_paddle_rows(0), self.boat_paddle_rows(1)];
        let tick_count = self.tick_count;
        self.boat
            .get_or_insert_with(Default::default)
            .tick(under_water, paddling, tick_count);
    }

    /// The part of a ridden boat's tick before its own movement.
    pub(crate) fn tick_boat_start(&mut self) {
        self.boat
            .get_or_insert_with(Default::default)
            .tick_start(false);
    }

    /// The part of a ridden boat's tick after its movement: the paddles row and the bubbles shake.
    pub(crate) fn tick_boat_end(&mut self, paddling: [bool; 2]) {
        let tick_count = self.tick_count;
        self.boat
            .get_or_insert_with(Default::default)
            .tick_end(paddling, tick_count);
    }

    /// A boat's data: its hit, paddles and bubbles.
    pub(crate) fn set_boat_data(&mut self, data: crate::metadata::BoatData) {
        let boat = self.boat.get_or_insert_with(Default::default);
        if let Some(hurt) = data.hurt_time {
            boat.hurt_time = hurt;
        }
        if let Some(dir) = data.hurt_dir {
            boat.hurt_dir = dir;
        }
        if let Some(damage) = data.damage {
            boat.damage = damage;
        }
        if let Some(bubbles) = data.bubble_time {
            boat.bubble_time = bubbles;
        }
        if let Some(left) = data.paddle_left {
            self.paddle_left = left;
        }
        if let Some(right) = data.paddle_right {
            self.paddle_right = right;
        }
    }

    /// `Parrot.getVariant`'s id.
    pub fn parrot_variant(&self) -> i32 {
        self.living.parrot_variant
    }

    /// A parrot's wing beats, if it has ticked.
    pub fn parrot(&self) -> Option<&crate::parrot::ParrotState> {
        self.parrot.as_deref()
    }

    /// `Parrot.calculateFlapping` after `super.aiStep`: the wings beat by
    /// whether the parrot is in the air, and one falling there sinks slowly.
    pub(crate) fn tick_parrot(&mut self) {
        if voxel_data::entity_type::name(self.type_id) != Some("minecraft:parrot") {
            return;
        }
        let (on_ground, passenger) = (self.on_ground, self.vehicle_id.is_some());
        if !on_ground && f64::from_bits(self.motion_y_bits) < 0.0 {
            self.motion_y_bits = (f64::from_bits(self.motion_y_bits) * 0.6).to_bits();
        }
        self.parrot
            .get_or_insert_with(Default::default)
            .tick(on_ground, passenger);
    }

    /// A bat's animation states, if it has ticked.
    pub fn bat(&self) -> Option<&crate::bat::BatState> {
        self.bat.as_deref()
    }

    /// An allay's progress, if it has ticked.
    pub fn allay(&self) -> Option<&crate::allay::AllayState> {
        self.allay.as_deref()
    }

    /// `Bat.tick` after `super.tick`: a hanging bat does not move and hangs
    /// from the block above it; one flying slows its fall and climb.
    pub(crate) fn tick_bat(&mut self) {
        if voxel_data::entity_type::name(self.type_id) != Some("minecraft:bat") {
            return;
        }
        let resting = self.living.bat_resting;
        if resting {
            self.motion_x_bits = 0.0_f64.to_bits();
            self.motion_y_bits = 0.0_f64.to_bits();
            self.motion_z_bits = 0.0_f64.to_bits();
            let height = voxel_data::entity_type::pick(self.type_id)
                .map_or(0.0, |pick| pick.size.1 * self.scale());
            let y = f64::from_bits(self.y_bits);
            self.y_bits = (y.floor() + 1.0 - f64::from(height)).to_bits();
        } else {
            self.motion_y_bits = (f64::from_bits(self.motion_y_bits) * 0.6).to_bits();
        }
        let tick_count = self.tick_count;
        self.bat
            .get_or_insert_with(Default::default)
            .tick(resting, tick_count);
    }

    /// `Allay.tick`'s client half, after `super.tick`.
    pub(crate) fn tick_allay(&mut self) {
        if voxel_data::entity_type::name(self.type_id) != Some("minecraft:allay") {
            return;
        }
        let holds = self.equipment[EquipmentSlot::MainHand as usize].is_some();
        let dancing = self.living.allay_dancing;
        self.allay
            .get_or_insert_with(Default::default)
            .tick(holds, dancing);
    }

    /// `Ghast.isCharging`: the ghast has its mouth open for a fireball.
    pub fn is_ghast_charging(&self) -> bool {
        self.living.ghast_charging
    }

    /// `Phantom.getPhantomSize`.
    pub fn phantom_size(&self) -> i32 {
        self.living.phantom_size
    }

    /// A guardian's animations, if it has ticked.
    pub fn guardian(&self) -> Option<&crate::guardian::GuardianState> {
        self.guardian.as_deref()
    }

    pub(crate) fn is_guardian(&self) -> bool {
        matches!(
            voxel_data::entity_type::name(self.type_id),
            Some("minecraft:guardian" | "minecraft:elder_guardian")
        )
    }

    /// `Guardian.getActiveAttackTarget`'s id: what the beam aims at, if anything.
    pub fn guardian_attack_target(&self) -> Option<i32> {
        (self.living.guardian_target != 0).then_some(self.living.guardian_target)
    }

    /// `Guardian.isMoving`.
    pub fn is_guardian_moving(&self) -> bool {
        self.living.guardian_moving
    }

    /// `Guardian.onSyncedDataUpdated`: when the data holds the attack target,
    /// the beam starts over.
    pub(crate) fn set_guardian_data(&mut self, moving: Option<bool>, target: Option<i32>) {
        if let Some(moving) = moving {
            self.living.guardian_moving = moving;
        }
        if let Some(target) = target {
            self.living.guardian_target = target;
            self.guardian
                .get_or_insert_with(|| Box::new(crate::guardian::GuardianState::new()))
                .restart_attack();
        }
    }

    /// `Guardian.aiStep`'s client half, while the guardian lives: the beam's
    /// target (which the world resolves from its id) is where the head turns.
    pub(crate) fn tick_guardian(&mut self, target: Option<crate::guardian::GuardianTarget>) {
        if !self.is_guardian() || !self.alive {
            return;
        }
        let eye_height = self
            .dimensions()
            .scale(self.scale(), self.scale())
            .eye_height();
        let place = crate::guardian::GuardianPlace {
            in_water: self.living.in_water,
            moving: self.living.guardian_moving,
            position: self.position(),
            eye_height,
            target,
        };
        let mut pose = crate::guardian::GuardianPose {
            y_rot: f32::from_bits(self.y_rot_bits),
            x_rot: f32::from_bits(self.x_rot_bits),
            y_head_rot: f32::from_bits(self.y_head_rot_bits),
            motion: self.motion(),
            on_ground: self.on_ground,
        };
        let has_target = self.living.guardian_target != 0;
        let duration = self.guardian_attack_duration();
        self.guardian
            .get_or_insert_with(|| Box::new(crate::guardian::GuardianState::new()))
            .tick(&place, has_target, duration, &mut pose);
        self.y_rot_bits = pose.y_rot.to_bits();
        self.x_rot_bits = pose.x_rot.to_bits();
        self.y_head_rot_bits = pose.y_head_rot.to_bits();
        (self.motion_x_bits, self.motion_y_bits, self.motion_z_bits) = (
            pose.motion.0.to_bits(),
            pose.motion.1.to_bits(),
            pose.motion.2.to_bits(),
        );
        self.on_ground = pose.on_ground;
    }

    /// `Guardian.getAttackDuration`, shorter for the elder.
    pub fn guardian_attack_duration(&self) -> i32 {
        if voxel_data::entity_type::name(self.type_id) == Some("minecraft:elder_guardian") {
            60
        } else {
            80
        }
    }

    /// Whether this is a frog.
    pub(crate) fn is_frog(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:frog")
    }

    /// A frog's animation states, if it has any.
    pub fn frog(&self) -> Option<&crate::frog::FrogState> {
        self.frog.as_deref()
    }

    /// `Frog.onSyncedDataUpdated(DATA_POSE)`: the pose the data names starts
    /// or stops the frog's animations.
    pub(crate) fn set_pose(&mut self, pose: ClientEntityPose) {
        self.pose = pose;
        if self.is_frog() {
            let tick_count = self.tick_count;
            self.frog
                .get_or_insert_with(Default::default)
                .set_pose(pose, tick_count);
        }
        if self.is_breeze() {
            let tick_count = self.tick_count;
            self.breeze
                .get_or_insert_with(Default::default)
                .set_pose(pose, tick_count);
        }
        if self.is_warden() {
            let tick_count = self.tick_count;
            self.warden
                .get_or_insert_with(Default::default)
                .set_pose(pose, tick_count);
        }
    }

    /// A sniffer's state and animation states, if it has any.
    pub fn sniffer(&self) -> Option<&crate::sniffer::SnifferState> {
        self.sniffer.as_ref()
    }

    /// `Sniffer.onSyncedDataUpdated(DATA_STATE)`.
    pub(crate) fn set_sniffer_mode(&mut self, mode: crate::sniffer::SnifferMode) {
        let tick_count = self.tick_count;
        self.sniffer
            .get_or_insert_with(Default::default)
            .set_mode(mode, tick_count);
    }

    /// `FireworkRocketEntity.isShotAtAngle`.
    pub fn is_shot_at_angle(&self) -> bool {
        self.shot_at_angle
    }

    /// Whether the type draws as an item facing the camera (`ThrownItemRenderer`, with its
    /// scale and whether it is fully lit), and so has its item among its data.
    pub fn thrown_item_look(&self) -> Option<(f32, bool)> {
        match voxel_data::entity_type::name(self.type_id)? {
            "minecraft:egg"
            | "minecraft:ender_pearl"
            | "minecraft:experience_bottle"
            | "minecraft:lingering_potion"
            | "minecraft:snowball"
            | "minecraft:splash_potion" => Some((1.0, false)),
            "minecraft:eye_of_ender" => Some((1.0, true)),
            "minecraft:fireball" => Some((3.0, true)),
            "minecraft:small_fireball" => Some((0.75, true)),
            _ => None,
        }
    }

    /// Whether this is an arrow, a spectral arrow or a trident (`AbstractArrow`).
    pub fn is_abstract_arrow(&self) -> bool {
        matches!(
            voxel_data::entity_type::name(self.type_id),
            Some("minecraft:arrow" | "minecraft:spectral_arrow" | "minecraft:trident")
        )
    }

    /// The block the entity was added in (`FallingBlockEntity.getStartPos`).
    pub fn start_block(&self) -> (i32, i32, i32) {
        self.start_block
    }

    /// The id of the entity this one is leashed to, if it is.
    pub fn leash_holder_id(&self) -> Option<i32> {
        (self.leash_holder_id != 0).then_some(self.leash_holder_id)
    }

    /// Whether this is a lightning bolt (`LightningBolt`).
    pub fn is_lightning_bolt(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:lightning_bolt")
    }

    /// A lightning bolt's shape and flashes, if it has begun to tick.
    pub fn lightning(&self) -> Option<&crate::lightning::LightningState> {
        self.lightning.as_ref()
    }

    pub(crate) fn lightning_finished(&self) -> bool {
        self.lightning_done
    }

    /// Whether this is a block of primed tnt (`PrimedTnt`).
    pub fn is_primed_tnt(&self) -> bool {
        self.type_id == VANILLA_PRIMED_TNT_ENTITY_TYPE_ID
    }

    /// A primed tnt's fuse and block, if it has any.
    pub fn primed_tnt(&self) -> Option<&crate::primed_tnt::PrimedTntState> {
        self.primed_tnt.as_ref()
    }

    pub(crate) fn primed_tnt_mut(&mut self) -> &mut crate::primed_tnt::PrimedTntState {
        self.primed_tnt.get_or_insert_with(Default::default)
    }

    /// Whether this is a pair of evoker fangs.
    pub fn is_evoker_fangs(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:evoker_fangs")
    }

    /// Evoker fangs' bite, if it has begun to be told of.
    pub fn evoker_fangs(&self) -> Option<&crate::evoker_fangs::FangsState> {
        self.evoker_fangs.as_ref()
    }

    pub(crate) fn evoker_fangs_mut(&mut self) -> &mut crate::evoker_fangs::FangsState {
        self.evoker_fangs.get_or_insert_with(Default::default)
    }

    /// An arrow's shake and data, if it has any.
    pub fn arrow(&self) -> Option<&crate::arrow::ArrowState> {
        self.arrow.as_ref()
    }

    pub(crate) fn arrow_mut(&mut self) -> &mut crate::arrow::ArrowState {
        self.arrow.get_or_insert_with(Default::default)
    }

    /// Whether this is one of the minecarts (`AbstractMinecart`).
    pub fn is_minecart(&self) -> bool {
        voxel_data::entity_type::name(self.type_id)
            .and_then(crate::minecart::default_display_offset)
            .is_some()
    }

    /// A minecart's hit, carried block and fuel, if it has any.
    pub fn minecart(&self) -> Option<&crate::minecart::MinecartState> {
        self.minecart.as_ref()
    }

    pub(crate) fn minecart_mut(&mut self) -> &mut crate::minecart::MinecartState {
        self.minecart.get_or_insert_with(Default::default)
    }

    /// `AbstractMinecart.getDisplayBlockState`: the block state the server named, else the
    /// type's own (the plain minecart shows none).
    pub fn minecart_display_block(&self) -> Option<u32> {
        let state = self.minecart.unwrap_or_default();
        state.custom_display.or_else(|| {
            voxel_data::entity_type::name(self.type_id)
                .and_then(|name| crate::minecart::default_display_block(name, state.fuel))
        })
    }

    /// `AbstractMinecart.getDisplayOffset`, in sixteenths of a block.
    pub fn minecart_display_offset(&self) -> i32 {
        let default = voxel_data::entity_type::name(self.type_id)
            .and_then(crate::minecart::default_display_offset)
            .unwrap_or(6);
        self.minecart.unwrap_or_default().display_offset(default)
    }

    /// `AbstractMinecart.tick` on the client, with `OldMinecartBehavior.tick`'s client half: the
    /// hit runs down, then the position and turn ease to the last packet's while that lasts,
    /// else the turns are kept under a whole turn.
    pub(crate) fn tick_minecart(&mut self) {
        self.minecart_mut().tick();
        if voxel_data::entity_type::name(self.type_id) == Some("minecraft:tnt_minecart") {
            self.minecart_mut().burn_fuse();
        }
        if self.is_interpolating() {
            self.tick_position_interpolation();
        } else {
            let (y_rot, x_rot) = (
                f32::from_bits(self.y_rot_bits),
                f32::from_bits(self.x_rot_bits),
            );
            self.x_rot_bits = (x_rot % 360.0).to_bits();
            self.y_rot_bits = (y_rot % 360.0).to_bits();
        }
    }

    pub(crate) fn is_end_crystal(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:end_crystal")
    }

    /// An end crystal's age and data, if it has any.
    pub fn end_crystal(&self) -> Option<&crate::end_crystal::EndCrystalState> {
        self.end_crystal.as_ref()
    }

    pub(crate) fn end_crystal_mut(&mut self) -> &mut crate::end_crystal::EndCrystalState {
        let seed = i64::from(self.id);
        self.end_crystal
            .get_or_insert_with(|| crate::end_crystal::EndCrystalState::new(seed))
    }

    pub(crate) fn is_ender_dragon(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:ender_dragon")
    }

    /// An ender dragon's flight history, wing beat and phase, if it has any.
    pub fn ender_dragon(&self) -> Option<&crate::ender_dragon::EnderDragonState> {
        self.ender_dragon.as_deref()
    }

    pub(crate) fn ender_dragon_mut(&mut self) -> &mut crate::ender_dragon::EnderDragonState {
        let seed = i64::from(self.id);
        self.ender_dragon
            .get_or_insert_with(|| Box::new(crate::ender_dragon::EnderDragonState::new(seed)))
    }

    /// `EnderDragon.aiStep` on the client, which replaces `LivingEntity.aiStep`: the wing
    /// beat and the flight history, then the position eases to the last packet's. Nothing
    /// fades its motion and its head does not turn after anything; while it dies, none of it.
    pub(crate) fn tick_ender_dragon(&mut self, targets: &EntityTargets) {
        self.ender_dragon_mut().begin_ai_step();
        if self.is_dead_or_dying() {
            return;
        }
        let tick_count = self.tick_count;
        let heals = self
            .ender_dragon_mut()
            .check_crystals(targets.dragon, tick_count);
        if heals
            && let (Some(health), Some(max)) = (self.health(), self.max_health())
            && health < max
        {
            self.health_bits = Some((health + 1.0).to_bits());
        }
        let y_rot = voxel_core::math::wrap_degrees_f32(f32::from_bits(self.y_rot_bits));
        self.y_rot_bits = y_rot.to_bits();
        let (motion, no_ai, y) = (self.motion(), self.is_no_ai(), self.position().1);
        self.ender_dragon_mut().fly(motion, no_ai, (y, y_rot));
        if !no_ai {
            self.tick_position_interpolation();
            self.y_body_rot_bits = self.y_rot_bits;
            let (at, y_rot) = (self.position(), f32::from_bits(self.y_rot_bits));
            self.ender_dragon_mut().tick_parts(at, y_rot);
        }
    }

    pub(crate) fn is_wither(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:wither")
    }

    /// A wither's head turns, targets and invulnerable ticks, if it has any.
    pub fn wither(&self) -> Option<&crate::wither::WitherState> {
        self.wither.as_ref()
    }

    pub(crate) fn wither_mut(&mut self) -> &mut crate::wither::WitherState {
        self.wither.get_or_insert_with(Default::default)
    }

    /// The ids of the entities a wither's side heads look at, zero for none.
    pub(crate) fn wither_targets(&self) -> [i32; 2] {
        self.wither.map_or([0; 2], |wither| wither.targets)
    }

    /// `WitherBoss.aiStep` before `super.aiStep`: the motion's height settles by
    /// a factor of 0.6, and a body moving faster than that turns to face where it goes.
    pub(crate) fn tick_wither_motion(&mut self) {
        if !self.is_wither() {
            return;
        }
        let (x, y, z) = self.motion();
        self.motion_y_bits = (y * 0.6).to_bits();
        if x * x + z * z > 0.05 {
            let turn = voxel_core::math::mth_atan2(z, x) as f32
                * (180.0_f32 / std::f64::consts::PI as f32)
                - 90.0;
            self.y_rot_bits = turn.to_bits();
        }
    }

    /// `WitherBoss.aiStep` after `super.aiStep`: the side heads turn.
    pub(crate) fn tick_wither_heads(
        &mut self,
        targets: [Option<crate::guardian::GuardianTarget>; 2],
    ) {
        if !self.is_wither() {
            return;
        }
        let place = crate::wither::WitherPlace {
            position: self.position(),
            body_rot: f32::from_bits(self.y_body_rot_bits),
            scale: self.scale(),
        };
        self.wither_mut().tick_heads(place, targets);
    }

    pub(crate) fn is_armadillo(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:armadillo")
    }

    /// An armadillo's state and animation states, if it has any.
    pub fn armadillo(&self) -> Option<&crate::armadillo::ArmadilloState> {
        self.armadillo.as_ref()
    }

    pub(crate) fn armadillo_mut(&mut self) -> &mut crate::armadillo::ArmadilloState {
        self.armadillo.get_or_insert_with(Default::default)
    }

    /// `Armadillo.tick`'s client half, after the rest of the tick: the animation
    /// states follow the state, a scared armadillo's head is held to its body
    /// (`Mob.clampHeadRotationToBody`), and the time in the state counts on.
    pub(crate) fn tick_armadillo(&mut self) {
        if !self.is_armadillo() {
            return;
        }
        let tick_count = self.tick_count;
        let state = self.armadillo_mut();
        state.animate(tick_count);
        let scared = state.is_scared();
        if scared {
            let (head, body) = (
                f32::from_bits(self.y_head_rot_bits),
                f32::from_bits(self.y_body_rot_bits),
            );
            let limit = 75.0_f32;
            let delta = voxel_core::math::wrap_degrees_f32(body - head);
            let target = voxel_core::math::wrap_degrees_f32(body - head).clamp(-limit, limit);
            self.y_head_rot_bits = (head + delta - target).to_bits();
        }
        self.armadillo_mut().finish_tick();
    }

    pub(crate) fn is_creaking(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:creaking")
    }

    /// A creaking's animation states and flags, if it has any.
    pub fn creaking(&self) -> Option<&crate::creaking::CreakingState> {
        self.creaking.as_deref()
    }

    /// The creaking's state, made if it has none yet.
    pub(crate) fn creaking_mut(&mut self) -> &mut crate::creaking::CreakingState {
        let seed = i64::from(self.id);
        self.creaking
            .get_or_insert_with(|| Box::new(crate::creaking::CreakingState::new(seed)))
    }

    /// `Creaking.tick`'s client half, after the rest of the tick.
    pub(crate) fn tick_creaking(&mut self) {
        if self.is_creaking() {
            let (tick_count, death_time) = (self.tick_count, self.living.death_time);
            self.creaking_mut().tick(tick_count, death_time);
        }
    }

    pub(crate) fn is_warden(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:warden")
    }

    /// A warden's animation states and pulses, if it has any.
    pub fn warden(&self) -> Option<&crate::warden::WardenState> {
        self.warden.as_deref()
    }

    /// The warden's state, made if it has none yet.
    pub(crate) fn warden_mut(&mut self) -> &mut crate::warden::WardenState {
        self.warden.get_or_insert_with(Default::default)
    }

    /// `Warden.tick`'s client half, after the rest of the tick.
    pub(crate) fn tick_warden(&mut self) {
        if self.is_warden() {
            let tick_count = self.tick_count;
            self.warden_mut().tick(tick_count);
        }
    }

    pub(crate) fn is_breeze(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:breeze")
    }

    /// A breeze's animation states, if it has any.
    pub fn breeze(&self) -> Option<&crate::breeze::BreezeState> {
        self.breeze.as_deref()
    }

    /// `Breeze.tick`'s client half, before the rest of the tick.
    pub(crate) fn tick_breeze(&mut self) {
        if !self.is_breeze() {
            return;
        }
        let (pose, tick_count) = (self.pose, self.tick_count);
        self.breeze
            .get_or_insert_with(Default::default)
            .tick(pose, tick_count);
    }

    /// `Frog.tick`'s client half, which runs before the rest of the tick and
    /// so sees where the last one left the frog.
    pub(crate) fn tick_frog(&mut self) {
        if !self.is_frog() {
            return;
        }
        let (in_water, moving, tick_count) =
            (self.living.in_water, self.walk.is_moving(), self.tick_count);
        self.frog
            .get_or_insert_with(Default::default)
            .tick(in_water, moving, tick_count);
    }

    /// An axolotl's animators and animation states, if it has ticked.
    pub fn axolotl(&self) -> Option<&crate::axolotl::AxolotlState> {
        self.axolotl.as_deref()
    }

    /// `Axolotl.getVariant`: the index of the variant (an id no variant has is the first).
    pub fn axolotl_variant(&self) -> usize {
        usize::try_from(self.living.axolotl_variant)
            .ok()
            .filter(|id| *id < 5)
            .unwrap_or(0)
    }

    /// `Axolotl.baseTick` on the client: the animators and animation states
    /// follow where the axolotl is and whether it walks. The rotation test of
    /// the reference never holds here: `LivingEntity.baseTick` has set the old
    /// turn to the new before it.
    pub(crate) fn tick_axolotl(&mut self) {
        if voxel_data::entity_type::name(self.type_id) != Some("minecraft:axolotl") {
            return;
        }
        let input = crate::axolotl::AxolotlInput {
            baby: self.is_baby(),
            playing_dead: self.living.axolotl_playing_dead,
            in_water: self.living.in_water,
            on_ground: self.on_ground,
            moving: self.walk.is_moving(),
            tick_count: self.tick_count,
        };
        self.axolotl
            .get_or_insert_with(|| Box::new(crate::axolotl::AxolotlState::new()))
            .tick(input);
    }

    /// `Turtle.hasEgg`.
    pub fn turtle_has_egg(&self) -> bool {
        self.living.turtle_has_egg
    }

    /// `Turtle.isLayingEgg`.
    pub fn turtle_laying_egg(&self) -> bool {
        self.living.turtle_laying_egg
    }

    /// What `DolphinRenderer.extractRenderState` calls moving: the motion the
    /// server gave it covers more than a ten-millionth of a block squared in a tick.
    pub fn is_moving_horizontally(&self) -> bool {
        let (x, _, z) = self.motion();
        x * x + z * z > 1.0E-7
    }

    /// Whether this is one of the fish `AbstractFish` makes.
    pub fn is_fish(&self) -> bool {
        matches!(
            voxel_data::entity_type::name(self.type_id),
            Some(
                "minecraft:cod"
                    | "minecraft:salmon"
                    | "minecraft:tropical_fish"
                    | "minecraft:pufferfish"
            )
        )
    }

    /// `Salmon.getVariant`: medium until the data says.
    pub fn salmon_size(&self) -> crate::fish::SalmonSize {
        crate::fish::SalmonSize::from_id(self.living.fish_variant.unwrap_or(1))
    }

    /// `TropicalFish.getPackedVariant`: a white kob until the data says.
    pub fn tropical_fish_variant(&self) -> i32 {
        self.living.fish_variant.unwrap_or(0)
    }

    /// `Pufferfish.getPuffState`.
    pub fn puff_state(&self) -> i32 {
        self.living.fish_variant.unwrap_or(0)
    }

    /// What `getDefaultDimensions` scales a fish's box by, from the size the
    /// type's table holds (that of a new one: a medium salmon, an unpuffed
    /// pufferfish, whose half is undone by doubling): a salmon by its size, a
    /// pufferfish by how puffed it is.
    pub(crate) fn fish_dimension_scale(&self) -> f32 {
        match voxel_data::entity_type::name(self.type_id) {
            Some("minecraft:salmon") => self.salmon_size().bounding_box_scale(),
            Some("minecraft:pufferfish") => {
                crate::fish::pufferfish_bounding_box_scale(self.puff_state()) * 2.0
            }
            _ => 1.0,
        }
    }

    /// `Entity.isInWater` for a mob that tracks it.
    pub fn is_in_water(&self) -> bool {
        self.living.in_water
    }

    /// Whether a position interpolation is still under way
    /// (`LivingEntity.isInterpolating`).
    pub(crate) fn is_interpolating(&self) -> bool {
        self.interpolation
            .as_ref()
            .is_some_and(|interpolation| interpolation.steps > 0)
    }

    /// What `LivingEntity.aiStep` does to the motion of a mob the client does not move:
    /// it decays by a fiftieth less while no position interpolation runs, and motion
    /// under three thousandths a tick is dropped.
    pub(crate) fn tick_mob_motion(&mut self, was_interpolating: bool) {
        let (mut x, mut y, mut z) = self.motion();
        if !was_interpolating {
            x *= 0.98;
            y *= 0.98;
            z *= 0.98;
        }
        if x.abs() < 0.003 {
            x = 0.0;
        }
        if z.abs() < 0.003 {
            z = 0.0;
        }
        if y.abs() < 0.003 {
            y = 0.0;
        }
        self.motion_x_bits = x.to_bits();
        self.motion_y_bits = y.to_bits();
        self.motion_z_bits = z.to_bits();
    }

    /// `Entity.baseTick`'s fluid pass, then `LivingEntity.aiStep`: a living entity
    /// eases to the position and turn its last packet gave and its head to the head
    /// turn, and a mob's motion fades; then what a species does of its own there.
    pub(crate) fn tick_ai_step(
        &mut self,
        living: Option<LivingKind>,
        in_water: Option<bool>,
        targets: EntityTargets,
    ) {
        if self.is_ender_dragon() {
            self.tick_ender_dragon(&targets);
            return;
        }
        if self.is_minecart() {
            self.tick_minecart();
            return;
        }
        self.tick_frog();
        self.tick_breeze();
        self.tick_wither_motion();
        if let Some(in_water) = in_water {
            self.living.in_water = in_water;
        }
        self.tick_axolotl();
        self.tick_guardian(targets.guardian);
        let was_interpolating = self.is_interpolating();
        if living.is_some() {
            self.tick_position_interpolation();
        }
        self.tick_head_rotation();
        if living == Some(LivingKind::Mob) {
            self.tick_mob_motion(was_interpolating);
            self.tick_squid();
            self.tick_cube_mob();
            self.tick_shulker();
        }
        self.tick_wither_heads(targets.wither);
    }

    /// `Squid.aiStep`, which turns the body (and with it the entity) in water.
    pub(crate) fn tick_squid(&mut self) {
        if !self.is_squid() {
            return;
        }
        let (in_water, motion, body) = (
            self.living.in_water,
            self.motion(),
            f32::from_bits(self.y_body_rot_bits),
        );
        let turned = self
            .squid
            .get_or_insert_with(|| Box::new(crate::squid::SquidState::new()))
            .ai_step(in_water, motion, body);
        if let Some(body) = turned {
            self.y_body_rot_bits = body.to_bits();
            self.y_rot_bits = body.to_bits();
        }
        // `GlowSquid.aiStep`: the dark ticks a hurt glow squid was given run out.
        if self.living.glow_dark_ticks > 0 {
            self.living.glow_dark_ticks -= 1;
        }
    }

    pub(crate) fn is_shulker(&self) -> bool {
        voxel_data::entity_type::name(self.type_id) == Some("minecraft:shulker")
    }

    /// `Shulker.getAttachFace`'s data value (down 0, up 1, north 2, south 3, west 4, east 5).
    pub fn shulker_attach_face(&self) -> i32 {
        self.living.shulker_face
    }

    /// `Shulker.getColor`'s dye id, none for the default.
    pub fn shulker_color(&self) -> Option<u8> {
        self.living.shulker_color
    }

    /// A shulker's lid between two ticks.
    pub fn shulker(&self) -> &crate::shulker::ShulkerState {
        &self.living.shulker
    }

    /// `Shulker.tick`'s lid.
    pub(crate) fn tick_shulker(&mut self) {
        if self.is_shulker() {
            let raw = self.living.shulker_raw_peek;
            self.living.shulker.tick(raw);
            self.living.shulker.tick_teleport();
        }
    }

    /// `Shulker.makeBoundingBox`: the body and the lid as far open as it is,
    /// on the side opposite the one it clings to.
    pub fn shulker_bounding_box(&self) -> Aabb {
        let physical = crate::shulker::physical_peek(self.living.shulker.current_peek());
        crate::shulker::progress_delta_aabb(
            self.shulker_scale(),
            self.living.shulker_face ^ 1,
            (-1.0, physical),
            self.position(),
        )
    }

    /// `Shulker.sanitizeScale` of `getScale`.
    fn shulker_scale(&self) -> f32 {
        self.scale().min(3.0)
    }

    /// `Shulker.getRenderPosition`: where the slide to its block draws it,
    /// relative to the block.
    pub fn shulker_render_offset(&self, partial_tick: f32) -> Option<[f64; 3]> {
        self.living.shulker.render_offset(
            block_of(self.position()),
            self.shulker_scale(),
            partial_tick,
        )
    }

    /// `Shulker.setPos`: unless it rides, it is held to the middle of its
    /// block (`floor(y + 0.5)` vertically); having come to another block after
    /// its first tick it closes its lid, and a shulker that is not riding
    /// slides there from the old one.
    fn shulker_set_pos(&mut self, position: (f64, f64, f64), old_block: [i32; 3]) {
        let riding = self.vehicle_id.is_some();
        let (x, y, z) = if riding {
            position
        } else {
            (
                position.0.floor() + 0.5,
                (position.1 + 0.5).floor(),
                position.2.floor() + 0.5,
            )
        };
        self.x_bits = x.to_bits();
        self.y_bits = y.to_bits();
        self.z_bits = z.to_bits();
        let block = block_of((x, y, z));
        if self.tick_count != 0 && block != old_block {
            self.living.shulker_raw_peek = 0;
            if !riding && self.living.shulker.start_teleport(old_block, block) {
                self.old_x_bits = x.to_bits();
                self.old_y_bits = y.to_bits();
                self.old_z_bits = z.to_bits();
            }
        }
    }

    /// `Shulker.startRiding` (client side): it stops sliding and clings to
    /// the floor of its vehicle.
    pub(crate) fn shulker_starts_riding(&mut self) {
        self.living.shulker.cancel_teleport();
        self.living.shulker_face = 0;
    }

    /// `Shulker.stopRiding` (client side): it slides from where it stands, and
    /// its body faces south again.
    pub(crate) fn shulker_stops_riding(&mut self) {
        let block = block_of(self.position());
        self.living.shulker.set_old_attach(block);
        self.y_body_rot_bits = 0.0_f32.to_bits();
        self.old_y_body_rot_bits = 0.0_f32.to_bits();
    }

    /// Whether this is a slime or a magma cube (`AbstractCubeMob`).
    pub fn is_cube_mob(&self) -> bool {
        crate::metadata::is_cube_mob_type(self.type_id)
    }

    /// `SulfurCube.getFuse`.
    pub fn sulfur_fuse(&self) -> i32 {
        self.living.cube.fuse()
    }

    /// `AbstractCubeMob.getSize`.
    pub fn cube_size(&self) -> i32 {
        self.living.cube_size.unwrap_or(1)
    }

    /// A slime's or a magma cube's squish and what it eases to.
    pub fn cube_mob(&self) -> &crate::cube_mob::CubeMobState {
        &self.living.cube
    }

    /// `AbstractCubeMob.onSyncedDataUpdated` for the size: each time the data
    /// says it (the same size too), the body and the entity turn to where the
    /// head looks.
    pub(crate) fn set_cube_size(&mut self, size: i32) {
        self.y_rot_bits = self.y_head_rot_bits;
        self.y_body_rot_bits = self.y_head_rot_bits;
        self.living.cube_size = Some(size);
    }

    /// `AbstractCubeMob.tick`'s squish (`decreaseSquish`: a magma cube's target
    /// fades slower).
    pub(crate) fn tick_cube_mob(&mut self) {
        if !self.is_cube_mob() {
            return;
        }
        let magma = voxel_data::entity_type::name(self.type_id) == Some("minecraft:magma_cube");
        let on_ground = self.on_ground;
        // `SulfurCube.tick` runs its fuse before `AbstractCubeMob.tick`.
        self.living.cube.tick_fuse();
        self.living
            .cube
            .tick(on_ground, if magma { 0.9 } else { 0.6 });
    }

    /// `GlowSquid.getDarkTicksRemaining`.
    pub fn glow_dark_ticks(&self) -> i32 {
        self.living.glow_dark_ticks
    }

    /// A copper golem's animation states, if it has ticked.
    pub fn copper_golem(&self) -> Option<&crate::copper_golem::CopperGolemAnimations> {
        self.copper_golem.as_deref()
    }

    /// `CopperGolem.getState`.
    pub fn copper_golem_action(&self) -> crate::copper_golem::GolemAction {
        self.living.copper_golem_action
    }

    /// `CopperGolem.getWeatherState`: unaffected, exposed, weathered, oxidized (0 to 3).
    pub fn copper_golem_weather(&self) -> u8 {
        self.living.copper_golem_weather
    }

    /// `Goat.hasLeftHorn` and `hasRightHorn`.
    pub fn goat_horns(&self) -> (bool, bool) {
        (!self.living.lost_left_horn, !self.living.lost_right_horn)
    }

    /// `Goat.getRammingXHeadRot`: how far the head is lowered, in radians.
    pub fn goat_ramming_x_head_rot(&self) -> f32 {
        let maximum = if self.baby { 52.5 } else { 30.0 };
        self.living.animal.goat_lower_head_tick as f32 / 20.0
            * maximum
            * (std::f64::consts::PI / 180.0) as f32
    }

    /// `Creeper.isPowered`.
    pub fn is_powered(&self) -> bool {
        self.living.powered
    }

    /// `Bee.getRollAmount`: how far the bee has rolled over, between two ticks.
    pub fn bee_roll_amount(&self, partial: f32) -> f32 {
        self.living.animal.bee_roll_at(partial)
    }

    /// `Bee.hasStung`.
    pub fn bee_has_stung(&self) -> bool {
        self.living.bee_flags & 4 != 0
    }

    /// `Bee.hasNectar`.
    pub fn bee_has_nectar(&self) -> bool {
        self.living.bee_flags & 8 != 0
    }

    /// The game time a neutral mob's anger ends (`getPersistentAngerEndTime`).
    pub fn anger_end(&self) -> i64 {
        self.living.anger_end
    }

    /// `NeutralMob.isAngry`: the anger ends later than the game time now.
    pub fn is_angry(&self, game_time: u64) -> bool {
        let end = self.living.anger_end;
        end > 0 && end - game_time as i64 > 0
    }

    /// `TamableAnimal.isInSittingPose`.
    pub fn is_sitting(&self) -> bool {
        self.living.tamable_flags & 1 != 0
    }

    /// `TamableAnimal.isTame`.
    pub fn is_tame(&self) -> bool {
        self.living.tamable_flags & 4 != 0
    }

    /// The dye of a wolf's collar (`Wolf.getCollarColor`).
    pub fn collar_dye(&self) -> i32 {
        // `DyeColor.RED` until the data says otherwise.
        self.living.collar.unwrap_or(14)
    }

    /// Whether this is a cat whose data says it lies down.
    pub(crate) fn is_lying_cat(&self) -> bool {
        self.living.cat_lying
            && voxel_data::entity_type::name(self.type_id) == Some("minecraft:cat")
    }

    /// `Cat.tick` after `super.tick`: the lie-down and relax amounts.
    pub(crate) fn tick_cat(&mut self, sleeper_near: bool) {
        if voxel_data::entity_type::name(self.type_id) == Some("minecraft:cat") {
            let (lying, relaxed) = (self.living.cat_lying, self.living.cat_relaxed);
            self.living.cat.tick(lying, relaxed, sleeper_near);
        }
    }

    /// A panda's data and own client state (its counters and how far it sits,
    /// lies on its back and rolls); the starting state until data comes.
    pub fn panda(&self) -> crate::panda::PandaState {
        self.panda.as_deref().cloned().unwrap_or_default()
    }

    /// Fixes the source a panda's chance to eat is drawn from (see
    /// [`crate::panda::PandaState::fix_seed`]).
    pub fn fix_panda_seed(&mut self, seed: i64) {
        self.panda
            .get_or_insert_with(Default::default)
            .fix_seed(seed);
    }

    /// A ravager's counters (the starting state until an event comes).
    pub fn ravager(&self) -> crate::ravager::RavagerState {
        self.ravager.as_deref().cloned().unwrap_or_default()
    }

    /// What a cat or a panda does of its own after `super.tick`, given what
    /// the world around it holds (a sleeper near a cat, thunder).
    pub(crate) fn tick_species(&mut self, (sleeper_near_cat, thundering): (bool, bool)) {
        self.tick_cat(sleeper_near_cat);
        self.tick_panda(thundering);
        self.tick_bat();
        self.tick_parrot();
        self.tick_allay();
        self.tick_warden();
        self.tick_creaking();
        self.tick_armadillo();
        if self.is_end_crystal() {
            self.end_crystal_mut().tick();
        }
        if self.is_abstract_arrow() {
            self.arrow_mut().tick();
        }
        if self.is_evoker_fangs() {
            self.evoker_fangs_mut().tick();
        }
        if self.is_primed_tnt() {
            self.primed_tnt_mut().tick();
        }
        if self.is_lightning_bolt() {
            let seed = i64::from(self.id);
            self.lightning_done = self
                .lightning
                .get_or_insert_with(|| crate::lightning::LightningState::new(seed))
                .tick();
        }
    }

    /// `Panda.tick` after `super.tick`; a sitting panda levels its pitch.
    pub(crate) fn tick_panda(&mut self, thundering: bool) {
        if voxel_data::entity_type::name(self.type_id) != Some("minecraft:panda") {
            return;
        }
        let holds_item = self.equipment[EquipmentSlot::MainHand as usize].is_some();
        let around = crate::panda::PandaSurroundings {
            thundering,
            holds_item,
        };
        let panda = self.panda.get_or_insert_with(Default::default);
        if panda.tick(around) {
            self.x_rot_bits = 0.0_f32.to_bits();
        }
    }

    /// A fox's flags, variant and own client state (its head roll and crouch).
    pub fn fox(&self) -> &crate::fox::FoxState {
        &self.living.fox
    }

    /// `LivingEntityRenderer.isShaking` with the overrides of the hoglin
    /// (`Hoglin.isConverting`) and the strider (`Strider.isSuffocating`):
    /// frozen through, converting to a zoglin, or out of lava and cold.
    pub fn is_shaking(&self, piglins_zombify: bool) -> bool {
        self.is_fully_frozen()
            || match voxel_data::entity_type::name(self.type_id) {
                Some("minecraft:hoglin") => {
                    !self.living.hoglin_immune && !self.is_no_ai() && piglins_zombify
                }
                Some("minecraft:strider") => self.living.strider_suffocating,
                _ => false,
            }
    }

    /// `Strider.isSuffocating`.
    pub fn is_suffocating(&self) -> bool {
        self.living.strider_suffocating
    }

    /// `Entity.isVehicle`: something rides it.
    pub fn is_vehicle(&self) -> bool {
        !self.passenger_ids.is_empty()
    }

    /// A polar bear's own client state (how far it has stood up).
    pub fn polar_bear(&self) -> &crate::polar_bear::PolarBearState {
        &self.living.polar_bear
    }

    /// A cat's own client state (how far it lies and relaxes).
    pub fn cat(&self) -> &crate::cat::CatState {
        &self.living.cat
    }

    /// `Entity.isCrouching`: the crouching pose.
    pub fn is_crouching(&self) -> bool {
        self.pose == ClientEntityPose::Crouching
    }

    /// A wolf's own client state (its interest and its shake).
    pub fn wolf(&self) -> &crate::wolf::WolfState {
        &self.living.wolf
    }

    /// `Wolf.getTailAngle`: raised while angry, by health while tame.
    pub fn wolf_tail_angle(&self, game_time: u64) -> f32 {
        let pi = std::f64::consts::PI as f32;
        if self.is_angry(game_time) {
            1.539_380_4
        } else if self.is_tame() {
            let max_health = self.max_health().unwrap_or(0.0);
            let health = self.health().unwrap_or(max_health);
            let damage_ratio = (max_health - health) / max_health;
            (0.55 - damage_ratio * 0.4) * pi
        } else {
            (std::f64::consts::PI / 5.0) as f32
        }
    }

    /// `Rabbit.getVariant`'s id: zero is brown (an id the variants lack is brown).
    pub fn rabbit_variant(&self) -> i32 {
        self.living.rabbit_type
    }

    /// `MushroomCow.getVariant`'s id: zero is red, one brown (an id past the
    /// last takes the last).
    pub fn mooshroom_variant(&self) -> i32 {
        self.living.mooshroom_type.clamp(0, 1)
    }

    /// `SnowGolem.hasPumpkin`.
    pub fn has_pumpkin(&self) -> bool {
        !self.living.bare_head
    }

    /// `EnderMan.isCreepy`.
    pub fn is_creepy(&self) -> bool {
        self.living.creepy
    }

    /// `Hoglin.getAttackAnimationRemainingTicks` (a zoglin's too).
    pub fn hoglin_attack_ticks(&self) -> i32 {
        self.living.animal.hoglin_attack_ticks
    }

    /// An iron golem's `getAttackAnimationTick` and `getOfferFlowerTick`.
    pub fn iron_golem_ticks(&self) -> (i32, i32) {
        let animal = &self.living.animal;
        (animal.golem_attack_tick, animal.golem_offer_flower_tick)
    }

    /// `LivingEntity.getAttributeValue`: the attribute's value, from what the
    /// type starts with and what the server has changed (`AttributeInstance.getValue`:
    /// the base, the modifiers in their order, kept in the attribute's range).
    /// Nothing for an attribute the type does not have.
    pub fn attribute_value(&self, attribute: &str) -> Option<f64> {
        let (id, range) = voxel_data::entity_type::attribute(attribute)?;
        let default = voxel_data::entity_type::default_attribute(
            voxel_data::entity_type::name(self.type_id)?,
            attribute,
        )?;
        let (low, high) = range.range.unwrap_or((f64::NEG_INFINITY, f64::INFINITY));
        let changed = self
            .attributes
            .iter()
            .find(|state| usize::try_from(state.attribute_id) == Ok(id));
        Some(changed.map_or_else(
            || ClientAttributeState::new(0, default, Vec::new()).effective_value(low, high),
            |state| state.effective_value(low, high),
        ))
    }

    /// `LivingEntity.getMaxHealth`.
    pub fn max_health(&self) -> Option<f32> {
        self.attribute_value("minecraft:max_health")
            .map(|value| value as f32)
    }

    /// `LivingEntity.getScale`: one for an entity that has no scale attribute.
    pub fn scale(&self) -> f32 {
        self.attribute_value("minecraft:scale")
            .map_or(1.0, |value| value as f32)
    }

    /// `Raider.isCelebrating`.
    pub fn is_celebrating(&self) -> bool {
        self.living.celebrating
    }

    /// `SpellcasterIllager.isCastingSpell`.
    pub fn is_casting_spell(&self) -> bool {
        self.living.casting_spell
    }

    /// `Sheep.getColor`: the dye's id (`DyeColor.getId`).
    pub fn sheep_dye_id(&self) -> u8 {
        self.living.animal.wool & 15
    }

    /// `Sheep.isSheared`.
    pub fn is_sheared(&self) -> bool {
        self.living.animal.wool & 16 != 0
    }

    /// `Sheep.getHeadEatPositionScale` and `getHeadEatAngleScale` at a point
    /// between two ticks, with the pitch the sheep is drawn at, in degrees.
    pub fn sheep_head_eat_scales(&self, partial: f32, pitch: f32) -> (f32, f32) {
        let tick = self.living.animal.eat_tick;
        (
            crate::animal::head_eat_position_scale(tick, partial),
            crate::animal::head_eat_angle_scale(tick, partial, pitch),
        )
    }

    /// `Creeper.getSwelling`: how far the fuse has burned, between two ticks.
    pub fn creeper_swelling(&self, partial: f32) -> f32 {
        self.living.animal.creeper_swelling(partial)
    }

    /// A chicken's `flap` and `flapSpeed` between two ticks.
    pub fn chicken_flap(&self, partial: f32) -> (f32, f32) {
        let animal = &self.living.animal;
        (animal.flap_at(partial), animal.flap_speed_at(partial))
    }

    /// An armor stand's flags and poses.
    pub fn armor_stand(&self) -> &crate::armor_stand::ArmorStandState {
        &self.living.armor_stand
    }

    /// The swing of an arm (`LivingEntity.swing`).
    pub fn swing_state(&self) -> &crate::living::SwingState {
        &self.living.swing
    }

    /// What a living entity's swing does each tick: the last tick's attack
    /// animation is kept, and the swing runs on for a player and a monster
    /// (`Player.aiStep`, `Monster.aiStep`).
    pub(crate) fn tick_swing(&mut self, kind: LivingKind) {
        self.living.swing.begin_tick();
        let monster = voxel_data::entity_type::pick(self.type_id).is_some_and(|pick| pick.monster);
        if kind == LivingKind::Player || monster {
            let durations = self.swing_durations();
            self.living.swing.update(durations);
        }
    }

    /// `ItemStack.getSwingAnimation().duration()` of what each hand holds, the main hand's first.
    pub(crate) fn swing_durations(&self) -> [i32; 2] {
        [EquipmentSlot::MainHand, EquipmentSlot::OffHand].map(|slot| {
            self.equipment(slot)
                .and_then(|stack| crate::block_rules::item(stack.item_registry_id()))
                .and_then(|item| voxel_data::spear::spear_of(item.name))
                .map_or(crate::living::DEFAULT_SWING_DURATION, |spear| {
                    spear.swing_ticks
                })
        })
    }

    /// What a living entity does each tick besides moving: hurt and death,
    /// its pose, its swing, and what its kind does of its own.
    pub(crate) fn tick_living(&mut self, kind: LivingKind, tick_death: bool, game_time: u64) {
        self.living.using_item_ticks = if self.is_using_item() {
            self.living.using_item_ticks.saturating_add(1)
        } else {
            0
        };
        let (mx, my, mz) = self.motion();
        self.living.previous_motion = [mx.to_bits(), my.to_bits(), mz.to_bits()];
        self.living.fall_fly_ticks = if self.is_fall_flying() {
            self.living.fall_fly_ticks.saturating_add(1)
        } else {
            0
        };
        self.living.elytra.tick(crate::elytra::ElytraInput {
            fall_flying: self.is_fall_flying(),
            motion: self.motion(),
            crouching: self.is_crouching(),
        });
        self.tick_hurt_and_death(tick_death);
        self.tick_living_pose(kind);
        self.tick_swing(kind);
        self.tick_animal(game_time);
    }

    /// What a sheep or a chicken does of its own each tick (`aiStep`).
    pub(crate) fn tick_animal(&mut self, game_time: u64) {
        match voxel_data::entity_type::name(self.type_id) {
            Some("minecraft:sheep") => self.living.animal.tick_sheep(),
            Some("minecraft:chicken") => self.living.animal.tick_chicken(self.on_ground),
            Some("minecraft:creeper") => self.living.animal.tick_creeper(self.alive),
            Some("minecraft:iron_golem") => self.living.animal.tick_iron_golem(),
            Some("minecraft:goat") => self.living.animal.tick_goat(),
            Some("minecraft:hoglin" | "minecraft:zoglin") => self.living.animal.tick_hoglin(),
            Some("minecraft:ravager") => {
                let alive = self.alive;
                self.ravager
                    .get_or_insert_with(Default::default)
                    .tick(alive);
            }
            Some("minecraft:bee") => {
                let rolling = self.living.bee_flags & 2 != 0;
                self.living.animal.tick_bee(rolling);
            }
            Some(
                "minecraft:horse"
                | "minecraft:donkey"
                | "minecraft:mule"
                | "minecraft:skeleton_horse"
                | "minecraft:zombie_horse"
                | "minecraft:llama"
                | "minecraft:trader_llama"
                | "minecraft:camel"
                | "minecraft:camel_husk",
            ) => {
                let (alive, camel) = (
                    self.alive,
                    matches!(
                        voxel_data::entity_type::name(self.type_id),
                        Some("minecraft:camel" | "minecraft:camel_husk")
                    ),
                );
                let (tick_count, on_ground, passenger) =
                    (self.tick_count, self.on_ground, self.vehicle_id.is_some());
                let horse = self
                    .horse
                    .get_or_insert_with(|| Box::new(crate::horse::HorseState::new()));
                horse.tick(alive);
                if camel {
                    horse.tick_camel((tick_count, game_time as i64), (on_ground, passenger));
                }
            }
            Some("minecraft:rabbit") => {
                let (tick_count, no_ai) = (self.tick_count, self.is_no_ai());
                self.rabbit
                    .get_or_insert_with(|| Box::new(crate::rabbit::RabbitState::new()))
                    .tick(tick_count, no_ai);
            }
            Some("minecraft:copper_golem") => {
                let (action, tick_count, flags) = (
                    self.living.copper_golem_action,
                    self.tick_count,
                    (self.is_no_ai(), self.alive),
                );
                self.copper_golem
                    .get_or_insert_with(|| {
                        Box::new(crate::copper_golem::CopperGolemAnimations::new())
                    })
                    .tick(action, tick_count, flags);
            }
            Some("minecraft:polar_bear") => self.living.polar_bear.tick(),
            Some("minecraft:fox") => self.living.fox.tick(),
            Some("minecraft:wolf") => {
                let (alive, interested) = (self.alive, self.living.wolf_interested);
                self.living.wolf.tick(alive, interested);
            }
            _ => {}
        }
    }

    /// `getItemHeldByArm`.
    pub fn item_held_by_arm(&self, left: bool) -> Option<&ClientItemStackState> {
        self.equipment(if left == self.main_arm_is_left() {
            EquipmentSlot::MainHand
        } else {
            EquipmentSlot::OffHand
        })
    }

    /// `ItemEntity.bobOffs`.
    pub fn bob_offset(&self) -> f32 {
        f32::from_bits(self.bob_offset_bits)
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
