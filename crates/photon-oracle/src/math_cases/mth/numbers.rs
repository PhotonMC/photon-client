use super::*;

pub(crate) fn mth_floating_primitives_case(request: &Value) -> Result<Value, String> {
    let function = string(request, "fn")?;
    let bits = match function {
        "sin" => format!(
            "{:08x}",
            math::mth_sin_f64(f64_field(request, "value")?).to_bits()
        ),
        "cos" => format!(
            "{:08x}",
            math::mth_cos_f64(f64_field(request, "value")?).to_bits()
        ),
        "sqrt" => format!(
            "{:08x}",
            math::mth_sqrt_f32(f32_field(request, "value")?).to_bits()
        ),
        "abs" => format!(
            "{:08x}",
            math::mth_abs_f32(f32_field(request, "value")?).to_bits()
        ),
        "clamp" => match string(request, "type")? {
            "f32" => format!(
                "{:08x}",
                math::clamp_f32(
                    f32_field(request, "value")?,
                    f32_field(request, "min")?,
                    f32_field(request, "max")?,
                )
                .to_bits()
            ),
            "f64" => format!(
                "{:016x}",
                math::clamp_f64(
                    f64_field(request, "value")?,
                    f64_field(request, "min")?,
                    f64_field(request, "max")?,
                )
                .to_bits()
            ),
            _ => return Err("unknown Mth clamp type".into()),
        },
        _ => return Err("unknown Mth floating primitive".into()),
    };
    Ok(json!({"bits": bits}))
}

pub(crate) fn mth_constants_case(request: &Value) -> Result<Value, String> {
    match string(request, "name")? {
        "pi" => Ok(json!({"bits": format!("{:08x}", math::MTH_PI.to_bits())})),
        "half_pi" => Ok(json!({"bits": format!("{:08x}", math::MTH_HALF_PI.to_bits())})),
        "two_pi" => Ok(json!({"bits": format!("{:08x}", math::MTH_TWO_PI.to_bits())})),
        "deg_to_rad" => Ok(json!({"bits": format!("{:08x}", math::MTH_DEG_TO_RAD.to_bits())})),
        "rad_to_deg" => Ok(json!({"bits": format!("{:08x}", math::MTH_RAD_TO_DEG.to_bits())})),
        "epsilon" => Ok(json!({"bits": format!("{:08x}", math::MTH_EPSILON.to_bits())})),
        "sqrt_of_two" => Ok(json!({"bits": format!("{:08x}", math::MTH_SQRT_OF_TWO.to_bits())})),
        "y_axis" => Ok(json!({"value": vec3f_snapshot(math::MTH_Y_AXIS)})),
        "x_axis" => Ok(json!({"value": vec3f_snapshot(math::MTH_X_AXIS)})),
        "z_axis" => Ok(json!({"value": vec3f_snapshot(math::MTH_Z_AXIS)})),
        _ => Err("unknown Mth constant".into()),
    }
}

pub(crate) fn mth_power_helpers_case(request: &Value) -> Result<Value, String> {
    let function = string(request, "fn")?;
    let input = string(request, "input")?;
    let value = match string(request, "type")? {
        "i32" => {
            let input = input.parse::<i32>().map_err(|error| error.to_string())?;
            match function {
                "smallest_encompassing_power_of_two" => {
                    json!(math::smallest_encompassing_power_of_two(input).to_string())
                }
                "smallest_square_side" => {
                    json!(math::smallest_square_side(input)?.to_string())
                }
                "is_power_of_two" => json!(math::is_power_of_two_i32(input)),
                "ceillog2" => json!(math::ceillog2(input).to_string()),
                "log2" => json!(math::log2(input).to_string()),
                _ => return Err("unknown Mth i32 power helper".into()),
            }
        }
        "i64" => {
            let input = input.parse::<i64>().map_err(|error| error.to_string())?;
            if function != "is_power_of_two" {
                return Err("unknown Mth i64 power helper".into());
            }
            json!(math::is_power_of_two_i64(input))
        }
        _ => return Err("unknown Mth power helper type".into()),
    };
    Ok(json!({"value": value}))
}

