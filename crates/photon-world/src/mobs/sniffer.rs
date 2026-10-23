//! What a sniffer does of its own on the client: the animation states its
//! synced state starts (`Sniffer.onSyncedDataUpdated`).

use crate::animation_state::AnimationState;

/// `Sniffer.DATA_STATE`'s data id: `AgeableMob` has the baby flag at 16 and the age-locked flag at 17.
pub const STATE_DATA_ID: u8 = 18;

/// `Sniffer.State`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SnifferMode {
    #[default]
    Idling,
    FeelingHappy,
    Scenting,
    Sniffing,
    Searching,
    Digging,
    Rising,
}

impl SnifferMode {
    /// `ByIdMap` with the zero strategy: an id out of range is idling.
    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::FeelingHappy,
            2 => Self::Scenting,
            3 => Self::Sniffing,
            4 => Self::Searching,
            5 => Self::Digging,
            6 => Self::Rising,
            _ => Self::Idling,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SnifferState {
    pub digging: AnimationState,
    pub sniffing: AnimationState,
    pub rising: AnimationState,
    pub feeling_happy: AnimationState,
    pub scenting: AnimationState,
    mode: SnifferMode,
}

impl SnifferState {
    /// `Sniffer.isSearching`.
    pub fn is_searching(&self) -> bool {
        self.mode == SnifferMode::Searching
    }

    /// `onSyncedDataUpdated(DATA_STATE)`: every animation stops, then the
    /// state's own starts (if stopped, which it now is).
    pub(crate) fn set_mode(&mut self, mode: SnifferMode, tick_count: i32) {
        self.mode = mode;
        self.digging.stop();
        self.sniffing.stop();
        self.rising.stop();
        self.feeling_happy.stop();
        self.scenting.stop();
        match mode {
            SnifferMode::FeelingHappy => self.feeling_happy.start_if_stopped(tick_count),
            SnifferMode::Scenting => self.scenting.start_if_stopped(tick_count),
            SnifferMode::Sniffing => self.sniffing.start_if_stopped(tick_count),
            SnifferMode::Digging => self.digging.start_if_stopped(tick_count),
            SnifferMode::Rising => self.rising.start_if_stopped(tick_count),
            SnifferMode::Idling | SnifferMode::Searching => {}
        }
    }
}
