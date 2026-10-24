//! The state reports of a passenger's rotation.

use super::*;

pub(crate) fn client_world_entity_passenger_rotation_state(
    label: &str,
    entity_ids: &[EntityId],
    passenger_type_ids: &[i32],
    world: &ClientWorld,
) -> Result<Value, String> {
    let mut state = client_world_entity_tick_state(label, true, entity_ids, world)?;
    let entities = state["entities"]
        .as_array_mut()
        .ok_or("passenger rotation state entities must be an array")?;
    for (index, passenger_type_id) in passenger_type_ids.iter().enumerate() {
        if !matches!(*passenger_type_id, 17 | 156) {
            continue;
        }
        let entity_id = entity_ids
            .get(index * 2 + 1)
            .ok_or("passenger rotation state entity list is incomplete")?;
        let passenger = world
            .entities()
            .at(*entity_id)
            .ok_or("passenger rotation fixture entity disappeared")?;
        let rotation = passenger.rotation();
        let previous_rotation = passenger.previous_rotation();
        let entity = entities
            .get_mut(index * 2 + 1)
            .and_then(Value::as_object_mut)
            .ok_or("passenger rotation state passenger is not an object")?;
        entity.insert(
            "head_rotation".into(),
            json!(format!("{:08x}", rotation.2.to_bits())),
        );
        entity.insert(
            "previous_head_rotation".into(),
            json!(format!("{:08x}", previous_rotation.2.to_bits())),
        );
        entity.insert(
            "body_rotation_y".into(),
            json!(format!("{:08x}", passenger.body_y_rotation().to_bits())),
        );
        entity.insert(
            "previous_body_rotation_y".into(),
            json!(format!(
                "{:08x}",
                passenger.previous_body_y_rotation().to_bits()
            )),
        );
    }
    Ok(state)
}

pub(crate) fn client_world_entity_passenger_animal_rotation_state(
    label: &str,
    entity_ids: &[EntityId],
    world: &ClientWorld,
) -> Result<Value, String> {
    let mut state = client_world_entity_tick_state(label, true, entity_ids, world)?;
    let entities = state["entities"]
        .as_array_mut()
        .ok_or("passenger animal rotation state entities must be an array")?;
    if !entity_ids.len().is_multiple_of(3) {
        return Err("passenger animal rotation state entity list is incomplete".into());
    }
    for row_index in 0..(entity_ids.len() / 3) {
        let animal_id = entity_ids
            .get(row_index * 3 + 1)
            .ok_or("passenger animal rotation state entity list is incomplete")?;
        let animal = world
            .entities()
            .at(*animal_id)
            .ok_or("passenger animal rotation fixture entity disappeared")?;
        let rotation = animal.rotation();
        let previous_rotation = animal.previous_rotation();
        let entity = entities
            .get_mut(row_index * 3 + 1)
            .and_then(Value::as_object_mut)
            .ok_or("passenger animal rotation state animal is not an object")?;
        entity.insert(
            "head_rotation".into(),
            json!(format!("{:08x}", rotation.2.to_bits())),
        );
        entity.insert(
            "previous_head_rotation".into(),
            json!(format!("{:08x}", previous_rotation.2.to_bits())),
        );
        entity.insert(
            "body_rotation_y".into(),
            json!(format!("{:08x}", animal.body_y_rotation().to_bits())),
        );
        entity.insert(
            "previous_body_rotation_y".into(),
            json!(format!(
                "{:08x}",
                animal.previous_body_y_rotation().to_bits()
            )),
        );
    }
    Ok(state)
}
