use super::*;

pub(super) fn string<'a>(request: &'a Value, name: &str) -> Result<&'a str, String> {
    request[name]
        .as_str()
        .ok_or_else(|| format!("{name} must be a string"))
}

pub(super) fn string_value(value: &Value, name: &str) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("{name} must be a string"))
}

pub(super) fn decode_hex(text: &str) -> Result<Vec<u8>, String> {
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

pub(super) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(super) fn hex_bytes(text: &str) -> Result<Vec<u8>, String> {
    (0..text.len() / 2)
        .map(|index| u8::from_str_radix(&text[index * 2..index * 2 + 2], 16))
        .collect::<Result<_, _>>()
        .map_err(|error| format!("hex: {error}"))
}

pub(super) fn read(input: &mut &[u8], wide: bool) -> Result<i64, ReadError> {
    if wide {
        wire::read_varlong(input)
    } else {
        wire::read_varint(input).map(i64::from)
    }
}

/// The wire operations that have a function of their own, by name.
type WireOperation = fn(&Value) -> Result<Value, String>;
const WIRE_OPERATIONS: &[(&str, WireOperation)] = &[
    ("frame_prepend", frame_prepend_result),
    ("frame_decode", frame_decode_result),
    ("handshake_intention", handshake_intention_result),
    ("login_hello", login_hello_result),
    (
        "crypto_server_hash",
        crate::wire_crypto::crypto_server_hash_result,
    ),
    ("crypto_stream", crate::wire_crypto::crypto_stream_result),
    ("login_key", crate::wire_crypto::login_key_result),
    ("login_compression", login_compression_result),
    ("login_server_hello", login_server_hello_result),
    ("login_acknowledged", login_acknowledged_result),
    ("login_finished", login_finished_result),
    ("configuration_unit", configuration_unit_result),
    ("configuration_misc", configuration_misc_result),
    ("protocol_metadata", protocol_metadata_result),
    ("client_information", client_information_result),
    ("common_packet", common_packet_result),
    ("enabled_features", enabled_features_result),
    ("award_stats", award_stats_result),
    ("update_advancements", update_advancements_result),
    ("select_advancements_tab", select_advancements_tab_result),
    ("update_tags", update_tags_result),
    ("registry_data", registry_data_result),
    ("status_request", status_request_result),
    ("ping", ping_result),
];

pub(super) fn wire_result(request: &Value) -> Result<Value, String> {
    if let Some((_, operation)) = WIRE_OPERATIONS
        .iter()
        .find(|(name, _)| request["operation"].as_str() == Some(*name))
    {
        return operation(request);
    }
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

pub(super) fn frame_error(error: FrameError) -> &'static str {
    match error {
        FrameError::LengthTooWide | FrameError::ZeroLength => "decoder_error",
        FrameError::PacketTooLarge => "encoder_error",
    }
}

pub(super) fn frame_prepend_result(request: &Value) -> Result<Value, String> {
    let length = request["body_length"]
        .as_u64()
        .ok_or("body_length must be an unsigned integer")? as usize;
    let skip = request["reader_skip"]
        .as_u64()
        .ok_or("reader_skip must be an unsigned integer")? as usize;
    let mut message = vec![0_u8; skip.saturating_add(length)];
    for (index, byte) in message[skip..].iter_mut().enumerate() {
        *byte = index as u8;
    }
    let body = &message[skip..];
    match frame::prepend(body) {
        Ok(framed) => Ok(json!({
            "outcome": "value",
            "written": framed.len(),
            "hex": hex(&framed),
        })),
        Err(error) => Ok(json!({
            "outcome": "error",
            "error": frame_error(error),
            "written": 0,
            "hex": "",
        })),
    }
}

pub(super) fn frame_decode_result(request: &Value) -> Result<Value, String> {
    let chunks = request["chunks"]
        .as_array()
        .ok_or("chunks must be an array")?
        .iter()
        .map(|chunk| {
            chunk
                .as_str()
                .ok_or_else(|| "chunk entries must be strings".to_owned())
                .and_then(decode_hex)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut decoder = Varint21FrameDecoder::new();
    let mut steps = Vec::with_capacity(chunks.len());
    for chunk in chunks {
        let mut step = serde_json::Map::new();
        let mut frames = Vec::new();
        let failed = match decoder.feed(&chunk) {
            Ok(Some(frame)) => {
                frames.push(hex(&frame));
                false
            }
            Ok(None) => false,
            Err(error) => {
                step.insert("outcome".into(), json!("error"));
                step.insert("error".into(), json!(frame_error(error)));
                true
            }
        };
        step.insert("frames".into(), json!(frames));
        if !failed {
            step.insert("outcome".into(), json!("value"));
        }
        step.insert("consumed".into(), json!(decoder.consumed()));
        step.insert("remaining_hex".into(), json!(hex(decoder.remaining())));
        steps.push(Value::Object(step));
        if failed {
            break;
        }
    }
    Ok(json!({"steps": steps}))
}

pub(super) fn handshake_read_error(error: handshake::ReadError) -> &'static str {
    match error {
        handshake::ReadError::BufferUnderflow => "buffer_underflow",
        handshake::ReadError::DecoderError => "decoder_error",
        handshake::ReadError::TooBig => "too_big",
    }
}

pub(super) fn handshake_intent(value: &str) -> Result<ClientIntent, String> {
    match value {
        "status" => Ok(ClientIntent::Status),
        "login" => Ok(ClientIntent::Login),
        "transfer" => Ok(ClientIntent::Transfer),
        _ => Err("unknown client intent".into()),
    }
}

pub(super) fn utf16_hex_units(value: &str, name: &str) -> Result<Vec<u16>, String> {
    if !value.is_ascii() || !value.len().is_multiple_of(4) {
        return Err(format!(
            "{name} must contain complete ASCII unit quadruples"
        ));
    }
    (0..value.len())
        .step_by(4)
        .map(|index| {
            u16::from_str_radix(&value[index..index + 4], 16)
                .map_err(|_| format!("invalid UTF-16 unit in {name}"))
        })
        .collect()
}

pub(super) fn handshake_intention_result(request: &Value) -> Result<Value, String> {
    match string(request, "direction")? {
        "write" => {
            let protocol_version = request["protocol_version"]
                .as_i64()
                .ok_or("protocol_version must be an integer")?
                .try_into()
                .map_err(|_| "protocol_version must fit in i32")?;
            let host_name = utf16_hex_units(string(request, "host_utf16_hex")?, "host_utf16_hex")?;
            let port = request["port"]
                .as_i64()
                .ok_or("port must be an integer")?
                .try_into()
                .map_err(|_| "port must fit in i32")?;
            let intention = handshake_intent(string(request, "intention")?)?;
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let packet = ClientIntentionPacket::new(protocol_version, host_name, port, intention);
            let mut buffer = prefix.clone();
            let mut result = match packet.encode(&mut buffer) {
                Ok(()) => json!({"outcome": "value"}),
                Err(handshake::WriteError::EncoderError) => {
                    json!({"outcome": "error", "error": "encoder_error"})
                }
            };
            result["written"] = json!(buffer.len() - prefix.len());
            result["hex"] = json!(hex(&buffer));
            Ok(result)
        }
        "read" => {
            let bytes = decode_hex(string(request, "hex")?)?;
            let mut remaining = bytes.as_slice();
            let mut result = match ClientIntentionPacket::decode(&mut remaining) {
                Ok(packet) => json!({
                    "protocol_version": packet.protocol_version,
                    "host_utf16_hex": packet.host_name.iter()
                        .map(|unit| format!("{unit:04x}"))
                        .collect::<String>(),
                    "port": packet.port,
                    "intention": packet.intention.name(),
                    "intention_id": packet.intention.id(),
                    "outcome": "value",
                }),
                Err(error) => json!({
                    "outcome": "error",
                    "error": handshake_read_error(error),
                }),
            };
            result["consumed"] = json!(bytes.len() - remaining.len());
            result["remaining_hex"] = json!(hex(remaining));
            Ok(result)
        }
        _ => Err("unknown handshake direction".into()),
    }
}

pub(super) fn login_read_error(error: photon_net::login::ReadError) -> &'static str {
    match error {
        photon_net::login::ReadError::BufferUnderflow => "buffer_underflow",
        photon_net::login::ReadError::DecoderError => "decoder_error",
        photon_net::login::ReadError::TooBig => "too_big",
    }
}

pub(super) fn uuid_hex_field(request: &Value, name: &str) -> Result<u64, String> {
    let value = string(request, name)?;
    if value.len() != 16 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be sixteen hexadecimal digits"));
    }
    u64::from_str_radix(value, 16).map_err(|_| format!("invalid hexadecimal {name}"))
}

