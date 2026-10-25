//! `Pig.mobInteract` and `Strider.mobInteract`: a saddled mount is boarded (a swing, the server
//! does the rest), food feeds it, and a saddle in hand goes on through the item.
//!
//! Not covered: the eating sound of a strider and what boarding clears locally.

use super::*;

impl Context<'_> {
    pub(super) fn interact_saddleable(&mut self, name: &str, id: EntityId, hand: u8) -> EntityUse {
        let tag = if name == "minecraft:pig" {
            "minecraft:pig_food"
        } else {
            "minecraft:strider_food"
        };
        let Some((saddled, rider, baby)) = self.saddle_state(id) else {
            return EntityUse::Pass;
        };
        let food = self.holds_tagged(hand, tag);
        if !food && saddled && !rider && !self.player.is_shift_key_down() {
            return EntityUse::Success { swing: true };
        }
        let fed = self.feed_animal(id, hand, tag);
        if fed == EntityUse::Pass && self.saddle_goes_on(hand, saddled, baby) {
            return EntityUse::Success { swing: true };
        }
        fed
    }

    /// The saddle item's `interactLivingEntity`: it equips a mob that can wear it and has none.
    fn saddle_goes_on(&self, hand: u8, saddled: bool, baby: bool) -> bool {
        let saddle = crate::item::id_of("minecraft:saddle");
        let holds = saddle.is_some() && self.held(hand).map(|(item, _)| item) == saddle;
        holds && !saddled && !baby
    }

    /// Whether the mob wears a saddle, has a passenger, and is a baby.
    fn saddle_state(&self, id: EntityId) -> Option<(bool, bool, bool)> {
        let entity = self.world.entities().at(id)?;
        let saddled = entity
            .equipment(photon_world::EquipmentSlot::Saddle)
            .is_some();
        let rider = self
            .world
            .entity_passengers(id)
            .is_some_and(|passengers| !passengers.is_empty());
        Some((saddled, rider, entity.is_baby()))
    }
}
