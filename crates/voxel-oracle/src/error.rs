use super::*;

pub(super) fn client_information_result(request: &Value) -> Result<Value, String> {
    match string(request, "direction")? {
        "write" => {
            let language =
                utf16_hex_units(string(request, "language_utf16_hex")?, "language_utf16_hex")?;
            let view_distance = request["view_distance"]
                .as_i64()
                .ok_or("view_distance must be an integer")?
                .try_into()
                .map_err(|_| "view_distance must fit in i32")?;
            let chat_visibility =
                client_information_chat_visibility(string(request, "chat_visibility")?)?;
            let chat_colors = request["chat_colors"]
                .as_bool()
                .ok_or("chat_colors must be a boolean")?;
            let model_customisation = request["model_customisation"]
                .as_i64()
                .ok_or("model_customisation must be an integer")?
                .try_into()
                .map_err(|_| "model_customisation must fit in i32")?;
            let main_hand = client_information_main_hand(string(request, "main_hand")?)?;
            let text_filtering_enabled = request["text_filtering_enabled"]
                .as_bool()
                .ok_or("text_filtering_enabled must be a boolean")?;
            let allows_listing = request["allows_listing"]
                .as_bool()
                .ok_or("allows_listing must be a boolean")?;
            let particle_status =
                client_information_particle_status(string(request, "particle_status")?)?;
            let packet = ServerboundClientInformationPacket::new(ClientInformation::new(
                language,
                view_distance,
                chat_visibility,
                chat_colors,
                model_customisation,
                main_hand,
                text_filtering_enabled,
                allows_listing,
                particle_status,
            ));
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let mut buffer = prefix.clone();
            let mut result = match packet.encode(&mut buffer) {
                Ok(()) => json!({"outcome": "value"}),
                Err(voxel_net::configuration::WriteError::EncoderError) => {
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
            let mut result = match ServerboundClientInformationPacket::decode(&mut remaining) {
                Ok(packet) => {
                    let information = packet.information;
                    json!({
                        "language_utf16_hex": information.language.iter()
                            .map(|unit| format!("{unit:04x}"))
                            .collect::<String>(),
                        "view_distance": information.view_distance,
                        "chat_visibility": client_information_chat_visibility_name(information.chat_visibility),
                        "chat_colors": information.chat_colors,
                        "model_customisation": information.model_customisation,
                        "main_hand": client_information_main_hand_name(information.main_hand),
                        "text_filtering_enabled": information.text_filtering_enabled,
                        "allows_listing": information.allows_listing,
                        "particle_status": client_information_particle_status_name(information.particle_status),
                        "outcome": "value",
                    })
                }
                Err(error) => json!({
                    "outcome": "error",
                    "error": login_read_error(error),
                }),
            };
            result["consumed"] = json!(bytes.len() - remaining.len());
            result["remaining_hex"] = json!(hex(remaining));
            Ok(result)
        }
        _ => Err("unknown client information direction".into()),
    }
}

pub(super) fn common_read_error(error: voxel_net::common::ReadError) -> &'static str {
    match error {
        voxel_net::common::ReadError::BufferUnderflow => "buffer_underflow",
        voxel_net::common::ReadError::DecoderError => "decoder_error",
        voxel_net::common::ReadError::TooBig => "too_big",
    }
}

pub(super) fn common_packet_result(request: &Value) -> Result<Value, String> {
    let packet = string(request, "packet")?;
    match string(request, "direction")? {
        "write" => {
            let id = string(request, "id")?
                .parse::<i64>()
                .map_err(|error| error.to_string())?;
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let mut buffer = prefix.clone();
            match packet {
                "client-ping" => {
                    ClientboundPingPacket::new(id.try_into().map_err(|_| "id must fit in i32")?)
                        .encode(&mut buffer);
                }
                "server-pong" => {
                    ServerboundPongPacket::new(id.try_into().map_err(|_| "id must fit in i32")?)
                        .encode(&mut buffer);
                }
                "client-keepalive" => {
                    ClientboundKeepAlivePacket::new(id).encode(&mut buffer);
                }
                "server-keepalive" => {
                    ServerboundKeepAlivePacket::new(id).encode(&mut buffer);
                }
                _ => return Err("unknown common packet".into()),
            }
            Ok(json!({
                "outcome": "value",
                "written": buffer.len() - prefix.len(),
                "hex": hex(&buffer),
            }))
        }
        "read" => {
            let bytes = decode_hex(string(request, "hex")?)?;
            let mut remaining = bytes.as_slice();
            let result = match packet {
                "client-ping" => ClientboundPingPacket::decode(&mut remaining)
                    .map(|value| json!({"id": value.id.to_string(), "outcome": "value"})),
                "server-pong" => ServerboundPongPacket::decode(&mut remaining)
                    .map(|value| json!({"id": value.id.to_string(), "outcome": "value"})),
                "client-keepalive" => ClientboundKeepAlivePacket::decode(&mut remaining)
                    .map(|value| json!({"id": value.id.to_string(), "outcome": "value"})),
                "server-keepalive" => ServerboundKeepAlivePacket::decode(&mut remaining)
                    .map(|value| json!({"id": value.id.to_string(), "outcome": "value"})),
                _ => return Err("unknown common packet".into()),
            };
            let mut result = match result {
                Ok(value) => value,
                Err(error) => json!({
                    "outcome": "error",
                    "error": common_read_error(error),
                }),
            };
            result["consumed"] = json!(bytes.len() - remaining.len());
            result["remaining_hex"] = json!(hex(remaining));
            Ok(result)
        }
        _ => Err("unknown common packet direction".into()),
    }
}

pub(super) fn enabled_features_result(request: &Value) -> Result<Value, String> {
    match string(request, "direction")? {
        "write" => {
            let features = request["features"]
                .as_array()
                .ok_or("features must be an array")?
                .iter()
                .map(|value| {
                    Identifier::parse(value.as_str().ok_or("feature entries must be strings")?)
                        .map_err(|_| "invalid feature identifier".to_owned())
                })
                .collect::<Result<Vec<_>, String>>()?;
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let mut buffer = prefix.clone();
            let mut result =
                match ClientboundUpdateEnabledFeaturesPacket::new(features).encode(&mut buffer) {
                    Ok(()) => json!({"outcome": "value"}),
                    Err(voxel_net::configuration::WriteError::EncoderError) => {
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
            let mut result = match ClientboundUpdateEnabledFeaturesPacket::decode(&mut remaining) {
                Ok(packet) => json!({
                    "features": packet.features.iter()
                        .map(Identifier::as_string)
                        .collect::<Vec<_>>(),
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
        _ => Err("unknown enabled features direction".into()),
    }
}

pub(super) fn update_tags_result(request: &Value) -> Result<Value, String> {
    match string(request, "direction")? {
        "write" => {
            let registries = request["registries"]
                .as_array()
                .ok_or("registries must be an array")?
                .iter()
                .map(|registry| {
                    let registry = registry
                        .as_object()
                        .ok_or("registry entries must be objects")?;
                    let registry_id = Identifier::parse(
                        registry["registry"]
                            .as_str()
                            .ok_or("registry must be a string")?,
                    )
                    .map_err(|_| "invalid registry identifier".to_owned())?;
                    let tags = registry["tags"]
                        .as_array()
                        .ok_or("tags must be an array")?
                        .iter()
                        .map(|tag| {
                            let tag = tag.as_object().ok_or("tag entries must be objects")?;
                            let id = Identifier::parse(
                                tag["id"].as_str().ok_or("tag id must be a string")?,
                            )
                            .map_err(|_| "invalid tag identifier".to_owned())?;
                            let values = tag["values"]
                                .as_array()
                                .ok_or("tag values must be an array")?
                                .iter()
                                .map(|value| {
                                    value
                                        .as_i64()
                                        .ok_or("tag values must be integers")?
                                        .try_into()
                                        .map_err(|_| "tag value must fit in i32".to_owned())
                                })
                                .collect::<Result<Vec<i32>, String>>()?;
                            Ok(TagEntry::new(id, values))
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    Ok(RegistryTags::new(registry_id, tags))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let mut buffer = prefix.clone();
            let mut result = match ClientboundUpdateTagsPacket::new(registries).encode(&mut buffer)
            {
                Ok(()) => json!({"outcome": "value"}),
                Err(voxel_core::tags::WriteError::EncoderError) => {
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
            let mut result = match ClientboundUpdateTagsPacket::decode(&mut remaining) {
                Ok(packet) => json!({
                    "registries": packet.registries.iter().map(|registry| json!({
                        "registry": registry.registry.as_string(),
                        "tags": registry.tags.iter().map(|tag| json!({
                            "id": tag.id.as_string(),
                            "values": tag.values,
                        })).collect::<Vec<_>>(),
                    })).collect::<Vec<_>>(),
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
        _ => Err("unknown update tags direction".into()),
    }
}

pub(super) fn registry_data_read_error(error: voxel_net::registry_data::ReadError) -> &'static str {
    match error {
        voxel_net::registry_data::ReadError::BufferUnderflow => "buffer_underflow",
        voxel_net::registry_data::ReadError::DecoderError => "decoder_error",
        voxel_net::registry_data::ReadError::EncoderError => "encoder_error",
        voxel_net::registry_data::ReadError::TooBig => "too_big",
    }
}

pub(super) fn registry_data_result(request: &Value) -> Result<Value, String> {
    match string(request, "direction")? {
        "write" => {
            let registry = Identifier::parse(string(request, "registry")?)
                .map_err(|_| "invalid registry identifier".to_owned())?;
            let entries = request["entries"]
                .as_array()
                .ok_or("entries must be an array")?
                .iter()
                .map(|entry| {
                    let entry = entry
                        .as_object()
                        .ok_or("registry entries must be objects")?;
                    let id =
                        Identifier::parse(entry["id"].as_str().ok_or("entry id must be a string")?)
                            .map_err(|_| "invalid entry identifier".to_owned())?;
                    let data = match entry.get("data") {
                        None | Some(Value::Null) => None,
                        Some(tree) => Some(nbt_tree_value(tree)?),
                    };
                    Ok(PackedRegistryEntry::new(id, data))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let mut buffer = prefix.clone();
            let mut result =
                match ClientboundRegistryDataPacket::new(registry, entries).encode(&mut buffer) {
                    Ok(()) => json!({"outcome": "value"}),
                    Err(voxel_net::registry_data::WriteError::EncoderError) => {
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
            let mut result = match ClientboundRegistryDataPacket::decode(&mut remaining) {
                Ok(packet) => json!({
                    "registry": packet.registry.as_string(),
                    "entries": packet.entries.iter().map(|entry| {
                        let mut result = json!({"id": entry.id.as_string()});
                        if let Some(data) = &entry.data {
                            result["data"] = nbt_description_json(data);
                        }
                        result
                    }).collect::<Vec<_>>(),
                    "outcome": "value",
                }),
                Err(error) => json!({
                    "outcome": "error",
                    "error": registry_data_read_error(error),
                }),
            };
            result["consumed"] = json!(bytes.len() - remaining.len());
            result["remaining_hex"] = json!(hex(remaining));
            Ok(result)
        }
        _ => Err("unknown registry data direction".into()),
    }
}

pub(super) fn status_request_result(request: &Value) -> Result<Value, String> {
    match string(request, "direction")? {
        "write" => {
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let mut buffer = prefix.clone();
            ServerboundStatusRequestPacket::INSTANCE.encode(&mut buffer);
            Ok(json!({
                "outcome": "value",
                "written": buffer.len() - prefix.len(),
                "hex": hex(&buffer),
            }))
        }
        "read" => {
            let bytes = decode_hex(string(request, "hex")?)?;
            let mut remaining = bytes.as_slice();
            let packet = ServerboundStatusRequestPacket::decode(&mut remaining)
                .map_err(|error| format!("status request decode failed: {error:?}"))?;
            Ok(json!({
                "singleton": packet == ServerboundStatusRequestPacket::INSTANCE,
                "outcome": "value",
                "consumed": bytes.len() - remaining.len(),
                "remaining_hex": hex(remaining),
            }))
        }
        _ => Err("unknown status request direction".into()),
    }
}

pub(super) fn ping_read_error(error: voxel_net::status::ReadError) -> &'static str {
    match error {
        voxel_net::status::ReadError::BufferUnderflow => "buffer_underflow",
        voxel_net::status::ReadError::DecoderError => "decoder_error",
        voxel_net::status::ReadError::TooBig => "too_big",
    }
}

pub(super) fn ping_result(request: &Value) -> Result<Value, String> {
    let packet = string(request, "packet")?;
    match string(request, "direction")? {
        "write" => {
            let time = millis(request, "time")?;
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let mut buffer = prefix.clone();
            match packet {
                "request" => ServerboundPingRequestPacket::new(time).encode(&mut buffer),
                "response" => ClientboundPongResponsePacket::new(time).encode(&mut buffer),
                _ => return Err("unknown ping packet".into()),
            }
            Ok(json!({
                "outcome": "value",
                "written": buffer.len() - prefix.len(),
                "hex": hex(&buffer),
            }))
        }
        "read" => {
            let bytes = decode_hex(string(request, "hex")?)?;
            let mut remaining = bytes.as_slice();
            let result = match packet {
                "request" => ServerboundPingRequestPacket::decode(&mut remaining)
                    .map(|value| json!({"time": value.time.to_string(), "outcome": "value"})),
                "response" => ClientboundPongResponsePacket::decode(&mut remaining)
                    .map(|value| json!({"time": value.time.to_string(), "outcome": "value"})),
                _ => return Err("unknown ping packet".into()),
            };
            let mut result = match result {
                Ok(value) => value,
                Err(error) => json!({
                    "outcome": "error",
                    "error": ping_read_error(error),
                }),
            };
            result["consumed"] = json!(bytes.len() - remaining.len());
            result["remaining_hex"] = json!(hex(remaining));
            Ok(result)
        }
        _ => Err("unknown ping direction".into()),
    }
}

pub(super) fn f32_field(value: &Value, name: &str) -> Result<f32, String> {
    let text = string(value, name)?;
    if text.len() != 8 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be eight hexadecimal digits"));
    }
    u32::from_str_radix(text, 16)
        .map(f32::from_bits)
        .map_err(|error| error.to_string())
}
