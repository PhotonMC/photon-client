//! The local player's whole client tick.
//!
//! This is `LocalPlayer.tick` and everything it runs, in the reference's
//! order: the level's old-position step, `Player.tick`, the living base tick
//! with its fluid update, `LocalPlayer.aiStep` down through `travel` and
//! `Entity.move`, the block effects, the pose update, and the packets the
//! tick sends. The leaves it composes are pinned on their own elsewhere;
//! this module pins the composition.
//!
//! A branch the port does not carry yet is reported as [`Unsupported`]
//! rather than approximated, so a scene that reaches one fails loudly.
//!
//! The reference writes `!(a > b)` where a NaN must fall through the other
//! way than `a <= b` would send it, so those negations are kept as written.
#![expect(
    clippy::neg_cmp_op_on_partial_ord,
    reason = "the reference's negated comparisons, which a NaN also passes"
)]

use crate::input::{Input, KeyboardInput, LocalPlayerInputModifiers, modify_local_player_input};
use crate::java_math;
use voxel_core::aabb::Aabb;
use voxel_core::binary::BlockPos;
use voxel_core::direction::{Axis, Direction};
use voxel_core::math::{
    MTH_DEG_TO_RAD, Vec3, equal_f64, floor_f64, mth_cos_f64, mth_inv_sqrt_f32, mth_sin_f64,
};
use voxel_core::shape::{BooleanOp, Shapes, VoxelShape};

use super::ClientWorld;
use super::collision::{
    collide_entity_movement, is_minor_horizontal_collision, select_step_up_movement,
};
use super::entity::{ClientEntityDimensions, ClientEntityPose};
use super::player::{ClientPlayerAbilities, ClientPlayerGameMode};
use voxel_data::block::classes::BlockClass;
use voxel_data::block::definition::Fluid;
use voxel_data::block::registry;

/// A branch of the reference tick this port does not carry yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unsupported(pub String);

impl std::fmt::Display for Unsupported {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "unported local player branch: {}", self.0)
    }
}

impl std::error::Error for Unsupported {}

/// `ServerboundPlayerCommandPacket.Action`, by ordinal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerCommandAction {
    StartSprinting = 1,
    StopSprinting = 2,
    StartRidingJump = 3,
}

/// A packet the tick asks the connection to send.
#[derive(Debug, Clone, Copy)]
pub enum ServerboundMovement {
    PlayerInput(Input),
    PlayerCommand {
        entity_id: i32,
        action: PlayerCommandAction,
        /// The action's datum: a riding jump's charge, in hundredths.
        data: i32,
    },
    PlayerAbilities {
        flying: bool,
    },
    MovePlayerPos {
        position: Vec3,
        on_ground: bool,
        horizontal_collision: bool,
    },
    MovePlayerPosRot {
        position: Vec3,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
    },
    MovePlayerRot {
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
        horizontal_collision: bool,
    },
    MovePlayerStatusOnly {
        on_ground: bool,
        horizontal_collision: bool,
    },
    /// `ServerboundMoveVehiclePacket`: where the vehicle the player is the authority of is.
    MoveVehicle {
        position: Vec3,
        y_rot: f32,
        x_rot: f32,
        on_ground: bool,
    },
    /// `ServerboundPaddleBoatPacket`: which paddles of the boat it rides row.
    PaddleBoat {
        left: bool,
        right: bool,
    },
}

