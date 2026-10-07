use super::*;

pub(super) fn client_world_player_keyboard_input_result(request: &Value) -> Result<Value, String> {
    let player_id = i32_field(request, "player_id")?;
    let mut world = ClientWorld::new(DimensionBounds::new(0, 256));
    world.set_local_player_id(player_id);
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());

    for operation in operations {
        if string(operation, "action")? != "apply_keyboard_input" {
            return Err("unknown player keyboard input action".into());
        }
        let attributes = operation["attributes"]
            .as_array()
            .ok_or("attributes must be an array")?
            .iter()
            .map(client_attribute_state)
            .collect::<Result<Vec<_>, _>>()?;
        let operation_player_id = i32_field(operation, "player_id")?;
        let accepted = world.apply_local_player_attributes(operation_player_id, &attributes);
        let keys = Input::new(
            operation["up"].as_bool().ok_or("up must be a boolean")?,
            operation["down"]
                .as_bool()
                .ok_or("down must be a boolean")?,
            operation["left"]
                .as_bool()
                .ok_or("left must be a boolean")?,
            operation["right"]
                .as_bool()
                .ok_or("right must be a boolean")?,
            operation["jump"]
                .as_bool()
                .ok_or("jump must be a boolean")?,
            operation["shift"]
                .as_bool()
                .ok_or("shift must be a boolean")?,
            operation["sprint"]
                .as_bool()
                .ok_or("sprint must be a boolean")?,
        );
        let mut keyboard = KeyboardInput::new(keys);
        keyboard.tick();
        let modifiers = LocalPlayerInputModifiers {
            using_item: operation["using_item"]
                .as_bool()
                .ok_or("using_item must be a boolean")?,
            passenger: operation["passenger"]
                .as_bool()
                .ok_or("passenger must be a boolean")?,
            item_use_speed_multiplier: f32_field(operation, "item_use_speed_multiplier")?,
            moving_slowly: operation["moving_slowly"]
                .as_bool()
                .ok_or("moving_slowly must be a boolean")?,
            sneaking_movement_factor: 0.3,
        };
        if !world.apply_local_player_keyboard_input(
            keys,
            operation["controlled_camera"]
                .as_bool()
                .ok_or("controlled_camera must be a boolean")?,
            modifiers,
        ) {
            return Err("local player is not initialized".into());
        }
        let player = world
            .local_player()
            .ok_or("local player is not initialized")?;
        states.push(client_world_player_keyboard_input_state(
            string(operation, "label")?,
            accepted,
            &keyboard,
            player,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_player_tick_state(
    label: &str,
    player: &voxel_world::ClientPlayerState,
) -> Value {
    let position = player.position();
    let previous_position = player.previous_position();
    let rotation = player.rotation();
    let previous_rotation = player.previous_rotation();
    let effects = player
        .effects()
        .iter()
        .map(|effect| {
            json!({
                "effect_id": effect.effect_id(),
                "amplifier": effect.amplifier(),
                "duration_ticks": effect.duration_ticks(),
                "flags": effect.flags(),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "label": label,
        "tick_count": player.tick_count(),
        "position": vec3_snapshot(Vec3::new(position.0, position.1, position.2)),
        "previous_position": vec3_snapshot(Vec3::new(
            previous_position.0,
            previous_position.1,
            previous_position.2,
        )),
        "rotation_y": format!("{:08x}", rotation.0.to_bits()),
        "rotation_x": format!("{:08x}", rotation.1.to_bits()),
        "previous_rotation_y": format!("{:08x}", previous_rotation.0.to_bits()),
        "previous_rotation_x": format!("{:08x}", previous_rotation.1.to_bits()),
        "head_rotation": format!("{:08x}", player.head_rotation().to_bits()),
        "previous_head_rotation": format!("{:08x}", player.previous_head_rotation().to_bits()),
        "effects": effects,
    })
}

pub(super) fn client_world_player_tick_result(request: &Value) -> Result<Value, String> {
    let player_id = i32_field(request, "player_id")?;
    let initial_position = f64_vec3(
        request
            .get("initial_position")
            .ok_or("initial_position must be a vector")?,
    )?;
    let initial_rotation = request
        .get("initial_rotation")
        .ok_or("initial_rotation must be an object")?;
    let initial_y_rotation = f32_field(initial_rotation, "y")?;
    let initial_x_rotation = f32_field(initial_rotation, "x")?;
    let initial_head = f32_field(request, "initial_head")?;

    let mut world = ClientWorld::new(DimensionBounds::new(0, 256));
    world.set_local_player_id(player_id);
    if !world.apply_entity(EntityInput {
        entity_id: player_id,
        uuid: Uuid::new(0, u64::from(player_id as u32)),
        type_id: 156,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, initial_head),
        data: 0,
    }) {
        return Err("unable to initialize local player entity state".into());
    }
    if !world.apply_local_player_position(
        (initial_position.x, initial_position.y, initial_position.z),
        (0.0, 0.0, 0.0),
        (initial_y_rotation, initial_x_rotation),
    ) {
        return Err("local player is not initialized".into());
    }
    for entry in request
        .get("initial_effects")
        .ok_or("initial_effects must be an array")?
        .as_array()
        .ok_or("initial_effects must be an array")?
    {
        let effect = entry
            .as_object()
            .ok_or("initial effect must be an object")?;
        if !world.apply_local_player_mob_effect(
            player_id,
            i32_field(entry, "effect_id")?,
            i32_field(entry, "amplifier")?,
            i32_field(entry, "duration_ticks")?,
            effect
                .get("flags")
                .and_then(Value::as_u64)
                .ok_or("flags must be an unsigned integer")?
                .try_into()
                .map_err(|_| "flags must fit in a byte")?,
        ) {
            return Err("local player effect could not be applied".into());
        }
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());
    for operation in operations {
        if string(operation, "action")? != "tick" {
            return Err("unknown player tick action".into());
        }
        let position = f64_vec3(
            operation
                .get("position")
                .ok_or("position must be a vector")?,
        )?;
        let rotation = operation
            .get("rotation")
            .ok_or("rotation must be an object")?;
        if !world.apply_local_player_position(
            (position.x, position.y, position.z),
            (0.0, 0.0, 0.0),
            (f32_field(rotation, "y")?, f32_field(rotation, "x")?),
        ) {
            return Err("local player is not initialized".into());
        }
        if let Some(head_target) = operation
            .get("head_target")
            .filter(|value| !value.is_null())
            && !world.apply_local_player_head_rotation(f32_value(head_target, "head_target")?)
        {
            return Err("local player head target could not be applied".into());
        }
        for entry in operation
            .get("effects")
            .ok_or("effects must be an array")?
            .as_array()
            .ok_or("effects must be an array")?
        {
            let effect = entry.as_object().ok_or("effect must be an object")?;
            if !world.apply_local_player_mob_effect(
                player_id,
                i32_field(entry, "effect_id")?,
                i32_field(entry, "amplifier")?,
                i32_field(entry, "duration_ticks")?,
                effect
                    .get("flags")
                    .and_then(Value::as_u64)
                    .ok_or("flags must be an unsigned integer")?
                    .try_into()
                    .map_err(|_| "flags must fit in a byte")?,
            ) {
                return Err("local player effect could not be applied".into());
            }
        }
        let ticks = operation["ticks"]
            .as_u64()
            .ok_or("ticks must be an unsigned integer")?;
        for _ in 0..ticks {
            world.tick();
        }
        let player = world
            .local_player()
            .ok_or("local player is not initialized")?;
        states.push(client_world_player_tick_state(
            string(operation, "label")?,
            player,
        ));
    }
    Ok(json!({"player_id": player_id, "states": states}))
}

pub(super) fn client_world_entity_player_head_rotation_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    let rotation = entity.rotation();
    let previous_rotation = entity.previous_rotation();
    json!({
        "label": label,
        "accepted": accepted,
        "head_rotation": format!("{:08x}", rotation.2.to_bits()),
        "previous_head_rotation": format!("{:08x}", previous_rotation.2.to_bits()),
        "head_target": format!("{:08x}", entity.head_rotation_target().to_bits()),
        "head_lerp_steps": entity.head_rotation_lerp_steps(),
        "body_rotation_y": format!("{:08x}", entity.body_y_rotation().to_bits()),
    })
}

pub(super) fn client_world_entity_player_head_rotation_result(
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
    let initial_head = f32_field(request, "initial_head")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: initial_position.x,
        y: initial_position.y,
        z: initial_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (
            f32_field(initial_rotation, "y")?,
            f32_field(initial_rotation, "x")?,
            initial_head,
        ),
        data: 0,
    }) {
        return Err("remote player head fixture entity was rejected".into());
    }

    let entity = world
        .entity_at(entity_id)
        .ok_or("remote player head fixture entity disappeared")?;
    let mut states = vec![client_world_entity_player_head_rotation_state(
        "initial", true, entity,
    )];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        if string(operation, "action")? != "rotate_head" {
            return Err("unknown remote player head action".into());
        }
        if let Some(head_target) = operation
            .get("head_target")
            .filter(|value| !value.is_null())
            && !world.apply_entity_head_rotation(entity_id, f32_value(head_target, "head_target")?)
        {
            return Err("remote player head target could not be applied".into());
        }
        let ticks = operation["ticks"]
            .as_u64()
            .ok_or("ticks must be an unsigned integer")?;
        for _ in 0..ticks {
            world.tick();
        }
        let entity = world
            .entity_at(entity_id)
            .ok_or("remote player head fixture entity disappeared")?;
        states.push(client_world_entity_player_head_rotation_state(
            string(operation, "label")?,
            true,
            entity,
        ));
    }
    Ok(json!({"entity_id": entity_id, "states": states}))
}

