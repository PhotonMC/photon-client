//! `Wolf.mobInteract` and `Cat.mobInteract` on the client: feeding a hurt pet, the collar dye,
//! the wolf's armor, and the sit toggle that an owner's pass-through click ends in.
//!
//! Not covered: repairing a wolf armor (it needs the armor's damage) and the sounds.

use super::*;

/// What a click on a wolf or cat asks of the pet.
struct Pet {
    tame: bool,
    owned: bool,
    hurt: bool,
    baby: bool,
    armored: bool,
}

impl Context<'_> {
    pub(super) fn interact_wolf(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let Some(pet) = self.pet_state(id) else {
            return EntityUse::Pass;
        };
        let food = "minecraft:wolf_food";
        if !pet.tame {
            return self.feed_animal(id, hand, food);
        }
        if pet.hurt && self.holds_tagged(hand, food) {
            self.use_up(hand);
            return EntityUse::Success { swing: true };
        }
        if let Some(answer) = self.wolf_dye_or_armor(id, hand, &pet) {
            return answer;
        }
        owner_sits(pet.owned, self.feed_animal(id, hand, food))
    }

    /// A tame wolf's collar dye (an owner's, of another colour) or body armor (an adult owner's
    /// wolf without any); the dye of the collar's own colour ends in the plain feeding answer.
    fn wolf_dye_or_armor(&mut self, id: EntityId, hand: u8, pet: &Pet) -> Option<EntityUse> {
        let dye = self
            .held_dye(hand)
            .filter(|_| pet.owned && self.holds_tagged(hand, "minecraft:wolf_collar_dyes"));
        if let Some(dye) = dye {
            if i32::from(dye) == self.collar_of(id) {
                return Some(self.feed_animal(id, hand, "minecraft:wolf_food"));
            }
        } else if !(self.holds_wolf_armor(hand) && !pet.armored && pet.owned && !pet.baby) {
            return None;
        }
        self.use_up(hand);
        Some(EntityUse::Success { swing: true })
    }

    pub(super) fn interact_cat(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let Some(pet) = self.pet_state(id) else {
            return EntityUse::Pass;
        };
        let food = "minecraft:cat_food";
        let fed = self.holds_tagged(hand, food);
        if !pet.tame && fed {
            return EntityUse::Success { swing: true };
        }
        if pet.tame && pet.owned {
            if self.cat_dye_or_heal(id, hand, &pet, fed) {
                return EntityUse::Success { swing: true };
            }
            return owner_sits(true, self.feed_animal(id, hand, food));
        }
        self.feed_animal(id, hand, food)
    }

    /// An owner's cat takes a collar dye of another colour, or food while hurt (the server
    /// uses the item up).
    fn cat_dye_or_heal(&self, id: EntityId, hand: u8, pet: &Pet, fed: bool) -> bool {
        let dye = self
            .held_dye(hand)
            .filter(|_| self.holds_tagged(hand, "minecraft:cat_collar_dyes"));
        match dye {
            Some(dye) => i32::from(dye) != self.collar_of(id),
            None => fed && pet.hurt,
        }
    }

    /// What the pet's data says: tame, owned by the local player, hurt, a baby, in body armor.
    fn pet_state(&self, id: EntityId) -> Option<Pet> {
        let entity = self.world.entities().at(id)?;
        Some(Pet {
            tame: entity.is_tame(),
            owned: entity.owner().is_some() && entity.owner() == self.profile_id,
            hurt: matches!((entity.health(), entity.max_health()), (Some(h), Some(m)) if h < m),
            baby: entity.is_baby(),
            armored: entity
                .equipment(photon_world::EquipmentSlot::Body)
                .is_some(),
        })
    }

    fn collar_of(&self, id: EntityId) -> i32 {
        self.world
            .entities()
            .at(id)
            .map_or(14, |entity| entity.collar_dye())
    }

    /// `usePlayerItem` / `consume(1, player)`: one is used up unless materials are infinite.
    fn use_up(&mut self, hand: u8) {
        if !self.instabuild() {
            let slot = self.hand_slot(hand);
            self.world.shrink_local_player_slot(slot, 1);
        }
    }

    /// Whether the held item is an equippable for the wolf's body (`Wolf.isEquippableInSlot`).
    fn holds_wolf_armor(&self, hand: u8) -> bool {
        self.held(hand)
            .and_then(|(item, _)| photon_world::block::rules::item(item)?.equippable)
            .is_some_and(|equip| equip.slot == "body" && equip.allowed.contains(&"minecraft:wolf"))
    }
}

/// An owner's click that nothing else answered toggles the sitting pose (a swing).
fn owner_sits(owned: bool, inner: EntityUse) -> EntityUse {
    if inner == EntityUse::Pass && owned {
        EntityUse::Success { swing: true }
    } else {
        inner
    }
}
