//! `TickRateManager` on a client: the tick rate the server runs the level at, whether it is frozen
//! and how many frozen ticks it is to run anyway (`/tick freeze`, `/tick step`).

/// The level's tick rate state, which starts at twenty a second, not frozen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TickRateManager {
    tick_rate: f32,
    frozen_ticks_to_run: i32,
    run_game_elements: bool,
    frozen: bool,
}

impl Default for TickRateManager {
    fn default() -> Self {
        Self {
            tick_rate: 20.0,
            frozen_ticks_to_run: 0,
            run_game_elements: true,
            frozen: false,
        }
    }
}

impl TickRateManager {
    /// `setTickRate`: at least one a second.
    pub fn set_tick_rate(&mut self, rate: f32) {
        self.tick_rate = rate.max(1.0);
    }

    pub fn tick_rate(&self) -> f32 {
        self.tick_rate
    }

    /// `setFrozen`.
    pub fn set_frozen(&mut self, frozen: bool) {
        self.frozen = frozen;
    }

    pub fn is_frozen(&self) -> bool {
        self.frozen
    }

    /// `setFrozenTicksToRun`.
    pub fn set_frozen_ticks_to_run(&mut self, ticks: i32) {
        self.frozen_ticks_to_run = ticks;
    }

    pub fn frozen_ticks_to_run(&self) -> i32 {
        self.frozen_ticks_to_run
    }

    /// `runsNormally`: the game's elements tick, as of the last [`Self::tick`].
    pub fn runs_normally(&self) -> bool {
        self.run_game_elements
    }

    /// `tick`, once a client tick: whether the elements run this tick, and a step is used up.
    pub fn tick(&mut self) {
        self.run_game_elements = !self.frozen || self.frozen_ticks_to_run > 0;
        if self.frozen_ticks_to_run > 0 {
            self.frozen_ticks_to_run -= 1;
        }
    }
}
