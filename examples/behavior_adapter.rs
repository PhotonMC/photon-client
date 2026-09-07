//! JSONL test glue. Behavior is delegated to the production library.

use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use voxel_client::binary::{self, BlockPos, Uuid};
use voxel_client::math::{self, Vec2};
use voxel_client::timer::Timer;
use voxel_client::utf8;
use voxel_client::wire::{self, ReadError};

fn string<'a>(request: &'a Value, name: &str) -> Result<&'a str, String> {
    request[name]
        .as_str()
        .ok_or_else(|| format!("{name} must be a string"))
}

fn decode_hex(text: &str) -> Result<Vec<u8>, String> {
    if !text.is_ascii() || !text.len().is_multiple_of(2) {
        return Err("hex must contain complete ASCII byte pairs".into());
    }
    (0..text.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&text[index..index + 2], 16)
                .map_err(|_| "invalid hex byte".to_owned())
        })
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn read(input: &mut &[u8], wide: bool) -> Result<i64, ReadError> {
    if wide {
        wire::read_varlong(input)
    } else {
        wire::read_varint(input).map(i64::from)
    }
}

fn wire_result(request: &Value) -> Result<Value, String> {
    let wide = match string(request, "codec")? {
        "varint" => false,
        "varlong" => true,
        _ => return Err("unknown codec".into()),
    };
    let mut bytes = Vec::new();
    let mut result;
    match string(request, "operation")? {
        "write" => {
            let text = string(request, "value")?;
            let size = if wide {
                let value = text.parse::<i64>().map_err(|error| error.to_string())?;
                wire::write_varlong(value, &mut bytes);
                wire::varlong_size(value)
            } else {
                let value = text.parse::<i32>().map_err(|error| error.to_string())?;
                wire::write_varint(value, &mut bytes);
                wire::varint_size(value)
            };
            let mut remaining = bytes.as_slice();
            let value = read(&mut remaining, wide)
                .map_err(|error| format!("cannot decode library output: {error:?}"))?;
            result = json!({"outcome": "value", "size": size, "hex": hex(&bytes),
                "decoded": value.to_string(), "consumed": bytes.len() - remaining.len(),
                "remaining_hex": hex(remaining)});
        }
        "read" => {
            bytes = decode_hex(string(request, "hex")?)?;
            let mut remaining = bytes.as_slice();
            result = match read(&mut remaining, wide) {
                Ok(value) => json!({"outcome": "value", "value": value.to_string()}),
                Err(error) => json!({"outcome": "error", "error": match error {
                    ReadError::BufferUnderflow => "buffer_underflow",
                    ReadError::TooBig => "too_big",
                }}),
            };
            result["consumed"] = json!(bytes.len() - remaining.len());
            result["remaining_hex"] = json!(hex(remaining));
        }
        _ => return Err("unknown wire operation".into()),
    }
    Ok(result)
}

fn f32_field(value: &Value, name: &str) -> Result<f32, String> {
    let text = string(value, name)?;
    if text.len() != 8 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be eight hexadecimal digits"));
    }
    u32::from_str_radix(text, 16)
        .map(f32::from_bits)
        .map_err(|error| error.to_string())
}

fn millis(value: &Value, name: &str) -> Result<i64, String> {
    string(value, name)?
        .parse::<i64>()
        .map_err(|error| error.to_string())
}

fn f64_field(value: &Value, name: &str) -> Result<f64, String> {
    let text = string(value, name)?;
    if text.len() != 16 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be sixteen hexadecimal digits"));
    }
    u64::from_str_radix(text, 16)
        .map(f64::from_bits)
        .map_err(|error| error.to_string())
}

fn vec2_snapshot(vector: Vec2) -> Value {
    json!({
        "x": format!("{:08x}", vector.x.to_bits()),
        "y": format!("{:08x}", vector.y.to_bits()),
        "length": format!("{:08x}", vector.length().to_bits()),
        "length_squared": format!("{:08x}", vector.length_squared().to_bits()),
    })
}

fn math_result(request: &Value) -> Result<Value, String> {
    let operation = string(request, "operation")?;
    if operation == "vec2" {
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
        return Ok(json!({"events": events}));
    }
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

fn timer_snapshot(timer: &Timer) -> Value {
    json!({
        "game_delta": format!("{:08x}", timer.game_time_delta_ticks().to_bits()),
        "partial": format!("{:08x}", timer.game_time_delta_partial_tick(false).to_bits()),
        "partial_ignore_frozen": format!("{:08x}", timer.game_time_delta_partial_tick(true).to_bits()),
        "real_delta": format!("{:08x}", timer.realtime_delta_ticks().to_bits()),
    })
}

fn timer_result(request: &Value) -> Result<Value, String> {
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

fn utf8_result(request: &Value) -> Result<Value, String> {
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

fn binary_result(request: &Value) -> Result<Value, String> {
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

    let prefix = decode_hex(string(request, "prefix_hex")?)?;
    let codec = string(request, "codec")?;
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
                    Err(binary::ReadError::BufferUnderflow) => Ok(json!({
                        "outcome": "error",
                        "error": "buffer_underflow",
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
                    Err(binary::ReadError::BufferUnderflow) => Ok(json!({
                        "outcome": "error",
                        "error": "buffer_underflow",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    })),
                },
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

fn respond(request: Value) -> Result<Value, String> {
    if request["schema_version"].as_u64() != Some(1) {
        return Err("unsupported schema_version".into());
    }
    let case_id = string(&request, "case_id")?;
    let result = match string(&request, "family")? {
        "wire" => wire_result(&request)?,
        "timer" => timer_result(&request)?,
        "math" => math_result(&request)?,
        "utf8" => utf8_result(&request)?,
        "binary" => binary_result(&request)?,
        _ => return Err("unknown behavior family".into()),
    };
    Ok(json!({"schema_version": 1, "case_id": case_id, "result": result}))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = io::stdin().lock();
    let mut output = io::BufWriter::new(io::stdout().lock());
    for (index, line) in input.lines().enumerate() {
        let request = serde_json::from_str(&line?)?;
        let response = respond(request)
            .map_err(|message| io::Error::other(format!("line {}: {message}", index + 1)))?;
        serde_json::to_writer(&mut output, &response)?;
        writeln!(output)?;
    }
    output.flush()?;
    Ok(())
}
