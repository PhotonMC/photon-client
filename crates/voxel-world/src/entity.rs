//! `ClientWorld` entity layer split out of the former single-file module.

use super::*;

use voxel_core::aabb::Aabb;
use voxel_core::java_float::{DVec3, JavaF32, JavaF64};
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
    /// `WitherSkull.isDangerous`: the blue skull, which flies slower.
    pub(crate) wither_skull_dangerous: bool,
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
    pub(crate) width: JavaF32,
    pub(crate) height: JavaF32,
    pub(crate) eye_height: JavaF32,
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
            width: JavaF32::new(width),
            height: JavaF32::new(height),
            eye_height: JavaF32::new(eye_height),
            fixed,
        }
    }

    pub fn width(self) -> f32 {
        self.width.get()
    }

    pub fn height(self) -> f32 {
        self.height.get()
    }

    pub fn eye_height(self) -> f32 {
        self.eye_height.get()
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

pub(crate) fn advance_head_rotation(current: &mut JavaF32, target: JavaF32, steps: &mut u8) {
    if *steps == 0 {
        return;
    }
    let step_count = f64::from(*steps);
    let from = f64::from(current.get());
    let to = f64::from(target.get());
    *current = JavaF32::new(mth_rot_lerp_f64(1.0 / step_count, from, to) as f32);
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

/// Which display a type is, if it is one (`Display`).
pub(crate) fn display_kind_of(entity_type_id: i32) -> Option<crate::display::DisplayKind> {
    use crate::display::DisplayKind;
    match voxel_data::entity_type::name(entity_type_id)? {
        "minecraft:block_display" => Some(DisplayKind::Block),
        "minecraft:item_display" => Some(DisplayKind::Item),
        "minecraft:text_display" => Some(DisplayKind::Text),
        _ => None,
    }
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

mod accessors;
mod animation;
mod kinds;
mod state;
