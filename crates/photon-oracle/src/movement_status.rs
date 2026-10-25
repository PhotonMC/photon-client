//! The effects and attributes the server gave the player of a movement row.

use super::*;

/// The mob effects a row gives the player, then its attributes, as the server's packets do.
pub(super) fn apply_effects(
    world: &mut photon_world::ClientWorld,
    player: &mut photon_world::local_player::LocalPlayer,
    breaking: &Value,
) -> Result<(), String> {
    for effect in breaking["effects"].as_array().into_iter().flatten() {
        let name = string(effect, "effect")?;
        let id = photon_data::item::effect::effect_id(name)
            .ok_or_else(|| format!("unknown effect {name}"))?;
        world.apply_local_player_mob_effect(
            player.entity_id(),
            id,
            i32_field(effect, "amplifier")?,
            1_000_000,
            0,
        );
    }
    apply_attributes(world, player, breaking)
}

/// The attributes the row's server sent (`handleUpdateAttributes`), into the world and the player.
fn apply_attributes(
    world: &mut photon_world::ClientWorld,
    player: &mut photon_world::local_player::LocalPlayer,
    breaking: &Value,
) -> Result<(), String> {
    let mut attributes = Vec::new();
    for described in breaking["attributes"].as_array().into_iter().flatten() {
        let name = string(described, "attribute")?;
        let (id, _) = photon_data::entity_type::attribute(name)
            .ok_or_else(|| format!("unknown attribute {name}"))?;
        let mut modifiers = Vec::new();
        for modifier in described["modifiers"].as_array().into_iter().flatten() {
            modifiers.push(photon_world::ClientAttributeModifierState::new(
                string(modifier, "id")?.to_owned(),
                crate::living_ticks::bits64(modifier, "amount")?,
                u8::try_from(i32_field(modifier, "operation")?)
                    .map_err(|error| error.to_string())?,
            ));
        }
        attributes.push(photon_world::ClientAttributeState::new(
            i32::try_from(id).map_err(|error| error.to_string())?,
            crate::living_ticks::bits64(described, "base")?,
            modifiers,
        ));
    }
    world.apply_local_player_attributes(player.entity_id(), &attributes);
    player.set_attributes(&attributes);
    Ok(())
}
