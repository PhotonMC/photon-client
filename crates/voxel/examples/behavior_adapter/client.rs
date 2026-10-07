use super::*;

pub(super) fn client_world_entity_mob_flags_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "mob_flags": entity.mob_flags(),
        "is_no_ai": entity.is_no_ai(),
        "is_left_handed": entity.is_left_handed(),
        "is_aggressive": entity.is_aggressive(),
    })
}

pub(super) fn client_world_entity_mob_flags_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("mob flags fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_mob_flags_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("mob flags fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("mob flags fixture entity disappeared")?;
        states.push(client_world_entity_mob_flags_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_arrow_count_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "arrow_count": entity.arrow_count(),
        "living_flags": entity.living_flags(),
    })
}

pub(super) fn client_world_entity_arrow_count_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("arrow count fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_arrow_count_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("arrow count fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("arrow count fixture entity disappeared")?;
        states.push(client_world_entity_arrow_count_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_effect_ambience_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "living_flags": entity.living_flags(),
        "effect_ambience": entity.is_effect_ambient(),
    })
}

pub(super) fn client_world_entity_effect_ambience_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("effect ambience fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_effect_ambience_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("effect ambience fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("effect ambience fixture entity disappeared")?;
        states.push(client_world_entity_effect_ambience_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_effect_particles_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "living_flags": entity.living_flags(),
        "effect_ambience": entity.is_effect_ambient(),
        "effect_particle_type_ids": entity.effect_particle_type_ids(),
    })
}

pub(super) fn client_world_entity_effect_particles_result(
    request: &Value,
) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("effect particles fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_effect_particles_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("effect particles fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("effect particles fixture entity disappeared")?;
        states.push(client_world_entity_effect_particles_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_sleeping_position_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    let mut state = json!({
        "label": label,
        "accepted": accepted,
        "living_flags": entity.living_flags(),
        "is_sleeping": entity.is_sleeping(),
    });
    if let Some(position) = entity.sleeping_pos() {
        state
            .as_object_mut()
            .expect("sleeping position state object")
            .insert("sleeping_pos".into(), blockpos_snapshot(position));
    }
    state
}

pub(super) fn client_world_entity_sleeping_position_result(
    request: &Value,
) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("sleeping position fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_sleeping_position_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("sleeping position fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("sleeping position fixture entity disappeared")?;
        states.push(client_world_entity_sleeping_position_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_stinger_count_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "stinger_count": entity.stinger_count(),
        "living_flags": entity.living_flags(),
    })
}

pub(super) fn client_world_entity_stinger_count_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("stinger count fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_stinger_count_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("stinger count fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("stinger count fixture entity disappeared")?;
        states.push(client_world_entity_stinger_count_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_chunk_cache_snapshot_json(snapshot: &ClientChunkCacheSnapshot) -> Value {
    json!({
        "storage_radius": snapshot.storage_radius,
        "view_range": snapshot.view_range,
        "view_center_x": snapshot.view_center_x,
        "view_center_z": snapshot.view_center_z,
        "loaded_chunks": snapshot.loaded_chunks,
        "added_empty_sections": snapshot.added_empty_sections,
        "removed_empty_sections": snapshot.removed_empty_sections,
        "added_loaded_chunks": snapshot.added_loaded_chunks,
        "removed_loaded_chunks": snapshot.removed_loaded_chunks,
    })
}
