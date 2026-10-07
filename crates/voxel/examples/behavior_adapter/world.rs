use super::*;

pub(super) fn client_world_entity_snowball_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_tick_result(request, 120)
}

pub(super) fn client_world_entity_player_interpolation_state(
    label: &str,
    entity: &ClientEntity,
) -> Value {
    let position = entity.position();
    let previous_position = entity.previous_position();
    let rotation = entity.rotation();
    let previous_rotation = entity.previous_rotation();
    let interpolation_active = entity.interpolation_steps() > 0;
    let mut state = json!({
        "label": label,
        "accepted": true,
        "tick_count": entity.tick_count(),
        "position": client_world_entity_tick_vector(position),
        "previous_position": client_world_entity_tick_vector(previous_position),
        "rotation_y": format!("{:08x}", rotation.0.to_bits()),
        "rotation_x": format!("{:08x}", rotation.1.to_bits()),
        "previous_rotation_y": format!("{:08x}", previous_rotation.0.to_bits()),
        "previous_rotation_x": format!("{:08x}", previous_rotation.1.to_bits()),
        "interpolation_active": interpolation_active,
        "interpolation_steps": entity.interpolation_steps(),
    });
    if let Some(target_position) = entity.interpolation_target_position() {
        state["interpolation_target_position"] = client_world_entity_tick_vector(target_position);
    }
    if let Some(target_rotation) = entity.interpolation_target_rotation() {
        state["interpolation_target_rotation"] = json!({
            "y": format!("{:08x}", target_rotation.0.to_bits()),
            "x": format!("{:08x}", target_rotation.1.to_bits()),
        });
    }
    state
}

pub(super) fn client_world_entity_player_interpolation_result(
    request: &Value,
) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let initial_position = f64_vec3(
        request
            .get("initial_position")
            .ok_or("initial_position must be a vector")?,
    )?;
    let initial_rotation = request
        .get("initial_rotation")
        .ok_or("initial_rotation must be an object")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: 156,
        x: initial_position.x,
        y: initial_position.y,
        z: initial_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (
            f32_field(initial_rotation, "y")?,
            f32_field(initial_rotation, "x")?,
            f32_field(request, "initial_head")?,
        ),
        data: 0,
    }) {
        return Err("remote player interpolation fixture was rejected".into());
    }

    let mut states = vec![client_world_entity_player_interpolation_state(
        "initial",
        world
            .entity_at(entity_id)
            .ok_or("remote player interpolation fixture disappeared")?,
    )];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        match string(operation, "action")? {
            "move" => {
                let delta = operation.get("delta").ok_or("delta must be an object")?;
                let delta_i16 = |name: &str| {
                    delta[name]
                        .as_i64()
                        .ok_or_else(|| format!("delta {name} must be an integer"))?
                        .try_into()
                        .map_err(|_| format!("delta {name} must fit in a short"))
                };
                let rotation = operation
                    .get("rotation")
                    .ok_or("rotation must be an object")?;
                if !world.apply_entity_move(EntityMoveInput {
                    entity_id,
                    delta_x: delta_i16("x")?,
                    delta_y: delta_i16("y")?,
                    delta_z: delta_i16("z")?,
                    has_position: true,
                    y_rot: Some(f32_field(rotation, "y")?),
                    x_rot: Some(f32_field(rotation, "x")?),
                }) {
                    return Err("remote player interpolation move was rejected".into());
                }
            }
            "tick" => {
                let ticks = operation["ticks"]
                    .as_u64()
                    .ok_or("ticks must be an unsigned integer")?;
                for _ in 0..ticks {
                    world.tick();
                }
            }
            _ => return Err("unknown remote player interpolation action".into()),
        }
        states.push(client_world_entity_player_interpolation_state(
            string(operation, "label")?,
            world
                .entity_at(entity_id)
                .ok_or("remote player interpolation fixture disappeared")?,
        ));
    }
    Ok(json!({"entity_id": entity_id, "states": states}))
}

