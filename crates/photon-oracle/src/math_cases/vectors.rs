use super::*;

pub(super) fn vec3_codec_case(request: &Value) -> Result<Value, String> {
    match string(request, "mode")? {
        "decode" => {
            let values = request["values"]
                .as_array()
                .ok_or("values must be an array")?
                .iter()
                .enumerate()
                .map(|(index, value)| f64_value(value, &format!("values[{index}]")))
                .collect::<Result<Vec<_>, _>>()?;
            match Vec3::codec_decode(&values) {
                Ok(value) => Ok(json!({
                    "outcome": "value",
                    "value": vec3_snapshot(value),
                })),
                Err(math::Vec3CodecError::WrongLength { partial, .. }) => {
                    let mut result = json!({
                        "outcome": "error",
                        "error": "Input is not a list of 3 elements",
                    });
                    if let Some(value) = partial {
                        result["partial"] = vec3_snapshot(value);
                    }
                    Ok(result)
                }
            }
        }
        "encode" => {
            let value = Vec3::new(
                f64_field(request, "x")?,
                f64_field(request, "y")?,
                f64_field(request, "z")?,
            );
            Ok(json!({
                "outcome": "value",
                "value": value
                    .codec_encode()
                    .into_iter()
                    .map(|component| format!("{:016x}", component.to_bits()))
                    .collect::<Vec<_>>(),
            }))
        }
        _ => Err("unknown Vec3 codec mode".into()),
    }
}

pub(super) fn vec3_dot_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
    Ok(json!({"value": format!("{:016x}", first.dot(second).to_bits())}))
}

pub(super) fn vec3_distance_to_sqr_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
    Ok(json!({"value": format!("{:016x}", first.distance_to_sqr(second).to_bits())}))
}

pub(super) fn vec3_distance_to_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
    Ok(json!({"value": format!("{:016x}", first.distance_to(second).to_bits())}))
}

pub(super) fn vec3_distance_to_sqr_components_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    let value = first.distance_to_sqr_components(
        f64_field(request, "x")?,
        f64_field(request, "y")?,
        f64_field(request, "z")?,
    );
    Ok(json!({"value": format!("{:016x}", value.to_bits())}))
}

pub(super) fn vec3_horizontal_distances_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    Ok(json!({
        "distance": format!("{:016x}", first.horizontal_distance().to_bits()),
        "distance_squared": format!("{:016x}", first.horizontal_distance_squared().to_bits()),
    }))
}

pub(super) fn vec3_closer_than_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
    Ok(json!({
        "position": first.closer_than_position(second, f64_field(request, "distance")?),
        "xz_y": first.closer_than_xz_y(
            second,
            f64_field(request, "distance_xz")?,
            f64_field(request, "distance_y")?,
        ),
    }))
}

pub(super) fn vec3_offset_random_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    let offset = f32_field(request, "offset")?;
    let seed = random_seed(request)?;
    let mut random = RandomSource::create(seed);
    let offset_random = first.offset_random(&mut random, offset);
    let mut random_xz = RandomSource::create(seed);
    let offset_random_xz = first.offset_random_xz(&mut random_xz, offset);
    Ok(json!({
        "offset_random": vec3_snapshot(offset_random),
        "offset_random_xz": vec3_snapshot(offset_random_xz),
    }))
}

pub(super) fn vec3_lerp_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
    let alpha = f64_field(request, "alpha")?;
    Ok(json!({"value": vec3_snapshot(first.lerp(second, alpha))}))
}

pub(super) fn vec3_rotate_clockwise_90_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    Ok(json!({"value": vec3_snapshot(first.rotate_clockwise_90())}))
}

pub(super) fn vec3_is_finite_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    Ok(json!({"value": first.is_finite()}))
}

pub(super) fn vec3_get_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
    Ok(json!({"value": format!("{:016x}", first.get(axis).to_bits())}))
}

pub(super) fn vec3_with_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
    let value = f64_field(request, "value")?;
    Ok(json!({"value": vec3_snapshot(first.with(axis, value))}))
}

