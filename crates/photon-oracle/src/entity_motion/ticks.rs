//! The entity tick report: the head rotation state and the tick state.

use super::*;
use photon_world::EntityView;

pub(crate) fn client_world_entity_player_head_rotation_state(
    label: &str,
    accepted: bool,
    entity: EntityView<'_>,
) -> Value {
    let rotation = entity.rotation();
    let previous_rotation = entity.previous_rotation();
    json!({
        "label": label,
        "accepted": accepted,
        "head_rotation": format!("{:08x}", rotation.2.to_bits()),
        "previous_head_rotation": format!("{:08x}", previous_rotation.2.to_bits()),
        "head_target": format!("{:08x}", entity.head_rotation_target().to_bits()),
        "head_lerp_steps": entity.head_rotation_lerp_steps(),
        "body_rotation_y": format!("{:08x}", entity.body_y_rotation().to_bits()),
    })
}

pub(crate) fn client_world_entity_tick_vector(vector: (f64, f64, f64)) -> Value {
    json!({
        "x": format!("{:016x}", vector.0.to_bits()),
        "y": format!("{:016x}", vector.1.to_bits()),
        "z": format!("{:016x}", vector.2.to_bits()),
    })
}

pub(crate) fn client_world_entity_tick_state(
    label: &str,
    accepted: bool,
    entity_ids: &[EntityId],
    world: &ClientWorld,
) -> Result<Value, String> {
    let entities = entity_ids
        .iter()
        .map(|entity_id| {
            let entity = world
                .entities()
                .at(*entity_id)
                .ok_or_else(|| "entity tick fixture entity disappeared".to_owned())?;
            let rotation = entity.rotation();
            let previous_rotation = entity.previous_rotation();
            Ok(json!({
                "entity_id": entity.id().0,
                "tick_count": entity.tick_count(),
                "position": client_world_entity_tick_vector(entity.position()),
                "previous_position": client_world_entity_tick_vector(
                    entity.previous_position()
                ),
                "motion": client_world_entity_tick_vector(entity.motion()),
                "rotation_y": format!("{:08x}", rotation.0.to_bits()),
                "rotation_x": format!("{:08x}", rotation.1.to_bits()),
                "previous_rotation_y": format!("{:08x}", previous_rotation.0.to_bits()),
                "previous_rotation_x": format!("{:08x}", previous_rotation.1.to_bits()),
                "no_gravity": entity.is_no_gravity(),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "label": label,
        "accepted": accepted,
        "entities": entities,
    }))
}