pub(crate) fn mth_frac_case(request: &Value) -> Result<Value, String> {
    if string(request, "fn")? != "frac" {
        return Err("unknown Mth frac function".into());
    }
    let bits = match string(request, "type")? {
        "f32" => format!(
            "{:08x}",
            math::frac_f32(f32_field(request, "input")?).to_bits()
        ),
        "f64" => format!(
            "{:016x}",
            math::frac_f64(f64_field(request, "input")?).to_bits()
        ),
        _ => return Err("unknown Mth frac type".into()),
    };
    Ok(json!({"bits": bits}))
}

pub(crate) fn mth_get_int_case(request: &Value) -> Result<Value, String> {
    let input = if request["input"].is_null() {
        None
    } else {
        Some(string(request, "input")?)
    };
    let default = string(request, "default")?
        .parse::<i32>()
        .map_err(|error| error.to_string())?;
    Ok(json!({"value": math::mth_get_int(input, default).to_string()}))
}

pub(crate) fn mth_integer_helpers_case(request: &Value) -> Result<Value, String> {
    let function = string(request, "fn")?;
    let input = string(request, "input")?;
    let divisor = string(request, "divisor")?;
    let value = match string(request, "type")? {
        "i32" => {
            let input = input.parse::<i32>().map_err(|error| error.to_string())?;
            let divisor = divisor.parse::<i32>().map_err(|error| error.to_string())?;
            match function {
                "floor_div" => math::floor_div_i32(input, divisor),
                "positive_modulo" => math::positive_modulo_i32(input, divisor),
                "positive_ceil_div" => math::positive_ceil_div_i32(input, divisor),
                "round_toward" => math::round_toward_i32(input, divisor),
                _ => return Err("unknown Mth i32 function".into()),
            }
            .to_string()
        }
        "i64" => {
            let input = input.parse::<i64>().map_err(|error| error.to_string())?;
            let divisor = divisor.parse::<i64>().map_err(|error| error.to_string())?;
            match function {
                "positive_ceil_div" => math::positive_ceil_div_i64(input, divisor),
                "round_toward" => math::round_toward_i64(input, divisor),
                _ => return Err("unknown Mth i64 function".into()),
            }
            .to_string()
        }
        _ => return Err("unknown Mth integer type".into()),
    };
    Ok(json!({"value": value}))
}

pub(crate) fn mth_integer_basics_case(request: &Value) -> Result<Value, String> {
    let function = string(request, "fn")?;
    let input = string(request, "input")?;
    let value = match string(request, "type")? {
        "i32" => {
            let input = input.parse::<i32>().map_err(|error| error.to_string())?;
            match function {
                "abs" => json!(math::abs_i32(input).to_string()),
                "clamp" => json!(
                    math::clamp_i32(
                        input,
                        string(request, "min")?
                            .parse::<i32>()
                            .map_err(|error| error.to_string())?,
                        string(request, "max")?
                            .parse::<i32>()
                            .map_err(|error| error.to_string())?,
                    )
                    .to_string()
                ),
                "is_multiple" => json!(math::is_multiple_of_i32(
                    input,
                    string(request, "divisor")?
                        .parse::<i32>()
                        .map_err(|error| error.to_string())?,
                )),
                "wrap_degrees" => json!(math::wrap_degrees_i32(input).to_string()),
                _ => return Err("unknown Mth i32 basic function".into()),
            }
        }
        "i64" => {
            let input = input.parse::<i64>().map_err(|error| error.to_string())?;
            if function != "clamp" {
                return Err("unknown Mth i64 basic function".into());
            }
            json!(
                math::clamp_i64(
                    input,
                    string(request, "min")?
                        .parse::<i64>()
                        .map_err(|error| error.to_string())?,
                    string(request, "max")?
                        .parse::<i64>()
                        .map_err(|error| error.to_string())?,
                )
                .to_string()
            )
        }
        _ => return Err("unknown Mth integer basic type".into()),
    };
    Ok(json!({"value": value}))
}