pub(super) fn vec3_relative_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    let direction =
        Direction::from_name(string(request, "direction")?).ok_or("unknown direction")?;
    let distance = f64_field(request, "distance")?;
    Ok(json!({"value": vec3_snapshot(first.relative(direction, distance))}))
}

pub(super) fn vec3_align_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    let axis_values = request["axes"].as_array().ok_or("axes must be an array")?;
    let mut axes = Vec::with_capacity(axis_values.len());
    for axis in axis_values {
        let name = axis.as_str().ok_or("axis must be a string")?;
        axes.push(Axis::from_name(name).ok_or("unknown axis")?);
    }
    Ok(json!({"value": vec3_snapshot(first.align(&axes))}))
}

pub(super) fn vec3_projected_on_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    let onto = f64_vec3(request.get("onto").ok_or("onto must be an object")?)?;
    Ok(json!({"value": vec3_snapshot(first.projected_on(onto))}))
}

pub(super) fn vec3_to_vector3f_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    Ok(json!({"value": vec3f_snapshot(first.to_vector3f())}))
}

pub(super) fn vec3_coordinates_case(request: &Value) -> Result<Value, String> {
    let pos = blockpos(request)?;
    let value = match string(request, "kind")? {
        "lower_corner" => Vec3::at_lower_corner_of(pos),
        "lower_corner_offset" => Vec3::at_lower_corner_with_offset(
            pos,
            f64_field(request, "offset_x")?,
            f64_field(request, "offset_y")?,
            f64_field(request, "offset_z")?,
        ),
        "center" => Vec3::at_center_of(pos),
        "bottom_center" => Vec3::at_bottom_center_of(pos),
        "up_from_bottom_center" => {
            Vec3::up_from_bottom_center_of(pos, f64_field(request, "y_offset")?)
        }
        _ => return Err("unknown Vec3 coordinate helper".into()),
    };
    Ok(json!({"value": vec3_snapshot(value)}))
}

pub(super) fn vec3_equality_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
    Ok(json!({
        "equals": first.equals_java(second),
        "first_hash": first.hash_code_java().to_string(),
        "second_hash": second.hash_code_java().to_string(),
    }))
}

pub(super) fn vec3_to_string_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    Ok(json!({"value": first.to_string_java()}))
}

pub(super) fn vec3_rotate_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    let radians = f32_field(request, "radians")?;
    let value = match string(request, "axis")? {
        "x" => first.x_rot(radians),
        "y" => first.y_rot(radians),
        "z" => first.z_rot(radians),
        _ => return Err("unknown Vec3 rotation axis".into()),
    };
    Ok(json!({"value": vec3_snapshot(value)}))
}

pub(super) fn vec3_direction_from_rotation_case(request: &Value) -> Result<Value, String> {
    let value = match string(request, "kind")? {
        "components" => Vec3::direction_from_rotation_components(
            f32_field(request, "rot_x")?,
            f32_field(request, "rot_y")?,
        ),
        "vec2" => {
            let rotation = request
                .get("rotation")
                .ok_or("rotation must be an object")?;
            Vec3::direction_from_rotation(Vec2::new(
                f32_field(rotation, "x")?,
                f32_field(rotation, "y")?,
            ))
        }
        _ => return Err("unknown Vec3 direction rotation kind".into()),
    };
    Ok(json!({"value": vec3_snapshot(value)}))
}

pub(super) fn vec3_local_coordinates_case(request: &Value) -> Result<Value, String> {
    let value = match string(request, "kind")? {
        "apply" => {
            let rotation = request
                .get("rotation")
                .ok_or("rotation must be an object")?;
            Vec3::apply_local_coordinates_to_rotation(
                Vec2::new(f32_field(rotation, "x")?, f32_field(rotation, "y")?),
                f64_vec3(
                    request
                        .get("direction")
                        .ok_or("direction must be an object")?,
                )?,
            )
        }
        "add" => {
            let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
            let direction = f64_vec3(
                request
                    .get("direction")
                    .ok_or("direction must be an object")?,
            )?;
            first.add_local_coordinates(direction)
        }
        _ => return Err("unknown Vec3 local coordinate kind".into()),
    };
    Ok(json!({"value": vec3_snapshot(value)}))
}

