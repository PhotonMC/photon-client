use super::*;

pub(crate) fn mth_clamped_lerp_case(request: &Value) -> Result<Value, String> {
    let width = string(request, "type")?;
    let bits = match width {
        "f32" => format!(
            "{:08x}",
            math::clamped_lerp_f32(
                f32_field(request, "factor")?,
                f32_field(request, "min")?,
                f32_field(request, "max")?,
            )
            .to_bits()
        ),
        "f64" => format!(
            "{:016x}",
            math::clamped_lerp_f64(
                f64_field(request, "factor")?,
                f64_field(request, "min")?,
                f64_field(request, "max")?,
            )
            .to_bits()
        ),
        _ => return Err("unknown Mth clampedLerp type".into()),
    };
    Ok(json!({"bits": bits}))
}

pub(crate) fn mth_clamped_map_f64_case(request: &Value) -> Result<Value, String> {
    let value = math::mth_clamped_map_f64(
        f64_field(request, "value")?,
        f64_field(request, "from_min")?,
        f64_field(request, "from_max")?,
        f64_field(request, "to_min")?,
        f64_field(request, "to_max")?,
    );
    Ok(json!({"bits": format!("{:016x}", value.to_bits())}))
}

pub(crate) fn mth_clamped_map_f32_case(request: &Value) -> Result<Value, String> {
    let value = math::mth_clamped_map_f32(
        f32_field(request, "value")?,
        f32_field(request, "from_min")?,
        f32_field(request, "from_max")?,
        f32_field(request, "to_min")?,
        f32_field(request, "to_max")?,
    );
    Ok(json!({"bits": format!("{:08x}", value.to_bits())}))
}

pub(crate) fn mth_map_f64_case(request: &Value) -> Result<Value, String> {
    let value = math::mth_map_f64(
        f64_field(request, "value")?,
        f64_field(request, "from_min")?,
        f64_field(request, "from_max")?,
        f64_field(request, "to_min")?,
        f64_field(request, "to_max")?,
    );
    Ok(json!({"bits": format!("{:016x}", value.to_bits())}))
}

pub(crate) fn mth_map_f32_case(request: &Value) -> Result<Value, String> {
    let value = math::mth_map_f32(
        f32_field(request, "value")?,
        f32_field(request, "from_min")?,
        f32_field(request, "from_max")?,
        f32_field(request, "to_min")?,
        f32_field(request, "to_max")?,
    );
    Ok(json!({"bits": format!("{:08x}", value.to_bits())}))
}

pub(crate) fn mth_wobble_f64_case(request: &Value) -> Result<Value, String> {
    let value = math::mth_wobble_f64(f64_field(request, "coord")?);
    Ok(json!({"bits": format!("{:016x}", value.to_bits())}))
}

pub(crate) fn mth_interpolation_case(request: &Value) -> Result<Value, String> {
    let function = string(request, "fn")?;
    let bits = match function {
        "lerp2" => format!(
            "{:016x}",
            math::mth_lerp2_f64(
                f64_field(request, "alpha1")?,
                f64_field(request, "alpha2")?,
                f64_field(request, "x00")?,
                f64_field(request, "x10")?,
                f64_field(request, "x01")?,
                f64_field(request, "x11")?,
            )
            .to_bits()
        ),
        "lerp3" => format!(
            "{:016x}",
            math::mth_lerp3_f64(
                f64_field(request, "alpha1")?,
                f64_field(request, "alpha2")?,
                f64_field(request, "alpha3")?,
                f64_field(request, "x000")?,
                f64_field(request, "x100")?,
                f64_field(request, "x010")?,
                f64_field(request, "x110")?,
                f64_field(request, "x001")?,
                f64_field(request, "x101")?,
                f64_field(request, "x011")?,
                f64_field(request, "x111")?,
            )
            .to_bits()
        ),
        "catmullrom" => format!(
            "{:08x}",
            math::mth_catmullrom_f32(
                f32_field(request, "alpha")?,
                f32_field(request, "p0")?,
                f32_field(request, "p1")?,
                f32_field(request, "p2")?,
                f32_field(request, "p3")?,
            )
            .to_bits()
        ),
        _ => return Err("unknown Mth interpolation function".into()),
    };
    Ok(json!({"bits": bits}))
}

