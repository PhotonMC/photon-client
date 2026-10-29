use super::*;

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
    let entity_id = entity_id_field(request, "entity_id")?;
    let mut world = crate::case_ids::world(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id.0 as u32)),
        type_id: crate::case_ids::entity_type(i32_field(request, "entity_type_id")?),
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
            .entities()
            .at(entity_id)
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
    let entity_id = entity_id_field(request, "entity_id")?;
    let mut world = crate::case_ids::world(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id.0 as u32)),
        type_id: crate::case_ids::entity_type(i32_field(request, "entity_type_id")?),
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
            .entities()
            .at(entity_id)
            .ok_or("air supply fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entities()
            .at(entity_id)
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
    let entity_id = entity_id_field(request, "entity_id")?;
    let mut world = crate::case_ids::world(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id.0 as u32)),
        type_id: crate::case_ids::entity_type(i32_field(request, "entity_type_id")?),
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
            .entities()
            .at(entity_id)
            .ok_or("pose fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entities()
            .at(entity_id)
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
    let entity_id = entity_id_field(request, "entity_id")?;
    let mut world = crate::case_ids::world(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id.0 as u32)),
        type_id: crate::case_ids::entity_type(i32_field(request, "entity_type_id")?),
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
            .entities()
            .at(entity_id)
            .ok_or("ticks frozen fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entities()
            .at(entity_id)
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
    let entity_id = entity_id_field(request, "entity_id")?;
    let mut world = crate::case_ids::world(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id.0 as u32)),
        type_id: crate::case_ids::entity_type(i32_field(request, "entity_type_id")?),
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
            .entities()
            .at(entity_id)
            .ok_or("custom name fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entities()
            .at(entity_id)
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
    let entity_id = entity_id_field(request, "entity_id")?;
    let mut world = crate::case_ids::world(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id.0 as u32)),
        type_id: crate::case_ids::entity_type(i32_field(request, "entity_type_id")?),
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
            .entities()
            .at(entity_id)
            .ok_or("living flags fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entities()
            .at(entity_id)
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
    let entity_id = entity_id_field(request, "entity_id")?;
    let mut world = crate::case_ids::world(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id.0 as u32)),
        type_id: crate::case_ids::entity_type(i32_field(request, "entity_type_id")?),
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
            .entities()
            .at(entity_id)
            .ok_or("health fixture entity disappeared")?;
        states.push(client_world_entity_health_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}
