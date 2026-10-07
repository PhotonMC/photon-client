use super::*;

pub(super) fn vec2_case(request: &Value) -> Result<Value, String> {
    let mut vector = Vec2::new(f32_field(request, "x")?, f32_field(request, "y")?);
    let mut events = vec![vec2_snapshot(vector)];
    let steps = request["steps"]
        .as_array()
        .ok_or("steps must be an array")?;
    for step in steps {
        vector = match string(step, "action")? {
            "normalize" => vector.normalized(),
            "scale" => vector.scale(f32_field(step, "factor")?),
            _ => return Err("unknown Vec2 action".into()),
        };
        events.push(vec2_snapshot(vector));
    }
    Ok(json!({"events": events}))
}

pub(super) fn keyboard_input_case(request: &Value) -> Result<Value, String> {
    let keys = Input::new(
        request["up"].as_bool().ok_or("up must be a boolean")?,
        request["down"].as_bool().ok_or("down must be a boolean")?,
        request["left"].as_bool().ok_or("left must be a boolean")?,
        request["right"]
            .as_bool()
            .ok_or("right must be a boolean")?,
        request["jump"].as_bool().ok_or("jump must be a boolean")?,
        request["shift"]
            .as_bool()
            .ok_or("shift must be a boolean")?,
        request["sprint"]
            .as_bool()
            .ok_or("sprint must be a boolean")?,
    );
    let mut keyboard = KeyboardInput::new(keys);
    keyboard.tick();
    Ok(json!({
        "key_presses": input_snapshot(keyboard.key_presses),
        "move_vector": vec2_snapshot(keyboard.move_vector()),
    }))
}

pub(super) fn direction_case(request: &Value) -> Result<Value, String> {
    let mut facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
    let mut events = vec![direction_snapshot(facing)];
    let steps = request["steps"]
        .as_array()
        .ok_or("steps must be an array")?;
    for step in steps {
        let axis = Axis::from_name(string(step, "axis")?).ok_or("unknown axis")?;
        let clockwise = step["clockwise"]
            .as_bool()
            .ok_or("clockwise must be a boolean")?;
        facing = if clockwise {
            facing.clockwise(axis)
        } else {
            facing.counter_clockwise(axis)
        };
        events.push(direction_snapshot(facing));
    }
    Ok(json!({"events": events}))
}

pub(super) fn coord_case(request: &Value) -> Result<Value, String> {
    let value = string(request, "value")?
        .parse::<i32>()
        .map_err(|error| error.to_string())?;
    let out = match string(request, "fn")? {
        "block_to_section" => math::block_to_section_coord(value),
        "section_to_block" => math::section_to_block_coord(value),
        "section_relative" => math::section_relative(value),
        _ => return Err("unknown coord function".into()),
    };
    Ok(json!({"value": out.to_string()}))
}

pub(super) fn vec3_case(request: &Value) -> Result<Value, String> {
    let mut vector = Vec3::new(
        f64_field(request, "x")?,
        f64_field(request, "y")?,
        f64_field(request, "z")?,
    );
    let mut events = vec![vec3_snapshot(vector)];
    let steps = request["steps"]
        .as_array()
        .ok_or("steps must be an array")?;
    for step in steps {
        vector = match string(step, "action")? {
            "normalize" => vector.normalized(),
            "scale" => vector.scale(f64_field(step, "factor")?),
            "reverse" => vector.reverse(),
            "multiply_vec3" => vector.multiply_vec3(f64_vec3(
                step.get("scale").ok_or("scale must be an object")?,
            )?),
            "multiply_components" => vector.multiply_components(
                f64_field(step, "x")?,
                f64_field(step, "y")?,
                f64_field(step, "z")?,
            ),
            "horizontal" => vector.horizontal(),
            "add_vec3" => vector.add_vec3(f64_vec3(
                step.get("delta").ok_or("delta must be an object")?,
            )?),
            "add_scalar" => vector.add_scalar(f64_field(step, "value")?),
            "add_components" => vector.add_components(
                f64_field(step, "x")?,
                f64_field(step, "y")?,
                f64_field(step, "z")?,
            ),
            "subtract_vec3" => vector.subtract_vec3(f64_vec3(
                step.get("delta").ok_or("delta must be an object")?,
            )?),
            "subtract_scalar" => vector.subtract_scalar(f64_field(step, "value")?),
            "subtract_components" => vector.subtract_components(
                f64_field(step, "x")?,
                f64_field(step, "y")?,
                f64_field(step, "z")?,
            ),
            "vector_to" => vector.vector_to(f64_vec3(
                step.get("target").ok_or("target must be an object")?,
            )?),
            "cross" => vector.cross(f64_vec3(
                step.get("other").ok_or("other must be an object")?,
            )?),
            _ => return Err("unknown Vec3 action".into()),
        };
        events.push(vec3_snapshot(vector));
    }
    Ok(json!({"events": events}))
}