impl ServerboundMovement {
    /// The packet type's resource name.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::PlayerInput(_) => "minecraft:player_input",
            Self::PlayerCommand { .. } => "minecraft:player_command",
            Self::PlayerAbilities { .. } => "minecraft:player_abilities",
            Self::MovePlayerPos { .. } => "minecraft:move_player_pos",
            Self::MovePlayerPosRot { .. } => "minecraft:move_player_pos_rot",
            Self::MovePlayerRot { .. } => "minecraft:move_player_rot",
            Self::MovePlayerStatusOnly { .. } => "minecraft:move_player_status_only",
            Self::MoveVehicle { .. } => "minecraft:move_vehicle",
            Self::PaddleBoat { .. } => "minecraft:paddle_boat",
        }
    }

    /// The body the packet's codec writes, without the packet id.
    pub fn encode_body(&self) -> Vec<u8> {
        fn move_flags(on_ground: bool, horizontal_collision: bool) -> u8 {
            u8::from(on_ground) | (u8::from(horizontal_collision) << 1)
        }
        fn var_int(output: &mut Vec<u8>, value: i32) {
            let mut value = value as u32;
            loop {
                if value & !0x7f == 0 {
                    output.push(value as u8);
                    return;
                }
                output.push((value as u8 & 0x7f) | 0x80);
                value >>= 7;
            }
        }
        let mut output = Vec::new();
        match *self {
            Self::PlayerInput(input) => output.push(
                u8::from(input.forward)
                    | u8::from(input.backward) << 1
                    | u8::from(input.left) << 2
                    | u8::from(input.right) << 3
                    | u8::from(input.jump) << 4
                    | u8::from(input.shift) << 5
                    | u8::from(input.sprint) << 6,
            ),
            Self::PlayerCommand {
                entity_id,
                action,
                data,
            } => {
                var_int(&mut output, entity_id);
                var_int(&mut output, action as i32);
                var_int(&mut output, data);
            }
            Self::PlayerAbilities { flying } => output.push(if flying { 2 } else { 0 }),
            Self::MovePlayerPos {
                position,
                on_ground,
                horizontal_collision,
            } => {
                output.extend_from_slice(&position.x.to_be_bytes());
                output.extend_from_slice(&position.y.to_be_bytes());
                output.extend_from_slice(&position.z.to_be_bytes());
                output.push(move_flags(on_ground, horizontal_collision));
            }
            Self::MovePlayerPosRot {
                position,
                y_rot,
                x_rot,
                on_ground,
                horizontal_collision,
            } => {
                output.extend_from_slice(&position.x.to_be_bytes());
                output.extend_from_slice(&position.y.to_be_bytes());
                output.extend_from_slice(&position.z.to_be_bytes());
                output.extend_from_slice(&y_rot.to_be_bytes());
                output.extend_from_slice(&x_rot.to_be_bytes());
                output.push(move_flags(on_ground, horizontal_collision));
            }
            Self::MovePlayerRot {
                y_rot,
                x_rot,
                on_ground,
                horizontal_collision,
            } => {
                output.extend_from_slice(&y_rot.to_be_bytes());
                output.extend_from_slice(&x_rot.to_be_bytes());
                output.push(move_flags(on_ground, horizontal_collision));
            }
            Self::MovePlayerStatusOnly {
                on_ground,
                horizontal_collision,
            } => {
                output.push(move_flags(on_ground, horizontal_collision));
            }
            Self::MoveVehicle {
                position,
                y_rot,
                x_rot,
                on_ground,
            } => {
                output.extend_from_slice(&position.x.to_be_bytes());
                output.extend_from_slice(&position.y.to_be_bytes());
                output.extend_from_slice(&position.z.to_be_bytes());
                output.extend_from_slice(&y_rot.to_be_bytes());
                output.extend_from_slice(&x_rot.to_be_bytes());
                output.push(u8::from(on_ground));
            }
            Self::PaddleBoat { left, right } => {
                output.push(u8::from(left));
                output.push(u8::from(right));
            }
        }
        output
    }
}

// Player attribute defaults, as the reference registers them.
const MOVEMENT_SPEED_BASE: f32 = 0.1;
const SPRINTING_MODIFIER: f32 = 0.3;
const JUMP_STRENGTH: f32 = 0.42;
const STEP_HEIGHT: f64 = 0.6;
const GRAVITY: f64 = 0.08;
const SNEAKING_SPEED: f64 = 0.3;
const SPRINT_WINDOW: i32 = 7;
use crate::fluid::{FluidTracker, LAVA_FLOW_SCALE, WATER_FLOW_SCALE};

/// A stretch of movement recorded for the block-effects pass.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Movement {
    pub(crate) from: Vec3,
    pub(crate) to: Vec3,
    /// The movement asked for, when the move was split per axis.
    pub(crate) original: Option<Vec3>,
}

/// What the vehicle the player rides tells its tick: the top of the vehicle's box and whether the
/// vehicle is under water (`modifyPassengerFluidInteractionBox`), and whether it can be sprinted
/// in (`Entity.canSprint`).
#[derive(Debug, Clone, Copy)]
struct Riding {
    vehicle_max_y: f64,
    vehicle_under_water: bool,
    /// Whether the vehicle changes the box the player's fluids are found in (a boat does).
    modifies_fluid_box: bool,
    can_sprint: bool,
    /// Whether the jump key charges a jump of the vehicle (`PlayerRideableJumping.canJump`).
    can_jump: bool,
}

