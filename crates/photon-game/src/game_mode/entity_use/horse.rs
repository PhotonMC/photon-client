//! The horse family's part of a click on the client: `Horse`, `Donkey`, `Mule`, `Llama`,
//! `ZombieHorse` and `SkeletonHorse` over `AbstractHorse.mobInteract`. Camels have their own.
//!
//! Not covered: body armor and saddles equipped by the click, the riding state the click clears (eating, rearing) and a horse that has a rider.

use super::*;
use photon_core::ids::ItemId;
use photon_data::item::ids as items;

/// The classes whose `mobInteract` sends a baby holding a golden dandelion straight to the
/// animal rules; `ZombieHorse` has no such bypass and `SkeletonHorse` no class layer at all.
fn has_dandelion_bypass(name: &str) -> bool {
    !matches!(name, "minecraft:zombie_horse" | "minecraft:skeleton_horse")
}

/// The horse family this module answers for (camels have their own rule).
pub(super) fn is_equine(name: &str) -> bool {
    is_chested(name)
        || matches!(
            name,
            "minecraft:horse" | "minecraft:zombie_horse" | "minecraft:skeleton_horse"
        )
}

const LLAMA_FOOD: &str = "minecraft:llama_food";

/// `isFood`'s tag for the class.
fn food_tag(name: &str) -> &'static str {
    match name {
        "minecraft:zombie_horse" => "minecraft:zombie_horse_food",
        "minecraft:llama" | "minecraft:trader_llama" => LLAMA_FOOD,
        _ => "minecraft:horse_food",
    }
}

/// `AbstractChestedHorse`: the donkeys, mules and llamas.
fn is_chested(name: &str) -> bool {
    matches!(
        name,
        "minecraft:donkey" | "minecraft:mule" | "minecraft:llama" | "minecraft:trader_llama"
    )
}

impl Context<'_> {
    /// The class layer of `mobInteract` (food, makeMad, the chest), then `AbstractHorse`'s.
    pub(super) fn interact_horse(&mut self, name: &str, id: EntityId, hand: u8) -> EntityUse {
        let Some(entity) = self.world.entities().at(id) else {
            return EntityUse::Pass;
        };
        let baby = entity.is_baby();
        let tamed = entity.horse().is_some_and(|horse| horse.is_tamed());
        let ridden = self
            .world
            .entity_passengers(id)
            .is_some_and(|riders| !riders.is_empty());
        if name == "minecraft:skeleton_horse" {
            // `SkeletonHorse.mobInteract`: no class layer, and an untamed one passes.
            return if tamed {
                self.abstract_horse_layer(name, id, hand, ridden || baby)
            } else {
                EntityUse::Pass
            };
        }
        if let Some(answer) = self.horse_class_layer(name, id, hand, (baby, tamed, ridden)) {
            return answer;
        }
        self.abstract_horse_layer(name, id, hand, ridden || baby)
    }

    /// `AbstractHorse.mobInteract` after the sneaking owner's inventory (which also succeeds
    /// with a swing): a ridden horse or a baby follows the animal rules, a grown one is ridden.
    fn abstract_horse_layer(
        &mut self,
        name: &str,
        id: EntityId,
        hand: u8,
        animal_rules: bool,
    ) -> EntityUse {
        if animal_rules {
            self.feed_animal(id, hand, food_tag(name))
        } else {
            EntityUse::Success { swing: true }
        }
    }

    /// What a held item does before `AbstractHorse.mobInteract`, for the facts (baby, tamed,
    /// ridden): food is eaten (the client answers SUCCESS_SERVER, no swing), an untamed horse is
    /// angered (a swing), a chest goes on a tamed chested horse. Nothing happens for a ridden
    /// horse, the owner's sneak or a baby the golden dandelion is offered to.
    fn horse_class_layer(
        &mut self,
        name: &str,
        id: EntityId,
        hand: u8,
        (baby, tamed, ridden): (bool, bool, bool),
    ) -> Option<EntityUse> {
        let opens_inventory = !baby && tamed && self.player.is_shift_key_down();
        let bypass = has_dandelion_bypass(name) && baby && self.holds_golden_dandelion();
        if ridden || opens_inventory || bypass {
            return None;
        }
        let (item, _) = self.held(hand)?;
        let food = self
            .world
            .block_state_registry()
            .item_tags()
            .contains(item.0, food_tag(name));
        if food {
            self.eat_to_heal(name, id, hand);
            return Some(EntityUse::Success { swing: false });
        }
        if !tamed {
            return Some(EntityUse::Success { swing: true });
        }
        self.equip_chest(name, id, hand, item)
            .then_some(EntityUse::Success { swing: true })
    }

    /// `AbstractHorse.handleEating`, which runs on the client too: a hurt animal is healed by the
    /// item and one is used up. Temper, age and love are the server's.
    fn eat_to_heal(&mut self, name: &str, id: EntityId, hand: u8) {
        let Some((item, _)) = self.held(hand) else {
            return;
        };
        if self.heal_local(id, food_heal(name, item)) {
            self.use_up(hand);
        }
    }

    /// `LivingEntity.heal` on the client's copy of an animal that is hurt; false when it is not,
    /// or the amount heals nothing.
    pub(super) fn heal_local(&mut self, id: EntityId, heal: f32) -> bool {
        let Some(entity) = self.world.entities().at(id) else {
            return false;
        };
        let (Some(health), Some(max)) = (entity.health(), entity.max_health()) else {
            return false;
        };
        if heal <= 0.0 || health >= max {
            return false;
        }
        let mut data = vec![photon_world::ENTITY_HEALTH_DATA_ID, 3];
        data.extend_from_slice(&(health + heal).min(max).to_be_bytes());
        data.push(255);
        self.world.apply_entity_metadata(id, &data);
        true
    }

    /// `AbstractChestedHorse.equipChest`, which runs on the client too: the chest appears and
    /// one is used up. False when the animal or the item is not eligible.
    fn equip_chest(&mut self, name: &str, id: EntityId, hand: u8, item: ItemId) -> bool {
        let bare = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| !entity.horse_has_chest());
        if !(is_chested(name) && bare && crate::item::id_of("minecraft:chest") == Some(item)) {
            return false;
        }
        self.world.entities_mut().set_horse_chest(id, true);
        if !self.instabuild() {
            let slot = self.hand_slot(hand);
            self.world.shrink_local_player_slot(slot, 1);
        }
        true
    }

    /// `Player.isHolding(GOLDEN_DANDELION)`: either hand.
    pub(super) fn holds_golden_dandelion(&self) -> bool {
        let golden = crate::item::id_of("minecraft:golden_dandelion");
        [MAIN_HAND, OFF_HAND].into_iter().any(|hand| {
            self.held(hand)
                .is_some_and(|(item, _)| golden == Some(item))
        })
    }
}

/// The health a food item heals: the llama's own table, or `AbstractHorse`'s for the rest.
fn food_heal(name: &str, item: ItemId) -> f32 {
    if food_tag(name) == LLAMA_FOOD {
        return match item {
            items::WHEAT => 3.0,
            items::HAY_BLOCK => 6.0,
            _ => 0.0,
        };
    }
    match item {
        items::WHEAT => 2.0,
        items::SUGAR => 1.0,
        items::HAY_BLOCK => 20.0,
        items::APPLE | items::CARROT => 3.0,
        items::GOLDEN_CARROT => 4.0,
        items::GOLDEN_APPLE | items::ENCHANTED_GOLDEN_APPLE => 10.0,
        _ => 0.0,
    }
}
