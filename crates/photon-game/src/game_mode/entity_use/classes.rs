//! The `mobInteract` of the classes that answer one fixed way on the client: ocelot, dolphin, iron
//! golem, allay, bogged, piglin, panda, happy ghast and armadillo.
//!
//! Not covered: an allay whose hand shows a tick early, a piglin whose brain refuses to admire,
//! and the sounds.

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
            | "minecraft:copper_golem"
            | "minecraft:sulfur_cube"
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
            "minecraft:bogged" => self.interact_bogged(id, hand),
            _ => self.interact_listed_class_more(name, id, hand),
        }
    }

    fn interact_listed_class_more(&mut self, name: &str, id: EntityId, hand: u8) -> EntityUse {
        match name {
            "minecraft:piglin" => self.interact_piglin(id, hand),
            "minecraft:panda" => self.interact_panda(id, hand),
            "minecraft:happy_ghast" => self.interact_happy_ghast(id, hand),
            "minecraft:bee" => self.interact_bee(id, hand),
            "minecraft:copper_golem" => self.interact_copper_golem(id, hand),
            "minecraft:sulfur_cube" => self.interact_sulfur_cube(id, hand),
            _ => self.interact_armadillo(id, hand),
        }
    }

    fn holds_id(&self, hand: u8, wanted: photon_core::ids::ItemId) -> bool {
        self.held(hand).is_some_and(|(item, _)| item == wanted)
    }

    fn holds_item(&self, hand: u8, name: &str) -> bool {
        self.held(hand)
            .is_some_and(|(item, _)| crate::item::id_of(name) == Some(item))
    }

    /// `Ocelot.mobInteract`: food from an ocelot that is not yet trusting is used up.
    fn interact_ocelot(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let near = self.world.entities().at(id).is_some_and(|entity| {
            let (at, own) = (entity.position(), self.player.position());
            let (dx, dy, dz) = (own.x - at.0, own.y - at.1, own.z - at.2);
            dx * dx + dy * dy + dz * dz < 9.0
        });
        let trusting = self
            .world
            .entities()
            .at(id)
            .and_then(|entity| entity.is_trusting_ocelot().then_some(()))
            .is_some();
        if near && !trusting && self.holds_tagged(hand, "minecraft:ocelot_food") {
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

    /// `IronGolem.mobInteract`: an ingot heals a golem that is hurt by twenty-five.
    fn interact_iron_golem(&mut self, id: EntityId, hand: u8) -> EntityUse {
        if self.holds_id(hand, photon_data::item::ids::IRON_INGOT) && self.heal_local(id, 25.0) {
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
        let dancing = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| entity.is_allay_dancing());
        let cooling = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| entity.is_allay_cooling());
        if dancing && !cooling && self.holds_tagged(hand, "minecraft:duplicates_allays") {
            self.use_up(hand);
            self.allay_sound(id, "minecraft:block.amethyst_block.chime");
            EntityUse::Success { swing: true }
        } else if !carries && holds {
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
    fn interact_bogged(&self, id: EntityId, hand: u8) -> EntityUse {
        let sheared = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| entity.is_bogged_sheared());
        if !sheared && self.holds_id(hand, photon_data::item::ids::SHEARS) {
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
        if let Some(panda) = self.world.entities().at(id).map(|entity| entity.panda()) {
            if panda.is_scared(self.world.is_thundering()) {
                return EntityUse::Pass;
            }
            // Lying on its back, it is righted first (the flag is set on the client too).
            if panda.is_on_back() {
                // `setOnBack(false)` runs on the client too, until the server's data says.
                let flags = panda.flags() & !photon_world::mobs::panda::ON_BACK;
                let data = [photon_world::PANDA_FLAGS_DATA_ID, 0, flags, 255];
                self.world.apply_entity_metadata(id, &data);
                return EntityUse::Success { swing: true };
            }
        }
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
        let effect = self.holds_tagged(hand, "minecraft:bee_food")
            && ["wither_rose", "open_eyeblossom"]
                .iter()
                .any(|flower| self.holds_item(hand, &format!("minecraft:{flower}")));
        if effect {
            self.use_up(hand);
            return EntityUse::Success { swing: true };
        }
        self.feed_animal(id, hand, "minecraft:bee_food")
    }

    /// `CopperGolem.mobInteract` on the client: an empty hand takes what the golem carries, and
    /// shears take the antenna flower; the honey and the axe are the server's (the client passes).
    fn interact_copper_golem(&self, id: EntityId, hand: u8) -> EntityUse {
        use photon_world::EquipmentSlot;
        let Some(entity) = self.world.entities().at(id) else {
            return EntityUse::Pass;
        };
        let carries = entity.equipment(EquipmentSlot::MainHand).is_some();
        let shearable = entity.equipment(EquipmentSlot::Saddle).is_some_and(|worn| {
            self.world.block_state_registry().item_tags().contains(
                worn.item_registry_id().0,
                "minecraft:shearable_from_copper_golem",
            )
        });
        let takes = self.held(hand).is_none() && carries;
        if takes || (shearable && self.holds_id(hand, photon_data::item::ids::SHEARS)) {
            EntityUse::Success { swing: true }
        } else {
            EntityUse::Pass
        }
    }

    /// `SulfurCube.mobInteract` on the client: a baby takes slime; an adult's shears work when it
    /// holds something, a swallowable item it does not already hold is swallowed (one is used
    /// up, the server does the rest) and an empty bucket scoops it up.
    ///
    /// A primed cube passes. A fire starter does nothing on the client even to a cube that swallowed
    /// TNT: the explosion data is set where the equipment is checked, on the server only.
    fn interact_sulfur_cube(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let Some(entity) = self.world.entities().at(id) else {
            return EntityUse::Pass;
        };
        let (baby, locked) = (entity.is_baby(), entity.is_age_locked());
        let body = entity
            .equipment(photon_world::EquipmentSlot::Body)
            .map(|worn| worn.item_registry_id());
        if baby {
            return if !locked && self.holds_tagged(hand, "minecraft:sulfur_cube_food") {
                self.use_up(hand);
                EntityUse::Success { swing: true }
            } else {
                self.golden_dandelion_on_baby(id, hand)
            };
        }
        self.interact_adult_sulfur_cube(id, hand, body)
    }

    fn interact_adult_sulfur_cube(
        &mut self,
        id: EntityId,
        hand: u8,
        body: Option<photon_core::ids::ItemId>,
    ) -> EntityUse {
        let held = self.held(hand).map(|(item, _)| item);
        let primed = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| entity.sulfur_fuse() >= 0);
        if primed {
            EntityUse::Pass
        } else if body.is_some() && self.holds_id(hand, photon_data::item::ids::SHEARS) {
            EntityUse::Success { swing: true }
        } else if self.holds_tagged(hand, "minecraft:sulfur_cube_swallowable") {
            if body.is_some() && body == held {
                return EntityUse::Pass;
            }
            self.use_up(hand);
            EntityUse::Success { swing: false }
        } else if self.holds_item(hand, "minecraft:bucket") {
            self.scoop_up(id, hand, "minecraft:sulfur_cube_bucket")
        } else {
            EntityUse::Pass
        }
    }

    /// `Bucketable.bucketMobPickup` for a mob that an empty bucket picks up.
    fn scoop_up(&mut self, id: EntityId, hand: u8, filled: &str) -> EntityUse {
        let Some(bucket) = crate::item::id_of(filled) else {
            return EntityUse::Pass;
        };
        buckets::plain_result(self, hand, bucket);
        self.world.remove_entity(id);
        EntityUse::Success { swing: true }
    }

    /// `Armadillo.mobInteract`: a brush takes a scute off an adult, otherwise its food.
    fn interact_armadillo(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let adult = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| !entity.is_baby());
        let scared = self.world.entities().at(id).is_some_and(|entity| {
            entity.armadillo().is_some_and(|armadillo| {
                armadillo.mode() != photon_world::mobs::armadillo::ArmadilloMode::Idle
            })
        });
        if adult && self.holds_id(hand, photon_data::item::ids::BRUSH) {
            EntityUse::Success { swing: true }
        } else if scared {
            // `InteractionResult.FAIL`.
            EntityUse::Pass
        } else {
            self.feed_animal(id, hand, "minecraft:armadillo_food")
        }
    }
}
