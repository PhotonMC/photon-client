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

fn calculate_impulse(positive: bool, negative: bool) -> f32 {
    if positive == negative {
        0.0
    } else if positive {
        1.0
    } else {
        -1.0
    }
}
