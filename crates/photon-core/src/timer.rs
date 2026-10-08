//! Tick accumulation and render-time deltas for the pinned Java client.
//!
//! The caller supplies millisecond timestamps and the current target tick duration.
//! This is not the game loop: it neither executes ticks nor applies the frame cap.

#[derive(Debug)]
pub struct Timer {
    nominal_ms_per_tick: f32,
    game_clock_ms: i64,
    real_clock_ms: i64,
    game_delta: f32,
    real_delta: f32,
    residual: f32,
    paused_residual: Option<f32>,
    frozen: bool,
}

impl Timer {
    pub fn new(ticks_per_second: f32, current_ms: i64) -> Self {
        Self {
            nominal_ms_per_tick: 1000.0 / ticks_per_second,
            game_clock_ms: current_ms,
            real_clock_ms: current_ms,
            game_delta: 0.0,
            real_delta: 0.0,
            residual: 0.0,
            paused_residual: None,
            frozen: false,
        }
    }

    /// Accumulate elapsed game time using the caller's current tick duration.
    /// Pause/freeze affect exposed partial ticks, not this accumulation step.
    pub fn advance_game_time(&mut self, current_ms: i64, target_ms_per_tick: f32) -> i32 {
        // Subtract at i64 precision before converting to f32, including Java's
        // wrapping integer arithmetic. Do not round each timestamp separately.
        self.game_delta = current_ms.wrapping_sub(self.game_clock_ms) as f32 / target_ms_per_tick;
        self.game_clock_ms = current_ms;
        self.residual += self.game_delta;
        let ticks = self.residual as i32;
        self.residual -= ticks as f32;
        ticks
    }

    /// Advance the independent UI/real-time clock at the constructor's tick rate.
    pub fn advance_real_time(&mut self, current_ms: i64) {
        self.real_delta =
            current_ms.wrapping_sub(self.real_clock_ms) as f32 / self.nominal_ms_per_tick;
        self.real_clock_ms = current_ms;
    }

    pub fn update_pause_state(&mut self, paused: bool) {
        if paused {
            self.paused_residual.get_or_insert(self.residual);
        } else if let Some(saved) = self.paused_residual.take() {
            self.residual = saved;
        }
    }

    pub fn update_frozen_state(&mut self, frozen: bool) {
        self.frozen = frozen;
    }

    pub fn game_time_delta_ticks(&self) -> f32 {
        self.game_delta
    }

    pub fn game_time_delta_partial_tick(&self, ignore_frozen_game: bool) -> f32 {
        if self.frozen && !ignore_frozen_game {
            1.0
        } else {
            self.paused_residual.unwrap_or(self.residual)
        }
    }

    pub fn realtime_delta_ticks(&self) -> f32 {
        if self.real_delta > 7.0 {
            0.5
        } else {
            self.real_delta
        }
    }
}
