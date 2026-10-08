//! What a cat does of its own on the client (`Cat.tick`): how far it has
//! lain down, how far its tail has, how far it has relaxed, and whether it
//! lies on top of a sleeping player.

use photon_core::math::mth_lerp_f32 as lerp_f32;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CatState {
    lie_down_amount: f32,
    lie_down_amount_o: f32,
    lie_down_amount_tail: f32,
    lie_down_amount_tail_o: f32,
    relax_state_one_amount: f32,
    relax_state_one_amount_o: f32,
    lying_on_top_of_sleeping_player: bool,
}

// The amounts are finite, so equality is total.
impl Eq for CatState {}

impl CatState {
    /// One client tick (`Cat.handleLieDown`): the amounts move towards the
    /// pose the data asks for, and a cat that lies down looks for a sleeping
    /// player near it (`sleeper_near`, only read while it lies).
    pub(crate) fn tick(&mut self, lying: bool, relaxed: bool, sleeper_near: bool) {
        self.lie_down_amount_o = self.lie_down_amount;
        self.lie_down_amount_tail_o = self.lie_down_amount_tail;
        if lying {
            self.lie_down_amount = (self.lie_down_amount + 0.15).min(1.0);
            self.lie_down_amount_tail = (self.lie_down_amount_tail + 0.08).min(1.0);
        } else {
            self.lie_down_amount = (self.lie_down_amount - 0.22).max(0.0);
            self.lie_down_amount_tail = (self.lie_down_amount_tail - 0.13).max(0.0);
        }
        self.relax_state_one_amount_o = self.relax_state_one_amount;
        if relaxed {
            self.relax_state_one_amount = (self.relax_state_one_amount + 0.1).min(1.0);
        } else {
            self.relax_state_one_amount = (self.relax_state_one_amount - 0.13).max(0.0);
        }
        self.lying_on_top_of_sleeping_player = lying && sleeper_near;
    }

    /// `Cat.getLieDownAmount`.
    pub fn lie_down_amount(&self, partial: f32) -> f32 {
        lerp_f32(partial, self.lie_down_amount_o, self.lie_down_amount)
    }

    /// `Cat.getLieDownAmountTail`.
    pub fn lie_down_amount_tail(&self, partial: f32) -> f32 {
        lerp_f32(
            partial,
            self.lie_down_amount_tail_o,
            self.lie_down_amount_tail,
        )
    }

    /// `Cat.getRelaxStateOneAmount`.
    pub fn relax_state_one_amount(&self, partial: f32) -> f32 {
        lerp_f32(
            partial,
            self.relax_state_one_amount_o,
            self.relax_state_one_amount,
        )
    }

    /// `Cat.isLyingOnTopOfSleepingPlayer`.
    pub fn is_lying_on_top_of_sleeping_player(&self) -> bool {
        self.lying_on_top_of_sleeping_player
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lying_down_is_slower_than_getting_up_and_the_tail_trails() {
        let mut cat = CatState::default();
        let mut ticks = 0;
        while cat.lie_down_amount(1.0) < 1.0 {
            cat.tick(true, false, false);
            ticks += 1;
        }
        assert_eq!(ticks, 7);
        assert!(cat.lie_down_amount_tail(1.0) < 1.0);
        let mut up = 0;
        while cat.lie_down_amount(1.0) > 0.0 {
            cat.tick(false, false, false);
            up += 1;
        }
        assert_eq!(up, 5);
    }

    #[test]
    fn a_sleeper_only_counts_while_the_cat_lies() {
        let mut cat = CatState::default();
        cat.tick(false, false, true);
        assert!(!cat.is_lying_on_top_of_sleeping_player());
        cat.tick(true, false, true);
        assert!(cat.is_lying_on_top_of_sleeping_player());
    }

    #[test]
    fn relaxing_lifts_the_head_in_ten_ticks() {
        let mut cat = CatState::default();
        for _ in 0..10 {
            cat.tick(false, true, false);
        }
        assert_eq!(cat.relax_state_one_amount(1.0), 1.0);
        assert!((cat.relax_state_one_amount(0.0) - 0.9).abs() < 1e-6);
    }
}
