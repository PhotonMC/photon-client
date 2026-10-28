use super::*;

pub(super) fn key_mapping_result(request: &Value) -> Result<Value, String> {
    let primary_key = input_key(&request["primary"])?;
    let primary_default_key = primary_key;
    let mut secondary_key = input_key(&request["secondary"])?;
    let mut other_key = input_key(&request["other"])?;
    let rebind_key = input_key(&request["rebind"])?;
    let mut current_primary_key = primary_key;
    let prefix = format!("behavior.key-mapping.{}", string(request, "case_id")?);
    let mut registry = KeyMappingRegistry::default();
    let primary = registry.register(format!("{prefix}.primary"), primary_key);
    let secondary = registry.register(format!("{prefix}.secondary"), secondary_key);
    let other = registry.register(format!("{prefix}.other"), other_key);
    let key_event = (
        i32_field(&request["key_event"], "key")?,
        i32_field(&request["key_event"], "scancode")?,
    );
    let unknown_key_event = (
        i32_field(&request["unknown_key_event"], "key")?,
        i32_field(&request["unknown_key_event"], "scancode")?,
    );
    let mouse_button = i32_field(request, "mouse_button")?;
    let initial = key_mapping_state(&registry, primary, secondary, other)?;
    let initial_properties = key_mapping_properties(
        &registry,
        [primary, secondary, other],
        key_event,
        unknown_key_event,
        mouse_button,
    )?;
    let mut events = Vec::new();
    for action in request["actions"]
        .as_array()
        .ok_or("actions must be an array")?
    {
        let kind = string(action, "op")?;
        let mut consumed = None;
        match kind {
            "set_down" => {
                let id = mapping_id(string(action, "target")?, primary, secondary, other)?;
                registry
                    .mapping_mut(id)
                    .ok_or("missing key mapping")?
                    .set_down(action["value"].as_bool().ok_or("value must be a boolean")?);
            }
            "set" => {
                let key = action_key(
                    string(action, "key")?,
                    current_primary_key,
                    primary_default_key,
                    secondary_key,
                    other_key,
                    rebind_key,
                )?;
                registry.set(
                    key,
                    action["value"].as_bool().ok_or("value must be a boolean")?,
                );
            }
            "click" => {
                let key = action_key(
                    string(action, "key")?,
                    current_primary_key,
                    primary_default_key,
                    secondary_key,
                    other_key,
                    rebind_key,
                )?;
                registry.click(key);
            }
            "consume_click" => {
                let id = mapping_id(string(action, "target")?, primary, secondary, other)?;
                consumed = Some(
                    registry
                        .mapping_mut(id)
                        .ok_or("missing key mapping")?
                        .consume_click(),
                );
            }
            "set_key" => {
                let target = string(action, "target")?;
                let replacement = action_key(
                    string(action, "key")?,
                    current_primary_key,
                    primary_default_key,
                    secondary_key,
                    other_key,
                    rebind_key,
                )?;
                let id = mapping_id(target, primary, secondary, other)?;
                registry
                    .mapping_mut(id)
                    .ok_or("missing key mapping")?
                    .set_key(replacement);
                match target {
                    "primary" => current_primary_key = replacement,
                    "secondary" => secondary_key = replacement,
                    "other" => other_key = replacement,
                    _ => return Err("unknown key mapping target".into()),
                }
            }
            "release_all" => registry.release_all(),
            _ => return Err("unknown key mapping action".into()),
        }
        let mut event = key_mapping_state(&registry, primary, secondary, other)?;
        if let Some(value) = consumed {
            event["consumed"] = json!(value);
        }
        events.push(event);
    }
    let properties = key_mapping_properties(
        &registry,
        [primary, secondary, other],
        key_event,
        unknown_key_event,
        mouse_button,
    )?;
    Ok(
        json!({"initial": initial, "initial_properties": initial_properties,
        "events": events, "properties": properties}),
    )
}

