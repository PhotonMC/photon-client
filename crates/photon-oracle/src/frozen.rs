//! `isEntityFrozen` of entities with passengers, at a level in a given state.

use photon_core::binary::Uuid;
use photon_core::ids::EntityId;
use photon_world::{ClientWorld, EntityInput};
use serde_json::{Value, json};

/// The tick rate manager of a level that runs, is frozen or is stepping, after one tick.
fn manager_at(level: &str) -> Result<photon_world::tick_rate::TickRateManager, String> {
    let mut manager = photon_world::tick_rate::TickRateManager::default();
    match level {
        "running" => {}
        "frozen" => manager.set_frozen(true),
        "stepping" => {
            manager.set_frozen(true);
            manager.set_frozen_ticks_to_run(2);
        }
        other => return Err(format!("unknown level {other}")),
    }
    manager.tick();
    Ok(manager)
}

fn entity_type_of(kind: &str) -> Result<&'static str, String> {
    match kind {
        "stand" => Ok("minecraft:armor_stand"),
        "player" => Ok("minecraft:player"),
        "mannequin" => Ok("minecraft:mannequin"),
        other => Err(format!("unknown kind {other}")),
    }
}

/// A world with one entity per kind, entity `n` having id `n`.
fn world_of(kinds: &[Value]) -> Result<ClientWorld, String> {
    let mut world = ClientWorld::default();
    for (index, kind) in kinds.iter().enumerate() {
        let kind = kind.as_str().ok_or("kind")?;
        // The local player is the entity the world calls its own, and is not in the entity list.
        if kind == "local" {
            world.set_local_player_id(EntityId(index as i32));
            continue;
        }
        world.apply_entity(EntityInput {
            entity_id: EntityId(index as i32),
            uuid: Uuid::new(1, index as u64),
            type_id: photon_data::entity_type::id(entity_type_of(kind)?).ok_or("entity type")?,
            x: 0.0,
            y: 64.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        });
    }
    Ok(world)
}

/// `entity_frozen`: which of some entities a level in a given state calls frozen.
pub(crate) fn entity_frozen_result(request: &Value) -> Result<Value, String> {
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let manager = manager_at(row["level"].as_str().ok_or("level")?)?;
        let kinds = row["entities"].as_array().ok_or("entities")?;
        let mut world = world_of(kinds)?;
        let links = row["links"].as_array().ok_or("links")?;
        for vehicle in 0..kinds.len() {
            let riders: Vec<EntityId> = links
                .iter()
                .filter(|link| link[0].as_u64() == Some(vehicle as u64))
                .map(|link| EntityId(link[1].as_i64().unwrap_or(0) as i32))
                .collect();
            if !riders.is_empty() {
                world.apply_entity_passengers(EntityId(vehicle as i32), &riders);
            }
        }
        let frozen: Vec<bool> = (0..kinds.len())
            .map(|index| world.is_entity_frozen(EntityId(index as i32), manager.runs_normally()))
            .collect();
        rows.push(json!({"label": row["label"], "frozen": frozen}));
    }
    Ok(json!({"rows": rows}))
}
