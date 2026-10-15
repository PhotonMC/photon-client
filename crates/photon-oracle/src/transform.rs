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
        let entity_id = entity_id_field(entry, "entity_id")?;
        let position = f64_vec3(entry.get("position").ok_or("position must be a vector")?)?;
        let movement = f64_vec3(entry.get("movement").ok_or("movement must be a vector")?)?;
        let rotation = entry.get("rotation").ok_or("rotation must be an object")?;
        if !world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(0, u64::from(entity_id.0 as u32)),
            type_id: photon_core::ids::EntityTypeId(entity_type_id),
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
    player_id: EntityId,
    remote_entity_id: EntityId,
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
            "entity_id": player_id.0,
            "tick_count": player.tick_count(),
            "always_ticking": true,
            "ticking": true,
        },
        "entities": [{
            "entity_id": remote.id().0,
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
    let player_id = entity_id_field(player, "entity_id")?;
    let player_position = f64_vec3(
        player
            .get("position")
            .ok_or("player position must be a vector")?,
    )?;
    let remote = request.get("remote").ok_or("remote must be an object")?;
    let remote_entity_id = entity_id_field(remote, "entity_id")?;
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
        uuid: Uuid::new(0, u64::from(remote_entity_id.0 as u32)),
        type_id: photon_core::ids::EntityTypeId(remote_type_id),
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
