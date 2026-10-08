use super::*;

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
    player: &photon_world::ClientPlayerState,
) -> Value {
    json!({
        "label": label,
        "sprinting": player.is_sprinting(),
        "sprint_trigger_time": player.sprint_trigger_time(),
        "key_presses": input_snapshot(player.input_key_presses()),
        "move_vector": vec2_snapshot(player.input_move_vector()),
    })
}

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

pub(super) fn client_world_player_sprint_attribute_state(
    label: &str,
    accepted: bool,
    player: &photon_world::ClientPlayerState,
) -> Value {
    let attribute = player
        .attributes()
        .iter()
        .find(|attribute| attribute.attribute_id() == 26);
    let base_value = attribute.map_or(0.1, |attribute| attribute.base_value());
    let mut modifiers = attribute.map_or_else(Vec::new, |attribute| {
        attribute.modifiers().iter().collect::<Vec<_>>()
    });
    modifiers.sort_by(|left, right| left.id().cmp(right.id()));
    json!({
        "label": label,
        "accepted": accepted,
        "sprinting": player.is_sprinting(),
        "base_value": format!("{:016x}", base_value.to_bits()),
        "modifiers": modifiers
            .into_iter()
            .map(|modifier| json!({
                "id": modifier.id(),
                "amount": format!("{:016x}", modifier.amount().to_bits()),
                "operation": modifier.operation(),
            }))
            .collect::<Vec<_>>(),
        "movement_speed": format!("{:08x}", player.movement_speed().to_bits()),
    })
}

pub(super) fn client_world_player_sprint_attribute_result(
    request: &Value,
) -> Result<Value, String> {
    let player_id = i32_field(request, "player_id")?;
    let mut world = ClientWorld::new(DimensionBounds::new(0, 256));
    world.set_local_player_id(player_id);
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());

    for operation in operations {
        if string(operation, "action")? != "set_sprinting" {
            return Err("unknown player sprint action".into());
        }
        let attributes = operation["attributes"]
            .as_array()
            .ok_or("attributes must be an array")?
            .iter()
            .map(client_attribute_state)
            .collect::<Result<Vec<_>, _>>()?;
        let operation_player_id = i32_field(operation, "player_id")?;
        let accepted = world.apply_local_player_attributes(operation_player_id, &attributes);
        if accepted
            && !world.apply_local_player_sprinting(
                operation_player_id,
                operation["sprinting"]
                    .as_bool()
                    .ok_or("sprinting must be a boolean")?,
            )
        {
            return Err("local player is not initialized".into());
        }
        let player = world
            .local_player()
            .ok_or("local player is not initialized")?;
        states.push(client_world_player_sprint_attribute_state(
            string(operation, "label")?,
            accepted,
            player,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_player_sneaking_input_state(
    label: &str,
    accepted: bool,
    player: &photon_world::ClientPlayerState,
) -> Value {
    let bob = player.bob();
    let previous_bob = player.previous_bob();
    json!({
        "label": label,
        "accepted": accepted,
        "sneaking_speed": format!("{:016x}", player.sneaking_movement_factor_value().to_bits()),
        "movement": vec2_snapshot(player.input_impulses()),
        "jumping": player.is_jumping(),
        "x_bob": format!("{:08x}", bob.0.to_bits()),
        "y_bob": format!("{:08x}", bob.1.to_bits()),
        "x_bob_o": format!("{:08x}", previous_bob.0.to_bits()),
        "y_bob_o": format!("{:08x}", previous_bob.1.to_bits()),
    })
}

pub(super) fn client_world_player_sneaking_input_result(request: &Value) -> Result<Value, String> {
    let player_id = i32_field(request, "player_id")?;
    let mut world = ClientWorld::new(DimensionBounds::new(0, 256));
    world.set_local_player_id(player_id);
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());

    for operation in operations {
        if string(operation, "action")? != "apply_input" {
            return Err("unknown player sneaking input action".into());
        }
        let attributes = operation["attributes"]
            .as_array()
            .ok_or("attributes must be an array")?
            .iter()
            .map(client_attribute_state)
            .collect::<Result<Vec<_>, _>>()?;
        let operation_player_id = i32_field(operation, "player_id")?;
        let accepted = world.apply_local_player_attributes(operation_player_id, &attributes);
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
        if !world.apply_local_player_input_from_attributes(
            Vec2::new(
                f32_field(operation, "move_x")?,
                f32_field(operation, "move_y")?,
            ),
            operation["jump"]
                .as_bool()
                .ok_or("jump must be a boolean")?,
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
        states.push(client_world_player_sneaking_input_state(
            string(operation, "label")?,
            accepted,
            player,
        ));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_player_keyboard_input_state(
    label: &str,
    accepted: bool,
    keyboard: &KeyboardInput,
    player: &photon_world::ClientPlayerState,
) -> Value {
    let bob = player.bob();
    let previous_bob = player.previous_bob();
    json!({
        "label": label,
        "accepted": accepted,
        "key_presses": input_snapshot(keyboard.key_presses),
        "move_vector": vec2_snapshot(keyboard.move_vector()),
        "movement": vec2_snapshot(player.input_impulses()),
        "jumping": player.is_jumping(),
        "x_bob": format!("{:08x}", bob.0.to_bits()),
        "y_bob": format!("{:08x}", bob.1.to_bits()),
        "x_bob_o": format!("{:08x}", previous_bob.0.to_bits()),
        "y_bob_o": format!("{:08x}", previous_bob.1.to_bits()),
    })
}

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
    player: &photon_world::ClientPlayerState,
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
