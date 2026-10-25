//! `Bucketable.bucketMobPickup`: a water bucket scoops up a fish, an axolotl or a tadpole.
//!
//! Not covered: the components the bucket keeps (variant, age) and the sound.

use super::*;

/// The bucket a living fish, axolotl or tadpole turns a water bucket into.
pub(super) fn bucket_of(name: &str) -> Option<&'static str> {
    Some(match name {
        "minecraft:cod" => "minecraft:cod_bucket",
        "minecraft:salmon" => "minecraft:salmon_bucket",
        "minecraft:pufferfish" => "minecraft:pufferfish_bucket",
        "minecraft:tropical_fish" => "minecraft:tropical_fish_bucket",
        "minecraft:axolotl" => "minecraft:axolotl_bucket",
        "minecraft:tadpole" => "minecraft:tadpole_bucket",
        _ => return None,
    })
}

impl Context<'_> {
    /// The water bucket is taken first; an axolotl then answers as an animal, and a baby that
    /// is not age locked hands back a water bucket for the tropical fish bucket it is fed.
    pub(super) fn interact_bucketable(&mut self, name: &str, id: EntityId, hand: u8) -> EntityUse {
        let water = crate::item::id_of("minecraft:water_bucket");
        let held = self.held(hand).map(|(item, _)| item);
        let filled = bucket_of(name).and_then(crate::item::id_of);
        if let (Some(filled), true) = (filled, held.is_some() && held == water) {
            buckets::plain_result(self, hand, filled);
            self.world.remove_entity(id);
            return EntityUse::Success { swing: true };
        }
        if name == "minecraft:tadpole" {
            return self.interact_tadpole(id, hand);
        }
        if name != "minecraft:axolotl" {
            return EntityUse::Pass;
        }
        let tag = "minecraft:axolotl_food";
        let ages = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| entity.is_baby() && !entity.is_age_locked());
        if ages && self.holds_tagged(hand, tag) {
            if let Some(bucket) = water {
                buckets::filled_with(self, hand, ClientItemStackState::new(1, bucket));
            }
            return EntityUse::Success { swing: true };
        }
        self.feed_animal(id, hand, tag)
    }

    /// `Tadpole.mobInteract` after the bucket: frog food that is not age locked is eaten.
    fn interact_tadpole(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let locked = self
            .world
            .entities()
            .at(id)
            .is_some_and(|entity| entity.is_age_locked());
        if !locked && self.holds_tagged(hand, "minecraft:frog_food") {
            self.use_up(hand);
            return EntityUse::Success { swing: true };
        }
        self.golden_dandelion_on_baby(id, hand)
    }
}