pub(crate) fn mth_quantize_case(request: &Value) -> Result<Value, String> {
    let value = f64_field(request, "value")?;
    let resolution = string(request, "resolution")?
        .parse::<i32>()
        .map_err(|error| error.to_string())?;
    Ok(json!({
        "value": math::mth_quantize(value, resolution).to_string()
    }))
}

pub(crate) fn mth_inverse_lerp_case(request: &Value) -> Result<Value, String> {
    let bits = match string(request, "type")? {
        "f32" => format!(
            "{:08x}",
            math::mth_inverse_lerp_f32(
                f32_field(request, "value")?,
                f32_field(request, "min")?,
                f32_field(request, "max")?,
            )
            .to_bits()
        ),
        "f64" => format!(
            "{:016x}",
            math::mth_inverse_lerp_f64(
                f64_field(request, "value")?,
                f64_field(request, "min")?,
                f64_field(request, "max")?,
            )
            .to_bits()
        ),
        _ => return Err("unknown Mth inverseLerp type".into()),
    };
    Ok(json!({"bits": bits}))
}

pub(crate) fn mth_lerp_case(request: &Value) -> Result<Value, String> {
    let bits = match string(request, "type")? {
        "f32" => format!(
            "{:08x}",
            math::mth_lerp_f32(
                f32_field(request, "alpha")?,
                f32_field(request, "p0")?,
                f32_field(request, "p1")?,
            )
            .to_bits()
        ),
        "f64" => format!(
            "{:016x}",
            math::mth_lerp_f64(
                f64_field(request, "alpha")?,
                f64_field(request, "p0")?,
                f64_field(request, "p1")?,
            )
            .to_bits()
        ),
        _ => return Err("unknown Mth lerp type".into()),
    };
    Ok(json!({"bits": bits}))
}

pub(crate) fn mth_rot_lerp_case(request: &Value) -> Result<Value, String> {
    let bits = match string(request, "type")? {
        "f32" => format!(
            "{:08x}",
            math::mth_rot_lerp_f32(
                f32_field(request, "alpha")?,
                f32_field(request, "from")?,
                f32_field(request, "to")?,
            )
            .to_bits()
        ),
        "f64" => format!(
            "{:016x}",
            math::mth_rot_lerp_f64(
                f64_field(request, "alpha")?,
                f64_field(request, "from")?,
                f64_field(request, "to")?,
            )
            .to_bits()
        ),
        _ => return Err("unknown Mth rotLerp type".into()),
    };
    Ok(json!({"bits": bits}))
}

pub(crate) fn mth_rot_lerp_rad_case(request: &Value) -> Result<Value, String> {
    let value = math::mth_rot_lerp_rad(
        f32_field(request, "alpha")?,
        f32_field(request, "from")?,
        f32_field(request, "to")?,
    );
    Ok(json!({"bits": format!("{:08x}", value.to_bits())}))
}

pub(crate) fn mth_triangle_wave_case(request: &Value) -> Result<Value, String> {
    let value =
        math::mth_triangle_wave(f32_field(request, "index")?, f32_field(request, "period")?);
    Ok(json!({"bits": format!("{:08x}", value.to_bits())}))
}

pub(crate) fn mth_lerp_steps_case(request: &Value, operation: &str) -> Result<Value, String> {
    let alpha = f32_field(request, "alpha")?;
    let start = i32_string_field(request, "p0")?;
    let end = i32_string_field(request, "p1")?;
    let value = if operation == "mth_lerp_int" {
        math::mth_lerp_int(alpha, start, end)
    } else {
        math::mth_lerp_discrete(alpha, start, end)
    };
    Ok(json!({"value": value.to_string()}))
}

pub(crate) fn mth_smoothstep_pair_case(request: &Value, operation: &str) -> Result<Value, String> {
    let x = f64_field(request, "x")?;
    let value = if operation == "mth_smoothstep" {
        math::mth_smoothstep(x)
    } else {
        math::mth_smoothstep_derivative(x)
    };
    Ok(json!({"bits": format!("{:016x}", value.to_bits())}))
}
