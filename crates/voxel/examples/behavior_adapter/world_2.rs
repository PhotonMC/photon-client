use super::*;

pub(super) fn client_world_entity_passenger_turn_result(request: &Value) -> Result<Value, String> {
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
        return Err("passenger turn fixture chunk was rejected".into());
    }

    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut entity_ids = Vec::with_capacity(rows.len() * 2);
    let mut passenger_type_ids = Vec::with_capacity(rows.len());
    for row in rows {
        let vehicle = row
            .get("vehicle")
            .and_then(Value::as_object)
            .ok_or("passenger turn vehicle must be an object")?;
        let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
        if i32_field(&Value::Object(vehicle.clone()), "type_id")? != 89 {
            return Err("passenger turn fixture requires an oak boat".into());
        }
        let vehicle_position = f64_vec3(
            vehicle
                .get("position")
                .ok_or("passenger turn vehicle position must be a vector")?,
        )?;
        let vehicle_rotation = vehicle
            .get("rotation")
            .ok_or("passenger turn vehicle rotation must be an object")?;
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
            return Err("passenger turn vehicle was rejected".into());
        }
        if vehicle["no_gravity"]
            .as_bool()
            .ok_or("passenger turn vehicle no_gravity must be a boolean")?
            && !world.apply_entity_metadata(vehicle_id, &[5, 8, 1, 255])
        {
            return Err("passenger turn vehicle no-gravity was rejected".into());
        }

        let passenger = row
            .get("passenger")
            .and_then(Value::as_object)
            .ok_or("passenger turn passenger must be an object")?;
        let passenger_id = i32_field(&Value::Object(passenger.clone()), "entity_id")?;
        let passenger_type_id = i32_field(&Value::Object(passenger.clone()), "type_id")?;
        if !matches!(passenger_type_id, 17 | 156) {
            return Err("passenger turn fixture requires a Player or Breeze passenger".into());
        }
        let passenger_position = f64_vec3(
            passenger
                .get("position")
                .ok_or("passenger turn passenger position must be a vector")?,
        )?;
        let passenger_movement = f64_vec3(
            passenger
                .get("movement")
                .ok_or("passenger turn passenger movement must be a vector")?,
        )?;
        let passenger_rotation = passenger
            .get("rotation")
            .ok_or("passenger turn passenger rotation must be an object")?;
        if !world.apply_entity(EntityInput {
            entity_id: passenger_id,
            uuid: Uuid::new(0, u64::from(passenger_id as u32)),
            type_id: passenger_type_id,
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
                f32_field(passenger_rotation, "y")?,
            ),
            data: 0,
        }) {
            return Err("passenger turn passenger was rejected".into());
        }
        if passenger["no_gravity"]
            .as_bool()
            .ok_or("passenger turn passenger no_gravity must be a boolean")?
            && !world.apply_entity_metadata(passenger_id, &[5, 8, 1, 255])
        {
            return Err("passenger turn passenger no-gravity was rejected".into());
        }
        if !world.apply_entity_passengers(vehicle_id, &[passenger_id]) {
            return Err("passenger turn relation was rejected".into());
        }
        entity_ids.extend([vehicle_id, passenger_id]);
        passenger_type_ids.push(passenger_type_id);
    }

    let mut states = vec![client_world_entity_passenger_rotation_state(
        "initial",
        &entity_ids,
        &passenger_type_ids,
        &world,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        if string(operation, "action")? != "turn" {
            return Err("unknown passenger turn action".into());
        }
        let mouse_x = operation["mouse_x"]
            .as_f64()
            .ok_or("mouse_x must be a number")?;
        let mouse_y = operation["mouse_y"]
            .as_f64()
            .ok_or("mouse_y must be a number")?;
        for index in 0..passenger_type_ids.len() {
            let passenger_id = entity_ids
                .get(index * 2 + 1)
                .copied()
                .ok_or("passenger turn entity list is incomplete")?;
            if !world.turn_entity(passenger_id, mouse_x, mouse_y) {
                return Err("passenger turn operation was rejected".into());
            }
        }
        states.push(client_world_entity_passenger_rotation_state(
            string(operation, "label")?,
            &entity_ids,
            &passenger_type_ids,
            &world,
        )?);
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_entity_arrow_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_tick_result(request, 6)
}

pub(super) fn client_world_entity_experience_orb_tick_result(
    request: &Value,
) -> Result<Value, String> {
    client_world_entity_tick_result(request, 49)
}

pub(super) fn client_world_entity_primed_tnt_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_tick_result(request, 133)
}

