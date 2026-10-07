//! Keyboard-driven player input for the pinned Java client.

use crate::java_math;
use std::cmp::Ordering;

use voxel_core::math::{MTH_DEG_TO_RAD, Vec2, Vec3, clamp_f32, mth_cos_f64, mth_sin_f64};

/// The seven boolean fields sent by the Java player's input record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Input {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub shift: bool,
    pub sprint: bool,
}

impl Input {
    pub const EMPTY: Self = Self::new(false, false, false, false, false, false, false);

    pub const fn new(
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
        jump: bool,
        shift: bool,
        sprint: bool,
    ) -> Self {
        Self {
            forward,
            backward,
            left,
            right,
            jump,
            shift,
            sprint,
        }
    }
}

/// The state sampled by `KeyboardInput.tick()` from the client's key mappings.
#[derive(Debug, Clone, Copy)]
pub struct KeyboardInput {
    key_states: Input,
    pub key_presses: Input,
    move_vector: Vec2,
}

impl KeyboardInput {
    pub const fn new(key_states: Input) -> Self {
        Self {
            key_states,
            key_presses: Input::EMPTY,
            move_vector: Vec2::ZERO,
        }
    }

    /// Replace the key states that the next tick will sample.
    pub fn set_key_states(&mut self, key_states: Input) {
        self.key_states = key_states;
    }

    /// Snapshot key mappings and derive the normalized movement vector.
    pub fn tick(&mut self) {
        self.key_presses = self.key_states;
        let forward_impulse =
            calculate_impulse(self.key_presses.forward, self.key_presses.backward);
        let left_impulse = calculate_impulse(self.key_presses.left, self.key_presses.right);
        self.move_vector = Vec2::new(left_impulse, forward_impulse).normalized();
    }

    pub const fn move_vector(self) -> Vec2 {
        self.move_vector
    }
}

/// The LocalPlayer state that changes Java's normalized input vector before
/// movement consumes it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LocalPlayerInputModifiers {
    pub using_item: bool,
    pub passenger: bool,
    pub item_use_speed_multiplier: f32,
    pub moving_slowly: bool,
    pub sneaking_movement_factor: f32,
}

/// The fields written by Java LocalPlayer.applyInput for one tick.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerAppliedInput {
    pub movement: Vec2,
    pub jumping: bool,
    pub x_bob: f32,
    pub y_bob: f32,
    pub x_bob_old: f32,
    pub y_bob_old: f32,
}

/// The input and local state snapshot consumed by Java LocalPlayer.applyInput.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerInputState {
    pub move_vector: Vec2,
    pub jump_pressed: bool,
    pub current_impulses: Vec2,
    pub current_jumping: bool,
    pub current_bob: Vec2,
    pub previous_bob: Vec2,
    pub rotation: Vec2,
    pub controlled_camera: bool,
}

/// The non-passenger conditions read by LocalPlayer.canStartSprinting.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerSprintGateInput {
    pub sprinting: bool,
    pub forward_impulse: f32,
    pub mobility_restricted: bool,
    pub enough_food: bool,
    pub may_fly: bool,
    pub shallow_water: bool,
    pub flying: bool,
    pub slow_due_to_using_item: bool,
    pub fall_flying: bool,
    pub underwater: bool,
    pub moving_slowly: bool,
}

/// The non-passenger state used by LocalPlayer's sprint double-tap window.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerSprintTriggerConditions {
    pub mobility_restricted: bool,
    pub enough_food: bool,
    pub may_fly: bool,
    pub shallow_water: bool,
    pub flying: bool,
    pub slow_due_to_using_item: bool,
    pub fall_flying: bool,
    pub underwater: bool,
    pub moving_slowly: bool,
}

/// The previous and current input snapshots consumed by one LocalPlayer tick.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerSprintTriggerInput {
    pub sprinting: bool,
    pub sprint_trigger_time: i32,
    pub sprint_window: i32,
    pub previous_input: Input,
    pub previous_move_vector: Vec2,
    pub current_input: Input,
    pub current_move_vector: Vec2,
    pub conditions: LocalPlayerSprintTriggerConditions,
}

/// The local sprint state after one LocalPlayer sprint-trigger step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalPlayerSprintTriggerResult {
    pub sprinting: bool,
    pub sprint_trigger_time: i32,
}

/// The state consumed by LivingEntity's fluid falling adjustment.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerFluidFallingInput {
    pub base_gravity: f64,
    pub is_falling: bool,
    pub sprinting: bool,
    pub movement: Vec3,
}

