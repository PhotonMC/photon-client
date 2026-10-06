//! What a boat does of its own on the client (`AbstractBoat.tick`): the rocking
//! of a hit running out, the paddles' rowing, the shaking of a bubble column,
//! and whether it floats under the water's surface.

use voxel_core::math::{clamped_lerp_f32, mth_lerp_f32};

/// `(float) (Math.PI / 8)`: how far a paddle turns each tick it rows.
const PADDLE_STEP: f32 = (std::f64::consts::PI / 8.0) as f32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoatState {
    /// `VehicleEntity.getHurtTime`, `getHurtDir` and `getDamage`.
    pub hurt_time: i32,
    pub hurt_dir: i32,
    pub damage: f32,
    /// `AbstractBoat.getBubbleTime`.
    pub bubble_time: i32,
    bubble_multiplier: f32,
    bubble_angle: f32,
    bubble_angle_old: f32,
    paddle_positions: [f32; 2],
    /// `AbstractBoat.isUnderWater`: its status at the start of the last tick.
    pub is_under_water: bool,
}

impl Default for BoatState {
    fn default() -> Self {
        Self {
            hurt_time: 0,
            hurt_dir: 1,
            damage: 0.0,
            bubble_time: 0,
            bubble_multiplier: 0.0,
            bubble_angle: 0.0,
            bubble_angle_old: 0.0,
            paddle_positions: [0.0; 2],
            is_under_water: false,
        }
    }
}

/// The values are finite, so the comparison is a total one.
impl Eq for BoatState {}

impl BoatState {
    /// `AbstractBoat.tick` on the client: the status, the hit's time and
    /// damage running down, then the bubble column's shaking and the paddles
    /// (`paddling` is whether each is rowing with someone to row it).
    pub(crate) fn tick(&mut self, under_water: bool, paddling: [bool; 2], tick_count: i32) {
        self.tick_start(under_water);
        self.tick_end(paddling, tick_count);
    }

    /// The part of the tick before the boat's own movement: the hit's time and damage run down.
    pub(crate) fn tick_start(&mut self, under_water: bool) {
        self.is_under_water = under_water;
        if self.hurt_time > 0 {
            self.hurt_time -= 1;
        }
        if self.damage > 0.0 {
            self.damage -= 1.0;
        }
    }

    /// The part after it: the bubble column's shaking and the paddles.
    pub(crate) fn tick_end(&mut self, paddling: [bool; 2], tick_count: i32) {
        // `tickBubbleColumn`.
        if self.bubble_time > 0 {
            self.bubble_multiplier += 0.05;
        } else {
            self.bubble_multiplier -= 0.1;
        }
        self.bubble_multiplier = self.bubble_multiplier.clamp(0.0, 1.0);
        self.bubble_angle_old = self.bubble_angle;
        self.bubble_angle =
            10.0 * (0.5 * f64::from(tick_count)).sin() as f32 * self.bubble_multiplier;
        for (position, rows) in self.paddle_positions.iter_mut().zip(paddling) {
            *position = if rows { *position + PADDLE_STEP } else { 0.0 };
        }
    }

    /// How far a paddle has turned (`paddlePositions`).
    pub(crate) fn paddle_position(&self, side: usize) -> f32 {
        self.paddle_positions[side]
    }

    /// `getBubbleAngle`.
    pub fn bubble_angle(&self, partial_tick: f32) -> f32 {
        mth_lerp_f32(partial_tick, self.bubble_angle_old, self.bubble_angle)
    }

    /// `getRowingTime(side, a)`: how long a paddle has rowed, between two ticks
    /// (zero while it does not row).
    pub fn rowing_time(&self, side: usize, rows: bool, partial_tick: f32) -> f32 {
        if rows {
            clamped_lerp_f32(
                partial_tick,
                self.paddle_positions[side] - PADDLE_STEP,
                self.paddle_positions[side],
            )
        } else {
            0.0
        }
    }
}
