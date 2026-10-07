use super::*;

pub(super) fn client_world_entity_passenger_animal_rotation_result(
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
        return Err("passenger animal rotation fixture chunk was rejected".into());
    }

    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut entity_ids = Vec::with_capacity(rows.len() * 3);
    for row in rows {
        let vehicle = row
            .get("vehicle")
            .and_then(Value::as_object)
            .ok_or("passenger animal rotation vehicle must be an object")?;
        let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
        if i32_field(&Value::Object(vehicle.clone()), "type_id")? != 89 {
            return Err("passenger animal rotation fixture requires an oak boat".into());
        }
        let vehicle_position = f64_vec3(
            vehicle
                .get("position")
                .ok_or("passenger animal rotation vehicle position must be a vector")?,
        )?;
        let vehicle_rotation = vehicle
            .get("rotation")
            .ok_or("passenger animal rotation vehicle rotation must be an object")?;
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
            return Err("passenger animal rotation vehicle was rejected".into());
        }
        if vehicle["no_gravity"]
            .as_bool()
            .ok_or("passenger animal rotation vehicle no_gravity must be a boolean")?
            && !world.apply_entity_metadata(vehicle_id, &[5, 8, 1, 255])
        {
            return Err("passenger animal rotation vehicle no-gravity was rejected".into());
        }

        let animal = row
            .get("animal")
            .and_then(Value::as_object)
            .ok_or("passenger animal rotation animal must be an object")?;
        let animal_id = i32_field(&Value::Object(animal.clone()), "entity_id")?;
        if i32_field(&Value::Object(animal.clone()), "type_id")? != 30 {
            return Err("passenger animal rotation fixture requires a Cow".into());
        }
        let animal_position = f64_vec3(
            animal
                .get("position")
                .ok_or("passenger animal rotation animal position must be a vector")?,
        )?;
        let animal_movement = f64_vec3(
            animal
                .get("movement")
                .ok_or("passenger animal rotation animal movement must be a vector")?,
        )?;
        let animal_rotation = animal
            .get("rotation")
            .ok_or("passenger animal rotation animal rotation must be an object")?;
        if !world.apply_entity(EntityInput {
            entity_id: animal_id,
            uuid: Uuid::new(0, u64::from(animal_id as u32)),
            type_id: 30,
            x: animal_position.x,
            y: animal_position.y,
            z: animal_position.z,
            movement: (animal_movement.x, animal_movement.y, animal_movement.z),
            rotation: (
                f32_field(animal_rotation, "y")?,
                f32_field(animal_rotation, "x")?,
                f32_field(animal_rotation, "y")?,
            ),
            data: 0,
        }) {
            return Err("passenger animal rotation animal was rejected".into());
        }
        if animal["no_gravity"]
            .as_bool()
            .ok_or("passenger animal rotation animal no_gravity must be a boolean")?
            && !world.apply_entity_metadata(animal_id, &[5, 8, 1, 255])
        {
            return Err("passenger animal rotation animal no-gravity was rejected".into());
        }

        let filler = row
            .get("filler")
            .and_then(Value::as_object)
            .ok_or("passenger animal rotation filler must be an object")?;
        let filler_id = i32_field(&Value::Object(filler.clone()), "entity_id")?;
        if i32_field(&Value::Object(filler.clone()), "type_id")? != 120 {
            return Err("passenger animal rotation fixture requires a Snowball filler".into());
        }
        let filler_position = f64_vec3(
            filler
                .get("position")
                .ok_or("passenger animal rotation filler position must be a vector")?,
        )?;
        let filler_movement = f64_vec3(
            filler
                .get("movement")
                .ok_or("passenger animal rotation filler movement must be a vector")?,
        )?;
        let filler_rotation = filler
            .get("rotation")
            .ok_or("passenger animal rotation filler rotation must be an object")?;
        if !world.apply_entity(EntityInput {
            entity_id: filler_id,
            uuid: Uuid::new(0, u64::from(filler_id as u32)),
            type_id: 120,
            x: filler_position.x,
            y: filler_position.y,
            z: filler_position.z,
            movement: (filler_movement.x, filler_movement.y, filler_movement.z),
            rotation: (
                f32_field(filler_rotation, "y")?,
                f32_field(filler_rotation, "x")?,
                f32_field(filler_rotation, "y")?,
            ),
            data: 0,
        }) {
            return Err("passenger animal rotation filler was rejected".into());
        }
        if filler["no_gravity"]
            .as_bool()
            .ok_or("passenger animal rotation filler no_gravity must be a boolean")?
            && !world.apply_entity_metadata(filler_id, &[5, 8, 1, 255])
        {
            return Err("passenger animal rotation filler no-gravity was rejected".into());
        }
        if !world.apply_entity_passengers(vehicle_id, &[animal_id, filler_id]) {
            return Err("passenger animal rotation relation was rejected".into());
        }
        entity_ids.extend([vehicle_id, animal_id, filler_id]);
    }

    let mut states = vec![client_world_entity_passenger_animal_rotation_state(
        "initial",
        &entity_ids,
        &world,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        if string(operation, "action")? != "tick" {
            return Err("unknown passenger animal rotation action".into());
        }
        let ticks = operation["ticks"]
            .as_u64()
            .ok_or("ticks must be an unsigned integer")?;
        for _ in 0..ticks {
            world.tick();
        }
        states.push(client_world_entity_passenger_animal_rotation_state(
            string(operation, "label")?,
            &entity_ids,
            &world,
        )?);
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_passenger_animal_position_result(
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
        return Err("passenger animal position fixture chunk was rejected".into());
    }

    let vehicle = request["vehicle"]
        .as_object()
        .ok_or("vehicle must be an object")?;
    let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
    if i32_field(&Value::Object(vehicle.clone()), "type_id")? != 89 {
        return Err("passenger animal position fixture requires an oak boat".into());
    }
    let vehicle_position = f64_vec3(
        vehicle
            .get("position")
            .ok_or("passenger animal position vehicle position must be a vector")?,
    )?;
    let vehicle_rotation = vehicle
        .get("rotation")
        .ok_or("passenger animal position vehicle rotation must be an object")?;
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
        return Err("passenger animal position vehicle was rejected".into());
    }
    if vehicle["no_gravity"]
        .as_bool()
        .ok_or("passenger animal position vehicle no_gravity must be a boolean")?
        && !world.apply_entity_metadata(vehicle_id, &[5, 8, 1, 255])
    {
        return Err("passenger animal position vehicle no-gravity was rejected".into());
    }

    let animal = request["animal"]
        .as_object()
        .ok_or("animal must be an object")?;
    let animal_id = i32_field(&Value::Object(animal.clone()), "entity_id")?;
    if i32_field(&Value::Object(animal.clone()), "type_id")? != 30 {
        return Err("passenger animal position fixture requires a Cow".into());
    }
    let animal_position = f64_vec3(
        animal
            .get("position")
            .ok_or("passenger animal position animal position must be a vector")?,
    )?;
    let animal_movement = f64_vec3(
        animal
            .get("movement")
            .ok_or("passenger animal position animal movement must be a vector")?,
    )?;
    let animal_rotation = animal
        .get("rotation")
        .ok_or("passenger animal position animal rotation must be an object")?;
    if !world.apply_entity(EntityInput {
        entity_id: animal_id,
        uuid: Uuid::new(0, u64::from(animal_id as u32)),
        type_id: 30,
        x: animal_position.x,
        y: animal_position.y,
        z: animal_position.z,
        movement: (animal_movement.x, animal_movement.y, animal_movement.z),
        rotation: (
            f32_field(animal_rotation, "y")?,
            f32_field(animal_rotation, "x")?,
            f32_field(animal_rotation, "y")?,
        ),
        data: 0,
    }) {
        return Err("passenger animal position animal was rejected".into());
    }
    if animal["no_gravity"]
        .as_bool()
        .ok_or("passenger animal position animal no_gravity must be a boolean")?
        && !world.apply_entity_metadata(animal_id, &[5, 8, 1, 255])
    {
        return Err("passenger animal position animal no-gravity was rejected".into());
    }
    if !world.apply_entity_passengers(vehicle_id, &[animal_id]) {
        return Err("passenger animal position relation was rejected".into());
    }

    let entity_ids = [vehicle_id, animal_id];
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
            return Err("unknown passenger animal position action".into());
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
