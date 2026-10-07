use super::*;

pub(super) fn client_world_player_sprint_trigger_result(request: &Value) -> Result<Value, String> {
    let player_id = i32_field(request, "player_id")?;
    let sprint_window = i32_field(request, "sprint_window")?;
    let conditions = sprint_trigger_conditions(
        request
            .get("conditions")
            .ok_or("conditions must be an object")?,
    )?;
    let sequences = request["sequences"]
        .as_array()
        .ok_or("sequences must be an array")?;
    let mut sequence_states = Vec::with_capacity(sequences.len());

    for sequence in sequences {
        let mut world = ClientWorld::new(DimensionBounds::new(0, 256));
        world.set_local_player_id(player_id);
        let operations = sequence["operations"]
            .as_array()
            .ok_or("operations must be an array")?;
        let mut states = Vec::with_capacity(operations.len());
        for operation in operations {
            let current_input = sprint_trigger_input(
                operation
                    .get("key_presses")
                    .ok_or("key_presses must be an object")?,
            )?;
            let move_vector_value = operation
                .get("move_vector")
                .ok_or("move_vector must be an object")?;
            let move_vector = Vec2::new(
                f32_field(move_vector_value, "x")?,
                f32_field(move_vector_value, "y")?,
            );
            let ticks = operation["ticks"]
                .as_u64()
                .ok_or("ticks must be an unsigned integer")?;
            for _ in 0..ticks {
                if !world.apply_local_player_sprint_trigger(
                    current_input,
                    move_vector,
                    sprint_window,
                    conditions,
                ) {
                    return Err("local player is not initialized".into());
                }
            }
            let player = world
                .local_player()
                .ok_or("local player is not initialized")?;
            states.push(client_world_player_sprint_trigger_state(
                string(operation, "label")?,
                player,
            ));
        }
        sequence_states.push(json!({
            "label": string(sequence, "label")?,
            "states": states,
        }));
    }

    Ok(json!({
        "player_id": player_id,
        "sequences": sequence_states,
    }))
}

pub(super) fn client_world_entity_shared_flags_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "shared_flags": entity.shared_flags(),
        "on_fire": entity.is_on_fire(),
        "shift_key_down": entity.is_shift_key_down(),
        "sprinting": entity.is_sprinting(),
        "swimming": entity.is_swimming(),
        "invisible": entity.is_invisible(),
        "glowing": entity.is_glowing(),
        "fall_flying": entity.is_fall_flying(),
        "air_supply": entity.air_supply(),
    })
}

pub(super) fn client_world_entity_shared_flags_result(request: &Value) -> Result<Value, String> {
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
        return Err("shared flags fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("shared flags fixture entity disappeared")?;
        states.push(client_world_entity_shared_flags_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_air_supply_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "shared_flags": entity.shared_flags(),
        "air_supply": entity.air_supply(),
    })
}

pub(super) fn client_world_entity_air_supply_result(request: &Value) -> Result<Value, String> {
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
        return Err("air supply fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_air_supply_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("air supply fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("air supply fixture entity disappeared")?;
        states.push(client_world_entity_air_supply_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_pose_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "shared_flags": entity.shared_flags(),
        "pose_id": entity.pose().id(),
    })
}

pub(super) fn client_world_entity_pose_result(request: &Value) -> Result<Value, String> {
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
        return Err("pose fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_pose_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("pose fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("pose fixture entity disappeared")?;
        states.push(client_world_entity_pose_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_ticks_frozen_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "shared_flags": entity.shared_flags(),
        "ticks_frozen": entity.ticks_frozen(),
        "percent_frozen": format!("{:08x}", entity.percent_frozen().to_bits()),
        "is_freezing": entity.is_freezing(),
        "is_fully_frozen": entity.is_fully_frozen(),
    })
}

pub(super) fn client_world_entity_ticks_frozen_result(request: &Value) -> Result<Value, String> {
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
        return Err("ticks frozen fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_ticks_frozen_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("ticks frozen fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("ticks frozen fixture entity disappeared")?;
        states.push(client_world_entity_ticks_frozen_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_custom_name_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "custom_name_present": entity.has_custom_name(),
        "custom_name_visible": entity.is_custom_name_visible(),
        "should_show_name": entity.should_show_name(),
    })
}

pub(super) fn client_world_entity_custom_name_result(request: &Value) -> Result<Value, String> {
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
        return Err("custom name fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_custom_name_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("custom name fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("custom name fixture entity disappeared")?;
        states.push(client_world_entity_custom_name_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_living_flags_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "living_flags": entity.living_flags(),
        "is_using_item": entity.is_using_item(),
        "used_item_is_off_hand": entity.used_item_is_off_hand(),
        "is_auto_spin_attack": entity.is_auto_spin_attack(),
    })
}

pub(super) fn client_world_entity_living_flags_result(request: &Value) -> Result<Value, String> {
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
        return Err("living flags fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_living_flags_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("living flags fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("living flags fixture entity disappeared")?;
        states.push(client_world_entity_living_flags_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_health_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "living_flags": entity.living_flags(),
        "health_bits": entity
            .health()
            .map(|value| format!("{:08x}", value.to_bits())),
    })
}

pub(super) fn client_world_entity_health_result(request: &Value) -> Result<Value, String> {
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
        return Err("health fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("health fixture entity disappeared")?;
        states.push(client_world_entity_health_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}