pub(super) fn toggle_key_mapping_result(request: &Value) -> Result<Value, String> {
    let primary_key = input_key(&request["primary"])?;
    let secondary_key = input_key(&request["secondary"])?;
    let other_key = input_key(&request["other"])?;
    let prefix = format!("behavior.key-mapping.{}", string(request, "case_id")?);
    let mut registry = KeyMappingRegistry::default();
    let primary = registry.register_toggle(
        format!("{prefix}.primary"),
        primary_key,
        request["primary_needs_toggle"]
            .as_bool()
            .ok_or("primary_needs_toggle must be a boolean")?,
        request["primary_should_restore"]
            .as_bool()
            .ok_or("primary_should_restore must be a boolean")?,
    );
    let secondary = registry.register_toggle(
        format!("{prefix}.secondary"),
        secondary_key,
        request["secondary_needs_toggle"]
            .as_bool()
            .ok_or("secondary_needs_toggle must be a boolean")?,
        request["secondary_should_restore"]
            .as_bool()
            .ok_or("secondary_should_restore must be a boolean")?,
    );
    let other = registry.register_toggle(
        format!("{prefix}.other"),
        other_key,
        request["other_needs_toggle"]
            .as_bool()
            .ok_or("other_needs_toggle must be a boolean")?,
        request["other_should_restore"]
            .as_bool()
            .ok_or("other_should_restore must be a boolean")?,
    );
    let key_event = (
        i32_field(&request["key_event"], "key")?,
        i32_field(&request["key_event"], "scancode")?,
    );
    let unknown_key_event = (
        i32_field(&request["unknown_key_event"], "key")?,
        i32_field(&request["unknown_key_event"], "scancode")?,
    );
    let mouse_button = i32_field(request, "mouse_button")?;
    let initial = key_mapping_state(&registry, primary, secondary, other)?;
    let initial_properties = key_mapping_properties(
        &registry,
        [primary, secondary, other],
        key_event,
        unknown_key_event,
        mouse_button,
    )?;
    let mut events = Vec::new();
    for action in request["actions"]
        .as_array()
        .ok_or("actions must be an array")?
    {
        let kind = string(action, "op")?;
        let mut consumed = None;
        let mut should_restore = None;
        match kind {
            "set_down" => {
                let id = mapping_id(string(action, "target")?, primary, secondary, other)?;
                registry
                    .mapping_mut(id)
                    .ok_or("missing key mapping")?
                    .set_down(action["value"].as_bool().ok_or("value must be a boolean")?);
            }
            "set" => {
                let key = action_key(
                    string(action, "key")?,
                    primary_key,
                    primary_key,
                    secondary_key,
                    other_key,
                    primary_key,
                )?;
                registry.set(
                    key,
                    action["value"].as_bool().ok_or("value must be a boolean")?,
                );
            }
            "click" => {
                let key = action_key(
                    string(action, "key")?,
                    primary_key,
                    primary_key,
                    secondary_key,
                    other_key,
                    primary_key,
                )?;
                registry.click(key);
            }
            "consume_click" => {
                let id = mapping_id(string(action, "target")?, primary, secondary, other)?;
                consumed = Some(
                    registry
                        .mapping_mut(id)
                        .ok_or("missing key mapping")?
                        .consume_click(),
                );
            }
            "release_all" => registry.release_all(),
            "restore_toggle_states" => registry.restore_toggle_states_on_screen_closed(),
            "reset_toggle_keys" => registry.reset_toggle_keys(),
            "should_restore" => {
                let id = mapping_id(string(action, "target")?, primary, secondary, other)?;
                should_restore = Some(
                    registry
                        .mapping_mut(id)
                        .ok_or("missing key mapping")?
                        .should_restore_state_on_screen_closed(),
                );
            }
            _ => return Err("unknown toggle key mapping action".into()),
        }
        let mut event = key_mapping_state(&registry, primary, secondary, other)?;
        if let Some(value) = consumed {
            event["consumed"] = json!(value);
        }
        if let Some(value) = should_restore {
            event["should_restore"] = json!(value);
        }
        events.push(event);
    }
    let properties = key_mapping_properties(
        &registry,
        [primary, secondary, other],
        key_event,
        unknown_key_event,
        mouse_button,
    )?;
    Ok(
        json!({"initial": initial, "initial_properties": initial_properties,
        "events": events, "properties": properties}),
    )
}

pub(super) fn vec3_snapshot(vector: Vec3) -> Value {
    json!({
        "x": format!("{:016x}", vector.x.to_bits()),
        "y": format!("{:016x}", vector.y.to_bits()),
        "z": format!("{:016x}", vector.z.to_bits()),
        "length": format!("{:016x}", vector.length().to_bits()),
        "length_squared": format!("{:016x}", vector.length_squared().to_bits()),
    })
}

pub(super) fn vec3f_snapshot(vector: Vec3f) -> Value {
    json!({
        "x": format!("{:08x}", vector.x.to_bits()),
        "y": format!("{:08x}", vector.y.to_bits()),
        "z": format!("{:08x}", vector.z.to_bits()),
    })
}

pub(super) fn quaternion_snapshot(quaternion: Quaternionf) -> Value {
    json!({
        "x": format!("{:08x}", quaternion.x.to_bits()),
        "y": format!("{:08x}", quaternion.y.to_bits()),
        "z": format!("{:08x}", quaternion.z.to_bits()),
        "w": format!("{:08x}", quaternion.w.to_bits()),
    })
}

pub(super) fn direction_snapshot(facing: Direction) -> Value {
    let (step_x, step_y, step_z) = facing.step();
    json!({
        "name": facing.name(),
        "data3d": facing.data3d(),
        "data2d": facing.data2d(),
        "opposite": facing.opposite().name(),
        "axis": facing.axis().name(),
        "axis_direction": facing.axis_direction().name(),
        "step_x": step_x,
        "step_y": step_y,
        "step_z": step_z,
    })
}

