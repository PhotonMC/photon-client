use super::*;

pub(super) fn client_attribute_state_json(
    label: &str,
    attribute: &ClientAttributeState,
    min_value: f64,
    max_value: f64,
) -> Value {
    let mut modifiers = attribute.modifiers().iter().collect::<Vec<_>>();
    modifiers.sort_by(|left, right| left.id().cmp(right.id()));
    json!({
        "label": label,
        "attribute_id": attribute.attribute_id(),
        "found": true,
        "base_value": format!("{:016x}", attribute.base_value().to_bits()),
        "modifiers": modifiers
            .into_iter()
            .map(|modifier| json!({
                "id": modifier.id(),
                "amount": format!("{:016x}", modifier.amount().to_bits()),
                "operation": modifier.operation(),
            }))
            .collect::<Vec<_>>(),
        "effective_value": format!(
            "{:016x}",
            attribute.effective_value(min_value, max_value).to_bits()
        ),
    })
}

pub(super) fn client_world_player_attribute_queries(
    world: &ClientWorld,
    value: &Value,
) -> Result<Value, String> {
    let player = world
        .local_player()
        .ok_or("local player is not initialized")?;
    value
        .as_array()
        .ok_or("queries must be an array")?
        .iter()
        .map(|query| {
            let label = string(query, "label")?;
            let attribute_id = i32_field(query, "attribute_id")?;
            let min_value = f64_field(query, "min")?;
            let max_value = f64_field(query, "max")?;
            let Some(attribute) = player
                .attributes()
                .iter()
                .find(|attribute| attribute.attribute_id() == attribute_id)
            else {
                return Ok(json!({
                    "label": label,
                    "attribute_id": attribute_id,
                    "found": false,
                }));
            };
            Ok(client_attribute_state_json(
                label, attribute, min_value, max_value,
            ))
        })
        .collect::<Result<Vec<_>, String>>()
        .map(Value::Array)
}

pub(super) fn client_world_player_attributes_result(request: &Value) -> Result<Value, String> {
    let player_id = entity_id_field(request, "player_id")?;
    let mut world = crate::case_ids::world(DimensionBounds::new(0, 256));
    world.set_local_player_id(player_id);
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut events = Vec::with_capacity(operations.len());

    for operation in operations {
        if string(operation, "action")? != "apply" {
            return Err("unknown player attribute action".into());
        }
        let attributes = operation["attributes"]
            .as_array()
            .ok_or("attributes must be an array")?
            .iter()
            .map(client_attribute_state)
            .collect::<Result<Vec<_>, _>>()?;
        let accepted = world
            .apply_local_player_attributes(entity_id_field(operation, "player_id")?, &attributes);
        events.push(json!({
            "action": string(operation, "action")?,
            "label": string(operation, "label")?,
            "accepted": accepted,
            "queries": client_world_player_attribute_queries(
                &world,
                &operation["queries"],
            )?,
        }));
    }
    Ok(json!({"events": events}))
}

pub(super) fn client_world_player_movement_speed_result(request: &Value) -> Result<Value, String> {
    let player_id = entity_id_field(request, "player_id")?;
    let mut world = crate::case_ids::world(DimensionBounds::new(0, 256));
    world.set_local_player_id(player_id);
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());

    for operation in operations {
        let attributes = operation["attributes"]
            .as_array()
            .ok_or("attributes must be an array")?
            .iter()
            .map(client_attribute_state)
            .collect::<Result<Vec<_>, _>>()?;
        let accepted = world
            .apply_local_player_attributes(entity_id_field(operation, "player_id")?, &attributes);
        if !world.apply_local_player_friction_influenced_speed_from_attributes(
            operation["on_ground"]
                .as_bool()
                .ok_or("on_ground must be a boolean")?,
            f32_field(operation, "block_friction")?,
        ) {
            return Err("local player is not initialized".into());
        }
        let player = world
            .local_player()
            .ok_or("local player is not initialized")?;
        states.push(json!({
            "label": string(operation, "label")?,
            "accepted": accepted,
            "movement_speed": format!("{:08x}", player.movement_speed().to_bits()),
            "friction_influenced_speed": format!(
                "{:08x}",
                player.friction_influenced_speed().to_bits()
            ),
        }));
    }
    Ok(json!({"states": states}))
}

pub(super) fn client_world_player_attribute_water_result(request: &Value) -> Result<Value, String> {
    let player_id = entity_id_field(request, "player_id")?;
    let mut world = crate::case_ids::world(DimensionBounds::new(0, 256));
    world.set_local_player_id(player_id);
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());

    for operation in operations {
        let attributes = operation["attributes"]
            .as_array()
            .ok_or("attributes must be an array")?
            .iter()
            .map(client_attribute_state)
            .collect::<Result<Vec<_>, _>>()?;
        let accepted = world
            .apply_local_player_attributes(entity_id_field(operation, "player_id")?, &attributes);
        let delta = f64_vec3(operation.get("delta").ok_or("delta must be a vector")?)?;
        let y_rot = f32_field(operation, "y_rot")?;
        if !world.apply_local_player_position(
            (0.0, f64_field(operation, "current_y")?, 0.0),
            (delta.x, delta.y, delta.z),
            (y_rot, 0.0),
        ) {
            return Err("local player is not initialized".into());
        }
        let parameters = LocalPlayerTravelInWaterParameters {
            input: f64_vec3(operation.get("input").ok_or("input must be a vector")?)?,
            y_rot,
            sprinting: operation["sprinting"]
                .as_bool()
                .ok_or("sprinting must be a boolean")?,
            water_slow_down: f32_field(operation, "water_slow_down")?,
            water_efficiency: f32_field(operation, "water_efficiency")?,
            on_ground: operation["on_ground"]
                .as_bool()
                .ok_or("on_ground must be a boolean")?,
            speed: 0.0,
            dolphins_grace: operation["dolphins_grace"]
                .as_bool()
                .ok_or("dolphins_grace must be a boolean")?,
            horizontal_collision: operation["horizontal_collision"]
                .as_bool()
                .ok_or("horizontal_collision must be a boolean")?,
            on_climbable: operation["on_climbable"]
                .as_bool()
                .ok_or("on_climbable must be a boolean")?,
            free_after_jump: operation["free_after_jump"]
                .as_bool()
                .ok_or("free_after_jump must be a boolean")?,
            base_gravity: f64_field(operation, "base_gravity")?,
            is_falling: operation["is_falling"]
                .as_bool()
                .ok_or("is_falling must be a boolean")?,
        };
        if !world.apply_local_player_travel_in_water_from_attributes(parameters) {
            return Err("local player is not initialized".into());
        }
        let player = world
            .local_player()
            .ok_or("local player is not initialized")?;
        let motion = player.motion();
        states.push(json!({
            "label": string(operation, "label")?,
            "accepted": accepted,
            "movement_speed": format!("{:08x}", player.movement_speed().to_bits()),
            "movement": vec3_snapshot(Vec3::new(motion.0, motion.1, motion.2)),
        }));
    }
    Ok(json!({"states": states}))
}
