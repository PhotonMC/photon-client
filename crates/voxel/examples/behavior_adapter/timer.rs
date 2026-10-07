use super::*;

pub(super) fn timer_snapshot(timer: &Timer) -> Value {
    json!({
        "game_delta": format!("{:08x}", timer.game_time_delta_ticks().to_bits()),
        "partial": format!("{:08x}", timer.game_time_delta_partial_tick(false).to_bits()),
        "partial_ignore_frozen": format!("{:08x}", timer.game_time_delta_partial_tick(true).to_bits()),
        "real_delta": format!("{:08x}", timer.realtime_delta_ticks().to_bits()),
    })
}

pub(super) fn timer_result(request: &Value) -> Result<Value, String> {
    if string(request, "operation")? != "trace" {
        return Err("unknown timer operation".into());
    }
    let mut timer = Timer::new(f32_field(request, "tps")?, millis(request, "start_ms")?);
    let mut target = f32_field(request, "target_mspt")?;
    let mut events = vec![timer_snapshot(&timer)];
    let steps = request["steps"]
        .as_array()
        .ok_or("steps must be an array")?;
    for step in steps {
        let ticks = match string(step, "action")? {
            "game" => Some(timer.advance_game_time(millis(step, "ms")?, target)),
            "real" => {
                timer.advance_real_time(millis(step, "ms")?);
                None
            }
            "pause" | "freeze" => {
                let enabled = step["value"].as_bool().ok_or("value must be a boolean")?;
                if step["action"] == "pause" {
                    timer.update_pause_state(enabled);
                } else {
                    timer.update_frozen_state(enabled);
                }
                None
            }
            "target" => {
                target = f32_field(step, "value")?;
                None
            }
            _ => return Err("unknown timer action".into()),
        };
        let mut state = timer_snapshot(&timer);
        if let Some(ticks) = ticks {
            state["ticks"] = json!(ticks);
        }
        events.push(state);
    }
    Ok(json!({"events": events}))
}