pub(super) fn vec3_rotation_case(request: &Value) -> Result<Value, String> {
    let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
    Ok(json!({"value": vec2_snapshot(first.rotation())}))
}

pub(super) fn vec2_rotate_case(request: &Value) -> Result<Value, String> {
    let first = request.get("first").ok_or("first must be an object")?;
    let vector = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
    Ok(json!({
        "value": vec2_snapshot(vector.rotate(f64_field(request, "angle")?)),
    }))
}

pub(super) fn vec2_dot_case(request: &Value) -> Result<Value, String> {
    let first = request.get("first").ok_or("first must be an object")?;
    let second = request.get("second").ok_or("second must be an object")?;
    let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
    let second = Vec2::new(f32_field(second, "x")?, f32_field(second, "y")?);
    Ok(json!({
        "value": format!("{:08x}", first.dot(second).to_bits()),
    }))
}

pub(super) fn vec2_add_vec2_case(request: &Value) -> Result<Value, String> {
    let first = request.get("first").ok_or("first must be an object")?;
    let second = request.get("second").ok_or("second must be an object")?;
    let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
    let second = Vec2::new(f32_field(second, "x")?, f32_field(second, "y")?);
    Ok(json!({
        "value": vec2_snapshot(first.add_vec2(second)),
    }))
}

pub(super) fn vec2_add_scalar_case(request: &Value) -> Result<Value, String> {
    let first = request.get("first").ok_or("first must be an object")?;
    let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
    Ok(json!({
        "value": vec2_snapshot(first.add_scalar(f32_field(request, "value")?)),
    }))
}

pub(super) fn vec2_distance_to_sqr_case(request: &Value) -> Result<Value, String> {
    let first = request.get("first").ok_or("first must be an object")?;
    let second = request.get("second").ok_or("second must be an object")?;
    let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
    let second = Vec2::new(f32_field(second, "x")?, f32_field(second, "y")?);
    Ok(json!({
        "value": format!("{:08x}", first.distance_to_sqr(second).to_bits()),
    }))
}

pub(super) fn vec2_negated_case(request: &Value) -> Result<Value, String> {
    let first = request.get("first").ok_or("first must be an object")?;
    let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
    Ok(json!({
        "value": vec2_snapshot(first.negated()),
    }))
}

pub(super) fn vec2_equality_case(request: &Value) -> Result<Value, String> {
    let first = request.get("first").ok_or("first must be an object")?;
    let second = request.get("second").ok_or("second must be an object")?;
    let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
    let second = Vec2::new(f32_field(second, "x")?, f32_field(second, "y")?);
    Ok(json!({
        "equals": first.equals_java(second),
        "first_hash": first.hash_code_java().to_string(),
        "second_hash": second.hash_code_java().to_string(),
    }))
}

pub(super) fn vec2_constant_case(request: &Value) -> Result<Value, String> {
    let value = match string(request, "name")? {
        "zero" => Vec2::ZERO,
        "one" => Vec2::ONE,
        "unit_x" => Vec2::UNIT_X,
        "neg_unit_x" => Vec2::NEG_UNIT_X,
        "unit_y" => Vec2::UNIT_Y,
        "neg_unit_y" => Vec2::NEG_UNIT_Y,
        "max" => Vec2::MAX,
        "min" => Vec2::MIN,
        _ => return Err("unknown Vec2 constant".into()),
    };
    Ok(json!({"value": vec2_snapshot(value)}))
}

pub(super) fn vec3_constant_case(request: &Value) -> Result<Value, String> {
    let value = match string(request, "name")? {
        "zero" => Vec3::ZERO,
        "x_axis" => Vec3::X_AXIS,
        "y_axis" => Vec3::Y_AXIS,
        "z_axis" => Vec3::Z_AXIS,
        _ => return Err("unknown Vec3 constant".into()),
    };
    Ok(json!({"value": vec3_snapshot(value)}))
}
