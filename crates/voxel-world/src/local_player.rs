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
use crate::fluid::{FluidTracker, LAVA_FLOW_SCALE, WATER_FLOW_SCALE};
use voxel_data::block::classes::BlockClass;
use voxel_data::block::definition::Fluid;
use voxel_data::block::registry;

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

/// What the vehicle the player rides tells its tick: the top of the vehicle's box and whether the
/// vehicle is under water (`modifyPassengerFluidInteractionBox`), and whether it can be sprinted
/// in (`Entity.canSprint`).
#[derive(Debug, Clone, Copy)]
pub(super) struct Riding {
    pub(super) vehicle_max_y: f64,
    pub(super) vehicle_under_water: bool,
    /// Whether the vehicle changes the box the player's fluids are found in (a boat does).
    pub(super) modifies_fluid_box: bool,
    pub(super) can_sprint: bool,
    /// Whether the jump key charges a jump of the vehicle (`PlayerRideableJumping.canJump`).
    pub(super) can_jump: bool,
}

/// The client's own player, ticked as the reference ticks it.
#[derive(Debug, Clone)]
pub struct LocalPlayer {
    pub(super) entity_id: i32,
    pub(super) position: Vec3,
    pub(super) old_position: Vec3,
    pub(super) delta: Vec3,
    pub(super) y_rot: f32,
    pub(super) x_rot: f32,
    pub(super) bounding_box: Aabb,
    pub(super) pose: ClientEntityPose,
    pub(super) dimensions: ClientEntityDimensions,
    pub(super) on_ground: bool,
    pub(super) horizontal_collision: bool,
    pub(super) vertical_collision: bool,
    pub(super) vertical_collision_below: bool,
    pub(super) minor_horizontal_collision: bool,
    pub(super) main_supporting_block: Option<BlockPos>,
    pub(super) on_ground_no_blocks: bool,
    pub(super) fall_distance: f64,
    pub(super) stuck_speed_multiplier: Vec3,
    /// `ClientAvatarState`: the distance walked and the view bob, now and a
    /// tick ago, which only the camera reads.
    pub(super) walk_dist: f32,
    pub(super) walk_dist_old: f32,
    /// The body's and the head's facing, now and a tick ago, which the
    /// player model is turned by.
    pub(super) y_body_rot: f32,
    pub(super) y_body_rot_old: f32,
    pub(super) y_head_rot: f32,
    pub(super) y_head_rot_old: f32,
    /// `WalkAnimationState`: how fast and how far the limbs have swung.
    pub(super) walk: crate::living::WalkAnimation,
    /// What the vehicle the player sits in makes of it, while it rides.
    pub(super) riding: Option<Riding>,
    /// `LocalPlayer.jumpRidingTicks` and `jumpRidingScale`: how long the jump key has charged a
    /// mount's jump, and how hard that is; the jump let go of this tick, for the mount.
    pub(super) jump_riding_ticks: i32,
    pub(super) jump_riding_scale: f32,
    pub(super) ride_jump: Option<i32>,
    /// The first-person hand's lagging view: it chases the rotation by
    /// half the gap each tick.
    pub(super) x_bob: f32,
    pub(super) y_bob: f32,
    pub(super) x_bob_old: f32,
    pub(super) y_bob_old: f32,
    /// `LivingEntity`'s arm swing: whether one is under way, how many
    /// ticks in, and how far through, now and a tick ago.
    pub(super) swinging: bool,
    pub(super) swing_time: i32,
    /// `swingingArm`: the hand that swung last (the main hand until one swings).
    pub(super) swinging_off_hand: bool,
    pub(super) attack_anim: f32,
    pub(super) attack_anim_old: f32,
    /// `Player.attackStrengthTicker` and `itemSwapTicker`: ticks since the
    /// last attack or change of held item.
    pub(super) attack_strength_ticker: i32,
    /// The main hand item's registry id (air when empty), and the one the
    /// last tick saw.
    pub(super) main_hand_item: i32,
    pub(super) last_main_hand_item: i32,
    /// What the hands hold, the effects and the use of an item, as the world last heard them.
    pub(super) hands: HandState,
    /// How many ticks the item in use has been used (`getTicksUsingItem`).
    pub(super) using_item_ticks: i32,
    pub(super) item_swap_ticker: i32,
    /// `LivingEntity`'s health as the client last set it, and the hurt
    /// state `LocalPlayer.hurtTo` keeps: ticks of damage immunity, of the
    /// hurt flash, and whether the next health update may flash at all.
    pub(super) health: f32,
    pub(super) flash_on_set_health: bool,
    pub(super) invulnerable_time: i32,
    pub(super) hurt_time: i32,
    /// `hurtDuration`: what the flinch started from.
    pub(super) hurt_duration: i32,
    /// `Player.hurtDir`: where the last blow came from, in degrees.
    pub(super) hurt_dir: f32,
    /// `deathTime`: the ticks since the health ran out.
    pub(super) death_time: i32,
    /// How far into the swimming pose the model has turned.
    pub(super) swim_amount: f32,
    pub(super) swim_amount_old: f32,
    pub(super) bob: f32,
    pub(super) bob_old: f32,
    pub(super) first_tick: bool,
    pub(super) tick_count: i32,
    /// `Entity.random`: the pitch of the player's sounds is drawn from it. The reference seeds
    /// it from nowhere in particular; this one from the entity's number.
    pub(super) random: voxel_core::random::RandomSource,
    /// `BubbleColumnAmbientSoundHandler`: whether the last tick was in a bubble column, and
    /// whether it is the first.
    pub(super) was_in_bubble_column: bool,
    pub(super) ambient_first_tick: bool,
    /// `LivingEntity.elytraAnimationState`: the local player never glides in this port.
    pub(super) elytra: crate::elytra::ElytraState,
    /// `Entity.moveDist` and `nextStep`: how far the player has walked, and where the next step sounds.
    pub(super) move_dist: f32,
    pub(super) next_step: f32,
    pub(super) pending_sounds: Vec<LocalSound>,
    pub(super) was_touching_water: bool,
    pub(super) was_eye_in_water: bool,
    pub(super) water: FluidTracker,
    pub(super) lava: FluidTracker,
    pub(super) sprinting: bool,
    pub(super) swimming: bool,
    pub(super) crouching: bool,
    pub(super) input: KeyboardInput,
    pub(super) xxa: f32,
    pub(super) zza: f32,
    pub(super) jumping: bool,
    pub(super) no_jump_delay: i32,
    pub(super) jump_trigger_time: i32,
    pub(super) sprint_trigger_time: i32,
    pub(super) auto_jump_time: i32,
    pub(super) abilities: ClientPlayerAbilities,
    pub(super) game_mode: ClientPlayerGameMode,
    pub(super) no_physics: bool,
    pub(super) auto_jump_option: bool,
    pub(super) auto_jump_enabled: bool,
    pub(super) movements_this_tick: Vec<Movement>,
    pub(super) x_last: f64,
    pub(super) y_last: f64,
    pub(super) z_last: f64,
    pub(super) y_rot_last: f32,
    pub(super) x_rot_last: f32,
    pub(super) last_on_ground: bool,
    pub(super) last_horizontal_collision: bool,
    pub(super) was_sprinting: bool,
    pub(super) position_reminder: i32,
    pub(super) last_sent_input: Input,
    pub(super) sent: Vec<ServerboundMovement>,
}

mod block_info;
pub(crate) use block_info::*;
mod falling;
use falling::*;
mod fluids;
pub use fluids::*;
mod hand;
pub use hand::*;
mod inside_blocks;
pub(crate) use inside_blocks::*;
mod level;
pub(crate) use level::*;
mod pose;
pub use pose::*;
mod rotation;
use rotation::*;
mod serverbound;
pub use serverbound::*;
mod sprinting;
mod state;
pub use state::*;
mod ticking;
mod travel;
pub(crate) use travel::*;
