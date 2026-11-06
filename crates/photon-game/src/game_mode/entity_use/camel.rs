//! `Camel.mobInteract` on the client: its own order, not `AbstractHorse`'s.
//!
//! Not covered: what riding clears locally.

use super::*;

fn is_husk(name: &str) -> bool {
    name == "minecraft:camel_husk"
}

/// The item tag of what the camel family eats.
fn camel_food(name: &str) -> &'static str {
    if is_husk(name) {
        "minecraft:camel_husk_food"
    } else {
        "minecraft:camel_food"
    }
}

impl Context<'_> {
    /// A sneaking owner opens the inventory (a swing); food is answered by the server (no
    /// swing, one used up only when it ages a baby); a golden dandelion goes to the age lock;
    /// a grown camel with room is ridden (the click is consumed without a swing); everything else fails
    /// the click, which goes on to the held item.
    pub(super) fn interact_camel(&mut self, name: &str, id: EntityId, hand: u8) -> EntityUse {
        let Some(entity) = self.world.entities().at(id) else {
            return EntityUse::Pass;
        };
        // The reference answers a camel husk as a grown one whatever the baby flag says (rows
        // `use-baby-camel-husk-*`): it never ages.
        let baby = entity.is_baby() && !is_husk(name);
        let locked = entity.is_age_locked();
        if self.camel_click_swings(id, hand, baby) {
            return EntityUse::Success { swing: true };
        }
        if self.holds_tagged(hand, camel_food(name)) {
            self.eat_camel_food(id, hand, baby && !locked);
            return EntityUse::Success { swing: false };
        }
        if self.camel_ridden_by_click(id, baby) {
            return EntityUse::Success { swing: false };
        }
        if baby && self.holds_golden_dandelion() {
            return self.golden_dandelion_on_baby(id, hand);
        }
        // 26.3 fails the click, which goes on to the held item.
        EntityUse::Pass
    }

    /// A sneaking owner opens the inventory, or the click equips a saddle or armor: a swing.
    fn camel_click_swings(&mut self, id: EntityId, hand: u8, baby: bool) -> bool {
        if self.player.is_shift_key_down() && !baby {
            return true;
        }
        self.equips_on_interact(id, hand)
    }

    /// `doPlayerRide`: a grown camel with a free seat takes the rider, and the click is consumed.
    fn camel_ridden_by_click(&self, id: EntityId, baby: bool) -> bool {
        let riders = self
            .world
            .entity_passengers(id)
            .map_or(0, |riders| riders.len());
        riders < 2 && !baby
    }

    /// `Camel.handleEating`: the food heals a hurt camel by two, and is used up when it heals or
    /// ages a baby.
    fn eat_camel_food(&mut self, id: EntityId, hand: u8, ages: bool) {
        let healed = self.heal_local(id, 2.0);
        self.use_up_for_growth(hand, healed || ages);
    }

    /// `itemStack.consume(1, player)` once the food ages a baby up (not with infinite materials).
    fn use_up_for_growth(&mut self, hand: u8, grows: bool) {
        if grows && !self.instabuild() {
            let slot = self.hand_slot(hand);
            self.world.shrink_local_player_slot(slot, 1);
        }
    }

    /// Whether the item in a hand is in an item tag.
    pub(super) fn holds_tagged(&self, hand: u8, tag: &str) -> bool {
        self.held(hand).is_some_and(|(item, _)| {
            self.world
                .block_state_registry()
                .item_tags()
                .contains(item.0, tag)
        })
    }
}
