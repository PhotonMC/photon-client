//! The villagers' part of a click on the client: `Villager.mobInteract`,
//! `WanderingTrader.mobInteract` and `ZombieVillager.mobInteract`.
//!
//! The trade itself is the server's; the client only answers with a swing. Not covered: the
//! zombie villager taking the golden apple when it has weakness (the effects of other entities
//! are not tracked), and a villager that is already trading (the client does not know).

use super::*;

impl Context<'_> {
    /// The villagers' rule for the clicked class.
    pub(super) fn interact_villager_class(
        &mut self,
        name: &str,
        id: EntityId,
        hand: u8,
    ) -> EntityUse {
        match name {
            "minecraft:villager" => self.interact_villager(id, hand),
            "minecraft:wandering_trader" => self.interact_wandering_trader(id, hand),
            _ => self.interact_zombie_villager(hand),
        }
    }

    /// `Villager.mobInteract` on the client: a sleeping villager falls to `AgeableMob`; a baby
    /// shakes its head (the unhappy counter is set here too) and an adult talks, both swinging.
    pub(super) fn interact_villager(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let Some(entity) = self.world.entities().at(id) else {
            return EntityUse::Pass;
        };
        if entity.is_sleeping() {
            return self.golden_dandelion_on_baby(id, hand);
        }
        if entity.is_baby() {
            self.world.entities_mut().set_unhappy_counter(id, 40);
        }
        EntityUse::Success { swing: true }
    }

    /// `WanderingTrader.mobInteract`: a grown trader talks (a swing), a baby falls to
    /// `AgeableMob`.
    pub(super) fn interact_wandering_trader(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let grown = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| !entity.is_baby());
        if grown {
            EntityUse::Success { swing: true }
        } else {
            self.golden_dandelion_on_baby(id, hand)
        }
    }

    /// `ZombieVillager.mobInteract`: a golden apple is answered by the server (no swing, no
    /// `use_item`); anything else is `Zombie`'s, which passes.
    pub(super) fn interact_zombie_villager(&mut self, hand: u8) -> EntityUse {
        let apple = self
            .held(hand)
            .is_some_and(|(item, _)| crate::item::id_of("minecraft:golden_apple") == Some(item));
        if apple {
            EntityUse::Success { swing: false }
        } else {
            EntityUse::Pass
        }
    }
}
