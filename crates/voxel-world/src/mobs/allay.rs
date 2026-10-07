//! What an allay does of its own on the client (`Allay.tick`): how far it has
//! raised its arms to what it holds, and how far into its dance it is.

use voxel_core::math::mth_lerp_f32;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct AllayState {
    holding_item_ticks_old: f32,
    holding_item_ticks: f32,
    dancing_ticks: f32,
    spinning_ticks_old: f32,
    spinning_ticks: f32,
}

impl AllayState {
    /// `Allay.tick` on the client, after `super.tick`.
    pub(crate) fn tick(&mut self, has_item_in_hand: bool, dancing: bool) {
        self.holding_item_ticks_old = self.holding_item_ticks;
        let step = if has_item_in_hand { 1.0 } else { -1.0 };
        self.holding_item_ticks = (self.holding_item_ticks + step).clamp(0.0, 5.0);
        if dancing {
            self.dancing_ticks += 1.0;
            self.spinning_ticks_old = self.spinning_ticks;
            self.spinning_ticks += if self.is_spinning() { 1.0 } else { -1.0 };
            self.spinning_ticks = self.spinning_ticks.clamp(0.0, 15.0);
        } else {
            self.dancing_ticks = 0.0;
            self.spinning_ticks = 0.0;
            self.spinning_ticks_old = 0.0;
        }
    }

    /// `isSpinning`: the first fifteen ticks of every fifty-five of the dance.
    pub fn is_spinning(&self) -> bool {
        self.dancing_ticks % 55.0 < 15.0
    }

    /// `getHoldingItemAnimationProgress`.
    pub fn holding_item_progress(&self, partial_tick: f32) -> f32 {
        mth_lerp_f32(
            partial_tick,
            self.holding_item_ticks_old,
            self.holding_item_ticks,
        ) / 5.0
    }

    /// `getSpinningProgress`.
    pub fn spinning_progress(&self, partial_tick: f32) -> f32 {
        mth_lerp_f32(partial_tick, self.spinning_ticks_old, self.spinning_ticks) / 15.0
    }
}

/// The counts are whole numbers of ticks, so the comparison is a total one.
impl Eq for AllayState {}
