//! The armour and off-hand stacks a movement row puts on the player.

use super::*;

/// The container slots a row may fill besides the main hand: its key and the slot it names.
const WORN: [(&str, i16); 2] = [("chest", 6), ("off_hand", 45)];

/// Put the row's worn items into the player's inventory, as the server's slot packets do.
pub(super) fn worn_items(
    world: &mut photon_world::ClientWorld,
    breaking: &Value,
) -> Result<(), String> {
    for (key, slot) in WORN {
        let Some(name) = breaking[key].as_str() else {
            continue;
        };
        let item = photon_game::item::id_of(name).ok_or_else(|| format!("unknown item {name}"))?;
        world.apply_local_player_container_slot(
            0,
            0,
            slot,
            Some(&photon_world::ClientItemStackState::new(1, item)),
        );
    }
    Ok(())
}