pub(super) fn login_hello_result(request: &Value) -> Result<Value, String> {
    match string(request, "direction")? {
        "write" => {
            let name = utf16_hex_units(string(request, "name_utf16_hex")?, "name_utf16_hex")?;
            let profile_id = photon_core::binary::Uuid::new(
                uuid_hex_field(request, "profile_most_hex")?,
                uuid_hex_field(request, "profile_least_hex")?,
            );
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let packet = ServerboundHelloPacket::new(name, profile_id);
            let mut buffer = prefix.clone();
            let mut result = match packet.encode(&mut buffer) {
                Ok(()) => json!({"outcome": "value"}),
                Err(photon_net::login::WriteError::EncoderError) => {
                    json!({"outcome": "error", "error": "encoder_error"})
                }
            };
            result["written"] = json!(buffer.len() - prefix.len());
            result["hex"] = json!(hex(&buffer));
            Ok(result)
        }
        "read" => {
            let bytes = decode_hex(string(request, "hex")?)?;
            let mut remaining = bytes.as_slice();
            let mut result = match ServerboundHelloPacket::decode(&mut remaining) {
                Ok(packet) => json!({
                    "name_utf16_hex": packet.name.iter()
                        .map(|unit| format!("{unit:04x}"))
                        .collect::<String>(),
                    "profile_most_hex": format!("{:016x}", packet.profile_id.most),
                    "profile_least_hex": format!("{:016x}", packet.profile_id.least),
                    "outcome": "value",
                }),
                Err(error) => json!({
                    "outcome": "error",
                    "error": login_read_error(error),
                }),
            };
            result["consumed"] = json!(bytes.len() - remaining.len());
            result["remaining_hex"] = json!(hex(remaining));
            Ok(result)
        }
        _ => Err("unknown login direction".into()),
    }
}

