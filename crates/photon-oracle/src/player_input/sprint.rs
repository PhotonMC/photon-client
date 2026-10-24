use super::*;

pub(crate) fn sprint_trigger_input(value: &Value) -> Result<Input, String> {
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

pub(crate) fn sprint_trigger_conditions(
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

pub(crate) fn client_world_player_sprint_trigger_state(
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

pub(crate) fn client_world_player_sprint_attribute_state(
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

pub(crate) fn client_world_player_sprint_attribute_result(
    request: &Value,
) -> Result<Value, String> {
    let player_id = entity_id_field(request, "player_id")?;
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
        let operation_player_id = entity_id_field(operation, "player_id")?;
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

pub(crate) fn client_world_player_sneaking_input_state(
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

pub(crate) fn client_world_player_sneaking_input_result(request: &Value) -> Result<Value, String> {
    let player_id = entity_id_field(request, "player_id")?;
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
        let operation_player_id = entity_id_field(operation, "player_id")?;
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
