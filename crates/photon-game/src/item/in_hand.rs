//! `ItemInHandRenderer`'s state: the stacks each hand shows, which lag the
//! inventory while the hand dips out of view and back, and how high each
//! hand is.

use photon_core::ids::ItemId;
use photon_world::ClientItemStackState;

/// A hand's shown stack and its height, 1 when fully up.
#[derive(Debug, Clone, Default)]
struct Hand {
    item: Option<ClientItemStackState>,
    height: f32,
    height_old: f32,
}

/// What `ItemInHandRenderer.tick` keeps across ticks.
#[derive(Debug, Clone, Default)]
pub struct ItemInHand {
    main: Hand,
    off: Hand,
    invisible: bool,
}

/// A client item's swap properties, by item id: whether a new stack dips
/// the hand, and how far.
pub type SwapProperties<'a> = dyn Fn(ItemId) -> (bool, f32) + 'a;

/// `ItemInHandRenderer.shouldInstantlyReplaceVisibleItem`. An empty stack
/// has no item model component, so it always animates.
fn replaces_instantly(
    shown: Option<&ClientItemStackState>,
    next: Option<&ClientItemStackState>,
    swap: &SwapProperties<'_>,
) -> bool {
    match (shown, next) {
        (None, None) => true,
        (Some(shown), Some(next)) if shown.matches_for_swap(next) => true,
        (_, Some(next)) => !swap(next.item_registry_id()).0,
        (_, None) => false,
    }
}

/// The reference compares the shown and the next stack as objects.
fn same_object(shown: Option<&ClientItemStackState>, next: Option<&ClientItemStackState>) -> bool {
    match (shown, next) {
        (None, None) => true,
        (Some(shown), Some(next)) => shown.is_same_object(next),
        _ => false,
    }
}

impl ItemInHand {
    /// Whether the player is invisible: its bare arm is then not drawn.
    pub fn set_invisible(&mut self, invisible: bool) {
        self.invisible = invisible;
    }

    pub fn invisible(&self) -> bool {
        self.invisible
    }

    /// `ItemInHandRenderer.tick`, with the player's stacks now, its item
    /// swap scale and whether its hands are busy.
    pub fn tick(
        &mut self,
        next_main: Option<&ClientItemStackState>,
        next_off: Option<&ClientItemStackState>,
        item_swap_scale: f32,
        hands_busy: bool,
        swap: &SwapProperties<'_>,
    ) {
        self.main.height_old = self.main.height;
        self.off.height_old = self.off.height;
        if replaces_instantly(self.main.item.as_ref(), next_main, swap) {
            self.main.item = next_main.cloned();
        }
        if replaces_instantly(self.off.item.as_ref(), next_off, swap) {
            self.off.item = next_off.cloned();
        }
        if hands_busy {
            self.main.height = (self.main.height - 0.4).clamp(0.0, 1.0);
            self.off.height = (self.off.height - 0.4).clamp(0.0, 1.0);
        } else {
            let main_target = if same_object(self.main.item.as_ref(), next_main) {
                item_swap_scale * item_swap_scale * item_swap_scale
            } else {
                0.0
            };
            let off_target = if same_object(self.off.item.as_ref(), next_off) {
                1.0
            } else {
                0.0
            };
            self.main.height += (main_target - self.main.height).clamp(-0.4, 0.4);
            self.off.height += (off_target - self.off.height).clamp(-0.4, 0.4);
        }
        if self.main.height < 0.1 {
            self.main.item = next_main.cloned();
        }
        if self.off.height < 0.1 {
            self.off.item = next_off.cloned();
        }
    }

    /// `ItemInHandRenderer.itemUsed`: the hand drops to the bottom.
    pub fn item_used(&mut self, hand: u8) {
        if hand == crate::game_mode::MAIN_HAND {
            self.main.height = 0.0;
        } else {
            self.off.height = 0.0;
        }
    }

    /// The stack a hand shows.
    pub fn item(&self, hand: u8) -> Option<&ClientItemStackState> {
        self.hand(hand).item.as_ref()
    }

    /// How far a hand is out of view between the last two ticks, scaled by
    /// its stack's swap animation scale.
    pub fn inverse_arm_height(
        &self,
        hand: u8,
        partial_tick: f32,
        swap: &SwapProperties<'_>,
    ) -> f32 {
        let hand = self.hand(hand);
        let scale = hand
            .item
            .as_ref()
            .map_or(1.0, |stack| swap(stack.item_registry_id()).1);
        scale * (1.0 - (hand.height_old + partial_tick * (hand.height - hand.height_old)))
    }

    fn hand(&self, hand: u8) -> &Hand {
        if hand == crate::game_mode::MAIN_HAND {
            &self.main
        } else {
            &self.off
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULTS: &SwapProperties<'static> = &|_| (true, 1.0);

    #[test]
    fn a_new_stack_dips_the_hand_and_swaps_at_the_bottom() {
        let mut hand = ItemInHand::default();
        let stone = ClientItemStackState::new(64, photon_core::ids::ItemId(1));
        for _ in 0..3 {
            hand.tick(None, None, 1.0, false, DEFAULTS);
        }
        assert_eq!(hand.inverse_arm_height(0, 1.0, DEFAULTS), 0.0);
        // Down 0.4 a tick while the empty hand still shows.
        hand.tick(Some(&stone), None, 1.0, false, DEFAULTS);
        assert!(hand.item(0).is_none());
        hand.tick(Some(&stone), None, 1.0, false, DEFAULTS);
        assert!(hand.item(0).is_none());
        hand.tick(Some(&stone), None, 1.0, false, DEFAULTS);
        // Below a tenth the new stack shows; then it rises.
        assert_eq!(hand.item(0).map(|stack| stack.item_count()), Some(64));
        hand.tick(Some(&stone), None, 1.0, false, DEFAULTS);
        assert!(hand.inverse_arm_height(0, 1.0, DEFAULTS) < 0.7);
    }

    #[test]
    fn the_same_stack_changed_in_place_does_not_dip() {
        let mut hand = ItemInHand::default();
        let mut stone = ClientItemStackState::new(64, photon_core::ids::ItemId(1));
        for _ in 0..5 {
            hand.tick(Some(&stone), None, 1.0, false, DEFAULTS);
        }
        stone.set_count(63);
        hand.tick(Some(&stone), None, 1.0, false, DEFAULTS);
        assert_eq!(hand.inverse_arm_height(0, 1.0, DEFAULTS), 0.0);
        assert_eq!(hand.item(0).map(|stack| stack.item_count()), Some(63));
    }
}
