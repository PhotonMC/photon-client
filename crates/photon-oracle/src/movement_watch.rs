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
    with_animations(with_rockets(watched, world, breaking), world, row)
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

/// The tick's report with the block animations the row watches: how far a chest's lid is open, a
/// bell's ticks and whether it shakes, and a head's animation value.
fn with_animations(
    mut tick: Value,
    world: &photon_world::ClientWorld,
    row: &Value,
) -> Result<Value, String> {
    let bits = |value: f32| format!("{:08x}", value.to_bits());
    let place = |key: &str| -> Result<Option<(i32, i32, i32)>, String> {
        let Some(at) = row.get(key) else {
            return Ok(None);
        };
        Ok(Some((
            i32_field(at, "x")?,
            i32_field(at, "y")?,
            i32_field(at, "z")?,
        )))
    };
    if let Some(at) = place("lid")? {
        tick["lid"] = json!(bits(world.chest_openness(at, None, 0.0)));
    }
    if let Some(at) = place("bell")? {
        let (ticks, direction) = world.bell_shake(at, 0.0).ok_or("no bell to watch")?;
        tick["bell"] = json!({"ticks": bits(ticks), "shaking": direction.is_some()});
    }
    if let Some(at) = place("skull")? {
        let view = world.skull_block_view(at, 0.0).ok_or("no head to watch")?;
        tick["skull"] = json!(bits(view.animation));
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
        // The worried gene (id two) as the main gene, which a dominant gene alone decides.
        (
            "worried",
            [photon_world::PANDA_MAIN_GENE_DATA_ID, 0, 2, 255],
        ),
        ("dancing", [photon_world::ALLAY_DANCING_DATA_ID, 8, 1, 255]),
        (
            "responds",
            [photon_world::INTERACTION_RESPONSE_DATA_ID, 8, 1, 255],
        ),
    ] {
        if described.get(flag).is_some() {
            world.apply_entity_metadata(id, &data);
        }
    }
}
