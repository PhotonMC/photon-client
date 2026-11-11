//! `interact` of the boats, the interaction entity and the leash knot: the entities that are
//! neither mobs nor the older vehicles handled in the parent module.

use super::*;

/// Whether the type is a boat or a raft, with or without a chest.
fn is_boat(name: &str) -> bool {
    name.ends_with("_boat")
        || name.ends_with("_raft")
        || name.ends_with("_chest_boat")
        || name.ends_with("_chest_raft")
}

/// Whether the entity type is the cushion (26.3).
fn is_cushion(name: &str) -> bool {
    photon_data::entity_type::name(photon_data::entity_type::ids::CUSHION) == Some(name)
}

impl Context<'_> {
    /// `Cushion.interact`: a sitter gets on unless sneaking or the cushion is taken; the client's
    /// answer is a consume, which is a success without a swing, and the server seats the rider.
    fn interact_cushion(&self, id: EntityId) -> EntityUse {
        let taken = self
            .world
            .entity_passengers(id)
            .is_some_and(|riders| !riders.is_empty());
        if self.player.is_shift_key_down() || taken {
            EntityUse::Pass
        } else {
            EntityUse::Success { swing: false }
        }
    }

    /// The answer of an entity that has no `Mob.mobInteract`, or the mob class when it is one.
    pub(super) fn interact_other_entity(
        &mut self,
        name: &str,
        id: EntityId,
        hand: u8,
    ) -> EntityUse {
        match name {
            // `AbstractBoat.interact` boards unless sneaking; a chest boat then opens its chest
            // (a success on the client).
            n if is_boat(n) => {
                let chest = n.contains("_chest_");
                let out_of_control = self
                    .world
                    .entities()
                    .at(id)
                    .and_then(|entity| entity.boat().map(|boat| boat.is_out_of_control()))
                    .unwrap_or(false);
                let sneaking = self.player.is_shift_key_down();
                // A chest boat that boarding refuses opens its chest instead; one that has been
                // under the water has its rider's eyes in it, so that refusal is certain.
                if !chest && (sneaking || out_of_control) {
                    EntityUse::Pass
                } else {
                    EntityUse::Success { swing: true }
                }
            }
            // `Cushion.interact`.
            n if is_cushion(n) => self.interact_cushion(id),
            // `Interaction.interact` on the client: consume, or succeed when it answers.
            "minecraft:interaction" => {
                let responds = self
                    .world
                    .entities()
                    .at(id)
                    .is_some_and(|entity| entity.interaction_responds());
                EntityUse::Success { swing: responds }
            }
            // `LeashFenceKnotEntity.interact` succeeds on the client.
            "minecraft:leash_knot" => EntityUse::Success { swing: true },
            _ => self.interact_mob_class(name, id, hand),
        }
    }
}
