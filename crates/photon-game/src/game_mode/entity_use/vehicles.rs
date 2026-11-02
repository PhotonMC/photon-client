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

impl Context<'_> {
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
                if (sneaking && !chest) || (out_of_control && !(sneaking && chest)) {
                    EntityUse::Pass
                } else {
                    EntityUse::Success { swing: true }
                }
            }
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