pub(super) fn client_world_entity_player_snap_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let initial_position = f64_vec3(
        request
            .get("initial_position")
            .ok_or("initial_position must be a vector")?,
    )?;
    let initial_rotation = request
        .get("initial_rotation")
        .ok_or("initial_rotation must be an object")?;
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    for chunk_x in [0, 4, 5, 9, 13] {
        if !world.replace_chunk(
            chunk_x,
            -3,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        ) {
            return Err("remote player snap fixture chunk was rejected".into());
        }
    }
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: 156,
        x: initial_position.x,
        y: initial_position.y,
        z: initial_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (
            f32_field(initial_rotation, "y")?,
            f32_field(initial_rotation, "x")?,
            f32_field(request, "initial_head")?,
        ),
        data: 0,
    }) {
        return Err("remote player snap fixture was rejected".into());
    }

    let mut states = vec![client_world_entity_player_interpolation_state(
        "initial",
        world
            .entity_at(entity_id)
            .ok_or("remote player snap fixture disappeared")?,
    )];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        match string(operation, "action")? {
            "position_sync" => {
                let target = f64_vec3(
                    operation
                        .get("target_position")
                        .ok_or("target position must be a vector")?,
                )?;
                let rotation = operation
                    .get("rotation")
                    .ok_or("rotation must be an object")?;
                if !world.apply_entity_position_sync(
                    entity_id,
                    (target.x, target.y, target.z),
                    (f32_field(rotation, "y")?, f32_field(rotation, "x")?),
                ) {
                    return Err("remote player snap position sync was rejected".into());
                }
            }
            "tick" => {
                let ticks = operation["ticks"]
                    .as_u64()
                    .ok_or("ticks must be an unsigned integer")?;
                for _ in 0..ticks {
                    world.tick();
                }
            }
            _ => return Err("unknown remote player snap action".into()),
        }
        states.push(client_world_entity_player_interpolation_state(
            string(operation, "label")?,
            world
                .entity_at(entity_id)
                .ok_or("remote player snap fixture disappeared")?,
        ));
    }
    Ok(json!({"entity_id": entity_id, "states": states}))
}

pub(super) fn client_world_entity_player_motion_state(label: &str, entity: &ClientEntity) -> Value {
    let position = entity.position();
    let previous_position = entity.previous_position();
    let motion = entity.motion();
    let mut state = json!({
        "label": label,
        "accepted": true,
        "tick_count": entity.tick_count(),
        "position": client_world_entity_tick_vector(position),
        "previous_position": client_world_entity_tick_vector(previous_position),
        "motion": client_world_entity_tick_vector(motion),
        "motion_steps": entity.motion_interpolation_steps(),
    });
    if let Some(target_motion) = entity.motion_interpolation_target() {
        state["motion_target"] = client_world_entity_tick_vector(target_motion);
    }
    state
}

pub(super) fn client_world_entity_player_motion_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let initial_position = f64_vec3(
        request
            .get("initial_position")
            .ok_or("initial_position must be a vector")?,
    )?;
    let initial_rotation = request
        .get("initial_rotation")
        .ok_or("initial_rotation must be an object")?;
    let initial_motion = f64_vec3(
        request
            .get("initial_motion")
            .ok_or("initial_motion must be a vector")?,
    )?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: 156,
        x: initial_position.x,
        y: initial_position.y,
        z: initial_position.z,
        movement: (initial_motion.x, initial_motion.y, initial_motion.z),
        rotation: (
            f32_field(initial_rotation, "y")?,
            f32_field(initial_rotation, "x")?,
            f32_field(request, "initial_head")?,
        ),
        data: 0,
    }) {
        return Err("remote player motion fixture was rejected".into());
    }

    let mut states = vec![client_world_entity_player_motion_state(
        "initial",
        world
            .entity_at(entity_id)
            .ok_or("remote player motion fixture disappeared")?,
    )];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        match string(operation, "action")? {
            "motion" => {
                let target = f64_vec3(
                    operation
                        .get("target_motion")
                        .ok_or("target motion must be a vector")?,
                )?;
                if !world.apply_entity_motion(EntityMotionInput {
                    entity_id,
                    x: target.x,
                    y: target.y,
                    z: target.z,
                }) {
                    return Err("remote player motion was rejected".into());
                }
            }
            "tick" => {
                let ticks = operation["ticks"]
                    .as_u64()
                    .ok_or("ticks must be an unsigned integer")?;
                for _ in 0..ticks {
                    world.tick();
                }
            }
            _ => return Err("unknown remote player motion action".into()),
        }
        states.push(client_world_entity_player_motion_state(
            string(operation, "label")?,
            world
                .entity_at(entity_id)
                .ok_or("remote player motion fixture disappeared")?,
        ));
    }
    Ok(json!({"entity_id": entity_id, "states": states}))
}

