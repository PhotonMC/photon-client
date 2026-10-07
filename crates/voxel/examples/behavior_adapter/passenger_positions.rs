use super::*;

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

pub(super) fn client_world_entity_passengers_result(request: &Value) -> Result<Value, String> {
    let entity_entries = request["entities"]
        .as_array()
        .ok_or("entities must be an array")?;
    let entity_ids: Vec<i32> = entity_entries
        .iter()
        .map(|entity| i32_field(entity, "entity_id"))
        .collect::<Result<_, _>>()?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    for entity in entity_entries {
        let entity_id = i32_field(entity, "entity_id")?;
        let type_id = i32_field(entity, "type_id")?;
        if !world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(0, u64::from(entity_id as u32)),
            type_id,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }) {
            return Err("passenger relation fixture entity was rejected".into());
        }
    }

    for link in request["initial_links"]
        .as_array()
        .ok_or("initial_links must be an array")?
    {
        let vehicle_id = i32_field(link, "vehicle_id")?;
        let passenger_ids = link["passenger_ids"]
            .as_array()
            .ok_or("initial passenger_ids must be an array")?
            .iter()
            .map(|passenger| {
                passenger
                    .as_i64()
                    .and_then(|value| i32::try_from(value).ok())
                    .ok_or("initial passenger ID must be a 32-bit integer".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !world.apply_entity_passengers(vehicle_id, &passenger_ids) {
            return Err("initial passenger relation was rejected".into());
        }
    }

    let mut states = vec![client_world_entity_passengers_state(
        "initial",
        true,
        &world,
        &entity_ids,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        let vehicle_id = i32_field(operation, "vehicle_id")?;
        let passenger_ids = operation["passenger_ids"]
            .as_array()
            .ok_or("passenger_ids must be an array")?
            .iter()
            .map(|passenger| {
                passenger
                    .as_i64()
                    .and_then(|value| i32::try_from(value).ok())
                    .ok_or("passenger ID must be a 32-bit integer".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let accepted = world.apply_entity_passengers(vehicle_id, &passenger_ids);
        states.push(client_world_entity_passengers_state(
            string(operation, "label")?,
            accepted,
            &world,
            &entity_ids,
        )?);
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_passenger_tick_result(request: &Value) -> Result<Value, String> {
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
        return Err("passenger tick fixture chunk was rejected".into());
    }

    let vehicle = request["vehicle"]
        .as_object()
        .ok_or("vehicle must be an object")?;
    let passenger = request["passenger"]
        .as_object()
        .ok_or("passenger must be an object")?;
    let root = request["root_entity"]
        .as_object()
        .ok_or("root_entity must be an object")?;
    let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
    let passenger_id = i32_field(&Value::Object(passenger.clone()), "entity_id")?;
    let root_id = i32_field(&Value::Object(root.clone()), "entity_id")?;
    let entity_ids = [vehicle_id, passenger_id, root_id];

    for input in [vehicle, passenger, root] {
        let entity_id = i32_field(&Value::Object(input.clone()), "entity_id")?;
        let type_id = i32_field(&Value::Object(input.clone()), "type_id")?;
        let position = f64_vec3(
            input
                .get("position")
                .ok_or("passenger tick position must be a vector")?,
        )?;
        let movement = input
            .get("movement")
            .map(f64_vec3)
            .transpose()?
            .unwrap_or(Vec3::new(0.0, 0.0, 0.0));
        let rotation = if let Some(value) = input.get("rotation") {
            (f32_field(value, "y")?, f32_field(value, "x")?, 0.0)
        } else {
            (0.0, 0.0, 0.0)
        };
        if !world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(0, u64::from(entity_id as u32)),
            type_id,
            x: position.x,
            y: position.y,
            z: position.z,
            movement: (movement.x, movement.y, movement.z),
            rotation,
            data: 0,
        }) {
            return Err("passenger tick fixture entity was rejected".into());
        }
    }
    if !world.apply_entity_passengers(vehicle_id, &[passenger_id]) {
        return Err("passenger tick fixture relation was rejected".into());
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
            return Err("unknown passenger tick action".into());
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

pub(super) fn client_world_entity_passenger_position_result(
    request: &Value,
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
        return Err("passenger position fixture chunk was rejected".into());
    }

    let vehicle = request["vehicle"]
        .as_object()
        .ok_or("vehicle must be an object")?;
    if i32_field(&Value::Object(vehicle.clone()), "type_id")? != 89 {
        return Err("passenger position fixture requires an oak boat".into());
    }
    let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
    let vehicle_position = f64_vec3(
        vehicle
            .get("position")
            .ok_or("passenger position vehicle position must be a vector")?,
    )?;
    let vehicle_rotation = vehicle
        .get("rotation")
        .ok_or("passenger position vehicle rotation must be an object")?;
    let vehicle_y_rot = f32_field(vehicle_rotation, "y")?;
    let mut entity_ids = vec![vehicle_id];
    for passenger in request["passengers"]
        .as_array()
        .ok_or("passengers must be an array")?
    {
        let passenger_id = i32_field(passenger, "entity_id")?;
        if i32_field(passenger, "type_id")? != 120 {
            return Err("passenger position fixture requires snowball passengers".into());
        }
        let position = f64_vec3(
            passenger
                .get("position")
                .ok_or("passenger position must be a vector")?,
        )?;
        let movement = f64_vec3(
            passenger
                .get("movement")
                .ok_or("passenger movement must be a vector")?,
        )?;
        let rotation = passenger
            .get("rotation")
            .ok_or("passenger rotation must be an object")?;
        if !world.apply_entity(EntityInput {
            entity_id: passenger_id,
            uuid: Uuid::new(0, u64::from(passenger_id as u32)),
            type_id: 120,
            x: position.x,
            y: position.y,
            z: position.z,
            movement: (movement.x, movement.y, movement.z),
            rotation: (f32_field(rotation, "y")?, f32_field(rotation, "x")?, 0.0),
            data: 0,
        }) {
            return Err("passenger position fixture entity was rejected".into());
        }
        entity_ids.push(passenger_id);
    }
    if !world.apply_entity(EntityInput {
        entity_id: vehicle_id,
        uuid: Uuid::new(0, u64::from(vehicle_id as u32)),
        type_id: 89,
        x: vehicle_position.x,
        y: vehicle_position.y,
        z: vehicle_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (vehicle_y_rot, f32_field(vehicle_rotation, "x")?, 0.0),
        data: 0,
    }) {
        return Err("passenger position vehicle was rejected".into());
    }

    let passenger_ids = request["initial_passenger_ids"]
        .as_array()
        .ok_or("initial_passenger_ids must be an array")?
        .iter()
        .map(|value| {
            value
                .as_i64()
                .ok_or_else(|| "initial passenger ID must be an integer".to_owned())
                .and_then(|value| {
                    value
                        .try_into()
                        .map_err(|_| "initial passenger ID must fit in i32".to_owned())
                })
        })
        .collect::<Result<Vec<i32>, _>>()?;
    if !world.apply_entity_passengers(vehicle_id, &passenger_ids) {
        return Err("passenger position fixture relation was rejected".into());
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
        match string(operation, "action")? {
            "tick" => {}
            "set-passengers-and-tick" => {
                let passenger_ids = operation["passenger_ids"]
                    .as_array()
                    .ok_or("passenger_ids must be an array")?
                    .iter()
                    .map(|value| {
                        value
                            .as_i64()
                            .ok_or_else(|| "passenger ID must be an integer".to_owned())
                            .and_then(|value| {
                                value
                                    .try_into()
                                    .map_err(|_| "passenger ID must fit in i32".to_owned())
                            })
                    })
                    .collect::<Result<Vec<i32>, _>>()?;
                if !world.apply_entity_passengers(vehicle_id, &passenger_ids) {
                    return Err("passenger position fixture relation was rejected".into());
                }
            }
            _ => return Err("unknown passenger position action".into()),
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

pub(super) fn client_world_entity_passenger_player_position_result(
    request: &Value,
) -> Result<Value, String> {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            block_states: &blocks,
            biomes: &biomes,
            fluid_count: 0,
        }],
    ) {
        return Err("passenger player position fixture chunk was rejected".into());
    }

    let vehicle = request["vehicle"]
        .as_object()
        .ok_or("vehicle must be an object")?;
    if i32_field(&Value::Object(vehicle.clone()), "type_id")? != 89 {
        return Err("passenger player position fixture requires an oak boat".into());
    }
    let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
    let vehicle_position = f64_vec3(
        vehicle
            .get("position")
            .ok_or("passenger player position vehicle position must be a vector")?,
    )?;
    let vehicle_rotation = vehicle
        .get("rotation")
        .ok_or("passenger player position vehicle rotation must be an object")?;
    if !world.apply_entity(EntityInput {
        entity_id: vehicle_id,
        uuid: Uuid::new(0, u64::from(vehicle_id as u32)),
        type_id: 89,
        x: vehicle_position.x,
        y: vehicle_position.y,
        z: vehicle_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (
            f32_field(vehicle_rotation, "y")?,
            f32_field(vehicle_rotation, "x")?,
            0.0,
        ),
        data: 0,
    }) {
        return Err("passenger player position vehicle was rejected".into());
    }
    if vehicle["no_gravity"]
        .as_bool()
        .ok_or("vehicle no_gravity must be a boolean")?
        && !world.apply_entity_metadata(vehicle_id, &[5, 8, 1, 255])
    {
        return Err("passenger player position vehicle no-gravity was rejected".into());
    }

    let passenger = request["passenger"]
        .as_object()
        .ok_or("passenger must be an object")?;
    if i32_field(&Value::Object(passenger.clone()), "type_id")? != 156 {
        return Err("passenger player position fixture requires a Player passenger".into());
    }
    let passenger_id = i32_field(&Value::Object(passenger.clone()), "entity_id")?;
    let passenger_position = f64_vec3(
        passenger
            .get("position")
            .ok_or("passenger player position must be a vector")?,
    )?;
    let passenger_movement = f64_vec3(
        passenger
            .get("movement")
            .ok_or("passenger player movement must be a vector")?,
    )?;
    let passenger_rotation = passenger
        .get("rotation")
        .ok_or("passenger player rotation must be an object")?;
    if !world.apply_entity(EntityInput {
        entity_id: passenger_id,
        uuid: Uuid::new(0, u64::from(passenger_id as u32)),
        type_id: 156,
        x: passenger_position.x,
        y: passenger_position.y,
        z: passenger_position.z,
        movement: (
            passenger_movement.x,
            passenger_movement.y,
            passenger_movement.z,
        ),
        rotation: (
            f32_field(passenger_rotation, "y")?,
            f32_field(passenger_rotation, "x")?,
            0.0,
        ),
        data: 0,
    }) {
        return Err("passenger player position fixture entity was rejected".into());
    }
    if passenger["no_gravity"]
        .as_bool()
        .ok_or("passenger no_gravity must be a boolean")?
        && !world.apply_entity_metadata(passenger_id, &[5, 8, 1, 255])
    {
        return Err("passenger player position no-gravity was rejected".into());
    }
    if !world.apply_entity_passengers(vehicle_id, &[passenger_id]) {
        return Err("passenger player position fixture relation was rejected".into());
    }

    let entity_ids = [vehicle_id, passenger_id];
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
            return Err("unknown passenger player position action".into());
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