/// What one tick of the player sitting in a boat gives the session: the packets to send, and the
/// keys to hand the boat (`AbstractBoat.setInput`).
#[derive(Debug, Clone)]
pub struct RideTick {
    pub sent: Vec<ServerboundMovement>,
    pub boat_input: crate::entity::BoatInput,
    /// The jump the rider let go of, as `PlayerRideableJumping.onPlayerJump` is told it
    /// (the charge, in hundredths).
    pub jump: Option<i32>,
}

/// The tags and classes movement tests of a block.
mod sounds;
pub use sounds::{LocalSound, LocalSoundSource, SoundQueue};

pub(crate) mod flag {
    pub const CLIMBABLE: u16 = 1 << 0;
    pub const FENCE: u16 = 1 << 1;
    pub const WALL: u16 = 1 << 2;
    pub const CAN_GLIDE_THROUGH: u16 = 1 << 3;
    pub const SUPPRESSES_BOUNCE: u16 = 1 << 4;
    pub const FENCE_GATE: u16 = 1 << 5;
    pub const TRAPDOOR: u16 = 1 << 6;
    pub const LADDER: u16 = 1 << 7;
}

/// What a block state holds for movement, looked up once.
#[derive(Clone, Copy)]
pub(crate) struct StateInfo {
    pub(crate) name: &'static str,
    pub(crate) friction: f32,
    pub(crate) speed_factor: f32,
    pub(crate) jump_factor: f32,
    pub(crate) bounce: f32,
    pub(crate) flags: u16,
    pub(crate) fluid: Fluid,
    own_height: f32,
    falling: bool,
}

impl StateInfo {
    /// A state against the server's tags; a number outside the registry
    /// reads as air.
    fn of(state_id: Option<i32>, tags: &crate::block::tags::BlockTags) -> Self {
        let state = crate::block::state_or_air(state_id.map_or(0, i64::from));
        let block = state.block();
        let index = usize::from(state.block);
        let tagged = [
            ("minecraft:climbable", flag::CLIMBABLE),
            ("minecraft:fences", flag::FENCE),
            ("minecraft:walls", flag::WALL),
            ("minecraft:can_glide_through", flag::CAN_GLIDE_THROUGH),
            ("minecraft:suppresses_bounce", flag::SUPPRESSES_BOUNCE),
        ];
        let classes = [
            (BlockClass::FenceGateBlock, flag::FENCE_GATE),
            (BlockClass::TrapDoorBlock, flag::TRAPDOOR),
            (BlockClass::LadderBlock, flag::LADDER),
        ];
        let flags = tagged
            .iter()
            .filter(|(tag, _)| tags.contains(index, tag))
            .map(|(_, bit)| bit)
            .chain(
                classes
                    .iter()
                    .filter(|(class, _)| block.class.is(*class))
                    .map(|(_, bit)| bit),
            )
            .fold(0, |flags, bit| flags | bit);
        Self {
            name: block.name,
            friction: block.friction,
            speed_factor: block.speed_factor,
            jump_factor: block.jump_factor,
            bounce: block.bounce_restitution,
            flags,
            fluid: state.fluid.fluid,
            own_height: state.fluid.own_height(),
            falling: state.fluid.falling,
        }
    }

    fn is_air(&self) -> bool {
        matches!(
            self.name,
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        )
    }

    pub(crate) fn is_water(&self) -> bool {
        matches!(self.fluid, Fluid::Water | Fluid::FlowingWater)
    }

    pub(crate) fn is_lava(&self) -> bool {
        matches!(self.fluid, Fluid::Lava | Fluid::FlowingLava)
    }
}

/// The level as the tick reads it.
#[derive(Clone, Copy)]
pub(crate) struct Level<'a> {
    pub(crate) world: &'a ClientWorld,
    /// The height of the feet of an entity that can stand on lava (`EntityCollisionContext` of a
    /// strider): the lava's source blocks below it then hold it up.
    pub(crate) lava_stand: Option<f64>,
}