/// The environment state consumed by LivingEntity's climbable movement clamp.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerClimbableMovementInput {
    pub movement: Vec3,
    pub on_climbable: bool,
    pub scaffolding: bool,
    pub suppress_sliding_down_ladder: bool,
}

/// The state consumed by LivingEntity's friction-influenced speed helper.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerFrictionInfluencedSpeedInput {
    pub on_ground: bool,
    pub block_friction: f32,
    pub speed: f32,
    pub flying: bool,
    pub flying_speed: f32,
    pub sprinting: bool,
    pub passenger: bool,
}

/// The block friction and attribute modifier consumed by LivingEntity travel.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerModifiedFrictionInput {
    pub friction: f32,
    pub modifier: f32,
}

/// The environment state consumed by LivingEntity.travelFlying.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelFlyingParameters {
    pub input: Vec3,
    pub y_rot: f32,
    pub in_water: bool,
    pub in_lava: bool,
    pub water_speed: f32,
    pub lava_speed: f32,
    pub air_speed: f32,
}

/// The environment and motion state consumed by LivingEntity.travelFlying.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelFlyingInput {
    pub input: Vec3,
    pub delta_movement: Vec3,
    pub y_rot: f32,
    pub in_water: bool,
    pub in_lava: bool,
    pub water_speed: f32,
    pub lava_speed: f32,
    pub air_speed: f32,
}

/// The environment state consumed by LivingEntity.travelInWater.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelInWaterParameters {
    pub input: Vec3,
    pub y_rot: f32,
    pub sprinting: bool,
    pub water_slow_down: f32,
    pub water_efficiency: f32,
    pub on_ground: bool,
    pub speed: f32,
    pub dolphins_grace: bool,
    pub horizontal_collision: bool,
    pub on_climbable: bool,
    pub free_after_jump: bool,
    pub base_gravity: f64,
    pub is_falling: bool,
}

/// The environment and motion state consumed by LivingEntity.travelInWater.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelInWaterInput {
    pub input: Vec3,
    pub delta_movement: Vec3,
    pub y_rot: f32,
    pub sprinting: bool,
    pub water_slow_down: f32,
    pub water_efficiency: f32,
    pub on_ground: bool,
    pub speed: f32,
    pub dolphins_grace: bool,
    pub horizontal_collision: bool,
    pub on_climbable: bool,
    pub free_after_jump: bool,
    pub base_gravity: f64,
    pub is_falling: bool,
}

/// The environment state consumed by LivingEntity.travelInLava.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelInLavaParameters {
    pub input: Vec3,
    pub y_rot: f32,
    pub sprinting: bool,
    pub shallow_lava: bool,
    pub horizontal_collision: bool,
    pub free_after_jump: bool,
    pub base_gravity: f64,
    pub is_falling: bool,
}

/// The environment and motion state consumed by LivingEntity.travelInLava.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelInLavaInput {
    pub input: Vec3,
    pub delta_movement: Vec3,
    pub y_rot: f32,
    pub sprinting: bool,
    pub shallow_lava: bool,
    pub horizontal_collision: bool,
    pub free_after_jump: bool,
    pub base_gravity: f64,
    pub is_falling: bool,
}

/// The environment state consumed by LivingEntity's relative friction path.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerHandleRelativeFrictionParameters {
    pub input: Vec3,
    pub y_rot: f32,
    pub on_ground: bool,
    pub block_friction: f32,
    pub speed: f32,
    pub flying_speed: f32,
    pub flying: bool,
    pub sprinting: bool,
    pub passenger: bool,
    pub on_climbable: bool,
    pub scaffolding: bool,
    pub suppress_sliding_down_ladder: bool,
    pub horizontal_collision: bool,
    pub jumping: bool,
}

/// The environment and motion state consumed by LivingEntity's relative
/// friction path.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerHandleRelativeFrictionInput {
    pub input: Vec3,
    pub delta_movement: Vec3,
    pub y_rot: f32,
    pub on_ground: bool,
    pub block_friction: f32,
    pub speed: f32,
    pub flying_speed: f32,
    pub flying: bool,
    pub sprinting: bool,
    pub passenger: bool,
    pub on_climbable: bool,
    pub scaffolding: bool,
    pub suppress_sliding_down_ladder: bool,
    pub horizontal_collision: bool,
    pub jumping: bool,
}

