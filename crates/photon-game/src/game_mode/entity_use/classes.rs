//! The `mobInteract` of the classes that answer one fixed way on the client: ocelot, dolphin, iron
//! golem, allay, bogged, piglin, panda, happy ghast and armadillo.
//!
//! Not covered: a panda that is scared or on its back, a trusting ocelot and its distance check, an allay
//! that is dancing (or whose hand shows a tick early), a sheared bogged, a scared armadillo, a piglin
//! whose brain refuses to admire, the iron golem's health after a repair and its attribute-based
//! maximum, and the sounds.

use super::*;
use photon_world::local_player::{LocalSound, LocalSoundSource};

pub(super) fn handles(name: &str) -> bool {
    matches!(
        name,
        "minecraft:ocelot"
            | "minecraft:dolphin"
            | "minecraft:iron_golem"
            | "minecraft:allay"
            | "minecraft:bogged"
            | "minecraft:piglin"
            | "minecraft:armadillo"
            | "minecraft:panda"
            | "minecraft:happy_ghast"
            | "minecraft:bee"
    )
}

impl Context<'_> {
    pub(super) fn interact_listed_class(
        &mut self,
        name: &str,
        id: EntityId,
        hand: u8,
    ) -> EntityUse {
        match name {
            "minecraft:ocelot" => self.interact_ocelot(id, hand),
            "minecraft:dolphin" => self.interact_dolphin(id, hand),
            "minecraft:iron_golem" => self.interact_iron_golem(id, hand),
            "minecraft:allay" => self.interact_allay(id, hand),
            "minecraft:bogged" => self.interact_bogged(hand),
            "minecraft:piglin" => self.interact_piglin(id, hand),
            "minecraft:panda" => self.interact_panda(id, hand),
            "minecraft:happy_ghast" => self.interact_happy_ghast(id, hand),
            "minecraft:bee" => self.interact_bee(id, hand),
            _ => self.interact_armadillo(id, hand),
        }
    }

    fn holds_item(&self, hand: u8, name: &str) -> bool {
        self.held(hand)
            .is_some_and(|(item, _)| crate::item::id_of(name) == Some(item))
    }

    /// `Ocelot.mobInteract`: food from an ocelot that is not yet trusting is used up.
    fn interact_ocelot(&mut self, id: EntityId, hand: u8) -> EntityUse {
        if self.holds_tagged(hand, "minecraft:ocelot_food") {
            self.use_up(hand);
            return EntityUse::Success { swing: true };
        }
        self.feed_animal(id, hand, "minecraft:ocelot_food")
    }

    /// `Dolphin.mobInteract`: any fish is eaten.
    fn interact_dolphin(&mut self, id: EntityId, hand: u8) -> EntityUse {
        if self.holds_tagged(hand, "minecraft:fishes") {
            self.use_up(hand);
            EntityUse::Success { swing: true }
        } else {
            self.golden_dandelion_on_baby(id, hand)
        }
    }

    /// `IronGolem.mobInteract`: an ingot repairs a golem that is hurt.
    fn interact_iron_golem(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let hurt = self
            .world
            .entities()
            .at(id)
            .and_then(|entity| entity.health())
            .is_some_and(|health| health < IRON_GOLEM_HEALTH);
        if hurt && self.holds_item(hand, "minecraft:iron_ingot") {
            self.use_up(hand);
            EntityUse::Success { swing: true }
        } else {
            EntityUse::Pass
        }
    }

    /// `Allay.mobInteract`: an empty-handed allay is given the item; the main hand takes it back.
    fn interact_allay(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let carries = self.world.entities().at(id).is_some_and(|entity| {
            entity
                .equipment(photon_world::EquipmentSlot::MainHand)
                .is_some()
        });
        let holds = self.held(hand).is_some();
        if !carries && holds {
            self.use_up(hand);
            self.allay_sound(id, "minecraft:entity.allay.item_given");
            EntityUse::Success { swing: true }
        } else if carries && hand == 0 && !holds {
            self.allay_sound(id, "minecraft:entity.allay.item_taken");
            EntityUse::Success { swing: true }
        } else {
            self.golden_dandelion_on_baby(id, hand)
        }
    }

    /// `level.playSound(player, allay, ..)`: the local player is the excluded one, so it hears it.
    fn allay_sound(&mut self, id: EntityId, event: &'static str) {
        let Some(at) = self.world.entities().at(id).map(|entity| entity.position()) else {
            return;
        };
        self.world.push_sound(LocalSound::at(
            event,
            LocalSoundSource::Neutral,
            (2.0, 1.0),
            at,
        ));
    }

    /// `Bogged.mobInteract`: shears work on a bogged that has not been sheared.
    fn interact_bogged(&self, hand: u8) -> EntityUse {
        if self.holds_item(hand, "minecraft:shears") {
            EntityUse::Success { swing: true }
        } else {
            EntityUse::Pass
        }
    }

    /// `Piglin.mobInteract` on the client: an adult admires the gold it is shown.
    fn interact_piglin(&self, id: EntityId, hand: u8) -> EntityUse {
        let Some(entity) = self.world.entities().at(id) else {
            return EntityUse::Pass;
        };
        // `getArmPose() != ADMIRING_ITEM`: a loved item in the off hand.
        let admiring = entity
            .equipment(photon_world::EquipmentSlot::OffHand)
            .is_some_and(|worn| {
                self.world
                    .block_state_registry()
                    .item_tags()
                    .contains(worn.item_registry_id().0, "minecraft:piglin_loved")
            });
        if !entity.is_baby() && !admiring && self.holds_item(hand, "minecraft:gold_ingot") {
            EntityUse::Success { swing: true }
        } else {
            EntityUse::Pass
        }
    }

    /// `Panda.mobInteract` on the client: bamboo feeds a baby that can still age (a success the
    /// server owns); an adult's bamboo and any other item pass.
    fn interact_panda(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let ages = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| entity.is_baby() && !entity.is_age_locked());
        if ages && self.holds_tagged(hand, "minecraft:panda_food") {
            self.use_up(hand);
            EntityUse::Success { swing: false }
        } else {
            self.golden_dandelion_on_baby(id, hand)
        }
    }

    /// `HappyGhast.mobInteract`: an adult tries the held item first (a harness goes on), and one
    /// wearing a harness is boarded unless the rider sneaks; otherwise it is `Animal`'s feeding.
    fn interact_happy_ghast(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let Some(entity) = self.world.entities().at(id) else {
            return EntityUse::Pass;
        };
        let baby = entity.is_baby();
        let harnessed = entity
            .equipment(photon_world::EquipmentSlot::Body)
            .and_then(|worn| photon_world::block::rules::item(worn.item_registry_id())?.equippable)
            .is_some_and(|equip| {
                equip.slot == "body" && equip.allowed.contains(&"minecraft:happy_ghast")
            });
        if !baby {
            if self.equips_on_interact(id, hand) {
                return EntityUse::Success { swing: true };
            }
            if harnessed && !self.player.is_shift_key_down() {
                return EntityUse::Success { swing: true };
            }
        }
        self.feed_animal(id, hand, "minecraft:happy_ghast_food")
    }

    /// `Bee.mobInteract`: a flower with a bee effect (the wither rose and the eyeblossoms) is
    /// used up; any other bee food is `Animal`'s feeding.
    fn interact_bee(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let effect = ["wither_rose", "open_eyeblossom", "closed_eyeblossom"]
            .iter()
            .any(|flower| self.holds_item(hand, &format!("minecraft:{flower}")));
        if effect {
            self.use_up(hand);
            return EntityUse::Success { swing: true };
        }
        self.feed_animal(id, hand, "minecraft:bee_food")
    }

    /// `Armadillo.mobInteract`: a brush takes a scute off an adult, otherwise its food.
    fn interact_armadillo(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let adult = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| !entity.is_baby());
        if adult && self.holds_item(hand, "minecraft:brush") {
            EntityUse::Success { swing: true }
        } else {
            self.feed_animal(id, hand, "minecraft:armadillo_food")
        }
    }
}

/// `IronGolem`'s maximum health.
const IRON_GOLEM_HEALTH: f32 = 100.0;