pub(super) fn client_world_entity_falling_block_tick_result(
    request: &Value,
) -> Result<Value, String> {
    client_world_entity_tick_result(request, 51)
}

pub(super) fn client_world_entity_firework_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_tick_result(request, 53)
}

pub(super) fn client_world_entity_boat_tick_state(
    label: &str,
    world: &ClientWorld,
    entity_id: i32,
) -> Result<Value, String> {
    let entity = world
        .entity_at(entity_id)
        .ok_or_else(|| "boat tick fixture entity disappeared".to_owned())?;
    let rotation = entity.rotation();
    let previous_rotation = entity.previous_rotation();
    Ok(json!({
        "label": label,
        "accepted": true,
        "entities": [{
            "entity_id": entity.id(),
            "tick_count": entity.tick_count(),
            "position": client_world_entity_tick_vector(entity.position()),
            "previous_position": client_world_entity_tick_vector(entity.previous_position()),
            "motion": client_world_entity_tick_vector(entity.motion()),
            "rotation_y": format!("{:08x}", rotation.0.to_bits()),
            "rotation_x": format!("{:08x}", rotation.1.to_bits()),
            "previous_rotation_y": format!("{:08x}", previous_rotation.0.to_bits()),
            "previous_rotation_x": format!("{:08x}", previous_rotation.1.to_bits()),
            "no_gravity": entity.is_no_gravity(),
            "delta_rotation": format!("{:08x}", entity.boat_delta_rotation().to_bits()),
            "paddle_left": entity.paddle_state().0,
            "paddle_right": entity.paddle_state().1,
        }],
    }))
}

pub(super) fn client_world_entity_boat_air_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_boat_tick_result(request, true)
}

pub(super) fn client_world_entity_boat_remote_tick_result(
    request: &Value,
) -> Result<Value, String> {
    client_world_entity_boat_tick_result(request, false)
}