impl<'a> Level<'a> {
    pub(crate) fn new(world: &'a ClientWorld) -> Self {
        Self {
            world,
            lava_stand: None,
        }
    }

    /// The level as seen by an entity that can stand on lava, with its feet at `bottom`.
    pub(crate) fn standing_on_lava_at(self, bottom: f64) -> Self {
        Self {
            lava_stand: Some(bottom),
            ..self
        }
    }
}

/// The client's own player, ticked as the reference ticks it.
#[derive(Debug, Clone)]
pub struct LocalPlayer {
    entity_id: i32,
    position: Vec3,
    old_position: Vec3,
    delta: Vec3,
    y_rot: f32,
    x_rot: f32,
    bounding_box: Aabb,
    pose: ClientEntityPose,
    dimensions: ClientEntityDimensions,
    on_ground: bool,
    horizontal_collision: bool,
    vertical_collision: bool,
    vertical_collision_below: bool,
    minor_horizontal_collision: bool,
    main_supporting_block: Option<BlockPos>,
    on_ground_no_blocks: bool,
    fall_distance: f64,
    stuck_speed_multiplier: Vec3,
    /// `ClientAvatarState`: the distance walked and the view bob, now and a
    /// tick ago, which only the camera reads.
    walk_dist: f32,
    walk_dist_old: f32,
    /// The body's and the head's facing, now and a tick ago, which the
    /// player model is turned by.
    y_body_rot: f32,
    y_body_rot_old: f32,
    y_head_rot: f32,
    y_head_rot_old: f32,
    /// `WalkAnimationState`: how fast and how far the limbs have swung.
    walk: crate::living::WalkAnimation,
    /// What the vehicle the player sits in makes of it, while it rides.
    riding: Option<Riding>,
    /// `LocalPlayer.jumpRidingTicks` and `jumpRidingScale`: how long the jump key has charged a
    /// mount's jump, and how hard that is; the jump let go of this tick, for the mount.
    jump_riding_ticks: i32,
    jump_riding_scale: f32,
    ride_jump: Option<i32>,
    /// The first-person hand's lagging view: it chases the rotation by
    /// half the gap each tick.
    x_bob: f32,
    y_bob: f32,
    x_bob_old: f32,
    y_bob_old: f32,
    /// `LivingEntity`'s arm swing: whether one is under way, how many
    /// ticks in, and how far through, now and a tick ago.
    swinging: bool,
    swing_time: i32,
    /// `swingingArm`: the hand that swung last (the main hand until one swings).
    swinging_off_hand: bool,
    attack_anim: f32,
    attack_anim_old: f32,
    /// `Player.attackStrengthTicker` and `itemSwapTicker`: ticks since the
    /// last attack or change of held item.
    attack_strength_ticker: i32,
    /// The main hand item's registry id (air when empty), and the one the
    /// last tick saw.
    main_hand_item: i32,
    last_main_hand_item: i32,
    /// What the hands hold, the effects and the use of an item, as the world last heard them.
    hands: HandState,
    /// How many ticks the item in use has been used (`getTicksUsingItem`).
    using_item_ticks: i32,
    item_swap_ticker: i32,
    /// `LivingEntity`'s health as the client last set it, and the hurt
    /// state `LocalPlayer.hurtTo` keeps: ticks of damage immunity, of the
    /// hurt flash, and whether the next health update may flash at all.
    health: f32,
    flash_on_set_health: bool,
    invulnerable_time: i32,
    hurt_time: i32,
    /// `hurtDuration`: what the flinch started from.
    hurt_duration: i32,
    /// `Player.hurtDir`: where the last blow came from, in degrees.
    hurt_dir: f32,
    /// `deathTime`: the ticks since the health ran out.
    death_time: i32,
    /// How far into the swimming pose the model has turned.
    swim_amount: f32,
    swim_amount_old: f32,
    bob: f32,
    bob_old: f32,
    first_tick: bool,
    tick_count: i32,
    /// `Entity.random`: the pitch of the player's sounds is drawn from it. The reference seeds
    /// it from nowhere in particular; this one from the entity's number.
    random: voxel_core::random::RandomSource,
    /// `BubbleColumnAmbientSoundHandler`: whether the last tick was in a bubble column, and
    /// whether it is the first.
    was_in_bubble_column: bool,
    ambient_first_tick: bool,
    /// `LivingEntity.elytraAnimationState`: the local player never glides in this port.
    elytra: crate::elytra::ElytraState,
    /// `Entity.moveDist` and `nextStep`: how far the player has walked, and where the next step sounds.
    move_dist: f32,
    next_step: f32,
    pending_sounds: Vec<LocalSound>,
    was_touching_water: bool,
    was_eye_in_water: bool,
    water: FluidTracker,
    lava: FluidTracker,
    sprinting: bool,
    swimming: bool,
    crouching: bool,
    input: KeyboardInput,
    xxa: f32,
    zza: f32,
    jumping: bool,
    no_jump_delay: i32,
    jump_trigger_time: i32,
    sprint_trigger_time: i32,
    auto_jump_time: i32,
    abilities: ClientPlayerAbilities,
    game_mode: ClientPlayerGameMode,
    no_physics: bool,
    auto_jump_option: bool,
    auto_jump_enabled: bool,
    movements_this_tick: Vec<Movement>,
    x_last: f64,
    y_last: f64,
    z_last: f64,
    y_rot_last: f32,
    x_rot_last: f32,
    last_on_ground: bool,
    last_horizontal_collision: bool,
    was_sprinting: bool,
    position_reminder: i32,
    last_sent_input: Input,
    sent: Vec<ServerboundMovement>,
}

