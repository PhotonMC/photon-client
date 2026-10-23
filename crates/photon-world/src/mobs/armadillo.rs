//! What an armadillo does of its own on the client: the animation states its
//! synced state starts (`Armadillo.setupAnimationStates`), how long it has been
//! in that state, and the peek event.

use crate::animation_state::AnimationState;

/// `Armadillo.ARMADILLO_STATE`'s data id: `AgeableMob` has the baby flag at 16 and the age-locked flag at 17.
pub const STATE_DATA_ID: u8 = 18;
/// The event that tells a scared armadillo to peek again.
pub const PEEK_EVENT: i8 = 64;

/// `Armadillo.ArmadilloState`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ArmadilloMode {
    #[default]
    Idle,
    Rolling,
    Scared,
    Unrolling,
}

impl ArmadilloMode {
    /// `ByIdMap` with the zero strategy: an id out of range is idle.
    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Rolling,
            2 => Self::Scared,
            3 => Self::Unrolling,
            _ => Self::Idle,
        }
    }

    /// `ArmadilloState.shouldHideInShell`.
    fn hides_in_shell(self, ticks_in_state: i64) -> bool {
        match self {
            Self::Idle => false,
            Self::Rolling => ticks_in_state > 5,
            Self::Scared => true,
            Self::Unrolling => ticks_in_state < 26,
        }
    }
}

/// `ArmadilloState.SCARED.animationDuration()`.
const SCARED_ANIMATION_DURATION: i32 = 50;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ArmadilloState {
    pub roll_out: AnimationState,
    pub roll_up: AnimationState,
    pub peek: AnimationState,
    mode: ArmadilloMode,
    in_state_ticks: i64,
    peek_received: bool,
}

impl ArmadilloState {
    pub fn mode(&self) -> ArmadilloMode {
        self.mode
    }

    pub fn is_scared(&self) -> bool {
        self.mode != ArmadilloMode::Idle
    }

    /// `Armadillo.shouldHideInShell`.
    pub fn should_hide_in_shell(&self) -> bool {
        self.mode.hides_in_shell(self.in_state_ticks)
    }

    /// `onSyncedDataUpdated(ARMADILLO_STATE)`: the state is set (sent again, too) and the time in it restarts.
    pub(crate) fn set_mode(&mut self, mode: ArmadilloMode) {
        self.mode = mode;
        self.in_state_ticks = 0;
    }

    /// `Armadillo.handleEntityEvent`: whether the armadillo answers the event itself.
    pub(crate) fn event(&mut self, event: i8) -> bool {
        if event == PEEK_EVENT {
            self.peek_received = true;
        }
        event == PEEK_EVENT
    }

    /// `Armadillo.setupAnimationStates`, in the tick after the rest of it.
    pub(crate) fn animate(&mut self, tick_count: i32) {
        match self.mode {
            ArmadilloMode::Idle => {
                self.roll_out.stop();
                self.roll_up.stop();
                self.peek.stop();
            }
            ArmadilloMode::Rolling => {
                self.roll_out.stop();
                self.roll_up.start_if_stopped(tick_count);
                self.peek.stop();
            }
            ArmadilloMode::Scared => {
                self.roll_out.stop();
                self.roll_up.stop();
                if self.peek_received {
                    self.peek.stop();
                    self.peek_received = false;
                }
                if self.in_state_ticks == 0 {
                    self.peek.start(tick_count);
                    self.peek.fast_forward(SCARED_ANIMATION_DURATION, 1.0);
                } else {
                    self.peek.start_if_stopped(tick_count);
                }
            }
            ArmadilloMode::Unrolling => {
                self.roll_out.start_if_stopped(tick_count);
                self.roll_up.stop();
                self.peek.stop();
            }
        }
    }

    /// The end of `Armadillo.tick`.
    pub(crate) fn finish_tick(&mut self) {
        self.in_state_ticks += 1;
    }
}
