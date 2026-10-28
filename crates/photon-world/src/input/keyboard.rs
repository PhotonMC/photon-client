use super::*;

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

pub(super) fn modify_input_speed_for_square_movement(input: Vec2) -> Vec2 {
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
    let modified_length = java_math::min_f32(length * distance_to_unit_square, 1.0_f32);
    direction.scale(modified_length)
}

pub(super) fn java_max_f64(left: f64, right: f64) -> f64 {
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

pub(super) fn calculate_impulse(positive: bool, negative: bool) -> f32 {
    if positive == negative {
        0.0
    } else if positive {
        1.0
    } else {
        -1.0
    }
}

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

/// The state sampled by `KeyboardInput.tick()` from the client's key mappings.
#[derive(Debug, Clone, Copy)]
pub struct KeyboardInput {
    pub(super) key_states: Input,
    pub key_presses: Input,
    pub(super) move_vector: Vec2,
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

impl Input {
    pub const EMPTY: Self = Self::new((false, false, false, false), false, false, false);

    /// `walk` is forward, backward, left and right.
    pub const fn new(
        (forward, backward, left, right): (bool, bool, bool, bool),
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
