use super::*;

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

pub(super) fn client_world_entity_passenger_boat_family_position_result(
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
        return Err("boat family position fixture chunk was rejected".into());
    }

    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut entity_ids = Vec::with_capacity(rows.len() * 2);
    for row in rows {
        let vehicle = row
            .get("vehicle")
            .and_then(Value::as_object)
            .ok_or("boat family vehicle must be an object")?;
        let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
        let vehicle_type_id = i32_field(&Value::Object(vehicle.clone()), "type_id")?;
        if !matches!(vehicle_type_id, 8 | 9 | 89 | 90) {
            return Err("unknown boat family position vehicle type".into());
        }
        let vehicle_position = f64_vec3(
            vehicle
                .get("position")
                .ok_or("boat family vehicle position must be a vector")?,
        )?;
        let vehicle_rotation = vehicle
            .get("rotation")
            .ok_or("boat family vehicle rotation must be an object")?;
        let vehicle_no_gravity = vehicle
            .get("no_gravity")
            .and_then(Value::as_bool)
            .ok_or("boat family vehicle no_gravity must be a boolean")?;
        if !world.apply_entity(EntityInput {
            entity_id: vehicle_id,
            uuid: Uuid::new(0, u64::from(vehicle_id as u32)),
            type_id: vehicle_type_id,
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
            return Err("boat family vehicle was rejected".into());
        }
        if vehicle_no_gravity && !world.apply_entity_metadata(vehicle_id, &[5, 8, 1, 255]) {
            return Err("boat family vehicle no-gravity was rejected".into());
        }

        let passenger = row
            .get("passenger")
            .and_then(Value::as_object)
            .ok_or("boat family passenger must be an object")?;
        let passenger_id = i32_field(&Value::Object(passenger.clone()), "entity_id")?;
        if i32_field(&Value::Object(passenger.clone()), "type_id")? != 120 {
            return Err("boat family position fixture requires snowball passengers".into());
        }
        let passenger_position = f64_vec3(
            passenger
                .get("position")
                .ok_or("boat family passenger position must be a vector")?,
        )?;
        let passenger_movement = f64_vec3(
            passenger
                .get("movement")
                .ok_or("boat family passenger movement must be a vector")?,
        )?;
        let passenger_rotation = passenger
            .get("rotation")
            .ok_or("boat family passenger rotation must be an object")?;
        let passenger_no_gravity = passenger
            .get("no_gravity")
            .and_then(Value::as_bool)
            .ok_or("boat family passenger no_gravity must be a boolean")?;
        if !world.apply_entity(EntityInput {
            entity_id: passenger_id,
            uuid: Uuid::new(0, u64::from(passenger_id as u32)),
            type_id: 120,
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
            return Err("boat family passenger was rejected".into());
        }
        if passenger_no_gravity && !world.apply_entity_metadata(passenger_id, &[5, 8, 1, 255]) {
            return Err("boat family passenger no-gravity was rejected".into());
        }
        if !world.apply_entity_passengers(vehicle_id, &[passenger_id]) {
            return Err("boat family passenger relation was rejected".into());
        }
        entity_ids.push(vehicle_id);
        entity_ids.push(passenger_id);
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
            return Err("unknown boat family position action".into());
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

pub(super) fn client_world_entity_passenger_rotation_state(
    label: &str,
    entity_ids: &[i32],
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
            .entity_at(*entity_id)
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

pub(super) fn client_world_entity_passenger_animal_rotation_state(
    label: &str,
    entity_ids: &[i32],
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
            .entity_at(*animal_id)
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
