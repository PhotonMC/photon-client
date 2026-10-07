use super::*;

pub(crate) fn mth_angle_wrap_difference_case(request: &Value) -> Result<Value, String> {
    if string(request, "type")? != "f32" {
        return Err("unknown Mth angle wrap/difference type".into());
    }
    let value = match string(request, "fn")? {
        "wrap_degrees_90" => math::mth_wrap_degrees_90_f32(f32_field(request, "value")?),
        "degrees_difference" => {
            math::mth_degrees_difference_f32(f32_field(request, "from")?, f32_field(request, "to")?)
        }
        "degrees_difference_abs" => math::mth_degrees_difference_abs_f32(
            f32_field(request, "from")?,
            f32_field(request, "to")?,
        ),
        _ => return Err("unknown Mth angle wrap/difference function".into()),
    };
    Ok(json!({"bits": format!("{:08x}", value.to_bits())}))
}

pub(crate) fn mth_angle_approach_case(request: &Value) -> Result<Value, String> {
    if string(request, "type")? != "f32" {
        return Err("unknown Mth angle approach type".into());
    }
    let value = match string(request, "fn")? {
        "rotate_if_necessary" => math::mth_rotate_if_necessary_f32(
            f32_field(request, "base")?,
            f32_field(request, "target")?,
            f32_field(request, "max")?,
        ),
        "approach" => math::mth_approach_f32(
            f32_field(request, "current")?,
            f32_field(request, "target")?,
            f32_field(request, "increment")?,
        ),
        "approach_degrees" => math::mth_approach_degrees_f32(
            f32_field(request, "current")?,
            f32_field(request, "target")?,
            f32_field(request, "increment")?,
        ),
        _ => return Err("unknown Mth angle approach function".into()),
    };
    Ok(json!({"bits": format!("{:08x}", value.to_bits())}))
}

pub(crate) fn mth_atan2_case(request: &Value) -> Result<Value, String> {
    let value = math::mth_atan2(f64_field(request, "y")?, f64_field(request, "x")?);
    Ok(json!({"bits": format!("{:016x}", value.to_bits())}))
}

pub(crate) fn mth_angle_helpers_case(request: &Value) -> Result<Value, String> {
    let function = string(request, "fn")?;
    let input = string(request, "input")?;
    let result = match function {
        "pack_degrees" => {
            if string(request, "type")? != "f32" {
                return Err("unknown Mth packDegrees type".into());
            }
            let value = f32_field(request, "input")?;
            json!({"value": math::pack_degrees_f32(value).to_string()})
        }
        "unpack_degrees" => {
            if string(request, "type")? != "i8" {
                return Err("unknown Mth unpackDegrees type".into());
            }
            let value = input.parse::<i8>().map_err(|error| error.to_string())?;
            json!({"bits": format!("{:08x}", math::unpack_degrees_i8(value).to_bits())})
        }
        "wrap_degrees" => {
            if string(request, "type")? != "i64" {
                return Err("unknown Mth long wrapDegrees type".into());
            }
            let value = input.parse::<i64>().map_err(|error| error.to_string())?;
            json!({"bits": format!("{:08x}", math::wrap_degrees_i64(value).to_bits())})
        }
        _ => return Err("unknown Mth angle function".into()),
    };
    Ok(result)
}

pub(crate) fn mth_rotation_around_axis_case(request: &Value) -> Result<Value, String> {
    let axis = Vec3f::new(
        f32_field(request, "axis_x")?,
        f32_field(request, "axis_y")?,
        f32_field(request, "axis_z")?,
    );
    let rotation = Quaternionf::new(
        f32_field(request, "rotation_x")?,
        f32_field(request, "rotation_y")?,
        f32_field(request, "rotation_z")?,
        f32_field(request, "rotation_w")?,
    );
    Ok(json!({
        "value": quaternion_snapshot(math::mth_rotation_around_axis(axis, rotation))
    }))
}
