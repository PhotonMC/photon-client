//! Powder snow's freeze on a living entity (`Entity.getTicksFrozen`, `getPercentFrozen`,
//! `isFullyFrozen` and `isFreezing`).

use super::view::EntityView;
use crate::DEFAULT_TICKS_REQUIRED_TO_FREEZE;

impl EntityView<'_> {
    pub fn ticks_frozen(&self) -> i32 {
        self.vitals.ticks_frozen
    }

    pub fn percent_frozen(&self) -> f32 {
        (self
            .vitals
            .ticks_frozen
            .min(DEFAULT_TICKS_REQUIRED_TO_FREEZE) as f32)
            / DEFAULT_TICKS_REQUIRED_TO_FREEZE as f32
    }

    pub fn is_fully_frozen(&self) -> bool {
        self.vitals.ticks_frozen >= DEFAULT_TICKS_REQUIRED_TO_FREEZE
    }

    pub fn is_freezing(&self) -> bool {
        self.vitals.ticks_frozen > 0
    }
}
