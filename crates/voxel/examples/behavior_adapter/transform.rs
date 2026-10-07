use super::*;

pub(super) fn client_world_entity_tick_result(
    request: &Value,
    entity_type_id: i32,
) -> Result<Value, String> {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ) {
        return Err("entity tick fixture chunk was rejected".into());
    }

    let input_entities = request["entities"]
        .as_array()
        .ok_or("entities must be an array")?;
    let mut entity_ids = Vec::with_capacity(input_entities.len());
    for entry in input_entities {
        let entity_id = i32_field(entry, "entity_id")?;
        let position = f64_vec3(entry.get("position").ok_or("position must be a vector")?)?;
        let movement = f64_vec3(entry.get("movement").ok_or("movement must be a vector")?)?;
        let rotation = entry.get("rotation").ok_or("rotation must be an object")?;
        if !world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(0, u64::from(entity_id as u32)),
            type_id: entity_type_id,
            x: position.x,
            y: position.y,
            z: position.z,
            movement: (movement.x, movement.y, movement.z),
            rotation: (f32_field(rotation, "y")?, f32_field(rotation, "x")?, 0.0),
            data: 0,
        }) {
            return Err("entity tick fixture entity was rejected".into());
        }
        if entry["no_gravity"]
            .as_bool()
            .ok_or("no_gravity must be a boolean")?
            && !world.apply_entity_metadata(entity_id, &[5, 8, 1, 255])
        {
            return Err("entity tick no-gravity metadata was rejected".into());
        }
        entity_ids.push(entity_id);
    }

    let mut states = vec![client_world_entity_tick_state(
        "initial",
        true,
        &entity_ids,
        &world,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        if string(operation, "action")? != "tick" {
            return Err("unknown entity tick action".into());
        }
        let ticks = operation["ticks"]
            .as_u64()
            .ok_or("ticks must be an unsigned integer")?;
        for _ in 0..ticks {
            world.tick();
        }
        states.push(client_world_entity_tick_state(
            string(operation, "label")?,
            true,
            &entity_ids,
            &world,
        )?);
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_ticking_eligibility_state(
    label: &str,
    world: &ClientWorld,
    player_id: i32,
    remote_entity_id: i32,
) -> Result<Value, String> {
    let player = world
        .local_player()
        .ok_or("ticking eligibility local player disappeared")?;
    let remote = world
        .entities()
        .at(remote_entity_id)
        .ok_or("ticking eligibility remote entity disappeared")?;
    Ok(json!({
        "label": label,
        "accepted": true,
        "game_time": world.game_time(),
        "local_player": {
            "entity_id": player_id,
            "tick_count": player.tick_count(),
            "always_ticking": true,
            "ticking": true,
        },
        "entities": [{
            "entity_id": remote.id(),
            "tick_count": remote.tick_count(),
            "always_ticking": remote.is_always_ticking(),
            "ticking": remote.is_ticking(),
        }],
    }))
}

pub(super) fn client_world_entity_ticking_eligibility_result(
    request: &Value,
) -> Result<Value, String> {
    let player = request.get("player").ok_or("player must be an object")?;
    let player_id = i32_field(player, "entity_id")?;
    let player_position = f64_vec3(
        player
            .get("position")
            .ok_or("player position must be a vector")?,
    )?;
    let remote = request.get("remote").ok_or("remote must be an object")?;
    let remote_entity_id = i32_field(remote, "entity_id")?;
    let remote_position = f64_vec3(
        remote
            .get("position")
            .ok_or("remote position must be a vector")?,
    )?;
    let remote_type_id = i32_field(remote, "type_id")?;

    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        -3,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ) {
        return Err("ticking eligibility fixture chunk was rejected".into());
    }
    world.set_local_player_id(player_id);
    if !world.apply_local_player_position(
        (player_position.x, player_position.y, player_position.z),
        (0.0, 0.0, 0.0),
        (0.0, 0.0),
    ) {
        return Err("ticking eligibility local player was rejected".into());
    }
    if !world.apply_entity(EntityInput {
        entity_id: remote_entity_id,
        uuid: Uuid::new(0, u64::from(remote_entity_id as u32)),
        type_id: remote_type_id,
        x: remote_position.x,
        y: remote_position.y,
        z: remote_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("ticking eligibility remote entity was rejected".into());
    }

    let mut states = vec![client_world_entity_ticking_eligibility_state(
        "initial",
        &world,
        player_id,
        remote_entity_id,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        match string(operation, "action")? {
            "tick" => {
                let ticks = operation["ticks"]
                    .as_u64()
                    .ok_or("ticks must be an unsigned integer")?;
                for _ in 0..ticks {
                    world.tick();
                }
            }
            "unload_chunk" => {
                let chunk_x = i32_field(operation, "chunk_x")?;
                let chunk_z = i32_field(operation, "chunk_z")?;
                if !world.unload_chunk(chunk_x, chunk_z) {
                    return Err("ticking eligibility chunk unload was rejected".into());
                }
            }
            "load_chunk" => {
                let chunk_x = i32_field(operation, "chunk_x")?;
                let chunk_z = i32_field(operation, "chunk_z")?;
                if !world.replace_chunk(
                    chunk_x,
                    chunk_z,
                    [SectionInput {
                        non_empty_block_count: 0,
                        fluid_count: 0,
                        block_states: &blocks,
                        biomes: &biomes,
                    }],
                ) {
                    return Err("ticking eligibility chunk reload was rejected".into());
                }
            }
            _ => return Err("unknown ticking eligibility action".into()),
        }
        states.push(client_world_entity_ticking_eligibility_state(
            string(operation, "label")?,
            &world,
            player_id,
            remote_entity_id,
        )?);
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_transform_history_state(
    label: &str,
    world: &ClientWorld,
    player_id: i32,
    entity_id: i32,
    partial_tick: f32,
) -> Result<Value, String> {
    let player = world
        .local_player()
        .ok_or("transform history local player disappeared")?;
    let entity = world
        .entities()
        .at(entity_id)
        .ok_or("transform history entity disappeared")?;
    let player_position = player.position();
    let player_previous_position = player.previous_position();
    let player_rotation = player.rotation();
    let player_previous_rotation = player.previous_rotation();
    let player_interpolated_position = player.interpolated_position(partial_tick);
    let player_interpolated_rotation = player.interpolated_rotation(partial_tick);
    let entity_position = entity.position();
    let entity_previous_position = entity.previous_position();
    let entity_rotation = entity.rotation();
    let entity_previous_rotation = entity.previous_rotation();
    let entity_interpolated_position = entity.interpolated_position(partial_tick);
    let entity_interpolated_rotation = entity.interpolated_rotation(partial_tick);
    Ok(json!({
        "label": label,
        "accepted": true,
        "game_time": world.game_time(),
        "partial_tick": format!("{:08x}", partial_tick.to_bits()),
        "local_player": {
            "entity_id": player_id,
            "tick_count": player.tick_count(),
            "position": client_world_entity_tick_vector(player_position),
            "previous_position": client_world_entity_tick_vector(player_previous_position),
            "rotation_y": format!("{:08x}", player_rotation.0.to_bits()),
            "rotation_x": format!("{:08x}", player_rotation.1.to_bits()),
            "previous_rotation_y": format!("{:08x}", player_previous_rotation.0.to_bits()),
            "previous_rotation_x": format!("{:08x}", player_previous_rotation.1.to_bits()),
            "interpolated_position": client_world_entity_tick_vector(
                player_interpolated_position,
            ),
            "interpolated_rotation_y": format!(
                "{:08x}",
                player_interpolated_rotation.0.to_bits()
            ),
            "interpolated_rotation_x": format!(
                "{:08x}",
                player_interpolated_rotation.1.to_bits()
            ),
        },
        "entities": [{
            "entity_id": entity.id(),
            "tick_count": entity.tick_count(),
            "position": client_world_entity_tick_vector(entity_position),
            "previous_position": client_world_entity_tick_vector(entity_previous_position),
            "rotation_y": format!("{:08x}", entity_rotation.0.to_bits()),
            "rotation_x": format!("{:08x}", entity_rotation.1.to_bits()),
            "previous_rotation_y": format!("{:08x}", entity_previous_rotation.0.to_bits()),
            "previous_rotation_x": format!("{:08x}", entity_previous_rotation.1.to_bits()),
            "head_rotation": format!("{:08x}", entity_rotation.2.to_bits()),
            "previous_head_rotation": format!("{:08x}", entity_previous_rotation.2.to_bits()),
            "interpolated_position": client_world_entity_tick_vector(
                entity_interpolated_position,
            ),
            "interpolated_rotation_y": format!(
                "{:08x}",
                entity_interpolated_rotation.0.to_bits()
            ),
            "interpolated_rotation_x": format!(
                "{:08x}",
                entity_interpolated_rotation.1.to_bits()
            ),
            "interpolated_head_rotation": format!(
                "{:08x}",
                entity_interpolated_rotation.2.to_bits()
            ),
        }],
    }))
}

pub(super) fn client_world_entity_transform_history_result(
    request: &Value,
) -> Result<Value, String> {
    let player = request.get("player").ok_or("player must be an object")?;
    let player_id = i32_field(player, "entity_id")?;
    let player_position = f64_vec3(
        player
            .get("position")
            .ok_or("player position must be a vector")?,
    )?;
    let player_rotation = player
        .get("rotation")
        .ok_or("player rotation must be an object")?;
    let remote = request.get("entity").ok_or("entity must be an object")?;
    let entity_id = i32_field(remote, "entity_id")?;
    let entity_type_id = i32_field(remote, "type_id")?;
    let entity_position = f64_vec3(
        remote
            .get("position")
            .ok_or("entity position must be a vector")?,
    )?;
    let entity_rotation = remote
        .get("rotation")
        .ok_or("entity rotation must be an object")?;

    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        -3,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ) {
        return Err("transform history fixture chunk was rejected".into());
    }
    world.set_local_player_id(player_id);
    if !world.apply_local_player_position(
        (player_position.x, player_position.y, player_position.z),
        (0.0, 0.0, 0.0),
        (
            f32_field(player_rotation, "y")?,
            f32_field(player_rotation, "x")?,
        ),
    ) {
        return Err("transform history local player was rejected".into());
    }
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: entity_type_id,
        x: entity_position.x,
        y: entity_position.y,
        z: entity_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (
            f32_field(entity_rotation, "y")?,
            f32_field(entity_rotation, "x")?,
            f32_field(entity_rotation, "head")?,
        ),
        data: 0,
    }) {
        return Err("transform history entity was rejected".into());
    }

    let mut states = vec![client_world_entity_transform_history_state(
        "initial", &world, player_id, entity_id, 0.5,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        match string(operation, "action")? {
            "tick" => {
                let ticks = operation["ticks"]
                    .as_u64()
                    .ok_or("ticks must be an unsigned integer")?;
                for _ in 0..ticks {
                    world.tick();
                }
            }
            "update" => {
                let updated_player = operation
                    .get("player")
                    .ok_or("updated player must be an object")?;
                let updated_player_position = f64_vec3(
                    updated_player
                        .get("position")
                        .ok_or("updated player position must be a vector")?,
                )?;
                let updated_player_rotation = updated_player
                    .get("rotation")
                    .ok_or("updated player rotation must be an object")?;
                if !world.apply_local_player_position(
                    (
                        updated_player_position.x,
                        updated_player_position.y,
                        updated_player_position.z,
                    ),
                    (0.0, 0.0, 0.0),
                    (
                        f32_field(updated_player_rotation, "y")?,
                        f32_field(updated_player_rotation, "x")?,
                    ),
                ) {
                    return Err("updated transform history local player was rejected".into());
                }

                let updated_entity = operation
                    .get("entity")
                    .ok_or("updated entity must be an object")?;
                let updated_entity_position = f64_vec3(
                    updated_entity
                        .get("position")
                        .ok_or("updated entity position must be a vector")?,
                )?;
                if !world.update_entity_position(
                    entity_id,
                    (
                        updated_entity_position.x,
                        updated_entity_position.y,
                        updated_entity_position.z,
                    ),
                ) {
                    return Err("updated transform history entity was rejected".into());
                }
                if !world.apply_entity_head_rotation(entity_id, f32_field(updated_entity, "head")?)
                {
                    return Err("updated transform history head rotation was rejected".into());
                }
            }
            "sample" => {}
            _ => return Err("unknown transform history action".into()),
        }
        let partial_tick = if operation.get("partial_tick").is_some() {
            f32_field(operation, "partial_tick")?
        } else {
            0.5
        };
        states.push(client_world_entity_transform_history_state(
            string(operation, "label")?,
            &world,
            player_id,
            entity_id,
            partial_tick,
        )?);
    }
    Ok(json!({"states": states}))
}
