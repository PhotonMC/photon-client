use super::*;

pub(super) fn axis_cycle_helpers_case(request: &Value) -> Result<Value, String> {
    let x = i32_string_field(request, "x")?;
    let y = i32_string_field(request, "y")?;
    let z = i32_string_field(request, "z")?;
    let double_x = f64_field(request, "double_x")?;
    let double_y = f64_field(request, "double_y")?;
    let double_z = f64_field(request, "double_z")?;
    let mut cycles = serde_json::Map::new();
    for cycle in AxisCycle::VALUES {
        cycles.insert(
            cycle.name().into(),
            json!({
                "int_x": cycle.cycle_i32(x, y, z, Axis::X),
                "int_y": cycle.cycle_i32(x, y, z, Axis::Y),
                "int_z": cycle.cycle_i32(x, y, z, Axis::Z),
                "double_x": format!("{:016x}", cycle.cycle_f64(double_x, double_y, double_z, Axis::X).to_bits()),
                "double_y": format!("{:016x}", cycle.cycle_f64(double_x, double_y, double_z, Axis::Y).to_bits()),
                "double_z": format!("{:016x}", cycle.cycle_f64(double_x, double_y, double_z, Axis::Z).to_bits()),
                "axis_x": cycle.cycle_axis(Axis::X).name(),
                "axis_y": cycle.cycle_axis(Axis::Y).name(),
                "axis_z": cycle.cycle_axis(Axis::Z).name(),
                "inverse": cycle.inverse().name(),
            }),
        );
    }
    let mut between = Vec::new();
    for from in [Axis::X, Axis::Y, Axis::Z] {
        for to in [Axis::X, Axis::Y, Axis::Z] {
            between.push(json!({
                "from": from.name(),
                "to": to.name(),
                "cycle": AxisCycle::between(from, to).name(),
            }));
        }
    }
    let mut mutable_set = serde_json::Map::new();
    for cycle in AxisCycle::VALUES {
        let mut value = MutableBlockPos::new(0, 0, 0);
        value.set_axis_cycle(cycle, x, y, z);
        mutable_set.insert(cycle.name().into(), blockpos_snapshot(value.immutable()));
    }
    Ok(json!({
        "cycles": Value::Object(cycles),
        "between": between,
        "mutable_set": Value::Object(mutable_set),
    }))
}

pub(super) fn direction_axial_rotation_case(request: &Value) -> Result<Value, String> {
    let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
    let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
    Ok(json!({
        "clockwise": direction_snapshot(facing.clockwise(axis)),
        "counter_clockwise": direction_snapshot(facing.counter_clockwise(axis)),
    }))
}

pub(super) fn direction_y_rotation_case(request: &Value) -> Result<Value, String> {
    let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
    let clockwise = match facing.clockwise_y() {
        Ok(direction) => json!({"outcome": "value", "value": direction_snapshot(direction)}),
        Err(DirectionYRotationError::ClockwiseVertical(direction)) => json!({
            "outcome": "error",
            "error": format!("Unable to get Y-rotated facing of {}", direction.name()),
        }),
        Err(DirectionYRotationError::CounterClockwiseVertical(_)) => {
            unreachable!("clockwise_y cannot return a counter-clockwise error")
        }
    };
    let counter_clockwise = match facing.counter_clockwise_y() {
        Ok(direction) => json!({"outcome": "value", "value": direction_snapshot(direction)}),
        Err(DirectionYRotationError::CounterClockwiseVertical(direction)) => json!({
            "outcome": "error",
            "error": format!("Unable to get CCW facing of {}", direction.name()),
        }),
        Err(DirectionYRotationError::ClockwiseVertical(_)) => {
            unreachable!("counter_clockwise_y cannot return a clockwise error")
        }
    };
    Ok(json!({"clockwise": clockwise, "counter_clockwise": counter_clockwise}))
}

pub(super) fn direction_data_case(request: &Value) -> Result<Value, String> {
    let value = string(request, "value")?
        .parse::<i32>()
        .map_err(|error| error.to_string())?;
    let facing = match string(request, "type")? {
        "3d" => Direction::from_3d_data_value(value),
        "2d" => Direction::from_2d_data_value(value),
        _ => return Err("unknown Direction data type".into()),
    };
    Ok(json!({"value": direction_snapshot(facing)}))
}