pub(super) fn blockpos(value: &Value) -> Result<BlockPos, String> {
    let coordinate = |name: &str| {
        string(value, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    Ok(BlockPos::new(
        coordinate("x")?,
        coordinate("y")?,
        coordinate("z")?,
    ))
}

pub(super) fn bounding_box(value: &Value) -> Result<BoundingBox, String> {
    let coordinate = |name: &str| {
        string(value, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    Ok(BoundingBox::new(
        coordinate("min_x")?,
        coordinate("min_y")?,
        coordinate("min_z")?,
        coordinate("max_x")?,
        coordinate("max_y")?,
        coordinate("max_z")?,
    ))
}

pub(super) fn blockpos_snapshot(value: BlockPos) -> Value {
    json!({
        "x": value.x.to_string(),
        "y": value.y.to_string(),
        "z": value.z.to_string(),
    })
}

pub(super) fn blockpos_packed_helpers_snapshot(
    value: BlockPos,
    neighbor_node: u64,
    direction: Direction,
    step_x: i32,
    step_y: i32,
    step_z: i32,
) -> Value {
    let packed = value.as_long();
    json!({
        "zero": blockpos_snapshot(BlockPos::ZERO),
        "packed_horizontal_length": BlockPos::PACKED_HORIZONTAL_LENGTH,
        "packed_y_length": BlockPos::PACKED_Y_LENGTH,
        "max_horizontal_coordinate": BlockPos::MAX_HORIZONTAL_COORDINATE,
        "packed": format!("{packed:x}"),
        "coordinate_packed": format!("{:x}", BlockPos::as_long_coordinates(value.x, value.y, value.z)),
        "decoded": blockpos_snapshot(BlockPos::of(packed)),
        "accessors": {
            "x": BlockPos::get_x(packed).to_string(),
            "y": BlockPos::get_y(packed).to_string(),
            "z": BlockPos::get_z(packed).to_string(),
        },
        "flat_index": format!("{:x}", BlockPos::get_flat_index(neighbor_node)),
        "offset_direction": format!("{:x}", BlockPos::offset_packed(neighbor_node, direction)),
        "offset_steps": format!("{:x}", BlockPos::offset_packed_by(
            neighbor_node, step_x, step_y, step_z,
        )),
    })
}

pub(super) fn blockpos_relative_helpers_snapshot(
    value: BlockPos,
    direction: Direction,
    axis: Axis,
    steps: i32,
) -> Value {
    json!({
        "relative_direction": blockpos_snapshot(value.relative_direction(direction, 1)),
        "relative_direction_steps": blockpos_snapshot(value.relative_direction(direction, steps)),
        "relative_axis_steps": blockpos_snapshot(value.relative_axis(axis, steps)),
        "cardinal": {
            "above": blockpos_snapshot(value.above()),
            "above_steps": blockpos_snapshot(value.above_steps(steps)),
            "below": blockpos_snapshot(value.below()),
            "below_steps": blockpos_snapshot(value.below_steps(steps)),
            "north": blockpos_snapshot(value.north()),
            "north_steps": blockpos_snapshot(value.north_steps(steps)),
            "south": blockpos_snapshot(value.south()),
            "south_steps": blockpos_snapshot(value.south_steps(steps)),
            "west": blockpos_snapshot(value.west()),
            "west_steps": blockpos_snapshot(value.west_steps(steps)),
            "east": blockpos_snapshot(value.east()),
            "east_steps": blockpos_snapshot(value.east_steps(steps)),
        },
    })
}

pub(super) fn blockpos_vector_helpers_snapshot(
    value: BlockPos,
    delta: BlockPos,
    scale: i32,
) -> Value {
    json!({
        "offset_vec3i": blockpos_snapshot(value.offset_by_block_pos(delta)),
        "subtract_vec3i": blockpos_snapshot(value.subtract_block_pos(delta)),
        "multiply": blockpos_snapshot(value.multiply(scale)),
    })
}

pub(super) fn blockpos_geometry_helpers_snapshot(
    value: BlockPos,
    rotation: BlockRotation,
    up_vector: BlockPos,
    y: i32,
) -> Value {
    json!({
        "rotate": blockpos_snapshot(value.rotate(rotation)),
        "cross": blockpos_snapshot(value.cross(up_vector)),
        "at_y": blockpos_snapshot(value.at_y(y)),
    })
}

pub(super) fn blockpos_distance_helpers_snapshot(
    value: BlockPos,
    other: BlockPos,
    point: Vec3,
    distance: f64,
) -> Value {
    json!({
        "dist_sqr": format!("{:016x}", value.dist_sqr(other).to_bits()),
        "dist_low_corner_sqr": format!(
            "{:016x}",
            value.dist_to_low_corner_sqr(point.x, point.y, point.z).to_bits(),
        ),
        "dist_center_sqr": format!(
            "{:016x}",
            value.dist_to_center_sqr(point.x, point.y, point.z).to_bits(),
        ),
        "closer_than": value.closer_than(other, distance),
    })
}