pub(crate) fn mth_rounding_helpers_case(request: &Value) -> Result<Value, String> {
    let width = string(request, "type")?;
    let function = string(request, "fn")?;
    let value = match (function, width) {
        ("ceil", "f32") => math::ceil_f32(f32_field(request, "input")?).to_string(),
        ("ceil", "f64") => math::ceil_f64(f64_field(request, "input")?).to_string(),
        ("lfloor", "f64") => math::lfloor_f64(f64_field(request, "input")?).to_string(),
        ("ceil_long", "f64") => math::ceil_long_f64(f64_field(request, "input")?).to_string(),
        ("ceil", _) => return Err("unknown Mth ceil type".into()),
        ("lfloor", _) => return Err("unknown Mth lfloor type".into()),
        ("ceil_long", _) => return Err("unknown Mth ceilLong type".into()),
        _ => return Err("unknown Mth rounding function".into()),
    };
    Ok(json!({"value": value}))
}

pub(crate) fn mth_positive_modulo_case(request: &Value) -> Result<Value, String> {
    let width = string(request, "type")?;
    let bits = match width {
        "f32" => format!(
            "{:08x}",
            math::positive_modulo_f32(f32_field(request, "input")?, f32_field(request, "mod")?)
                .to_bits()
        ),
        "f64" => format!(
            "{:016x}",
            math::positive_modulo_f64(f64_field(request, "input")?, f64_field(request, "mod")?)
                .to_bits()
        ),
        _ => return Err("unknown Mth positiveModulo type".into()),
    };
    Ok(json!({"bits": bits}))
}

pub(crate) fn mth_abs_max_case(request: &Value) -> Result<Value, String> {
    let width = string(request, "type")?;
    let bits = match width {
        "f32" => format!(
            "{:08x}",
            math::abs_max_f32(f32_field(request, "first")?, f32_field(request, "second")?)
                .to_bits()
        ),
        "f64" => format!(
            "{:016x}",
            math::abs_max_f64(f64_field(request, "first")?, f64_field(request, "second")?)
                .to_bits()
        ),
        _ => return Err("unknown Mth absMax type".into()),
    };
    Ok(json!({"bits": bits}))
}

pub(crate) fn mth_equal_case(request: &Value) -> Result<Value, String> {
    let width = string(request, "type")?;
    let equal = match width {
        "f32" => math::equal_f32(f32_field(request, "first")?, f32_field(request, "second")?),
        "f64" => math::equal_f64(f64_field(request, "first")?, f64_field(request, "second")?),
        _ => return Err("unknown Mth equal type".into()),
    };
    Ok(json!({"equal": equal}))
}

pub(crate) fn mth_length_case(request: &Value) -> Result<Value, String> {
    let width = string(request, "type")?;
    let function = string(request, "fn")?;
    let bits = match (width, function) {
        ("f64_2", "length_squared") => format!(
            "{:016x}",
            math::mth_length_squared_f64_2(f64_field(request, "x")?, f64_field(request, "y")?,)
                .to_bits()
        ),
        ("f64_2", "length") => format!(
            "{:016x}",
            math::mth_length_f64_2(f64_field(request, "x")?, f64_field(request, "y")?,).to_bits()
        ),
        ("f32_2", "length") => format!(
            "{:08x}",
            math::mth_length_f32_2(f32_field(request, "x")?, f32_field(request, "y")?,).to_bits()
        ),
        ("f64_3", "length_squared") => format!(
            "{:016x}",
            math::mth_length_squared_f64_3(
                f64_field(request, "x")?,
                f64_field(request, "y")?,
                f64_field(request, "z")?,
            )
            .to_bits()
        ),
        ("f64_3", "length") => format!(
            "{:016x}",
            math::mth_length_f64_3(
                f64_field(request, "x")?,
                f64_field(request, "y")?,
                f64_field(request, "z")?,
            )
            .to_bits()
        ),
        ("f32_3", "length_squared") => format!(
            "{:08x}",
            math::mth_length_squared_f32_3(
                f32_field(request, "x")?,
                f32_field(request, "y")?,
                f32_field(request, "z")?,
            )
            .to_bits()
        ),
        _ => return Err("unknown Mth length overload".into()),
    };
    Ok(json!({"bits": bits}))
}