pub(super) fn direction_y_rot_case(request: &Value) -> Result<Value, String> {
    let y_rot = f64_field(request, "value")?;
    Ok(json!({
        "value": direction_snapshot(Direction::from_y_rot(y_rot)),
    }))
}

pub(super) fn direction_axis_case(request: &Value) -> Result<Value, String> {
    let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
    let direction = match string(request, "direction")? {
        "positive" => AxisDirection::Positive,
        "negative" => AxisDirection::Negative,
        _ => return Err("unknown axis direction".into()),
    };
    Ok(json!({
        "value": direction_snapshot(Direction::from_axis_and_direction(axis, direction)),
    }))
}

pub(super) fn direction_get_case(request: &Value) -> Result<Value, String> {
    let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
    let direction = match string(request, "direction")? {
        "positive" => AxisDirection::Positive,
        "negative" => AxisDirection::Negative,
        _ => return Err("unknown axis direction".into()),
    };
    Ok(json!({
        "value": direction_snapshot(Direction::get(direction, axis)),
    }))
}

pub(super) fn direction_unit_vectors_case(request: &Value) -> Result<Value, String> {
    let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
    let (x, y, z) = facing.step();
    Ok(json!({
        "unit_vec3i": {"x": x, "y": y, "z": z},
        "unit_vec3": vec3_snapshot(facing.unit_vec3()),
        "unit_vec3f": vec3f_snapshot(facing.unit_vec3f()),
    }))
}

pub(super) fn direction_step_case(request: &Value) -> Result<Value, String> {
    let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
    Ok(json!({
        "step_x": facing.get_step_x(),
        "step_y": facing.get_step_y(),
        "step_z": facing.get_step_z(),
        "step": vec3f_snapshot(facing.step_vec3f()),
    }))
}

pub(super) fn direction_axis_orientation_case(request: &Value) -> Result<Value, String> {
    let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
    Ok(json!({
        "vertical": axis.is_vertical(),
        "horizontal": axis.is_horizontal(),
    }))
}

pub(super) fn direction_axis_directions_case(request: &Value) -> Result<Value, String> {
    let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
    let directions = axis.get_directions();
    Ok(json!({
        "positive": direction_snapshot(axis.get_positive()),
        "negative": direction_snapshot(axis.get_negative()),
        "directions": directions.into_iter().map(direction_snapshot).collect::<Vec<_>>(),
    }))
}

pub(super) fn direction_axis_direction_values_case(request: &Value) -> Result<Value, String> {
    let direction = match string(request, "direction")? {
        "positive" => AxisDirection::Positive,
        "negative" => AxisDirection::Negative,
        _ => return Err("unknown axis direction".into()),
    };
    Ok(json!({
        "step": direction.get_step(),
        "name": direction.name(),
        "opposite": direction.opposite().name(),
    }))
}

