//! What an entity makes of a click on the client's side of `Entity.interact`.
//!
//! The server decides the real outcome; the client's own answer only decides whether the click
//! goes on to the held item and whether the hand swings. A class whose rule is not written here
//! answers Pass, as `Entity.interact` does on the client for a class without an override, so a
//! class with an override that is missing is listed as open on the issue.

use super::*;
use photon_core::ids::EntityId;

/// The client's answer to a click on an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EntityUse {
    /// `InteractionResult.PASS`, or a fail: the click goes on to the held item.
    Pass,
    /// A success; `swing` is whether the client swings the hand itself.
    Success { swing: bool },
}

impl Context<'_> {
    /// `Player.interactOn` for a player who is not a spectator, on the client.
    pub(super) fn interact_entity(&self, id: EntityId, hand: u8) -> EntityUse {
        if self.spawn_egg_on_living_mob(id, hand) {
            return EntityUse::Success { swing: false };
        }
        if let Some(answer) = self.lead_on_mob(id, hand) {
            return answer;
        }
        match self.interact_entity_side(id, hand) {
            EntityUse::Pass => self.item_interacts_with_living(id, hand),
            success => success,
        }
    }

    /// `Entity.interact` of the clicked class.
    fn interact_entity_side(&self, id: EntityId, hand: u8) -> EntityUse {
        let Some(name) = self.world.entity_type_name(id) else {
            return EntityUse::Pass;
        };
        let secondary = self.player.is_shift_key_down();
        match name {
            // `Minecart.interact`: a rider gets on unless sneaking or the cart is taken.
            "minecraft:minecart" => {
                let taken = self
                    .world
                    .entity_passengers(id)
                    .is_some_and(|riders| !riders.is_empty());
                if secondary || taken {
                    EntityUse::Pass
                } else {
                    EntityUse::Success { swing: true }
                }
            }
            // `AbstractMinecartContainer.interact` and `MinecartFurnace.interact` always succeed.
            "minecraft:chest_minecart"
            | "minecraft:hopper_minecart"
            | "minecraft:furnace_minecart" => EntityUse::Success { swing: true },
            "minecraft:armor_stand" => self.interact_armor_stand(id, hand),
            "minecraft:item_frame" | "minecraft:glow_item_frame" => {
                self.interact_item_frame(id, hand)
            }
            _ => match animal_food_tag(name) {
                Some(tag) => self.feed_animal(id, hand, tag),
                None => EntityUse::Pass,
            },
        }
    }

    /// `ArmorStand.interact`: the server does the work, unless the click is a marker's or a name
    /// tag's, which `Entity.interact` answers (nothing, on the client).
    fn interact_armor_stand(&self, id: EntityId, hand: u8) -> EntityUse {
        let marker = self
            .world
            .entity_click_view(id)
            .is_some_and(|view| view.marker);
        if marker || self.holds_name_tag(hand) {
            EntityUse::Pass
        } else {
            EntityUse::Success { swing: false }
        }
    }

    /// `ItemFrame.interact` on the client: an empty frame and an empty hand do nothing. (A fixed
    /// frame passes too, but the client is not told the flag.)
    fn interact_item_frame(&self, id: EntityId, hand: u8) -> EntityUse {
        let filled = self
            .world
            .entity_click_view(id)
            .is_some_and(|view| view.frame_has_item);
        if filled || self.held(hand).is_some() {
            EntityUse::Success { swing: true }
        } else {
            EntityUse::Pass
        }
    }

    /// `Animal.mobInteract` on the client, for the animals that add nothing to it: their food
    /// either ages a baby up (a success) or is consumed with no swing; any other item passes.
    /// (A golden dandelion on a baby and a lead on a leashed animal come first in the
    /// reference and are not covered.)
    fn feed_animal(&self, id: EntityId, hand: u8, tag: &str) -> EntityUse {
        let Some(entity) = self.world.entities().at(id) else {
            return EntityUse::Pass;
        };
        let food = self.held(hand).is_some_and(|(item, _)| {
            self.world
                .block_state_registry()
                .item_tags()
                .contains(item.0, tag)
        });
        if !food || entity.health().is_some_and(|health| health <= 0.0) {
            EntityUse::Pass
        } else if entity.is_baby() {
            EntityUse::Success { swing: true }
        } else {
            EntityUse::Success { swing: false }
        }
    }

    /// The end of `Entity.interact` for a `Leashable` that lives, which `Mob.interact` runs
    /// before the class's own rule: a lead in hand, on a mob no player holds, is consumed on the
    /// client (the server ties it). Taking back a lead the player holds (a swing) and shears
    /// are not covered.
    fn lead_on_mob(&self, id: EntityId, hand: u8) -> Option<EntityUse> {
        let entity = self.world.entities().at(id)?;
        if !entity.is_mob() || entity.health().is_some_and(|health| health <= 0.0) {
            return None;
        }
        let holder_is_player = entity
            .leash_holder_id()
            .is_some_and(|holder| self.world.entity_type_name(holder) == Some("minecraft:player"));
        let lead = self
            .held(hand)
            .is_some_and(|(item, _)| crate::item::id_of("minecraft:lead") == Some(item));
        (lead && !holder_is_player).then_some(EntityUse::Success { swing: false })
    }

    /// `Mob.checkAndHandleImportantInteractions` for a spawn egg: the server spawns the
    /// offspring, the client's answer is the server's success (no swing) for a mob that lives.
    fn spawn_egg_on_living_mob(&self, id: EntityId, hand: u8) -> bool {
        let Some(entity) = self.world.entities().at(id) else {
            return false;
        };
        let alive = entity.health().is_none_or(|health| health > 0.0);
        let egg = self.held(hand).is_some_and(|(item, _)| {
            photon_world::block::rules::item(item)
                .is_some_and(|row| row.class == photon_data::item::classes::ItemClass::SpawnEggItem)
        });
        entity.is_mob() && alive && egg
    }

    /// `ItemStack.interactLivingEntity` on the client, for what the stack alone decides: a name
    /// tag that carries a name succeeds on any living entity that is not a player.
    fn item_interacts_with_living(&self, id: EntityId, hand: u8) -> EntityUse {
        let living = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| entity.is_living_entity());
        let player = self.world.entity_type_name(id) == Some("minecraft:player");
        if living && !player && self.holds_named_name_tag(hand) {
            EntityUse::Success { swing: true }
        } else {
            EntityUse::Pass
        }
    }

    /// `NameTagItem.interactLivingEntity`'s condition on the stack: a name tag with a name.
    fn holds_named_name_tag(&self, hand: u8) -> bool {
        let Some(stack) = self
            .world
            .local_player()
            .and_then(|state| state.inventory_slot(self.hand_slot(hand)))
        else {
            return false;
        };
        let item = stack.item_registry_id();
        crate::item::id_of("minecraft:name_tag") == Some(item)
            && stack.item_count() > 0
            && crate::item::values::Values::new(item, stack.patch_bytes())
                .custom_name()
                .is_some()
    }

    fn holds_name_tag(&self, hand: u8) -> bool {
        self.held(hand)
            .is_some_and(|(item, _)| crate::item::id_of("minecraft:name_tag") == Some(item))
    }
}

/// The item tag an animal's `isFood` asks for, for the animals whose class has no `mobInteract`
/// of its own and so answers with `Animal.mobInteract`.
fn animal_food_tag(name: &str) -> Option<&'static str> {
    Some(match name {
        "minecraft:chicken" => "minecraft:chicken_food",
        "minecraft:rabbit" => "minecraft:rabbit_food",
        "minecraft:turtle" => "minecraft:turtle_food",
        "minecraft:frog" => "minecraft:frog_food",
        "minecraft:fox" => "minecraft:fox_food",
        _ => return None,
    })
}
