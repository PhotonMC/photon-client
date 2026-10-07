use super::*;

pub(super) fn login_server_hello_result(request: &Value) -> Result<Value, String> {
    match string(request, "direction")? {
        "write" => {
            let server_id = utf16_hex_units(
                string(request, "server_id_utf16_hex")?,
                "server_id_utf16_hex",
            )?;
            let public_key = decode_hex(string(request, "public_key_hex")?)?;
            let challenge = decode_hex(string(request, "challenge_hex")?)?;
            let should_authenticate = request["should_authenticate"]
                .as_bool()
                .ok_or("should_authenticate must be a boolean")?;
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let packet =
                ClientboundHelloPacket::new(server_id, public_key, challenge, should_authenticate);
            let mut buffer = prefix.clone();
            let mut result = match packet.encode(&mut buffer) {
                Ok(()) => json!({"outcome": "value"}),
                Err(voxel_net::login::WriteError::EncoderError) => {
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
            let mut result = match ClientboundHelloPacket::decode(&mut remaining) {
                Ok(packet) => json!({
                    "server_id_utf16_hex": packet.server_id.iter()
                        .map(|unit| format!("{unit:04x}"))
                        .collect::<String>(),
                    "public_key_hex": hex(&packet.public_key),
                    "challenge_hex": hex(&packet.challenge),
                    "should_authenticate": packet.should_authenticate,
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
        _ => Err("unknown server hello direction".into()),
    }
}

pub(super) fn login_acknowledged_result(request: &Value) -> Result<Value, String> {
    match string(request, "direction")? {
        "write" => {
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let mut buffer = prefix.clone();
            ServerboundLoginAcknowledgedPacket::INSTANCE.encode(&mut buffer);
            Ok(json!({
                "outcome": "value",
                "written": buffer.len() - prefix.len(),
                "hex": hex(&buffer),
            }))
        }
        "read" => {
            let bytes = decode_hex(string(request, "hex")?)?;
            let mut remaining = bytes.as_slice();
            let packet = ServerboundLoginAcknowledgedPacket::decode(&mut remaining)
                .map_err(|error| format!("login acknowledgement decode failed: {error:?}"))?;
            Ok(json!({
                "singleton": packet == ServerboundLoginAcknowledgedPacket::INSTANCE,
                "outcome": "value",
                "consumed": bytes.len() - remaining.len(),
                "remaining_hex": hex(remaining),
            }))
        }
        _ => Err("unknown login acknowledgement direction".into()),
    }
}

pub(super) fn login_finished_result(request: &Value) -> Result<Value, String> {
    match string(request, "direction")? {
        "write" => {
            let profile_id = voxel_core::binary::Uuid::new(
                uuid_hex_field(request, "profile_most_hex")?,
                uuid_hex_field(request, "profile_least_hex")?,
            );
            let name = utf16_hex_units(string(request, "name_utf16_hex")?, "name_utf16_hex")?;
            let properties = request["properties"]
                .as_array()
                .ok_or("properties must be an array")?
                .iter()
                .map(|property| {
                    let name = utf16_hex_units(
                        string(property, "name_utf16_hex")?,
                        "property.name_utf16_hex",
                    )?;
                    let value = utf16_hex_units(
                        string(property, "value_utf16_hex")?,
                        "property.value_utf16_hex",
                    )?;
                    let signature = match property.get("signature_utf16_hex") {
                        Some(value) if !value.is_null() => Some(utf16_hex_units(
                            string(property, "signature_utf16_hex")?,
                            "property.signature_utf16_hex",
                        )?),
                        _ => None,
                    };
                    Ok(ProfileProperty::new(name, value, signature))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let session_id = voxel_core::binary::Uuid::new(
                uuid_hex_field(request, "session_most_hex")?,
                uuid_hex_field(request, "session_least_hex")?,
            );
            let packet = ClientboundLoginFinishedPacket::new(
                GameProfile::new(profile_id, name, properties),
                session_id,
            );
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let mut buffer = prefix.clone();
            let mut result = match packet.encode(&mut buffer) {
                Ok(()) => json!({"outcome": "value"}),
                Err(voxel_net::login::WriteError::EncoderError) => {
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
            let mut result = match ClientboundLoginFinishedPacket::decode(&mut remaining) {
                Ok(packet) => json!({
                    "profile_most_hex": format!("{:016x}", packet.game_profile.id.most),
                    "profile_least_hex": format!("{:016x}", packet.game_profile.id.least),
                    "name_utf16_hex": packet.game_profile.name.iter()
                        .map(|unit| format!("{unit:04x}"))
                        .collect::<String>(),
                    "properties": packet.game_profile.properties.iter().map(|property| json!({
                        "name_utf16_hex": property.name.iter()
                            .map(|unit| format!("{unit:04x}"))
                            .collect::<String>(),
                        "value_utf16_hex": property.value.iter()
                            .map(|unit| format!("{unit:04x}"))
                            .collect::<String>(),
                        "signature_utf16_hex": property.signature.as_ref().map(|signature| {
                            signature.iter()
                                .map(|unit| format!("{unit:04x}"))
                                .collect::<String>()
                        }),
                    })).collect::<Vec<_>>(),
                    "session_most_hex": format!("{:016x}", packet.session_id.most),
                    "session_least_hex": format!("{:016x}", packet.session_id.least),
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
        _ => Err("unknown login finished direction".into()),
    }
}

pub(super) fn configuration_unit_result(request: &Value) -> Result<Value, String> {
    let packet = string(request, "packet")?;
    match string(request, "direction")? {
        "write" => {
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let mut buffer = prefix.clone();
            match packet {
                "server-finish" => {
                    ServerboundFinishConfigurationPacket::INSTANCE.encode(&mut buffer)
                }
                "client-finish" => {
                    ClientboundFinishConfigurationPacket::INSTANCE.encode(&mut buffer)
                }
                "accept-code-of-conduct" => {
                    ServerboundAcceptCodeOfConductPacket::INSTANCE.encode(&mut buffer)
                }
                _ => return Err("unknown configuration unit packet".into()),
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
            let singleton = match packet {
                "server-finish" => ServerboundFinishConfigurationPacket::decode(&mut remaining)
                    .map(|value| value == ServerboundFinishConfigurationPacket::INSTANCE),
                "client-finish" => ClientboundFinishConfigurationPacket::decode(&mut remaining)
                    .map(|value| value == ClientboundFinishConfigurationPacket::INSTANCE),
                "accept-code-of-conduct" => {
                    ServerboundAcceptCodeOfConductPacket::decode(&mut remaining)
                        .map(|value| value == ServerboundAcceptCodeOfConductPacket::INSTANCE)
                }
                _ => return Err("unknown configuration unit packet".into()),
            }
            .map_err(|error| format!("configuration unit decode failed: {error:?}"))?;
            Ok(json!({
                "singleton": singleton,
                "outcome": "value",
                "consumed": bytes.len() - remaining.len(),
                "remaining_hex": hex(remaining),
            }))
        }
        _ => Err("unknown configuration unit direction".into()),
    }
}

pub(super) fn configuration_misc_result(request: &Value) -> Result<Value, String> {
    let packet = string(request, "packet")?;
    match string(request, "direction")? {
        "write" => {
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let mut buffer = prefix.clone();
            let mut result = match packet {
                "code-of-conduct" => {
                    let code_of_conduct = utf16_hex_units(
                        string(request, "code_of_conduct_utf16_hex")?,
                        "code_of_conduct_utf16_hex",
                    )?;
                    match ClientboundCodeOfConductPacket::new(code_of_conduct).encode(&mut buffer) {
                        Ok(()) => json!({"outcome": "value"}),
                        Err(voxel_net::configuration::WriteError::EncoderError) => {
                            json!({"outcome": "error", "error": "encoder_error"})
                        }
                    }
                }
                "reset-chat" => {
                    ClientboundResetChatPacket::INSTANCE.encode(&mut buffer);
                    json!({"outcome": "value"})
                }
                _ => return Err("unknown configuration misc packet".into()),
            };
            result["written"] = json!(buffer.len() - prefix.len());
            result["hex"] = json!(hex(&buffer));
            Ok(result)
        }
        "read" => {
            let bytes = decode_hex(string(request, "hex")?)?;
            let mut remaining = bytes.as_slice();
            let result = match packet {
                "code-of-conduct" => {
                    ClientboundCodeOfConductPacket::decode(&mut remaining).map(|packet| {
                        json!({
                            "code_of_conduct_utf16_hex": packet.code_of_conduct.iter()
                                .map(|unit| format!("{unit:04x}"))
                                .collect::<String>(),
                            "outcome": "value",
                        })
                    })
                }
                "reset-chat" => ClientboundResetChatPacket::decode(&mut remaining).map(|packet| {
                    json!({
                        "singleton": packet == ClientboundResetChatPacket::INSTANCE,
                        "outcome": "value",
                    })
                }),
                _ => return Err("unknown configuration misc packet".into()),
            };
            let mut result = match result {
                Ok(value) => value,
                Err(error) => json!({
                    "outcome": "error",
                    "error": login_read_error(error),
                }),
            };
            result["consumed"] = json!(bytes.len() - remaining.len());
            result["remaining_hex"] = json!(hex(remaining));
            Ok(result)
        }
        _ => Err("unknown configuration misc direction".into()),
    }
}

pub(super) fn protocol_metadata_result(request: &Value) -> Result<Value, String> {
    let kind = string(request, "kind")?;
    let name = string(request, "name")?;
    match kind {
        "connection" => {
            let protocol = match name {
                "HANDSHAKING" => ConnectionProtocol::Handshaking,
                "PLAY" => ConnectionProtocol::Play,
                "STATUS" => ConnectionProtocol::Status,
                "LOGIN" => ConnectionProtocol::Login,
                "CONFIGURATION" => ConnectionProtocol::Configuration,
                _ => return Err("unknown connection protocol".into()),
            };
            Ok(json!({
                "id": protocol.id(),
                "name": protocol.name(),
                "ordinal": protocol.ordinal(),
                "outcome": "value",
            }))
        }
        "flow" => {
            let flow = match name {
                "SERVERBOUND" => PacketFlow::Serverbound,
                "CLIENTBOUND" => PacketFlow::Clientbound,
                _ => return Err("unknown packet flow".into()),
            };
            Ok(json!({
                "id": flow.id(),
                "name": flow.name(),
                "opposite": flow.opposite().name(),
                "ordinal": flow.ordinal(),
                "outcome": "value",
            }))
        }
        _ => Err("unknown protocol metadata kind".into()),
    }
}

pub(super) fn client_information_chat_visibility(value: &str) -> Result<ChatVisibility, String> {
    match value {
        "full" => Ok(ChatVisibility::Full),
        "system" => Ok(ChatVisibility::System),
        "hidden" => Ok(ChatVisibility::Hidden),
        _ => Err("unknown chat visibility".into()),
    }
}

pub(super) fn client_information_main_hand(value: &str) -> Result<MainHand, String> {
    match value {
        "left" => Ok(MainHand::Left),
        "right" => Ok(MainHand::Right),
        _ => Err("unknown main hand".into()),
    }
}

pub(super) fn client_information_particle_status(value: &str) -> Result<ParticleStatus, String> {
    match value {
        "all" => Ok(ParticleStatus::All),
        "decreased" => Ok(ParticleStatus::Decreased),
        "minimal" => Ok(ParticleStatus::Minimal),
        _ => Err("unknown particle status".into()),
    }
}

pub(super) fn client_information_chat_visibility_name(value: ChatVisibility) -> &'static str {
    match value {
        ChatVisibility::Full => "full",
        ChatVisibility::System => "system",
        ChatVisibility::Hidden => "hidden",
    }
}

pub(super) fn client_information_main_hand_name(value: MainHand) -> &'static str {
    match value {
        MainHand::Left => "left",
        MainHand::Right => "right",
    }
}

pub(super) fn client_information_particle_status_name(value: ParticleStatus) -> &'static str {
    match value {
        ParticleStatus::All => "all",
        ParticleStatus::Decreased => "decreased",
        ParticleStatus::Minimal => "minimal",
    }
}
