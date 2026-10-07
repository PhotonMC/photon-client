//! What evoker fangs do of their own on the client (`EvokerFangs`): they lie still until the server
//! says they bite (entity event 4), then count their life down and snap shut along the way.

/// `EvokerFangs.handleEntityEvent`: the bite begins.
pub const ATTACK_EVENT: i8 = 4;

/// `EvokerFangs.lifeTicks` when it is made.
const LIFE_TICKS: i32 = 22;

/// `EvokerFangs.LIFE_OFFSET`.
const LIFE_OFFSET: i32 = 2;

/// How many ticks the bite takes (`EvokerFangs.ATTACK_DURATION`).
const ATTACK_DURATION: f32 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FangsState {
    life_ticks: i32,
    attack_started: bool,
}

impl Default for FangsState {
    fn default() -> Self {
        Self {
            life_ticks: LIFE_TICKS,
            attack_started: false,
        }
    }
}

impl FangsState {
    pub(crate) fn start_attack(&mut self) {
        self.attack_started = true;
    }

    /// `EvokerFangs.tick` on the client: once the bite has begun, the life runs down.
    pub(crate) fn tick(&mut self) {
        if self.attack_started {
            self.life_ticks -= 1;
        }
    }

    /// `EvokerFangs.getAnimationProgress`: nothing before the bite, then how far through it, between two ticks.
    pub fn animation_progress(&self, partial: f32) -> f32 {
        if !self.attack_started {
            return 0.0;
        }
        let remaining = self.life_ticks - LIFE_OFFSET;
        if remaining <= 0 {
            1.0
        } else {
            1.0 - (remaining as f32 - partial) / ATTACK_DURATION
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fangs_do_not_bite_until_told_and_then_run_their_life_down() {
        let mut state = FangsState::default();
        state.tick();
        assert_eq!(state.animation_progress(0.5), 0.0);
        state.start_attack();
        assert_eq!(state.animation_progress(0.0), 1.0 - 20.0 / 20.0);
        for _ in 0..20 {
            state.tick();
        }
        assert_eq!(state.animation_progress(0.0), 1.0);
    }
}