pub(super) fn utf8_result(request: &Value) -> Result<Value, String> {
    let max_length = request["max_length"]
        .as_u64()
        .ok_or("max_length must be an unsigned integer")? as usize;
    let prefix = decode_hex(string(request, "prefix_hex")?)?;
    match string(request, "operation")? {
        "read" => {
            let framed = decode_hex(string(request, "hex")?)?;
            let mut remaining = framed.as_slice();
            match utf8::read_utf8(&mut remaining, max_length) {
                Ok(units) => Ok(json!({
                    "utf16_hex": units.iter().map(|unit| format!("{unit:04x}")).collect::<String>(),
                    "utf16_length": units.len(),
                    "outcome": "value",
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
                Err(error) => Ok(json!({
                    "outcome": "error",
                    "error": match error {
                        utf8::ReadError::DecoderError => "decoder_error",
                        utf8::ReadError::BufferUnderflow => "buffer_underflow",
                        utf8::ReadError::TooBig => "too_big",
                    },
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
            }
        }
        "write" => {
            let text = string(request, "utf16_hex")?;
            if !text.is_ascii() || !text.len().is_multiple_of(4) {
                return Err("utf16_hex must contain complete ASCII unit quadruples".into());
            }
            let mut units = Vec::with_capacity(text.len() / 4);
            for index in (0..text.len()).step_by(4) {
                let unit = u16::from_str_radix(&text[index..index + 4], 16)
                    .map_err(|_| "invalid UTF-16 unit".to_owned())?;
                units.push(unit);
            }
            let mut buffer = prefix.clone();
            match utf8::write_utf8(&mut buffer, &units, max_length) {
                Ok(()) => Ok(json!({
                    "outcome": "value",
                    "written": buffer.len() - prefix.len(),
                    "hex": hex(&buffer),
                })),
                Err(utf8::WriteError::EncoderError) => Ok(json!({
                    "outcome": "error",
                    "error": "encoder_error",
                    "written": buffer.len() - prefix.len(),
                    "hex": hex(&buffer),
                })),
            }
        }
        _ => Err("unknown UTF-8 operation".into()),
    }
}

pub(super) fn binary_result(request: &Value) -> Result<Value, String> {
    fn hex_u64(text: &str) -> Result<u64, String> {
        if text.len() != 16 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err("UUID half must be sixteen hexadecimal digits".into());
        }
        u64::from_str_radix(text, 16).map_err(|error| error.to_string())
    }

    fn coordinate(value: &Value, name: &str) -> Result<i32, String> {
        value[name]
            .as_str()
            .ok_or_else(|| format!("{name} must be a string"))?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    }

    fn bit_indices(value: &Value) -> Result<Vec<u64>, String> {
        let bits = value["bits"].as_array().ok_or("bits must be an array")?;
        let mut indices = Vec::with_capacity(bits.len());
        for entry in bits {
            indices.push(
                entry
                    .as_u64()
                    .ok_or("bit index must be an unsigned integer")?,
            );
        }
        Ok(indices)
    }

    fn bit_size(value: &Value) -> Result<usize, String> {
        value["size"]
            .as_u64()
            .ok_or("size must be an unsigned integer".to_owned())
            .map(|size| size as usize)
    }

    let prefix = decode_hex(string(request, "prefix_hex")?)?;
    let codec = string(request, "codec")?;
    fn binary_error(error: binary::ReadError) -> &'static str {
        match error {
            binary::ReadError::BufferUnderflow => "buffer_underflow",
            binary::ReadError::DecoderError => "decoder_error",
            binary::ReadError::TooBig => "too_big",
        }
    }
    match string(request, "operation")? {
        "read" => {
            let framed = decode_hex(string(request, "hex")?)?;
            let mut remaining = framed.as_slice();
            match codec {
                "uuid" => match binary::read_uuid(&mut remaining) {
                    Ok(value) => Ok(json!({
                        "most_hex": format!("{:016x}", value.most),
                        "least_hex": format!("{:016x}", value.least),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "blockpos" => match binary::read_blockpos(&mut remaining) {
                    Ok(value) => Ok(json!({
                        "x": value.x.to_string(),
                        "y": value.y.to_string(),
                        "z": value.z.to_string(),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "sectionpos" => match binary::read_sectionpos(&mut remaining) {
                    Ok(value) => Ok(json!({
                        "x": value.x.to_string(),
                        "y": value.y.to_string(),
                        "z": value.z.to_string(),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "chunkpos" => match binary::read_chunkpos(&mut remaining) {
                    Ok(value) => Ok(json!({
                        "x": value.x.to_string(),
                        "z": value.z.to_string(),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "vec3" => match binary::read_vec3(&mut remaining) {
                    Ok(value) => Ok(json!({
                        "x": format!("{:016x}", value.x.to_bits()),
                        "y": format!("{:016x}", value.y.to_bits()),
                        "z": format!("{:016x}", value.z.to_bits()),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "lp_vec3" => match binary::read_lp_vec3(&mut remaining) {
                    Ok(value) => Ok(json!({
                        "x": format!("{:016x}", value.x.to_bits()),
                        "y": format!("{:016x}", value.y.to_bits()),
                        "z": format!("{:016x}", value.z.to_bits()),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "longarray" => match binary::read_long_array(&mut remaining) {
                    Ok(values) => Ok(json!({
                        "values": values.iter().map(ToString::to_string).collect::<Vec<_>>(),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "bitset" => match binary::read_bitset(&mut remaining) {
                    Ok(value) => Ok(json!({
                        "bits": value.to_bits(),
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                    Err(error) => Ok(json!({
                        "outcome": "error",
                        "error": binary_error(error),
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
                "fixedbitset" => {
                    let size = bit_size(request)?;
                    match binary::read_fixed_bitset(&mut remaining, size) {
                        Ok(value) => Ok(json!({
                            "bits": value.to_bits(),
                            "outcome": "value",
                            "consumed": framed.len() - remaining.len(),
                            "remaining_hex": hex(remaining),
                        })),
                        Err(error) => Ok(json!({
                            "outcome": "error",
                            "error": binary_error(error),
                            "consumed": framed.len() - remaining.len(),
                            "remaining_hex": hex(remaining),
                        })),
                    }
                }
                _ => Err("unknown binary codec".into()),
            }
        }
        "write" => {
            let mut buffer = prefix.clone();
            match codec {
                "uuid" => {
                    let value = Uuid::new(
                        hex_u64(string(request, "most_hex")?)?,
                        hex_u64(string(request, "least_hex")?)?,
                    );
                    binary::write_uuid(value, &mut buffer);
                }
                "blockpos" => {
                    let value = BlockPos::new(
                        coordinate(request, "x")?,
                        coordinate(request, "y")?,
                        coordinate(request, "z")?,
                    );
                    binary::write_blockpos(value, &mut buffer);
                }
                "sectionpos" => {
                    let value = SectionPos::new(
                        coordinate(request, "x")?,
                        coordinate(request, "y")?,
                        coordinate(request, "z")?,
                    );
                    binary::write_sectionpos(value, &mut buffer);
                }
                "chunkpos" => {
                    let value = ChunkPos::new(coordinate(request, "x")?, coordinate(request, "z")?);
                    binary::write_chunkpos(value, &mut buffer);
                }
                "vec3" => {
                    let value = Vec3::new(
                        f64_field(request, "x")?,
                        f64_field(request, "y")?,
                        f64_field(request, "z")?,
                    );
                    binary::write_vec3(value, &mut buffer);
                }
                "lp_vec3" => {
                    let value = Vec3::new(
                        f64_field(request, "x")?,
                        f64_field(request, "y")?,
                        f64_field(request, "z")?,
                    );
                    binary::write_lp_vec3(value, &mut buffer);
                }
                "longarray" => {
                    let values = request["values"]
                        .as_array()
                        .ok_or("values must be an array")?;
                    let mut longs = Vec::with_capacity(values.len());
                    for entry in values {
                        longs.push(
                            entry
                                .as_str()
                                .ok_or("long value must be a string")?
                                .parse::<i64>()
                                .map_err(|error| error.to_string())?,
                        );
                    }
                    binary::write_long_array(&longs, &mut buffer);
                }
                "bitset" => {
                    let indices = bit_indices(request)?;
                    binary::write_bitset(&binary::BitSet::from_bits(&indices), &mut buffer);
                }
                "fixedbitset" => {
                    let size = bit_size(request)?;
                    let indices = bit_indices(request)?;
                    match binary::write_fixed_bitset(
                        &binary::BitSet::from_bits(&indices),
                        size,
                        &mut buffer,
                    ) {
                        Ok(()) => {}
                        Err(binary::WriteError::EncoderError) => {
                            return Ok(json!({
                                "outcome": "error",
                                "error": "encoder_error",
                                "written": buffer.len() - prefix.len(),
                                "hex": hex(&buffer),
                            }));
                        }
                    }
                }
                _ => return Err("unknown binary codec".into()),
            }
            Ok(json!({
                "outcome": "value",
                "written": buffer.len() - prefix.len(),
                "hex": hex(&buffer),
            }))
        }
        _ => Err("unknown binary operation".into()),
    }
}

pub(super) fn nbt_type_name(value: &FieldValue) -> &'static str {
    match value {
        FieldValue::Byte(_) => "byte",
        FieldValue::Short(_) => "short",
        FieldValue::Int(_) => "int",
        FieldValue::Long(_) => "long",
        FieldValue::Float(_) => "float",
        FieldValue::Double(_) => "double",
        FieldValue::ByteArray(_) => "byte-array",
        FieldValue::Str(_) => "string",
        FieldValue::List(_) => "list",
        FieldValue::Compound(_) => "compound",
        FieldValue::IntArray(_) => "int-array",
        FieldValue::LongArray(_) => "long-array",
    }
}