pub(super) fn client_world_entity_boat_tick_result(
    request: &Value,
    local_authority: bool,
) -> Result<Value, String> {
    let (passenger_id, passenger_type_id) = if local_authority {
        (i32_field(request, "player_id")?, 156)
    } else {
        let passenger = request
            .get("passenger")
            .ok_or("passenger must be an object")?;
        (
            i32_field(passenger, "entity_id")?,
            i32_field(passenger, "type_id")?,
        )
    };
    let boat = request.get("boat").ok_or("boat must be an object")?;
    let boat_entity_id = i32_field(boat, "entity_id")?;
    let position = f64_vec3(
        boat.get("position")
            .ok_or("boat position must be a vector")?,
    )?;
    let movement = f64_vec3(
        boat.get("movement")
            .ok_or("boat movement must be a vector")?,
    )?;
    let rotation = boat
        .get("rotation")
        .ok_or("boat rotation must be an object")?;
    let no_gravity = boat["no_gravity"]
        .as_bool()
        .ok_or("boat no_gravity must be a boolean")?;
    let input = request.get("input").ok_or("input must be an object")?;
    let boat_input = BoatInput::new(
        input["left"]
            .as_bool()
            .ok_or("input left must be a boolean")?,
        input["right"]
            .as_bool()
            .ok_or("input right must be a boolean")?,
        input["forward"]
            .as_bool()
            .ok_or("input forward must be a boolean")?,
        input["backward"]
            .as_bool()
            .ok_or("input backward must be a boolean")?,
    );

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
        return Err("boat air tick fixture chunk was rejected".into());
    }
    if !world.apply_entity(EntityInput {
        entity_id: boat_entity_id,
        uuid: Uuid::new(0, u64::from(boat_entity_id as u32)),
        type_id: 89,
        x: position.x,
        y: position.y,
        z: position.z,
        movement: (movement.x, movement.y, movement.z),
        rotation: (f32_field(rotation, "y")?, f32_field(rotation, "x")?, 0.0),
        data: 0,
    }) {
        return Err("boat air tick fixture entity was rejected".into());
    }
    if !world.apply_entity(EntityInput {
        entity_id: passenger_id,
        uuid: Uuid::new(0, u64::from(passenger_id as u32)),
        type_id: passenger_type_id,
        x: position.x,
        y: position.y,
        z: position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("boat tick fixture passenger was rejected".into());
    }
    if local_authority {
        world.set_local_player_id(passenger_id);
    }
    if !world.apply_entity_passengers(boat_entity_id, &[passenger_id]) {
        return Err("boat tick fixture passenger relation was rejected".into());
    }
    if !world.set_boat_input(boat_entity_id, boat_input) {
        return Err("boat tick fixture input was rejected".into());
    }
    if no_gravity && !world.apply_entity_metadata(boat_entity_id, &[5, 8, 1, 255]) {
        return Err("boat tick fixture no-gravity metadata was rejected".into());
    }

    let mut states = vec![client_world_entity_boat_tick_state(
        "initial",
        &world,
        boat_entity_id,
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
            "set_no_gravity" => {
                let value = operation["value"]
                    .as_bool()
                    .ok_or("no-gravity value must be a boolean")?;
                let metadata = if value {
                    &[5, 8, 1, 255]
                } else {
                    &[5, 8, 0, 255]
                };
                if !world.apply_entity_metadata(boat_entity_id, metadata) {
                    return Err("boat tick fixture no-gravity update was rejected".into());
                }
            }
            _ => return Err("unknown boat tick action".into()),
        }
        states.push(client_world_entity_boat_tick_state(
            string(operation, "label")?,
            &world,
            boat_entity_id,
        )?);
    }
    Ok(json!({"states": states}))
}

pub(super) fn sprint_trigger_input(value: &Value) -> Result<Input, String> {
    Ok(Input::new(
        value["forward"]
            .as_bool()
            .ok_or("forward must be a boolean")?,
        value["backward"]
            .as_bool()
            .ok_or("backward must be a boolean")?,
        value["left"].as_bool().ok_or("left must be a boolean")?,
        value["right"].as_bool().ok_or("right must be a boolean")?,
        value["jump"].as_bool().ok_or("jump must be a boolean")?,
        value["shift"].as_bool().ok_or("shift must be a boolean")?,
        value["sprint"]
            .as_bool()
            .ok_or("sprint must be a boolean")?,
    ))
}

pub(super) fn sprint_trigger_conditions(
    value: &Value,
) -> Result<LocalPlayerSprintTriggerConditions, String> {
    let boolean = |name: &str| {
        value[name]
            .as_bool()
            .ok_or_else(|| format!("{name} must be a boolean"))
    };
    Ok(LocalPlayerSprintTriggerConditions {
        mobility_restricted: boolean("mobility_restricted")?,
        enough_food: boolean("enough_food")?,
        may_fly: boolean("mayfly")?,
        shallow_water: boolean("shallow_water")?,
        flying: boolean("flying")?,
        slow_due_to_using_item: boolean("slow_due_to_using_item")?,
        fall_flying: boolean("fall_flying")?,
        underwater: boolean("underwater")?,
        moving_slowly: boolean("moving_slowly")?,
    })
}

pub(super) fn client_world_player_sprint_trigger_state(
    label: &str,
    player: &voxel_world::ClientPlayerState,
) -> Value {
    json!({
        "label": label,
        "sprinting": player.is_sprinting(),
        "sprint_trigger_time": player.sprint_trigger_time(),
        "key_presses": input_snapshot(player.input_key_presses()),
        "move_vector": vec2_snapshot(player.input_move_vector()),
    })
}