pub(super) fn compression_read_error(error: wire::ReadError) -> &'static str {
    match error {
        wire::ReadError::BufferUnderflow => "buffer_underflow",
        wire::ReadError::TooBig => "too_big",
    }
}

pub(super) fn login_compression_result(request: &Value) -> Result<Value, String> {
    match string(request, "direction")? {
        "write" => {
            let threshold = request["threshold"]
                .as_i64()
                .ok_or("threshold must be an integer")?
                .try_into()
                .map_err(|_| "threshold must fit in i32")?;
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let packet = ClientboundLoginCompressionPacket::new(threshold);
            let mut buffer = prefix.clone();
            packet.encode(&mut buffer);
            Ok(json!({
                "outcome": "value",
                "written": buffer.len() - prefix.len(),
                "hex": hex(&buffer),
            }))
        }
        "read" => {
            let bytes = decode_hex(string(request, "hex")?)?;
            let mut remaining = bytes.as_slice();
            let mut result = match ClientboundLoginCompressionPacket::decode(&mut remaining) {
                Ok(packet) => json!({
                    "threshold": packet.threshold,
                    "outcome": "value",
                }),
                Err(error) => json!({
                    "outcome": "error",
                    "error": compression_read_error(error),
                }),
            };
            result["consumed"] = json!(bytes.len() - remaining.len());
            result["remaining_hex"] = json!(hex(remaining));
            Ok(result)
        }
        _ => Err("unknown compression direction".into()),
    }
}