/// A living body as the blocks it is inside act on it.
pub(crate) struct LivingEffects<'a> {
    pub(crate) delta: &'a mut Vec3,
    pub(crate) fall_distance: &'a mut f64,
    pub(crate) stuck: &'a mut Vec3,
    pub(crate) position: Vec3,
    pub(crate) on_ground: bool,
    pub(crate) width: f32,
    /// The block the body's feet are in.
    pub(crate) block_pos: BlockPos,
}

/// A block the walk reached, and what its visit needs to act on.
pub(crate) struct InsideVisit {
    pub(crate) pos: BlockPos,
    pub(crate) state: Option<i32>,
    pub(crate) info: StateInfo,
    /// `isPrecise`: the entity's box really overlaps the block, or it moved a long way.
    pub(crate) precise: bool,
}

/// A body whose movement is walked for the blocks it passes through (`Entity.checkInsideBlocks`).
pub(crate) trait InsideBody {
    /// The body's box with its feet at a position.
    fn box_at(&self, position: Vec3) -> Aabb;
    /// The collision shape powder snow has for this body, which depends on how far it has fallen.
    fn powder_snow_shape(&self) -> VoxelShape;
    /// What the block does to the body.
    fn inside_block_effect(&mut self, level: &Level<'_>, visit: &InsideVisit);
}

impl InsideBody for LocalPlayer {
    fn box_at(&self, position: Vec3) -> Aabb {
        self.dimensions.make_bounding_box(position)
    }

    fn powder_snow_shape(&self) -> VoxelShape {
        powder_snow_collision_shape(self.fall_distance)
    }

    fn inside_block_effect(&mut self, level: &Level<'_>, visit: &InsideVisit) {
        LocalPlayer::inside_block_effect(self, level, visit);
    }
}

/// `SwingAnimation.DEFAULT`'s duration: an empty hand's swing, in ticks.
/// How long a swing lasts with nothing special in the hand (`SwingAnimation.DEFAULT`).
const SWING_DURATION: i32 = 6;

/// What the local player's swing and use of items read of the world's player state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HandState {
    /// The registry ids of what the main and off hands hold (air when empty).
    pub main_item: i32,
    pub off_item: i32,
    /// The greater of the haste and conduit power amplifiers, if either is on.
    pub dig_speed: Option<i32>,
    pub mining_fatigue: Option<i32>,
    /// The server's flags: an item is in use, and in the off hand.
    pub using_item: bool,
    pub using_off_hand: bool,
}

/// What the player model is posed from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelPose {
    pub body_rot: f32,
    pub head_rot: f32,
    pub x_rot: f32,
    pub walk_position: f32,
    pub walk_speed: f32,
    pub swim_amount: f32,
    pub crouching: bool,
    pub pose: ClientEntityPose,
}

mod core;
mod effects;
mod movement;

mod level_block;

mod inside;
pub use inside::*;
