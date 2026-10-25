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
        let Some((saddled, rider)) = self.saddle_state(id) else {
            return EntityUse::Pass;
        };
        let food = self.holds_tagged(hand, tag);
        if !food && saddled && !rider && !self.player.is_shift_key_down() {
            return EntityUse::Success { swing: true };
        }
        self.feed_animal(id, hand, tag)
    }

    /// Whether the mob wears a saddle and has a passenger.
    fn saddle_state(&self, id: EntityId) -> Option<(bool, bool)> {
        let entity = self.world.entities().at(id)?;
        let saddled = entity
            .equipment(photon_world::EquipmentSlot::Saddle)
            .is_some();
        let rider = self
            .world
            .entity_passengers(id)
            .is_some_and(|passengers| !passengers.is_empty());
        Some((saddled, rider))
    }

    /// `Equippable.equipOnTarget` through `Item.interactLivingEntity`: an item that equips on
    /// interact goes on a living mob that may wear it and has nothing in that slot (the server
    /// does the equipping).
    pub(super) fn equips_on_interact(&self, id: EntityId, hand: u8) -> bool {
        let Some((item, _)) = self.held(hand) else {
            return false;
        };
        let Some(equip) = photon_world::block::rules::item(item).and_then(|row| row.equippable)
        else {
            return false;
        };
        let (Some(name), Some(entity)) = (
            self.world.entity_type_name(id),
            self.world.entities().at(id),
        ) else {
            return false;
        };
        let Some(slot) = slot_named(equip.slot) else {
            return false;
        };
        equip.equip_on_interact
            && equip.allowed.contains(&name)
            && can_use_slot(name, slot, entity.is_baby(), entity.is_tame())
            && entity.equipment(slot).is_none()
    }
}

fn slot_named(name: &str) -> Option<photon_world::EquipmentSlot> {
    use photon_world::EquipmentSlot as Slot;
    Some(match name {
        "saddle" => Slot::Saddle,
        "body" => Slot::Body,
        "head" => Slot::Head,
        "chest" => Slot::Chest,
        "legs" => Slot::Legs,
        "feet" => Slot::Feet,
        _ => return None,
    })
}

/// `LivingEntity.canUseSlot` for the mobs that restrict a slot to adults. A camel is always
/// tame (`Camel.isTamed`); a nautilus' saddle and body slots need an adult that is tame; the horse
/// family never reaches here (its own `mobInteract` answers first).
fn can_use_slot(name: &str, slot: photon_world::EquipmentSlot, baby: bool, tame: bool) -> bool {
    use photon_world::EquipmentSlot as Slot;
    match (name, slot) {
        (
            "minecraft:pig" | "minecraft:strider" | "minecraft:camel" | "minecraft:camel_husk",
            Slot::Saddle,
        ) => !baby,
        ("minecraft:happy_ghast" | "minecraft:sulfur_cube", Slot::Body) => !baby,
        ("minecraft:nautilus" | "minecraft:zombie_nautilus", Slot::Saddle | Slot::Body) => {
            !baby && tame
        }
        _ => true,
    }
}