pub(super) fn client_world_entity_tick_vector(vector: (f64, f64, f64)) -> Value {
    json!({
        "x": format!("{:016x}", vector.0.to_bits()),
        "y": format!("{:016x}", vector.1.to_bits()),
        "z": format!("{:016x}", vector.2.to_bits()),
    })
}

pub(super) fn client_world_entity_tick_state(
    label: &str,
    accepted: bool,
    entity_ids: &[i32],
    world: &ClientWorld,
) -> Result<Value, String> {
    let entities = entity_ids
        .iter()
        .map(|entity_id| {
            let entity = world
                .entity_at(*entity_id)
                .ok_or_else(|| "entity tick fixture entity disappeared".to_owned())?;
            let rotation = entity.rotation();
            let previous_rotation = entity.previous_rotation();
            Ok(json!({
                "entity_id": entity.id(),
                "tick_count": entity.tick_count(),
                "position": client_world_entity_tick_vector(entity.position()),
                "previous_position": client_world_entity_tick_vector(
                    entity.previous_position()
                ),
                "motion": client_world_entity_tick_vector(entity.motion()),
                "rotation_y": format!("{:08x}", rotation.0.to_bits()),
                "rotation_x": format!("{:08x}", rotation.1.to_bits()),
                "previous_rotation_y": format!("{:08x}", previous_rotation.0.to_bits()),
                "previous_rotation_x": format!("{:08x}", previous_rotation.1.to_bits()),
                "no_gravity": entity.is_no_gravity(),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "label": label,
        "accepted": accepted,
        "entities": entities,
    }))
}
