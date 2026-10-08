use super::*;

pub(crate) fn mth_create_insecure_uuid_case(request: &Value) -> Result<Value, String> {
    let seed = string(request, "seed")?
        .parse::<i64>()
        .map_err(|error| error.to_string())?;
    let mut random = RandomSource::create(seed);
    let uuid = math::mth_create_insecure_uuid(&mut random);
    Ok(json!({
        "most_hex": format!("{:016x}", uuid.most),
        "least_hex": format!("{:016x}", uuid.least),
    }))
}

pub(crate) fn mth_murmur_hash3_mixer_case(request: &Value) -> Result<Value, String> {
    let input = string(request, "input")?
        .parse::<i32>()
        .map_err(|error| error.to_string())?;
    Ok(json!({
        "value": math::mth_murmur_hash3_mixer(input).to_string(),
    }))
}

pub(crate) fn mth_binary_search_case(request: &Value) -> Result<Value, String> {
    let from = string(request, "from")?
        .parse::<i32>()
        .map_err(|error| error.to_string())?;
    let to = string(request, "to")?
        .parse::<i32>()
        .map_err(|error| error.to_string())?;
    let threshold = string(request, "threshold")?
        .parse::<i32>()
        .map_err(|error| error.to_string())?;
    let value = match string(request, "condition")? {
        "ge" => math::mth_binary_search(from, to, |value| value >= threshold),
        "gt" => math::mth_binary_search(from, to, |value| value > threshold),
        _ => return Err("unknown Mth binary search condition".into()),
    };
    Ok(json!({"value": value.to_string()}))
}

pub(crate) fn mth_ray_intersects_aabb_case(request: &Value) -> Result<Value, String> {
    let ray_start = f64_vec3(
        request
            .get("ray_start")
            .ok_or("ray_start must be an object")?,
    )?;
    let ray_dir = f64_vec3(request.get("ray_dir").ok_or("ray_dir must be an object")?)?;
    let aabb = aabb_box(request.get("aabb").ok_or("aabb must be an object")?)?;
    Ok(json!({
        "value": math::mth_ray_intersects_aabb(ray_start, ray_dir, aabb),
    }))
}

pub(crate) fn mth_fast_inv_sqrt_case(request: &Value) -> Result<Value, String> {
    let bits = math::mth_fast_inv_sqrt_f64(f64_field(request, "input")?).to_bits();
    Ok(json!({"bits": format!("{bits:016x}")}))
}

pub(crate) fn mth_fast_inv_cube_root_case(request: &Value) -> Result<Value, String> {
    let bits = math::mth_fast_inv_cube_root_f32(f32_field(request, "input")?).to_bits();
    Ok(json!({"bits": format!("{bits:08x}")}))
}

pub(crate) fn mth_hsv_to_color_case(request: &Value) -> Result<Value, String> {
    let hue = f32_field(request, "hue")?;
    let saturation = f32_field(request, "saturation")?;
    let value = f32_field(request, "value")?;
    let alpha = string(request, "alpha")?
        .parse::<i32>()
        .map_err(|error| error.to_string())?;
    let color = match string(request, "mode")? {
        "rgb" => math::mth_hsv_to_rgb_f32(hue, saturation, value),
        "argb" => math::mth_hsv_to_argb_f32(hue, saturation, value, alpha),
        _ => return Err("unknown Mth HSV color mode".into()),
    };
    Ok(match color {
        Ok(value) => json!({"value": value.to_string()}),
        Err(error) => json!({"outcome": "error", "error": error}),
    })
}

pub(crate) fn mth_get_seed_case(request: &Value) -> Result<Value, String> {
    let value = math::mth_get_seed(
        i32_field(request, "x")?,
        i32_field(request, "y")?,
        i32_field(request, "z")?,
    );
    Ok(json!({"value": value.to_string()}))
}

pub(crate) fn mth_inv_sqrt_case(request: &Value) -> Result<Value, String> {
    let bits = match string(request, "type")? {
        "f32" => format!(
            "{:08x}",
            math::mth_inv_sqrt_f32(f32_field(request, "input")?).to_bits()
        ),
        "f64" => format!(
            "{:016x}",
            math::mth_inv_sqrt_f64(f64_field(request, "input")?).to_bits()
        ),
        _ => return Err("unknown Mth invSqrt type".into()),
    };
    Ok(json!({"bits": bits}))
}

pub(crate) fn mth_next_random_case(request: &Value) -> Result<Value, String> {
    let mut random = RandomSource::create(random_seed(request)?);
    let width = string(request, "type")?;
    match width {
        "i32" => {
            let value = math::mth_next_int(
                &mut random,
                i32_string_field(request, "min")?,
                i32_string_field(request, "max")?,
            )
            .map_err(|_| "bound must be positive".to_owned())?;
            Ok(json!({"value": value.to_string()}))
        }
        "f32" => Ok(json!({
            "bits": format!(
                "{:08x}",
                math::mth_next_float(
                    &mut random,
                    f32_field(request, "min")?,
                    f32_field(request, "max")?,
                )
                .to_bits()
            )
        })),
        "f64" => Ok(json!({
            "bits": format!(
                "{:016x}",
                math::mth_next_double(
                    &mut random,
                    f64_field(request, "min")?,
                    f64_field(request, "max")?,
                )
                .to_bits()
            )
        })),
        _ => Err("unknown Mth next random type".into()),
    }
}

pub(crate) fn mth_random_helpers_case(request: &Value) -> Result<Value, String> {
    let mut random = RandomSource::create(random_seed(request)?);
    match string(request, "fn")? {
        "inclusive" => {
            let value = math::mth_random_between_inclusive(
                &mut random,
                i32_string_field(request, "min")?,
                i32_string_field(request, "max")?,
            )
            .map_err(|_| "bound must be positive".to_owned())?;
            Ok(json!({
                "value": value.to_string(),
                "follow_up": random.next_int().to_string(),
            }))
        }
        "between" => {
            let value = math::mth_random_between(
                &mut random,
                f32_field(request, "min")?,
                f32_field(request, "max")?,
            );
            Ok(json!({
                "bits": format!("{:08x}", value.to_bits()),
                "follow_up_bits": format!("{:08x}", random.next_float().to_bits()),
            }))
        }
        "normal" => {
            let value = math::mth_normal(
                &mut random,
                f32_field(request, "mean")?,
                f32_field(request, "deviation")?,
            );
            Ok(json!({
                "bits": format!("{:08x}", value.to_bits()),
                "follow_up_bits": format!("{:016x}", random.next_gaussian().to_bits()),
            }))
        }
        _ => Err("unknown Mth random helper".into()),
    }
}
