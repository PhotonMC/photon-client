//! Keyboard-driven player input for the pinned Java client.

use crate::math::Vec2;

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

fn calculate_impulse(positive: bool, negative: bool) -> f32 {
    if positive == negative {
        0.0
    } else if positive {
        1.0
    } else {
        -1.0
    }
}
