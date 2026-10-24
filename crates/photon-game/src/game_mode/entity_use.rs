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
    pub(super) fn interact_entity(&self, id: EntityId) -> EntityUse {
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
            _ => EntityUse::Pass,
        }
    }
}
