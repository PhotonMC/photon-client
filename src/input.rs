//! Keyboard-driven player input for the pinned Java client.

use crate::math::{Vec2, Vec3};

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

/// Apply Java LocalPlayer's item, slow-movement, and square-speed modifiers.
pub fn modify_local_player_input(input: Vec2, modifiers: LocalPlayerInputModifiers) -> Vec2 {
    if input.length_squared() == 0.0 {
        return input;
    }

    let mut modified = input.scale(0.98_f32);
    if modifiers.using_item && !modifiers.passenger {
        modified = modified.scale(modifiers.item_use_speed_multiplier);
    }
    if modifiers.moving_slowly {
        modified = modified.scale(modifiers.sneaking_movement_factor);
    }

    modify_input_speed_for_square_movement(modified)
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

/// Apply Java LocalPlayer.applyInput to the local player's movement fields.
pub fn apply_local_player_input(
    state: LocalPlayerInputState,
    modifiers: LocalPlayerInputModifiers,
) -> LocalPlayerAppliedInput {
    if !state.controlled_camera {
        return LocalPlayerAppliedInput {
            movement: state.current_impulses.scale(0.98_f32),
            jumping: state.current_jumping,
            x_bob: state.current_bob.x,
            y_bob: state.current_bob.y,
            x_bob_old: state.previous_bob.x,
            y_bob_old: state.previous_bob.y,
        };
    }

    let modified_input = modify_local_player_input(state.move_vector, modifiers);
    let bob_delta = Vec2::new(
        (state.rotation.x - state.current_bob.x) * 0.5_f32,
        (state.rotation.y - state.current_bob.y) * 0.5_f32,
    );
    let next_bob = state.current_bob.add_vec2(bob_delta);
    LocalPlayerAppliedInput {
        movement: modified_input,
        jumping: state.jump_pressed,
        x_bob: next_bob.x,
        y_bob: next_bob.y,
        x_bob_old: state.current_bob.x,
        y_bob_old: state.current_bob.y,
    }
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

/// Apply the non-passenger LocalPlayer.canStartSprinting predicate.
pub fn can_start_local_player_sprinting(state: LocalPlayerSprintGateInput) -> bool {
    !state.sprinting
        && state.forward_impulse > 1.0e-5_f32
        && !state.mobility_restricted
        && (state.enough_food || state.may_fly)
        && (state.flying || !state.shallow_water)
        && !state.slow_due_to_using_item
        && (!state.fall_flying || state.underwater)
        && (!state.moving_slowly || state.underwater)
}

/// The state consumed by LivingEntity's fluid falling adjustment.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerFluidFallingInput {
    pub base_gravity: f64,
    pub is_falling: bool,
    pub sprinting: bool,
    pub movement: Vec3,
}

/// Apply Java LivingEntity.getFluidFallingAdjustedMovement.
pub fn adjust_local_player_fluid_falling_movement(state: LocalPlayerFluidFallingInput) -> Vec3 {
    if state.base_gravity != 0.0 && !state.sprinting {
        let gravity_step = state.base_gravity / 16.0;
        let movement_y = if state.is_falling
            && (state.movement.y - 0.005).abs() >= 0.003
            && (state.movement.y - gravity_step).abs() < 0.003
        {
            -0.003
        } else {
            state.movement.y - gravity_step
        };
        Vec3::new(state.movement.x, movement_y, state.movement.z)
    } else {
        state.movement
    }
}

/// The environment state consumed by LivingEntity's climbable movement clamp.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerClimbableMovementInput {
    pub movement: Vec3,
    pub on_climbable: bool,
    pub scaffolding: bool,
    pub suppress_sliding_down_ladder: bool,
}

/// Apply Java LivingEntity.handleOnClimbable to local player motion.
pub fn clamp_local_player_climbable_movement(state: LocalPlayerClimbableMovementInput) -> Vec3 {
    if !state.on_climbable {
        return state.movement;
    }

    // The Java source passes -0.15F and 0.15F to the double overload, so the
    // bounds are the f32 values widened to f64.
    let max = f64::from(0.15_f32);
    let mut movement_y = java_max_f64(state.movement.y, -max);
    if movement_y < 0.0 && !state.scaffolding && state.suppress_sliding_down_ladder {
        movement_y = 0.0;
    }
    Vec3::new(
        java_clamp_f64(state.movement.x, -max, max),
        movement_y,
        java_clamp_f64(state.movement.z, -max, max),
    )
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

/// Apply Java LivingEntity.getFrictionInfluencedSpeed and getFlyingSpeed.
pub fn local_player_friction_influenced_speed(
    state: LocalPlayerFrictionInfluencedSpeedInput,
) -> f32 {
    if state.on_ground {
        if state.block_friction > 0.6_f32 {
            state.speed
                * (0.21600002_f32
                    / (state.block_friction * state.block_friction * state.block_friction))
        } else {
            state.speed
        }
    } else if state.flying && !state.passenger {
        if state.sprinting {
            state.flying_speed * 2.0_f32
        } else {
            state.flying_speed
        }
    } else if state.sprinting {
        0.025999999_f32
    } else {
        0.02_f32
    }
}

fn modify_input_speed_for_square_movement(input: Vec2) -> Vec2 {
    let length = input.length();
    if length <= 0.0 {
        return input;
    }

    let direction = input.scale(1.0_f32 / length);
    let direction_x = direction.x.abs();
    let direction_y = direction.y.abs();
    let tangent = if direction_y > direction_x {
        direction_x / direction_y
    } else {
        direction_y / direction_x
    };
    // Vec2.length() follows Mth.sqrt(1.0F + square(tangent)) with the
    // reference's f32 addition and f64 sqrt narrowing.
    let distance_to_unit_square = Vec2::new(1.0_f32, tangent).length();
    let modified_length = java_min_f32(length * distance_to_unit_square, 1.0_f32);
    direction.scale(modified_length)
}

fn java_min_f32(left: f32, right: f32) -> f32 {
    if left.is_nan() {
        left
    } else if right.is_nan() {
        right
    } else if left == 0.0 && right == 0.0 {
        f32::from_bits(left.to_bits() | right.to_bits())
    } else if left <= right {
        left
    } else {
        right
    }
}

fn java_clamp_f64(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else {
        java_min_f64(value, max)
    }
}

fn java_min_f64(left: f64, right: f64) -> f64 {
    if left.is_nan() || right.is_nan() {
        f64::NAN
    } else if left == 0.0 && right == 0.0 {
        f64::from_bits(left.to_bits() | right.to_bits())
    } else if left <= right {
        left
    } else {
        right
    }
}

fn java_max_f64(left: f64, right: f64) -> f64 {
    if left.is_nan() || right.is_nan() {
        f64::NAN
    } else if left == 0.0 && right == 0.0 {
        f64::from_bits(left.to_bits() | right.to_bits())
    } else if left >= right {
        left
    } else {
        right
    }
}

fn calculate_impulse(positive: bool, negative: bool) -> f32 {
    if positive == negative {
        0.0
    } else if positive {
        1.0
    } else {
        -1.0
    }
}
