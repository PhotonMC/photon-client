use super::*;

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

/// State produced by Java Entity.refreshDimensions before optional server
/// position correction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientEntityDimensionsRefresh {
    pub dimensions: ClientEntityDimensions,
    pub bounding_box: Aabb,
    pub should_fudge_position: bool,
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

pub(crate) const VANILLA_BOAT_PASSENGER_MAX_ROTATION: f32 = 105.0_f32;

/// How a living entity's pose is ticked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LivingKind {
    Player,
    /// A `Mob`: its body follows the head by `BodyRotationControl`.
    Mob,
    /// A living entity that is neither, an armor stand.
    Other,
}
