//! `Pig.mobInteract` and `Strider.mobInteract`: a saddled mount is boarded (a swing, the server
//! does the rest), food feeds it, and a saddle in hand goes on through the item.
//!
//! Not covered: what boarding clears locally. (The strider's eating sound is played with no
//! excluded player, which a client never plays.)

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

    /// `Mob.attemptToShearEquipment` through `Entity.interact`, with shears in hand: a mob that
    /// is not ridden (a wolf: only its owner's) and whose player is not sneaking loses the first
    /// shearable thing it wears, saddle and body armor alike. The shears' wear is the server's.
    pub(super) fn shear_equipment(&mut self, id: EntityId) -> bool {
        use photon_world::EquipmentSlot::{Body, Saddle};
        let ridden = self
            .world
            .entity_passengers(id)
            .is_some_and(|riders| !riders.is_empty());
        let Some(entity) = self.world.entities().at(id) else {
            return false;
        };
        let owned = entity.owner().is_some() && entity.owner() == self.profile_id;
        if ridden || self.player.is_shift_key_down() || (entity.is_wolf() && !owned) {
            return false;
        }
        // The slots in `EquipmentSlot.VALUES` order: the body before the saddle.
        let Some(slot) = [Body, Saddle]
            .into_iter()
            .find(|&slot| entity.equipment(slot).is_some_and(shearable))
        else {
            return false;
        };
        self.world.entities_mut().apply_equipment(id, slot, None);
        true
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
        self.equippable_by_click(id, hand, true)
    }

    /// Whether the held item fits an empty slot of the mob it is used on: one that equips itself
    /// on the click (a saddle) with `on_interact`, otherwise body armor (horse armor).
    pub(super) fn equippable_by_click(&self, id: EntityId, hand: u8, on_interact: bool) -> bool {
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
        (if on_interact {
            equip.equip_on_interact
        } else {
            slot == photon_world::EquipmentSlot::Body
        }) && equip.allowed.contains(&name)
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

/// `Equippable.canBeSheared`: the saddle, harnesses, wolf, horse and nautilus armor and a llama's
/// carpet, which are the equippables of the saddle and body slots.
fn shearable(worn: &photon_world::ClientItemStackState) -> bool {
    photon_world::block::rules::item(worn.item_registry_id())
        .and_then(|row| row.equippable)
        .is_some_and(|equip| matches!(equip.slot, "saddle" | "body"))
}