pub(crate) fn mth_out_from_origin_case(request: &Value) -> Result<Value, String> {
    let origin = i32_string_field(request, "origin")?;
    let lower_bound = i32_string_field(request, "lower")?;
    let upper_bound = i32_string_field(request, "upper")?;
    let step_size = match string(request, "mode")? {
        "default" => 1,
        "step" => i32_string_field(request, "step")?,
        _ => return Err("unknown Mth outFromOrigin mode".into()),
    };
    let result = match math::mth_out_from_origin(origin, lower_bound, upper_bound, step_size) {
        Ok(values) => json!({"outcome": "value", "value": values}),
        Err(error) => json!({"outcome": "error", "error": error}),
    };
    Ok(result)
}

pub(crate) fn mth_mul_and_truncate_case(request: &Value) -> Result<Value, String> {
    let numerator = i32_string_field(request, "numerator")?;
    let denominator = i32_string_field(request, "denominator")?;
    let factor = i32_string_field(request, "factor")?;
    Ok(json!({
        "value": math::mth_mul_and_truncate(numerator, denominator, factor).to_string()
    }))
}

pub(crate) fn mth_sign_case(request: &Value) -> Result<Value, String> {
    let value = math::mth_sign(f64_field(request, "x")?);
    Ok(json!({"value": value.to_string()}))
}

pub(crate) fn mth_square_f32_case(request: &Value) -> Result<Value, String> {
    let value = math::mth_square_f32(f32_field(request, "x")?);
    Ok(json!({"bits": format!("{:08x}", value.to_bits())}))
}

pub(crate) fn mth_cube_f32_case(request: &Value) -> Result<Value, String> {
    let value = math::mth_cube_f32(f32_field(request, "x")?);
    Ok(json!({"bits": format!("{:08x}", value.to_bits())}))
}

pub(crate) fn mth_square_f64_case(request: &Value) -> Result<Value, String> {
    let value = math::mth_square_f64(f64_field(request, "x")?);
    Ok(json!({"bits": format!("{:016x}", value.to_bits())}))
}

pub(crate) fn mth_square_i32_case(request: &Value) -> Result<Value, String> {
    let value = math::mth_square_i32(
        string(request, "x")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?,
    );
    Ok(json!({"value": value.to_string()}))
}

pub(crate) fn mth_square_i64_case(request: &Value) -> Result<Value, String> {
    let value = math::mth_square_i64(
        string(request, "x")?
            .parse::<i64>()
            .map_err(|error| error.to_string())?,
    );
    Ok(json!({"value": value.to_string()}))
}

pub(crate) fn scalar_case(request: &Value, operation: &str) -> Result<Value, String> {
    match string(request, "type")? {
        "f32" => {
            let value = f32_field(request, "value")?;
            match operation {
                "floor" => Ok(json!({"value": math::floor_f32(value).to_string()})),
                "wrap_degrees" => {
                    Ok(json!({"bits": format!("{:08x}", math::wrap_degrees_f32(value).to_bits())}))
                }
                "clamp" => Ok(json!({"bits": format!("{:08x}", math::clamp_f32(value,
                f32_field(request, "min")?, f32_field(request, "max")?).to_bits())})),
                _ => Err("unknown math operation".into()),
            }
        }
        "f64" => {
            let value = f64_field(request, "value")?;
            match operation {
                "floor" => Ok(json!({"value": math::floor_f64(value).to_string()})),
                "wrap_degrees" => {
                    Ok(json!({"bits": format!("{:016x}", math::wrap_degrees_f64(value).to_bits())}))
                }
                "clamp" => Ok(json!({"bits": format!("{:016x}", math::clamp_f64(value,
                f64_field(request, "min")?, f64_field(request, "max")?).to_bits())})),
                _ => Err("unknown floating-point operation".into()),
            }
        }
        _ => Err("unknown floating-point type".into()),
    }
}