/// The environment state consumed by LivingEntity.travelInAir.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelInAirParameters {
    pub input: Vec3,
    pub y_rot: f32,
    pub on_ground: bool,
    pub block_friction: f32,
    pub friction_modifier: f32,
    pub speed: f32,
    pub flying_speed: f32,
    pub flying: bool,
    pub sprinting: bool,
    pub passenger: bool,
    pub on_climbable: bool,
    pub scaffolding: bool,
    pub suppress_sliding_down_ladder: bool,
    pub horizontal_collision: bool,
    pub jumping: bool,
    pub levitation_amplifier: Option<i32>,
    pub gravity: f64,
    pub client_side: bool,
    pub has_chunk_at: bool,
    pub current_y: f64,
    pub min_y: i32,
    pub should_discard_friction: bool,
    pub air_drag_modifier: f32,
    pub omnidirectional_air_mover: bool,
}

/// The environment and motion state consumed by LivingEntity.travelInAir.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelInAirInput {
    pub input: Vec3,
    pub delta_movement: Vec3,
    pub y_rot: f32,
    pub on_ground: bool,
    pub block_friction: f32,
    pub friction_modifier: f32,
    pub speed: f32,
    pub flying_speed: f32,
    pub flying: bool,
    pub sprinting: bool,
    pub passenger: bool,
    pub on_climbable: bool,
    pub scaffolding: bool,
    pub suppress_sliding_down_ladder: bool,
    pub horizontal_collision: bool,
    pub jumping: bool,
    pub levitation_amplifier: Option<i32>,
    pub gravity: f64,
    pub client_side: bool,
    pub has_chunk_at: bool,
    pub current_y: f64,
    pub min_y: i32,
    pub should_discard_friction: bool,
    pub air_drag_modifier: f32,
    pub omnidirectional_air_mover: bool,
}

/// The environment state consumed by LivingEntity.updateFallFlyingMovement.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerUpdateFallFlyingMovementParameters {
    pub look_angle: Vec3,
    pub x_rot: f32,
    pub gravity: f64,
}

/// The environment and motion state consumed by
/// LivingEntity.updateFallFlyingMovement.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerUpdateFallFlyingMovementInput {
    pub look_angle: Vec3,
    pub delta_movement: Vec3,
    pub x_rot: f32,
    pub gravity: f64,
}

/// The four top-level branches selected by LivingEntity.travel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalPlayerTravelMode {
    Water,
    Lava,
    FallFlying,
    Air,
}

impl LocalPlayerTravelMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Water => "water",
            Self::Lava => "lava",
            Self::FallFlying => "fall_flying",
            Self::Air => "air",
        }
    }
}

/// The environment state consumed by LivingEntity.travel dispatch.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelDispatchParameters {
    pub input: Vec3,
    pub y_rot: f32,
    pub x_rot: f32,
    pub look_angle: Vec3,
    pub in_water: bool,
    pub in_lava: bool,
    pub affected_by_fluids: bool,
    pub can_stand_on_fluid: bool,
    pub fall_flying: bool,
    pub on_climbable: bool,
    pub sprinting: bool,
    pub water_slow_down: f32,
    pub water_efficiency: f32,
    pub on_ground: bool,
    pub block_friction: f32,
    pub speed: f32,
    pub dolphins_grace: bool,
    pub horizontal_collision: bool,
    pub free_after_jump: bool,
    pub shallow_lava: bool,
    pub flying_speed: f32,
    pub flying: bool,
    pub passenger: bool,
    pub scaffolding: bool,
    pub suppress_sliding_down_ladder: bool,
    pub jumping: bool,
    pub levitation_amplifier: Option<i32>,
    pub gravity: f64,
    pub client_side: bool,
    pub has_chunk_at: bool,
    pub current_y: f64,
    pub min_y: i32,
    pub should_discard_friction: bool,
    pub friction_modifier: f32,
    pub air_drag_modifier: f32,
    pub omnidirectional_air_mover: bool,
}

/// The environment and current motion consumed by LivingEntity.travel dispatch.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelDispatchInput {
    pub parameters: LocalPlayerTravelDispatchParameters,
    pub delta_movement: Vec3,
}

/// The selected travel branch and resulting no-collision movement.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerTravelDispatchResult {
    pub mode: LocalPlayerTravelMode,
    pub fall_flying_climbable_fallback: bool,
    pub movement: Vec3,
}

mod friction;
pub use friction::*;
mod square;
pub use square::*;