pub(super) fn client_world_entity_teleport_state(label: &str, entity: &ClientEntity) -> Value {
    let mut state = client_world_entity_player_interpolation_state(label, entity);
    state["motion"] = client_world_entity_tick_vector(entity.motion());
    state
}

pub(super) fn client_world_entity_teleport_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let initial_position = f64_vec3(
        request
            .get("initial_position")
            .ok_or("initial_position must be a vector")?,
    )?;
    let initial_rotation = request
        .get("initial_rotation")
        .ok_or("initial_rotation must be an object")?;
    let initial_motion = f64_vec3(
        request
            .get("initial_motion")
            .ok_or("initial_motion must be a vector")?,
    )?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: 156,
        x: initial_position.x,
        y: initial_position.y,
        z: initial_position.z,
        movement: (initial_motion.x, initial_motion.y, initial_motion.z),
        rotation: (
            f32_field(initial_rotation, "y")?,
            f32_field(initial_rotation, "x")?,
            f32_field(request, "initial_head")?,
        ),
        data: 0,
    }) {
        return Err("remote entity teleport fixture was rejected".into());
    }

    let mut states = vec![client_world_entity_teleport_state(
        "initial",
        world
            .entity_at(entity_id)
            .ok_or("remote entity teleport fixture disappeared")?,
    )];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        match string(operation, "action")? {
            "teleport" => {
                let position = f64_vec3(
                    operation
                        .get("position")
                        .ok_or("teleport position must be a vector")?,
                )?;
                let delta_motion = f64_vec3(
                    operation
                        .get("delta_motion")
                        .ok_or("teleport delta motion must be a vector")?,
                )?;
                let rotation = operation
                    .get("rotation")
                    .ok_or("teleport rotation must be an object")?;
                if !world.apply_entity_teleport(EntityTeleportInput {
                    entity_id,
                    x: position.x,
                    y: position.y,
                    z: position.z,
                    delta_x: delta_motion.x,
                    delta_y: delta_motion.y,
                    delta_z: delta_motion.z,
                    y_rot: f32_field(rotation, "y")?,
                    x_rot: f32_field(rotation, "x")?,
                    relative_flags: i32_field(operation, "relative_flags")?,
                }) {
                    return Err("remote entity teleport was rejected".into());
                }
            }
            "tick" => {
                let ticks = operation["ticks"]
                    .as_u64()
                    .ok_or("ticks must be an unsigned integer")?;
                for _ in 0..ticks {
                    world.tick();
                }
            }
            _ => return Err("unknown remote entity teleport action".into()),
        }
        states.push(client_world_entity_teleport_state(
            string(operation, "label")?,
            world
                .entity_at(entity_id)
                .ok_or("remote entity teleport fixture disappeared")?,
        ));
    }
    Ok(json!({"entity_id": entity_id, "states": states}))
}

pub(super) fn client_world_entity_passengers_state(
    label: &str,
    accepted: bool,
    world: &ClientWorld,
    entity_ids: &[i32],
) -> Result<Value, String> {
    let mut entities = Vec::with_capacity(entity_ids.len());
    for &entity_id in entity_ids {
        let entity = world
            .entity_at(entity_id)
            .ok_or("passenger relation fixture entity disappeared")?;
        let mut state = json!({
            "entity_id": entity_id,
            "passenger_ids": entity.passenger_ids(),
        });
        if let Some(vehicle_id) = entity.vehicle_id() {
            state["vehicle_id"] = json!(vehicle_id);
        }
        entities.push(state);
    }
    Ok(json!({
        "label": label,
        "accepted": accepted,
        "entities": entities,
    }))
}