/// `AwardStats` on bytes: each statistic as its type id, value id and number, in id order.
fn award_stats_result(request: &Value) -> Result<Value, String> {
    use photon_protocol::play::{AwardStats, Decode};
    let bytes = decode_hex(string(request, "hex")?)?;
    Ok(
        match AwardStats::decode(
            &bytes,
            &photon_data::version_registries::VersionRegistries::newest(),
        ) {
            Ok(AwardStats(stats)) => {
                let mut last = std::collections::HashMap::new();
                for (key, amount) in stats {
                    last.insert((key.stat_type, key.value), amount);
                }
                let mut rows: Vec<_> = last.into_iter().collect();
                rows.sort_unstable();
                let stats: Vec<Value> = rows
                    .into_iter()
                    .map(|((stat_type, value), amount)| json!([stat_type, value, amount]))
                    .collect();
                json!({"outcome": "value", "stats": stats})
            }
            Err(_) => json!({"outcome": "error"}),
        },
    )
}

/// `UpdateAdvancements` on bytes, as the game's codec is read: see `update_advancements` of the
/// oracle for the fields.
fn update_advancements_result(request: &Value) -> Result<Value, String> {
    use photon_protocol::play::{Decode, UpdateAdvancements};
    let bytes = decode_hex(string(request, "hex")?)?;
    let lang = photon_game::text::Lang::new(std::iter::empty());
    let Ok(packet) = UpdateAdvancements::decode(
        &bytes,
        &photon_data::version_registries::VersionRegistries::newest(),
    ) else {
        return Ok(json!({"outcome": "error"}));
    };
    let added: Vec<Value> = packet
        .added
        .iter()
        .map(|advancement| {
            let display = advancement.display.as_ref().map(|display| {
                json!({
                    "title": lang.plain(&display.title),
                    "description": lang.plain(&display.description),
                    "icon": display.icon.0.0,
                    "icon_count": display.icon.1,
                    "icon_patched": display.icon.2.added.len() + display.icon.2.removed.len(),
                    "frame": display.frame as i32,
                    "show_toast": display.show_toast,
                    "hidden": display.hidden,
                    "background": display.background,
                    "x": format!("{:08x}", display.x.to_bits()),
                    "y": format!("{:08x}", display.y.to_bits()),
                })
            });
            json!({
                "id": advancement.id,
                "parent": advancement.parent,
                "requirements": advancement.requirements,
                "telemetry": advancement.sends_telemetry_event,
                "display": display,
            })
        })
        .collect();
    let mut progress: Vec<_> = packet.progress.iter().collect();
    progress.sort_by(|a, b| a.0.cmp(&b.0));
    let progress: Vec<Value> = progress
        .into_iter()
        .map(|(id, criteria)| {
            let mut criteria: Vec<_> = criteria.iter().collect();
            criteria.sort_by(|a, b| a.0.cmp(&b.0));
            json!({
                "id": id,
                "criteria": criteria.into_iter().map(|(name, obtained)| json!([name, obtained])).collect::<Vec<_>>(),
            })
        })
        .collect();
    Ok(without_nulls(json!({
        "outcome": "value",
        "reset": packet.reset,
        "show": packet.show_advancements,
        "added": added,
        "removed": packet.removed,
        "progress": progress,
    })))
}

/// `SelectAdvancementsTab` on bytes.
fn select_advancements_tab_result(request: &Value) -> Result<Value, String> {
    use photon_protocol::play::{Decode, SelectAdvancementsTab};
    let bytes = decode_hex(string(request, "hex")?)?;
    Ok(
        match SelectAdvancementsTab::decode(
            &bytes,
            &photon_data::version_registries::VersionRegistries::newest(),
        ) {
            Ok(SelectAdvancementsTab(tab)) => {
                without_nulls(json!({"outcome": "value", "tab": tab}))
            }
            Err(_) => json!({"outcome": "error"}),
        },
    )
}

/// The oracle leaves out what is null (Gson drops a null member), so a member that is not there is
/// the same answer.
pub(crate) fn without_nulls(value: Value) -> Value {
    match value {
        Value::Object(members) => Value::Object(
            members
                .into_iter()
                .filter(|(_, member)| !member.is_null())
                .map(|(key, member)| (key, without_nulls(member)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.into_iter().map(without_nulls).collect()),
        other => other,
    }
}
