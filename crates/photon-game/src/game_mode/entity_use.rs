//! What an entity makes of a click on the client's side of `Entity.interact`.
//!
//! The server decides the real outcome; the client's own answer only decides whether the click
//! goes on to the held item and whether the hand swings. A class whose rule is not written here
//! answers Pass, as `Entity.interact` does on the client for a class without an override, so a
//! class with an override that is missing is listed as open on the issue.

use super::*;
use photon_core::ids::EntityId;

mod camel;
mod horse;
mod pickups;
mod saddles;
mod sheep;
mod villager;

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
    pub(super) fn interact_entity(&mut self, id: EntityId, hand: u8) -> EntityUse {
        if self.named_tag_on_living_mob(id, hand) {
            return EntityUse::Success { swing: true };
        }
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
    fn interact_entity_side(&mut self, id: EntityId, hand: u8) -> EntityUse {
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
            _ => self.interact_mob_class(name, id, hand),
        }
    }

    /// `Mob.mobInteract` of the clicked class.
    fn interact_mob_class(&mut self, name: &str, id: EntityId, hand: u8) -> EntityUse {
        // `Mob.interact` passes for a mob that is not alive.
        let dead = self
            .world
            .entities()
            .at(id)
            .is_none_or(|entity| entity.health().is_some_and(|health| health <= 0.0));
        if dead {
            return EntityUse::Pass;
        }
        match name {
            "minecraft:creeper" => self.interact_creeper(id, hand),
            "minecraft:snow_golem" => self.interact_snow_golem(id, hand),
            "minecraft:villager" | "minecraft:wandering_trader" | "minecraft:zombie_villager" => {
                self.interact_villager_class(name, id, hand)
            }
            n if horse::is_equine(n) => self.interact_horse(name, id, hand),
            "minecraft:camel" | "minecraft:camel_husk" => self.interact_camel(name, id, hand),
            n if pickups::bucket_of(n).is_some() => self.interact_bucketable(name, id, hand),
            _ => self.interact_herd_class(name, id, hand),
        }
    }

    /// The cattle, goats, sheep and the animals that only take their food.
    fn interact_herd_class(&mut self, name: &str, id: EntityId, hand: u8) -> EntityUse {
        match name {
            "minecraft:cow" => self.interact_cow(id, hand),
            "minecraft:goat" => self.interact_goat(id, hand),
            "minecraft:mooshroom" => self.interact_mooshroom(id, hand),
            "minecraft:sheep" => self.interact_sheep(id, hand),
            "minecraft:pig" | "minecraft:strider" => self.interact_saddleable(name, id, hand),
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
    /// (A lead on a leashed animal comes first in the reference and is not covered.)
    fn feed_animal(&mut self, id: EntityId, hand: u8, tag: &str) -> EntityUse {
        let Some(entity) = self.world.entities().at(id) else {
            return EntityUse::Pass;
        };
        // `canAgeUp` is false for the undead horses.
        let ages = !matches!(
            self.world.entity_type_name(id),
            Some("minecraft:zombie_horse" | "minecraft:skeleton_horse")
        );
        let (baby, locked) = (entity.is_baby() && ages, entity.is_age_locked());
        let dead = entity.health().is_some_and(|health| health <= 0.0);
        let food = self.held(hand).is_some_and(|(item, _)| {
            self.world
                .block_state_registry()
                .item_tags()
                .contains(item.0, tag)
        });
        if dead {
            EntityUse::Pass
        } else if !food {
            // `AgeableMob.mobInteract`, which `Animal.mobInteract` ends in.
            self.golden_dandelion_on_baby(id, hand)
        } else if baby && !locked {
            // `usePlayerItem` takes one unless materials are infinite.
            if !self.instabuild() {
                let slot = self.hand_slot(hand);
                self.world.shrink_local_player_slot(slot, 1);
            }
            EntityUse::Success { swing: true }
        } else {
            EntityUse::Success { swing: false }
        }
    }

    /// `AgeableMob.mobInteract`: a golden dandelion on a baby, when the last use is over and the
    /// type may be locked, flips the age lock and is used up (the sound excludes nobody, so the
    /// client plays none, and the particles are not compared).
    fn golden_dandelion_on_baby(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let Some(entity) = self.world.entities().at(id) else {
            return EntityUse::Pass;
        };
        let ready = entity.is_baby() && entity.age_lock_timer() == 0;
        let barred = self
            .world
            .block_state_registry()
            .entity_type_tags()
            .contains(entity.type_id.0, "minecraft:cannot_be_age_locked");
        let golden = self.held(hand).is_some_and(|(item, _)| {
            crate::item::id_of("minecraft:golden_dandelion") == Some(item)
        });
        if !(ready && golden && !barred) {
            return EntityUse::Pass;
        }
        self.world.entities_mut().toggle_age_lock(id);
        if !self.instabuild() {
            let slot = self.hand_slot(hand);
            self.world.shrink_local_player_slot(slot, 1);
        }
        EntityUse::Success { swing: true }
    }

    /// `Creeper.mobInteract`: a flint and steel or fire charge lights the fuse, which the client
    /// answers with a swing and the sound of the igniter, its pitch drawn from a source that runs
    /// on from click to click, as the creeper's own does.
    fn interact_creeper(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let Some(entity) = self.world.entities().at(id) else {
            return EntityUse::Pass;
        };
        let alive = entity.health().is_none_or(|health| health > 0.0);
        let at = entity.position();
        let igniter = self.held(hand).filter(|(item, _)| {
            self.world
                .block_state_registry()
                .item_tags()
                .contains(item.0, "minecraft:creeper_igniters")
        });
        let Some((item, _)) = igniter.filter(|_| alive) else {
            return EntityUse::Pass;
        };
        let event = if crate::item::id_of("minecraft:fire_charge") == Some(item) {
            "minecraft:item.firecharge.use"
        } else {
            "minecraft:item.flintandsteel.use"
        };
        let pitch = self.world.entity_sound_pitch();
        self.world
            .push_sound(photon_world::local_player::LocalSound::at(
                event,
                photon_world::local_player::LocalSoundSource::Hostile,
                (1.0, pitch),
                at,
            ));
        EntityUse::Success { swing: true }
    }

    /// `AbstractCow.mobInteract`: a bucket on a grown cow is filled with milk on the client too,
    /// with the milking sound; the rest is `Animal.mobInteract`.
    fn interact_cow(&mut self, id: EntityId, hand: u8) -> EntityUse {
        self.milk_or_feed(id, hand, "minecraft:entity.cow.milk", "minecraft:cow_food")
    }

    /// `Goat.mobInteract`: the same with the goat's milking sound, which a screaming goat changes.
    /// (Its eating sound reaches no one: it excludes nobody, which the client does not play.)
    fn interact_goat(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let screaming = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| entity.is_screaming_goat());
        let sound = if screaming {
            "minecraft:entity.goat.screaming.milk"
        } else {
            "minecraft:entity.goat.milk"
        };
        self.milk_or_feed(id, hand, sound, "minecraft:goat_food")
    }

    /// A bucket on a grown animal is milked with `sound`; anything else is fed with `food`.
    fn milk_or_feed(
        &mut self,
        id: EntityId,
        hand: u8,
        sound: &'static str,
        food: &str,
    ) -> EntityUse {
        let grown = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| !entity.is_baby());
        let bucket = self
            .held(hand)
            .is_some_and(|(item, _)| crate::item::id_of("minecraft:bucket") == Some(item));
        if !(grown && bucket) {
            // `Animal.mobInteract`.
            return self.feed_animal(id, hand, food);
        }
        let at = self.player.position();
        self.world
            .push_sound(photon_world::local_player::LocalSound::at(
                sound,
                photon_world::local_player::LocalSoundSource::Players,
                (1.0, 1.0),
                (at.x, at.y, at.z),
            ));
        super::buckets::milk_result(self, hand);
        EntityUse::Success { swing: true }
    }

    /// `MushroomCow.mobInteract`: a bowl on an adult is filled with stew (suspicious when a
    /// flower was fed to it; no sound reaches the client: the mooshroom plays it with no player
    /// excluded), shears on an adult swing, a flower on a brown adult is eaten (used up unless
    /// materials are infinite) and sets the stew effects; the rest is a cow's.
    fn interact_mooshroom(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let Some(entity) = self.world.entities().at(id) else {
            return EntityUse::Pass;
        };
        let (grown, brown, stewed) = (
            !entity.is_baby(),
            entity.mooshroom_variant() == 1,
            entity.mooshroom_has_stew_effects(),
        );
        let held = |name: &str| {
            self.held(hand)
                .is_some_and(|(item, _)| crate::item::id_of(name) == Some(item))
        };
        if grown && held("minecraft:bowl") {
            self.fill_bowl_from_mooshroom(id, hand, stewed);
            EntityUse::Success { swing: true }
        } else if grown && held("minecraft:shears") {
            EntityUse::Success { swing: true }
        } else if grown && brown && self.holds_suspicious_effect_item(hand) {
            self.eat_flower_as_mooshroom(id, hand, stewed);
            EntityUse::Success { swing: true }
        } else {
            self.interact_cow(id, hand)
        }
    }

    /// The bowl branch: stew, suspicious when the mooshroom holds a flower's effects, which the
    /// bowl then clears.
    fn fill_bowl_from_mooshroom(&mut self, id: EntityId, hand: u8, stewed: bool) {
        let stew = if stewed {
            "minecraft:suspicious_stew"
        } else {
            "minecraft:mushroom_stew"
        };
        self.world.entities_mut().set_mooshroom_stew(id, false);
        super::buckets::stew_result(self, hand, stew);
    }

    /// The flower branch: eaten (used up unless materials are infinite) when the mooshroom holds
    /// no effects yet, which it then does.
    fn eat_flower_as_mooshroom(&mut self, id: EntityId, hand: u8, stewed: bool) {
        if stewed {
            return;
        }
        if !self.instabuild() {
            let slot = self.hand_slot(hand);
            self.world.shrink_local_player_slot(slot, 1);
        }
        self.world.entities_mut().set_mooshroom_stew(id, true);
    }

    /// `SuspiciousEffectHolder.tryGet` of the held item: the block items of `FlowerBlock` and its
    /// two subclasses (the wither rose and the eyeblossoms). A type check in the reference, not a
    /// tag, so the server's tags do not change it.
    fn holds_suspicious_effect_item(&self, hand: u8) -> bool {
        const HOLDERS: [&str; 17] = [
            "minecraft:dandelion",
            "minecraft:golden_dandelion",
            "minecraft:torchflower",
            "minecraft:poppy",
            "minecraft:blue_orchid",
            "minecraft:allium",
            "minecraft:azure_bluet",
            "minecraft:red_tulip",
            "minecraft:orange_tulip",
            "minecraft:white_tulip",
            "minecraft:pink_tulip",
            "minecraft:oxeye_daisy",
            "minecraft:cornflower",
            "minecraft:wither_rose",
            "minecraft:lily_of_the_valley",
            "minecraft:open_eyeblossom",
            "minecraft:closed_eyeblossom",
        ];
        self.held(hand).is_some_and(|(item, _)| {
            HOLDERS
                .iter()
                .any(|name| crate::item::id_of(name) == Some(item))
        })
    }

    /// `SnowGolem.mobInteract`: shears on a golem that wears its pumpkin succeed with a swing (the
    /// server shears it).
    fn interact_snow_golem(&self, id: EntityId, hand: u8) -> EntityUse {
        let wears = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| entity.has_pumpkin());
        let shears = self
            .held(hand)
            .is_some_and(|(item, _)| crate::item::id_of("minecraft:shears") == Some(item));
        if wears && shears {
            EntityUse::Success { swing: true }
        } else {
            EntityUse::Pass
        }
    }

    /// The end of `Entity.interact` for a `Leashable` that lives, which `Mob.interact` runs
    /// before the class's own rule: taking back the lead the player holds succeeds with a swing;
    /// a lead in hand, on a mob no player holds, is consumed on the client (the server ties it).
    /// Shears on a leashed mob are not covered.
    fn lead_on_mob(&self, id: EntityId, hand: u8) -> Option<EntityUse> {
        let entity = self.world.entities().at(id)?;
        if !entity.is_mob() || entity.health().is_some_and(|health| health <= 0.0) {
            return None;
        }
        let me = self.player.entity_id();
        let holder = entity.leash_holder_id();
        // `Entity.interact`: shears cut a lead the mob is tied by, whoever holds it.
        let shears = self
            .held(hand)
            .is_some_and(|(item, _)| crate::item::id_of("minecraft:shears") == Some(item));
        if holder == Some(me) || (shears && holder.is_some()) {
            return Some(EntityUse::Success { swing: true });
        }
        let holder_is_player = holder
            .is_some_and(|holder| self.world.entity_type_name(holder) == Some("minecraft:player"));
        let lead = self
            .held(hand)
            .is_some_and(|(item, _)| crate::item::id_of("minecraft:lead") == Some(item));
        (lead && !holder_is_player).then_some(EntityUse::Success { swing: false })
    }

    /// `Mob.checkAndHandleImportantInteractions` for a name tag with a name, which comes before
    /// the lead and the class's own rule. (Other living entities reach it through the item.)
    fn named_tag_on_living_mob(&self, id: EntityId, hand: u8) -> bool {
        let alive = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| entity.is_mob() && entity.health().is_none_or(|h| h > 0.0));
        alive && self.holds_named_name_tag(hand)
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
    fn item_interacts_with_living(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let living = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| entity.is_living_entity());
        let player = self.world.entity_type_name(id) == Some("minecraft:player");
        let named = living && !player && self.holds_named_name_tag(hand);
        if named || (living && (self.dye_on_sheep(id, hand) || self.equips_on_interact(id, hand))) {
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
