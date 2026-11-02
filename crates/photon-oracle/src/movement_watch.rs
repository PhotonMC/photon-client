//! The blocks a tick of a movement row watches.

use serde_json::{Value, json};

use super::movement_gear::with_rockets;
use super::*;

/// The tick's report with the rockets and the watched blocks added.
pub(super) fn finished_tick(
    described: Value,
    world: &photon_world::ClientWorld,
    (row, breaking, tick): (&Value, Option<&Value>, &Value),
) -> Result<Value, String> {
    let watched = with_watched(described, world, tick)?;
    with_lid(with_rockets(watched, world, breaking), world, row)
}

/// The tick's report with the block state ids at the places the tick watches, if it names any.
fn with_watched(
    mut described: Value,
    world: &photon_world::ClientWorld,
    tick: &Value,
) -> Result<Value, String> {
    let Some(places) = tick.get("watch") else {
        return Ok(described);
    };
    let state_at = |place: &Value| -> Result<i32, String> {
        let at = (
            i32_field(place, "x")?,
            i32_field(place, "y")?,
            i32_field(place, "z")?,
        );
        Ok(world
            .block_state_at(at.0, at.1, at.2)
            .map_or(0, crate::case_ids::wire_state))
    };
    let states = places.as_array().ok_or("watch must be an array")?;
    let states: Result<Vec<i32>, String> = states.iter().map(state_at).collect();
    described["watched_states"] = json!(states?);
    Ok(described)
}

/// The tick's report with how far the watched chest's lid is open, if the row watches one.
fn with_lid(
    mut tick: Value,
    world: &photon_world::ClientWorld,
    row: &Value,
) -> Result<Value, String> {
    if let Some(lid) = row.get("lid") {
        let at = (
            i32_field(lid, "x")?,
            i32_field(lid, "y")?,
            i32_field(lid, "z")?,
        );
        tick["lid"] = json!(format!(
            "{:08x}",
            world.chest_openness(at, None, 0.0).to_bits()
        ));
    }
    Ok(tick)
}

/// The species' own synced data a row sets: an ocelot that trusts players, a goat that screams,
/// an armadillo that is scared (`ARMADILLO_STATE`, serializer 36, scared is two) and a panda
/// lying on its back (the byte flags, sixteen).
pub(super) fn apply_species_data(
    world: &mut photon_world::ClientWorld,
    id: EntityId,
    described: &Value,
) {
    for (flag, data) in [
        (
            "trusting",
            [photon_world::OCELOT_TRUSTING_DATA_ID, 8, 1, 255],
        ),
        (
            "screaming",
            [photon_world::GOAT_SCREAMING_DATA_ID, 8, 1, 255],
        ),
        (
            "scared",
            [photon_world::mobs::armadillo::STATE_DATA_ID, 36, 2, 255],
        ),
        ("on_back", [photon_world::PANDA_FLAGS_DATA_ID, 0, 16, 255]),
    ] {
        if described.get(flag).is_some() {
            world.apply_entity_metadata(id, &data);
        }
    }
}