pub(super) fn direction_axis_choose_case(request: &Value) -> Result<Value, String> {
    let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
    let int_value = |name: &str| {
        string(request, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    let double_value = |name: &str| f64_field(request, name);
    let bool_value = |name: &str| {
        request[name]
            .as_bool()
            .ok_or_else(|| format!("{name} must be a boolean"))
    };
    Ok(json!({
        "int": axis.choose_i32(int_value("int_x")?, int_value("int_y")?, int_value("int_z")?),
        "double": format!("{:016x}", axis.choose_f64(
            double_value("double_x")?, double_value("double_y")?, double_value("double_z")?,
        ).to_bits()),
        "boolean": axis.choose_bool(
            bool_value("bool_x")?, bool_value("bool_y")?, bool_value("bool_z")?,
        ),
    }))
}

pub(super) fn direction_axis_test_case(request: &Value) -> Result<Value, String> {
    let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
    let input = match string(request, "direction")? {
        "none" => None,
        name => Some(Direction::from_name(name).ok_or("unknown direction")?),
    };
    Ok(json!({"value": axis.test(input)}))
}

pub(super) fn direction_axis_plane_case(request: &Value) -> Result<Value, String> {
    let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
    let plane = match axis.get_plane() {
        Plane::Horizontal => "horizontal",
        Plane::Vertical => "vertical",
    };
    Ok(json!({"plane": plane}))
}

pub(super) fn direction_plane_test_case(request: &Value) -> Result<Value, String> {
    let plane = match string(request, "plane")? {
        "horizontal" => Plane::Horizontal,
        "vertical" => Plane::Vertical,
        _ => return Err("unknown plane".into()),
    };
    let input = match string(request, "direction")? {
        "none" => None,
        name => Some(Direction::from_name(name).ok_or("unknown direction")?),
    };
    Ok(json!({"value": plane.test(input)}))
}

pub(super) fn direction_plane_order_case(request: &Value) -> Result<Value, String> {
    let plane = match string(request, "plane")? {
        "horizontal" => Plane::Horizontal,
        "vertical" => Plane::Vertical,
        _ => return Err("unknown plane".into()),
    };
    Ok(json!({
        "faces": plane.directions().iter().map(|direction| direction.name()).collect::<Vec<_>>(),
        "stream_faces": plane.directions().iter().map(|direction| direction.name()).collect::<Vec<_>>(),
        "axes": plane.axes().iter().map(|axis| axis.name()).collect::<Vec<_>>(),
        "length": plane.length(),
    }))
}

pub(super) fn direction_random_case(request: &Value) -> Result<Value, String> {
    let seed = string(request, "seed")?
        .parse::<i64>()
        .map_err(|error| error.to_string())?;
    let plane = match string(request, "plane")? {
        "horizontal" => Plane::Horizontal,
        "vertical" => Plane::Vertical,
        _ => return Err("unknown plane".into()),
    };
    let mut random = RandomSource::create(seed);
    let direction = Direction::get_random(&mut random);
    let axis = Axis::get_random(&mut random);
    let plane_direction = plane.get_random_direction(&mut random);
    let plane_axis = plane.get_random_axis(&mut random);
    Ok(json!({
        "direction": direction_snapshot(direction),
        "axis": axis.name(),
        "plane_direction": direction_snapshot(plane_direction),
        "plane_axis": plane_axis.name(),
        "tail": random.next_int().to_string(),
    }))
}

pub(super) fn direction_plane_shuffle_case(request: &Value) -> Result<Value, String> {
    let seed = string(request, "seed")?
        .parse::<i64>()
        .map_err(|error| error.to_string())?;
    let plane = match string(request, "plane")? {
        "horizontal" => Plane::Horizontal,
        "vertical" => Plane::Vertical,
        _ => return Err("unknown plane".into()),
    };
    let mut random = RandomSource::create(seed);
    let faces = plane
        .shuffled_copy(&mut random)
        .into_iter()
        .map(|direction| direction.name())
        .collect::<Vec<_>>();
    Ok(json!({
        "faces": faces,
        "tail": random.next_int().to_string(),
    }))
}

pub(super) fn direction_all_shuffle_case(request: &Value) -> Result<Value, String> {
    let seed = string(request, "seed")?
        .parse::<i64>()
        .map_err(|error| error.to_string())?;
    let mut random = RandomSource::create(seed);
    let faces = Direction::all_shuffled(&mut random)
        .into_iter()
        .map(|direction| direction.name())
        .collect::<Vec<_>>();
    Ok(json!({
        "faces": faces,
        "tail": random.next_int().to_string(),
    }))
}

pub(super) fn direction_stream_case() -> Result<Value, String> {
    let faces = Direction::stream()
        .map(|direction| direction.name())
        .collect::<Vec<_>>();
    Ok(json!({"faces": faces}))
}

pub(super) fn direction_to_y_rot_case(request: &Value) -> Result<Value, String> {
    let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
    Ok(json!({
        "bits": format!("{:08x}", facing.to_y_rot().to_bits()),
    }))
}

pub(super) fn direction_get_y_rot_case(request: &Value) -> Result<Value, String> {
    let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
    Ok(match Direction::get_y_rot(facing) {
        Ok(value) => json!({"outcome": "value", "bits": format!("{:08x}", value.to_bits())}),
        Err(DirectionYRotError::Vertical(direction)) => json!({
            "outcome": "error",
            "error": format!("No y-Rot for vertical axis: {}", direction.name()),
        }),
    })
}

pub(super) fn direction_rotation_case(request: &Value) -> Result<Value, String> {
    let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
    Ok(json!({"value": quaternion_snapshot(facing.get_rotation())}))
}

pub(super) fn direction_name_lookup_case(request: &Value) -> Result<Value, String> {
    let name = string(request, "name")?;
    match string(request, "kind")? {
        "direction" => Ok(match Direction::from_name(name) {
            Some(direction) => json!({
                "found": true,
                "name": direction.name(),
                "serialized_name": direction.name(),
                "text": direction.name(),
            }),
            None => json!({"found": false}),
        }),
        "axis" => Ok(match Axis::from_name(name) {
            Some(axis) => json!({
                "found": true,
                "name": axis.name(),
                "serialized_name": axis.name(),
                "text": axis.name(),
            }),
            None => json!({"found": false}),
        }),
        _ => Err("unknown name lookup kind".into()),
    }
}

pub(super) fn direction_enum_text_case(request: &Value) -> Result<Value, String> {
    let name = string(request, "name")?;
    match string(request, "kind")? {
        "axis_direction" => {
            let direction = match name {
                "positive" => AxisDirection::Positive,
                "negative" => AxisDirection::Negative,
                _ => return Err("unknown axis direction".into()),
            };
            Ok(json!({
                "enum_name": direction.enum_name(),
                "name": direction.name(),
                "text": direction.to_string_java(),
            }))
        }
        "plane" => {
            let plane = match name {
                "horizontal" => Plane::Horizontal,
                "vertical" => Plane::Vertical,
                _ => return Err("unknown plane".into()),
            };
            Ok(json!({
                "enum_name": plane.enum_name(),
                "text": plane.to_string_java(),
            }))
        }
        _ => Err("unknown enum text kind".into()),
    }
}

pub(super) fn direction_enum_name_case(request: &Value) -> Result<Value, String> {
    let name = string(request, "name")?;
    match string(request, "kind")? {
        "direction" => {
            let direction = Direction::from_name(name).ok_or("unknown direction name")?;
            Ok(json!({
                "enum_name": direction.enum_name(),
                "name": direction.name(),
                "serialized_name": direction.name(),
                "text": direction.name(),
            }))
        }
        "axis" => {
            let axis = Axis::from_name(name).ok_or("unknown axis name")?;
            Ok(json!({
                "enum_name": axis.enum_name(),
                "name": axis.name(),
                "serialized_name": axis.name(),
                "text": axis.name(),
            }))
        }
        _ => Err("unknown enum name kind".into()),
    }
}

pub(super) fn direction_approximate_nearest_case(request: &Value) -> Result<Value, String> {
    let facing = Direction::approximate_nearest(
        f64_field(request, "x")?,
        f64_field(request, "y")?,
        f64_field(request, "z")?,
    );
    Ok(json!({"value": direction_snapshot(facing)}))
}

pub(super) fn direction_approximate_nearest_overloads_case(
    request: &Value,
) -> Result<Value, String> {
    let f32_facing = Direction::approximate_nearest_f32(
        f32_field(request, "f32_x")?,
        f32_field(request, "f32_y")?,
        f32_field(request, "f32_z")?,
    );
    let f64_vector = Vec3::new(
        f64_field(request, "x")?,
        f64_field(request, "y")?,
        f64_field(request, "z")?,
    );
    Ok(json!({
        "f32": direction_snapshot(f32_facing),
        "f64": direction_snapshot(Direction::approximate_nearest(
            f64_vector.x, f64_vector.y, f64_vector.z,
        )),
        "vec3": direction_snapshot(Direction::approximate_nearest_vec3(f64_vector)),
    }))
}

pub(super) fn direction_nearest_case(request: &Value) -> Result<Value, String> {
    let coordinate = |name: &str| {
        string(request, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    let or_else = match string(request, "or_else")? {
        "none" => None,
        name => Some(Direction::from_name(name).ok_or("unknown fallback direction")?),
    };
    let facing = Direction::get_nearest(
        coordinate("x")?,
        coordinate("y")?,
        coordinate("z")?,
        or_else,
    );
    Ok(match facing {
        Some(facing) => json!({"value": direction_snapshot(facing)}),
        None => json!({}),
    })
}

pub(super) fn direction_axis_step_order_case(request: &Value) -> Result<Value, String> {
    let axes = Direction::axis_step_order(f64_field(request, "x")?, f64_field(request, "z")?);
    Ok(json!({
        "axes": axes.into_iter().map(Axis::name).collect::<Vec<_>>(),
    }))
}

pub(super) fn direction_is_facing_angle_case(request: &Value) -> Result<Value, String> {
    let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
    Ok(json!({
        "value": facing.is_facing_angle(f32_field(request, "angle")?),
    }))
}
