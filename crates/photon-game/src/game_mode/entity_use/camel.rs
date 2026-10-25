//! `Camel.mobInteract` on the client: its own order, not `AbstractHorse`'s.
//!
//! Not covered: the heal-by-food item a hurt camel takes, and what riding clears locally.

use super::*;

impl Context<'_> {
    /// A sneaking owner opens the inventory (a swing); food is answered by the server (no
    /// swing, one used up only when it ages a baby); a golden dandelion goes to the age lock;
    /// everything else, riding included, consumes the click without a swing.
    pub(super) fn interact_camel(&mut self, name: &str, id: EntityId, hand: u8) -> EntityUse {
        let Some(entity) = self.world.entities().at(id) else {
            return EntityUse::Pass;
        };
        let (baby, locked) = (entity.is_baby(), entity.is_age_locked());
        if self.player.is_shift_key_down() && !baby {
            return EntityUse::Success { swing: true };
        }
        let tag = if name == "minecraft:camel_husk" {
            "minecraft:camel_husk_food"
        } else {
            "minecraft:camel_food"
        };
        if self.holds_tagged(hand, tag) {
            self.use_up_for_growth(hand, baby && !locked);
            return EntityUse::Success { swing: false };
        }
        if baby && self.holds_golden_dandelion() {
            return self.golden_dandelion_on_baby(id, hand);
        }
        EntityUse::Success { swing: false }
    }

    /// `itemStack.consume(1, player)` once the food ages a baby up (not with infinite materials).
    fn use_up_for_growth(&mut self, hand: u8, grows: bool) {
        if grows && !self.instabuild() {
            let slot = self.hand_slot(hand);
            self.world.shrink_local_player_slot(slot, 1);
        }
    }

    /// Whether the item in a hand is in an item tag.
    fn holds_tagged(&self, hand: u8, tag: &str) -> bool {
        self.held(hand).is_some_and(|(item, _)| {
            self.world
                .block_state_registry()
                .item_tags()
                .contains(item.0, tag)
        })
    }
}
