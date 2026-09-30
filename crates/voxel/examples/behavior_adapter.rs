//! JSONL test glue. Behavior is delegated to the production library.

#![expect(
    clippy::cognitive_complexity,
    clippy::too_many_lines,
    reason = "the adapter answers every behaviour family in one dispatch"
)]

use serde_json::{Value, json};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};
use std::rc::Rc;
use voxel_core::aabb::{Aabb, AabbBuilder, AabbBuilderError, BlockHitResult, BoundingBox};
use voxel_core::axis_cycle::AxisCycle;
use voxel_core::binary::{
    self, BitSet, BlockPos, BlockRotation, ChunkPos, MutableBlockPos, SectionPos, Uuid,
};
use voxel_core::direction::{
    Axis, AxisDirection, Direction, DirectionYRotError, DirectionYRotationError, Plane,
};
use voxel_core::identifier::Identifier;
use voxel_core::math::{self, Quaternionf, Vec2, Vec3, Vec3f};
use voxel_core::nbt::{self, Field, FieldValue};
use voxel_core::random::RandomSource;
use voxel_core::shape::{BooleanOp, ShapeRotation, Shapes, VoxelShape};
use voxel_core::smooth::SmoothDouble;
use voxel_core::tags::{ClientboundUpdateTagsPacket, RegistryTags, TagEntry};
use voxel_core::timer::Timer;
use voxel_core::utf8;
use voxel_core::wire::{self, ReadError};
use voxel_game::input_event::{
    CharacterEvent, InputWithModifiers, KeyEvent as InputKeyEvent, MouseButtonEvent,
    MouseButtonInfo, PreeditEvent,
};
use voxel_game::key_mapping::{InputKey, KeyMapping, KeyMappingRegistry};
use voxel_game::option::{
    AltEnum, ClampingLazyMaxIntRange, CycleableEnum, IntRange, LazyEnum, SliderableEnum, UnitDouble,
};
use voxel_game::scroll::ScrollWheelHandler;
use voxel_net::common::{
    ClientboundKeepAlivePacket, ClientboundPingPacket, ServerboundKeepAlivePacket,
    ServerboundPongPacket,
};
use voxel_net::configuration::{
    ChatVisibility, ClientInformation, ClientboundCodeOfConductPacket,
    ClientboundFinishConfigurationPacket, ClientboundResetChatPacket,
    ClientboundUpdateEnabledFeaturesPacket, MainHand, ParticleStatus,
    ServerboundAcceptCodeOfConductPacket, ServerboundClientInformationPacket,
    ServerboundFinishConfigurationPacket,
};
use voxel_net::connection::{
    decode_chunk_section_data, parse_game_event_packet, parse_player_abilities_packet,
};
use voxel_net::frame::{self, FrameError, Varint21FrameDecoder};
use voxel_net::handshake::{self, ClientIntent, ClientIntentionPacket};
use voxel_net::login::{
    ClientboundHelloPacket, ClientboundLoginCompressionPacket, ClientboundLoginFinishedPacket,
    GameProfile, ProfileProperty, ServerboundHelloPacket, ServerboundLoginAcknowledgedPacket,
};
use voxel_net::protocol::{ConnectionProtocol, PacketFlow};
use voxel_net::registry_data::{ClientboundRegistryDataPacket, PackedRegistryEntry};
use voxel_net::resources::{
    self, AssetIndex, AssetResolution, KnownPack, KnownPackCodecError, KnownPackListCodecError,
    PackCompatibility, PackFormat, ReloadableResourceManager, ResourcePack, ResourcePackStack,
    VanillaPackResources,
};
use voxel_net::status::{
    ClientboundPongResponsePacket, ServerboundPingRequestPacket, ServerboundStatusRequestPacket,
};
use voxel_render::window;
use voxel_world::input::{
    Input, KeyboardInput, LocalPlayerClimbableMovementInput, LocalPlayerFluidFallingInput,
    LocalPlayerFrictionInfluencedSpeedInput, LocalPlayerHandleRelativeFrictionInput,
    LocalPlayerInputModifiers, LocalPlayerInputState, LocalPlayerModifiedFrictionInput,
    LocalPlayerSprintGateInput, LocalPlayerSprintTriggerConditions, LocalPlayerTravelDispatchInput,
    LocalPlayerTravelDispatchParameters, LocalPlayerTravelFlyingInput, LocalPlayerTravelInAirInput,
    LocalPlayerTravelInLavaInput, LocalPlayerTravelInWaterInput,
    LocalPlayerTravelInWaterParameters, LocalPlayerUpdateFallFlyingMovementInput,
    adjust_local_player_fluid_falling_movement, apply_local_player_input,
    can_start_local_player_sprinting, clamp_local_player_climbable_movement,
    local_player_friction_influenced_speed, local_player_handle_relative_friction,
    local_player_modified_friction, local_player_travel_dispatch, local_player_travel_flying,
    local_player_travel_in_air, local_player_travel_in_lava, local_player_travel_in_water,
    local_player_update_fall_flying_movement, modify_local_player_input,
};
use voxel_world::{
    BoatInput, ClientAttributeModifierState, ClientAttributeState, ClientChunkCacheSnapshot,
    ClientEntity, ClientEntityCollisionCandidate, ClientEntityDimensions, ClientEntityPose,
    ClientPlayerAbilities, ClientPlayerGameMode, ClientPlayerPoseInput, ClientWorld,
    ClientWorldBorder, ClientWorldBorderInitialization, ClientWorldBorderStatus,
    ClientWorldCollisionSource, DimensionBounds, EntityInput, EntityMotionInput, EntityMoveInput,
    EntityTeleportInput, LightLayerInput, LightUpdateInput, SectionInput,
    apply_entity_movement_state, collect_candidate_step_up_heights,
    collect_entity_collision_shapes, collide_entity_movement, collide_entity_movement_state,
    is_minor_horizontal_collision, living_entity_dimensions, refresh_entity_dimensions,
    select_step_up_movement, world_border_collision_shape,
};

fn string<'a>(request: &'a Value, name: &str) -> Result<&'a str, String> {
    request[name]
        .as_str()
        .ok_or_else(|| format!("{name} must be a string"))
}

fn string_value(value: &Value, name: &str) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_owned)
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

fn hex_bytes(text: &str) -> Result<Vec<u8>, String> {
    (0..text.len() / 2)
        .map(|index| u8::from_str_radix(&text[index * 2..index * 2 + 2], 16))
        .collect::<Result<_, _>>()
        .map_err(|error| format!("hex: {error}"))
}

fn read(input: &mut &[u8], wide: bool) -> Result<i64, ReadError> {
    if wide {
        wire::read_varlong(input)
    } else {
        wire::read_varint(input).map(i64::from)
    }
}

fn wire_result(request: &Value) -> Result<Value, String> {
    if request["operation"].as_str() == Some("frame_prepend") {
        return frame_prepend_result(request);
    }
    if request["operation"].as_str() == Some("frame_decode") {
        return frame_decode_result(request);
    }
    if request["operation"].as_str() == Some("handshake_intention") {
        return handshake_intention_result(request);
    }
    if request["operation"].as_str() == Some("login_hello") {
        return login_hello_result(request);
    }
    if request["operation"].as_str() == Some("login_compression") {
        return login_compression_result(request);
    }
    if request["operation"].as_str() == Some("login_server_hello") {
        return login_server_hello_result(request);
    }
    if request["operation"].as_str() == Some("login_acknowledged") {
        return login_acknowledged_result(request);
    }
    if request["operation"].as_str() == Some("login_finished") {
        return login_finished_result(request);
    }
    if request["operation"].as_str() == Some("configuration_unit") {
        return configuration_unit_result(request);
    }
    if request["operation"].as_str() == Some("configuration_misc") {
        return configuration_misc_result(request);
    }
    if request["operation"].as_str() == Some("protocol_metadata") {
        return protocol_metadata_result(request);
    }
    if request["operation"].as_str() == Some("client_information") {
        return client_information_result(request);
    }
    if request["operation"].as_str() == Some("common_packet") {
        return common_packet_result(request);
    }
    if request["operation"].as_str() == Some("enabled_features") {
        return enabled_features_result(request);
    }
    if request["operation"].as_str() == Some("update_tags") {
        return update_tags_result(request);
    }
    if request["operation"].as_str() == Some("registry_data") {
        return registry_data_result(request);
    }
    if request["operation"].as_str() == Some("status_request") {
        return status_request_result(request);
    }
    if request["operation"].as_str() == Some("ping") {
        return ping_result(request);
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

fn frame_error(error: FrameError) -> &'static str {
    match error {
        FrameError::LengthTooWide | FrameError::ZeroLength => "decoder_error",
        FrameError::PacketTooLarge => "encoder_error",
    }
}

fn frame_prepend_result(request: &Value) -> Result<Value, String> {
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

fn frame_decode_result(request: &Value) -> Result<Value, String> {
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

fn handshake_read_error(error: handshake::ReadError) -> &'static str {
    match error {
        handshake::ReadError::BufferUnderflow => "buffer_underflow",
        handshake::ReadError::DecoderError => "decoder_error",
        handshake::ReadError::TooBig => "too_big",
    }
}

fn handshake_intent(value: &str) -> Result<ClientIntent, String> {
    match value {
        "status" => Ok(ClientIntent::Status),
        "login" => Ok(ClientIntent::Login),
        "transfer" => Ok(ClientIntent::Transfer),
        _ => Err("unknown client intent".into()),
    }
}

fn utf16_hex_units(value: &str, name: &str) -> Result<Vec<u16>, String> {
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

fn handshake_intention_result(request: &Value) -> Result<Value, String> {
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

fn login_read_error(error: voxel_net::login::ReadError) -> &'static str {
    match error {
        voxel_net::login::ReadError::BufferUnderflow => "buffer_underflow",
        voxel_net::login::ReadError::DecoderError => "decoder_error",
        voxel_net::login::ReadError::TooBig => "too_big",
    }
}

fn uuid_hex_field(request: &Value, name: &str) -> Result<u64, String> {
    let value = string(request, name)?;
    if value.len() != 16 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be sixteen hexadecimal digits"));
    }
    u64::from_str_radix(value, 16).map_err(|_| format!("invalid hexadecimal {name}"))
}

fn login_hello_result(request: &Value) -> Result<Value, String> {
    match string(request, "direction")? {
        "write" => {
            let name = utf16_hex_units(string(request, "name_utf16_hex")?, "name_utf16_hex")?;
            let profile_id = voxel_core::binary::Uuid::new(
                uuid_hex_field(request, "profile_most_hex")?,
                uuid_hex_field(request, "profile_least_hex")?,
            );
            let prefix = decode_hex(string(request, "prefix_hex")?)?;
            let packet = ServerboundHelloPacket::new(name, profile_id);
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

fn compression_read_error(error: wire::ReadError) -> &'static str {
    match error {
        wire::ReadError::BufferUnderflow => "buffer_underflow",
        wire::ReadError::TooBig => "too_big",
    }
}

fn login_compression_result(request: &Value) -> Result<Value, String> {
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

fn login_server_hello_result(request: &Value) -> Result<Value, String> {
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

fn login_acknowledged_result(request: &Value) -> Result<Value, String> {
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

fn login_finished_result(request: &Value) -> Result<Value, String> {
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

fn configuration_unit_result(request: &Value) -> Result<Value, String> {
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

fn configuration_misc_result(request: &Value) -> Result<Value, String> {
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

fn protocol_metadata_result(request: &Value) -> Result<Value, String> {
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

fn client_information_chat_visibility(value: &str) -> Result<ChatVisibility, String> {
    match value {
        "full" => Ok(ChatVisibility::Full),
        "system" => Ok(ChatVisibility::System),
        "hidden" => Ok(ChatVisibility::Hidden),
        _ => Err("unknown chat visibility".into()),
    }
}

fn client_information_main_hand(value: &str) -> Result<MainHand, String> {
    match value {
        "left" => Ok(MainHand::Left),
        "right" => Ok(MainHand::Right),
        _ => Err("unknown main hand".into()),
    }
}

fn client_information_particle_status(value: &str) -> Result<ParticleStatus, String> {
    match value {
        "all" => Ok(ParticleStatus::All),
        "decreased" => Ok(ParticleStatus::Decreased),
        "minimal" => Ok(ParticleStatus::Minimal),
        _ => Err("unknown particle status".into()),
    }
}

fn client_information_chat_visibility_name(value: ChatVisibility) -> &'static str {
    match value {
        ChatVisibility::Full => "full",
        ChatVisibility::System => "system",
        ChatVisibility::Hidden => "hidden",
    }
}

fn client_information_main_hand_name(value: MainHand) -> &'static str {
    match value {
        MainHand::Left => "left",
        MainHand::Right => "right",
    }
}

fn client_information_particle_status_name(value: ParticleStatus) -> &'static str {
    match value {
        ParticleStatus::All => "all",
        ParticleStatus::Decreased => "decreased",
        ParticleStatus::Minimal => "minimal",
    }
}

fn client_information_result(request: &Value) -> Result<Value, String> {
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

fn common_read_error(error: voxel_net::common::ReadError) -> &'static str {
    match error {
        voxel_net::common::ReadError::BufferUnderflow => "buffer_underflow",
        voxel_net::common::ReadError::DecoderError => "decoder_error",
        voxel_net::common::ReadError::TooBig => "too_big",
    }
}

fn common_packet_result(request: &Value) -> Result<Value, String> {
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

fn enabled_features_result(request: &Value) -> Result<Value, String> {
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

fn update_tags_result(request: &Value) -> Result<Value, String> {
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

fn registry_data_read_error(error: voxel_net::registry_data::ReadError) -> &'static str {
    match error {
        voxel_net::registry_data::ReadError::BufferUnderflow => "buffer_underflow",
        voxel_net::registry_data::ReadError::DecoderError => "decoder_error",
        voxel_net::registry_data::ReadError::EncoderError => "encoder_error",
        voxel_net::registry_data::ReadError::TooBig => "too_big",
    }
}

fn registry_data_result(request: &Value) -> Result<Value, String> {
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

fn status_request_result(request: &Value) -> Result<Value, String> {
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

fn ping_read_error(error: voxel_net::status::ReadError) -> &'static str {
    match error {
        voxel_net::status::ReadError::BufferUnderflow => "buffer_underflow",
        voxel_net::status::ReadError::DecoderError => "decoder_error",
        voxel_net::status::ReadError::TooBig => "too_big",
    }
}

fn ping_result(request: &Value) -> Result<Value, String> {
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

fn f32_field(value: &Value, name: &str) -> Result<f32, String> {
    let text = string(value, name)?;
    if text.len() != 8 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be eight hexadecimal digits"));
    }
    u32::from_str_radix(text, 16)
        .map(f32::from_bits)
        .map_err(|error| error.to_string())
}

fn f32_value(value: &Value, name: &str) -> Result<f32, String> {
    let text = value
        .as_str()
        .ok_or_else(|| format!("{name} must be a string"))?;
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

fn u64_hex_field(value: &Value, name: &str) -> Result<u64, String> {
    let text = string(value, name)?;
    if text.len() != 16 || !text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{name} must be sixteen hexadecimal digits"));
    }
    u64::from_str_radix(text, 16).map_err(|error| error.to_string())
}

fn f64_value(value: &Value, name: &str) -> Result<f64, String> {
    let text = value
        .as_str()
        .ok_or_else(|| format!("{name} must be a string"))?;
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

fn input_snapshot(input: Input) -> Value {
    json!({
        "forward": input.forward,
        "backward": input.backward,
        "left": input.left,
        "right": input.right,
        "jump": input.jump,
        "shift": input.shift,
        "sprint": input.sprint,
    })
}

fn utf16_snapshot(units: &[u16]) -> Value {
    Value::Array(
        units
            .iter()
            .map(|unit| json!(format!("{unit:04x}")))
            .collect(),
    )
}

fn character_event_result(request: &Value) -> Result<Value, String> {
    let event = CharacterEvent::new(i32_field(request, "codepoint")?);
    let units = event
        .codepoint_as_utf16()
        .map_err(|error| format!("invalid codepoint: {}", error.0))?;
    Ok(json!({
        "codepoint": event.codepoint,
        "utf16": utf16_snapshot(&units),
        "is_allowed_chat_character": event.is_allowed_chat_character(),
    }))
}

fn i32_array_field(value: &Value, name: &str) -> Result<Vec<i32>, String> {
    value[name]
        .as_array()
        .ok_or_else(|| format!("{name} must be an array"))?
        .iter()
        .map(|entry| {
            entry
                .as_i64()
                .ok_or_else(|| format!("{name} entries must be integers"))?
                .try_into()
                .map_err(|_| format!("{name} entry must fit in i32"))
        })
        .collect()
}

fn preedit_snapshot(event: &PreeditEvent) -> Value {
    json!({
        "full_text": utf16_snapshot(&event.full_text),
        "caret_position": event.caret_position,
        "blocks": event.blocks.iter().map(|block| utf16_snapshot(block)).collect::<Vec<_>>(),
        "focused_block": event.focused_block,
    })
}

fn preedit_event_result(request: &Value) -> Result<Value, String> {
    let codepoints = i32_array_field(request, "codepoints")?;
    let block_sizes = i32_array_field(request, "block_sizes")?;
    let preedit_size = i32::try_from(codepoints.len())
        .map_err(|_| "codepoint array is too large for Java int".to_owned())?;
    let event = PreeditEvent::create_from_callback(
        preedit_size,
        &codepoints,
        &block_sizes,
        i32_field(request, "focused_block")?,
        i32_field(request, "caret")?,
    );
    Ok(match event {
        None => json!({}),
        Some(Ok(event)) => json!({"event": preedit_snapshot(&event)}),
        Some(Err(error)) => return Err(format!("preedit event error: {error:?}")),
    })
}

fn option_values_result(request: &Value) -> Result<Value, String> {
    let range = IntRange::with_apply_value_immediately(
        i32_field(request, "min")?,
        i32_field(request, "max")?,
        request["apply_value_immediately"]
            .as_bool()
            .ok_or("apply_value_immediately must be a boolean")?,
    );
    let int_values = request["int_values"]
        .as_array()
        .ok_or("int_values must be an array")?
        .iter()
        .map(|value| {
            let value = value
                .as_i64()
                .ok_or("int_values entries must be integers")?
                .try_into()
                .map_err(|_| "int_values entry must fit in i32")?;
            Ok(json!({
                "value": value,
                "valid": range.validate_value(value).is_some(),
                "next": range.next(value),
                "previous": range.previous(value),
                "to_slider": format!("{:016x}", range.to_slider_value(value).to_bits()),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let sliders = request["sliders"]
        .as_array()
        .ok_or("sliders must be an array")?
        .iter()
        .map(|value| {
            let slider = f64_value(value, "slider")?;
            Ok(json!({
                "slider": format!("{:016x}", slider.to_bits()),
                "from_slider": range.from_slider_value(slider),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let unit_values = request["unit_values"]
        .as_array()
        .ok_or("unit_values must be an array")?
        .iter()
        .map(|value| {
            let value = f64_value(value, "unit_value")?;
            let mut state = serde_json::Map::new();
            state.insert("value".into(), json!(format!("{:016x}", value.to_bits())));
            state.insert(
                "valid".into(),
                json!(UnitDouble::validate_value(value).is_some()),
            );
            if let Some(validated) = UnitDouble::validate_value(value) {
                state.insert(
                    "validated".into(),
                    json!(format!("{:016x}", validated.to_bits())),
                );
            }
            state.insert(
                "to_slider".into(),
                json!(format!(
                    "{:016x}",
                    UnitDouble::to_slider_value(value).to_bits()
                )),
            );
            state.insert(
                "from_slider".into(),
                json!(format!(
                    "{:016x}",
                    UnitDouble::from_slider_value(value).to_bits()
                )),
            );
            Ok(Value::Object(state))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "min": range.min_inclusive,
        "max": range.max_inclusive,
        "apply_value_immediately": range.apply_value_immediately,
        "int_values": int_values,
        "sliders": sliders,
        "unit_values": unit_values,
    }))
}

fn sliderable_enum_result(request: &Value) -> Result<Value, String> {
    const GRAPHICS_PRESETS: [&str; 4] = ["FAST", "FANCY", "FABULOUS", "CUSTOM"];
    let value_set = SliderableEnum::new(
        GRAPHICS_PRESETS
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    );
    let probes = request["probes"]
        .as_array()
        .ok_or("probes must be an array")?
        .iter()
        .map(|probe| {
            let value = string(probe, "value")?.to_owned();
            let next = value_set.next(&value);
            let previous = value_set.previous(&value);
            Ok(json!({
                "value": value,
                "valid": value_set.validate_value(&value).is_some(),
                "to_slider": format!("{:016x}", value_set.to_slider_value(&value).to_bits()),
                "next": next,
                "previous": previous,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let sliders = request["sliders"]
        .as_array()
        .ok_or("sliders must be an array")?
        .iter()
        .map(|entry| {
            let slider = f64_value(entry, "slider")?;
            Ok(json!({
                "slider": format!("{:016x}", slider.to_bits()),
                "from_slider": value_set.from_slider_value(slider),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "values": value_set.values(),
        "probes": probes,
        "sliders": sliders,
    }))
}

fn cycleable_enum_result(request: &Value) -> Result<Value, String> {
    let values = request["values"]
        .as_array()
        .ok_or("values must be an array")?
        .iter()
        .map(|value| string_value(value, "values entry"))
        .collect::<Result<Vec<_>, String>>()?;
    let value_set = CycleableEnum::new(values);
    let probes = request["probes"]
        .as_array()
        .ok_or("probes must be an array")?
        .iter()
        .map(|probe| {
            let value = string_value(probe, "probe")?;
            let mut state = serde_json::Map::new();
            state.insert("value".into(), json!(&value));
            state.insert(
                "valid".into(),
                json!(value_set.validate_value(&value).is_some()),
            );
            if let Some(validated) = value_set.validate_value(&value) {
                state.insert("validated".into(), json!(validated));
            }
            Ok(Value::Object(state))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "values": value_set.values(),
        "selected_values": value_set.values(),
        "probes": probes,
    }))
}

fn alt_enum_result(request: &Value) -> Result<Value, String> {
    let values = request["values"]
        .as_array()
        .ok_or("values must be an array")?
        .iter()
        .map(|value| string_value(value, "values entry"))
        .collect::<Result<Vec<_>, String>>()?;
    let alt_values = request["alt_values"]
        .as_array()
        .ok_or("alt_values must be an array")?
        .iter()
        .map(|value| string_value(value, "alt_values entry"))
        .collect::<Result<Vec<_>, String>>()?;
    let conditions = request["conditions"]
        .as_array()
        .ok_or("conditions must be an array")?
        .iter()
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| "condition entries must be booleans".to_owned())
        })
        .collect::<Result<Vec<_>, String>>()?;
    if conditions.is_empty() {
        return Err("conditions must not be empty".into());
    }
    let supplied_conditions = conditions.clone();
    let condition_calls = Rc::new(RefCell::new(0_usize));
    let condition_counter = Rc::clone(&condition_calls);
    let value_set = AltEnum::new(values, alt_values, move || {
        let mut call = condition_counter.borrow_mut();
        let index = (*call).min(supplied_conditions.len() - 1);
        *call += 1;
        supplied_conditions[index]
    });
    let mut selected_values = Vec::with_capacity(conditions.len());
    for _ in &conditions {
        selected_values.push(value_set.selected_values());
    }
    let probes = request["probes"]
        .as_array()
        .ok_or("probes must be an array")?
        .iter()
        .map(|probe| {
            let value = string_value(probe, "probe")?;
            let mut state = serde_json::Map::new();
            state.insert("value".into(), json!(&value));
            let validated = value_set.validate_value(&value);
            state.insert("valid".into(), json!(validated.is_some()));
            if let Some(validated) = validated {
                state.insert("validated".into(), json!(validated));
            }
            Ok(Value::Object(state))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "default_values": value_set.values(),
        "alt_values": value_set.alt_values(),
        "selected_values": selected_values,
        "probes": probes,
    }))
}

fn lazy_enum_result(request: &Value) -> Result<Value, String> {
    let value_lists = request["value_lists"]
        .as_array()
        .ok_or("value_lists must be an array")?
        .iter()
        .map(|values| {
            values
                .as_array()
                .ok_or("value list must be an array")?
                .iter()
                .map(|value| string_value(value, "value list entry"))
                .collect::<Result<Vec<_>, String>>()
        })
        .collect::<Result<Vec<_>, String>>()?;
    if value_lists.is_empty() {
        return Err("value_lists must not be empty".into());
    }
    let valid_values = request["valid_values"]
        .as_array()
        .ok_or("valid_values must be an array")?
        .iter()
        .map(|value| string_value(value, "valid value"))
        .collect::<Result<Vec<_>, String>>()?;
    let calls = Rc::new(RefCell::new(0_usize));
    let supplier_calls = Rc::clone(&calls);
    let supplied_lists = value_lists.clone();
    let value_set = LazyEnum::new(
        move || {
            let mut call = supplier_calls.borrow_mut();
            let index = (*call).min(supplied_lists.len() - 1);
            *call += 1;
            supplied_lists[index].clone()
        },
        move |value: &String| {
            valid_values
                .iter()
                .any(|candidate| candidate == value)
                .then(|| value.clone())
        },
    );
    let first_values = value_set.values();
    let second_values = value_set.values();
    let probes = request["probes"]
        .as_array()
        .ok_or("probes must be an array")?
        .iter()
        .map(|probe| {
            let value = string_value(probe, "probe")?;
            let mut state = serde_json::Map::new();
            state.insert("value".into(), json!(&value));
            state.insert(
                "valid".into(),
                json!(value_set.validate_value(&value).is_some()),
            );
            if let Some(validated) = value_set.validate_value(&value) {
                state.insert("validated".into(), json!(validated));
            }
            Ok(Value::Object(state))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "first_default": first_values,
        "first_selected": first_values.clone(),
        "second_default": second_values,
        "second_selected": second_values.clone(),
        "probes": probes,
    }))
}

fn clamping_lazy_max_int_range_result(request: &Value) -> Result<Value, String> {
    let max_values = request["max_values"]
        .as_array()
        .ok_or("max_values must be an array")?
        .iter()
        .map(|value| {
            value
                .as_i64()
                .ok_or("max_values entries must be integers")?
                .try_into()
                .map_err(|_| "max_values entry must fit in i32".to_owned())
        })
        .collect::<Result<Vec<i32>, String>>()?;
    if max_values.is_empty() {
        return Err("max_values must not be empty".into());
    }
    let supplied_maxima = max_values.clone();
    let max_calls = Rc::new(RefCell::new(0_usize));
    let supplier_calls = Rc::clone(&max_calls);
    let range = ClampingLazyMaxIntRange::new(
        i32_field(request, "min")?,
        move || {
            let mut call = supplier_calls.borrow_mut();
            let index = (*call).min(supplied_maxima.len() - 1);
            *call += 1;
            supplied_maxima[index]
        },
        i32_field(request, "encodable_max")?,
    );
    let maxima = vec![range.max_inclusive(), range.max_inclusive()];
    let probes = request["probes"]
        .as_array()
        .ok_or("probes must be an array")?
        .iter()
        .map(|entry| {
            let value = entry
                .as_i64()
                .ok_or("probe entries must be integers")?
                .try_into()
                .map_err(|_| "probe entry must fit in i32".to_owned())?;
            Ok(json!({
                "value": value,
                "validated": range.validate_value(value),
                "next": range.next(value),
                "previous": range.previous(value),
                "to_slider": format!("{:016x}", range.to_slider_value(value).to_bits()),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let sliders = request["sliders"]
        .as_array()
        .ok_or("sliders must be an array")?
        .iter()
        .map(|entry| {
            let slider = f64_value(entry, "slider")?;
            Ok(json!({
                "slider": format!("{:016x}", slider.to_bits()),
                "from_slider": range.from_slider_value(slider),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let values = range.values();
    Ok(json!({
        "min": range.min_inclusive(),
        "encodable_max": range.encodable_max_inclusive(),
        "maxima": maxima,
        "create_cycle_button": range.create_cycle_button(),
        "values": values.clone(),
        "selected_values": values,
        "probes": probes,
        "sliders": sliders,
    }))
}

fn mapped_int_range_result(request: &Value) -> Result<Value, String> {
    let range = IntRange::new(i32_field(request, "min")?, i32_field(request, "max")?);
    let discrete = request["discrete"]
        .as_bool()
        .ok_or("discrete must be a boolean")?;
    let mapped = range.xmap(
        |value| value.to_string(),
        |value: &String| {
            value
                .parse::<i32>()
                .expect("mapped test values must be signed integers")
        },
        discrete,
    );
    let values = request["values"]
        .as_array()
        .ok_or("values must be an array")?
        .iter()
        .map(|value| string_value(value, "mapped value"))
        .collect::<Result<Vec<_>, String>>()?;
    let value_results = values
        .iter()
        .map(|value| {
            let mut state = serde_json::Map::new();
            state.insert("value".into(), json!(value));
            let validated = mapped.validate_value(value);
            state.insert("valid".into(), json!(validated.is_some()));
            if let Some(validated) = validated {
                state.insert("validated".into(), json!(validated));
            }
            state.insert(
                "to_slider".into(),
                json!(format!("{:016x}", mapped.to_slider_value(value).to_bits())),
            );
            if let Some(next) = mapped.next(value) {
                state.insert("next".into(), json!(next));
            }
            if let Some(previous) = mapped.previous(value) {
                state.insert("previous".into(), json!(previous));
            }
            Value::Object(state)
        })
        .collect::<Vec<_>>();
    let sliders = request["sliders"]
        .as_array()
        .ok_or("sliders must be an array")?
        .iter()
        .map(|entry| {
            let slider = f64_value(entry, "slider")?;
            Ok(json!({
                "slider": format!("{:016x}", slider.to_bits()),
                "from_slider": mapped.from_slider_value(slider),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "values": value_results,
        "sliders": sliders,
    }))
}

fn mapped_unit_double_result(request: &Value) -> Result<Value, String> {
    let mapped = UnitDouble::xmap(|value| value * value, |value: &f64| value.sqrt());
    let values = request["values"]
        .as_array()
        .ok_or("values must be an array")?
        .iter()
        .map(|value| f64_value(value, "mapped value"))
        .collect::<Result<Vec<_>, String>>()?;
    let value_results = values
        .iter()
        .map(|value| {
            let mut state = serde_json::Map::new();
            state.insert("value".into(), json!(format!("{:016x}", value.to_bits())));
            let validated = mapped.validate_value(value);
            state.insert("valid".into(), json!(validated.is_some()));
            if let Some(validated) = validated {
                state.insert(
                    "validated".into(),
                    json!(format!("{:016x}", validated.to_bits())),
                );
            }
            state.insert(
                "to_slider".into(),
                json!(format!("{:016x}", mapped.to_slider_value(value).to_bits())),
            );
            Value::Object(state)
        })
        .collect::<Vec<_>>();
    let sliders = request["sliders"]
        .as_array()
        .ok_or("sliders must be an array")?
        .iter()
        .map(|entry| {
            let slider = f64_value(entry, "slider")?;
            Ok(json!({
                "slider": format!("{:016x}", slider.to_bits()),
                "from_slider": format!("{:016x}", mapped.from_slider_value(slider).to_bits()),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "values": value_results,
        "sliders": sliders,
    }))
}

fn scroll_wheel_result(request: &Value) -> Result<Value, String> {
    let mut handler = ScrollWheelHandler::new();
    let events = request["scrolls"]
        .as_array()
        .ok_or("scrolls must be an array")?
        .iter()
        .map(|step| {
            let (x, y) = handler.on_mouse_scroll(f64_field(step, "x")?, f64_field(step, "y")?);
            Ok(json!({"x": x, "y": y}))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let selections = request["selections"]
        .as_array()
        .ok_or("selections must be an array")?
        .iter()
        .map(|selection| {
            Ok(json!({
                "wheel": format!("{:016x}", f64_field(selection, "wheel")?.to_bits()),
                "current": i32_field(selection, "current")?,
                "limit": i32_field(selection, "limit")?,
                "selected": ScrollWheelHandler::next_scroll_wheel_selection(
                    f64_field(selection, "wheel")?,
                    i32_field(selection, "current")?,
                    i32_field(selection, "limit")?,
                ),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"scrolls": events, "selections": selections}))
}

fn smooth_double_result(request: &Value) -> Result<Value, String> {
    let mut smooth = SmoothDouble::new();
    let events = request["steps"]
        .as_array()
        .ok_or("steps must be an array")?
        .iter()
        .map(|step| {
            let operation = string(step, "op")?;
            if operation == "reset" {
                smooth.reset();
                Ok(json!({"op": "reset"}))
            } else if operation == "delta" {
                let value = smooth.get_new_delta_value(
                    f64_field(step, "target_delta")?,
                    f64_field(step, "time")?,
                );
                Ok(json!({"op": "delta", "value": format!("{:016x}", value.to_bits())}))
            } else {
                Err("unknown SmoothDouble operation".into())
            }
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"events": events}))
}

fn resources_result(request: &Value) -> Result<Value, String> {
    if request["operation"].as_str() == Some("packs") {
        return resource_packs_result(request);
    }
    if request["operation"].as_str() == Some("reload") {
        return resource_reload_result(request);
    }
    if request["operation"].as_str() == Some("compatibility") {
        return resource_compatibility_result(request);
    }
    if request["operation"].as_str() == Some("vanilla") {
        return resource_vanilla_result(request);
    }
    if request["operation"].as_str() == Some("known_pack") {
        return resource_known_pack_result(request);
    }
    if request["operation"].as_str() == Some("known_pack_codec") {
        return resource_known_pack_codec_result(request);
    }
    if request["operation"].as_str() == Some("known_pack_list_codec") {
        return resource_known_pack_list_codec_result(request);
    }

    let entries = request["entries"]
        .as_array()
        .ok_or("entries must be an array")?
        .iter()
        .map(|entry| {
            Ok((
                string(entry, "path")?.to_owned(),
                string(entry, "hash")?.to_owned(),
            ))
        })
        .collect::<Result<Vec<(String, String)>, String>>()?;
    let index = AssetIndex::from_entries(entries)
        .map_err(|error| format!("invalid asset index: {error:?}"))?;
    let objects = request["objects"]
        .as_array()
        .ok_or("objects must be an array")?
        .iter()
        .map(|entry| {
            Ok((
                string(entry, "hash")?.to_owned(),
                decode_hex(string(entry, "hex")?)?,
            ))
        })
        .collect::<Result<std::collections::HashMap<String, Vec<u8>>, String>>()?;
    let queries = request["queries"]
        .as_array()
        .ok_or("queries must be an array")?
        .iter()
        .map(|entry| {
            let path = entry
                .as_str()
                .ok_or("query entries must be strings")?
                .to_owned();
            let mut result = serde_json::Map::new();
            result.insert("path".into(), json!(&path));
            match index.resolve(&path) {
                AssetResolution::Missing => {
                    result.insert("exists".into(), json!(false));
                    result.insert("directory".into(), json!(false));
                    result.insert("readable".into(), json!(false));
                }
                AssetResolution::Directory => {
                    result.insert("exists".into(), json!(true));
                    result.insert("directory".into(), json!(true));
                    result.insert("readable".into(), json!(false));
                }
                AssetResolution::File { hash } => {
                    result.insert("exists".into(), json!(true));
                    result.insert("directory".into(), json!(false));
                    result.insert("object_path".into(), json!(resources::object_path(hash)));
                    if let Some(bytes) = objects.get(hash) {
                        result.insert("readable".into(), json!(true));
                        result.insert("bytes_hex".into(), json!(hex(bytes)));
                    } else {
                        result.insert("readable".into(), json!(false));
                    }
                }
            }
            Ok(Value::Object(result))
        })
        .collect::<Result<Vec<Value>, String>>()?;
    Ok(json!({"queries": queries}))
}

fn resource_packs_result(request: &Value) -> Result<Value, String> {
    let pack_stack = ResourcePackStack::new(parse_resource_packs(&request["packs"])?);
    let queries = resource_queries_result(&pack_stack, &request["queries"])?;
    Ok(json!({"queries": queries}))
}

fn parse_resource_packs(value: &Value) -> Result<Vec<ResourcePack>, String> {
    value
        .as_array()
        .ok_or("packs must be an array")?
        .iter()
        .map(|pack_value| {
            let mut pack = ResourcePack::new(string(pack_value, "id")?.to_owned());
            for resource_value in pack_value["resources"]
                .as_array()
                .ok_or("pack resources must be an array")?
            {
                pack.insert(
                    string(resource_value, "id")?.to_owned(),
                    decode_hex(string(resource_value, "hex")?)?,
                );
            }
            Ok(pack)
        })
        .collect::<Result<Vec<_>, String>>()
}

fn resource_queries_result(
    pack_stack: &ResourcePackStack,
    query_values: &Value,
) -> Result<Value, String> {
    let queries = query_values
        .as_array()
        .ok_or("queries must be an array")?
        .iter()
        .map(|query_value| {
            let id = query_value
                .as_str()
                .ok_or("query entries must be strings")?;
            let stack = pack_stack.resource_stack(id);
            let mut result = serde_json::Map::new();
            result.insert("id".into(), json!(id));
            result.insert("exists".into(), json!(pack_stack.resource(id).is_some()));
            result.insert(
                "stack".into(),
                json!(
                    stack
                        .iter()
                        .map(|(pack_id, _)| *pack_id)
                        .collect::<Vec<_>>()
                ),
            );
            if let Some((source, bytes)) = pack_stack.resource(id) {
                result.insert("source".into(), json!(source));
                result.insert("bytes_hex".into(), json!(hex(bytes)));
            }
            Ok(Value::Object(result))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!(queries))
}

fn resource_reload_result(request: &Value) -> Result<Value, String> {
    let mut manager = ReloadableResourceManager::new();
    let initial = json!({
        "packs": manager.resources().pack_ids(),
        "namespaces": manager.resources().namespaces(),
        "queries": resource_queries_result(manager.resources(), &request["initial_queries"])?,
    });
    let rounds = request["rounds"]
        .as_array()
        .ok_or("rounds must be an array")?
        .iter()
        .enumerate()
        .map(|(round, round_value)| {
            manager.reload(parse_resource_packs(&round_value["packs"])?);
            Ok(json!({
                "packs": manager.resources().pack_ids(),
                "namespaces": manager.resources().namespaces(),
                "queries": resource_queries_result(manager.resources(), &round_value["queries"])?,
                "round": round,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"initial": initial, "rounds": rounds}))
}

fn resource_compatibility_result(request: &Value) -> Result<Value, String> {
    let game = PackFormat::new(88, 0);
    let probes = request["probes"]
        .as_array()
        .ok_or("probes must be an array")?
        .iter()
        .map(|probe| {
            let declared_min = PackFormat::new(
                i32_field(probe, "min_major")?,
                i32_field(probe, "min_minor")?,
            );
            let declared_max = PackFormat::new(
                i32_field(probe, "max_major")?,
                i32_field(probe, "max_minor")?,
            );
            let compatibility = PackCompatibility::for_version(declared_min, declared_max, game);
            Ok(json!({
                "compatibility": compatibility.name(),
                "is_compatible": compatibility.is_compatible(),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "game_major": game.major(),
        "game_minor": game.minor(),
        "probes": probes,
    }))
}

fn resource_vanilla_result(request: &Value) -> Result<Value, String> {
    let namespaces = request["namespaces"]
        .as_array()
        .ok_or("namespaces must be an array")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| "namespace entries must be strings".to_owned())
                .map(str::to_owned)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let resources = VanillaPackResources::new(namespaces, parse_resource_packs(&request["roots"])?);
    let queries = request["queries"]
        .as_array()
        .ok_or("queries must be an array")?
        .iter()
        .map(|query_value| {
            let id = query_value
                .as_str()
                .ok_or("query entries must be strings")?;
            let mut result = serde_json::Map::new();
            result.insert("id".into(), json!(id));
            if let Some((_root_id, bytes)) = resources.resource(id) {
                result.insert("exists".into(), json!(true));
                result.insert("stack".into(), json!(["vanilla"]));
                result.insert("source".into(), json!("vanilla"));
                result.insert("bytes_hex".into(), json!(hex(bytes)));
            } else {
                result.insert("exists".into(), json!(false));
                result.insert("stack".into(), json!([]));
            }
            Ok(Value::Object(result))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "namespaces": resources.namespaces(),
        "queries": queries,
    }))
}

fn resource_known_pack_result(request: &Value) -> Result<Value, String> {
    let packs = request["probes"]
        .as_array()
        .ok_or("probes must be an array")?
        .iter()
        .map(|probe| {
            let pack = if probe["kind"].as_str() == Some("vanilla") {
                KnownPack::vanilla(string(probe, "id")?.to_owned())
            } else {
                KnownPack::new(
                    string(probe, "namespace")?.to_owned(),
                    string(probe, "id")?.to_owned(),
                    string(probe, "version")?.to_owned(),
                )
            };
            Ok(json!({
                "namespace": pack.namespace(),
                "id": pack.id(),
                "version": pack.version(),
                "is_vanilla": pack.is_vanilla(),
                "string": pack.to_string(),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"packs": packs}))
}

fn known_pack_codec_error(error: KnownPackCodecError) -> &'static str {
    match error {
        KnownPackCodecError::Read(utf8::ReadError::DecoderError) => "decoder_error",
        KnownPackCodecError::Read(utf8::ReadError::BufferUnderflow) => "buffer_underflow",
        KnownPackCodecError::Read(utf8::ReadError::TooBig) => "too_big",
        KnownPackCodecError::Write(utf8::WriteError::EncoderError) => "encoder_error",
        KnownPackCodecError::InvalidUtf16 => "decoder_error",
    }
}

fn resource_known_pack_codec_result(request: &Value) -> Result<Value, String> {
    let prefix = decode_hex(string(request, "prefix_hex")?)?;
    match string(request, "mode")? {
        "encode" => {
            let pack = KnownPack::new(
                string(request, "namespace")?.to_owned(),
                string(request, "id")?.to_owned(),
                string(request, "version")?.to_owned(),
            );
            let mut bytes = prefix.clone();
            let result = match pack.encode_stream(&mut bytes) {
                Ok(()) => json!({
                    "outcome": "value",
                    "written": bytes.len() - prefix.len(),
                    "hex": hex(&bytes),
                }),
                Err(error) => json!({
                    "outcome": "error",
                    "error": known_pack_codec_error(error),
                    "written": bytes.len() - prefix.len(),
                    "hex": hex(&bytes),
                }),
            };
            Ok(result)
        }
        "decode" => {
            let framed = decode_hex(string(request, "hex")?)?;
            let mut remaining = framed.as_slice();
            match KnownPack::decode_stream(&mut remaining) {
                Ok(pack) => Ok(json!({
                    "namespace": pack.namespace(),
                    "id": pack.id(),
                    "version": pack.version(),
                    "string": pack.to_string(),
                    "outcome": "value",
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
                Err(error) => Ok(json!({
                    "outcome": "error",
                    "error": known_pack_codec_error(error),
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
            }
        }
        _ => Err("unknown KnownPack codec operation".into()),
    }
}

fn known_pack_list_codec_error(error: KnownPackListCodecError, encoding: bool) -> &'static str {
    match error {
        KnownPackListCodecError::CountRead(error) => match error {
            wire::ReadError::BufferUnderflow => "buffer_underflow",
            wire::ReadError::TooBig => "too_big",
        },
        KnownPackListCodecError::CountTooBig => {
            if encoding {
                "encoder_error"
            } else {
                "decoder_error"
            }
        }
        KnownPackListCodecError::NegativeCount => "invalid_count",
        KnownPackListCodecError::Pack(error) => known_pack_codec_error(error),
    }
}

fn parse_known_pack_list(value: &Value) -> Result<Vec<KnownPack>, String> {
    value
        .as_array()
        .ok_or("packs must be an array")?
        .iter()
        .map(|pack| {
            Ok(KnownPack::new(
                string(pack, "namespace")?.to_owned(),
                string(pack, "id")?.to_owned(),
                string(pack, "version")?.to_owned(),
            ))
        })
        .collect()
}

fn known_pack_list_value(packs: &[KnownPack]) -> Value {
    Value::Array(
        packs
            .iter()
            .map(|pack| {
                json!({
                    "namespace": pack.namespace(),
                    "id": pack.id(),
                    "version": pack.version(),
                    "string": pack.to_string(),
                })
            })
            .collect(),
    )
}

fn resource_known_pack_list_codec_result(request: &Value) -> Result<Value, String> {
    let packet = string(request, "packet")?;
    let max_size = match packet {
        "clientbound" => i32::MAX as usize,
        "serverbound" => 64,
        _ => return Err("unknown KnownPack list packet".into()),
    };
    let prefix = decode_hex(string(request, "prefix_hex")?)?;
    match string(request, "mode")? {
        "encode" => {
            let packs = parse_known_pack_list(&request["packs"])?;
            let mut bytes = prefix.clone();
            let result = match KnownPack::encode_list(&packs, &mut bytes, max_size) {
                Ok(()) => json!({
                    "outcome": "value",
                    "written": bytes.len() - prefix.len(),
                    "hex": hex(&bytes),
                }),
                Err(error) => json!({
                    "outcome": "error",
                    "error": known_pack_list_codec_error(error, true),
                    "written": bytes.len() - prefix.len(),
                    "hex": hex(&bytes),
                }),
            };
            Ok(result)
        }
        "decode" => {
            let framed = decode_hex(string(request, "hex")?)?;
            let mut remaining = framed.as_slice();
            match KnownPack::decode_list(&mut remaining, max_size) {
                Ok(packs) => Ok(json!({
                    "packs": known_pack_list_value(&packs),
                    "outcome": "value",
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
                Err(error) => Ok(json!({
                    "outcome": "error",
                    "error": known_pack_list_codec_error(error, false),
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
            }
        }
        _ => Err("unknown KnownPack list codec operation".into()),
    }
}

fn input_with_modifiers_snapshot<T: InputWithModifiers>(input: &T) -> Value {
    json!({
        "input": input.input(),
        "modifiers": input.modifiers(),
        "is_selection": input.is_selection(),
        "is_confirmation": input.is_confirmation(),
        "is_escape": input.is_escape(),
        "is_left": input.is_left(),
        "is_right": input.is_right(),
        "is_up": input.is_up(),
        "is_down": input.is_down(),
        "is_cycle_focus": input.is_cycle_focus(),
        "digit": input.get_digit(),
        "has_alt_down": input.has_alt_down(),
        "has_shift_down": input.has_shift_down(),
        "has_control_down": input.has_control_down(),
        "has_control_down_with_quirk": input.has_control_down_with_quirk(),
        "is_select_all": input.is_select_all(),
        "is_copy": input.is_copy(),
        "is_paste": input.is_paste(),
        "is_cut": input.is_cut(),
    })
}

fn input_event_result(request: &Value) -> Result<Value, String> {
    let key_event = InputKeyEvent::new(
        i32_field(request, "key")?,
        i32_field(request, "scancode")?,
        i32_field(request, "modifiers")?,
    );
    let button_info = MouseButtonInfo::new(
        i32_field(request, "mouse_button")?,
        i32_field(request, "modifiers")?,
    );
    let mouse_event = MouseButtonEvent::new(
        f64_field(request, "mouse_x")?,
        f64_field(request, "mouse_y")?,
        button_info,
    );
    let mut key = input_with_modifiers_snapshot(&key_event);
    key["key"] = json!(key_event.key);
    key["scancode"] = json!(key_event.scancode);
    let mut info = input_with_modifiers_snapshot(&button_info);
    info["button"] = json!(button_info.button);
    let mut mouse = input_with_modifiers_snapshot(&mouse_event);
    mouse["x"] = json!(format!("{:016x}", mouse_event.x.to_bits()));
    mouse["y"] = json!(format!("{:016x}", mouse_event.y.to_bits()));
    mouse["button"] = json!(mouse_event.button());
    Ok(json!({"key_event": key, "mouse_button_info": info, "mouse_button_event": mouse}))
}

fn input_key(value: &Value) -> Result<InputKey, String> {
    let kind = string(value, "type")?;
    let number = i32_field(value, "value")?;
    match kind {
        "keysym" => Ok(InputKey::key_sym(number)),
        "scancode" => Ok(InputKey::scan_code(number)),
        "mouse" => Ok(InputKey::mouse(number)),
        _ => Err("unknown input key type".into()),
    }
}

fn mapping_id(
    target: &str,
    primary: usize,
    secondary: usize,
    other: usize,
) -> Result<usize, String> {
    match target {
        "primary" => Ok(primary),
        "secondary" => Ok(secondary),
        "other" => Ok(other),
        _ => Err("unknown key mapping target".into()),
    }
}

fn action_key(
    target: &str,
    primary: InputKey,
    primary_default: InputKey,
    secondary: InputKey,
    other: InputKey,
    rebind: InputKey,
) -> Result<InputKey, String> {
    match target {
        "primary" => Ok(primary),
        "primary_default" => Ok(primary_default),
        "secondary" => Ok(secondary),
        "other" => Ok(other),
        "rebind" => Ok(rebind),
        _ => Err("unknown action key".into()),
    }
}

fn key_mapping_state(
    registry: &KeyMappingRegistry,
    primary: usize,
    secondary: usize,
    other: usize,
) -> Result<Value, String> {
    let primary_down = registry
        .mapping(primary)
        .ok_or("missing primary key mapping")?
        .is_down();
    let secondary_down = registry
        .mapping(secondary)
        .ok_or("missing secondary key mapping")?
        .is_down();
    let other_down = registry
        .mapping(other)
        .ok_or("missing other key mapping")?
        .is_down();
    Ok(json!({
        "primary_down": primary_down,
        "secondary_down": secondary_down,
        "other_down": other_down,
    }))
}

fn key_mapping_property(
    mapping: &KeyMapping,
    key_event: (i32, i32),
    unknown_key_event: (i32, i32),
    mouse_button: i32,
) -> Value {
    json!({
        "name": mapping.name(),
        "key": mapping.key().name(),
        "default_key": mapping.default_key().name(),
        "is_toggle": mapping.is_toggle(),
        "is_default": mapping.is_default(),
        "is_unbound": mapping.is_unbound(),
        "save": mapping.save_string(),
        "matches_key_event": mapping.matches_key_event(key_event.0, key_event.1),
        "matches_unknown_key_event": mapping.matches_key_event(unknown_key_event.0, unknown_key_event.1),
        "matches_mouse": mapping.matches_mouse(mouse_button),
    })
}

fn key_mapping_properties(
    registry: &KeyMappingRegistry,
    primary: usize,
    secondary: usize,
    other: usize,
    key_event: (i32, i32),
    unknown_key_event: (i32, i32),
    mouse_button: i32,
) -> Result<Value, String> {
    let primary_mapping = registry
        .mapping(primary)
        .ok_or("missing primary key mapping")?;
    let secondary_mapping = registry
        .mapping(secondary)
        .ok_or("missing secondary key mapping")?;
    let other_mapping = registry.mapping(other).ok_or("missing other key mapping")?;
    Ok(json!({
        "primary": key_mapping_property(primary_mapping, key_event, unknown_key_event, mouse_button),
        "secondary": key_mapping_property(secondary_mapping, key_event, unknown_key_event, mouse_button),
        "other": key_mapping_property(other_mapping, key_event, unknown_key_event, mouse_button),
        "primary_same_secondary": primary_mapping.same(secondary_mapping),
        "primary_same_other": primary_mapping.same(other_mapping),
    }))
}

fn key_mapping_result(request: &Value) -> Result<Value, String> {
    let primary_key = input_key(&request["primary"])?;
    let primary_default_key = primary_key;
    let mut secondary_key = input_key(&request["secondary"])?;
    let mut other_key = input_key(&request["other"])?;
    let rebind_key = input_key(&request["rebind"])?;
    let mut current_primary_key = primary_key;
    let prefix = format!("behavior.key-mapping.{}", string(request, "case_id")?);
    let mut registry = KeyMappingRegistry::default();
    let primary = registry.register(format!("{prefix}.primary"), primary_key);
    let secondary = registry.register(format!("{prefix}.secondary"), secondary_key);
    let other = registry.register(format!("{prefix}.other"), other_key);
    let key_event = (
        i32_field(&request["key_event"], "key")?,
        i32_field(&request["key_event"], "scancode")?,
    );
    let unknown_key_event = (
        i32_field(&request["unknown_key_event"], "key")?,
        i32_field(&request["unknown_key_event"], "scancode")?,
    );
    let mouse_button = i32_field(request, "mouse_button")?;
    let initial = key_mapping_state(&registry, primary, secondary, other)?;
    let initial_properties = key_mapping_properties(
        &registry,
        primary,
        secondary,
        other,
        key_event,
        unknown_key_event,
        mouse_button,
    )?;
    let mut events = Vec::new();
    for action in request["actions"]
        .as_array()
        .ok_or("actions must be an array")?
    {
        let kind = string(action, "op")?;
        let mut consumed = None;
        match kind {
            "set_down" => {
                let id = mapping_id(string(action, "target")?, primary, secondary, other)?;
                registry
                    .mapping_mut(id)
                    .ok_or("missing key mapping")?
                    .set_down(action["value"].as_bool().ok_or("value must be a boolean")?);
            }
            "set" => {
                let key = action_key(
                    string(action, "key")?,
                    current_primary_key,
                    primary_default_key,
                    secondary_key,
                    other_key,
                    rebind_key,
                )?;
                registry.set(
                    key,
                    action["value"].as_bool().ok_or("value must be a boolean")?,
                );
            }
            "click" => {
                let key = action_key(
                    string(action, "key")?,
                    current_primary_key,
                    primary_default_key,
                    secondary_key,
                    other_key,
                    rebind_key,
                )?;
                registry.click(key);
            }
            "consume_click" => {
                let id = mapping_id(string(action, "target")?, primary, secondary, other)?;
                consumed = Some(
                    registry
                        .mapping_mut(id)
                        .ok_or("missing key mapping")?
                        .consume_click(),
                );
            }
            "set_key" => {
                let target = string(action, "target")?;
                let replacement = action_key(
                    string(action, "key")?,
                    current_primary_key,
                    primary_default_key,
                    secondary_key,
                    other_key,
                    rebind_key,
                )?;
                let id = mapping_id(target, primary, secondary, other)?;
                registry
                    .mapping_mut(id)
                    .ok_or("missing key mapping")?
                    .set_key(replacement);
                match target {
                    "primary" => current_primary_key = replacement,
                    "secondary" => secondary_key = replacement,
                    "other" => other_key = replacement,
                    _ => return Err("unknown key mapping target".into()),
                }
            }
            "release_all" => registry.release_all(),
            _ => return Err("unknown key mapping action".into()),
        }
        let mut event = key_mapping_state(&registry, primary, secondary, other)?;
        if let Some(value) = consumed {
            event["consumed"] = json!(value);
        }
        events.push(event);
    }
    let properties = key_mapping_properties(
        &registry,
        primary,
        secondary,
        other,
        key_event,
        unknown_key_event,
        mouse_button,
    )?;
    Ok(
        json!({"initial": initial, "initial_properties": initial_properties,
        "events": events, "properties": properties}),
    )
}

fn toggle_key_mapping_result(request: &Value) -> Result<Value, String> {
    let primary_key = input_key(&request["primary"])?;
    let secondary_key = input_key(&request["secondary"])?;
    let other_key = input_key(&request["other"])?;
    let prefix = format!("behavior.key-mapping.{}", string(request, "case_id")?);
    let mut registry = KeyMappingRegistry::default();
    let primary = registry.register_toggle(
        format!("{prefix}.primary"),
        primary_key,
        request["primary_needs_toggle"]
            .as_bool()
            .ok_or("primary_needs_toggle must be a boolean")?,
        request["primary_should_restore"]
            .as_bool()
            .ok_or("primary_should_restore must be a boolean")?,
    );
    let secondary = registry.register_toggle(
        format!("{prefix}.secondary"),
        secondary_key,
        request["secondary_needs_toggle"]
            .as_bool()
            .ok_or("secondary_needs_toggle must be a boolean")?,
        request["secondary_should_restore"]
            .as_bool()
            .ok_or("secondary_should_restore must be a boolean")?,
    );
    let other = registry.register_toggle(
        format!("{prefix}.other"),
        other_key,
        request["other_needs_toggle"]
            .as_bool()
            .ok_or("other_needs_toggle must be a boolean")?,
        request["other_should_restore"]
            .as_bool()
            .ok_or("other_should_restore must be a boolean")?,
    );
    let key_event = (
        i32_field(&request["key_event"], "key")?,
        i32_field(&request["key_event"], "scancode")?,
    );
    let unknown_key_event = (
        i32_field(&request["unknown_key_event"], "key")?,
        i32_field(&request["unknown_key_event"], "scancode")?,
    );
    let mouse_button = i32_field(request, "mouse_button")?;
    let initial = key_mapping_state(&registry, primary, secondary, other)?;
    let initial_properties = key_mapping_properties(
        &registry,
        primary,
        secondary,
        other,
        key_event,
        unknown_key_event,
        mouse_button,
    )?;
    let mut events = Vec::new();
    for action in request["actions"]
        .as_array()
        .ok_or("actions must be an array")?
    {
        let kind = string(action, "op")?;
        let mut consumed = None;
        let mut should_restore = None;
        match kind {
            "set_down" => {
                let id = mapping_id(string(action, "target")?, primary, secondary, other)?;
                registry
                    .mapping_mut(id)
                    .ok_or("missing key mapping")?
                    .set_down(action["value"].as_bool().ok_or("value must be a boolean")?);
            }
            "set" => {
                let key = action_key(
                    string(action, "key")?,
                    primary_key,
                    primary_key,
                    secondary_key,
                    other_key,
                    primary_key,
                )?;
                registry.set(
                    key,
                    action["value"].as_bool().ok_or("value must be a boolean")?,
                );
            }
            "click" => {
                let key = action_key(
                    string(action, "key")?,
                    primary_key,
                    primary_key,
                    secondary_key,
                    other_key,
                    primary_key,
                )?;
                registry.click(key);
            }
            "consume_click" => {
                let id = mapping_id(string(action, "target")?, primary, secondary, other)?;
                consumed = Some(
                    registry
                        .mapping_mut(id)
                        .ok_or("missing key mapping")?
                        .consume_click(),
                );
            }
            "release_all" => registry.release_all(),
            "restore_toggle_states" => registry.restore_toggle_states_on_screen_closed(),
            "reset_toggle_keys" => registry.reset_toggle_keys(),
            "should_restore" => {
                let id = mapping_id(string(action, "target")?, primary, secondary, other)?;
                should_restore = Some(
                    registry
                        .mapping_mut(id)
                        .ok_or("missing key mapping")?
                        .should_restore_state_on_screen_closed(),
                );
            }
            _ => return Err("unknown toggle key mapping action".into()),
        }
        let mut event = key_mapping_state(&registry, primary, secondary, other)?;
        if let Some(value) = consumed {
            event["consumed"] = json!(value);
        }
        if let Some(value) = should_restore {
            event["should_restore"] = json!(value);
        }
        events.push(event);
    }
    let properties = key_mapping_properties(
        &registry,
        primary,
        secondary,
        other,
        key_event,
        unknown_key_event,
        mouse_button,
    )?;
    Ok(
        json!({"initial": initial, "initial_properties": initial_properties,
        "events": events, "properties": properties}),
    )
}

fn vec3_snapshot(vector: Vec3) -> Value {
    json!({
        "x": format!("{:016x}", vector.x.to_bits()),
        "y": format!("{:016x}", vector.y.to_bits()),
        "z": format!("{:016x}", vector.z.to_bits()),
        "length": format!("{:016x}", vector.length().to_bits()),
        "length_squared": format!("{:016x}", vector.length_squared().to_bits()),
    })
}

fn vec3f_snapshot(vector: Vec3f) -> Value {
    json!({
        "x": format!("{:08x}", vector.x.to_bits()),
        "y": format!("{:08x}", vector.y.to_bits()),
        "z": format!("{:08x}", vector.z.to_bits()),
    })
}

fn quaternion_snapshot(quaternion: Quaternionf) -> Value {
    json!({
        "x": format!("{:08x}", quaternion.x.to_bits()),
        "y": format!("{:08x}", quaternion.y.to_bits()),
        "z": format!("{:08x}", quaternion.z.to_bits()),
        "w": format!("{:08x}", quaternion.w.to_bits()),
    })
}

fn direction_snapshot(facing: Direction) -> Value {
    let (step_x, step_y, step_z) = facing.step();
    json!({
        "name": facing.name(),
        "data3d": facing.data3d(),
        "data2d": facing.data2d(),
        "opposite": facing.opposite().name(),
        "axis": facing.axis().name(),
        "axis_direction": facing.axis_direction().name(),
        "step_x": step_x,
        "step_y": step_y,
        "step_z": step_z,
    })
}

fn blockpos(value: &Value) -> Result<BlockPos, String> {
    let coordinate = |name: &str| {
        string(value, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    Ok(BlockPos::new(
        coordinate("x")?,
        coordinate("y")?,
        coordinate("z")?,
    ))
}

fn bounding_box(value: &Value) -> Result<BoundingBox, String> {
    let coordinate = |name: &str| {
        string(value, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    Ok(BoundingBox::new(
        coordinate("min_x")?,
        coordinate("min_y")?,
        coordinate("min_z")?,
        coordinate("max_x")?,
        coordinate("max_y")?,
        coordinate("max_z")?,
    ))
}

fn blockpos_snapshot(value: BlockPos) -> Value {
    json!({
        "x": value.x.to_string(),
        "y": value.y.to_string(),
        "z": value.z.to_string(),
    })
}

fn blockpos_packed_helpers_snapshot(
    value: BlockPos,
    neighbor_node: u64,
    direction: Direction,
    step_x: i32,
    step_y: i32,
    step_z: i32,
) -> Value {
    let packed = value.as_long();
    json!({
        "zero": blockpos_snapshot(BlockPos::ZERO),
        "packed_horizontal_length": BlockPos::PACKED_HORIZONTAL_LENGTH,
        "packed_y_length": BlockPos::PACKED_Y_LENGTH,
        "max_horizontal_coordinate": BlockPos::MAX_HORIZONTAL_COORDINATE,
        "packed": format!("{packed:x}"),
        "coordinate_packed": format!("{:x}", BlockPos::as_long_coordinates(value.x, value.y, value.z)),
        "decoded": blockpos_snapshot(BlockPos::of(packed)),
        "accessors": {
            "x": BlockPos::get_x(packed).to_string(),
            "y": BlockPos::get_y(packed).to_string(),
            "z": BlockPos::get_z(packed).to_string(),
        },
        "flat_index": format!("{:x}", BlockPos::get_flat_index(neighbor_node)),
        "offset_direction": format!("{:x}", BlockPos::offset_packed(neighbor_node, direction)),
        "offset_steps": format!("{:x}", BlockPos::offset_packed_by(
            neighbor_node, step_x, step_y, step_z,
        )),
    })
}

fn blockpos_relative_helpers_snapshot(
    value: BlockPos,
    direction: Direction,
    axis: Axis,
    steps: i32,
) -> Value {
    json!({
        "relative_direction": blockpos_snapshot(value.relative_direction(direction, 1)),
        "relative_direction_steps": blockpos_snapshot(value.relative_direction(direction, steps)),
        "relative_axis_steps": blockpos_snapshot(value.relative_axis(axis, steps)),
        "cardinal": {
            "above": blockpos_snapshot(value.above()),
            "above_steps": blockpos_snapshot(value.above_steps(steps)),
            "below": blockpos_snapshot(value.below()),
            "below_steps": blockpos_snapshot(value.below_steps(steps)),
            "north": blockpos_snapshot(value.north()),
            "north_steps": blockpos_snapshot(value.north_steps(steps)),
            "south": blockpos_snapshot(value.south()),
            "south_steps": blockpos_snapshot(value.south_steps(steps)),
            "west": blockpos_snapshot(value.west()),
            "west_steps": blockpos_snapshot(value.west_steps(steps)),
            "east": blockpos_snapshot(value.east()),
            "east_steps": blockpos_snapshot(value.east_steps(steps)),
        },
    })
}

fn blockpos_vector_helpers_snapshot(value: BlockPos, delta: BlockPos, scale: i32) -> Value {
    json!({
        "offset_vec3i": blockpos_snapshot(value.offset_by_block_pos(delta)),
        "subtract_vec3i": blockpos_snapshot(value.subtract_block_pos(delta)),
        "multiply": blockpos_snapshot(value.multiply(scale)),
    })
}

fn blockpos_geometry_helpers_snapshot(
    value: BlockPos,
    rotation: BlockRotation,
    up_vector: BlockPos,
    y: i32,
) -> Value {
    json!({
        "rotate": blockpos_snapshot(value.rotate(rotation)),
        "cross": blockpos_snapshot(value.cross(up_vector)),
        "at_y": blockpos_snapshot(value.at_y(y)),
    })
}

fn blockpos_distance_helpers_snapshot(
    value: BlockPos,
    other: BlockPos,
    point: Vec3,
    distance: f64,
) -> Value {
    json!({
        "dist_sqr": format!("{:016x}", value.dist_sqr(other).to_bits()),
        "dist_low_corner_sqr": format!(
            "{:016x}",
            value.dist_to_low_corner_sqr(point.x, point.y, point.z).to_bits(),
        ),
        "dist_center_sqr": format!(
            "{:016x}",
            value.dist_to_center_sqr(point.x, point.y, point.z).to_bits(),
        ),
        "closer_than": value.closer_than(other, distance),
    })
}

fn blockpos_integer_helpers_snapshot(value: BlockPos, other: BlockPos) -> Value {
    json!({
        "dist_manhattan": value.dist_manhattan(other),
        "dist_chessboard": value.dist_chessboard(other),
        "get": {
            "x": value.get(Axis::X),
            "y": value.get(Axis::Y),
            "z": value.get(Axis::Z),
        },
    })
}

fn blockpos_clamp_location_snapshot(value: BlockPos, location: Vec3) -> Value {
    json!({
        "value": vec3_snapshot(value.clamp_location_within(location)),
    })
}

fn chunkpos(value: &Value) -> Result<ChunkPos, String> {
    let coordinate = |name: &str| {
        string(value, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    Ok(ChunkPos::new(coordinate("x")?, coordinate("z")?))
}

fn sectionpos(value: &Value) -> Result<SectionPos, String> {
    let coordinate = |name: &str| {
        string(value, name)?
            .parse::<i32>()
            .map_err(|error| error.to_string())
    };
    Ok(SectionPos::new(
        coordinate("x")?,
        coordinate("y")?,
        coordinate("z")?,
    ))
}

fn chunkpos_snapshot(value: ChunkPos) -> Value {
    json!({
        "x": value.x.to_string(),
        "z": value.z.to_string(),
    })
}

fn sectionpos_snapshot(value: SectionPos) -> Value {
    json!({
        "x": value.x.to_string(),
        "y": value.y.to_string(),
        "z": value.z.to_string(),
        "packed": format!("{:x}", value.as_long()),
    })
}

fn chunkpos_block_coordinates_snapshot(value: ChunkPos, offset: i32) -> Value {
    json!({
        "min_block_x": value.get_min_block_x().to_string(),
        "min_block_z": value.get_min_block_z().to_string(),
        "middle_block_x": value.get_middle_block_x().to_string(),
        "middle_block_z": value.get_middle_block_z().to_string(),
        "max_block_x": value.get_max_block_x().to_string(),
        "max_block_z": value.get_max_block_z().to_string(),
        "block_x": value.get_block_x(offset).to_string(),
        "block_z": value.get_block_z(offset).to_string(),
    })
}

fn sectionpos_block_coordinates_snapshot(value: SectionPos) -> Value {
    json!({
        "min_block_x": value.min_block_x().to_string(),
        "min_block_y": value.min_block_y().to_string(),
        "min_block_z": value.min_block_z().to_string(),
        "max_block_x": value.max_block_x().to_string(),
        "max_block_y": value.max_block_y().to_string(),
        "max_block_z": value.max_block_z().to_string(),
    })
}

fn sectionpos_spatial_snapshot(value: SectionPos) -> Value {
    json!({
        "origin": blockpos_snapshot(value.origin()),
        "center": blockpos_snapshot(value.center()),
        "chunk": chunkpos_snapshot(value.chunk()),
    })
}

fn sectionpos_packed_transformations_snapshot(
    block_node: u64,
    section_node: u64,
    x: i32,
    z: i32,
) -> Value {
    json!({
        "block_to_section": format!("{:x}", SectionPos::block_to_section(block_node)),
        "zero_node": format!("{:x}", SectionPos::get_zero_node(section_node)),
        "zero_node_coordinates": format!("{:x}", SectionPos::get_zero_node_from_coordinates(x, z)),
        "section_to_chunk": format!("{:x}", SectionPos::section_to_chunk(section_node)),
    })
}

fn sectionpos_as_long_snapshot(
    block_node: u64,
    x: i32,
    y: i32,
    z: i32,
    section_x: i32,
    section_y: i32,
    section_z: i32,
) -> Value {
    json!({
        "from_block_pos": format!("{:x}", SectionPos::as_long_from_block_pos(BlockPos::unpack(block_node))),
        "coordinates": format!("{:x}", SectionPos::as_long_coordinates(x, y, z)),
        "instance": format!("{:x}", SectionPos::new(section_x, section_y, section_z).as_long()),
    })
}

fn sectionpos_offset_snapshot(
    section_node: u64,
    direction: Direction,
    steps: (i32, i32, i32),
    initial: SectionPos,
) -> Value {
    let instance = initial.offset(steps.0, steps.1, steps.2);
    json!({
        "packed_direction": format!("{:x}", SectionPos::offset_packed(section_node, direction)),
        "packed_steps": format!("{:x}", SectionPos::offset_packed_by(section_node, steps.0, steps.1, steps.2)),
        "instance_x": instance.x.to_string(),
        "instance_y": instance.y.to_string(),
        "instance_z": instance.z.to_string(),
        "instance_packed": format!("{:x}", instance.as_long()),
    })
}

fn sectionpos_ranges_snapshot(values: Vec<SectionPos>) -> Value {
    json!({
        "positions": values
            .into_iter()
            .map(sectionpos_snapshot)
            .collect::<Vec<_>>(),
    })
}

fn blockpos_positions_snapshot(values: Vec<BlockPos>) -> Value {
    json!({
        "positions": values
            .into_iter()
            .map(blockpos_snapshot)
            .collect::<Vec<_>>(),
    })
}

fn blockpos_position_list(values: Vec<BlockPos>) -> Value {
    Value::Array(values.into_iter().map(blockpos_snapshot).collect())
}

fn sectionpos_nodes_snapshot(values: Vec<u64>) -> Value {
    json!({
        "section_nodes": values
            .into_iter()
            .map(|value| format!("{value:x}"))
            .collect::<Vec<_>>(),
    })
}

fn chunkpos_region_coordinates_snapshot(value: ChunkPos, region_x: i32, region_z: i32) -> Value {
    json!({
        "region_x": value.get_region_x().to_string(),
        "region_z": value.get_region_z().to_string(),
        "packed_region_x": ChunkPos::get_region_x_from_packed(value.pack()).to_string(),
        "packed_region_z": ChunkPos::get_region_z_from_packed(value.pack()).to_string(),
        "region_local_x": value.get_region_local_x().to_string(),
        "region_local_z": value.get_region_local_z().to_string(),
        "min_from_region": chunkpos_snapshot(ChunkPos::min_from_region(region_x, region_z)),
        "max_from_region": chunkpos_snapshot(ChunkPos::max_from_region(region_x, region_z)),
    })
}

fn chunkpos_block_positions_snapshot(
    value: ChunkPos,
    block_x: i32,
    y: i32,
    block_z: i32,
    contains: &[Value],
) -> Result<Value, String> {
    Ok(json!({
        "block_at": blockpos_snapshot(value.get_block_at(block_x, y, block_z)),
        "middle_block_position": blockpos_snapshot(value.get_middle_block_position(y)),
        "world_position": blockpos_snapshot(value.get_world_position()),
        "contains": contains
            .iter()
            .map(|query| blockpos(query).map(|pos| value.contains(pos)))
            .collect::<Result<Vec<_>, _>>()?,
    }))
}

fn chunkpos_representation_snapshot(value: ChunkPos, hash_x: i32, hash_z: i32) -> Value {
    json!({
        "to_string": value.to_string_java(),
        "hash_code": value.hash_code_java().to_string(),
        "static_hash": ChunkPos::hash(hash_x, hash_z).to_string(),
    })
}

fn chunkpos_from_blockpos_snapshot(value: BlockPos) -> Value {
    let containing = ChunkPos::containing(value);
    json!({
        "containing": chunkpos_snapshot(containing),
        "packed": format!("{:x}", ChunkPos::pack_block_pos(value)),
    })
}

fn chunkpos_distances_snapshot(value: ChunkPos, other: ChunkPos) -> Value {
    json!({
        "chessboard_chunk": value.get_chessboard_distance(other).to_string(),
        "chessboard_coordinates": value
            .get_chessboard_distance_coordinates(other.x, other.z)
            .to_string(),
        "distance_squared_chunk": value.distance_squared(other).to_string(),
        "distance_squared_packed": value
            .distance_squared_from_packed(other.pack())
            .to_string(),
    })
}

fn chunkpos_validity_snapshot(value: ChunkPos) -> Value {
    json!({
        "instance": value.is_valid(),
        "static": ChunkPos::is_valid_coordinates(value.x, value.z),
    })
}

fn chunkpos_range_closed_snapshot(values: Vec<ChunkPos>) -> Value {
    json!({
        "positions": values
            .into_iter()
            .map(chunkpos_snapshot)
            .collect::<Vec<_>>(),
    })
}

fn blockpos_result(request: &Value) -> Result<Value, String> {
    let mut value = blockpos(request)?;
    let mut events = vec![blockpos_snapshot(value)];
    for step in request["steps"]
        .as_array()
        .ok_or("steps must be an array")?
    {
        value = match string(step, "action")? {
            "offset" => value.offset(
                string(step, "x")?
                    .parse::<i32>()
                    .map_err(|error| error.to_string())?,
                string(step, "y")?
                    .parse::<i32>()
                    .map_err(|error| error.to_string())?,
                string(step, "z")?
                    .parse::<i32>()
                    .map_err(|error| error.to_string())?,
            ),
            "min" => BlockPos::min(value, blockpos(step)?),
            "max" => BlockPos::max(value, blockpos(step)?),
            _ => return Err("unknown BlockPos action".into()),
        };
        events.push(blockpos_snapshot(value));
    }
    Ok(json!({"events": events}))
}

fn section_relative_result(request: &Value) -> Result<Value, String> {
    let relative = SectionPos::section_relative_pos(blockpos(request)?);
    Ok(json!({
        "relative": relative.to_string(),
        "x": SectionPos::section_relative_x(relative).to_string(),
        "y": SectionPos::section_relative_y(relative).to_string(),
        "z": SectionPos::section_relative_z(relative).to_string(),
    }))
}

fn random_result(request: &Value) -> Result<Value, String> {
    let seed = random_seed(request)?;
    let mut random = RandomSource::create(seed);
    let mut events = vec![json!({})];
    for step in request["steps"]
        .as_array()
        .ok_or("steps must be an array")?
    {
        events.push(random_state(&mut random, step)?);
    }
    Ok(json!({"events": events}))
}

fn random_seed(request: &Value) -> Result<i64, String> {
    string(request, "seed")?
        .parse::<i64>()
        .map_err(|error| error.to_string())
}

fn random_state(random: &mut RandomSource, step: &Value) -> Result<Value, String> {
    let mut state = serde_json::Map::new();
    match string(step, "op")? {
        "nextInt" => {
            state.insert("value".into(), json!(random.next_int().to_string()));
        }
        "nextIntBound" => {
            let bound = step["bound"]
                .as_i64()
                .ok_or("bound must be an integer")?
                .try_into()
                .map_err(|_| "bound must fit in i32".to_owned())?;
            let value = random
                .next_int_bound(bound)
                .map_err(|_| "bound must be positive".to_owned())?;
            state.insert("value".into(), json!(value.to_string()));
        }
        "nextLong" => {
            state.insert("value".into(), json!(random.next_long().to_string()));
        }
        "nextBoolean" => {
            state.insert("value".into(), json!(random.next_boolean().to_string()));
        }
        "nextFloat" => {
            state.insert(
                "bits".into(),
                json!(format!("{:08x}", random.next_float().to_bits())),
            );
        }
        "nextDouble" => {
            state.insert(
                "bits".into(),
                json!(format!("{:016x}", random.next_double().to_bits())),
            );
        }
        "nextGaussian" => {
            state.insert(
                "bits".into(),
                json!(format!("{:016x}", random.next_gaussian().to_bits())),
            );
        }
        "setSeed" => {
            let seed = string(step, "seed")?
                .parse::<i64>()
                .map_err(|error| error.to_string())?;
            random.set_seed(seed);
            state.insert("op".into(), json!("setSeed"));
        }
        "nextIntBetweenInclusive" => {
            let value = random
                .next_int_between_inclusive(i32_field(step, "min")?, i32_field(step, "max")?)
                .map_err(|_| "bound must be positive".to_owned())?;
            state.insert("value".into(), json!(value.to_string()));
        }
        "nextIntRange" => {
            let value = random
                .next_int_range(i32_field(step, "origin")?, i32_field(step, "bound")?)
                .map_err(|_| "bound - origin must be positive".to_owned())?;
            state.insert("value".into(), json!(value.to_string()));
        }
        "triangleDouble" => {
            state.insert(
                "bits".into(),
                json!(format!(
                    "{:016x}",
                    random
                        .triangle_f64(f64_field(step, "mean")?, f64_field(step, "spread")?)
                        .to_bits()
                )),
            );
        }
        "triangleFloat" => {
            state.insert(
                "bits".into(),
                json!(format!(
                    "{:08x}",
                    random
                        .triangle_f32(f32_field(step, "mean")?, f32_field(step, "spread")?)
                        .to_bits()
                )),
            );
        }
        "consumeCount" => {
            random.consume_count(i32_field(step, "rounds")?);
            state.insert("op".into(), json!("consumeCount"));
        }
        _ => return Err("unknown random operation".into()),
    }
    Ok(Value::Object(state))
}

fn random_steps(random: &mut RandomSource, value: &Value) -> Result<Value, String> {
    let mut events = Vec::new();
    for step in value.as_array().ok_or("random steps must be an array")? {
        events.push(random_state(random, step)?);
    }
    Ok(Value::Array(events))
}

fn random_fork_result(request: &Value) -> Result<Value, String> {
    let mut random = RandomSource::create(random_seed(request)?);
    let prefix_events = random_steps(
        &mut random,
        request
            .get("prefix_steps")
            .ok_or("prefix_steps must be an array")?,
    )?;
    let mut child = random.fork();
    let child_events = random_steps(
        &mut child,
        request
            .get("child_steps")
            .ok_or("child_steps must be an array")?,
    )?;
    let parent_events = random_steps(
        &mut random,
        request
            .get("parent_steps")
            .ok_or("parent_steps must be an array")?,
    )?;
    Ok(json!({
        "prefix_events": prefix_events,
        "child_events": child_events,
        "parent_events": parent_events,
    }))
}

fn i32_field(value: &Value, name: &str) -> Result<i32, String> {
    value[name]
        .as_i64()
        .ok_or_else(|| format!("{name} must be an integer"))?
        .try_into()
        .map_err(|_| format!("{name} must fit in i32"))
}

fn i64_field(value: &Value, name: &str) -> Result<i64, String> {
    value[name]
        .as_i64()
        .ok_or_else(|| format!("{name} must be an i64"))
}

fn i32_string_field(value: &Value, name: &str) -> Result<i32, String> {
    string(value, name)?
        .parse::<i32>()
        .map_err(|error| format!("{name} must fit in i32: {error}"))
}

fn random_positional_result(request: &Value) -> Result<Value, String> {
    let mut random = RandomSource::create(random_seed(request)?);
    let factory = random.fork_positional();
    let mut factory_events = Vec::new();
    for step in request
        .get("factory_steps")
        .ok_or("factory_steps must be an array")?
        .as_array()
        .ok_or("factory_steps must be an array")?
    {
        let mut child = match string(step, "op")? {
            "at" => factory.at(
                i32_field(step, "x")?,
                i32_field(step, "y")?,
                i32_field(step, "z")?,
            ),
            "fromHashOf" => factory.from_hash_of(string(step, "name")?),
            "fromSeed" => factory.from_seed(
                string(step, "seed")?
                    .parse::<i64>()
                    .map_err(|error| error.to_string())?,
            ),
            _ => return Err("unknown positional random operation".into()),
        };
        let events = random_steps(
            &mut child,
            step.get("steps").ok_or("steps must be an array")?,
        )?;
        factory_events.push(json!({"op": string(step, "op")?, "events": events}));
    }
    let parent_events = random_steps(
        &mut random,
        request
            .get("parent_steps")
            .ok_or("parent_steps must be an array")?,
    )?;
    Ok(json!({
        "factory_events": factory_events,
        "parent_events": parent_events,
    }))
}

fn aabb_box(value: &Value) -> Result<Aabb, String> {
    Ok(Aabb::new(
        f64_field(value, "min_x")?,
        f64_field(value, "min_y")?,
        f64_field(value, "min_z")?,
        f64_field(value, "max_x")?,
        f64_field(value, "max_y")?,
        f64_field(value, "max_z")?,
    ))
}

fn aabb_snapshot(value: Aabb) -> Value {
    json!({
        "min_x": format!("{:016x}", value.min_x.to_bits()),
        "min_y": format!("{:016x}", value.min_y.to_bits()),
        "min_z": format!("{:016x}", value.min_z.to_bits()),
        "max_x": format!("{:016x}", value.max_x.to_bits()),
        "max_y": format!("{:016x}", value.max_y.to_bits()),
        "max_z": format!("{:016x}", value.max_z.to_bits()),
    })
}

fn aabb_builder_result(request: &Value) -> Result<Value, String> {
    let points = request
        .get("points")
        .ok_or("points must be an array")?
        .as_array()
        .ok_or("points must be an array")?;
    let mut builder = AabbBuilder::new();
    let mut events = vec![json!({"defined": builder.is_defined()})];
    for point in points {
        builder.include(
            f32_field(point, "x")?,
            f32_field(point, "y")?,
            f32_field(point, "z")?,
        );
        events.push(json!({"defined": builder.is_defined()}));
    }
    let build = match builder.build() {
        Ok(value) => json!({"outcome": "value", "box": aabb_snapshot(value)}),
        Err(AabbBuilderError::Undefined) => json!({
            "outcome": "error",
            "error": "undefined",
        }),
    };
    Ok(json!({"events": events, "build": build}))
}

fn aabb_sizes_snapshot(value: Aabb) -> Value {
    json!({
        "x_size": format!("{:016x}", value.x_size().to_bits()),
        "y_size": format!("{:016x}", value.y_size().to_bits()),
        "z_size": format!("{:016x}", value.z_size().to_bits()),
        "size": format!("{:016x}", value.size().to_bits()),
    })
}

fn aabb_positions_snapshot(value: Aabb) -> Value {
    json!({
        "center": vec3_snapshot(value.center()),
        "bottom_center": vec3_snapshot(value.bottom_center()),
        "min_position": vec3_snapshot(value.min_position()),
        "max_position": vec3_snapshot(value.max_position()),
    })
}

fn block_hit_result_snapshot(value: Option<BlockHitResult>) -> Value {
    match value {
        Some(hit) => json!({
            "hit": true,
            "type": "BLOCK",
            "location": clip_snapshot(hit.location),
            "direction": hit.direction.name(),
            "block_pos": blockpos_snapshot(hit.block_pos),
            "inside": hit.inside,
            "world_border_hit": hit.world_border_hit,
        }),
        None => json!({"hit": false}),
    }
}

fn shape_box(value: &Value) -> Result<VoxelShape, String> {
    Shapes::box_shape(
        f64_field(value, "min_x")?,
        f64_field(value, "min_y")?,
        f64_field(value, "min_z")?,
        f64_field(value, "max_x")?,
        f64_field(value, "max_y")?,
        f64_field(value, "max_z")?,
    )
    .map_err(|_| "shape minimum must not exceed maximum".to_owned())
}

fn block_collision_fixture(
    request: &Value,
) -> Result<(ClientWorld, BTreeMap<i32, VoxelShape>), String> {
    let mut shapes_by_state = BTreeMap::new();
    for entry in request
        .get("shape_states")
        .ok_or("shape_states must be an array")?
        .as_array()
        .ok_or("shape_states must be an array")?
    {
        let state_id = i32_field(entry, "state_id")?;
        let shape = shape_box(entry.get("shape").ok_or("shape must be an object")?)?;
        shapes_by_state.insert(state_id, shape);
    }

    Ok((block_collision_world_fixture(request)?, shapes_by_state))
}

fn block_collision_world_fixture(request: &Value) -> Result<ClientWorld, String> {
    let mut chunk_blocks = BTreeMap::new();
    for entry in request
        .get("placements")
        .ok_or("placements must be an array")?
        .as_array()
        .ok_or("placements must be an array")?
    {
        let x = i32_field(entry, "x")?;
        let y = i32_field(entry, "y")?;
        let z = i32_field(entry, "z")?;
        if !(0..16).contains(&y) {
            return Err("block collision fixture Y must be in 0..16".into());
        }
        let state_id = i32_field(entry, "state_id")?;
        let blocks = chunk_blocks
            .entry((x.div_euclid(16), z.div_euclid(16)))
            .or_insert_with(|| vec![0; 4_096]);
        let local_x = x.rem_euclid(16) as usize;
        let local_y = y as usize;
        let local_z = z.rem_euclid(16) as usize;
        blocks[local_y * 256 + local_z * 16 + local_x] = state_id;
    }

    let biomes = vec![0; 64];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    for ((chunk_x, chunk_z), blocks) in chunk_blocks {
        world.replace_chunk(
            chunk_x,
            chunk_z,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
    }

    Ok(world)
}

fn entity_collision_candidates(
    request: &Value,
) -> Result<Vec<ClientEntityCollisionCandidate>, String> {
    request
        .get("candidates")
        .ok_or("candidates must be an array")?
        .as_array()
        .ok_or("candidates must be an array")?
        .iter()
        .map(|entry| {
            let candidate = entry
                .as_object()
                .ok_or("entity collision candidate must be an object")?;
            Ok(ClientEntityCollisionCandidate {
                bounding_box: aabb_box(
                    candidate
                        .get("box")
                        .ok_or("candidate box must be an object")?,
                )?,
                is_source: candidate
                    .get("is_source")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                spectator: candidate
                    .get("spectator")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                can_be_collided_with: candidate
                    .get("can_be_collided_with")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                is_pushable: candidate
                    .get("is_pushable")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                same_vehicle: candidate
                    .get("same_vehicle")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            })
        })
        .collect()
}

fn entity_dimensions_snapshot(value: ClientEntityDimensions) -> Value {
    json!({
        "width": format!("{:08x}", value.width().to_bits()),
        "height": format!("{:08x}", value.height().to_bits()),
        "eye_height": format!("{:08x}", value.eye_height().to_bits()),
        "fixed": value.is_fixed(),
    })
}

fn shape_snapshot(value: &VoxelShape) -> Value {
    let mut state = json!({
        "empty": value.is_empty(),
        "boxes": value.to_aabbs().into_iter().map(aabb_snapshot).collect::<Vec<_>>(),
    });
    if let Some(bounds) = value.bounds() {
        state["bounds"] = aabb_snapshot(bounds);
    }
    state
}

fn client_world_border_status_snapshot(status: ClientWorldBorderStatus) -> &'static str {
    match status {
        ClientWorldBorderStatus::Stationary => "STATIONARY",
        ClientWorldBorderStatus::Shrinking => "SHRINKING",
        ClientWorldBorderStatus::Growing => "GROWING",
    }
}

fn client_world_border_snapshot(event: &str, game_time: u64, border: &ClientWorldBorder) -> Value {
    let samples = [0.0_f32, 0.5_f32, 1.0_f32]
        .into_iter()
        .map(|partial_tick| {
            json!({
                "partial_tick": format!("{:08x}", partial_tick.to_bits()),
                "min_x": format!("{:016x}", border.min_x(partial_tick).to_bits()),
                "max_x": format!("{:016x}", border.max_x(partial_tick).to_bits()),
                "min_z": format!("{:016x}", border.min_z(partial_tick).to_bits()),
                "max_z": format!("{:016x}", border.max_z(partial_tick).to_bits()),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "event": event,
        "game_time": game_time,
        "center_x": format!("{:016x}", border.center_x().to_bits()),
        "center_z": format!("{:016x}", border.center_z().to_bits()),
        "absolute_max_size": border.absolute_max_size(),
        "warning_blocks": border.warning_blocks(),
        "warning_time": border.warning_time(),
        "size": format!("{:016x}", border.size().to_bits()),
        "lerp_time": border.lerp_time(),
        "lerp_target": format!("{:016x}", border.lerp_target().to_bits()),
        "status": client_world_border_status_snapshot(border.status()),
        "samples": samples,
        "collision_shape": shape_snapshot(&border.collision_shape()),
    })
}

fn shape_snapshot_with_coords(value: &VoxelShape) -> Value {
    let mut state = shape_snapshot(value);
    state["coords"] = json!({
        "x": value
            .coords(Axis::X)
            .iter()
            .map(|coordinate| format!("{:016x}", coordinate.to_bits()))
            .collect::<Vec<_>>(),
        "y": value
            .coords(Axis::Y)
            .iter()
            .map(|coordinate| format!("{:016x}", coordinate.to_bits()))
            .collect::<Vec<_>>(),
        "z": value
            .coords(Axis::Z)
            .iter()
            .map(|coordinate| format!("{:016x}", coordinate.to_bits()))
            .collect::<Vec<_>>(),
    });
    state
}

fn shape_axis_limits_snapshot(value: &VoxelShape) -> Value {
    json!({
        "min_x": format!("{:016x}", value.min(Axis::X).to_bits()),
        "max_x": format!("{:016x}", value.max(Axis::X).to_bits()),
        "min_y": format!("{:016x}", value.min(Axis::Y).to_bits()),
        "max_y": format!("{:016x}", value.max(Axis::Y).to_bits()),
        "min_z": format!("{:016x}", value.min(Axis::Z).to_bits()),
        "max_z": format!("{:016x}", value.max(Axis::Z).to_bits()),
    })
}

fn shape_result(request: &Value) -> Result<Value, String> {
    match string(request, "kind")? {
        "player_abilities_packet" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let states = inputs
                .iter()
                .map(|value| {
                    let input = value
                        .as_object()
                        .ok_or("player abilities input must be an object")?;
                    let bytes = decode_hex(
                        input
                            .get("hex")
                            .and_then(Value::as_str)
                            .ok_or("hex must be a string")?,
                    )?;
                    let packet = parse_player_abilities_packet(&bytes)
                        .map_err(|error| format!("player abilities packet: {error:?}"))?;
                    let abilities = ClientPlayerAbilities::new(
                        packet.invulnerable,
                        packet.flying,
                        packet.can_fly,
                        packet.can_instabuild,
                        packet.flying_speed,
                        packet.walking_speed,
                    );
                    Ok(json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "invulnerable": abilities.invulnerable(),
                        "flying": abilities.flying(),
                        "may_fly": abilities.may_fly(),
                        "instabuild": abilities.instabuild(),
                        "flying_speed": format!("{:08x}", abilities.flying_speed().to_bits()),
                        "walking_speed": format!("{:08x}", abilities.walking_speed().to_bits()),
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "player_game_mode_event" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let states = inputs
                .iter()
                .map(|value| {
                    let input = value
                        .as_object()
                        .ok_or("player game mode input must be an object")?;
                    let bytes = decode_hex(
                        input
                            .get("hex")
                            .and_then(Value::as_str)
                            .ok_or("hex must be a string")?,
                    )?;
                    let packet = parse_game_event_packet(&bytes)
                        .map_err(|error| format!("game event packet: {error:?}"))?;
                    if packet.event_id != 3 {
                        return Err("expected change-game-mode event".into());
                    }
                    let flying_speed = input
                        .get("initial_flying_speed")
                        .map(|value| f32_value(value, "initial_flying_speed"))
                        .transpose()?
                        .unwrap_or(0.05);
                    let walking_speed = input
                        .get("initial_walking_speed")
                        .map(|value| f32_value(value, "initial_walking_speed"))
                        .transpose()?
                        .unwrap_or(0.1);
                    let abilities = ClientPlayerAbilities::new(
                        false,
                        input
                            .get("initial_flying")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                        false,
                        false,
                        flying_speed,
                        walking_speed,
                    );
                    let mode = ClientPlayerGameMode::from_game_event_param(packet.param);
                    let abilities = mode.update_player_abilities(abilities);
                    Ok(json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "param": format!("{:08x}", packet.param.to_bits()),
                        "mode_id": mode.id(),
                        "mode_name": mode.name(),
                        "is_spectator": mode.is_spectator(),
                        "is_creative": mode.is_creative(),
                        "is_survival": mode.is_survival(),
                        "may_build": abilities.may_build(),
                        "invulnerable": abilities.invulnerable(),
                        "flying": abilities.flying(),
                        "may_fly": abilities.may_fly(),
                        "instabuild": abilities.instabuild(),
                        "flying_speed": format!("{:08x}", abilities.flying_speed().to_bits()),
                        "walking_speed": format!("{:08x}", abilities.walking_speed().to_bits()),
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "single" => {
            let mut value = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let mut events = vec![shape_snapshot(&value)];
            for step in request["steps"]
                .as_array()
                .ok_or("steps must be an array")?
            {
                value = match string(step, "op")? {
                    "move" => value.move_by(
                        f64_field(step, "dx")?,
                        f64_field(step, "dy")?,
                        f64_field(step, "dz")?,
                    ),
                    "move_vec3" => value.move_by_vec3(f64_vec3(
                        step.get("delta").ok_or("delta must be an object")?,
                    )?),
                    "move_block_pos" => value.move_by_block_pos(blockpos(
                        step.get("delta").ok_or("delta must be an object")?,
                    )?),
                    _ => return Err("unknown VoxelShape operation".into()),
                };
                events.push(shape_snapshot(&value));
            }
            Ok(json!({"events": events}))
        }
        "join" => {
            let operation = match string(request, "op")? {
                "or" => BooleanOp::Or,
                "and" => BooleanOp::And,
                "only_first" => BooleanOp::OnlyFirst,
                "only_second" => BooleanOp::OnlySecond,
                "false" => BooleanOp::False,
                "not_same" => BooleanOp::NotSame,
                "second" => BooleanOp::Second,
                "first" => BooleanOp::First,
                _ => return Err("unknown VoxelShape boolean operation".into()),
            };
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let joined = Shapes::join(&first, &second, operation);
            Ok(json!({
                "shape": shape_snapshot(&joined),
                "join_is_not_empty": Shapes::join_is_not_empty(&first, &second, operation),
            }))
        }
        "join_many" => {
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            if entries.is_empty() {
                return Err("join_many needs at least one shape".into());
            }
            let first = shape_box(&entries[0])?;
            let tail = entries[1..]
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let joined = Shapes::or_many(&first, &tail);
            Ok(json!({"shape": shape_snapshot(&joined)}))
        }
        "single_encompassing" => {
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            if entries.is_empty() {
                return Err("single_encompassing needs at least one shape".into());
            }
            let first = shape_box(&entries[0])?;
            let tail = entries[1..]
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let joined = Shapes::or_many(&first, &tail);
            Ok(json!({
                "shape": shape_snapshot(&joined.single_encompassing()),
            }))
        }
        "optimize" => {
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            if entries.is_empty() {
                return Err("optimize needs at least one shape".into());
            }
            let first = shape_box(&entries[0])?;
            let tail = entries[1..]
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let joined = tail.iter().fold(first.clone(), |joined, shape| {
                Shapes::join_unoptimized(&joined, shape, BooleanOp::Or)
            });
            Ok(json!({
                "events": [
                    shape_snapshot_with_coords(&joined),
                    shape_snapshot_with_coords(&joined.optimize()),
                ],
            }))
        }
        "edges" => {
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            if entries.is_empty() {
                return Err("edges needs at least one shape".into());
            }
            let first = shape_box(&entries[0])?;
            let tail = entries[1..]
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let joined = Shapes::or_many(&first, &tail);
            let mut edges = Vec::new();
            joined.for_all_edges(|x1, y1, z1, x2, y2, z2| {
                edges.push(vec![
                    format!("{:016x}", x1.to_bits()),
                    format!("{:016x}", y1.to_bits()),
                    format!("{:016x}", z1.to_bits()),
                    format!("{:016x}", x2.to_bits()),
                    format!("{:016x}", y2.to_bits()),
                    format!("{:016x}", z2.to_bits()),
                ]);
            });
            Ok(json!({"edges": edges}))
        }
        "join_coords" => {
            let operation = match string(request, "op")? {
                "or" => BooleanOp::Or,
                "and" => BooleanOp::And,
                "only_first" => BooleanOp::OnlyFirst,
                "only_second" => BooleanOp::OnlySecond,
                "not_same" => BooleanOp::NotSame,
                "second" => BooleanOp::Second,
                "first" => BooleanOp::First,
                _ => return Err("unknown VoxelShape boolean operation".into()),
            };
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let unoptimized = Shapes::join_unoptimized(&first, &second, operation);
            let optimized = Shapes::join(&first, &second, operation);
            Ok(json!({
                "unoptimized": shape_snapshot_with_coords(&unoptimized),
                "optimized": shape_snapshot_with_coords(&optimized),
            }))
        }
        "boxes" => {
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            if entries.is_empty() {
                return Err("boxes needs at least one shape".into());
            }
            let first = shape_box(&entries[0])?;
            let tail = entries[1..]
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let joined = Shapes::or_many(&first, &tail);
            let mut boxes = Vec::new();
            joined.for_all_boxes(|x1, y1, z1, x2, y2, z2| {
                boxes.push(vec![
                    format!("{:016x}", x1.to_bits()),
                    format!("{:016x}", y1.to_bits()),
                    format!("{:016x}", z1.to_bits()),
                    format!("{:016x}", x2.to_bits()),
                    format!("{:016x}", y2.to_bits()),
                    format!("{:016x}", z2.to_bits()),
                ]);
            });
            Ok(json!({"boxes": boxes}))
        }
        "create_aabb" => {
            let aabb = aabb_box(request.get("aabb").ok_or("aabb must be an object")?)?;
            let shape = Shapes::create_from_aabb(aabb)
                .map_err(|_| "shape minimum must not exceed maximum".to_owned())?;
            Ok(json!({"shape": shape_snapshot(&shape)}))
        }
        "create_coords" => {
            let value = request.get("box").ok_or("box must be an object")?;
            let shape = Shapes::create_from_coords(
                f64_field(value, "min_x")?,
                f64_field(value, "min_y")?,
                f64_field(value, "min_z")?,
                f64_field(value, "max_x")?,
                f64_field(value, "max_y")?,
                f64_field(value, "max_z")?,
            );
            Ok(json!({"shape": shape_snapshot(&shape)}))
        }
        "infinity" => {
            let infinity = Shapes::infinity();
            match string(request, "view")? {
                "snapshot" => Ok(json!({"shape": shape_snapshot(&infinity)})),
                "boxes" => {
                    let mut boxes = Vec::new();
                    infinity.for_all_boxes(|x1, y1, z1, x2, y2, z2| {
                        boxes.push(vec![
                            format!("{:016x}", x1.to_bits()),
                            format!("{:016x}", y1.to_bits()),
                            format!("{:016x}", z1.to_bits()),
                            format!("{:016x}", x2.to_bits()),
                            format!("{:016x}", y2.to_bits()),
                            format!("{:016x}", z2.to_bits()),
                        ]);
                    });
                    Ok(json!({"boxes": boxes}))
                }
                "not_same_self" => Ok(json!({
                    "shape": shape_snapshot(&Shapes::join(
                        &infinity,
                        &infinity,
                        BooleanOp::NotSame,
                    ))
                })),
                _ => Err("unknown infinity view".into()),
            }
        }
        "constants" => match string(request, "name")? {
            "epsilon" => Ok(json!({
                "epsilon": format!("{:016x}", Shapes::EPSILON.to_bits())
            })),
            "big_epsilon" => Ok(json!({
                "big_epsilon": format!("{:016x}", Shapes::BIG_EPSILON.to_bits())
            })),
            "both" => Ok(json!({
                "epsilon": format!("{:016x}", Shapes::EPSILON.to_bits()),
                "big_epsilon": format!("{:016x}", Shapes::BIG_EPSILON.to_bits()),
            })),
            _ => Err("unknown shape constant".into()),
        },
        "axis_limits" => {
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            if entries.is_empty() {
                return Err("axis_limits needs at least one shape".into());
            }
            let first = shape_box(&entries[0])?;
            let tail = entries[1..]
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let joined = Shapes::or_many(&first, &tail);
            Ok(shape_axis_limits_snapshot(&joined))
        }
        "axis_limits_at" => {
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            if entries.is_empty() {
                return Err("axis_limits_at needs at least one shape".into());
            }
            let first = shape_box(&entries[0])?;
            let tail = entries[1..]
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let joined = Shapes::or_many(&first, &tail);
            let values = request["queries"]
                .as_array()
                .ok_or("queries must be an array")?
                .iter()
                .map(|query| {
                    let axis =
                        Axis::from_name(string(query, "axis")?).ok_or("unknown shape axis")?;
                    let b = f64_field(query, "b")?;
                    let c = f64_field(query, "c")?;
                    let value = if string(query, "op")? == "min" {
                        joined.min_at(axis, b, c)
                    } else {
                        joined.max_at(axis, b, c)
                    };
                    Ok(format!("{:016x}", value.to_bits()))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"values": values}))
        }
        "equal" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            Ok(json!({"equal": Shapes::equal(&first, &second)}))
        }
        "object_equals" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let same_geometry = shape_box(request.get("same").ok_or("same must be an object")?)?;
            let different_geometry = shape_box(
                request
                    .get("different")
                    .ok_or("different must be an object")?,
            )?;
            Ok(json!({
                "self": shape.equals_java(&shape),
                "same_geometry": shape.equals_java(&same_geometry),
                "different_geometry": shape.equals_java(&different_geometry),
            }))
        }
        "object_equals_constants" => {
            let empty = Shapes::empty();
            let block = Shapes::block();
            let infinity = Shapes::infinity();
            let empty_thin = Shapes::box_shape(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)
                .map_err(|_| "shape minimum must not exceed maximum".to_owned())?;
            let block_box = Shapes::box_shape(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
                .map_err(|_| "shape minimum must not exceed maximum".to_owned())?;
            let infinity_box = Shapes::box_shape(
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::INFINITY,
                f64::INFINITY,
            )
            .map_err(|_| "shape minimum must not exceed maximum".to_owned())?;
            Ok(json!({
                "empty_self": empty.equals_java(&Shapes::empty()),
                "empty_thin": empty.equals_java(&empty_thin),
                "empty_move": empty.equals_java(&empty.move_by(1.0, 2.0, 3.0)),
                "block_self": block.equals_java(&Shapes::block()),
                "block_box": block.equals_java(&block_box),
                "block_move": block.equals_java(&block.move_by(0.0, 0.0, 0.0)),
                "infinity_self": infinity.equals_java(&Shapes::infinity()),
                "infinity_box": infinity.equals_java(&infinity_box),
            }))
        }
        "to_string" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            Ok(json!({"value": shape.to_string_java()}))
        }
        "rotate" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let rotation = ShapeRotation::from_name(string(request, "rotation")?)
                .ok_or("unknown shape rotation")?;
            Ok(json!({
                "shape": shape_snapshot(&Shapes::rotate(&shape, rotation)),
            }))
        }
        "rotate_center" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let rotation = ShapeRotation::from_name(string(request, "rotation")?)
                .ok_or("unknown shape rotation")?;
            let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
            Ok(json!({
                "shape": shape_snapshot(&Shapes::rotate_with_center(&shape, rotation, center)),
            }))
        }
        "rotate_join" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let rotation = ShapeRotation::from_name(string(request, "rotation")?)
                .ok_or("unknown shape rotation")?;
            let shape = Shapes::or(&first, &second);
            Ok(json!({
                "shape": shape_snapshot(&Shapes::rotate(&shape, rotation)),
            }))
        }
        "rotate_horizontal_axis" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let shapes = Shapes::rotate_horizontal_axis(&shape);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(axis, shape)| json!({
                        "axis": axis.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_horizontal_axis_center" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
            let shapes = Shapes::rotate_horizontal_axis_with_center(&shape, center);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(axis, shape)| json!({
                        "axis": axis.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_all_axis" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let shapes = Shapes::rotate_all_axis(&shape);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(axis, shape)| json!({
                        "axis": axis.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_all_axis_center" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
            let shapes = Shapes::rotate_all_axis_with_center(&shape, center);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(axis, shape)| json!({
                        "axis": axis.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_horizontal" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let shapes = Shapes::rotate_horizontal(&shape);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(direction, shape)| json!({
                        "direction": direction.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_horizontal_initial" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let initial = ShapeRotation::from_name(string(request, "initial")?)
                .ok_or("unknown initial shape rotation")?;
            let shapes = Shapes::rotate_horizontal_with_initial(&shape, initial);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(direction, shape)| json!({
                        "direction": direction.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_all" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let shapes = Shapes::rotate_all(&shape);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(direction, shape)| json!({
                        "direction": direction.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_all_initial" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let initial = ShapeRotation::from_name(string(request, "initial")?)
                .ok_or("unknown initial shape rotation")?;
            let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
            let shapes = Shapes::rotate_all_with_initial(&shape, initial, center);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(direction, shape)| json!({
                        "direction": direction.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_all_center" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let center = f64_vec3(request.get("center").ok_or("center must be an object")?)?;
            let shapes = Shapes::rotate_all_with_center(&shape, center);
            Ok(json!({
                "shapes": shapes
                    .into_iter()
                    .map(|(direction, shape)| json!({
                        "direction": direction.name(),
                        "shape": shape_snapshot(&shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_attach_face" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let faces = Shapes::rotate_attach_face(&shape);
            Ok(json!({
                "faces": faces
                    .into_iter()
                    .map(|(face, shapes)| json!({
                        "face": face.name(),
                        "shapes": shapes
                            .into_iter()
                            .map(|(direction, shape)| json!({
                                "direction": direction.name(),
                                "shape": shape_snapshot(&shape),
                            }))
                            .collect::<Vec<_>>(),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "rotate_attach_face_initial" => {
            let shape = shape_box(request.get("box").ok_or("box must be an object")?)?;
            let initial = ShapeRotation::from_name(string(request, "initial")?)
                .ok_or("unknown initial shape rotation")?;
            let faces = Shapes::rotate_attach_face_with_initial(&shape, initial);
            Ok(json!({
                "faces": faces
                    .into_iter()
                    .map(|(face, shapes)| json!({
                        "face": face.name(),
                        "shapes": shapes
                            .into_iter()
                            .map(|(direction, shape)| json!({
                                "direction": direction.name(),
                                "shape": shape_snapshot(&shape),
                            }))
                            .collect::<Vec<_>>(),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "collide" => {
            let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown collision axis")?;
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let moving = aabb_box(request.get("moving").ok_or("moving must be an object")?)?;
            let distance = f64_field(request, "distance")?;
            Ok(json!({
                "distance": format!("{:016x}", shape.collide(axis, moving, distance).to_bits()),
            }))
        }
        "collide_many" => {
            let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown collision axis")?;
            let moving = aabb_box(request.get("moving").ok_or("moving must be an object")?)?;
            let entries = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?;
            let shapes = entries
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let distance = f64_field(request, "distance")?;
            Ok(json!({
                "distance": format!(
                    "{:016x}",
                    Shapes::collide(axis, moving, &shapes, distance).to_bits()
                ),
            }))
        }
        "collide_movement" => {
            let movement = f64_vec3(
                request
                    .get("movement")
                    .ok_or("movement must be an object")?,
            )?;
            let moving = aabb_box(request.get("moving").ok_or("moving must be an object")?)?;
            let shapes = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(json!({
                "movement": vec3_snapshot(collide_entity_movement(movement, moving, &shapes)),
            }))
        }
        "entity_movement_state" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let states = inputs
                .iter()
                .map(|entry| {
                    let input = entry
                        .as_object()
                        .ok_or("movement state input must be an object")?;
                    let movement =
                        f64_vec3(input.get("movement").ok_or("movement must be an object")?)?;
                    let moving = aabb_box(input.get("moving").ok_or("moving must be an object")?)?;
                    let shapes = input
                        .get("shapes")
                        .ok_or("shapes must be an array")?
                        .as_array()
                        .ok_or("shapes must be an array")?
                        .iter()
                        .map(shape_box)
                        .collect::<Result<Vec<_>, _>>()?;
                    let state = collide_entity_movement_state(movement, moving, &shapes);
                    Ok(json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "movement": vec3_snapshot(state.movement),
                        "x_collision": state.x_collision,
                        "y_collision": state.y_collision,
                        "z_collision": state.z_collision,
                        "horizontal_collision": state.horizontal_collision,
                        "vertical_collision": state.vertical_collision,
                        "vertical_collision_below": state.vertical_collision_below,
                        "on_ground": state.on_ground,
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "entity_movement_state_retention" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let states = inputs
                .iter()
                .map(|entry| {
                    let input = entry
                        .as_object()
                        .ok_or("movement state input must be an object")?;
                    let delta = f64_vec3(input.get("delta").ok_or("delta must be an object")?)?;
                    let moving = aabb_box(input.get("moving").ok_or("moving must be an object")?)?;
                    let shapes = input
                        .get("shapes")
                        .ok_or("shapes must be an array")?
                        .as_array()
                        .ok_or("shapes must be an array")?
                        .iter()
                        .map(shape_box)
                        .collect::<Result<Vec<_>, _>>()?;
                    let resolved = collide_entity_movement(delta, moving, &shapes);
                    let previous_input = input
                        .get("previous")
                        .and_then(Value::as_object)
                        .ok_or("previous must be an object")?;
                    let previous = voxel_world::ClientEntityMovementState {
                        movement: Vec3::ZERO,
                        x_collision: false,
                        y_collision: false,
                        z_collision: false,
                        horizontal_collision: false,
                        vertical_collision: previous_input
                            .get("vertical_collision")
                            .and_then(Value::as_bool)
                            .ok_or("previous.vertical_collision must be a boolean")?,
                        vertical_collision_below: previous_input
                            .get("vertical_collision_below")
                            .and_then(Value::as_bool)
                            .ok_or("previous.vertical_collision_below must be a boolean")?,
                        on_ground: previous_input
                            .get("on_ground")
                            .and_then(Value::as_bool)
                            .ok_or("previous.on_ground must be a boolean")?,
                    };
                    let state = apply_entity_movement_state(
                        delta,
                        resolved,
                        previous,
                        input
                            .get("authoritative")
                            .and_then(Value::as_bool)
                            .ok_or("authoritative must be a boolean")?,
                    );
                    Ok(json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "movement": vec3_snapshot(state.movement),
                        "x_collision": state.x_collision,
                        "y_collision": state.y_collision,
                        "z_collision": state.z_collision,
                        "horizontal_collision": state.horizontal_collision,
                        "vertical_collision": state.vertical_collision,
                        "vertical_collision_below": state.vertical_collision_below,
                        "on_ground": state.on_ground,
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "entity_collision_shapes" => {
            let source_present = request
                .get("source_present")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let test_area = aabb_box(
                request
                    .get("test_area")
                    .ok_or("test_area must be an object")?,
            )?;
            let candidates = entity_collision_candidates(request)?;
            let shapes = collect_entity_collision_shapes(source_present, test_area, &candidates);
            Ok(json!({
                "shapes": shapes.iter().map(shape_snapshot).collect::<Vec<_>>(),
            }))
        }
        "client_world_collision" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let states = inputs
                .iter()
                .map(|entry| {
                    let input = entry
                        .as_object()
                        .ok_or("client world collision input must be an object")?;
                    let test_area = aabb_box(
                        input
                            .get("test_area")
                            .ok_or("test_area must be an object")?,
                    )?;
                    let (world, shapes_by_state) = block_collision_fixture(entry)?;
                    let source = if input
                        .get("source_present")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                    {
                        Some(ClientWorldCollisionSource {
                            position: f64_vec3(
                                input
                                    .get("source_position")
                                    .ok_or("source_position must be an object")?,
                            )?,
                        })
                    } else {
                        None
                    };
                    let border = input.get("border").ok_or("border must be an object")?;
                    let size = f64_field(border, "size")?;
                    let mut world = world;
                    world.apply_world_border_initialize(ClientWorldBorderInitialization {
                        center_x: f64_field(border, "center_x")?,
                        center_z: f64_field(border, "center_z")?,
                        old_size: size,
                        new_size: size,
                        lerp_time: 0,
                        absolute_max_size: i32_field(border, "absolute_max_size")?,
                        warning_blocks: 5,
                        warning_time: 15,
                    });
                    let candidates = entity_collision_candidates(entry)?;
                    Ok(json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "no_block_collision": world.no_block_collision(
                            test_area,
                            &shapes_by_state,
                        ),
                        "no_entity_collision": world.no_entity_collision(
                            source.is_some(),
                            test_area,
                            &candidates,
                        ),
                        "no_border_collision": world.no_border_collision(source, test_area),
                        "no_collision": world.no_collision(
                            source,
                            test_area,
                            &shapes_by_state,
                            &candidates,
                        ),
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "client_world_indexed_collision" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let states = inputs
                .iter()
                .map(|entry| {
                    let input = entry
                        .as_object()
                        .ok_or("indexed collision input must be an object")?;
                    let test_area = aabb_box(
                        input
                            .get("test_area")
                            .ok_or("test_area must be an object")?,
                    )?;
                    let (mut world, shapes_by_state) = block_collision_fixture(entry)?;
                    for entity_value in input
                        .get("entities")
                        .ok_or("entities must be an array")?
                        .as_array()
                        .ok_or("entities must be an array")?
                    {
                        let entity = entity_value
                            .as_object()
                            .ok_or("indexed entity must be an object")?;
                        let entity_id = i32_field(entity_value, "entity_id")?;
                        let position =
                            f64_vec3(entity.get("position").ok_or("position must be an object")?)?;
                        world.apply_entity(EntityInput {
                            entity_id,
                            uuid: Uuid::new(0, u64::from(entity_id as u32)),
                            type_id: i32_field(entity_value, "type_id")?,
                            x: position.x,
                            y: position.y,
                            z: position.z,
                            movement: (0.0, 0.0, 0.0),
                            rotation: (0.0, 0.0, 0.0),
                            data: 0,
                        });
                        world.apply_entity_living_collision_state(
                            entity_id,
                            entity.get("alive").and_then(Value::as_bool).unwrap_or(true),
                            entity
                                .get("spectator")
                                .and_then(Value::as_bool)
                                .unwrap_or(false),
                            entity
                                .get("on_climbable")
                                .and_then(Value::as_bool)
                                .unwrap_or(false),
                        );
                        world.apply_entity_player_flying_state(
                            entity_id,
                            entity
                                .get("flying")
                                .and_then(Value::as_bool)
                                .unwrap_or(false),
                        );
                        if let Some(context) =
                            entity.get("climbable_context").and_then(Value::as_object)
                        {
                            let kind = context
                                .get("kind")
                                .and_then(Value::as_str)
                                .ok_or("climbable context kind must be a string")?;
                            if kind == "trapdoor_ladder" {
                                world.apply_entity_living_climbable_trapdoor_state(
                                    entity_id,
                                    context
                                        .get("trapdoor_open")
                                        .and_then(Value::as_bool)
                                        .unwrap_or(false),
                                    context
                                        .get("trapdoor_facing")
                                        .and_then(Value::as_str)
                                        .ok_or("trapdoor_facing must be a string")?,
                                    context
                                        .get("below_is_ladder")
                                        .and_then(Value::as_bool)
                                        .unwrap_or(true),
                                    context
                                        .get("ladder_facing")
                                        .and_then(Value::as_str)
                                        .ok_or("ladder_facing must be a string")?,
                                );
                            } else if kind == "climbable_tag" {
                                let state_id = context
                                    .get("state_id")
                                    .and_then(Value::as_i64)
                                    .ok_or("climbable state_id must be an integer")?
                                    .try_into()
                                    .map_err(|_| "climbable state_id must fit in i32")?;
                                world
                                    .apply_entity_living_climbable_block_state(entity_id, state_id);
                            } else if kind == "glide_through_tag" {
                                let state_id = context
                                    .get("state_id")
                                    .and_then(Value::as_i64)
                                    .ok_or("glide-through state_id must be an integer")?
                                    .try_into()
                                    .map_err(|_| "glide-through state_id must fit in i32")?;
                                let fall_flying = context
                                    .get("fall_flying")
                                    .and_then(Value::as_bool)
                                    .unwrap_or(false);
                                world.apply_entity_living_climbable_glide_through_state(
                                    entity_id,
                                    state_id,
                                    fall_flying,
                                );
                            } else if kind == "update_tags_packet" {
                                let state_id = context
                                    .get("state_id")
                                    .and_then(Value::as_i64)
                                    .ok_or("update-tags state_id must be an integer")?
                                    .try_into()
                                    .map_err(|_| "update-tags state_id must fit in i32")?;
                                let block_tags = context
                                    .get("block_tags")
                                    .and_then(Value::as_object)
                                    .ok_or("block_tags must be an object")?;
                                let tags = block_tags
                                    .iter()
                                    .map(|(tag_id, values)| {
                                        let values = values
                                            .as_array()
                                            .ok_or("block tag values must be an array")?
                                            .iter()
                                            .map(|value| {
                                                value
                                                    .as_i64()
                                                    .ok_or("block tag value must be an integer")?
                                                    .try_into()
                                                    .map_err(|_| {
                                                        "block tag value must fit in i32".to_owned()
                                                    })
                                            })
                                            .collect::<Result<Vec<i32>, String>>()?;
                                        let id = Identifier::parse(tag_id).map_err(|_| {
                                            "invalid block tag identifier".to_owned()
                                        })?;
                                        Ok(TagEntry::new(id, values))
                                    })
                                    .collect::<Result<Vec<_>, String>>()?;
                                let registry = Identifier::parse("minecraft:block")
                                    .map_err(|_| "invalid block registry identifier".to_owned())?;
                                let packet =
                                    ClientboundUpdateTagsPacket::new(vec![RegistryTags::new(
                                        registry, tags,
                                    )]);
                                world.apply_update_tags_packet(&packet);
                                let fall_flying = context
                                    .get("fall_flying")
                                    .and_then(Value::as_bool)
                                    .unwrap_or(false);
                                world.apply_entity_living_climbable_glide_through_state(
                                    entity_id,
                                    state_id,
                                    fall_flying,
                                );
                            } else {
                                return Err("unknown climbable context kind".to_owned());
                            }
                        }
                    }
                    let source_entity_id = input
                        .get("source_entity_id")
                        .filter(|value| !value.is_null())
                        .map(|value| {
                            value
                                .as_i64()
                                .ok_or_else(|| "source_entity_id must be an integer".to_owned())
                                .and_then(|value| {
                                    value
                                        .try_into()
                                        .map_err(|_| "source_entity_id must fit in i32".to_owned())
                                })
                        })
                        .transpose()?;
                    let border = input.get("border").ok_or("border must be an object")?;
                    let size = f64_field(border, "size")?;
                    world.apply_world_border_initialize(ClientWorldBorderInitialization {
                        center_x: f64_field(border, "center_x")?,
                        center_z: f64_field(border, "center_z")?,
                        old_size: size,
                        new_size: size,
                        lerp_time: 0,
                        absolute_max_size: i32_field(border, "absolute_max_size")?,
                        warning_blocks: 5,
                        warning_time: 15,
                    });
                    Ok(json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "entity_count": world.loaded_entity_count(),
                        "no_entity_collision": world.no_entity_collision_for_entity(
                            source_entity_id,
                            test_area,
                        ),
                        "no_collision": world.no_collision_for_entity(
                            source_entity_id,
                            test_area,
                            &shapes_by_state,
                        ),
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "client_world_happy_ghast_collision" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let states = inputs
                .iter()
                .map(|entry| {
                    let input = entry
                        .as_object()
                        .ok_or("Happy Ghast collision input must be an object")?;
                    let test_area = aabb_box(
                        input
                            .get("test_area")
                            .ok_or("test_area must be an object")?,
                    )?;
                    let (mut world, shapes_by_state) = block_collision_fixture(entry)?;
                    for entity_value in input
                        .get("entities")
                        .ok_or("entities must be an array")?
                        .as_array()
                        .ok_or("entities must be an array")?
                    {
                        let entity = entity_value
                            .as_object()
                            .ok_or("indexed entity must be an object")?;
                        let entity_id = i32_field(entity_value, "entity_id")?;
                        let type_id = i32_field(entity_value, "type_id")?;
                        let position =
                            f64_vec3(entity.get("position").ok_or("position must be an object")?)?;
                        world.apply_entity(EntityInput {
                            entity_id,
                            uuid: Uuid::new(0, u64::from(entity_id as u32)),
                            type_id,
                            x: position.x,
                            y: position.y,
                            z: position.z,
                            movement: (0.0, 0.0, 0.0),
                            rotation: (0.0, 0.0, 0.0),
                            data: 0,
                        });
                        if type_id == 58 {
                            world.apply_happy_ghast_collision_state(
                                entity_id,
                                entity.get("baby").and_then(Value::as_bool).unwrap_or(false),
                                entity.get("alive").and_then(Value::as_bool).unwrap_or(true),
                                entity
                                    .get("still_timeout")
                                    .and_then(Value::as_bool)
                                    .unwrap_or(false),
                                entity
                                    .get("is_vehicle")
                                    .and_then(Value::as_bool)
                                    .unwrap_or(false),
                            );
                        }
                    }
                    let source_entity_id = input
                        .get("source_entity_id")
                        .filter(|value| !value.is_null())
                        .map(|value| {
                            value
                                .as_i64()
                                .ok_or_else(|| "source_entity_id must be an integer".to_owned())
                                .and_then(|value| {
                                    value
                                        .try_into()
                                        .map_err(|_| "source_entity_id must fit in i32".to_owned())
                                })
                        })
                        .transpose()?;
                    let border = input.get("border").ok_or("border must be an object")?;
                    let size = f64_field(border, "size")?;
                    world.apply_world_border_initialize(ClientWorldBorderInitialization {
                        center_x: f64_field(border, "center_x")?,
                        center_z: f64_field(border, "center_z")?,
                        old_size: size,
                        new_size: size,
                        lerp_time: 0,
                        absolute_max_size: i32_field(border, "absolute_max_size")?,
                        warning_blocks: 5,
                        warning_time: 15,
                    });
                    Ok(json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "entity_count": world.loaded_entity_count(),
                        "no_entity_collision": world.no_entity_collision_for_entity(
                            source_entity_id,
                            test_area,
                        ),
                        "no_collision": world.no_collision_for_entity(
                            source_entity_id,
                            test_area,
                            &shapes_by_state,
                        ),
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "client_world_entity_section_index" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let tracks = inputs
                .iter()
                .map(|entry| {
                    let input = entry
                        .as_object()
                        .ok_or("entity section input must be an object")?;
                    let (mut world, shapes_by_state) = block_collision_fixture(entry)?;
                    for entity_value in input
                        .get("entities")
                        .ok_or("entities must be an array")?
                        .as_array()
                        .ok_or("entities must be an array")?
                    {
                        let entity = entity_value
                            .as_object()
                            .ok_or("indexed entity must be an object")?;
                        let position =
                            f64_vec3(entity.get("position").ok_or("position must be an object")?)?;
                        world.apply_entity(EntityInput {
                            entity_id: i32_field(entity_value, "entity_id")?,
                            uuid: Uuid::new(
                                0,
                                u64::from(i32_field(entity_value, "entity_id")? as u32),
                            ),
                            type_id: i32_field(entity_value, "type_id")?,
                            x: position.x,
                            y: position.y,
                            z: position.z,
                            movement: (0.0, 0.0, 0.0),
                            rotation: (0.0, 0.0, 0.0),
                            data: 0,
                        });
                    }
                    let source_entity_id = input
                        .get("source_entity_id")
                        .filter(|value| !value.is_null())
                        .map(|value| {
                            value
                                .as_i64()
                                .ok_or_else(|| "source_entity_id must be an integer".to_owned())
                                .and_then(|value| {
                                    value
                                        .try_into()
                                        .map_err(|_| "source_entity_id must fit in i32".to_owned())
                                })
                        })
                        .transpose()?;
                    let border = input.get("border").ok_or("border must be an object")?;
                    let size = f64_field(border, "size")?;
                    world.apply_world_border_initialize(ClientWorldBorderInitialization {
                        center_x: f64_field(border, "center_x")?,
                        center_z: f64_field(border, "center_z")?,
                        old_size: size,
                        new_size: size,
                        lerp_time: 0,
                        absolute_max_size: i32_field(border, "absolute_max_size")?,
                        warning_blocks: 5,
                        warning_time: 15,
                    });
                    let mut states = Vec::new();
                    for event_value in input
                        .get("events")
                        .ok_or("events must be an array")?
                        .as_array()
                        .ok_or("events must be an array")?
                    {
                        let event = event_value
                            .as_object()
                            .ok_or("entity section event must be an object")?;
                        match event
                            .get("op")
                            .and_then(Value::as_str)
                            .ok_or("entity section event op must be a string")?
                        {
                            "move" => {
                                let position = f64_vec3(
                                    event.get("position").ok_or("position must be an object")?,
                                )?;
                                let entity_id = i32_field(event_value, "entity_id")?;
                                if !world.update_entity_position(
                                    entity_id,
                                    (position.x, position.y, position.z),
                                ) {
                                    return Err("cannot move missing entity".into());
                                }
                            }
                            "remove" => {
                                world.remove_entity(i32_field(event_value, "entity_id")?);
                            }
                            "add" => {
                                let entity_value =
                                    event.get("entity").ok_or("add entity must be an object")?;
                                let entity = entity_value
                                    .as_object()
                                    .ok_or("add entity must be an object")?;
                                let position = f64_vec3(
                                    entity.get("position").ok_or("position must be an object")?,
                                )?;
                                let entity_id = i32_field(entity_value, "entity_id")?;
                                world.apply_entity(EntityInput {
                                    entity_id,
                                    uuid: Uuid::new(0, u64::from(entity_id as u32)),
                                    type_id: i32_field(entity_value, "type_id")?,
                                    x: position.x,
                                    y: position.y,
                                    z: position.z,
                                    movement: (0.0, 0.0, 0.0),
                                    rotation: (0.0, 0.0, 0.0),
                                    data: 0,
                                });
                            }
                            "query" => {
                                let test_area = aabb_box(
                                    event
                                        .get("test_area")
                                        .ok_or("test_area must be an object")?,
                                )?;
                                states.push(json!({
                                    "label": event
                                        .get("label")
                                        .and_then(Value::as_str)
                                        .ok_or("label must be a string")?,
                                    "entity_count": world.loaded_entity_count(),
                                    "section_count": world.indexed_entity_section_count(),
                                    "no_entity_collision": world.no_entity_collision_for_entity(
                                        source_entity_id,
                                        test_area,
                                    ),
                                    "no_collision": world.no_collision_for_entity(
                                        source_entity_id,
                                        test_area,
                                        &shapes_by_state,
                                    ),
                                }));
                            }
                            _ => return Err("unknown entity section event".into()),
                        }
                    }
                    Ok(json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "states": states,
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"tracks": tracks}))
        }
        "client_world_stair_neighbor_collision" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let states = inputs
                .iter()
                .map(|entry| {
                    let input = entry
                        .as_object()
                        .ok_or("stair neighbor collision input must be an object")?;
                    let target = input
                        .get("target")
                        .and_then(Value::as_object)
                        .ok_or("stair target must be an object")?;
                    let x = i32_field(&Value::Object(target.clone()), "x")?;
                    let y = i32_field(&Value::Object(target.clone()), "y")?;
                    let z = i32_field(&Value::Object(target.clone()), "z")?;
                    let mut world = block_collision_world_fixture(entry)?;
                    let before_state_id = world
                        .block_state_at(x, y, z)
                        .ok_or("stair target must be loaded")?;
                    if !world.apply_stair_neighbor_shape_update(x, y, z) {
                        return Err("stair neighbor shape update was not applicable".to_owned());
                    }
                    let updated_state_id = world
                        .block_state_at(x, y, z)
                        .ok_or("updated stair target must remain loaded")?;
                    let collision_shape = world
                        .block_state_registry()
                        .collision_shape(updated_state_id);
                    let updated_shape = world
                        .block_state_registry()
                        .stair_shape_name(updated_state_id)
                        .ok_or("updated state must be a supported stair")?;
                    let test_area = aabb_box(
                        input
                            .get("test_area")
                            .ok_or("test_area must be an object")?,
                    )?;
                    let collisions = world.collect_registered_block_collision_shapes(test_area);
                    Ok(json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "before_state_id": before_state_id,
                        "updated_state_id": updated_state_id,
                        "updated_shape": updated_shape,
                        "updated_collision_shape": shape_snapshot(&collision_shape),
                        "collisions": collisions
                            .iter()
                            .map(|collision| shape_snapshot(&collision.shape))
                            .collect::<Vec<_>>(),
                        "no_block_collision": collisions.is_empty(),
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "local_player_minor_horizontal_collision" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let states = inputs
                .iter()
                .map(|entry| {
                    let input = entry
                        .as_object()
                        .ok_or("minor collision input must be an object")?;
                    let movement = f64_vec3(
                        input
                            .get("movement")
                            .ok_or("movement must be an object")?,
                    )?;
                    Ok(json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "minor_horizontal_collision": is_minor_horizontal_collision(
                            f32_value(
                                input.get("y_rot").ok_or("y_rot must be an f32 bit string")?,
                                "y_rot",
                            )?,
                            f32_value(input.get("xxa").ok_or("xxa must be an f32 bit string")?, "xxa")?,
                            f32_value(input.get("zza").ok_or("zza must be an f32 bit string")?, "zza")?,
                            movement,
                        ),
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "client_world_registry_collision" => {
            let input = request
                .as_object()
                .ok_or("registry collision input must be an object")?;
            let test_area = aabb_box(
                input
                    .get("test_area")
                    .ok_or("test_area must be an object")?,
            )?;
            let world = block_collision_world_fixture(request)?;
            let states = input
                .get("state_ids")
                .ok_or("state_ids must be an array")?
                .as_array()
                .ok_or("state_ids must be an array")?
                .iter()
                .map(|state_value| {
                    let requested_state_id = state_value
                        .as_i64()
                        .ok_or("state ID must be an integer")?
                        .try_into()
                        .map_err(|_| "state ID must fit in i32".to_owned())?;
                    let registry = world.block_state_registry();
                    let resolved_state_id = registry.resolve_state_id(requested_state_id);
                    let collision_shape = registry.collision_shape(requested_state_id);
                    Ok(json!({
                        "requested_state_id": requested_state_id,
                        "resolved_state_id": resolved_state_id,
                        "collision_shape": shape_snapshot(&collision_shape),
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let collisions = world.collect_registered_block_collision_shapes(test_area);
            Ok(json!({
                "states": states,
                "collisions": collisions
                    .iter()
                    .map(|collision| shape_snapshot(&collision.shape))
                    .collect::<Vec<_>>(),
                "no_block_collision": collisions.is_empty(),
            }))
        }
        "world_border_collision_shape" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let states = inputs
                .iter()
                .map(|entry| {
                    let input = entry
                        .as_object()
                        .ok_or("world border input must be an object")?;
                    let center_x = f64_value(
                        input
                            .get("center_x")
                            .ok_or("center_x must be a bit string")?,
                        "center_x",
                    )?;
                    let center_z = f64_value(
                        input
                            .get("center_z")
                            .ok_or("center_z must be a bit string")?,
                        "center_z",
                    )?;
                    let size = f64_value(
                        input.get("size").ok_or("size must be a bit string")?,
                        "size",
                    )?;
                    let absolute_max_size = input
                        .get("absolute_max_size")
                        .and_then(Value::as_i64)
                        .and_then(|value| i32::try_from(value).ok())
                        .ok_or("absolute_max_size must be an i32")?;
                    Ok(shape_snapshot(&world_border_collision_shape(
                        center_x,
                        center_z,
                        size,
                        absolute_max_size,
                    )))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "world_border_collision_shape_moving" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let states = inputs
                .iter()
                .map(|entry| {
                    let input = entry
                        .as_object()
                        .ok_or("world border input must be an object")?;
                    let center_x = f64_value(
                        input
                            .get("center_x")
                            .ok_or("center_x must be a bit string")?,
                        "center_x",
                    )?;
                    let center_z = f64_value(
                        input
                            .get("center_z")
                            .ok_or("center_z must be a bit string")?,
                        "center_z",
                    )?;
                    let from = f64_value(
                        input.get("from").ok_or("from must be a bit string")?,
                        "from",
                    )?;
                    let to = f64_value(
                        input.get("to").ok_or("to must be a bit string")?,
                        "to",
                    )?;
                    let absolute_max_size = input
                        .get("absolute_max_size")
                        .and_then(Value::as_i64)
                        .and_then(|value| i32::try_from(value).ok())
                        .ok_or("absolute_max_size must be an i32")?;
                    let duration = input
                        .get("duration")
                        .and_then(Value::as_i64)
                        .ok_or("duration must be an i64")?;
                    let tick_count = input
                        .get("tick_count")
                        .and_then(Value::as_i64)
                        .filter(|value| *value >= 0)
                        .ok_or("tick_count must be a non-negative i64")?;
                    let partial_ticks = input
                        .get("partial_ticks")
                        .ok_or("partial_ticks must be an array")?
                        .as_array()
                        .ok_or("partial_ticks must be an array")?
                        .iter()
                        .map(|value| f32_value(value, "partial_tick"))
                        .collect::<Result<Vec<_>, String>>()?;
                    let mut border = ClientWorldBorder::new(
                        center_x,
                        center_z,
                        absolute_max_size,
                        from,
                    );
                    border.lerp_size_between(from, to, duration);
                    let mut states = Vec::new();
                    for tick in 0..=tick_count {
                        let samples = partial_ticks
                            .iter()
                            .map(|partial_tick| {
                                json!({
                                    "partial_tick": format!("{:08x}", partial_tick.to_bits()),
                                    "min_x": format!("{:016x}", border.min_x(*partial_tick).to_bits()),
                                    "max_x": format!("{:016x}", border.max_x(*partial_tick).to_bits()),
                                    "min_z": format!("{:016x}", border.min_z(*partial_tick).to_bits()),
                                    "max_z": format!("{:016x}", border.max_z(*partial_tick).to_bits()),
                                })
                            })
                            .collect::<Vec<_>>();
                        let status = match border.status() {
                            ClientWorldBorderStatus::Stationary => "STATIONARY",
                            ClientWorldBorderStatus::Shrinking => "SHRINKING",
                            ClientWorldBorderStatus::Growing => "GROWING",
                        };
                        states.push(json!({
                            "size": format!("{:016x}", border.size().to_bits()),
                            "lerp_time": border.lerp_time(),
                            "lerp_target": format!("{:016x}", border.lerp_target().to_bits()),
                            "status": status,
                            "samples": samples,
                            "collision_shape": shape_snapshot(&border.collision_shape()),
                        }));
                        if tick < tick_count {
                            border.tick();
                        }
                    }
                    Ok(json!(states))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "client_world_border_state" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let tracks = inputs
                .iter()
                .map(|entry| {
                    let events = entry
                        .get("events")
                        .ok_or("events must be an array")?
                        .as_array()
                        .ok_or("events must be an array")?;
                    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
                    let mut states = Vec::new();
                    for event in events {
                        let operation = string(event, "op")?;
                        match operation {
                            "initialize" => world.apply_world_border_initialize(
                                ClientWorldBorderInitialization {
                                    center_x: f64_field(event, "center_x")?,
                                    center_z: f64_field(event, "center_z")?,
                                    old_size: f64_field(event, "old_size")?,
                                    new_size: f64_field(event, "new_size")?,
                                    lerp_time: i64_field(event, "lerp_time")?,
                                    absolute_max_size: i32_field(event, "absolute_max_size")?,
                                    warning_blocks: i32_field(event, "warning_blocks")?,
                                    warning_time: i32_field(event, "warning_time")?,
                                },
                            ),
                            "center" => world.apply_world_border_center(
                                f64_field(event, "center_x")?,
                                f64_field(event, "center_z")?,
                            ),
                            "lerp" => world.apply_world_border_lerp_size(
                                f64_field(event, "old_size")?,
                                f64_field(event, "new_size")?,
                                i64_field(event, "lerp_time")?,
                            ),
                            "size" => world.apply_world_border_size(f64_field(event, "size")?),
                            "warning_blocks" => {
                                world.apply_world_border_warning_blocks(i32_field(event, "value")?)
                            }
                            "warning_time" => {
                                world.apply_world_border_warning_time(i32_field(event, "value")?)
                            }
                            "tick" => world.tick(),
                            _ => return Err(format!("unknown world border event: {operation}")),
                        }
                        states.push(client_world_border_snapshot(
                            operation,
                            world.game_time(),
                            world.world_border(),
                        ));
                    }
                    Ok(json!(states))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"tracks": tracks}))
        }
        "collect_step_up_heights" => {
            let moving = aabb_box(request.get("moving").ok_or("moving must be an object")?)?;
            let shapes = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let candidates = collect_candidate_step_up_heights(
                moving,
                &shapes,
                f32_field(request, "max_step_height")?,
                f32_field(request, "step_height_to_skip")?,
            );
            Ok(json!({
                "candidates": candidates
                    .into_iter()
                    .map(|value| format!("{:08x}", value.to_bits()))
                .collect::<Vec<_>>(),
            }))
        }
        "select_step_up_movement" => {
            let bounding_box = aabb_box(
                request
                    .get("bounding_box")
                    .ok_or("bounding_box must be an object")?,
            )?;
            let grounded_box = aabb_box(
                request
                    .get("grounded_box")
                    .ok_or("grounded_box must be an object")?,
            )?;
            let movement = f64_vec3(
                request
                    .get("movement")
                    .ok_or("movement must be an object")?,
            )?;
            let movement_step = f64_vec3(
                request
                    .get("movement_step")
                    .ok_or("movement_step must be an object")?,
            )?;
            let shapes = request
                .get("shapes")
                .ok_or("shapes must be an array")?
                .as_array()
                .ok_or("shapes must be an array")?
                .iter()
                .map(shape_box)
                .collect::<Result<Vec<_>, _>>()?;
            let selected = select_step_up_movement(
                movement,
                movement_step,
                bounding_box,
                grounded_box,
                &shapes,
                f32_field(request, "max_step_height")?,
                f32_field(request, "step_height_to_skip")?,
            );
            Ok(json!({"movement": vec3_snapshot(selected)}))
        }
        "collect_block_collisions" => {
            let bounding_box = aabb_box(request.get("box").ok_or("box must be an object")?)?;
            let (world, shapes_by_state) = block_collision_fixture(request)?;
            let collisions = world.collect_block_collision_shapes(bounding_box, &shapes_by_state);
            Ok(json!({
                "collisions": collisions
                    .iter()
                    .map(|collision| json!({
                        "position": blockpos_snapshot(collision.position),
                        "shape": shape_snapshot(&collision.shape),
                    }))
                    .collect::<Vec<_>>(),
            }))
        }
        "entity_fit" => {
            let position = f64_vec3(
                request
                    .get("position")
                    .ok_or("position must be an object")?,
            )?;
            let dimension_input = request
                .get("dimensions")
                .ok_or("dimensions must be an object")?;
            let width = f32_field(dimension_input, "width")?;
            let height = f32_field(dimension_input, "height")?;
            let fixed = dimension_input
                .get("fixed")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let mut dimensions = if fixed {
                ClientEntityDimensions::fixed(width, height)
            } else {
                ClientEntityDimensions::scalable(width, height)
            };
            if dimension_input.get("width_scale").is_some()
                || dimension_input.get("height_scale").is_some()
            {
                let width_scale = dimension_input
                    .get("width_scale")
                    .map(|_| f32_field(dimension_input, "width_scale"))
                    .transpose()?
                    .unwrap_or(1.0);
                let height_scale = dimension_input
                    .get("height_scale")
                    .map(|_| f32_field(dimension_input, "height_scale"))
                    .transpose()?
                    .unwrap_or(1.0);
                dimensions = dimensions.scale(width_scale, height_scale);
            }
            if dimension_input.get("eye_height").is_some() {
                dimensions = dimensions.with_eye_height(f32_field(dimension_input, "eye_height")?);
            }
            let bounding_box = dimensions.make_bounding_box(position);
            let query_box = bounding_box.deflate(Shapes::EPSILON);
            let (world, shapes_by_state) = block_collision_fixture(request)?;
            Ok(json!({
                "dimensions": entity_dimensions_snapshot(dimensions),
                "box": aabb_snapshot(bounding_box),
                "query": aabb_snapshot(query_box),
                "fits": world.entity_fits_in_blocks(position, dimensions, &shapes_by_state),
            }))
        }
        "living_entity_dimensions_scale" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let states = inputs
                .iter()
                .map(|value| {
                    let input = value
                        .as_object()
                        .ok_or("living dimensions input must be an object")?;
                    let pose_id = input
                        .get("pose_id")
                        .and_then(Value::as_i64)
                        .ok_or("pose_id must be an integer")?
                        .try_into()
                        .map_err(|_| "pose_id must fit in i32".to_owned())?;
                    let pose = ClientEntityPose::from_id(pose_id);
                    let dimension_input = input
                        .get("default_dimensions")
                        .ok_or("default_dimensions must be an object")?;
                    let width = f32_field(dimension_input, "width")?;
                    let height = f32_field(dimension_input, "height")?;
                    let fixed = dimension_input
                        .get("fixed")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    let mut default_dimensions = if fixed {
                        ClientEntityDimensions::fixed(width, height)
                    } else {
                        ClientEntityDimensions::scalable(width, height)
                    };
                    if dimension_input.get("eye_height").is_some() {
                        default_dimensions = default_dimensions
                            .with_eye_height(f32_field(dimension_input, "eye_height")?);
                    }
                    let dimensions = living_entity_dimensions(
                        pose,
                        default_dimensions,
                        f32_field(value, "age_scale")?,
                        f32_field(value, "entity_scale")?,
                    );
                    let position =
                        f64_vec3(input.get("position").ok_or("position must be an object")?)?;
                    Ok(json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "requested_pose_id": pose_id,
                        "resolved_pose_id": pose.id(),
                        "dimensions": entity_dimensions_snapshot(dimensions),
                        "box": aabb_snapshot(dimensions.make_bounding_box(position)),
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "entity_dimensions_refresh" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let states = inputs
                .iter()
                .map(|value| {
                    let input = value
                        .as_object()
                        .ok_or("dimensions refresh input must be an object")?;
                    let dimensions = |name: &str| -> Result<ClientEntityDimensions, String> {
                        let dimension_input = input
                            .get(name)
                            .ok_or_else(|| format!("{name} must be an object"))?;
                        let width = f32_field(dimension_input, "width")?;
                        let height = f32_field(dimension_input, "height")?;
                        let fixed = dimension_input
                            .get("fixed")
                            .and_then(Value::as_bool)
                            .unwrap_or(false);
                        let dimensions = if fixed {
                            ClientEntityDimensions::fixed(width, height)
                        } else {
                            ClientEntityDimensions::scalable(width, height)
                        };
                        Ok(if dimension_input.get("eye_height").is_some() {
                            dimensions.with_eye_height(f32_field(dimension_input, "eye_height")?)
                        } else {
                            dimensions
                        })
                    };
                    let old_dimensions = dimensions("old_dimensions")?;
                    let new_dimensions = dimensions("new_dimensions")?;
                    let position =
                        f64_vec3(input.get("position").ok_or("position must be an object")?)?;
                    let refreshed = refresh_entity_dimensions(
                        old_dimensions,
                        new_dimensions,
                        position,
                        input
                            .get("level_client_side")
                            .and_then(Value::as_bool)
                            .ok_or("level_client_side must be a boolean")?,
                        input
                            .get("first_tick")
                            .and_then(Value::as_bool)
                            .ok_or("first_tick must be a boolean")?,
                        input
                            .get("no_physics")
                            .and_then(Value::as_bool)
                            .ok_or("no_physics must be a boolean")?,
                        input
                            .get("is_player")
                            .and_then(Value::as_bool)
                            .ok_or("is_player must be a boolean")?,
                    );
                    Ok(json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "old_dimensions": entity_dimensions_snapshot(old_dimensions),
                        "new_dimensions": entity_dimensions_snapshot(refreshed.dimensions),
                        "position_after_refresh": vec3_snapshot(position),
                        "bounding_box": aabb_snapshot(refreshed.bounding_box),
                        "should_fudge_position": refreshed.should_fudge_position,
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"states": states}))
        }
        "player_pose_dimensions" => {
            let position = f64_vec3(
                request
                    .get("position")
                    .ok_or("position must be an object")?,
            )?;
            let pose_ids = request
                .get("pose_ids")
                .ok_or("pose_ids must be an array")?
                .as_array()
                .ok_or("pose_ids must be an array")?;
            let poses = pose_ids
                .iter()
                .map(|value| {
                    let requested_id = value
                        .as_i64()
                        .ok_or_else(|| "pose ID must be an integer".to_owned())?
                        .try_into()
                        .map_err(|_| "pose ID must fit in i32".to_owned())?;
                    let pose = ClientEntityPose::from_id(requested_id);
                    let dimensions = pose.player_dimensions();
                    Ok(json!({
                        "requested_id": requested_id,
                        "resolved_id": pose.id(),
                        "dimensions": entity_dimensions_snapshot(dimensions),
                        "box": aabb_snapshot(dimensions.make_bounding_box(position)),
                    }))
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"poses": poses}))
        }
        "player_pose_selection" => {
            let inputs = request
                .get("inputs")
                .ok_or("inputs must be an array")?
                .as_array()
                .ok_or("inputs must be an array")?;
            let selections = inputs
                .iter()
                .map(|value| {
                    let input = value
                        .as_object()
                        .ok_or("pose selection input must be an object")?;
                    let boolean = |name: &str| {
                        input
                            .get(name)
                            .and_then(Value::as_bool)
                            .ok_or_else(|| format!("{name} must be a boolean"))
                    };
                    let pose_input = ClientPlayerPoseInput {
                        sleeping: boolean("sleeping")?,
                        swimming: boolean("swimming")?,
                        fall_flying: boolean("fall_flying")?,
                        auto_spin_attack: boolean("auto_spin_attack")?,
                        shift_key_down: boolean("shift_key_down")?,
                        flying: boolean("flying")?,
                        spectator: boolean("spectator")?,
                        passenger: boolean("passenger")?,
                        swimming_fits: boolean("swimming_fits")?,
                        desired_fits: boolean("desired_fits")?,
                        crouching_fits: boolean("crouching_fits")?,
                    };
                    let mut selection = json!({
                        "label": input
                            .get("label")
                            .and_then(Value::as_str)
                            .ok_or("label must be a string")?,
                        "desired_id": pose_input.desired_pose().id(),
                    });
                    if let Some(actual_pose) = pose_input.updated_pose() {
                        selection["actual_id"] = json!(actual_pose.id());
                    }
                    Ok(selection)
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(json!({"selections": selections}))
        }
        "clip" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let queries = request
                .get("queries")
                .ok_or("queries must be an array")?
                .as_array()
                .ok_or("queries must be an array")?;
            let mut events = Vec::with_capacity(queries.len());
            for query in queries {
                let from = f64_vec3(query.get("from").ok_or("from must be an object")?)?;
                let to = f64_vec3(query.get("to").ok_or("to must be an object")?)?;
                let pos = blockpos(query.get("pos").ok_or("pos must be an object")?)?;
                events.push(block_hit_result_snapshot(shape.clip(from, to, pos)));
            }
            Ok(json!({"events": events}))
        }
        "clip_join" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let shape = Shapes::join(&first, &second, BooleanOp::Or);
            let queries = request
                .get("queries")
                .ok_or("queries must be an array")?
                .as_array()
                .ok_or("queries must be an array")?;
            let mut events = Vec::with_capacity(queries.len());
            for query in queries {
                let from = f64_vec3(query.get("from").ok_or("from must be an object")?)?;
                let to = f64_vec3(query.get("to").ok_or("to must be an object")?)?;
                let pos = blockpos(query.get("pos").ok_or("pos must be an object")?)?;
                events.push(block_hit_result_snapshot(shape.clip(from, to, pos)));
            }
            Ok(json!({"events": events}))
        }
        "closest" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let point = f64_vec3(request.get("point").ok_or("point must be an object")?)?;
            Ok(match shape.closest_point_to(point) {
                Some(point) => json!({"point": vec3_snapshot(point)}),
                None => json!({}),
            })
        }
        "closest_join" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let shape = Shapes::join(&first, &second, BooleanOp::Or);
            let point = f64_vec3(request.get("point").ok_or("point must be an object")?)?;
            Ok(match shape.closest_point_to(point) {
                Some(point) => json!({"point": vec3_snapshot(point)}),
                None => json!({}),
            })
        }
        "face" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let direction = Direction::from_name(string(request, "direction")?)
                .ok_or("unknown face direction")?;
            Ok(json!({"shape": shape_snapshot(&shape.face_shape(direction))}))
        }
        "face_join" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let direction = Direction::from_name(string(request, "direction")?)
                .ok_or("unknown face direction")?;
            let shape = Shapes::or(&first, &second);
            Ok(json!({"shape": shape_snapshot(&shape.face_shape(direction))}))
        }
        "block_occludes" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let occluder = shape_box(
                request
                    .get("occluder")
                    .ok_or("occluder must be an object")?,
            )?;
            let direction = Direction::from_name(string(request, "direction")?)
                .ok_or("unknown occlusion direction")?;
            Ok(json!({
                "occludes": Shapes::block_occludes(&shape, &occluder, direction),
            }))
        }
        "block_occludes_join" => {
            let first = shape_box(request.get("first").ok_or("first must be an object")?)?;
            let second = shape_box(request.get("second").ok_or("second must be an object")?)?;
            let occluder = shape_box(
                request
                    .get("occluder")
                    .ok_or("occluder must be an object")?,
            )?;
            let direction = Direction::from_name(string(request, "direction")?)
                .ok_or("unknown occlusion direction")?;
            let shape = Shapes::or(&first, &second);
            Ok(json!({
                "occludes": Shapes::block_occludes(&shape, &occluder, direction),
            }))
        }
        "merged_occludes" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let occluder = shape_box(
                request
                    .get("occluder")
                    .ok_or("occluder must be an object")?,
            )?;
            let direction = Direction::from_name(string(request, "direction")?)
                .ok_or("unknown merged occlusion direction")?;
            Ok(json!({
                "occludes": Shapes::merged_face_occludes(&shape, &occluder, direction),
            }))
        }
        "face_occludes" => {
            let shape = shape_box(request.get("shape").ok_or("shape must be an object")?)?;
            let occluder = shape_box(
                request
                    .get("occluder")
                    .ok_or("occluder must be an object")?,
            )?;
            Ok(json!({
                "occludes": Shapes::face_shape_occludes(&shape, &occluder),
            }))
        }
        _ => Err("unknown VoxelShape kind".into()),
    }
}

fn f64_vec3(value: &Value) -> Result<Vec3, String> {
    Ok(Vec3::new(
        f64_field(value, "x")?,
        f64_field(value, "y")?,
        f64_field(value, "z")?,
    ))
}

fn local_player_travel_dispatch_parameters(
    input: &Value,
) -> Result<LocalPlayerTravelDispatchParameters, String> {
    let levitation_amplifier = i32_field(input, "levitation_amplifier")?;
    let levitation_amplifier = (levitation_amplifier >= 0).then_some(levitation_amplifier);
    Ok(LocalPlayerTravelDispatchParameters {
        input: f64_vec3(input.get("input").ok_or("input must be a vector")?)?,
        y_rot: f32_field(input, "y_rot")?,
        x_rot: f32_field(input, "x_rot")?,
        look_angle: f64_vec3(input.get("look").ok_or("look must be a vector")?)?,
        in_water: input["in_water"]
            .as_bool()
            .ok_or("in_water must be a boolean")?,
        in_lava: input["in_lava"]
            .as_bool()
            .ok_or("in_lava must be a boolean")?,
        affected_by_fluids: input["affected_by_fluids"]
            .as_bool()
            .ok_or("affected_by_fluids must be a boolean")?,
        can_stand_on_fluid: input["can_stand_on_fluid"]
            .as_bool()
            .ok_or("can_stand_on_fluid must be a boolean")?,
        fall_flying: input["fall_flying"]
            .as_bool()
            .ok_or("fall_flying must be a boolean")?,
        on_climbable: input["on_climbable"]
            .as_bool()
            .ok_or("on_climbable must be a boolean")?,
        sprinting: input["sprinting"]
            .as_bool()
            .ok_or("sprinting must be a boolean")?,
        water_slow_down: f32_field(input, "water_slow_down")?,
        water_efficiency: f32_field(input, "water_efficiency")?,
        on_ground: input["on_ground"]
            .as_bool()
            .ok_or("on_ground must be a boolean")?,
        block_friction: f32_field(input, "block_friction")?,
        speed: f32_field(input, "speed")?,
        dolphins_grace: input["dolphins_grace"]
            .as_bool()
            .ok_or("dolphins_grace must be a boolean")?,
        horizontal_collision: input["horizontal_collision"]
            .as_bool()
            .ok_or("horizontal_collision must be a boolean")?,
        free_after_jump: input["free_after_jump"]
            .as_bool()
            .ok_or("free_after_jump must be a boolean")?,
        shallow_lava: input["shallow_lava"]
            .as_bool()
            .ok_or("shallow_lava must be a boolean")?,
        flying_speed: f32_field(input, "flying_speed")?,
        flying: input["flying"]
            .as_bool()
            .ok_or("flying must be a boolean")?,
        passenger: input["passenger"]
            .as_bool()
            .ok_or("passenger must be a boolean")?,
        scaffolding: input["scaffolding"]
            .as_bool()
            .ok_or("scaffolding must be a boolean")?,
        suppress_sliding_down_ladder: input["suppress_sliding_down_ladder"]
            .as_bool()
            .ok_or("suppress_sliding_down_ladder must be a boolean")?,
        jumping: input["jumping"]
            .as_bool()
            .ok_or("jumping must be a boolean")?,
        levitation_amplifier,
        gravity: f64_field(input, "gravity")?,
        client_side: input["client_side"]
            .as_bool()
            .ok_or("client_side must be a boolean")?,
        has_chunk_at: input["has_chunk_at"]
            .as_bool()
            .ok_or("has_chunk_at must be a boolean")?,
        current_y: f64_field(input, "current_y")?,
        min_y: i32_field(input, "min_y")?,
        should_discard_friction: input["should_discard_friction"]
            .as_bool()
            .ok_or("should_discard_friction must be a boolean")?,
        friction_modifier: f32_field(input, "friction_modifier")?,
        air_drag_modifier: f32_field(input, "air_drag_modifier")?,
        omnidirectional_air_mover: input["omnidirectional_air_mover"]
            .as_bool()
            .ok_or("omnidirectional_air_mover must be a boolean")?,
    })
}

fn clip_snapshot(value: Vec3) -> Value {
    json!({
        "x": format!("{:016x}", value.x.to_bits()),
        "y": format!("{:016x}", value.y.to_bits()),
        "z": format!("{:016x}", value.z.to_bits()),
    })
}

fn aabb_result(request: &Value) -> Result<Value, String> {
    let mut value = aabb_box(request)?;
    let mut events = vec![aabb_snapshot(value)];
    for step in request["steps"]
        .as_array()
        .ok_or("steps must be an array")?
    {
        match string(step, "op")? {
            "contains" => events.push(json!({
                "hit": value.contains(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                )
            })),
            "contains_vec3" => {
                let position = f64_vec3(step.get("position").ok_or("position must be an object")?)?;
                events.push(json!({"hit": value.contains_vec3(position)}));
            }
            "intersects_coords" => {
                events.push(json!({
                    "hit": value.intersects_coords(
                        f64_field(step, "min_x")?,
                        f64_field(step, "min_y")?,
                        f64_field(step, "min_z")?,
                        f64_field(step, "max_x")?,
                        f64_field(step, "max_y")?,
                        f64_field(step, "max_z")?,
                    )
                }));
            }
            "intersects_vec3" => {
                let min = f64_vec3(step.get("min").ok_or("min must be an object")?)?;
                let max = f64_vec3(step.get("max").ok_or("max must be an object")?)?;
                events.push(json!({"hit": value.intersects_vec3(min, max)}));
            }
            "intersects_block_pos" => {
                let position = blockpos(step.get("position").ok_or("position must be an object")?)?;
                events.push(json!({"hit": value.intersects_block_pos(position)}));
            }
            "collided_along_vector" => {
                let vector = f64_vec3(step.get("vector").ok_or("vector must be an object")?)?;
                let entries = step
                    .get("boxes")
                    .ok_or("boxes must be an array")?
                    .as_array()
                    .ok_or("boxes must be an array")?;
                let boxes = entries
                    .iter()
                    .map(aabb_box)
                    .collect::<Result<Vec<_>, _>>()?;
                events.push(json!({
                    "hit": value.collided_along_vector(vector, &boxes),
                }));
            }
            "from_block_pos" => {
                let position = blockpos(step.get("position").ok_or("position must be an object")?)?;
                value = Aabb::from_block_pos(position);
                events.push(aabb_snapshot(value));
            }
            "from_vec3" => {
                let begin = f64_vec3(step.get("begin").ok_or("begin must be an object")?)?;
                let end = f64_vec3(step.get("end").ok_or("end must be an object")?)?;
                value = Aabb::from_vec3(begin, end);
                events.push(aabb_snapshot(value));
            }
            "unit_cube_from_lower_corner" => {
                let position = f64_vec3(step.get("position").ok_or("position must be an object")?)?;
                value = Aabb::unit_cube_from_lower_corner(position);
                events.push(aabb_snapshot(value));
            }
            "encapsulating_full_blocks" => {
                let first = blockpos(step.get("first").ok_or("first must be an object")?)?;
                let second = blockpos(step.get("second").ok_or("second must be an object")?)?;
                value = Aabb::encapsulating_full_blocks(first, second);
                events.push(aabb_snapshot(value));
            }
            "of_bounding_box" => {
                let bounds = bounding_box(step.get("box").ok_or("box must be an object")?)?;
                value = Aabb::of_bounding_box(bounds);
                events.push(aabb_snapshot(value));
            }
            "expand_vec3" => {
                value = value.expand_towards_vec3(f64_vec3(
                    step.get("delta").ok_or("delta must be an object")?,
                )?);
                events.push(aabb_snapshot(value));
            }
            "equals" => {
                let other = aabb_box(step.get("box").ok_or("box must be an object")?)?;
                events.push(json!({"equal": value.equals_java(other)}));
            }
            "hash_code" => events.push(json!({"hash_code": value.hash_code_java()})),
            "to_string" => events.push(json!({"value": value.to_string_java()})),
            "intersects" => {
                let other = aabb_box(step.get("box").ok_or("box must be an object")?)?;
                events.push(json!({"hit": value.intersects(other)}));
            }
            "intersect" => {
                let other = aabb_box(step.get("box").ok_or("box must be an object")?)?;
                value = value.intersect(other);
                events.push(aabb_snapshot(value));
            }
            "minmax" => {
                let other = aabb_box(step.get("box").ok_or("box must be an object")?)?;
                value = value.minmax(other);
                events.push(aabb_snapshot(value));
            }
            "move" => {
                value = value.move_by(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                );
                events.push(aabb_snapshot(value));
            }
            "move_block_pos" => {
                let position = blockpos(step.get("position").ok_or("position must be an object")?)?;
                value = value.move_by_block_pos(position);
                events.push(aabb_snapshot(value));
            }
            "move_vec3" => {
                let position = f64_vec3(step.get("position").ok_or("position must be an object")?)?;
                value = value.move_by_vec3(position);
                events.push(aabb_snapshot(value));
            }
            "move_vector3f" => {
                let position = step.get("position").ok_or("position must be an object")?;
                value = value.move_by_vector3f(
                    f32_field(position, "x")?,
                    f32_field(position, "y")?,
                    f32_field(position, "z")?,
                );
                events.push(aabb_snapshot(value));
            }
            "sizes" => events.push(aabb_sizes_snapshot(value)),
            "has_nan" => events.push(json!({"has_nan": value.has_nan()})),
            "positions" => events.push(aabb_positions_snapshot(value)),
            "min" | "max" => {
                let axis = Axis::from_name(string(step, "axis")?).ok_or("unknown axis")?;
                let endpoint = if string(step, "op")? == "min" {
                    value.min(axis)
                } else {
                    value.max(axis)
                };
                events.push(json!({
                    "value": format!("{:016x}", endpoint.to_bits()),
                }));
            }
            "distance_point" => {
                let point = f64_vec3(step.get("point").ok_or("point must be an object")?)?;
                events.push(json!({
                    "value": format!("{:016x}", value.distance_to_sqr_point(point).to_bits()),
                }));
            }
            "distance_box" => {
                let other = aabb_box(step.get("box").ok_or("box must be an object")?)?;
                events.push(json!({
                    "value": format!("{:016x}", value.distance_to_sqr_box(other).to_bits()),
                }));
            }
            "of_size" => {
                let center = f64_vec3(step.get("center").ok_or("center must be an object")?)?;
                value = Aabb::of_size(
                    center,
                    f64_field(step, "size_x")?,
                    f64_field(step, "size_y")?,
                    f64_field(step, "size_z")?,
                );
                events.push(aabb_snapshot(value));
            }
            "set_min_x" => {
                value = value.set_min_x(f64_field(step, "value")?);
                events.push(aabb_snapshot(value));
            }
            "set_min_y" => {
                value = value.set_min_y(f64_field(step, "value")?);
                events.push(aabb_snapshot(value));
            }
            "set_min_z" => {
                value = value.set_min_z(f64_field(step, "value")?);
                events.push(aabb_snapshot(value));
            }
            "set_max_x" => {
                value = value.set_max_x(f64_field(step, "value")?);
                events.push(aabb_snapshot(value));
            }
            "set_max_y" => {
                value = value.set_max_y(f64_field(step, "value")?);
                events.push(aabb_snapshot(value));
            }
            "set_max_z" => {
                value = value.set_max_z(f64_field(step, "value")?);
                events.push(aabb_snapshot(value));
            }
            "clip" => {
                let from = f64_vec3(step.get("from").ok_or("from must be an object")?)?;
                let to = f64_vec3(step.get("to").ok_or("to must be an object")?)?;
                events.push(match value.clip(from, to) {
                    Some(point) => json!({"hit": true, "point": clip_snapshot(point)}),
                    None => json!({"hit": false}),
                });
            }
            "clip_coords" => {
                let from = f64_vec3(step.get("from").ok_or("from must be an object")?)?;
                let to = f64_vec3(step.get("to").ok_or("to must be an object")?)?;
                events.push(
                    match Aabb::clip_coords(
                        Vec3::new(
                            f64_field(step, "min_x")?,
                            f64_field(step, "min_y")?,
                            f64_field(step, "min_z")?,
                        ),
                        Vec3::new(
                            f64_field(step, "max_x")?,
                            f64_field(step, "max_y")?,
                            f64_field(step, "max_z")?,
                        ),
                        from,
                        to,
                    ) {
                        Some(point) => json!({"hit": true, "point": clip_snapshot(point)}),
                        None => json!({"hit": false}),
                    },
                );
            }
            "clip_many" => {
                let entries = step
                    .get("boxes")
                    .ok_or("boxes must be an array")?
                    .as_array()
                    .ok_or("boxes must be an array")?;
                let boxes = entries
                    .iter()
                    .map(aabb_box)
                    .collect::<Result<Vec<_>, _>>()?;
                let from = f64_vec3(step.get("from").ok_or("from must be an object")?)?;
                let to = f64_vec3(step.get("to").ok_or("to must be an object")?)?;
                let pos = blockpos(step.get("pos").ok_or("pos must be an object")?)?;
                events.push(block_hit_result_snapshot(Aabb::clip_many(
                    &boxes, from, to, pos,
                )));
            }
            "expand" => {
                value = value.expand_towards(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                );
                events.push(aabb_snapshot(value));
            }
            "contract" => {
                value = value.contract(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                );
                events.push(aabb_snapshot(value));
            }
            "inflate" => {
                value = value.inflate_all(f64_field(step, "amount")?);
                events.push(aabb_snapshot(value));
            }
            "inflate_xyz" => {
                value = value.inflate(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                );
                events.push(aabb_snapshot(value));
            }
            "deflate" => {
                value = value.deflate(f64_field(step, "amount")?);
                events.push(aabb_snapshot(value));
            }
            "deflate_xyz" => {
                value = value.deflate_xyz(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                );
                events.push(aabb_snapshot(value));
            }
            _ => return Err("unknown AABB operation".into()),
        }
    }
    Ok(json!({"events": events}))
}

fn client_world_section_lookup_result(request: &Value) -> Result<Value, String> {
    let min_y = i32_field(request, "dimension_min_y")?;
    let height = i32_field(request, "dimension_height")?;
    let min_section_y = min_y.div_euclid(16);
    let tracks = request["tracks"]
        .as_array()
        .ok_or("tracks must be an array")?
        .iter()
        .map(|track| {
            let chunk_x = i32_field(track, "chunk_x")?;
            let chunk_z = i32_field(track, "chunk_z")?;
            let section_values = track["sections"]
                .as_array()
                .ok_or("sections must be an array")?;
            let mut section_data = Vec::new();
            for section in section_values {
                section_data.extend(decode_hex(
                    section.as_str().ok_or("section payload must be a string")?,
                )?);
            }
            let observations = decode_chunk_section_data(&section_data)?;
            if observations.len() != section_values.len() {
                return Err("section decoder count does not match input".into());
            }

            let mut world = ClientWorld::new(DimensionBounds::new(min_y, height));
            world.replace_chunk(
                chunk_x,
                chunk_z,
                observations.iter().map(|section| SectionInput {
                    non_empty_block_count: section.non_empty_block_count,
                    fluid_count: section.fluid_count,
                    block_states: &section.block_state_values,
                    biomes: &section.biome_values,
                }),
            );

            let sections = observations
                .iter()
                .enumerate()
                .map(|(index, section)| {
                    json!({
                        "section_y": min_section_y + index as i32,
                        "non_empty_block_count": section.non_empty_block_count,
                        "fluid_count": section.fluid_count,
                        "has_only_air": section.non_empty_block_count == 0,
                        "has_fluid": section.fluid_count > 0,
                    })
                })
                .collect::<Vec<_>>();

            let queries = track["queries"]
                .as_array()
                .ok_or("queries must be an array")?
                .iter()
                .map(|query| {
                    let x = i32_field(query, "x")?;
                    let y = i32_field(query, "y")?;
                    let z = i32_field(query, "z")?;
                    let query_section_y = y.div_euclid(16);
                    let mut state = json!({
                        "label": string(query, "label")?,
                        "section_y": query_section_y,
                    });
                    if let Some(block_state_id) = world.block_state_at(x, y, z) {
                        state["block_state_id"] = json!(block_state_id);
                    }
                    if let Some(biome_id) = world.biome_at(x, y, z) {
                        state["biome_id"] = json!(biome_id);
                    }
                    if let Some((non_empty, fluid)) =
                        world.section_counts_at(x.div_euclid(16), query_section_y, z.div_euclid(16))
                    {
                        state["section_counts"] = json!([non_empty, fluid]);
                    }
                    Ok(state)
                })
                .collect::<Result<Vec<_>, String>>()?;

            Ok(json!({
                "label": string(track, "label")?,
                "sections": sections,
                "queries": queries,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({"tracks": tracks}))
}

fn client_world_block_update_result(request: &Value) -> Result<Value, String> {
    let min_y = i32_field(request, "dimension_min_y")?;
    let height = i32_field(request, "dimension_height")?;
    let chunk_x = i32_field(request, "chunk_x")?;
    let chunk_z = i32_field(request, "chunk_z")?;
    let section_values = request["sections"]
        .as_array()
        .ok_or("sections must be an array")?;
    let mut section_data = Vec::new();
    for section in section_values {
        section_data.extend(decode_hex(
            section.as_str().ok_or("section payload must be a string")?,
        )?);
    }
    let observations = decode_chunk_section_data(&section_data)?;
    if observations.len() != section_values.len() {
        return Err("section decoder count does not match input".into());
    }

    let mut world = ClientWorld::new(DimensionBounds::new(min_y, height));
    if !world.replace_chunk(
        chunk_x,
        chunk_z,
        observations.iter().map(|section| SectionInput {
            non_empty_block_count: section.non_empty_block_count,
            fluid_count: section.fluid_count,
            block_states: &section.block_state_values,
            biomes: &section.biome_values,
        }),
    ) {
        return Err("initial chunk was rejected".into());
    }

    let updates = request["updates"]
        .as_array()
        .ok_or("updates must be an array")?;
    let mut events = Vec::with_capacity(updates.len());
    for update in updates {
        let x = i32_field(update, "x")?;
        let y = i32_field(update, "y")?;
        let z = i32_field(update, "z")?;
        let previous_state_id = world.block_state_at(x, y, z);
        let accepted = world.update_block_state(x, y, z, i32_field(update, "state_id")?);
        let mut state = json!({
            "label": string(update, "label")?,
            "accepted": accepted,
        });
        if accepted {
            let updated_state_id = world
                .block_state_at(x, y, z)
                .ok_or("accepted block update has no state")?;
            let section_counts = world
                .section_counts_at(x.div_euclid(16), y.div_euclid(16), z.div_euclid(16))
                .ok_or("accepted block update has no section counts")?;
            state["previous_state_id"] =
                json!(previous_state_id.ok_or("accepted block update has no previous state")?);
            state["updated_state_id"] = json!(updated_state_id);
            state["section_counts"] = json!([section_counts.0, section_counts.1]);
            state["has_only_air"] = json!(section_counts.0 == 0);
            state["has_fluid"] = json!(section_counts.1 > 0);
        }

        let queries = update["queries"]
            .as_array()
            .ok_or("queries must be an array")?
            .iter()
            .map(|query| {
                let query_x = i32_field(query, "x")?;
                let query_y = i32_field(query, "y")?;
                let query_z = i32_field(query, "z")?;
                let mut query_state = json!({
                    "label": string(query, "label")?,
                    "section_y": query_y.div_euclid(16),
                });
                if let Some(block_state_id) = world.block_state_at(query_x, query_y, query_z) {
                    query_state["block_state_id"] = json!(block_state_id);
                }
                if let Some((non_empty, fluid)) = world.section_counts_at(
                    query_x.div_euclid(16),
                    query_y.div_euclid(16),
                    query_z.div_euclid(16),
                ) {
                    query_state["section_counts"] = json!([non_empty, fluid]);
                }
                Ok(query_state)
            })
            .collect::<Result<Vec<_>, String>>()?;
        state["queries"] = json!(queries);
        events.push(state);
    }
    Ok(json!({"events": events}))
}

fn light_bitset_field(value: &Value, name: &str) -> Result<BitSet, String> {
    let bits = value[name]
        .as_array()
        .ok_or_else(|| format!("{name} must be an array"))?
        .iter()
        .map(|bit| {
            bit.as_u64()
                .ok_or_else(|| format!("{name} entries must be unsigned integers"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BitSet::from_bits(&bits))
}

fn light_updates_field(value: &Value, name: &str) -> Result<Vec<Vec<u8>>, String> {
    value[name]
        .as_array()
        .ok_or_else(|| format!("{name} must be an array"))?
        .iter()
        .map(|update| {
            decode_hex(
                update
                    .as_str()
                    .ok_or_else(|| format!("{name} entries must be hexadecimal strings"))?,
            )
        })
        .collect()
}

fn client_world_light_queries(world: &ClientWorld, value: &Value) -> Result<Value, String> {
    value
        .as_array()
        .ok_or("queries must be an array")?
        .iter()
        .map(|query| {
            let layer = string(query, "layer")?;
            let x = i32_field(query, "x")?;
            let y = i32_field(query, "y")?;
            let z = i32_field(query, "z")?;
            let light = match layer {
                "sky" => world.sky_light_at(x, y, z),
                "block" => world.block_light_at(x, y, z),
                _ => return Err(format!("unknown light layer: {layer}")),
            };
            let mut state = json!({"label": string(query, "label")?});
            if let Some(light) = light {
                state["value"] = json!(light);
            }
            Ok(state)
        })
        .collect::<Result<Vec<_>, String>>()
        .map(Value::Array)
}

fn client_world_light_result(request: &Value) -> Result<Value, String> {
    let min_y = i32_field(request, "dimension_min_y")?;
    let height = i32_field(request, "dimension_height")?;
    let mut world = ClientWorld::new(DimensionBounds::new(min_y, height));
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut events = Vec::with_capacity(operations.len());

    for operation in operations {
        let action = string(operation, "action")?;
        let label = string(operation, "label")?;
        let mut event = json!({
            "action": action,
            "label": label,
            "queued_for_unloaded_chunk": false,
            "stored": false,
            "applied_sky_sections": 0,
            "applied_block_sections": 0,
            "queries": [],
        });

        match action {
            "apply" => {
                let chunk_x = i32_field(operation, "chunk_x")?;
                let chunk_z = i32_field(operation, "chunk_z")?;
                let sky_mask = light_bitset_field(operation, "sky_mask")?;
                let sky_empty_mask = light_bitset_field(operation, "sky_empty_mask")?;
                let sky_updates = light_updates_field(operation, "sky_updates")?;
                let block_mask = light_bitset_field(operation, "block_mask")?;
                let block_empty_mask = light_bitset_field(operation, "block_empty_mask")?;
                let block_updates = light_updates_field(operation, "block_updates")?;
                let application = world.apply_light_updates(
                    chunk_x,
                    chunk_z,
                    LightUpdateInput {
                        sky: LightLayerInput {
                            mask: &sky_mask,
                            empty_mask: &sky_empty_mask,
                            updates: &sky_updates,
                        },
                        block: LightLayerInput {
                            mask: &block_mask,
                            empty_mask: &block_empty_mask,
                            updates: &block_updates,
                        },
                    },
                );
                if application.invalid_sky_updates != 0 || application.invalid_block_updates != 0 {
                    return Err("light update data layer is not 2048 bytes".into());
                }
                // The oracle runs the engine after each packet, as a frame would.
                world.flush_light();
                event["queued_for_unloaded_chunk"] = json!(application.queued);
                // `hasLightData`: any addressed section with a layer.
                let engine = world.light_engine();
                let min = engine.min_light_section();
                let stored = |layer, mask: &BitSet, empty: &BitSet| {
                    (0..engine.light_section_count()).any(|index| {
                        (mask.contains(index as u64) || empty.contains(index as u64))
                            && engine
                                .data_layer(layer, (chunk_x, min + index, chunk_z))
                                .is_some()
                    })
                };
                event["stored"] = json!(
                    stored(
                        voxel_world::light::LightLayer::Sky,
                        &sky_mask,
                        &sky_empty_mask
                    ) || stored(
                        voxel_world::light::LightLayer::Block,
                        &block_mask,
                        &block_empty_mask
                    )
                );
                event["applied_sky_sections"] = json!(application.applied_sky_sections);
                event["applied_block_sections"] = json!(application.applied_block_sections);
            }
            "load" => {
                let chunk_x = i32_field(operation, "chunk_x")?;
                let chunk_z = i32_field(operation, "chunk_z")?;
                let section_count = operation["section_count"]
                    .as_u64()
                    .ok_or("section_count must be an unsigned integer")?
                    .try_into()
                    .map_err(|_| "section_count does not fit in usize")?;
                let non_empty_sections = operation["non_empty_sections"]
                    .as_array()
                    .ok_or("non_empty_sections must be an array")?
                    .iter()
                    .map(|index| {
                        index
                            .as_u64()
                            .ok_or_else(|| {
                                "non_empty_sections entries must be unsigned integers".to_owned()
                            })?
                            .try_into()
                            .map_err(|_| "non_empty section index does not fit in usize".to_owned())
                    })
                    .collect::<Result<Vec<usize>, String>>()?;
                let mut block_sections = Vec::with_capacity(section_count);
                let mut biome_sections = Vec::with_capacity(section_count);
                for index in 0..section_count {
                    let non_empty = non_empty_sections.contains(&index);
                    let mut blocks = vec![0; 4_096];
                    if non_empty {
                        blocks[0] = 1;
                    }
                    block_sections.push(blocks);
                    biome_sections.push(vec![0; 64]);
                }
                if !world.replace_chunk(
                    chunk_x,
                    chunk_z,
                    block_sections
                        .iter()
                        .zip(&biome_sections)
                        .map(|(blocks, biomes)| SectionInput {
                            non_empty_block_count: if blocks[0] == 0 { 0 } else { 1 },
                            fluid_count: 0,
                            block_states: blocks,
                            biomes,
                        }),
                ) {
                    return Err("light test chunk was rejected".into());
                }
                world.enable_chunk_light(chunk_x, chunk_z);
                world.run_light_updates();
                let engine = world.light_engine();
                let section = (chunk_x, min_y.div_euclid(16), chunk_z);
                event["stored"] = json!(
                    engine
                        .data_layer(voxel_world::light::LightLayer::Sky, section)
                        .is_some()
                        || engine
                            .data_layer(voxel_world::light::LightLayer::Block, section)
                            .is_some()
                );
            }
            "unload" => {
                let chunk_x = i32_field(operation, "chunk_x")?;
                let chunk_z = i32_field(operation, "chunk_z")?;
                world.unload_chunk(chunk_x, chunk_z);
                world.flush_light();
            }
            _ => return Err(format!("unknown client world light action: {action}")),
        }

        event["queries"] = client_world_light_queries(
            &world,
            operation
                .get("queries")
                .unwrap_or(&Value::Array(Vec::new())),
        )?;
        events.push(event);
    }

    Ok(json!({"events": events}))
}

fn client_attribute_modifier(value: &Value) -> Result<ClientAttributeModifierState, String> {
    let operation = value["operation"]
        .as_u64()
        .ok_or("attribute modifier operation must be an unsigned integer")?
        .try_into()
        .map_err(|_| "attribute modifier operation does not fit in u8".to_owned())?;
    Ok(ClientAttributeModifierState::new(
        string(value, "id")?.to_owned(),
        f64_field(value, "amount")?,
        operation,
    ))
}

fn client_attribute_state(value: &Value) -> Result<ClientAttributeState, String> {
    let modifiers = value["modifiers"]
        .as_array()
        .ok_or("attribute modifiers must be an array")?
        .iter()
        .map(client_attribute_modifier)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ClientAttributeState::new(
        i32_field(value, "attribute_id")?,
        f64_field(value, "base")?,
        modifiers,
    ))
}

fn client_attribute_state_json(
    label: &str,
    attribute: &ClientAttributeState,
    min_value: f64,
    max_value: f64,
) -> Value {
    let mut modifiers = attribute.modifiers().iter().collect::<Vec<_>>();
    modifiers.sort_by(|left, right| left.id().cmp(right.id()));
    json!({
        "label": label,
        "attribute_id": attribute.attribute_id(),
        "found": true,
        "base_value": format!("{:016x}", attribute.base_value().to_bits()),
        "modifiers": modifiers
            .into_iter()
            .map(|modifier| json!({
                "id": modifier.id(),
                "amount": format!("{:016x}", modifier.amount().to_bits()),
                "operation": modifier.operation(),
            }))
            .collect::<Vec<_>>(),
        "effective_value": format!(
            "{:016x}",
            attribute.effective_value(min_value, max_value).to_bits()
        ),
    })
}

fn client_world_player_attribute_queries(
    world: &ClientWorld,
    value: &Value,
) -> Result<Value, String> {
    let player = world
        .local_player()
        .ok_or("local player is not initialized")?;
    value
        .as_array()
        .ok_or("queries must be an array")?
        .iter()
        .map(|query| {
            let label = string(query, "label")?;
            let attribute_id = i32_field(query, "attribute_id")?;
            let min_value = f64_field(query, "min")?;
            let max_value = f64_field(query, "max")?;
            let Some(attribute) = player
                .attributes()
                .iter()
                .find(|attribute| attribute.attribute_id() == attribute_id)
            else {
                return Ok(json!({
                    "label": label,
                    "attribute_id": attribute_id,
                    "found": false,
                }));
            };
            Ok(client_attribute_state_json(
                label, attribute, min_value, max_value,
            ))
        })
        .collect::<Result<Vec<_>, String>>()
        .map(Value::Array)
}

fn client_world_player_attributes_result(request: &Value) -> Result<Value, String> {
    let player_id = i32_field(request, "player_id")?;
    let mut world = ClientWorld::new(DimensionBounds::new(0, 256));
    world.set_local_player_id(player_id);
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut events = Vec::with_capacity(operations.len());

    for operation in operations {
        if string(operation, "action")? != "apply" {
            return Err("unknown player attribute action".into());
        }
        let attributes = operation["attributes"]
            .as_array()
            .ok_or("attributes must be an array")?
            .iter()
            .map(client_attribute_state)
            .collect::<Result<Vec<_>, _>>()?;
        let accepted =
            world.apply_local_player_attributes(i32_field(operation, "player_id")?, &attributes);
        events.push(json!({
            "action": string(operation, "action")?,
            "label": string(operation, "label")?,
            "accepted": accepted,
            "queries": client_world_player_attribute_queries(
                &world,
                &operation["queries"],
            )?,
        }));
    }
    Ok(json!({"events": events}))
}

fn client_world_player_movement_speed_result(request: &Value) -> Result<Value, String> {
    let player_id = i32_field(request, "player_id")?;
    let mut world = ClientWorld::new(DimensionBounds::new(0, 256));
    world.set_local_player_id(player_id);
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());

    for operation in operations {
        let attributes = operation["attributes"]
            .as_array()
            .ok_or("attributes must be an array")?
            .iter()
            .map(client_attribute_state)
            .collect::<Result<Vec<_>, _>>()?;
        let accepted =
            world.apply_local_player_attributes(i32_field(operation, "player_id")?, &attributes);
        if !world.apply_local_player_friction_influenced_speed_from_attributes(
            operation["on_ground"]
                .as_bool()
                .ok_or("on_ground must be a boolean")?,
            f32_field(operation, "block_friction")?,
        ) {
            return Err("local player is not initialized".into());
        }
        let player = world
            .local_player()
            .ok_or("local player is not initialized")?;
        states.push(json!({
            "label": string(operation, "label")?,
            "accepted": accepted,
            "movement_speed": format!("{:08x}", player.movement_speed().to_bits()),
            "friction_influenced_speed": format!(
                "{:08x}",
                player.friction_influenced_speed().to_bits()
            ),
        }));
    }
    Ok(json!({"states": states}))
}

fn client_world_player_attribute_water_result(request: &Value) -> Result<Value, String> {
    let player_id = i32_field(request, "player_id")?;
    let mut world = ClientWorld::new(DimensionBounds::new(0, 256));
    world.set_local_player_id(player_id);
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());

    for operation in operations {
        let attributes = operation["attributes"]
            .as_array()
            .ok_or("attributes must be an array")?
            .iter()
            .map(client_attribute_state)
            .collect::<Result<Vec<_>, _>>()?;
        let accepted =
            world.apply_local_player_attributes(i32_field(operation, "player_id")?, &attributes);
        let delta = f64_vec3(operation.get("delta").ok_or("delta must be a vector")?)?;
        let y_rot = f32_field(operation, "y_rot")?;
        if !world.apply_local_player_position(
            (0.0, f64_field(operation, "current_y")?, 0.0),
            (delta.x, delta.y, delta.z),
            (y_rot, 0.0),
        ) {
            return Err("local player is not initialized".into());
        }
        let parameters = LocalPlayerTravelInWaterParameters {
            input: f64_vec3(operation.get("input").ok_or("input must be a vector")?)?,
            y_rot,
            sprinting: operation["sprinting"]
                .as_bool()
                .ok_or("sprinting must be a boolean")?,
            water_slow_down: f32_field(operation, "water_slow_down")?,
            water_efficiency: f32_field(operation, "water_efficiency")?,
            on_ground: operation["on_ground"]
                .as_bool()
                .ok_or("on_ground must be a boolean")?,
            speed: 0.0,
            dolphins_grace: operation["dolphins_grace"]
                .as_bool()
                .ok_or("dolphins_grace must be a boolean")?,
            horizontal_collision: operation["horizontal_collision"]
                .as_bool()
                .ok_or("horizontal_collision must be a boolean")?,
            on_climbable: operation["on_climbable"]
                .as_bool()
                .ok_or("on_climbable must be a boolean")?,
            free_after_jump: operation["free_after_jump"]
                .as_bool()
                .ok_or("free_after_jump must be a boolean")?,
            base_gravity: f64_field(operation, "base_gravity")?,
            is_falling: operation["is_falling"]
                .as_bool()
                .ok_or("is_falling must be a boolean")?,
        };
        if !world.apply_local_player_travel_in_water_from_attributes(parameters) {
            return Err("local player is not initialized".into());
        }
        let player = world
            .local_player()
            .ok_or("local player is not initialized")?;
        let motion = player.motion();
        states.push(json!({
            "label": string(operation, "label")?,
            "accepted": accepted,
            "movement_speed": format!("{:08x}", player.movement_speed().to_bits()),
            "movement": vec3_snapshot(Vec3::new(motion.0, motion.1, motion.2)),
        }));
    }
    Ok(json!({"states": states}))
}

fn client_world_player_sprint_attribute_state(
    label: &str,
    accepted: bool,
    player: &voxel_world::ClientPlayerState,
) -> Value {
    let attribute = player
        .attributes()
        .iter()
        .find(|attribute| attribute.attribute_id() == 26);
    let base_value = attribute.map_or(0.1, |attribute| attribute.base_value());
    let mut modifiers = attribute.map_or_else(Vec::new, |attribute| {
        attribute.modifiers().iter().collect::<Vec<_>>()
    });
    modifiers.sort_by(|left, right| left.id().cmp(right.id()));
    json!({
        "label": label,
        "accepted": accepted,
        "sprinting": player.is_sprinting(),
        "base_value": format!("{:016x}", base_value.to_bits()),
        "modifiers": modifiers
            .into_iter()
            .map(|modifier| json!({
                "id": modifier.id(),
                "amount": format!("{:016x}", modifier.amount().to_bits()),
                "operation": modifier.operation(),
            }))
            .collect::<Vec<_>>(),
        "movement_speed": format!("{:08x}", player.movement_speed().to_bits()),
    })
}

fn client_world_player_sprint_attribute_result(request: &Value) -> Result<Value, String> {
    let player_id = i32_field(request, "player_id")?;
    let mut world = ClientWorld::new(DimensionBounds::new(0, 256));
    world.set_local_player_id(player_id);
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());

    for operation in operations {
        if string(operation, "action")? != "set_sprinting" {
            return Err("unknown player sprint action".into());
        }
        let attributes = operation["attributes"]
            .as_array()
            .ok_or("attributes must be an array")?
            .iter()
            .map(client_attribute_state)
            .collect::<Result<Vec<_>, _>>()?;
        let operation_player_id = i32_field(operation, "player_id")?;
        let accepted = world.apply_local_player_attributes(operation_player_id, &attributes);
        if accepted
            && !world.apply_local_player_sprinting(
                operation_player_id,
                operation["sprinting"]
                    .as_bool()
                    .ok_or("sprinting must be a boolean")?,
            )
        {
            return Err("local player is not initialized".into());
        }
        let player = world
            .local_player()
            .ok_or("local player is not initialized")?;
        states.push(client_world_player_sprint_attribute_state(
            string(operation, "label")?,
            accepted,
            player,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_world_player_sneaking_input_state(
    label: &str,
    accepted: bool,
    player: &voxel_world::ClientPlayerState,
) -> Value {
    let bob = player.bob();
    let previous_bob = player.previous_bob();
    json!({
        "label": label,
        "accepted": accepted,
        "sneaking_speed": format!("{:016x}", player.sneaking_movement_factor_value().to_bits()),
        "movement": vec2_snapshot(player.input_impulses()),
        "jumping": player.is_jumping(),
        "x_bob": format!("{:08x}", bob.0.to_bits()),
        "y_bob": format!("{:08x}", bob.1.to_bits()),
        "x_bob_o": format!("{:08x}", previous_bob.0.to_bits()),
        "y_bob_o": format!("{:08x}", previous_bob.1.to_bits()),
    })
}

fn client_world_player_sneaking_input_result(request: &Value) -> Result<Value, String> {
    let player_id = i32_field(request, "player_id")?;
    let mut world = ClientWorld::new(DimensionBounds::new(0, 256));
    world.set_local_player_id(player_id);
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());

    for operation in operations {
        if string(operation, "action")? != "apply_input" {
            return Err("unknown player sneaking input action".into());
        }
        let attributes = operation["attributes"]
            .as_array()
            .ok_or("attributes must be an array")?
            .iter()
            .map(client_attribute_state)
            .collect::<Result<Vec<_>, _>>()?;
        let operation_player_id = i32_field(operation, "player_id")?;
        let accepted = world.apply_local_player_attributes(operation_player_id, &attributes);
        let modifiers = LocalPlayerInputModifiers {
            using_item: operation["using_item"]
                .as_bool()
                .ok_or("using_item must be a boolean")?,
            passenger: operation["passenger"]
                .as_bool()
                .ok_or("passenger must be a boolean")?,
            item_use_speed_multiplier: f32_field(operation, "item_use_speed_multiplier")?,
            moving_slowly: operation["moving_slowly"]
                .as_bool()
                .ok_or("moving_slowly must be a boolean")?,
            sneaking_movement_factor: 0.3,
        };
        if !world.apply_local_player_input_from_attributes(
            Vec2::new(
                f32_field(operation, "move_x")?,
                f32_field(operation, "move_y")?,
            ),
            operation["jump"]
                .as_bool()
                .ok_or("jump must be a boolean")?,
            operation["controlled_camera"]
                .as_bool()
                .ok_or("controlled_camera must be a boolean")?,
            modifiers,
        ) {
            return Err("local player is not initialized".into());
        }
        let player = world
            .local_player()
            .ok_or("local player is not initialized")?;
        states.push(client_world_player_sneaking_input_state(
            string(operation, "label")?,
            accepted,
            player,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_world_player_keyboard_input_state(
    label: &str,
    accepted: bool,
    keyboard: &KeyboardInput,
    player: &voxel_world::ClientPlayerState,
) -> Value {
    let bob = player.bob();
    let previous_bob = player.previous_bob();
    json!({
        "label": label,
        "accepted": accepted,
        "key_presses": input_snapshot(keyboard.key_presses),
        "move_vector": vec2_snapshot(keyboard.move_vector()),
        "movement": vec2_snapshot(player.input_impulses()),
        "jumping": player.is_jumping(),
        "x_bob": format!("{:08x}", bob.0.to_bits()),
        "y_bob": format!("{:08x}", bob.1.to_bits()),
        "x_bob_o": format!("{:08x}", previous_bob.0.to_bits()),
        "y_bob_o": format!("{:08x}", previous_bob.1.to_bits()),
    })
}

fn client_world_player_keyboard_input_result(request: &Value) -> Result<Value, String> {
    let player_id = i32_field(request, "player_id")?;
    let mut world = ClientWorld::new(DimensionBounds::new(0, 256));
    world.set_local_player_id(player_id);
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());

    for operation in operations {
        if string(operation, "action")? != "apply_keyboard_input" {
            return Err("unknown player keyboard input action".into());
        }
        let attributes = operation["attributes"]
            .as_array()
            .ok_or("attributes must be an array")?
            .iter()
            .map(client_attribute_state)
            .collect::<Result<Vec<_>, _>>()?;
        let operation_player_id = i32_field(operation, "player_id")?;
        let accepted = world.apply_local_player_attributes(operation_player_id, &attributes);
        let keys = Input::new(
            operation["up"].as_bool().ok_or("up must be a boolean")?,
            operation["down"]
                .as_bool()
                .ok_or("down must be a boolean")?,
            operation["left"]
                .as_bool()
                .ok_or("left must be a boolean")?,
            operation["right"]
                .as_bool()
                .ok_or("right must be a boolean")?,
            operation["jump"]
                .as_bool()
                .ok_or("jump must be a boolean")?,
            operation["shift"]
                .as_bool()
                .ok_or("shift must be a boolean")?,
            operation["sprint"]
                .as_bool()
                .ok_or("sprint must be a boolean")?,
        );
        let mut keyboard = KeyboardInput::new(keys);
        keyboard.tick();
        let modifiers = LocalPlayerInputModifiers {
            using_item: operation["using_item"]
                .as_bool()
                .ok_or("using_item must be a boolean")?,
            passenger: operation["passenger"]
                .as_bool()
                .ok_or("passenger must be a boolean")?,
            item_use_speed_multiplier: f32_field(operation, "item_use_speed_multiplier")?,
            moving_slowly: operation["moving_slowly"]
                .as_bool()
                .ok_or("moving_slowly must be a boolean")?,
            sneaking_movement_factor: 0.3,
        };
        if !world.apply_local_player_keyboard_input(
            keys,
            operation["controlled_camera"]
                .as_bool()
                .ok_or("controlled_camera must be a boolean")?,
            modifiers,
        ) {
            return Err("local player is not initialized".into());
        }
        let player = world
            .local_player()
            .ok_or("local player is not initialized")?;
        states.push(client_world_player_keyboard_input_state(
            string(operation, "label")?,
            accepted,
            &keyboard,
            player,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_world_player_tick_state(label: &str, player: &voxel_world::ClientPlayerState) -> Value {
    let position = player.position();
    let previous_position = player.previous_position();
    let rotation = player.rotation();
    let previous_rotation = player.previous_rotation();
    let effects = player
        .effects()
        .iter()
        .map(|effect| {
            json!({
                "effect_id": effect.effect_id(),
                "amplifier": effect.amplifier(),
                "duration_ticks": effect.duration_ticks(),
                "flags": effect.flags(),
            })
        })
        .collect::<Vec<_>>();
    json!({
        "label": label,
        "tick_count": player.tick_count(),
        "position": vec3_snapshot(Vec3::new(position.0, position.1, position.2)),
        "previous_position": vec3_snapshot(Vec3::new(
            previous_position.0,
            previous_position.1,
            previous_position.2,
        )),
        "rotation_y": format!("{:08x}", rotation.0.to_bits()),
        "rotation_x": format!("{:08x}", rotation.1.to_bits()),
        "previous_rotation_y": format!("{:08x}", previous_rotation.0.to_bits()),
        "previous_rotation_x": format!("{:08x}", previous_rotation.1.to_bits()),
        "head_rotation": format!("{:08x}", player.head_rotation().to_bits()),
        "previous_head_rotation": format!("{:08x}", player.previous_head_rotation().to_bits()),
        "effects": effects,
    })
}

fn client_world_player_tick_result(request: &Value) -> Result<Value, String> {
    let player_id = i32_field(request, "player_id")?;
    let initial_position = f64_vec3(
        request
            .get("initial_position")
            .ok_or("initial_position must be a vector")?,
    )?;
    let initial_rotation = request
        .get("initial_rotation")
        .ok_or("initial_rotation must be an object")?;
    let initial_y_rotation = f32_field(initial_rotation, "y")?;
    let initial_x_rotation = f32_field(initial_rotation, "x")?;
    let initial_head = f32_field(request, "initial_head")?;

    let mut world = ClientWorld::new(DimensionBounds::new(0, 256));
    world.set_local_player_id(player_id);
    if !world.apply_entity(EntityInput {
        entity_id: player_id,
        uuid: Uuid::new(0, u64::from(player_id as u32)),
        type_id: 156,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, initial_head),
        data: 0,
    }) {
        return Err("unable to initialize local player entity state".into());
    }
    if !world.apply_local_player_position(
        (initial_position.x, initial_position.y, initial_position.z),
        (0.0, 0.0, 0.0),
        (initial_y_rotation, initial_x_rotation),
    ) {
        return Err("local player is not initialized".into());
    }
    for entry in request
        .get("initial_effects")
        .ok_or("initial_effects must be an array")?
        .as_array()
        .ok_or("initial_effects must be an array")?
    {
        let effect = entry
            .as_object()
            .ok_or("initial effect must be an object")?;
        if !world.apply_local_player_mob_effect(
            player_id,
            i32_field(entry, "effect_id")?,
            i32_field(entry, "amplifier")?,
            i32_field(entry, "duration_ticks")?,
            effect
                .get("flags")
                .and_then(Value::as_u64)
                .ok_or("flags must be an unsigned integer")?
                .try_into()
                .map_err(|_| "flags must fit in a byte")?,
        ) {
            return Err("local player effect could not be applied".into());
        }
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());
    for operation in operations {
        if string(operation, "action")? != "tick" {
            return Err("unknown player tick action".into());
        }
        let position = f64_vec3(
            operation
                .get("position")
                .ok_or("position must be a vector")?,
        )?;
        let rotation = operation
            .get("rotation")
            .ok_or("rotation must be an object")?;
        if !world.apply_local_player_position(
            (position.x, position.y, position.z),
            (0.0, 0.0, 0.0),
            (f32_field(rotation, "y")?, f32_field(rotation, "x")?),
        ) {
            return Err("local player is not initialized".into());
        }
        if let Some(head_target) = operation
            .get("head_target")
            .filter(|value| !value.is_null())
            && !world.apply_local_player_head_rotation(f32_value(head_target, "head_target")?)
        {
            return Err("local player head target could not be applied".into());
        }
        for entry in operation
            .get("effects")
            .ok_or("effects must be an array")?
            .as_array()
            .ok_or("effects must be an array")?
        {
            let effect = entry.as_object().ok_or("effect must be an object")?;
            if !world.apply_local_player_mob_effect(
                player_id,
                i32_field(entry, "effect_id")?,
                i32_field(entry, "amplifier")?,
                i32_field(entry, "duration_ticks")?,
                effect
                    .get("flags")
                    .and_then(Value::as_u64)
                    .ok_or("flags must be an unsigned integer")?
                    .try_into()
                    .map_err(|_| "flags must fit in a byte")?,
            ) {
                return Err("local player effect could not be applied".into());
            }
        }
        let ticks = operation["ticks"]
            .as_u64()
            .ok_or("ticks must be an unsigned integer")?;
        for _ in 0..ticks {
            world.tick();
        }
        let player = world
            .local_player()
            .ok_or("local player is not initialized")?;
        states.push(client_world_player_tick_state(
            string(operation, "label")?,
            player,
        ));
    }
    Ok(json!({"player_id": player_id, "states": states}))
}

fn client_world_entity_player_head_rotation_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    let rotation = entity.rotation();
    let previous_rotation = entity.previous_rotation();
    json!({
        "label": label,
        "accepted": accepted,
        "head_rotation": format!("{:08x}", rotation.2.to_bits()),
        "previous_head_rotation": format!("{:08x}", previous_rotation.2.to_bits()),
        "head_target": format!("{:08x}", entity.head_rotation_target().to_bits()),
        "head_lerp_steps": entity.head_rotation_lerp_steps(),
        "body_rotation_y": format!("{:08x}", entity.body_y_rotation().to_bits()),
    })
}

fn client_world_entity_player_head_rotation_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let initial_position = f64_vec3(
        request
            .get("initial_position")
            .ok_or("initial_position must be a vector")?,
    )?;
    let initial_rotation = request
        .get("initial_rotation")
        .ok_or("initial_rotation must be an object")?;
    let initial_head = f32_field(request, "initial_head")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: initial_position.x,
        y: initial_position.y,
        z: initial_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (
            f32_field(initial_rotation, "y")?,
            f32_field(initial_rotation, "x")?,
            initial_head,
        ),
        data: 0,
    }) {
        return Err("remote player head fixture entity was rejected".into());
    }

    let entity = world
        .entity_at(entity_id)
        .ok_or("remote player head fixture entity disappeared")?;
    let mut states = vec![client_world_entity_player_head_rotation_state(
        "initial", true, entity,
    )];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        if string(operation, "action")? != "rotate_head" {
            return Err("unknown remote player head action".into());
        }
        if let Some(head_target) = operation
            .get("head_target")
            .filter(|value| !value.is_null())
            && !world.apply_entity_head_rotation(entity_id, f32_value(head_target, "head_target")?)
        {
            return Err("remote player head target could not be applied".into());
        }
        let ticks = operation["ticks"]
            .as_u64()
            .ok_or("ticks must be an unsigned integer")?;
        for _ in 0..ticks {
            world.tick();
        }
        let entity = world
            .entity_at(entity_id)
            .ok_or("remote player head fixture entity disappeared")?;
        states.push(client_world_entity_player_head_rotation_state(
            string(operation, "label")?,
            true,
            entity,
        ));
    }
    Ok(json!({"entity_id": entity_id, "states": states}))
}

fn client_world_entity_tick_vector(vector: (f64, f64, f64)) -> Value {
    json!({
        "x": format!("{:016x}", vector.0.to_bits()),
        "y": format!("{:016x}", vector.1.to_bits()),
        "z": format!("{:016x}", vector.2.to_bits()),
    })
}

fn client_world_entity_tick_state(
    label: &str,
    accepted: bool,
    entity_ids: &[i32],
    world: &ClientWorld,
) -> Result<Value, String> {
    let entities = entity_ids
        .iter()
        .map(|entity_id| {
            let entity = world
                .entity_at(*entity_id)
                .ok_or_else(|| "entity tick fixture entity disappeared".to_owned())?;
            let rotation = entity.rotation();
            let previous_rotation = entity.previous_rotation();
            Ok(json!({
                "entity_id": entity.id(),
                "tick_count": entity.tick_count(),
                "position": client_world_entity_tick_vector(entity.position()),
                "previous_position": client_world_entity_tick_vector(
                    entity.previous_position()
                ),
                "motion": client_world_entity_tick_vector(entity.motion()),
                "rotation_y": format!("{:08x}", rotation.0.to_bits()),
                "rotation_x": format!("{:08x}", rotation.1.to_bits()),
                "previous_rotation_y": format!("{:08x}", previous_rotation.0.to_bits()),
                "previous_rotation_x": format!("{:08x}", previous_rotation.1.to_bits()),
                "no_gravity": entity.is_no_gravity(),
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(json!({
        "label": label,
        "accepted": accepted,
        "entities": entities,
    }))
}

fn client_world_entity_tick_result(request: &Value, entity_type_id: i32) -> Result<Value, String> {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ) {
        return Err("entity tick fixture chunk was rejected".into());
    }

    let input_entities = request["entities"]
        .as_array()
        .ok_or("entities must be an array")?;
    let mut entity_ids = Vec::with_capacity(input_entities.len());
    for entry in input_entities {
        let entity_id = i32_field(entry, "entity_id")?;
        let position = f64_vec3(entry.get("position").ok_or("position must be a vector")?)?;
        let movement = f64_vec3(entry.get("movement").ok_or("movement must be a vector")?)?;
        let rotation = entry.get("rotation").ok_or("rotation must be an object")?;
        if !world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(0, u64::from(entity_id as u32)),
            type_id: entity_type_id,
            x: position.x,
            y: position.y,
            z: position.z,
            movement: (movement.x, movement.y, movement.z),
            rotation: (f32_field(rotation, "y")?, f32_field(rotation, "x")?, 0.0),
            data: 0,
        }) {
            return Err("entity tick fixture entity was rejected".into());
        }
        if entry["no_gravity"]
            .as_bool()
            .ok_or("no_gravity must be a boolean")?
            && !world.apply_entity_metadata(entity_id, &[5, 8, 1, 255])
        {
            return Err("entity tick no-gravity metadata was rejected".into());
        }
        entity_ids.push(entity_id);
    }

    let mut states = vec![client_world_entity_tick_state(
        "initial",
        true,
        &entity_ids,
        &world,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        if string(operation, "action")? != "tick" {
            return Err("unknown entity tick action".into());
        }
        let ticks = operation["ticks"]
            .as_u64()
            .ok_or("ticks must be an unsigned integer")?;
        for _ in 0..ticks {
            world.tick();
        }
        states.push(client_world_entity_tick_state(
            string(operation, "label")?,
            true,
            &entity_ids,
            &world,
        )?);
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_ticking_eligibility_state(
    label: &str,
    world: &ClientWorld,
    player_id: i32,
    remote_entity_id: i32,
) -> Result<Value, String> {
    let player = world
        .local_player()
        .ok_or("ticking eligibility local player disappeared")?;
    let remote = world
        .entity_at(remote_entity_id)
        .ok_or("ticking eligibility remote entity disappeared")?;
    Ok(json!({
        "label": label,
        "accepted": true,
        "game_time": world.game_time(),
        "local_player": {
            "entity_id": player_id,
            "tick_count": player.tick_count(),
            "always_ticking": true,
            "ticking": true,
        },
        "entities": [{
            "entity_id": remote.id(),
            "tick_count": remote.tick_count(),
            "always_ticking": remote.is_always_ticking(),
            "ticking": remote.is_ticking(),
        }],
    }))
}

fn client_world_entity_ticking_eligibility_result(request: &Value) -> Result<Value, String> {
    let player = request.get("player").ok_or("player must be an object")?;
    let player_id = i32_field(player, "entity_id")?;
    let player_position = f64_vec3(
        player
            .get("position")
            .ok_or("player position must be a vector")?,
    )?;
    let remote = request.get("remote").ok_or("remote must be an object")?;
    let remote_entity_id = i32_field(remote, "entity_id")?;
    let remote_position = f64_vec3(
        remote
            .get("position")
            .ok_or("remote position must be a vector")?,
    )?;
    let remote_type_id = i32_field(remote, "type_id")?;

    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        -3,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ) {
        return Err("ticking eligibility fixture chunk was rejected".into());
    }
    world.set_local_player_id(player_id);
    if !world.apply_local_player_position(
        (player_position.x, player_position.y, player_position.z),
        (0.0, 0.0, 0.0),
        (0.0, 0.0),
    ) {
        return Err("ticking eligibility local player was rejected".into());
    }
    if !world.apply_entity(EntityInput {
        entity_id: remote_entity_id,
        uuid: Uuid::new(0, u64::from(remote_entity_id as u32)),
        type_id: remote_type_id,
        x: remote_position.x,
        y: remote_position.y,
        z: remote_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("ticking eligibility remote entity was rejected".into());
    }

    let mut states = vec![client_world_entity_ticking_eligibility_state(
        "initial",
        &world,
        player_id,
        remote_entity_id,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        match string(operation, "action")? {
            "tick" => {
                let ticks = operation["ticks"]
                    .as_u64()
                    .ok_or("ticks must be an unsigned integer")?;
                for _ in 0..ticks {
                    world.tick();
                }
            }
            "unload_chunk" => {
                let chunk_x = i32_field(operation, "chunk_x")?;
                let chunk_z = i32_field(operation, "chunk_z")?;
                if !world.unload_chunk(chunk_x, chunk_z) {
                    return Err("ticking eligibility chunk unload was rejected".into());
                }
            }
            "load_chunk" => {
                let chunk_x = i32_field(operation, "chunk_x")?;
                let chunk_z = i32_field(operation, "chunk_z")?;
                if !world.replace_chunk(
                    chunk_x,
                    chunk_z,
                    [SectionInput {
                        non_empty_block_count: 0,
                        fluid_count: 0,
                        block_states: &blocks,
                        biomes: &biomes,
                    }],
                ) {
                    return Err("ticking eligibility chunk reload was rejected".into());
                }
            }
            _ => return Err("unknown ticking eligibility action".into()),
        }
        states.push(client_world_entity_ticking_eligibility_state(
            string(operation, "label")?,
            &world,
            player_id,
            remote_entity_id,
        )?);
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_transform_history_state(
    label: &str,
    world: &ClientWorld,
    player_id: i32,
    entity_id: i32,
    partial_tick: f32,
) -> Result<Value, String> {
    let player = world
        .local_player()
        .ok_or("transform history local player disappeared")?;
    let entity = world
        .entity_at(entity_id)
        .ok_or("transform history entity disappeared")?;
    let player_position = player.position();
    let player_previous_position = player.previous_position();
    let player_rotation = player.rotation();
    let player_previous_rotation = player.previous_rotation();
    let player_interpolated_position = player.interpolated_position(partial_tick);
    let player_interpolated_rotation = player.interpolated_rotation(partial_tick);
    let entity_position = entity.position();
    let entity_previous_position = entity.previous_position();
    let entity_rotation = entity.rotation();
    let entity_previous_rotation = entity.previous_rotation();
    let entity_interpolated_position = entity.interpolated_position(partial_tick);
    let entity_interpolated_rotation = entity.interpolated_rotation(partial_tick);
    Ok(json!({
        "label": label,
        "accepted": true,
        "game_time": world.game_time(),
        "partial_tick": format!("{:08x}", partial_tick.to_bits()),
        "local_player": {
            "entity_id": player_id,
            "tick_count": player.tick_count(),
            "position": client_world_entity_tick_vector(player_position),
            "previous_position": client_world_entity_tick_vector(player_previous_position),
            "rotation_y": format!("{:08x}", player_rotation.0.to_bits()),
            "rotation_x": format!("{:08x}", player_rotation.1.to_bits()),
            "previous_rotation_y": format!("{:08x}", player_previous_rotation.0.to_bits()),
            "previous_rotation_x": format!("{:08x}", player_previous_rotation.1.to_bits()),
            "interpolated_position": client_world_entity_tick_vector(
                player_interpolated_position,
            ),
            "interpolated_rotation_y": format!(
                "{:08x}",
                player_interpolated_rotation.0.to_bits()
            ),
            "interpolated_rotation_x": format!(
                "{:08x}",
                player_interpolated_rotation.1.to_bits()
            ),
        },
        "entities": [{
            "entity_id": entity.id(),
            "tick_count": entity.tick_count(),
            "position": client_world_entity_tick_vector(entity_position),
            "previous_position": client_world_entity_tick_vector(entity_previous_position),
            "rotation_y": format!("{:08x}", entity_rotation.0.to_bits()),
            "rotation_x": format!("{:08x}", entity_rotation.1.to_bits()),
            "previous_rotation_y": format!("{:08x}", entity_previous_rotation.0.to_bits()),
            "previous_rotation_x": format!("{:08x}", entity_previous_rotation.1.to_bits()),
            "head_rotation": format!("{:08x}", entity_rotation.2.to_bits()),
            "previous_head_rotation": format!("{:08x}", entity_previous_rotation.2.to_bits()),
            "interpolated_position": client_world_entity_tick_vector(
                entity_interpolated_position,
            ),
            "interpolated_rotation_y": format!(
                "{:08x}",
                entity_interpolated_rotation.0.to_bits()
            ),
            "interpolated_rotation_x": format!(
                "{:08x}",
                entity_interpolated_rotation.1.to_bits()
            ),
            "interpolated_head_rotation": format!(
                "{:08x}",
                entity_interpolated_rotation.2.to_bits()
            ),
        }],
    }))
}

fn client_world_entity_transform_history_result(request: &Value) -> Result<Value, String> {
    let player = request.get("player").ok_or("player must be an object")?;
    let player_id = i32_field(player, "entity_id")?;
    let player_position = f64_vec3(
        player
            .get("position")
            .ok_or("player position must be a vector")?,
    )?;
    let player_rotation = player
        .get("rotation")
        .ok_or("player rotation must be an object")?;
    let remote = request.get("entity").ok_or("entity must be an object")?;
    let entity_id = i32_field(remote, "entity_id")?;
    let entity_type_id = i32_field(remote, "type_id")?;
    let entity_position = f64_vec3(
        remote
            .get("position")
            .ok_or("entity position must be a vector")?,
    )?;
    let entity_rotation = remote
        .get("rotation")
        .ok_or("entity rotation must be an object")?;

    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        -3,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ) {
        return Err("transform history fixture chunk was rejected".into());
    }
    world.set_local_player_id(player_id);
    if !world.apply_local_player_position(
        (player_position.x, player_position.y, player_position.z),
        (0.0, 0.0, 0.0),
        (
            f32_field(player_rotation, "y")?,
            f32_field(player_rotation, "x")?,
        ),
    ) {
        return Err("transform history local player was rejected".into());
    }
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: entity_type_id,
        x: entity_position.x,
        y: entity_position.y,
        z: entity_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (
            f32_field(entity_rotation, "y")?,
            f32_field(entity_rotation, "x")?,
            f32_field(entity_rotation, "head")?,
        ),
        data: 0,
    }) {
        return Err("transform history entity was rejected".into());
    }

    let mut states = vec![client_world_entity_transform_history_state(
        "initial", &world, player_id, entity_id, 0.5,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        match string(operation, "action")? {
            "tick" => {
                let ticks = operation["ticks"]
                    .as_u64()
                    .ok_or("ticks must be an unsigned integer")?;
                for _ in 0..ticks {
                    world.tick();
                }
            }
            "update" => {
                let updated_player = operation
                    .get("player")
                    .ok_or("updated player must be an object")?;
                let updated_player_position = f64_vec3(
                    updated_player
                        .get("position")
                        .ok_or("updated player position must be a vector")?,
                )?;
                let updated_player_rotation = updated_player
                    .get("rotation")
                    .ok_or("updated player rotation must be an object")?;
                if !world.apply_local_player_position(
                    (
                        updated_player_position.x,
                        updated_player_position.y,
                        updated_player_position.z,
                    ),
                    (0.0, 0.0, 0.0),
                    (
                        f32_field(updated_player_rotation, "y")?,
                        f32_field(updated_player_rotation, "x")?,
                    ),
                ) {
                    return Err("updated transform history local player was rejected".into());
                }

                let updated_entity = operation
                    .get("entity")
                    .ok_or("updated entity must be an object")?;
                let updated_entity_position = f64_vec3(
                    updated_entity
                        .get("position")
                        .ok_or("updated entity position must be a vector")?,
                )?;
                if !world.update_entity_position(
                    entity_id,
                    (
                        updated_entity_position.x,
                        updated_entity_position.y,
                        updated_entity_position.z,
                    ),
                ) {
                    return Err("updated transform history entity was rejected".into());
                }
                if !world.apply_entity_head_rotation(entity_id, f32_field(updated_entity, "head")?)
                {
                    return Err("updated transform history head rotation was rejected".into());
                }
            }
            "sample" => {}
            _ => return Err("unknown transform history action".into()),
        }
        let partial_tick = if operation.get("partial_tick").is_some() {
            f32_field(operation, "partial_tick")?
        } else {
            0.5
        };
        states.push(client_world_entity_transform_history_state(
            string(operation, "label")?,
            &world,
            player_id,
            entity_id,
            partial_tick,
        )?);
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_snowball_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_tick_result(request, 120)
}

fn client_world_entity_player_interpolation_state(label: &str, entity: &ClientEntity) -> Value {
    let position = entity.position();
    let previous_position = entity.previous_position();
    let rotation = entity.rotation();
    let previous_rotation = entity.previous_rotation();
    let interpolation_active = entity.interpolation_steps() > 0;
    let mut state = json!({
        "label": label,
        "accepted": true,
        "tick_count": entity.tick_count(),
        "position": client_world_entity_tick_vector(position),
        "previous_position": client_world_entity_tick_vector(previous_position),
        "rotation_y": format!("{:08x}", rotation.0.to_bits()),
        "rotation_x": format!("{:08x}", rotation.1.to_bits()),
        "previous_rotation_y": format!("{:08x}", previous_rotation.0.to_bits()),
        "previous_rotation_x": format!("{:08x}", previous_rotation.1.to_bits()),
        "interpolation_active": interpolation_active,
        "interpolation_steps": entity.interpolation_steps(),
    });
    if let Some(target_position) = entity.interpolation_target_position() {
        state["interpolation_target_position"] = client_world_entity_tick_vector(target_position);
    }
    if let Some(target_rotation) = entity.interpolation_target_rotation() {
        state["interpolation_target_rotation"] = json!({
            "y": format!("{:08x}", target_rotation.0.to_bits()),
            "x": format!("{:08x}", target_rotation.1.to_bits()),
        });
    }
    state
}

fn client_world_entity_player_interpolation_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let initial_position = f64_vec3(
        request
            .get("initial_position")
            .ok_or("initial_position must be a vector")?,
    )?;
    let initial_rotation = request
        .get("initial_rotation")
        .ok_or("initial_rotation must be an object")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: 156,
        x: initial_position.x,
        y: initial_position.y,
        z: initial_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (
            f32_field(initial_rotation, "y")?,
            f32_field(initial_rotation, "x")?,
            f32_field(request, "initial_head")?,
        ),
        data: 0,
    }) {
        return Err("remote player interpolation fixture was rejected".into());
    }

    let mut states = vec![client_world_entity_player_interpolation_state(
        "initial",
        world
            .entity_at(entity_id)
            .ok_or("remote player interpolation fixture disappeared")?,
    )];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        match string(operation, "action")? {
            "move" => {
                let delta = operation.get("delta").ok_or("delta must be an object")?;
                let delta_i16 = |name: &str| {
                    delta[name]
                        .as_i64()
                        .ok_or_else(|| format!("delta {name} must be an integer"))?
                        .try_into()
                        .map_err(|_| format!("delta {name} must fit in a short"))
                };
                let rotation = operation
                    .get("rotation")
                    .ok_or("rotation must be an object")?;
                if !world.apply_entity_move(EntityMoveInput {
                    entity_id,
                    delta_x: delta_i16("x")?,
                    delta_y: delta_i16("y")?,
                    delta_z: delta_i16("z")?,
                    has_position: true,
                    y_rot: Some(f32_field(rotation, "y")?),
                    x_rot: Some(f32_field(rotation, "x")?),
                }) {
                    return Err("remote player interpolation move was rejected".into());
                }
            }
            "tick" => {
                let ticks = operation["ticks"]
                    .as_u64()
                    .ok_or("ticks must be an unsigned integer")?;
                for _ in 0..ticks {
                    world.tick();
                }
            }
            _ => return Err("unknown remote player interpolation action".into()),
        }
        states.push(client_world_entity_player_interpolation_state(
            string(operation, "label")?,
            world
                .entity_at(entity_id)
                .ok_or("remote player interpolation fixture disappeared")?,
        ));
    }
    Ok(json!({"entity_id": entity_id, "states": states}))
}

fn client_world_entity_player_snap_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let initial_position = f64_vec3(
        request
            .get("initial_position")
            .ok_or("initial_position must be a vector")?,
    )?;
    let initial_rotation = request
        .get("initial_rotation")
        .ok_or("initial_rotation must be an object")?;
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    for chunk_x in [0, 4, 5, 9, 13] {
        if !world.replace_chunk(
            chunk_x,
            -3,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        ) {
            return Err("remote player snap fixture chunk was rejected".into());
        }
    }
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: 156,
        x: initial_position.x,
        y: initial_position.y,
        z: initial_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (
            f32_field(initial_rotation, "y")?,
            f32_field(initial_rotation, "x")?,
            f32_field(request, "initial_head")?,
        ),
        data: 0,
    }) {
        return Err("remote player snap fixture was rejected".into());
    }

    let mut states = vec![client_world_entity_player_interpolation_state(
        "initial",
        world
            .entity_at(entity_id)
            .ok_or("remote player snap fixture disappeared")?,
    )];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        match string(operation, "action")? {
            "position_sync" => {
                let target = f64_vec3(
                    operation
                        .get("target_position")
                        .ok_or("target position must be a vector")?,
                )?;
                let rotation = operation
                    .get("rotation")
                    .ok_or("rotation must be an object")?;
                if !world.apply_entity_position_sync(
                    entity_id,
                    (target.x, target.y, target.z),
                    (f32_field(rotation, "y")?, f32_field(rotation, "x")?),
                ) {
                    return Err("remote player snap position sync was rejected".into());
                }
            }
            "tick" => {
                let ticks = operation["ticks"]
                    .as_u64()
                    .ok_or("ticks must be an unsigned integer")?;
                for _ in 0..ticks {
                    world.tick();
                }
            }
            _ => return Err("unknown remote player snap action".into()),
        }
        states.push(client_world_entity_player_interpolation_state(
            string(operation, "label")?,
            world
                .entity_at(entity_id)
                .ok_or("remote player snap fixture disappeared")?,
        ));
    }
    Ok(json!({"entity_id": entity_id, "states": states}))
}

fn client_world_entity_player_motion_state(label: &str, entity: &ClientEntity) -> Value {
    let position = entity.position();
    let previous_position = entity.previous_position();
    let motion = entity.motion();
    let mut state = json!({
        "label": label,
        "accepted": true,
        "tick_count": entity.tick_count(),
        "position": client_world_entity_tick_vector(position),
        "previous_position": client_world_entity_tick_vector(previous_position),
        "motion": client_world_entity_tick_vector(motion),
        "motion_steps": entity.motion_interpolation_steps(),
    });
    if let Some(target_motion) = entity.motion_interpolation_target() {
        state["motion_target"] = client_world_entity_tick_vector(target_motion);
    }
    state
}

fn client_world_entity_player_motion_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let initial_position = f64_vec3(
        request
            .get("initial_position")
            .ok_or("initial_position must be a vector")?,
    )?;
    let initial_rotation = request
        .get("initial_rotation")
        .ok_or("initial_rotation must be an object")?;
    let initial_motion = f64_vec3(
        request
            .get("initial_motion")
            .ok_or("initial_motion must be a vector")?,
    )?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: 156,
        x: initial_position.x,
        y: initial_position.y,
        z: initial_position.z,
        movement: (initial_motion.x, initial_motion.y, initial_motion.z),
        rotation: (
            f32_field(initial_rotation, "y")?,
            f32_field(initial_rotation, "x")?,
            f32_field(request, "initial_head")?,
        ),
        data: 0,
    }) {
        return Err("remote player motion fixture was rejected".into());
    }

    let mut states = vec![client_world_entity_player_motion_state(
        "initial",
        world
            .entity_at(entity_id)
            .ok_or("remote player motion fixture disappeared")?,
    )];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        match string(operation, "action")? {
            "motion" => {
                let target = f64_vec3(
                    operation
                        .get("target_motion")
                        .ok_or("target motion must be a vector")?,
                )?;
                if !world.apply_entity_motion(EntityMotionInput {
                    entity_id,
                    x: target.x,
                    y: target.y,
                    z: target.z,
                }) {
                    return Err("remote player motion was rejected".into());
                }
            }
            "tick" => {
                let ticks = operation["ticks"]
                    .as_u64()
                    .ok_or("ticks must be an unsigned integer")?;
                for _ in 0..ticks {
                    world.tick();
                }
            }
            _ => return Err("unknown remote player motion action".into()),
        }
        states.push(client_world_entity_player_motion_state(
            string(operation, "label")?,
            world
                .entity_at(entity_id)
                .ok_or("remote player motion fixture disappeared")?,
        ));
    }
    Ok(json!({"entity_id": entity_id, "states": states}))
}

fn client_world_entity_teleport_state(label: &str, entity: &ClientEntity) -> Value {
    let mut state = client_world_entity_player_interpolation_state(label, entity);
    state["motion"] = client_world_entity_tick_vector(entity.motion());
    state
}

fn client_world_entity_teleport_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let initial_position = f64_vec3(
        request
            .get("initial_position")
            .ok_or("initial_position must be a vector")?,
    )?;
    let initial_rotation = request
        .get("initial_rotation")
        .ok_or("initial_rotation must be an object")?;
    let initial_motion = f64_vec3(
        request
            .get("initial_motion")
            .ok_or("initial_motion must be a vector")?,
    )?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: 156,
        x: initial_position.x,
        y: initial_position.y,
        z: initial_position.z,
        movement: (initial_motion.x, initial_motion.y, initial_motion.z),
        rotation: (
            f32_field(initial_rotation, "y")?,
            f32_field(initial_rotation, "x")?,
            f32_field(request, "initial_head")?,
        ),
        data: 0,
    }) {
        return Err("remote entity teleport fixture was rejected".into());
    }

    let mut states = vec![client_world_entity_teleport_state(
        "initial",
        world
            .entity_at(entity_id)
            .ok_or("remote entity teleport fixture disappeared")?,
    )];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        match string(operation, "action")? {
            "teleport" => {
                let position = f64_vec3(
                    operation
                        .get("position")
                        .ok_or("teleport position must be a vector")?,
                )?;
                let delta_motion = f64_vec3(
                    operation
                        .get("delta_motion")
                        .ok_or("teleport delta motion must be a vector")?,
                )?;
                let rotation = operation
                    .get("rotation")
                    .ok_or("teleport rotation must be an object")?;
                if !world.apply_entity_teleport(EntityTeleportInput {
                    entity_id,
                    x: position.x,
                    y: position.y,
                    z: position.z,
                    delta_x: delta_motion.x,
                    delta_y: delta_motion.y,
                    delta_z: delta_motion.z,
                    y_rot: f32_field(rotation, "y")?,
                    x_rot: f32_field(rotation, "x")?,
                    relative_flags: i32_field(operation, "relative_flags")?,
                }) {
                    return Err("remote entity teleport was rejected".into());
                }
            }
            "tick" => {
                let ticks = operation["ticks"]
                    .as_u64()
                    .ok_or("ticks must be an unsigned integer")?;
                for _ in 0..ticks {
                    world.tick();
                }
            }
            _ => return Err("unknown remote entity teleport action".into()),
        }
        states.push(client_world_entity_teleport_state(
            string(operation, "label")?,
            world
                .entity_at(entity_id)
                .ok_or("remote entity teleport fixture disappeared")?,
        ));
    }
    Ok(json!({"entity_id": entity_id, "states": states}))
}

fn client_world_entity_passengers_state(
    label: &str,
    accepted: bool,
    world: &ClientWorld,
    entity_ids: &[i32],
) -> Result<Value, String> {
    let mut entities = Vec::with_capacity(entity_ids.len());
    for &entity_id in entity_ids {
        let entity = world
            .entity_at(entity_id)
            .ok_or("passenger relation fixture entity disappeared")?;
        let mut state = json!({
            "entity_id": entity_id,
            "passenger_ids": entity.passenger_ids(),
        });
        if let Some(vehicle_id) = entity.vehicle_id() {
            state["vehicle_id"] = json!(vehicle_id);
        }
        entities.push(state);
    }
    Ok(json!({
        "label": label,
        "accepted": accepted,
        "entities": entities,
    }))
}

fn client_world_entity_passengers_result(request: &Value) -> Result<Value, String> {
    let entity_entries = request["entities"]
        .as_array()
        .ok_or("entities must be an array")?;
    let entity_ids: Vec<i32> = entity_entries
        .iter()
        .map(|entity| i32_field(entity, "entity_id"))
        .collect::<Result<_, _>>()?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    for entity in entity_entries {
        let entity_id = i32_field(entity, "entity_id")?;
        let type_id = i32_field(entity, "type_id")?;
        if !world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(0, u64::from(entity_id as u32)),
            type_id,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }) {
            return Err("passenger relation fixture entity was rejected".into());
        }
    }

    for link in request["initial_links"]
        .as_array()
        .ok_or("initial_links must be an array")?
    {
        let vehicle_id = i32_field(link, "vehicle_id")?;
        let passenger_ids = link["passenger_ids"]
            .as_array()
            .ok_or("initial passenger_ids must be an array")?
            .iter()
            .map(|passenger| {
                passenger
                    .as_i64()
                    .and_then(|value| i32::try_from(value).ok())
                    .ok_or("initial passenger ID must be a 32-bit integer".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !world.apply_entity_passengers(vehicle_id, &passenger_ids) {
            return Err("initial passenger relation was rejected".into());
        }
    }

    let mut states = vec![client_world_entity_passengers_state(
        "initial",
        true,
        &world,
        &entity_ids,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        let vehicle_id = i32_field(operation, "vehicle_id")?;
        let passenger_ids = operation["passenger_ids"]
            .as_array()
            .ok_or("passenger_ids must be an array")?
            .iter()
            .map(|passenger| {
                passenger
                    .as_i64()
                    .and_then(|value| i32::try_from(value).ok())
                    .ok_or("passenger ID must be a 32-bit integer".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let accepted = world.apply_entity_passengers(vehicle_id, &passenger_ids);
        states.push(client_world_entity_passengers_state(
            string(operation, "label")?,
            accepted,
            &world,
            &entity_ids,
        )?);
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_passenger_tick_result(request: &Value) -> Result<Value, String> {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ) {
        return Err("passenger tick fixture chunk was rejected".into());
    }

    let vehicle = request["vehicle"]
        .as_object()
        .ok_or("vehicle must be an object")?;
    let passenger = request["passenger"]
        .as_object()
        .ok_or("passenger must be an object")?;
    let root = request["root_entity"]
        .as_object()
        .ok_or("root_entity must be an object")?;
    let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
    let passenger_id = i32_field(&Value::Object(passenger.clone()), "entity_id")?;
    let root_id = i32_field(&Value::Object(root.clone()), "entity_id")?;
    let entity_ids = [vehicle_id, passenger_id, root_id];

    for input in [vehicle, passenger, root] {
        let entity_id = i32_field(&Value::Object(input.clone()), "entity_id")?;
        let type_id = i32_field(&Value::Object(input.clone()), "type_id")?;
        let position = f64_vec3(
            input
                .get("position")
                .ok_or("passenger tick position must be a vector")?,
        )?;
        let movement = input
            .get("movement")
            .map(f64_vec3)
            .transpose()?
            .unwrap_or(Vec3::new(0.0, 0.0, 0.0));
        let rotation = if let Some(value) = input.get("rotation") {
            (f32_field(value, "y")?, f32_field(value, "x")?, 0.0)
        } else {
            (0.0, 0.0, 0.0)
        };
        if !world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(0, u64::from(entity_id as u32)),
            type_id,
            x: position.x,
            y: position.y,
            z: position.z,
            movement: (movement.x, movement.y, movement.z),
            rotation,
            data: 0,
        }) {
            return Err("passenger tick fixture entity was rejected".into());
        }
    }
    if !world.apply_entity_passengers(vehicle_id, &[passenger_id]) {
        return Err("passenger tick fixture relation was rejected".into());
    }

    let mut states = vec![client_world_entity_tick_state(
        "initial",
        true,
        &entity_ids,
        &world,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        if string(operation, "action")? != "tick" {
            return Err("unknown passenger tick action".into());
        }
        let ticks = operation["ticks"]
            .as_u64()
            .ok_or("ticks must be an unsigned integer")?;
        for _ in 0..ticks {
            world.tick();
        }
        states.push(client_world_entity_tick_state(
            string(operation, "label")?,
            true,
            &entity_ids,
            &world,
        )?);
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_passenger_position_result(request: &Value) -> Result<Value, String> {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ) {
        return Err("passenger position fixture chunk was rejected".into());
    }

    let vehicle = request["vehicle"]
        .as_object()
        .ok_or("vehicle must be an object")?;
    if i32_field(&Value::Object(vehicle.clone()), "type_id")? != 89 {
        return Err("passenger position fixture requires an oak boat".into());
    }
    let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
    let vehicle_position = f64_vec3(
        vehicle
            .get("position")
            .ok_or("passenger position vehicle position must be a vector")?,
    )?;
    let vehicle_rotation = vehicle
        .get("rotation")
        .ok_or("passenger position vehicle rotation must be an object")?;
    let vehicle_y_rot = f32_field(vehicle_rotation, "y")?;
    let mut entity_ids = vec![vehicle_id];
    for passenger in request["passengers"]
        .as_array()
        .ok_or("passengers must be an array")?
    {
        let passenger_id = i32_field(passenger, "entity_id")?;
        if i32_field(passenger, "type_id")? != 120 {
            return Err("passenger position fixture requires snowball passengers".into());
        }
        let position = f64_vec3(
            passenger
                .get("position")
                .ok_or("passenger position must be a vector")?,
        )?;
        let movement = f64_vec3(
            passenger
                .get("movement")
                .ok_or("passenger movement must be a vector")?,
        )?;
        let rotation = passenger
            .get("rotation")
            .ok_or("passenger rotation must be an object")?;
        if !world.apply_entity(EntityInput {
            entity_id: passenger_id,
            uuid: Uuid::new(0, u64::from(passenger_id as u32)),
            type_id: 120,
            x: position.x,
            y: position.y,
            z: position.z,
            movement: (movement.x, movement.y, movement.z),
            rotation: (f32_field(rotation, "y")?, f32_field(rotation, "x")?, 0.0),
            data: 0,
        }) {
            return Err("passenger position fixture entity was rejected".into());
        }
        entity_ids.push(passenger_id);
    }
    if !world.apply_entity(EntityInput {
        entity_id: vehicle_id,
        uuid: Uuid::new(0, u64::from(vehicle_id as u32)),
        type_id: 89,
        x: vehicle_position.x,
        y: vehicle_position.y,
        z: vehicle_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (vehicle_y_rot, f32_field(vehicle_rotation, "x")?, 0.0),
        data: 0,
    }) {
        return Err("passenger position vehicle was rejected".into());
    }

    let passenger_ids = request["initial_passenger_ids"]
        .as_array()
        .ok_or("initial_passenger_ids must be an array")?
        .iter()
        .map(|value| {
            value
                .as_i64()
                .ok_or_else(|| "initial passenger ID must be an integer".to_owned())
                .and_then(|value| {
                    value
                        .try_into()
                        .map_err(|_| "initial passenger ID must fit in i32".to_owned())
                })
        })
        .collect::<Result<Vec<i32>, _>>()?;
    if !world.apply_entity_passengers(vehicle_id, &passenger_ids) {
        return Err("passenger position fixture relation was rejected".into());
    }

    let mut states = vec![client_world_entity_tick_state(
        "initial",
        true,
        &entity_ids,
        &world,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        match string(operation, "action")? {
            "tick" => {}
            "set-passengers-and-tick" => {
                let passenger_ids = operation["passenger_ids"]
                    .as_array()
                    .ok_or("passenger_ids must be an array")?
                    .iter()
                    .map(|value| {
                        value
                            .as_i64()
                            .ok_or_else(|| "passenger ID must be an integer".to_owned())
                            .and_then(|value| {
                                value
                                    .try_into()
                                    .map_err(|_| "passenger ID must fit in i32".to_owned())
                            })
                    })
                    .collect::<Result<Vec<i32>, _>>()?;
                if !world.apply_entity_passengers(vehicle_id, &passenger_ids) {
                    return Err("passenger position fixture relation was rejected".into());
                }
            }
            _ => return Err("unknown passenger position action".into()),
        }
        let ticks = operation["ticks"]
            .as_u64()
            .ok_or("ticks must be an unsigned integer")?;
        for _ in 0..ticks {
            world.tick();
        }
        states.push(client_world_entity_tick_state(
            string(operation, "label")?,
            true,
            &entity_ids,
            &world,
        )?);
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_passenger_player_position_result(request: &Value) -> Result<Value, String> {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            block_states: &blocks,
            biomes: &biomes,
            fluid_count: 0,
        }],
    ) {
        return Err("passenger player position fixture chunk was rejected".into());
    }

    let vehicle = request["vehicle"]
        .as_object()
        .ok_or("vehicle must be an object")?;
    if i32_field(&Value::Object(vehicle.clone()), "type_id")? != 89 {
        return Err("passenger player position fixture requires an oak boat".into());
    }
    let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
    let vehicle_position = f64_vec3(
        vehicle
            .get("position")
            .ok_or("passenger player position vehicle position must be a vector")?,
    )?;
    let vehicle_rotation = vehicle
        .get("rotation")
        .ok_or("passenger player position vehicle rotation must be an object")?;
    if !world.apply_entity(EntityInput {
        entity_id: vehicle_id,
        uuid: Uuid::new(0, u64::from(vehicle_id as u32)),
        type_id: 89,
        x: vehicle_position.x,
        y: vehicle_position.y,
        z: vehicle_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (
            f32_field(vehicle_rotation, "y")?,
            f32_field(vehicle_rotation, "x")?,
            0.0,
        ),
        data: 0,
    }) {
        return Err("passenger player position vehicle was rejected".into());
    }
    if vehicle["no_gravity"]
        .as_bool()
        .ok_or("vehicle no_gravity must be a boolean")?
        && !world.apply_entity_metadata(vehicle_id, &[5, 8, 1, 255])
    {
        return Err("passenger player position vehicle no-gravity was rejected".into());
    }

    let passenger = request["passenger"]
        .as_object()
        .ok_or("passenger must be an object")?;
    if i32_field(&Value::Object(passenger.clone()), "type_id")? != 156 {
        return Err("passenger player position fixture requires a Player passenger".into());
    }
    let passenger_id = i32_field(&Value::Object(passenger.clone()), "entity_id")?;
    let passenger_position = f64_vec3(
        passenger
            .get("position")
            .ok_or("passenger player position must be a vector")?,
    )?;
    let passenger_movement = f64_vec3(
        passenger
            .get("movement")
            .ok_or("passenger player movement must be a vector")?,
    )?;
    let passenger_rotation = passenger
        .get("rotation")
        .ok_or("passenger player rotation must be an object")?;
    if !world.apply_entity(EntityInput {
        entity_id: passenger_id,
        uuid: Uuid::new(0, u64::from(passenger_id as u32)),
        type_id: 156,
        x: passenger_position.x,
        y: passenger_position.y,
        z: passenger_position.z,
        movement: (
            passenger_movement.x,
            passenger_movement.y,
            passenger_movement.z,
        ),
        rotation: (
            f32_field(passenger_rotation, "y")?,
            f32_field(passenger_rotation, "x")?,
            0.0,
        ),
        data: 0,
    }) {
        return Err("passenger player position fixture entity was rejected".into());
    }
    if passenger["no_gravity"]
        .as_bool()
        .ok_or("passenger no_gravity must be a boolean")?
        && !world.apply_entity_metadata(passenger_id, &[5, 8, 1, 255])
    {
        return Err("passenger player position no-gravity was rejected".into());
    }
    if !world.apply_entity_passengers(vehicle_id, &[passenger_id]) {
        return Err("passenger player position fixture relation was rejected".into());
    }

    let entity_ids = [vehicle_id, passenger_id];
    let mut states = vec![client_world_entity_tick_state(
        "initial",
        true,
        &entity_ids,
        &world,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        if string(operation, "action")? != "tick" {
            return Err("unknown passenger player position action".into());
        }
        let ticks = operation["ticks"]
            .as_u64()
            .ok_or("ticks must be an unsigned integer")?;
        for _ in 0..ticks {
            world.tick();
        }
        states.push(client_world_entity_tick_state(
            string(operation, "label")?,
            true,
            &entity_ids,
            &world,
        )?);
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_passenger_boat_family_position_result(
    request: &Value,
) -> Result<Value, String> {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ) {
        return Err("boat family position fixture chunk was rejected".into());
    }

    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut entity_ids = Vec::with_capacity(rows.len() * 2);
    for row in rows {
        let vehicle = row
            .get("vehicle")
            .and_then(Value::as_object)
            .ok_or("boat family vehicle must be an object")?;
        let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
        let vehicle_type_id = i32_field(&Value::Object(vehicle.clone()), "type_id")?;
        if !matches!(vehicle_type_id, 8 | 9 | 89 | 90) {
            return Err("unknown boat family position vehicle type".into());
        }
        let vehicle_position = f64_vec3(
            vehicle
                .get("position")
                .ok_or("boat family vehicle position must be a vector")?,
        )?;
        let vehicle_rotation = vehicle
            .get("rotation")
            .ok_or("boat family vehicle rotation must be an object")?;
        let vehicle_no_gravity = vehicle
            .get("no_gravity")
            .and_then(Value::as_bool)
            .ok_or("boat family vehicle no_gravity must be a boolean")?;
        if !world.apply_entity(EntityInput {
            entity_id: vehicle_id,
            uuid: Uuid::new(0, u64::from(vehicle_id as u32)),
            type_id: vehicle_type_id,
            x: vehicle_position.x,
            y: vehicle_position.y,
            z: vehicle_position.z,
            movement: (0.0, 0.0, 0.0),
            rotation: (
                f32_field(vehicle_rotation, "y")?,
                f32_field(vehicle_rotation, "x")?,
                0.0,
            ),
            data: 0,
        }) {
            return Err("boat family vehicle was rejected".into());
        }
        if vehicle_no_gravity && !world.apply_entity_metadata(vehicle_id, &[5, 8, 1, 255]) {
            return Err("boat family vehicle no-gravity was rejected".into());
        }

        let passenger = row
            .get("passenger")
            .and_then(Value::as_object)
            .ok_or("boat family passenger must be an object")?;
        let passenger_id = i32_field(&Value::Object(passenger.clone()), "entity_id")?;
        if i32_field(&Value::Object(passenger.clone()), "type_id")? != 120 {
            return Err("boat family position fixture requires snowball passengers".into());
        }
        let passenger_position = f64_vec3(
            passenger
                .get("position")
                .ok_or("boat family passenger position must be a vector")?,
        )?;
        let passenger_movement = f64_vec3(
            passenger
                .get("movement")
                .ok_or("boat family passenger movement must be a vector")?,
        )?;
        let passenger_rotation = passenger
            .get("rotation")
            .ok_or("boat family passenger rotation must be an object")?;
        let passenger_no_gravity = passenger
            .get("no_gravity")
            .and_then(Value::as_bool)
            .ok_or("boat family passenger no_gravity must be a boolean")?;
        if !world.apply_entity(EntityInput {
            entity_id: passenger_id,
            uuid: Uuid::new(0, u64::from(passenger_id as u32)),
            type_id: 120,
            x: passenger_position.x,
            y: passenger_position.y,
            z: passenger_position.z,
            movement: (
                passenger_movement.x,
                passenger_movement.y,
                passenger_movement.z,
            ),
            rotation: (
                f32_field(passenger_rotation, "y")?,
                f32_field(passenger_rotation, "x")?,
                0.0,
            ),
            data: 0,
        }) {
            return Err("boat family passenger was rejected".into());
        }
        if passenger_no_gravity && !world.apply_entity_metadata(passenger_id, &[5, 8, 1, 255]) {
            return Err("boat family passenger no-gravity was rejected".into());
        }
        if !world.apply_entity_passengers(vehicle_id, &[passenger_id]) {
            return Err("boat family passenger relation was rejected".into());
        }
        entity_ids.push(vehicle_id);
        entity_ids.push(passenger_id);
    }

    let mut states = vec![client_world_entity_tick_state(
        "initial",
        true,
        &entity_ids,
        &world,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        if string(operation, "action")? != "tick" {
            return Err("unknown boat family position action".into());
        }
        let ticks = operation["ticks"]
            .as_u64()
            .ok_or("ticks must be an unsigned integer")?;
        for _ in 0..ticks {
            world.tick();
        }
        states.push(client_world_entity_tick_state(
            string(operation, "label")?,
            true,
            &entity_ids,
            &world,
        )?);
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_passenger_rotation_state(
    label: &str,
    entity_ids: &[i32],
    passenger_type_ids: &[i32],
    world: &ClientWorld,
) -> Result<Value, String> {
    let mut state = client_world_entity_tick_state(label, true, entity_ids, world)?;
    let entities = state["entities"]
        .as_array_mut()
        .ok_or("passenger rotation state entities must be an array")?;
    for (index, passenger_type_id) in passenger_type_ids.iter().enumerate() {
        if !matches!(*passenger_type_id, 17 | 156) {
            continue;
        }
        let entity_id = entity_ids
            .get(index * 2 + 1)
            .ok_or("passenger rotation state entity list is incomplete")?;
        let passenger = world
            .entity_at(*entity_id)
            .ok_or("passenger rotation fixture entity disappeared")?;
        let rotation = passenger.rotation();
        let previous_rotation = passenger.previous_rotation();
        let entity = entities
            .get_mut(index * 2 + 1)
            .and_then(Value::as_object_mut)
            .ok_or("passenger rotation state passenger is not an object")?;
        entity.insert(
            "head_rotation".into(),
            json!(format!("{:08x}", rotation.2.to_bits())),
        );
        entity.insert(
            "previous_head_rotation".into(),
            json!(format!("{:08x}", previous_rotation.2.to_bits())),
        );
        entity.insert(
            "body_rotation_y".into(),
            json!(format!("{:08x}", passenger.body_y_rotation().to_bits())),
        );
        entity.insert(
            "previous_body_rotation_y".into(),
            json!(format!(
                "{:08x}",
                passenger.previous_body_y_rotation().to_bits()
            )),
        );
    }
    Ok(state)
}

fn client_world_entity_passenger_animal_rotation_state(
    label: &str,
    entity_ids: &[i32],
    world: &ClientWorld,
) -> Result<Value, String> {
    let mut state = client_world_entity_tick_state(label, true, entity_ids, world)?;
    let entities = state["entities"]
        .as_array_mut()
        .ok_or("passenger animal rotation state entities must be an array")?;
    if !entity_ids.len().is_multiple_of(3) {
        return Err("passenger animal rotation state entity list is incomplete".into());
    }
    for row_index in 0..(entity_ids.len() / 3) {
        let animal_id = entity_ids
            .get(row_index * 3 + 1)
            .ok_or("passenger animal rotation state entity list is incomplete")?;
        let animal = world
            .entity_at(*animal_id)
            .ok_or("passenger animal rotation fixture entity disappeared")?;
        let rotation = animal.rotation();
        let previous_rotation = animal.previous_rotation();
        let entity = entities
            .get_mut(row_index * 3 + 1)
            .and_then(Value::as_object_mut)
            .ok_or("passenger animal rotation state animal is not an object")?;
        entity.insert(
            "head_rotation".into(),
            json!(format!("{:08x}", rotation.2.to_bits())),
        );
        entity.insert(
            "previous_head_rotation".into(),
            json!(format!("{:08x}", previous_rotation.2.to_bits())),
        );
        entity.insert(
            "body_rotation_y".into(),
            json!(format!("{:08x}", animal.body_y_rotation().to_bits())),
        );
        entity.insert(
            "previous_body_rotation_y".into(),
            json!(format!(
                "{:08x}",
                animal.previous_body_y_rotation().to_bits()
            )),
        );
    }
    Ok(state)
}

fn client_world_entity_passenger_rotation_result(request: &Value) -> Result<Value, String> {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ) {
        return Err("passenger rotation fixture chunk was rejected".into());
    }

    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut entity_ids = Vec::with_capacity(rows.len() * 2);
    let mut passenger_type_ids = Vec::with_capacity(rows.len());
    for row in rows {
        let vehicle = row
            .get("vehicle")
            .and_then(Value::as_object)
            .ok_or("passenger rotation vehicle must be an object")?;
        let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
        if i32_field(&Value::Object(vehicle.clone()), "type_id")? != 89 {
            return Err("passenger rotation fixture requires an oak boat".into());
        }
        let vehicle_position = f64_vec3(
            vehicle
                .get("position")
                .ok_or("passenger rotation vehicle position must be a vector")?,
        )?;
        let vehicle_rotation = vehicle
            .get("rotation")
            .ok_or("passenger rotation vehicle rotation must be an object")?;
        let vehicle_no_gravity = vehicle
            .get("no_gravity")
            .and_then(Value::as_bool)
            .ok_or("passenger rotation vehicle no_gravity must be a boolean")?;
        if !world.apply_entity(EntityInput {
            entity_id: vehicle_id,
            uuid: Uuid::new(0, u64::from(vehicle_id as u32)),
            type_id: 89,
            x: vehicle_position.x,
            y: vehicle_position.y,
            z: vehicle_position.z,
            movement: (0.0, 0.0, 0.0),
            rotation: (
                f32_field(vehicle_rotation, "y")?,
                f32_field(vehicle_rotation, "x")?,
                0.0,
            ),
            data: 0,
        }) {
            return Err("passenger rotation vehicle was rejected".into());
        }
        if vehicle_no_gravity && !world.apply_entity_metadata(vehicle_id, &[5, 8, 1, 255]) {
            return Err("passenger rotation vehicle no-gravity was rejected".into());
        }

        let passenger = row
            .get("passenger")
            .and_then(Value::as_object)
            .ok_or("passenger rotation passenger must be an object")?;
        let passenger_id = i32_field(&Value::Object(passenger.clone()), "entity_id")?;
        let passenger_type_id = i32_field(&Value::Object(passenger.clone()), "type_id")?;
        if !matches!(passenger_type_id, 120 | 156) {
            return Err("passenger rotation fixture requires snowball or Player passengers".into());
        }
        let passenger_position = f64_vec3(
            passenger
                .get("position")
                .ok_or("passenger rotation position must be a vector")?,
        )?;
        let passenger_movement = f64_vec3(
            passenger
                .get("movement")
                .ok_or("passenger rotation movement must be a vector")?,
        )?;
        let passenger_rotation = passenger
            .get("rotation")
            .ok_or("passenger rotation passenger rotation must be an object")?;
        let passenger_no_gravity = passenger
            .get("no_gravity")
            .and_then(Value::as_bool)
            .ok_or("passenger rotation passenger no_gravity must be a boolean")?;
        if !world.apply_entity(EntityInput {
            entity_id: passenger_id,
            uuid: Uuid::new(0, u64::from(passenger_id as u32)),
            type_id: passenger_type_id,
            x: passenger_position.x,
            y: passenger_position.y,
            z: passenger_position.z,
            movement: (
                passenger_movement.x,
                passenger_movement.y,
                passenger_movement.z,
            ),
            rotation: (
                f32_field(passenger_rotation, "y")?,
                f32_field(passenger_rotation, "x")?,
                f32_field(passenger_rotation, "y")?,
            ),
            data: 0,
        }) {
            return Err("passenger rotation passenger was rejected".into());
        }
        if passenger_no_gravity && !world.apply_entity_metadata(passenger_id, &[5, 8, 1, 255]) {
            return Err("passenger rotation passenger no-gravity was rejected".into());
        }
        if !world.apply_entity_passengers(vehicle_id, &[passenger_id]) {
            return Err("passenger rotation relation was rejected".into());
        }
        entity_ids.push(vehicle_id);
        entity_ids.push(passenger_id);
        passenger_type_ids.push(passenger_type_id);
    }

    let mut states = vec![client_world_entity_passenger_rotation_state(
        "initial",
        &entity_ids,
        &passenger_type_ids,
        &world,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        if string(operation, "action")? != "tick" {
            return Err("unknown passenger rotation action".into());
        }
        let ticks = operation["ticks"]
            .as_u64()
            .ok_or("ticks must be an unsigned integer")?;
        for _ in 0..ticks {
            world.tick();
        }
        states.push(client_world_entity_passenger_rotation_state(
            string(operation, "label")?,
            &entity_ids,
            &passenger_type_ids,
            &world,
        )?);
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_passenger_boat_turn_exemption_result(
    request: &Value,
) -> Result<Value, String> {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ) {
        return Err("passenger boat turn exemption fixture chunk was rejected".into());
    }

    let vehicle = request["vehicle"]
        .as_object()
        .ok_or("vehicle must be an object")?;
    if i32_field(&Value::Object(vehicle.clone()), "type_id")? != 89 {
        return Err("passenger boat turn exemption fixture requires an oak boat".into());
    }
    let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
    let vehicle_position = f64_vec3(
        vehicle
            .get("position")
            .ok_or("passenger boat turn exemption vehicle position must be a vector")?,
    )?;
    let vehicle_rotation = vehicle
        .get("rotation")
        .ok_or("passenger boat turn exemption vehicle rotation must be an object")?;
    if !world.apply_entity(EntityInput {
        entity_id: vehicle_id,
        uuid: Uuid::new(0, u64::from(vehicle_id as u32)),
        type_id: 89,
        x: vehicle_position.x,
        y: vehicle_position.y,
        z: vehicle_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (
            f32_field(vehicle_rotation, "y")?,
            f32_field(vehicle_rotation, "x")?,
            0.0,
        ),
        data: 0,
    }) {
        return Err("passenger boat turn exemption vehicle was rejected".into());
    }
    if vehicle["no_gravity"]
        .as_bool()
        .ok_or("vehicle no_gravity must be a boolean")?
        && !world.apply_entity_metadata(vehicle_id, &[5, 8, 1, 255])
    {
        return Err("passenger boat turn exemption vehicle no-gravity was rejected".into());
    }

    let passenger = request["passenger"]
        .as_object()
        .ok_or("passenger must be an object")?;
    if i32_field(&Value::Object(passenger.clone()), "type_id")? != 17 {
        return Err("passenger boat turn exemption fixture requires a Breeze passenger".into());
    }
    let passenger_id = i32_field(&Value::Object(passenger.clone()), "entity_id")?;
    let passenger_position = f64_vec3(
        passenger
            .get("position")
            .ok_or("passenger boat turn exemption position must be a vector")?,
    )?;
    let passenger_movement = f64_vec3(
        passenger
            .get("movement")
            .ok_or("passenger boat turn exemption movement must be a vector")?,
    )?;
    let passenger_rotation = passenger
        .get("rotation")
        .ok_or("passenger boat turn exemption rotation must be an object")?;
    if !world.apply_entity(EntityInput {
        entity_id: passenger_id,
        uuid: Uuid::new(0, u64::from(passenger_id as u32)),
        type_id: 17,
        x: passenger_position.x,
        y: passenger_position.y,
        z: passenger_position.z,
        movement: (
            passenger_movement.x,
            passenger_movement.y,
            passenger_movement.z,
        ),
        rotation: (
            f32_field(passenger_rotation, "y")?,
            f32_field(passenger_rotation, "x")?,
            f32_field(passenger_rotation, "y")?,
        ),
        data: 0,
    }) {
        return Err("passenger boat turn exemption passenger was rejected".into());
    }
    if passenger["no_gravity"]
        .as_bool()
        .ok_or("passenger no_gravity must be a boolean")?
        && !world.apply_entity_metadata(passenger_id, &[5, 8, 1, 255])
    {
        return Err("passenger boat turn exemption passenger no-gravity was rejected".into());
    }
    if !world.apply_entity_passengers(vehicle_id, &[passenger_id]) {
        return Err("passenger boat turn exemption relation was rejected".into());
    }

    let entity_ids = [vehicle_id, passenger_id];
    let passenger_type_ids = [17];
    let mut states = vec![client_world_entity_passenger_rotation_state(
        "initial",
        &entity_ids,
        &passenger_type_ids,
        &world,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        if string(operation, "action")? != "tick" {
            return Err("unknown passenger boat turn exemption action".into());
        }
        let ticks = operation["ticks"]
            .as_u64()
            .ok_or("ticks must be an unsigned integer")?;
        for _ in 0..ticks {
            world.tick();
        }
        states.push(client_world_entity_passenger_rotation_state(
            string(operation, "label")?,
            &entity_ids,
            &passenger_type_ids,
            &world,
        )?);
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_passenger_animal_rotation_result(request: &Value) -> Result<Value, String> {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ) {
        return Err("passenger animal rotation fixture chunk was rejected".into());
    }

    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut entity_ids = Vec::with_capacity(rows.len() * 3);
    for row in rows {
        let vehicle = row
            .get("vehicle")
            .and_then(Value::as_object)
            .ok_or("passenger animal rotation vehicle must be an object")?;
        let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
        if i32_field(&Value::Object(vehicle.clone()), "type_id")? != 89 {
            return Err("passenger animal rotation fixture requires an oak boat".into());
        }
        let vehicle_position = f64_vec3(
            vehicle
                .get("position")
                .ok_or("passenger animal rotation vehicle position must be a vector")?,
        )?;
        let vehicle_rotation = vehicle
            .get("rotation")
            .ok_or("passenger animal rotation vehicle rotation must be an object")?;
        if !world.apply_entity(EntityInput {
            entity_id: vehicle_id,
            uuid: Uuid::new(0, u64::from(vehicle_id as u32)),
            type_id: 89,
            x: vehicle_position.x,
            y: vehicle_position.y,
            z: vehicle_position.z,
            movement: (0.0, 0.0, 0.0),
            rotation: (
                f32_field(vehicle_rotation, "y")?,
                f32_field(vehicle_rotation, "x")?,
                0.0,
            ),
            data: 0,
        }) {
            return Err("passenger animal rotation vehicle was rejected".into());
        }
        if vehicle["no_gravity"]
            .as_bool()
            .ok_or("passenger animal rotation vehicle no_gravity must be a boolean")?
            && !world.apply_entity_metadata(vehicle_id, &[5, 8, 1, 255])
        {
            return Err("passenger animal rotation vehicle no-gravity was rejected".into());
        }

        let animal = row
            .get("animal")
            .and_then(Value::as_object)
            .ok_or("passenger animal rotation animal must be an object")?;
        let animal_id = i32_field(&Value::Object(animal.clone()), "entity_id")?;
        if i32_field(&Value::Object(animal.clone()), "type_id")? != 30 {
            return Err("passenger animal rotation fixture requires a Cow".into());
        }
        let animal_position = f64_vec3(
            animal
                .get("position")
                .ok_or("passenger animal rotation animal position must be a vector")?,
        )?;
        let animal_movement = f64_vec3(
            animal
                .get("movement")
                .ok_or("passenger animal rotation animal movement must be a vector")?,
        )?;
        let animal_rotation = animal
            .get("rotation")
            .ok_or("passenger animal rotation animal rotation must be an object")?;
        if !world.apply_entity(EntityInput {
            entity_id: animal_id,
            uuid: Uuid::new(0, u64::from(animal_id as u32)),
            type_id: 30,
            x: animal_position.x,
            y: animal_position.y,
            z: animal_position.z,
            movement: (animal_movement.x, animal_movement.y, animal_movement.z),
            rotation: (
                f32_field(animal_rotation, "y")?,
                f32_field(animal_rotation, "x")?,
                f32_field(animal_rotation, "y")?,
            ),
            data: 0,
        }) {
            return Err("passenger animal rotation animal was rejected".into());
        }
        if animal["no_gravity"]
            .as_bool()
            .ok_or("passenger animal rotation animal no_gravity must be a boolean")?
            && !world.apply_entity_metadata(animal_id, &[5, 8, 1, 255])
        {
            return Err("passenger animal rotation animal no-gravity was rejected".into());
        }

        let filler = row
            .get("filler")
            .and_then(Value::as_object)
            .ok_or("passenger animal rotation filler must be an object")?;
        let filler_id = i32_field(&Value::Object(filler.clone()), "entity_id")?;
        if i32_field(&Value::Object(filler.clone()), "type_id")? != 120 {
            return Err("passenger animal rotation fixture requires a Snowball filler".into());
        }
        let filler_position = f64_vec3(
            filler
                .get("position")
                .ok_or("passenger animal rotation filler position must be a vector")?,
        )?;
        let filler_movement = f64_vec3(
            filler
                .get("movement")
                .ok_or("passenger animal rotation filler movement must be a vector")?,
        )?;
        let filler_rotation = filler
            .get("rotation")
            .ok_or("passenger animal rotation filler rotation must be an object")?;
        if !world.apply_entity(EntityInput {
            entity_id: filler_id,
            uuid: Uuid::new(0, u64::from(filler_id as u32)),
            type_id: 120,
            x: filler_position.x,
            y: filler_position.y,
            z: filler_position.z,
            movement: (filler_movement.x, filler_movement.y, filler_movement.z),
            rotation: (
                f32_field(filler_rotation, "y")?,
                f32_field(filler_rotation, "x")?,
                f32_field(filler_rotation, "y")?,
            ),
            data: 0,
        }) {
            return Err("passenger animal rotation filler was rejected".into());
        }
        if filler["no_gravity"]
            .as_bool()
            .ok_or("passenger animal rotation filler no_gravity must be a boolean")?
            && !world.apply_entity_metadata(filler_id, &[5, 8, 1, 255])
        {
            return Err("passenger animal rotation filler no-gravity was rejected".into());
        }
        if !world.apply_entity_passengers(vehicle_id, &[animal_id, filler_id]) {
            return Err("passenger animal rotation relation was rejected".into());
        }
        entity_ids.extend([vehicle_id, animal_id, filler_id]);
    }

    let mut states = vec![client_world_entity_passenger_animal_rotation_state(
        "initial",
        &entity_ids,
        &world,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        if string(operation, "action")? != "tick" {
            return Err("unknown passenger animal rotation action".into());
        }
        let ticks = operation["ticks"]
            .as_u64()
            .ok_or("ticks must be an unsigned integer")?;
        for _ in 0..ticks {
            world.tick();
        }
        states.push(client_world_entity_passenger_animal_rotation_state(
            string(operation, "label")?,
            &entity_ids,
            &world,
        )?);
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_passenger_animal_position_result(request: &Value) -> Result<Value, String> {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ) {
        return Err("passenger animal position fixture chunk was rejected".into());
    }

    let vehicle = request["vehicle"]
        .as_object()
        .ok_or("vehicle must be an object")?;
    let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
    if i32_field(&Value::Object(vehicle.clone()), "type_id")? != 89 {
        return Err("passenger animal position fixture requires an oak boat".into());
    }
    let vehicle_position = f64_vec3(
        vehicle
            .get("position")
            .ok_or("passenger animal position vehicle position must be a vector")?,
    )?;
    let vehicle_rotation = vehicle
        .get("rotation")
        .ok_or("passenger animal position vehicle rotation must be an object")?;
    if !world.apply_entity(EntityInput {
        entity_id: vehicle_id,
        uuid: Uuid::new(0, u64::from(vehicle_id as u32)),
        type_id: 89,
        x: vehicle_position.x,
        y: vehicle_position.y,
        z: vehicle_position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (
            f32_field(vehicle_rotation, "y")?,
            f32_field(vehicle_rotation, "x")?,
            0.0,
        ),
        data: 0,
    }) {
        return Err("passenger animal position vehicle was rejected".into());
    }
    if vehicle["no_gravity"]
        .as_bool()
        .ok_or("passenger animal position vehicle no_gravity must be a boolean")?
        && !world.apply_entity_metadata(vehicle_id, &[5, 8, 1, 255])
    {
        return Err("passenger animal position vehicle no-gravity was rejected".into());
    }

    let animal = request["animal"]
        .as_object()
        .ok_or("animal must be an object")?;
    let animal_id = i32_field(&Value::Object(animal.clone()), "entity_id")?;
    if i32_field(&Value::Object(animal.clone()), "type_id")? != 30 {
        return Err("passenger animal position fixture requires a Cow".into());
    }
    let animal_position = f64_vec3(
        animal
            .get("position")
            .ok_or("passenger animal position animal position must be a vector")?,
    )?;
    let animal_movement = f64_vec3(
        animal
            .get("movement")
            .ok_or("passenger animal position animal movement must be a vector")?,
    )?;
    let animal_rotation = animal
        .get("rotation")
        .ok_or("passenger animal position animal rotation must be an object")?;
    if !world.apply_entity(EntityInput {
        entity_id: animal_id,
        uuid: Uuid::new(0, u64::from(animal_id as u32)),
        type_id: 30,
        x: animal_position.x,
        y: animal_position.y,
        z: animal_position.z,
        movement: (animal_movement.x, animal_movement.y, animal_movement.z),
        rotation: (
            f32_field(animal_rotation, "y")?,
            f32_field(animal_rotation, "x")?,
            f32_field(animal_rotation, "y")?,
        ),
        data: 0,
    }) {
        return Err("passenger animal position animal was rejected".into());
    }
    if animal["no_gravity"]
        .as_bool()
        .ok_or("passenger animal position animal no_gravity must be a boolean")?
        && !world.apply_entity_metadata(animal_id, &[5, 8, 1, 255])
    {
        return Err("passenger animal position animal no-gravity was rejected".into());
    }
    if !world.apply_entity_passengers(vehicle_id, &[animal_id]) {
        return Err("passenger animal position relation was rejected".into());
    }

    let entity_ids = [vehicle_id, animal_id];
    let mut states = vec![client_world_entity_tick_state(
        "initial",
        true,
        &entity_ids,
        &world,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        if string(operation, "action")? != "tick" {
            return Err("unknown passenger animal position action".into());
        }
        let ticks = operation["ticks"]
            .as_u64()
            .ok_or("ticks must be an unsigned integer")?;
        for _ in 0..ticks {
            world.tick();
        }
        states.push(client_world_entity_tick_state(
            string(operation, "label")?,
            true,
            &entity_ids,
            &world,
        )?);
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_passenger_turn_result(request: &Value) -> Result<Value, String> {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ) {
        return Err("passenger turn fixture chunk was rejected".into());
    }

    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut entity_ids = Vec::with_capacity(rows.len() * 2);
    let mut passenger_type_ids = Vec::with_capacity(rows.len());
    for row in rows {
        let vehicle = row
            .get("vehicle")
            .and_then(Value::as_object)
            .ok_or("passenger turn vehicle must be an object")?;
        let vehicle_id = i32_field(&Value::Object(vehicle.clone()), "entity_id")?;
        if i32_field(&Value::Object(vehicle.clone()), "type_id")? != 89 {
            return Err("passenger turn fixture requires an oak boat".into());
        }
        let vehicle_position = f64_vec3(
            vehicle
                .get("position")
                .ok_or("passenger turn vehicle position must be a vector")?,
        )?;
        let vehicle_rotation = vehicle
            .get("rotation")
            .ok_or("passenger turn vehicle rotation must be an object")?;
        if !world.apply_entity(EntityInput {
            entity_id: vehicle_id,
            uuid: Uuid::new(0, u64::from(vehicle_id as u32)),
            type_id: 89,
            x: vehicle_position.x,
            y: vehicle_position.y,
            z: vehicle_position.z,
            movement: (0.0, 0.0, 0.0),
            rotation: (
                f32_field(vehicle_rotation, "y")?,
                f32_field(vehicle_rotation, "x")?,
                0.0,
            ),
            data: 0,
        }) {
            return Err("passenger turn vehicle was rejected".into());
        }
        if vehicle["no_gravity"]
            .as_bool()
            .ok_or("passenger turn vehicle no_gravity must be a boolean")?
            && !world.apply_entity_metadata(vehicle_id, &[5, 8, 1, 255])
        {
            return Err("passenger turn vehicle no-gravity was rejected".into());
        }

        let passenger = row
            .get("passenger")
            .and_then(Value::as_object)
            .ok_or("passenger turn passenger must be an object")?;
        let passenger_id = i32_field(&Value::Object(passenger.clone()), "entity_id")?;
        let passenger_type_id = i32_field(&Value::Object(passenger.clone()), "type_id")?;
        if !matches!(passenger_type_id, 17 | 156) {
            return Err("passenger turn fixture requires a Player or Breeze passenger".into());
        }
        let passenger_position = f64_vec3(
            passenger
                .get("position")
                .ok_or("passenger turn passenger position must be a vector")?,
        )?;
        let passenger_movement = f64_vec3(
            passenger
                .get("movement")
                .ok_or("passenger turn passenger movement must be a vector")?,
        )?;
        let passenger_rotation = passenger
            .get("rotation")
            .ok_or("passenger turn passenger rotation must be an object")?;
        if !world.apply_entity(EntityInput {
            entity_id: passenger_id,
            uuid: Uuid::new(0, u64::from(passenger_id as u32)),
            type_id: passenger_type_id,
            x: passenger_position.x,
            y: passenger_position.y,
            z: passenger_position.z,
            movement: (
                passenger_movement.x,
                passenger_movement.y,
                passenger_movement.z,
            ),
            rotation: (
                f32_field(passenger_rotation, "y")?,
                f32_field(passenger_rotation, "x")?,
                f32_field(passenger_rotation, "y")?,
            ),
            data: 0,
        }) {
            return Err("passenger turn passenger was rejected".into());
        }
        if passenger["no_gravity"]
            .as_bool()
            .ok_or("passenger turn passenger no_gravity must be a boolean")?
            && !world.apply_entity_metadata(passenger_id, &[5, 8, 1, 255])
        {
            return Err("passenger turn passenger no-gravity was rejected".into());
        }
        if !world.apply_entity_passengers(vehicle_id, &[passenger_id]) {
            return Err("passenger turn relation was rejected".into());
        }
        entity_ids.extend([vehicle_id, passenger_id]);
        passenger_type_ids.push(passenger_type_id);
    }

    let mut states = vec![client_world_entity_passenger_rotation_state(
        "initial",
        &entity_ids,
        &passenger_type_ids,
        &world,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        if string(operation, "action")? != "turn" {
            return Err("unknown passenger turn action".into());
        }
        let mouse_x = operation["mouse_x"]
            .as_f64()
            .ok_or("mouse_x must be a number")?;
        let mouse_y = operation["mouse_y"]
            .as_f64()
            .ok_or("mouse_y must be a number")?;
        for index in 0..passenger_type_ids.len() {
            let passenger_id = entity_ids
                .get(index * 2 + 1)
                .copied()
                .ok_or("passenger turn entity list is incomplete")?;
            if !world.turn_entity(passenger_id, mouse_x, mouse_y) {
                return Err("passenger turn operation was rejected".into());
            }
        }
        states.push(client_world_entity_passenger_rotation_state(
            string(operation, "label")?,
            &entity_ids,
            &passenger_type_ids,
            &world,
        )?);
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_arrow_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_tick_result(request, 6)
}

fn client_world_entity_experience_orb_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_tick_result(request, 49)
}

fn client_world_entity_primed_tnt_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_tick_result(request, 133)
}

fn client_world_entity_falling_block_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_tick_result(request, 51)
}

fn client_world_entity_firework_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_tick_result(request, 53)
}

fn client_world_entity_boat_tick_state(
    label: &str,
    world: &ClientWorld,
    entity_id: i32,
) -> Result<Value, String> {
    let entity = world
        .entity_at(entity_id)
        .ok_or_else(|| "boat tick fixture entity disappeared".to_owned())?;
    let rotation = entity.rotation();
    let previous_rotation = entity.previous_rotation();
    Ok(json!({
        "label": label,
        "accepted": true,
        "entities": [{
            "entity_id": entity.id(),
            "tick_count": entity.tick_count(),
            "position": client_world_entity_tick_vector(entity.position()),
            "previous_position": client_world_entity_tick_vector(entity.previous_position()),
            "motion": client_world_entity_tick_vector(entity.motion()),
            "rotation_y": format!("{:08x}", rotation.0.to_bits()),
            "rotation_x": format!("{:08x}", rotation.1.to_bits()),
            "previous_rotation_y": format!("{:08x}", previous_rotation.0.to_bits()),
            "previous_rotation_x": format!("{:08x}", previous_rotation.1.to_bits()),
            "no_gravity": entity.is_no_gravity(),
            "delta_rotation": format!("{:08x}", entity.boat_delta_rotation().to_bits()),
            "paddle_left": entity.paddle_state().0,
            "paddle_right": entity.paddle_state().1,
        }],
    }))
}

fn client_world_entity_boat_air_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_boat_tick_result(request, true)
}

fn client_world_entity_boat_remote_tick_result(request: &Value) -> Result<Value, String> {
    client_world_entity_boat_tick_result(request, false)
}

fn client_world_entity_boat_tick_result(
    request: &Value,
    local_authority: bool,
) -> Result<Value, String> {
    let (passenger_id, passenger_type_id) = if local_authority {
        (i32_field(request, "player_id")?, 156)
    } else {
        let passenger = request
            .get("passenger")
            .ok_or("passenger must be an object")?;
        (
            i32_field(passenger, "entity_id")?,
            i32_field(passenger, "type_id")?,
        )
    };
    let boat = request.get("boat").ok_or("boat must be an object")?;
    let boat_entity_id = i32_field(boat, "entity_id")?;
    let position = f64_vec3(
        boat.get("position")
            .ok_or("boat position must be a vector")?,
    )?;
    let movement = f64_vec3(
        boat.get("movement")
            .ok_or("boat movement must be a vector")?,
    )?;
    let rotation = boat
        .get("rotation")
        .ok_or("boat rotation must be an object")?;
    let no_gravity = boat["no_gravity"]
        .as_bool()
        .ok_or("boat no_gravity must be a boolean")?;
    let input = request.get("input").ok_or("input must be an object")?;
    let boat_input = BoatInput::new(
        input["left"]
            .as_bool()
            .ok_or("input left must be a boolean")?,
        input["right"]
            .as_bool()
            .ok_or("input right must be a boolean")?,
        input["forward"]
            .as_bool()
            .ok_or("input forward must be a boolean")?,
        input["backward"]
            .as_bool()
            .ok_or("input backward must be a boolean")?,
    );

    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let blocks = vec![0; 4_096];
    let biomes = vec![0; 64];
    if !world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ) {
        return Err("boat air tick fixture chunk was rejected".into());
    }
    if !world.apply_entity(EntityInput {
        entity_id: boat_entity_id,
        uuid: Uuid::new(0, u64::from(boat_entity_id as u32)),
        type_id: 89,
        x: position.x,
        y: position.y,
        z: position.z,
        movement: (movement.x, movement.y, movement.z),
        rotation: (f32_field(rotation, "y")?, f32_field(rotation, "x")?, 0.0),
        data: 0,
    }) {
        return Err("boat air tick fixture entity was rejected".into());
    }
    if !world.apply_entity(EntityInput {
        entity_id: passenger_id,
        uuid: Uuid::new(0, u64::from(passenger_id as u32)),
        type_id: passenger_type_id,
        x: position.x,
        y: position.y,
        z: position.z,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("boat tick fixture passenger was rejected".into());
    }
    if local_authority {
        world.set_local_player_id(passenger_id);
    }
    if !world.apply_entity_passengers(boat_entity_id, &[passenger_id]) {
        return Err("boat tick fixture passenger relation was rejected".into());
    }
    if !world.set_boat_input(boat_entity_id, boat_input) {
        return Err("boat tick fixture input was rejected".into());
    }
    if no_gravity && !world.apply_entity_metadata(boat_entity_id, &[5, 8, 1, 255]) {
        return Err("boat tick fixture no-gravity metadata was rejected".into());
    }

    let mut states = vec![client_world_entity_boat_tick_state(
        "initial",
        &world,
        boat_entity_id,
    )?];
    for operation in request["operations"]
        .as_array()
        .ok_or("operations must be an array")?
    {
        match string(operation, "action")? {
            "tick" => {
                let ticks = operation["ticks"]
                    .as_u64()
                    .ok_or("ticks must be an unsigned integer")?;
                for _ in 0..ticks {
                    world.tick();
                }
            }
            "set_no_gravity" => {
                let value = operation["value"]
                    .as_bool()
                    .ok_or("no-gravity value must be a boolean")?;
                let metadata = if value {
                    &[5, 8, 1, 255]
                } else {
                    &[5, 8, 0, 255]
                };
                if !world.apply_entity_metadata(boat_entity_id, metadata) {
                    return Err("boat tick fixture no-gravity update was rejected".into());
                }
            }
            _ => return Err("unknown boat tick action".into()),
        }
        states.push(client_world_entity_boat_tick_state(
            string(operation, "label")?,
            &world,
            boat_entity_id,
        )?);
    }
    Ok(json!({"states": states}))
}

fn sprint_trigger_input(value: &Value) -> Result<Input, String> {
    Ok(Input::new(
        value["forward"]
            .as_bool()
            .ok_or("forward must be a boolean")?,
        value["backward"]
            .as_bool()
            .ok_or("backward must be a boolean")?,
        value["left"].as_bool().ok_or("left must be a boolean")?,
        value["right"].as_bool().ok_or("right must be a boolean")?,
        value["jump"].as_bool().ok_or("jump must be a boolean")?,
        value["shift"].as_bool().ok_or("shift must be a boolean")?,
        value["sprint"]
            .as_bool()
            .ok_or("sprint must be a boolean")?,
    ))
}

fn sprint_trigger_conditions(value: &Value) -> Result<LocalPlayerSprintTriggerConditions, String> {
    let boolean = |name: &str| {
        value[name]
            .as_bool()
            .ok_or_else(|| format!("{name} must be a boolean"))
    };
    Ok(LocalPlayerSprintTriggerConditions {
        mobility_restricted: boolean("mobility_restricted")?,
        enough_food: boolean("enough_food")?,
        may_fly: boolean("mayfly")?,
        shallow_water: boolean("shallow_water")?,
        flying: boolean("flying")?,
        slow_due_to_using_item: boolean("slow_due_to_using_item")?,
        fall_flying: boolean("fall_flying")?,
        underwater: boolean("underwater")?,
        moving_slowly: boolean("moving_slowly")?,
    })
}

fn client_world_player_sprint_trigger_state(
    label: &str,
    player: &voxel_world::ClientPlayerState,
) -> Value {
    json!({
        "label": label,
        "sprinting": player.is_sprinting(),
        "sprint_trigger_time": player.sprint_trigger_time(),
        "key_presses": input_snapshot(player.input_key_presses()),
        "move_vector": vec2_snapshot(player.input_move_vector()),
    })
}

fn client_world_player_sprint_trigger_result(request: &Value) -> Result<Value, String> {
    let player_id = i32_field(request, "player_id")?;
    let sprint_window = i32_field(request, "sprint_window")?;
    let conditions = sprint_trigger_conditions(
        request
            .get("conditions")
            .ok_or("conditions must be an object")?,
    )?;
    let sequences = request["sequences"]
        .as_array()
        .ok_or("sequences must be an array")?;
    let mut sequence_states = Vec::with_capacity(sequences.len());

    for sequence in sequences {
        let mut world = ClientWorld::new(DimensionBounds::new(0, 256));
        world.set_local_player_id(player_id);
        let operations = sequence["operations"]
            .as_array()
            .ok_or("operations must be an array")?;
        let mut states = Vec::with_capacity(operations.len());
        for operation in operations {
            let current_input = sprint_trigger_input(
                operation
                    .get("key_presses")
                    .ok_or("key_presses must be an object")?,
            )?;
            let move_vector_value = operation
                .get("move_vector")
                .ok_or("move_vector must be an object")?;
            let move_vector = Vec2::new(
                f32_field(move_vector_value, "x")?,
                f32_field(move_vector_value, "y")?,
            );
            let ticks = operation["ticks"]
                .as_u64()
                .ok_or("ticks must be an unsigned integer")?;
            for _ in 0..ticks {
                if !world.apply_local_player_sprint_trigger(
                    current_input,
                    move_vector,
                    sprint_window,
                    conditions,
                ) {
                    return Err("local player is not initialized".into());
                }
            }
            let player = world
                .local_player()
                .ok_or("local player is not initialized")?;
            states.push(client_world_player_sprint_trigger_state(
                string(operation, "label")?,
                player,
            ));
        }
        sequence_states.push(json!({
            "label": string(sequence, "label")?,
            "states": states,
        }));
    }

    Ok(json!({
        "player_id": player_id,
        "sequences": sequence_states,
    }))
}

fn client_world_entity_shared_flags_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "shared_flags": entity.shared_flags(),
        "on_fire": entity.is_on_fire(),
        "shift_key_down": entity.is_shift_key_down(),
        "sprinting": entity.is_sprinting(),
        "swimming": entity.is_swimming(),
        "invisible": entity.is_invisible(),
        "glowing": entity.is_glowing(),
        "fall_flying": entity.is_fall_flying(),
        "air_supply": entity.air_supply(),
    })
}

fn client_world_entity_shared_flags_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("shared flags fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("shared flags fixture entity disappeared")?;
        states.push(client_world_entity_shared_flags_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_air_supply_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "shared_flags": entity.shared_flags(),
        "air_supply": entity.air_supply(),
    })
}

fn client_world_entity_air_supply_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("air supply fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_air_supply_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("air supply fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("air supply fixture entity disappeared")?;
        states.push(client_world_entity_air_supply_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_pose_state(label: &str, accepted: bool, entity: &ClientEntity) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "shared_flags": entity.shared_flags(),
        "pose_id": entity.pose().id(),
    })
}

fn client_world_entity_pose_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("pose fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_pose_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("pose fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("pose fixture entity disappeared")?;
        states.push(client_world_entity_pose_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_ticks_frozen_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "shared_flags": entity.shared_flags(),
        "ticks_frozen": entity.ticks_frozen(),
        "percent_frozen": format!("{:08x}", entity.percent_frozen().to_bits()),
        "is_freezing": entity.is_freezing(),
        "is_fully_frozen": entity.is_fully_frozen(),
    })
}

fn client_world_entity_ticks_frozen_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("ticks frozen fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_ticks_frozen_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("ticks frozen fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("ticks frozen fixture entity disappeared")?;
        states.push(client_world_entity_ticks_frozen_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_custom_name_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "custom_name_present": entity.has_custom_name(),
        "custom_name_visible": entity.is_custom_name_visible(),
        "should_show_name": entity.should_show_name(),
    })
}

fn client_world_entity_custom_name_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("custom name fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_custom_name_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("custom name fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("custom name fixture entity disappeared")?;
        states.push(client_world_entity_custom_name_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_living_flags_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "living_flags": entity.living_flags(),
        "is_using_item": entity.is_using_item(),
        "used_item_is_off_hand": entity.used_item_is_off_hand(),
        "is_auto_spin_attack": entity.is_auto_spin_attack(),
    })
}

fn client_world_entity_living_flags_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("living flags fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_living_flags_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("living flags fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("living flags fixture entity disappeared")?;
        states.push(client_world_entity_living_flags_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_health_state(label: &str, accepted: bool, entity: &ClientEntity) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "living_flags": entity.living_flags(),
        "health_bits": entity
            .health()
            .map(|value| format!("{:08x}", value.to_bits())),
    })
}

fn client_world_entity_health_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("health fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = Vec::with_capacity(operations.len());
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("health fixture entity disappeared")?;
        states.push(client_world_entity_health_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_mob_flags_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "mob_flags": entity.mob_flags(),
        "is_no_ai": entity.is_no_ai(),
        "is_left_handed": entity.is_left_handed(),
        "is_aggressive": entity.is_aggressive(),
    })
}

fn client_world_entity_mob_flags_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("mob flags fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_mob_flags_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("mob flags fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("mob flags fixture entity disappeared")?;
        states.push(client_world_entity_mob_flags_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_arrow_count_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "arrow_count": entity.arrow_count(),
        "living_flags": entity.living_flags(),
    })
}

fn client_world_entity_arrow_count_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("arrow count fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_arrow_count_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("arrow count fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("arrow count fixture entity disappeared")?;
        states.push(client_world_entity_arrow_count_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_effect_ambience_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "living_flags": entity.living_flags(),
        "effect_ambience": entity.is_effect_ambient(),
    })
}

fn client_world_entity_effect_ambience_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("effect ambience fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_effect_ambience_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("effect ambience fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("effect ambience fixture entity disappeared")?;
        states.push(client_world_entity_effect_ambience_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_effect_particles_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "living_flags": entity.living_flags(),
        "effect_ambience": entity.is_effect_ambient(),
        "effect_particle_type_ids": entity.effect_particle_type_ids(),
    })
}

fn client_world_entity_effect_particles_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("effect particles fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_effect_particles_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("effect particles fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("effect particles fixture entity disappeared")?;
        states.push(client_world_entity_effect_particles_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_sleeping_position_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    let mut state = json!({
        "label": label,
        "accepted": accepted,
        "living_flags": entity.living_flags(),
        "is_sleeping": entity.is_sleeping(),
    });
    if let Some(position) = entity.sleeping_pos() {
        state
            .as_object_mut()
            .expect("sleeping position state object")
            .insert("sleeping_pos".into(), blockpos_snapshot(position));
    }
    state
}

fn client_world_entity_sleeping_position_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("sleeping position fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_sleeping_position_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("sleeping position fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("sleeping position fixture entity disappeared")?;
        states.push(client_world_entity_sleeping_position_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_world_entity_stinger_count_state(
    label: &str,
    accepted: bool,
    entity: &ClientEntity,
) -> Value {
    json!({
        "label": label,
        "accepted": accepted,
        "stinger_count": entity.stinger_count(),
        "living_flags": entity.living_flags(),
    })
}

fn client_world_entity_stinger_count_result(request: &Value) -> Result<Value, String> {
    let entity_id = i32_field(request, "entity_id")?;
    let mut world = ClientWorld::new(DimensionBounds::default());
    if !world.apply_entity(EntityInput {
        entity_id,
        uuid: Uuid::new(0, u64::from(entity_id as u32)),
        type_id: i32_field(request, "entity_type_id")?,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }) {
        return Err("stinger count fixture entity was rejected".into());
    }

    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut states = vec![client_world_entity_stinger_count_state(
        "initial",
        true,
        world
            .entity_at(entity_id)
            .ok_or("stinger count fixture entity disappeared")?,
    )];
    for operation in operations {
        let metadata = decode_hex(string(operation, "metadata_hex")?)?;
        let accepted = world.apply_entity_metadata(entity_id, &metadata);
        let entity = world
            .entity_at(entity_id)
            .ok_or("stinger count fixture entity disappeared")?;
        states.push(client_world_entity_stinger_count_state(
            string(operation, "label")?,
            accepted,
            entity,
        ));
    }
    Ok(json!({"states": states}))
}

fn client_chunk_cache_snapshot_json(snapshot: &ClientChunkCacheSnapshot) -> Value {
    json!({
        "storage_radius": snapshot.storage_radius,
        "view_range": snapshot.view_range,
        "view_center_x": snapshot.view_center_x,
        "view_center_z": snapshot.view_center_z,
        "loaded_chunks": snapshot.loaded_chunks,
        "added_empty_sections": snapshot.added_empty_sections,
        "removed_empty_sections": snapshot.removed_empty_sections,
        "added_loaded_chunks": snapshot.added_loaded_chunks,
        "removed_loaded_chunks": snapshot.removed_loaded_chunks,
    })
}

fn client_world_chunk_cache_result(request: &Value) -> Result<Value, String> {
    let mut world = ClientWorld::new(DimensionBounds::default());
    world.configure_chunk_cache(i32_field(request, "server_view_radius")?);
    let operations = request["operations"]
        .as_array()
        .ok_or("operations must be an array")?;
    let mut events = Vec::with_capacity(operations.len());

    for operation in operations {
        let action = string(operation, "action")?;
        let accepted = match action {
            "replace" => {
                let chunk_x = i32_field(operation, "chunk_x")?;
                let chunk_z = i32_field(operation, "chunk_z")?;
                let section_values = operation["sections"]
                    .as_array()
                    .ok_or("sections must be an array")?;
                let mut section_data = Vec::new();
                for section in section_values {
                    section_data.extend(decode_hex(
                        section.as_str().ok_or("section payload must be a string")?,
                    )?);
                }
                let observations = decode_chunk_section_data(&section_data)?;
                if observations.len() != section_values.len() {
                    return Err("section decoder count does not match input".into());
                }
                world.replace_chunk(
                    chunk_x,
                    chunk_z,
                    observations.iter().map(|section| SectionInput {
                        non_empty_block_count: section.non_empty_block_count,
                        fluid_count: section.fluid_count,
                        block_states: &section.block_state_values,
                        biomes: &section.biome_values,
                    }),
                )
            }
            "drop" => world.unload_chunk(
                i32_field(operation, "chunk_x")?,
                i32_field(operation, "chunk_z")?,
            ),
            "center" => {
                world.update_chunk_cache_center(
                    i32_field(operation, "chunk_x")?,
                    i32_field(operation, "chunk_z")?,
                );
                true
            }
            "radius" => {
                world.update_chunk_cache_radius(i32_field(operation, "server_view_radius")?);
                true
            }
            "section_emptiness" => {
                world.update_chunk_section_emptiness(
                    i32_field(operation, "section_x")?,
                    i32_field(operation, "section_y")?,
                    i32_field(operation, "section_z")?,
                    operation["empty"]
                        .as_bool()
                        .ok_or("empty must be a boolean")?,
                );
                true
            }
            "flip" => {
                world.flip_chunk_cache_update_sets();
                true
            }
            _ => return Err(format!("unknown client chunk cache action: {action}")),
        };

        let checks = operation["checks"]
            .as_array()
            .ok_or("checks must be an array")?
            .iter()
            .map(|check| {
                let chunk_x = i32_field(check, "x")?;
                let chunk_z = i32_field(check, "z")?;
                let (in_range, index) = world
                    .chunk_cache_location(chunk_x, chunk_z)
                    .ok_or("chunk cache is not configured")?;
                Ok(json!({
                    "x": chunk_x,
                    "z": chunk_z,
                    "in_range": in_range,
                    "index": index,
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let snapshot = world
            .client_chunk_cache_snapshot()
            .ok_or("chunk cache is not configured")?;
        events.push(json!({
            "action": action,
            "accepted": accepted,
            "cache": client_chunk_cache_snapshot_json(&snapshot),
            "checks": checks,
        }));
    }

    Ok(json!({"events": events}))
}

fn math_result(request: &Value) -> Result<Value, String> {
    let operation = string(request, "operation")?;
    if operation == "client_world_section_lookup" {
        return client_world_section_lookup_result(request);
    }
    if operation == "client_world_chunk_cache" {
        return client_world_chunk_cache_result(request);
    }
    if operation == "client_world_block_update" {
        return client_world_block_update_result(request);
    }
    if operation == "client_world_light" {
        return client_world_light_result(request);
    }
    if operation == "client_world_player_attributes" {
        return client_world_player_attributes_result(request);
    }
    if operation == "client_world_player_movement_speed" {
        return client_world_player_movement_speed_result(request);
    }
    if operation == "client_world_player_attribute_water" {
        return client_world_player_attribute_water_result(request);
    }
    if operation == "client_world_player_sprint_attribute" {
        return client_world_player_sprint_attribute_result(request);
    }
    if operation == "client_world_player_sneaking_input" {
        return client_world_player_sneaking_input_result(request);
    }
    if operation == "client_world_player_keyboard_input" {
        return client_world_player_keyboard_input_result(request);
    }
    if operation == "client_world_player_tick" {
        return client_world_player_tick_result(request);
    }
    if operation == "client_world_entity_player_head_rotation" {
        return client_world_entity_player_head_rotation_result(request);
    }
    if operation == "client_world_entity_snowball_tick" {
        return client_world_entity_snowball_tick_result(request);
    }
    if operation == "client_world_entity_arrow_tick" {
        return client_world_entity_arrow_tick_result(request);
    }
    if operation == "client_world_entity_experience_orb_tick" {
        return client_world_entity_experience_orb_tick_result(request);
    }
    if operation == "client_world_entity_primed_tnt_tick" {
        return client_world_entity_primed_tnt_tick_result(request);
    }
    if operation == "client_world_entity_falling_block_tick" {
        return client_world_entity_falling_block_tick_result(request);
    }
    if operation == "client_world_entity_firework_tick" {
        return client_world_entity_firework_tick_result(request);
    }
    if operation == "client_world_entity_boat_air_tick" {
        return client_world_entity_boat_air_tick_result(request);
    }
    if operation == "client_world_entity_boat_remote_tick" {
        return client_world_entity_boat_remote_tick_result(request);
    }
    if operation == "client_world_entity_ticking_eligibility" {
        return client_world_entity_ticking_eligibility_result(request);
    }
    if operation == "client_world_entity_transform_history" {
        return client_world_entity_transform_history_result(request);
    }
    if operation == "client_world_entity_player_interpolation" {
        return client_world_entity_player_interpolation_result(request);
    }
    if operation == "client_world_entity_player_snap" {
        return client_world_entity_player_snap_result(request);
    }
    if operation == "client_world_entity_player_motion" {
        return client_world_entity_player_motion_result(request);
    }
    if operation == "client_world_entity_teleport" {
        return client_world_entity_teleport_result(request);
    }
    if operation == "client_world_entity_passengers" {
        return client_world_entity_passengers_result(request);
    }
    if operation == "client_world_entity_passenger_tick" {
        return client_world_entity_passenger_tick_result(request);
    }
    if operation == "client_world_entity_passenger_position" {
        return client_world_entity_passenger_position_result(request);
    }
    if operation == "client_world_entity_passenger_player_position" {
        return client_world_entity_passenger_player_position_result(request);
    }
    if operation == "client_world_entity_passenger_boat_family_position" {
        return client_world_entity_passenger_boat_family_position_result(request);
    }
    if operation == "client_world_entity_passenger_rotation" {
        return client_world_entity_passenger_rotation_result(request);
    }
    if operation == "client_world_entity_passenger_boat_turn_exemption" {
        return client_world_entity_passenger_boat_turn_exemption_result(request);
    }
    if operation == "client_world_entity_passenger_animal_rotation" {
        return client_world_entity_passenger_animal_rotation_result(request);
    }
    if operation == "client_world_entity_passenger_animal_position" {
        return client_world_entity_passenger_animal_position_result(request);
    }
    if operation == "client_world_entity_passenger_turn" {
        return client_world_entity_passenger_turn_result(request);
    }
    if operation == "client_world_player_sprint_trigger" {
        return client_world_player_sprint_trigger_result(request);
    }
    if operation == "client_world_entity_shared_flags" {
        return client_world_entity_shared_flags_result(request);
    }
    if operation == "client_world_entity_air_supply" {
        return client_world_entity_air_supply_result(request);
    }
    if operation == "client_world_entity_pose" {
        return client_world_entity_pose_result(request);
    }
    if operation == "client_world_entity_ticks_frozen" {
        return client_world_entity_ticks_frozen_result(request);
    }
    if operation == "client_world_entity_custom_name" {
        return client_world_entity_custom_name_result(request);
    }
    if operation == "client_world_entity_living_flags" {
        return client_world_entity_living_flags_result(request);
    }
    if operation == "client_world_entity_health" {
        return client_world_entity_health_result(request);
    }
    if operation == "client_world_entity_mob_flags" {
        return client_world_entity_mob_flags_result(request);
    }
    if operation == "client_world_entity_arrow_count" {
        return client_world_entity_arrow_count_result(request);
    }
    if operation == "client_world_entity_effect_ambience" {
        return client_world_entity_effect_ambience_result(request);
    }
    if operation == "client_world_entity_effect_particles" {
        return client_world_entity_effect_particles_result(request);
    }
    if operation == "client_world_entity_sleeping_position" {
        return client_world_entity_sleeping_position_result(request);
    }
    if operation == "client_world_entity_stinger_count" {
        return client_world_entity_stinger_count_result(request);
    }
    if operation == "random" {
        return random_result(request);
    }
    if operation == "random_fork" {
        return random_fork_result(request);
    }
    if operation == "random_positional" {
        return random_positional_result(request);
    }
    if operation == "aabb_builder" {
        return aabb_builder_result(request);
    }
    if operation == "aabb" {
        return aabb_result(request);
    }
    if operation == "shape" {
        return shape_result(request);
    }
    if operation == "blockpos" {
        return blockpos_result(request);
    }
    if operation == "blockpos_packed_helpers" {
        let value = blockpos(request)?;
        let direction =
            Direction::from_name(string(request, "direction")?).ok_or("unknown direction name")?;
        let coordinate = |name: &str| {
            string(request, name)?
                .parse::<i32>()
                .map_err(|error| error.to_string())
        };
        return Ok(blockpos_packed_helpers_snapshot(
            value,
            u64_hex_field(request, "neighbor_node")?,
            direction,
            coordinate("step_x")?,
            coordinate("step_y")?,
            coordinate("step_z")?,
        ));
    }
    if operation == "blockpos_relative_helpers" {
        let value = blockpos(request)?;
        let direction =
            Direction::from_name(string(request, "direction")?).ok_or("unknown direction name")?;
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis name")?;
        let steps = string(request, "steps")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        return Ok(blockpos_relative_helpers_snapshot(
            value, direction, axis, steps,
        ));
    }
    if operation == "blockpos_vector_helpers" {
        let value = blockpos(request)?;
        let delta = BlockPos::new(
            i32_string_field(request, "dx")?,
            i32_string_field(request, "dy")?,
            i32_string_field(request, "dz")?,
        );
        let scale = i32_string_field(request, "scale")?;
        return Ok(blockpos_vector_helpers_snapshot(value, delta, scale));
    }
    if operation == "blockpos_geometry_helpers" {
        let value = blockpos(request)?;
        let rotation = BlockRotation::from_name(string(request, "rotation")?)
            .ok_or("unknown BlockPos rotation")?;
        let up_vector = BlockPos::new(
            i32_string_field(request, "up_x")?,
            i32_string_field(request, "up_y")?,
            i32_string_field(request, "up_z")?,
        );
        let y = i32_string_field(request, "at_y")?;
        return Ok(blockpos_geometry_helpers_snapshot(
            value, rotation, up_vector, y,
        ));
    }
    if operation == "blockpos_distance_helpers" {
        let value = blockpos(request)?;
        let other = blockpos(request.get("other").ok_or("other must be an object")?)?;
        let point = f64_vec3(request.get("point").ok_or("point must be an object")?)?;
        let distance = f64_field(request, "distance")?;
        return Ok(blockpos_distance_helpers_snapshot(
            value, other, point, distance,
        ));
    }
    if operation == "blockpos_integer_helpers" {
        let value = blockpos(request)?;
        let other = blockpos(request.get("other").ok_or("other must be an object")?)?;
        return Ok(blockpos_integer_helpers_snapshot(value, other));
    }
    if operation == "blockpos_clamp_location" {
        let value = blockpos(request)?;
        let location = f64_vec3(
            request
                .get("location")
                .ok_or("location must be an object")?,
        )?;
        return Ok(blockpos_clamp_location_snapshot(value, location));
    }
    if operation == "blockpos_between_closed" {
        let from = blockpos(request.get("from").ok_or("from must be an object")?)?;
        let to = blockpos(request.get("to").ok_or("to must be an object")?)?;
        return Ok(json!({
            "from_to": blockpos_position_list(BlockPos::between_closed(from, to)),
            "coordinates": blockpos_position_list(BlockPos::between_closed_coordinates(
                i32_string_field(request, "min_x")?,
                i32_string_field(request, "min_y")?,
                i32_string_field(request, "min_z")?,
                i32_string_field(request, "max_x")?,
                i32_string_field(request, "max_y")?,
                i32_string_field(request, "max_z")?,
            )),
        }));
    }
    if operation == "blockpos_between_closed_aabb" {
        let aabb = aabb_box(request.get("aabb").ok_or("aabb must be an object")?)?;
        return Ok(blockpos_positions_snapshot(BlockPos::between_closed_aabb(
            aabb,
        )));
    }
    if operation == "blockpos_between_closed_streams" {
        let from = blockpos(request.get("from").ok_or("from must be an object")?)?;
        let to = blockpos(request.get("to").ok_or("to must be an object")?)?;
        let bounds = bounding_box(
            request
                .get("bounding_box")
                .ok_or("bounding_box must be an object")?,
        )?;
        let aabb = aabb_box(request.get("aabb").ok_or("aabb must be an object")?)?;
        let min_x = i32_string_field(request, "min_x")?;
        let min_y = i32_string_field(request, "min_y")?;
        let min_z = i32_string_field(request, "min_z")?;
        let max_x = i32_string_field(request, "max_x")?;
        let max_y = i32_string_field(request, "max_y")?;
        let max_z = i32_string_field(request, "max_z")?;
        return Ok(json!({
            "block": blockpos_position_list(BlockPos::between_closed_stream(from, to)),
            "bounding_box": blockpos_position_list(
                BlockPos::between_closed_stream_bounding_box(bounds),
            ),
            "aabb": blockpos_position_list(BlockPos::between_closed_stream_aabb(aabb)),
            "coordinates": blockpos_position_list(
                BlockPos::between_closed_stream_coordinates(
                    min_x, min_y, min_z, max_x, max_y, max_z,
                ),
            ),
        }));
    }
    if operation == "blockpos_coordinate_helpers" {
        let containing = BlockPos::containing(
            f64_field(request, "x")?,
            f64_field(request, "y")?,
            f64_field(request, "z")?,
        );
        let point = f64_vec3(request.get("point").ok_or("point must be an object")?);
        let point = BlockPos::containing_vec3(point?);
        let first = blockpos(request.get("first").ok_or("first must be an object")?)?;
        let second = blockpos(request.get("second").ok_or("second must be an object")?)?;
        return Ok(json!({
            "containing": blockpos_snapshot(containing),
            "containing_position": blockpos_snapshot(point),
            "min": blockpos_snapshot(BlockPos::min(first, second)),
            "max": blockpos_snapshot(BlockPos::max(first, second)),
        }));
    }
    if operation == "blockpos_between_corners_direction" {
        let aabb = aabb_box(request.get("aabb").ok_or("aabb must be an object")?)?;
        let first = blockpos(request.get("first").ok_or("first must be an object")?)?;
        let second = blockpos(request.get("second").ok_or("second must be an object")?)?;
        let direction = f64_vec3(
            request
                .get("direction")
                .ok_or("direction must be an object")?,
        )?;
        return Ok(json!({
            "aabb": blockpos_position_list(
                BlockPos::between_corners_in_direction_aabb(aabb, direction),
            ),
            "block_pos": blockpos_position_list(
                BlockPos::between_corners_in_direction(first, second, direction),
            ),
            "coordinates": blockpos_position_list(
                BlockPos::between_corners_in_direction_coordinates(
                    i32_string_field(request, "first_x")?,
                    i32_string_field(request, "first_y")?,
                    i32_string_field(request, "first_z")?,
                    i32_string_field(request, "second_x")?,
                    i32_string_field(request, "second_y")?,
                    i32_string_field(request, "second_z")?,
                    direction,
                ),
            ),
        }));
    }
    if operation == "blockpos_breadth_first_traversal" {
        let start = blockpos(request.get("start").ok_or("start must be an object")?)?;
        let max_depth = i32_string_field(request, "max_depth")?;
        let max_count = i32_string_field(request, "max_count")?;
        let rules = request
            .get("nodes")
            .ok_or("nodes must be an array")?
            .as_array()
            .ok_or("nodes must be an array")?
            .iter()
            .map(|entry| {
                let position =
                    blockpos(entry.get("position").ok_or("position must be an object")?)?;
                let status = match string(entry, "status")? {
                    "accept" => binary::TraversalNodeStatus::Accept,
                    "skip" => binary::TraversalNodeStatus::Skip,
                    "stop" => binary::TraversalNodeStatus::Stop,
                    _ => return Err("unknown traversal status".into()),
                };
                let neighbours = entry
                    .get("neighbors")
                    .ok_or("neighbors must be an array")?
                    .as_array()
                    .ok_or("neighbors must be an array")?
                    .iter()
                    .map(blockpos)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((position, status, neighbours))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut processed = Vec::new();
        let mut provider_calls = Vec::new();
        let count = BlockPos::breadth_first_traversal(
            start,
            max_depth,
            max_count,
            |position, enqueue| {
                provider_calls.push(position);
                if let Some((_, _, neighbours)) =
                    rules.iter().find(|(node, _, _)| *node == position)
                {
                    for neighbour in neighbours {
                        enqueue(*neighbour);
                    }
                }
            },
            |position| {
                processed.push(position);
                rules
                    .iter()
                    .find(|(node, _, _)| *node == position)
                    .map_or(binary::TraversalNodeStatus::Accept, |(_, status, _)| {
                        *status
                    })
            },
        );
        return Ok(json!({
            "count": count,
            "processed": blockpos_position_list(processed),
            "provider_calls": blockpos_position_list(provider_calls),
        }));
    }
    if operation == "blockpos_mutable_helpers" {
        let initial = blockpos(request.get("initial").ok_or("initial must be an object")?)?;
        let mut mutable = initial.mutable();
        let mut constructors = serde_json::Map::new();
        constructors.insert("immutable".into(), blockpos_snapshot(initial.immutable()));
        constructors.insert(
            "mutable".into(),
            blockpos_snapshot(initial.mutable().immutable()),
        );
        constructors.insert(
            "double".into(),
            blockpos_snapshot(
                MutableBlockPos::from_f64(
                    f64_field(request, "double_x")?,
                    f64_field(request, "double_y")?,
                    f64_field(request, "double_z")?,
                )
                .immutable(),
            ),
        );
        let mut events = Vec::new();
        for step in request["steps"]
            .as_array()
            .ok_or("steps must be an array")?
        {
            let mut event = serde_json::Map::new();
            match string(step, "op")? {
                "set" => {
                    mutable.set(
                        i32_string_field(step, "x")?,
                        i32_string_field(step, "y")?,
                        i32_string_field(step, "z")?,
                    );
                }
                "set_double" => {
                    mutable.set_f64(
                        f64_field(step, "x")?,
                        f64_field(step, "y")?,
                        f64_field(step, "z")?,
                    );
                }
                "set_block_pos" => {
                    mutable.set_block_pos(blockpos(
                        step.get("value").ok_or("value must be an object")?,
                    )?);
                }
                "set_packed" => {
                    mutable.set_packed(u64_hex_field(step, "value")?);
                }
                "set_with_offset_direction" => {
                    let position =
                        blockpos(step.get("position").ok_or("position must be an object")?)?;
                    let direction = Direction::from_name(string(step, "direction")?)
                        .ok_or("unknown direction")?;
                    mutable.set_with_offset_direction(position, direction);
                }
                "set_with_offset_components" => {
                    mutable.set_with_offset_components(
                        blockpos(step.get("position").ok_or("position must be an object")?)?,
                        i32_string_field(step, "x")?,
                        i32_string_field(step, "y")?,
                        i32_string_field(step, "z")?,
                    );
                }
                "set_with_offset_block_pos" => {
                    mutable.set_with_offset_block_pos(
                        blockpos(step.get("position").ok_or("position must be an object")?)?,
                        blockpos(step.get("offset").ok_or("offset must be an object")?)?,
                    );
                }
                "move_direction" => {
                    let direction = Direction::from_name(string(step, "direction")?)
                        .ok_or("unknown direction")?;
                    mutable.move_direction(direction, 1);
                }
                "move_direction_steps" => {
                    let direction = Direction::from_name(string(step, "direction")?)
                        .ok_or("unknown direction")?;
                    mutable.move_direction(direction, i32_string_field(step, "steps")?);
                }
                "move_components" => {
                    mutable.move_components(
                        i32_string_field(step, "x")?,
                        i32_string_field(step, "y")?,
                        i32_string_field(step, "z")?,
                    );
                }
                "move_block_pos" => {
                    mutable.move_block_pos(blockpos(
                        step.get("value").ok_or("value must be an object")?,
                    )?);
                }
                "clamp" => {
                    let axis = Axis::from_name(string(step, "axis")?).ok_or("unknown axis")?;
                    mutable.clamp(
                        axis,
                        i32_string_field(step, "minimum")?,
                        i32_string_field(step, "maximum")?,
                    );
                }
                "set_x" => {
                    mutable.set_x(i32_string_field(step, "value")?);
                }
                "set_y" => {
                    mutable.set_y(i32_string_field(step, "value")?);
                }
                "set_z" => {
                    mutable.set_z(i32_string_field(step, "value")?);
                }
                "offset_return" => {
                    event.insert(
                        "returned".into(),
                        blockpos_snapshot(mutable.offset(
                            i32_string_field(step, "x")?,
                            i32_string_field(step, "y")?,
                            i32_string_field(step, "z")?,
                        )),
                    );
                }
                "multiply_return" => {
                    event.insert(
                        "returned".into(),
                        blockpos_snapshot(mutable.multiply(i32_string_field(step, "scale")?)),
                    );
                }
                "relative_return" => {
                    let direction = Direction::from_name(string(step, "direction")?)
                        .ok_or("unknown direction")?;
                    event.insert(
                        "returned".into(),
                        blockpos_snapshot(
                            mutable.relative_direction(direction, i32_string_field(step, "steps")?),
                        ),
                    );
                }
                "relative_axis_return" => {
                    let axis = Axis::from_name(string(step, "axis")?).ok_or("unknown axis")?;
                    event.insert(
                        "returned".into(),
                        blockpos_snapshot(
                            mutable.relative_axis(axis, i32_string_field(step, "steps")?),
                        ),
                    );
                }
                "rotate_return" => {
                    let rotation = BlockRotation::from_name(string(step, "rotation")?)
                        .ok_or("unknown rotation")?;
                    event.insert(
                        "returned".into(),
                        blockpos_snapshot(mutable.rotate(rotation)),
                    );
                }
                _ => return Err("unknown mutable BlockPos operation".into()),
            }
            event.insert("state".into(), blockpos_snapshot(mutable.immutable()));
            events.push(Value::Object(event));
        }
        return Ok(json!({
            "constructors": Value::Object(constructors),
            "events": events,
        }));
    }
    if operation == "axis_cycle_helpers" {
        let x = i32_string_field(request, "x")?;
        let y = i32_string_field(request, "y")?;
        let z = i32_string_field(request, "z")?;
        let double_x = f64_field(request, "double_x")?;
        let double_y = f64_field(request, "double_y")?;
        let double_z = f64_field(request, "double_z")?;
        let mut cycles = serde_json::Map::new();
        for cycle in AxisCycle::VALUES {
            cycles.insert(
                cycle.name().into(),
                json!({
                    "int_x": cycle.cycle_i32(x, y, z, Axis::X),
                    "int_y": cycle.cycle_i32(x, y, z, Axis::Y),
                    "int_z": cycle.cycle_i32(x, y, z, Axis::Z),
                    "double_x": format!("{:016x}", cycle.cycle_f64(double_x, double_y, double_z, Axis::X).to_bits()),
                    "double_y": format!("{:016x}", cycle.cycle_f64(double_x, double_y, double_z, Axis::Y).to_bits()),
                    "double_z": format!("{:016x}", cycle.cycle_f64(double_x, double_y, double_z, Axis::Z).to_bits()),
                    "axis_x": cycle.cycle_axis(Axis::X).name(),
                    "axis_y": cycle.cycle_axis(Axis::Y).name(),
                    "axis_z": cycle.cycle_axis(Axis::Z).name(),
                    "inverse": cycle.inverse().name(),
                }),
            );
        }
        let mut between = Vec::new();
        for from in [Axis::X, Axis::Y, Axis::Z] {
            for to in [Axis::X, Axis::Y, Axis::Z] {
                between.push(json!({
                    "from": from.name(),
                    "to": to.name(),
                    "cycle": AxisCycle::between(from, to).name(),
                }));
            }
        }
        let mut mutable_set = serde_json::Map::new();
        for cycle in AxisCycle::VALUES {
            let mut value = MutableBlockPos::new(0, 0, 0);
            value.set_axis_cycle(cycle, x, y, z);
            mutable_set.insert(cycle.name().into(), blockpos_snapshot(value.immutable()));
        }
        return Ok(json!({
            "cycles": Value::Object(cycles),
            "between": between,
            "mutable_set": Value::Object(mutable_set),
        }));
    }
    if operation == "blockpos_within_manhattan" {
        let origin = blockpos(request.get("origin").ok_or("origin must be an object")?)?;
        let reach_x = i32_string_field(request, "reach_x")?;
        let reach_y = i32_string_field(request, "reach_y")?;
        let reach_z = i32_string_field(request, "reach_z")?;
        return Ok(json!({
            "positions": blockpos_position_list(BlockPos::within_manhattan(
                origin, reach_x, reach_y, reach_z,
            )),
            "stream_positions": blockpos_position_list(BlockPos::within_manhattan(
                origin, reach_x, reach_y, reach_z,
            )),
        }));
    }
    if operation == "blockpos_spiral_around" {
        let center = blockpos(request.get("center").ok_or("center must be an object")?)?;
        let radius = i32_string_field(request, "radius")?;
        let first_direction = Direction::from_name(string(request, "first_direction")?)
            .ok_or("unknown first spiral direction")?;
        let second_direction = Direction::from_name(string(request, "second_direction")?)
            .ok_or("unknown second spiral direction")?;
        let positions = BlockPos::spiral_around(center, radius, first_direction, second_direction)
            .map_err(|_| "spiral directions must use different axes".to_owned())?;
        return Ok(blockpos_positions_snapshot(positions));
    }
    if operation == "blockpos_find_closest_match" {
        let start = blockpos(request.get("start").ok_or("start must be an object")?)?;
        let horizontal_radius = i32_string_field(request, "horizontal_radius")?;
        let vertical_radius = i32_string_field(request, "vertical_radius")?;
        let target = blockpos(request.get("target").ok_or("target must be an object")?)?;
        let value =
            BlockPos::find_closest_match(start, horizontal_radius, vertical_radius, |candidate| {
                candidate == target
            });
        let mut result = serde_json::Map::new();
        if let Some(value) = value {
            result.insert("value".into(), blockpos_snapshot(value));
        }
        return Ok(Value::Object(result));
    }
    if operation == "blockpos_neighbor_column" {
        return Ok(blockpos_positions_snapshot(BlockPos::neighbor_column(
            i32_string_field(request, "start_x")?,
            i32_string_field(request, "start_y")?,
            i32_string_field(request, "start_z")?,
            i32_string_field(request, "end_y")?,
        )));
    }
    if operation == "blockpos_square_out_south_east" {
        return Ok(blockpos_positions_snapshot(
            BlockPos::new(
                i32_string_field(request, "x")?,
                i32_string_field(request, "y")?,
                i32_string_field(request, "z")?,
            )
            .square_out_south_east(),
        ));
    }
    if operation == "blockpos_random_ranges" {
        let seed = random_seed(request)?;
        let limit = i32_string_field(request, "limit")?;
        let min_x = i32_string_field(request, "min_x")?;
        let min_y = i32_string_field(request, "min_y")?;
        let min_z = i32_string_field(request, "min_z")?;
        let max_x = i32_string_field(request, "max_x")?;
        let max_y = i32_string_field(request, "max_y")?;
        let max_z = i32_string_field(request, "max_z")?;
        let mut between_random = RandomSource::create(seed);
        let between = BlockPos::random_between_closed(
            &mut between_random,
            limit,
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        )
        .map_err(|_| "bound must be positive".to_owned())?;
        let after_between = between_random.next_int();
        let center = blockpos(request.get("center").ok_or("center must be an object")?)?;
        let size = i32_string_field(request, "size")?;
        let mut cube_random = RandomSource::create(seed);
        let cube = BlockPos::random_in_cube(&mut cube_random, limit, center, size)
            .map_err(|_| "bound must be positive".to_owned())?;
        let after_cube = cube_random.next_int();
        return Ok(json!({
            "between": blockpos_position_list(between),
            "after_between": after_between.to_string(),
            "cube": blockpos_position_list(cube),
            "after_cube": after_cube.to_string(),
        }));
    }
    if operation == "section_relative" {
        return section_relative_result(request);
    }
    if operation == "vec3_codec" {
        return match string(request, "mode")? {
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
        };
    }
    if operation == "chunkpos_codec" {
        return match string(request, "mode")? {
            "decode" => {
                let values = request["values"]
                    .as_array()
                    .ok_or("values must be an array")?
                    .iter()
                    .enumerate()
                    .map(|(index, value)| {
                        value
                            .as_str()
                            .ok_or_else(|| format!("values[{index}] must be a string"))?
                            .parse::<i32>()
                            .map_err(|error| error.to_string())
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                match ChunkPos::codec_decode(&values) {
                    Ok(value) => Ok(json!({
                        "outcome": "value",
                        "value": chunkpos_snapshot(value),
                    })),
                    Err(binary::ChunkPosCodecError::WrongLength { partial, .. }) => {
                        let mut result = json!({
                            "outcome": "error",
                            "error": "Input is not a list of 2 ints",
                        });
                        if let Some(value) = partial {
                            result["partial"] = chunkpos_snapshot(value);
                        }
                        Ok(result)
                    }
                }
            }
            "encode" => {
                let value = chunkpos(request)?;
                Ok(json!({
                    "outcome": "value",
                    "value": value.codec_encode(),
                }))
            }
            _ => Err("unknown ChunkPos codec mode".into()),
        };
    }
    if operation == "chunkpos_block_coordinates" {
        let value = chunkpos(request)?;
        let offset = string(request, "offset")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        return Ok(json!({
            "value": chunkpos_block_coordinates_snapshot(value, offset),
        }));
    }
    if operation == "chunkpos_region_coordinates" {
        let value = chunkpos(request)?;
        let region_x = string(request, "region_x")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let region_z = string(request, "region_z")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        return Ok(json!({
            "value": chunkpos_region_coordinates_snapshot(value, region_x, region_z),
        }));
    }
    if operation == "chunkpos_block_positions" {
        let value = chunkpos(request)?;
        let block_x = string(request, "block_x")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let y = string(request, "y")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let block_z = string(request, "block_z")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let contains = request["contains"]
            .as_array()
            .ok_or("contains must be an array")?;
        return Ok(json!({
            "value": chunkpos_block_positions_snapshot(value, block_x, y, block_z, contains)?,
        }));
    }
    if operation == "chunkpos_representation" {
        let value = chunkpos(request)?;
        let hash_x = string(request, "hash_x")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let hash_z = string(request, "hash_z")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        return Ok(json!({
            "value": chunkpos_representation_snapshot(value, hash_x, hash_z),
        }));
    }
    if operation == "chunkpos_from_blockpos" {
        return Ok(json!({
            "value": chunkpos_from_blockpos_snapshot(blockpos(request)?),
        }));
    }
    if operation == "chunkpos_distances" {
        let value = chunkpos(request)?;
        let other = ChunkPos::new(
            string(request, "other_x")?
                .parse::<i32>()
                .map_err(|error| error.to_string())?,
            string(request, "other_z")?
                .parse::<i32>()
                .map_err(|error| error.to_string())?,
        );
        return Ok(json!({
            "value": chunkpos_distances_snapshot(value, other),
        }));
    }
    if operation == "chunkpos_from_section_node" {
        let section_node = u64::from_str_radix(string(request, "section_node")?, 16)
            .map_err(|error| error.to_string())?;
        return Ok(json!({
            "value": format!("{:x}", ChunkPos::from_section_node(section_node)),
        }));
    }
    if operation == "chunkpos_range_closed" {
        let positions = match string(request, "mode")? {
            "center-radius" => {
                let center = chunkpos(request)?;
                let radius = string(request, "radius")?
                    .parse::<i32>()
                    .map_err(|error| error.to_string())?;
                ChunkPos::range_closed(center, radius)
            }
            "from-to" => {
                let from = ChunkPos::new(
                    string(request, "from_x")?
                        .parse::<i32>()
                        .map_err(|error| error.to_string())?,
                    string(request, "from_z")?
                        .parse::<i32>()
                        .map_err(|error| error.to_string())?,
                );
                let to = ChunkPos::new(
                    string(request, "to_x")?
                        .parse::<i32>()
                        .map_err(|error| error.to_string())?,
                    string(request, "to_z")?
                        .parse::<i32>()
                        .map_err(|error| error.to_string())?,
                );
                ChunkPos::range_closed_from_to(from, to)
            }
            _ => return Err("unknown ChunkPos range mode".into()),
        };
        return Ok(json!({
            "value": chunkpos_range_closed_snapshot(positions),
        }));
    }
    if operation == "chunkpos_constant" {
        let value = match string(request, "name")? {
            "zero" => json!({
                "chunk_pos": chunkpos_snapshot(ChunkPos::ZERO),
            }),
            "invalid_chunk_pos" => json!({
                "packed": format!("{:x}", ChunkPos::INVALID_CHUNK_POS),
            }),
            "region_size" => json!({
                "value": ChunkPos::REGION_SIZE.to_string(),
            }),
            "region_max_index" => json!({
                "value": ChunkPos::REGION_MAX_INDEX.to_string(),
            }),
            _ => return Err("unknown ChunkPos constant".into()),
        };
        return Ok(json!({"value": value}));
    }
    if operation == "chunkpos_packed_accessors" {
        let packed = u64_hex_field(request, "packed")?;
        return Ok(json!({
            "value": {
                "x": ChunkPos::get_x(packed).to_string(),
                "z": ChunkPos::get_z(packed).to_string(),
            },
        }));
    }
    if operation == "sectionpos_packed_accessors" {
        let packed = u64_hex_field(request, "packed")?;
        return Ok(json!({
            "value": {
                "x": SectionPos::get_x(packed).to_string(),
                "y": SectionPos::get_y(packed).to_string(),
                "z": SectionPos::get_z(packed).to_string(),
            },
        }));
    }
    if operation == "sectionpos_block_coordinates" {
        return Ok(json!({
            "value": sectionpos_block_coordinates_snapshot(sectionpos(request)?),
        }));
    }
    if operation == "sectionpos_relative_to_block" {
        let value = sectionpos(request)?;
        let relative = string(request, "relative")?
            .parse::<i16>()
            .map_err(|error| error.to_string())? as u16;
        return Ok(json!({
            "value": {
                "relative_x": SectionPos::section_relative_x(relative).to_string(),
                "relative_y": SectionPos::section_relative_y(relative).to_string(),
                "relative_z": SectionPos::section_relative_z(relative).to_string(),
                "block_x": value.relative_to_block_x(relative).to_string(),
                "block_y": value.relative_to_block_y(relative).to_string(),
                "block_z": value.relative_to_block_z(relative).to_string(),
                "block_position": blockpos_snapshot(value.relative_to_block_pos(relative)),
            },
        }));
    }
    if operation == "sectionpos_spatial" {
        return Ok(json!({
            "value": sectionpos_spatial_snapshot(sectionpos(request)?),
        }));
    }
    if operation == "sectionpos_packed_transformations" {
        let coordinate = |name: &str| {
            string(request, name)?
                .parse::<i32>()
                .map_err(|error| error.to_string())
        };
        return Ok(json!({
            "value": sectionpos_packed_transformations_snapshot(
                u64_hex_field(request, "block_node")?,
                u64_hex_field(request, "section_node")?,
                coordinate("x")?,
                coordinate("z")?,
            ),
        }));
    }
    if operation == "sectionpos_as_long" {
        let coordinate = |name: &str| {
            string(request, name)?
                .parse::<i32>()
                .map_err(|error| error.to_string())
        };
        return Ok(json!({
            "value": sectionpos_as_long_snapshot(
                u64_hex_field(request, "block_node")?,
                coordinate("x")?,
                coordinate("y")?,
                coordinate("z")?,
                coordinate("section_x")?,
                coordinate("section_y")?,
                coordinate("section_z")?,
            ),
        }));
    }
    if operation == "sectionpos_offset" {
        let coordinate = |name: &str| {
            string(request, name)?
                .parse::<i32>()
                .map_err(|error| error.to_string())
        };
        let direction = Direction::from_name(string(request, "direction")?)
            .ok_or("unknown SectionPos offset direction")?;
        return Ok(json!({
            "value": sectionpos_offset_snapshot(
                u64_hex_field(request, "section_node")?,
                direction,
                (
                    coordinate("step_x")?,
                    coordinate("step_y")?,
                    coordinate("step_z")?,
                ),
                SectionPos::new(coordinate("x")?, coordinate("y")?, coordinate("z")?),
            ),
        }));
    }
    if operation == "sectionpos_ranges" {
        let coordinate = |name: &str| {
            string(request, name)?
                .parse::<i32>()
                .map_err(|error| error.to_string())
        };
        let positions = match string(request, "mode")? {
            "between" => SectionPos::between_closed(
                coordinate("min_x")?,
                coordinate("min_y")?,
                coordinate("min_z")?,
                coordinate("max_x")?,
                coordinate("max_y")?,
                coordinate("max_z")?,
            ),
            "cube" => SectionPos::cube(
                SectionPos::new(coordinate("x")?, coordinate("y")?, coordinate("z")?),
                coordinate("radius")?,
            ),
            "around-chunk" => SectionPos::around_chunk(
                ChunkPos::new(coordinate("chunk_x")?, coordinate("chunk_z")?),
                coordinate("radius")?,
                coordinate("min_section")?,
                coordinate("max_section")?,
            ),
            _ => return Err("unknown SectionPos range mode".into()),
        };
        return Ok(json!({
            "value": sectionpos_ranges_snapshot(positions),
        }));
    }
    if operation == "sectionpos_blocks_inside" {
        return Ok(json!({
            "value": blockpos_positions_snapshot(sectionpos(request)?.blocks_inside()),
        }));
    }
    if operation == "sectionpos_around_and_at_block_pos" {
        let coordinate = |name: &str| {
            string(request, name)?
                .parse::<i32>()
                .map_err(|error| error.to_string())
        };
        let section_nodes = match string(request, "mode")? {
            "coordinates" => SectionPos::around_and_at_coordinates(
                coordinate("x")?,
                coordinate("y")?,
                coordinate("z")?,
            ),
            "blockpos" => SectionPos::around_and_at_block_pos(blockpos(request)?),
            "packed" => SectionPos::around_and_at_packed(u64_hex_field(request, "block_node")?),
            _ => return Err("unknown SectionPos around-and-at mode".into()),
        };
        return Ok(json!({
            "value": sectionpos_nodes_snapshot(section_nodes),
        }));
    }
    if operation == "sectionpos_double_coordinates" {
        let value = f64_field(request, "value")?;
        let result = match string(request, "fn")? {
            "pos_to_section" => SectionPos::pos_to_section_coord(value),
            "block_to_section" => SectionPos::block_to_section_coord_f64(value),
            _ => return Err("unknown SectionPos double coordinate function".into()),
        };
        return Ok(json!({"value": result.to_string()}));
    }
    if operation == "sectionpos_of_overloads" {
        let value = match string(request, "mode")? {
            "blockpos" => SectionPos::from_block_pos(blockpos(request)?),
            "chunkpos" => SectionPos::from_chunk_pos(
                chunkpos(request)?,
                string(request, "section_y")?
                    .parse::<i32>()
                    .map_err(|error| error.to_string())?,
            ),
            "position" => SectionPos::from_position(Vec3::new(
                f64_field(request, "x")?,
                f64_field(request, "y")?,
                f64_field(request, "z")?,
            )),
            "packed" => SectionPos::unpack(u64_hex_field(request, "section_node")?),
            _ => return Err("unknown SectionPos of overload mode".into()),
        };
        return Ok(json!({"value": sectionpos_snapshot(value)}));
    }
    if operation == "chunkpos_validity" {
        return Ok(json!({
            "value": chunkpos_validity_snapshot(chunkpos(request)?),
        }));
    }
    if operation == "mth_floating_primitives" {
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
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_constants" {
        return match string(request, "name")? {
            "pi" => Ok(json!({"bits": format!("{:08x}", math::MTH_PI.to_bits())})),
            "half_pi" => Ok(json!({"bits": format!("{:08x}", math::MTH_HALF_PI.to_bits())})),
            "two_pi" => Ok(json!({"bits": format!("{:08x}", math::MTH_TWO_PI.to_bits())})),
            "deg_to_rad" => Ok(json!({"bits": format!("{:08x}", math::MTH_DEG_TO_RAD.to_bits())})),
            "rad_to_deg" => Ok(json!({"bits": format!("{:08x}", math::MTH_RAD_TO_DEG.to_bits())})),
            "epsilon" => Ok(json!({"bits": format!("{:08x}", math::MTH_EPSILON.to_bits())})),
            "sqrt_of_two" => {
                Ok(json!({"bits": format!("{:08x}", math::MTH_SQRT_OF_TWO.to_bits())}))
            }
            "y_axis" => Ok(json!({"value": vec3f_snapshot(math::MTH_Y_AXIS)})),
            "x_axis" => Ok(json!({"value": vec3f_snapshot(math::MTH_X_AXIS)})),
            "z_axis" => Ok(json!({"value": vec3f_snapshot(math::MTH_Z_AXIS)})),
            _ => Err("unknown Mth constant".into()),
        };
    }
    if operation == "mth_power_helpers" {
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
        return Ok(json!({"value": value}));
    }
    if operation == "mth_frac" {
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
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_create_insecure_uuid" {
        let seed = string(request, "seed")?
            .parse::<i64>()
            .map_err(|error| error.to_string())?;
        let mut random = RandomSource::create(seed);
        let uuid = math::mth_create_insecure_uuid(&mut random);
        return Ok(json!({
            "most_hex": format!("{:016x}", uuid.most),
            "least_hex": format!("{:016x}", uuid.least),
        }));
    }
    if operation == "mth_murmur_hash3_mixer" {
        let input = string(request, "input")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        return Ok(json!({
            "value": math::mth_murmur_hash3_mixer(input).to_string(),
        }));
    }
    if operation == "mth_binary_search" {
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
        return Ok(json!({"value": value.to_string()}));
    }
    if operation == "mth_ray_intersects_aabb" {
        let ray_start = f64_vec3(
            request
                .get("ray_start")
                .ok_or("ray_start must be an object")?,
        )?;
        let ray_dir = f64_vec3(request.get("ray_dir").ok_or("ray_dir must be an object")?)?;
        let aabb = aabb_box(request.get("aabb").ok_or("aabb must be an object")?)?;
        return Ok(json!({
            "value": math::mth_ray_intersects_aabb(ray_start, ray_dir, aabb),
        }));
    }
    if operation == "mth_fast_inv_sqrt" {
        let bits = math::mth_fast_inv_sqrt_f64(f64_field(request, "input")?).to_bits();
        return Ok(json!({"bits": format!("{bits:016x}")}));
    }
    if operation == "mth_fast_inv_cube_root" {
        let bits = math::mth_fast_inv_cube_root_f32(f32_field(request, "input")?).to_bits();
        return Ok(json!({"bits": format!("{bits:08x}")}));
    }
    if operation == "mth_hsv_to_color" {
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
        return Ok(match color {
            Ok(value) => json!({"value": value.to_string()}),
            Err(error) => json!({"outcome": "error", "error": error}),
        });
    }
    if operation == "mth_angle_wrap_difference" {
        if string(request, "type")? != "f32" {
            return Err("unknown Mth angle wrap/difference type".into());
        }
        let value = match string(request, "fn")? {
            "wrap_degrees_90" => math::mth_wrap_degrees_90_f32(f32_field(request, "value")?),
            "degrees_difference" => math::mth_degrees_difference_f32(
                f32_field(request, "from")?,
                f32_field(request, "to")?,
            ),
            "degrees_difference_abs" => math::mth_degrees_difference_abs_f32(
                f32_field(request, "from")?,
                f32_field(request, "to")?,
            ),
            _ => return Err("unknown Mth angle wrap/difference function".into()),
        };
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_angle_approach" {
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
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_get_int" {
        let input = if request["input"].is_null() {
            None
        } else {
            Some(string(request, "input")?)
        };
        let default = string(request, "default")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        return Ok(json!({"value": math::mth_get_int(input, default).to_string()}));
    }
    if operation == "mth_get_seed" {
        let value = math::mth_get_seed(
            i32_field(request, "x")?,
            i32_field(request, "y")?,
            i32_field(request, "z")?,
        );
        return Ok(json!({"value": value.to_string()}));
    }
    if operation == "mth_atan2" {
        let value = math::mth_atan2(f64_field(request, "y")?, f64_field(request, "x")?);
        return Ok(json!({"bits": format!("{:016x}", value.to_bits())}));
    }
    if operation == "mth_inv_sqrt" {
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
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_integer_helpers" {
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
        return Ok(json!({"value": value}));
    }
    if operation == "mth_integer_basics" {
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
        return Ok(json!({"value": value}));
    }
    if operation == "mth_angle_helpers" {
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
        return Ok(result);
    }
    if operation == "mth_rounding_helpers" {
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
        return Ok(json!({"value": value}));
    }
    if operation == "mth_positive_modulo" {
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
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_abs_max" {
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
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_equal" {
        let width = string(request, "type")?;
        let equal = match width {
            "f32" => math::equal_f32(f32_field(request, "first")?, f32_field(request, "second")?),
            "f64" => math::equal_f64(f64_field(request, "first")?, f64_field(request, "second")?),
            _ => return Err("unknown Mth equal type".into()),
        };
        return Ok(json!({"equal": equal}));
    }
    if operation == "mth_clamped_lerp" {
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
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_clamped_map_f64" {
        let value = math::mth_clamped_map_f64(
            f64_field(request, "value")?,
            f64_field(request, "from_min")?,
            f64_field(request, "from_max")?,
            f64_field(request, "to_min")?,
            f64_field(request, "to_max")?,
        );
        return Ok(json!({"bits": format!("{:016x}", value.to_bits())}));
    }
    if operation == "mth_clamped_map_f32" {
        let value = math::mth_clamped_map_f32(
            f32_field(request, "value")?,
            f32_field(request, "from_min")?,
            f32_field(request, "from_max")?,
            f32_field(request, "to_min")?,
            f32_field(request, "to_max")?,
        );
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_map_f64" {
        let value = math::mth_map_f64(
            f64_field(request, "value")?,
            f64_field(request, "from_min")?,
            f64_field(request, "from_max")?,
            f64_field(request, "to_min")?,
            f64_field(request, "to_max")?,
        );
        return Ok(json!({"bits": format!("{:016x}", value.to_bits())}));
    }
    if operation == "mth_map_f32" {
        let value = math::mth_map_f32(
            f32_field(request, "value")?,
            f32_field(request, "from_min")?,
            f32_field(request, "from_max")?,
            f32_field(request, "to_min")?,
            f32_field(request, "to_max")?,
        );
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_wobble_f64" {
        let value = math::mth_wobble_f64(f64_field(request, "coord")?);
        return Ok(json!({"bits": format!("{:016x}", value.to_bits())}));
    }
    if operation == "mth_interpolation" {
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
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_next_random" {
        let mut random = RandomSource::create(random_seed(request)?);
        let width = string(request, "type")?;
        return match width {
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
        };
    }
    if operation == "mth_random_helpers" {
        let mut random = RandomSource::create(random_seed(request)?);
        return match string(request, "fn")? {
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
        };
    }
    if operation == "mth_length" {
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
                math::mth_length_f64_2(f64_field(request, "x")?, f64_field(request, "y")?,)
                    .to_bits()
            ),
            ("f32_2", "length") => format!(
                "{:08x}",
                math::mth_length_f32_2(f32_field(request, "x")?, f32_field(request, "y")?,)
                    .to_bits()
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
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_quantize" {
        let value = f64_field(request, "value")?;
        let resolution = string(request, "resolution")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        return Ok(json!({
            "value": math::mth_quantize(value, resolution).to_string()
        }));
    }
    if operation == "mth_out_from_origin" {
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
        return Ok(result);
    }
    if operation == "mth_mul_and_truncate" {
        let numerator = i32_string_field(request, "numerator")?;
        let denominator = i32_string_field(request, "denominator")?;
        let factor = i32_string_field(request, "factor")?;
        return Ok(json!({
            "value": math::mth_mul_and_truncate(numerator, denominator, factor).to_string()
        }));
    }
    if operation == "mth_rotation_around_axis" {
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
        return Ok(json!({
            "value": quaternion_snapshot(math::mth_rotation_around_axis(axis, rotation))
        }));
    }
    if operation == "mth_inverse_lerp" {
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
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_lerp" {
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
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_lerp_int" || operation == "mth_lerp_discrete" {
        let alpha = f32_field(request, "alpha")?;
        let start = i32_string_field(request, "p0")?;
        let end = i32_string_field(request, "p1")?;
        let value = if operation == "mth_lerp_int" {
            math::mth_lerp_int(alpha, start, end)
        } else {
            math::mth_lerp_discrete(alpha, start, end)
        };
        return Ok(json!({"value": value.to_string()}));
    }
    if operation == "mth_smoothstep" || operation == "mth_smoothstep_derivative" {
        let x = f64_field(request, "x")?;
        let value = if operation == "mth_smoothstep" {
            math::mth_smoothstep(x)
        } else {
            math::mth_smoothstep_derivative(x)
        };
        return Ok(json!({"bits": format!("{:016x}", value.to_bits())}));
    }
    if operation == "mth_sign" {
        let value = math::mth_sign(f64_field(request, "x")?);
        return Ok(json!({"value": value.to_string()}));
    }
    if operation == "mth_rot_lerp" {
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
        return Ok(json!({"bits": bits}));
    }
    if operation == "mth_rot_lerp_rad" {
        let value = math::mth_rot_lerp_rad(
            f32_field(request, "alpha")?,
            f32_field(request, "from")?,
            f32_field(request, "to")?,
        );
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_triangle_wave" {
        let value =
            math::mth_triangle_wave(f32_field(request, "index")?, f32_field(request, "period")?);
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_square_f32" {
        let value = math::mth_square_f32(f32_field(request, "x")?);
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_cube_f32" {
        let value = math::mth_cube_f32(f32_field(request, "x")?);
        return Ok(json!({"bits": format!("{:08x}", value.to_bits())}));
    }
    if operation == "mth_square_f64" {
        let value = math::mth_square_f64(f64_field(request, "x")?);
        return Ok(json!({"bits": format!("{:016x}", value.to_bits())}));
    }
    if operation == "mth_square_i32" {
        let value = math::mth_square_i32(
            string(request, "x")?
                .parse::<i32>()
                .map_err(|error| error.to_string())?,
        );
        return Ok(json!({"value": value.to_string()}));
    }
    if operation == "mth_square_i64" {
        let value = math::mth_square_i64(
            string(request, "x")?
                .parse::<i64>()
                .map_err(|error| error.to_string())?,
        );
        return Ok(json!({"value": value.to_string()}));
    }
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
    if operation == "keyboard_input" {
        let keys = Input::new(
            request["up"].as_bool().ok_or("up must be a boolean")?,
            request["down"].as_bool().ok_or("down must be a boolean")?,
            request["left"].as_bool().ok_or("left must be a boolean")?,
            request["right"]
                .as_bool()
                .ok_or("right must be a boolean")?,
            request["jump"].as_bool().ok_or("jump must be a boolean")?,
            request["shift"]
                .as_bool()
                .ok_or("shift must be a boolean")?,
            request["sprint"]
                .as_bool()
                .ok_or("sprint must be a boolean")?,
        );
        let mut keyboard = KeyboardInput::new(keys);
        keyboard.tick();
        return Ok(json!({
            "key_presses": input_snapshot(keyboard.key_presses),
            "move_vector": vec2_snapshot(keyboard.move_vector()),
        }));
    }
    if operation == "local_player_input" {
        let inputs = request["inputs"]
            .as_array()
            .ok_or("inputs must be an array")?;
        let states = inputs
            .iter()
            .map(|input| {
                let label = string(input, "label")?;
                let original = Vec2::new(f32_field(input, "x")?, f32_field(input, "y")?);
                let modifiers = LocalPlayerInputModifiers {
                    using_item: input["using_item"]
                        .as_bool()
                        .ok_or("using_item must be a boolean")?,
                    passenger: input["passenger"]
                        .as_bool()
                        .ok_or("passenger must be a boolean")?,
                    item_use_speed_multiplier: f32_field(input, "item_use_speed_multiplier")?,
                    moving_slowly: input["moving_slowly"]
                        .as_bool()
                        .ok_or("moving_slowly must be a boolean")?,
                    sneaking_movement_factor: f32_field(input, "sneaking_movement_factor")?,
                };
                let modified = modify_local_player_input(original, modifiers);
                Ok(json!({
                    "label": label,
                    "input": vec2_snapshot(original),
                    "modified": vec2_snapshot(modified),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(json!({"states": states}));
    }
    if operation == "local_player_apply_input" {
        let inputs = request["inputs"]
            .as_array()
            .ok_or("inputs must be an array")?;
        let states = inputs
            .iter()
            .map(|input| {
                let label = string(input, "label")?;
                let applied = apply_local_player_input(
                    LocalPlayerInputState {
                        move_vector: Vec2::new(
                            f32_field(input, "move_x")?,
                            f32_field(input, "move_y")?,
                        ),
                        jump_pressed: input["jump"].as_bool().ok_or("jump must be a boolean")?,
                        current_impulses: Vec2::new(
                            f32_field(input, "xxa")?,
                            f32_field(input, "zza")?,
                        ),
                        current_jumping: input["jumping"]
                            .as_bool()
                            .ok_or("jumping must be a boolean")?,
                        current_bob: Vec2::new(
                            f32_field(input, "x_bob")?,
                            f32_field(input, "y_bob")?,
                        ),
                        previous_bob: Vec2::new(
                            f32_field(input, "x_bob_o")?,
                            f32_field(input, "y_bob_o")?,
                        ),
                        rotation: Vec2::new(f32_field(input, "x_rot")?, f32_field(input, "y_rot")?),
                        controlled_camera: input["controlled_camera"]
                            .as_bool()
                            .ok_or("controlled_camera must be a boolean")?,
                    },
                    LocalPlayerInputModifiers {
                        using_item: false,
                        passenger: false,
                        item_use_speed_multiplier: 0.2,
                        moving_slowly: false,
                        sneaking_movement_factor: 0.3,
                    },
                );
                Ok(json!({
                    "label": label,
                    "movement": vec2_snapshot(applied.movement),
                    "jumping": applied.jumping,
                    "x_bob": format!("{:08x}", applied.x_bob.to_bits()),
                    "y_bob": format!("{:08x}", applied.y_bob.to_bits()),
                    "x_bob_o": format!("{:08x}", applied.x_bob_old.to_bits()),
                    "y_bob_o": format!("{:08x}", applied.y_bob_old.to_bits()),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(json!({"states": states}));
    }
    if operation == "local_player_can_start_sprinting" {
        let inputs = request["inputs"]
            .as_array()
            .ok_or("inputs must be an array")?;
        let states = inputs
            .iter()
            .map(|input| {
                let state = LocalPlayerSprintGateInput {
                    sprinting: input["sprinting"]
                        .as_bool()
                        .ok_or("sprinting must be a boolean")?,
                    forward_impulse: f32_field(input, "move_y")?,
                    mobility_restricted: input["mobility_restricted"]
                        .as_bool()
                        .ok_or("mobility_restricted must be a boolean")?,
                    enough_food: input["enough_food"]
                        .as_bool()
                        .ok_or("enough_food must be a boolean")?,
                    may_fly: input["mayfly"]
                        .as_bool()
                        .ok_or("mayfly must be a boolean")?,
                    shallow_water: input["shallow_water"]
                        .as_bool()
                        .ok_or("shallow_water must be a boolean")?,
                    flying: input["flying"]
                        .as_bool()
                        .ok_or("flying must be a boolean")?,
                    slow_due_to_using_item: input["using_item"]
                        .as_bool()
                        .ok_or("using_item must be a boolean")?,
                    fall_flying: input["fall_flying"]
                        .as_bool()
                        .ok_or("fall_flying must be a boolean")?,
                    underwater: input["underwater"]
                        .as_bool()
                        .ok_or("underwater must be a boolean")?,
                    moving_slowly: input["moving_slowly"]
                        .as_bool()
                        .ok_or("moving_slowly must be a boolean")?,
                };
                Ok(json!({
                    "label": string(input, "label")?,
                    "can_start": can_start_local_player_sprinting(state),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(json!({"states": states}));
    }
    if operation == "local_player_fluid_falling_adjustment" {
        let inputs = request["inputs"]
            .as_array()
            .ok_or("inputs must be an array")?;
        let states = inputs
            .iter()
            .map(|input| {
                let movement =
                    adjust_local_player_fluid_falling_movement(LocalPlayerFluidFallingInput {
                        base_gravity: f64_field(input, "base_gravity")?,
                        is_falling: input["is_falling"]
                            .as_bool()
                            .ok_or("is_falling must be a boolean")?,
                        sprinting: input["sprinting"]
                            .as_bool()
                            .ok_or("sprinting must be a boolean")?,
                        movement: Vec3::new(
                            f64_field(input, "x")?,
                            f64_field(input, "y")?,
                            f64_field(input, "z")?,
                        ),
                    });
                Ok(json!({
                    "label": string(input, "label")?,
                    "movement": vec3_snapshot(movement),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(json!({"states": states}));
    }
    if operation == "local_player_climbable_movement" {
        let inputs = request["inputs"]
            .as_array()
            .ok_or("inputs must be an array")?;
        let states = inputs
            .iter()
            .map(|input| {
                let movement =
                    clamp_local_player_climbable_movement(LocalPlayerClimbableMovementInput {
                        movement: Vec3::new(
                            f64_field(input, "x")?,
                            f64_field(input, "y")?,
                            f64_field(input, "z")?,
                        ),
                        on_climbable: input["on_climbable"]
                            .as_bool()
                            .ok_or("on_climbable must be a boolean")?,
                        scaffolding: input["scaffolding"]
                            .as_bool()
                            .ok_or("scaffolding must be a boolean")?,
                        suppress_sliding_down_ladder: input["suppress_sliding_down_ladder"]
                            .as_bool()
                            .ok_or("suppress_sliding_down_ladder must be a boolean")?,
                    });
                Ok(json!({
                    "label": string(input, "label")?,
                    "movement": vec3_snapshot(movement),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(json!({"states": states}));
    }
    if operation == "local_player_friction_influenced_speed" {
        let inputs = request["inputs"]
            .as_array()
            .ok_or("inputs must be an array")?;
        let states = inputs
            .iter()
            .map(|input| {
                let speed = local_player_friction_influenced_speed(
                    LocalPlayerFrictionInfluencedSpeedInput {
                        on_ground: input["on_ground"]
                            .as_bool()
                            .ok_or("on_ground must be a boolean")?,
                        block_friction: f32_field(input, "block_friction")?,
                        speed: f32_field(input, "speed")?,
                        flying: input["flying"]
                            .as_bool()
                            .ok_or("flying must be a boolean")?,
                        flying_speed: f32_field(input, "flying_speed")?,
                        sprinting: input["sprinting"]
                            .as_bool()
                            .ok_or("sprinting must be a boolean")?,
                        passenger: input["passenger"]
                            .as_bool()
                            .ok_or("passenger must be a boolean")?,
                    },
                );
                Ok(json!({
                    "label": string(input, "label")?,
                    "speed": format!("{:08x}", speed.to_bits()),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(json!({"states": states}));
    }
    if operation == "local_player_modified_friction" {
        let inputs = request["inputs"]
            .as_array()
            .ok_or("inputs must be an array")?;
        let states = inputs
            .iter()
            .map(|input| {
                let friction = local_player_modified_friction(LocalPlayerModifiedFrictionInput {
                    friction: f32_field(input, "friction")?,
                    modifier: f32_field(input, "modifier")?,
                });
                Ok(json!({
                    "label": string(input, "label")?,
                    "friction": format!("{:08x}", friction.to_bits()),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(json!({"states": states}));
    }
    if operation == "local_player_travel_flying" {
        let inputs = request["inputs"]
            .as_array()
            .ok_or("inputs must be an array")?;
        let states = inputs
            .iter()
            .map(|input| {
                let movement = local_player_travel_flying(LocalPlayerTravelFlyingInput {
                    input: f64_vec3(input.get("input").ok_or("input must be a vector")?)?,
                    delta_movement: f64_vec3(input.get("delta").ok_or("delta must be a vector")?)?,
                    y_rot: f32_field(input, "y_rot")?,
                    in_water: input["in_water"]
                        .as_bool()
                        .ok_or("in_water must be a boolean")?,
                    in_lava: input["in_lava"]
                        .as_bool()
                        .ok_or("in_lava must be a boolean")?,
                    water_speed: f32_field(input, "water_speed")?,
                    lava_speed: f32_field(input, "lava_speed")?,
                    air_speed: f32_field(input, "air_speed")?,
                });
                Ok(json!({
                    "label": string(input, "label")?,
                    "movement": vec3_snapshot(movement),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(json!({"states": states}));
    }
    if operation == "local_player_travel_in_water" {
        let inputs = request["inputs"]
            .as_array()
            .ok_or("inputs must be an array")?;
        let states = inputs
            .iter()
            .map(|input| {
                let movement = local_player_travel_in_water(LocalPlayerTravelInWaterInput {
                    input: f64_vec3(input.get("input").ok_or("input must be a vector")?)?,
                    delta_movement: f64_vec3(input.get("delta").ok_or("delta must be a vector")?)?,
                    y_rot: f32_field(input, "y_rot")?,
                    sprinting: input["sprinting"]
                        .as_bool()
                        .ok_or("sprinting must be a boolean")?,
                    water_slow_down: f32_field(input, "water_slow_down")?,
                    water_efficiency: f32_field(input, "water_efficiency")?,
                    on_ground: input["on_ground"]
                        .as_bool()
                        .ok_or("on_ground must be a boolean")?,
                    speed: f32_field(input, "speed")?,
                    dolphins_grace: input["dolphins_grace"]
                        .as_bool()
                        .ok_or("dolphins_grace must be a boolean")?,
                    horizontal_collision: input["horizontal_collision"]
                        .as_bool()
                        .ok_or("horizontal_collision must be a boolean")?,
                    on_climbable: input["on_climbable"]
                        .as_bool()
                        .ok_or("on_climbable must be a boolean")?,
                    free_after_jump: input["free_after_jump"]
                        .as_bool()
                        .ok_or("free_after_jump must be a boolean")?,
                    base_gravity: f64_field(input, "base_gravity")?,
                    is_falling: input["is_falling"]
                        .as_bool()
                        .ok_or("is_falling must be a boolean")?,
                });
                Ok(json!({
                    "label": string(input, "label")?,
                    "movement": vec3_snapshot(movement),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(json!({"states": states}));
    }
    if operation == "local_player_travel_in_lava" {
        let inputs = request["inputs"]
            .as_array()
            .ok_or("inputs must be an array")?;
        let states = inputs
            .iter()
            .map(|input| {
                let movement = local_player_travel_in_lava(LocalPlayerTravelInLavaInput {
                    input: f64_vec3(input.get("input").ok_or("input must be a vector")?)?,
                    delta_movement: f64_vec3(input.get("delta").ok_or("delta must be a vector")?)?,
                    y_rot: f32_field(input, "y_rot")?,
                    sprinting: input["sprinting"]
                        .as_bool()
                        .ok_or("sprinting must be a boolean")?,
                    shallow_lava: input["shallow_lava"]
                        .as_bool()
                        .ok_or("shallow_lava must be a boolean")?,
                    horizontal_collision: input["horizontal_collision"]
                        .as_bool()
                        .ok_or("horizontal_collision must be a boolean")?,
                    free_after_jump: input["free_after_jump"]
                        .as_bool()
                        .ok_or("free_after_jump must be a boolean")?,
                    base_gravity: f64_field(input, "base_gravity")?,
                    is_falling: input["is_falling"]
                        .as_bool()
                        .ok_or("is_falling must be a boolean")?,
                });
                Ok(json!({
                    "label": string(input, "label")?,
                    "movement": vec3_snapshot(movement),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(json!({"states": states}));
    }
    if operation == "local_player_handle_relative_friction" {
        let inputs = request["inputs"]
            .as_array()
            .ok_or("inputs must be an array")?;
        let states = inputs
            .iter()
            .map(|input| {
                let movement =
                    local_player_handle_relative_friction(LocalPlayerHandleRelativeFrictionInput {
                        input: f64_vec3(input.get("input").ok_or("input must be a vector")?)?,
                        delta_movement: f64_vec3(
                            input.get("delta").ok_or("delta must be a vector")?,
                        )?,
                        y_rot: f32_field(input, "y_rot")?,
                        on_ground: input["on_ground"]
                            .as_bool()
                            .ok_or("on_ground must be a boolean")?,
                        block_friction: f32_field(input, "block_friction")?,
                        speed: f32_field(input, "speed")?,
                        flying_speed: f32_field(input, "flying_speed")?,
                        flying: input["flying"]
                            .as_bool()
                            .ok_or("flying must be a boolean")?,
                        sprinting: input["sprinting"]
                            .as_bool()
                            .ok_or("sprinting must be a boolean")?,
                        passenger: input["passenger"]
                            .as_bool()
                            .ok_or("passenger must be a boolean")?,
                        on_climbable: input["on_climbable"]
                            .as_bool()
                            .ok_or("on_climbable must be a boolean")?,
                        scaffolding: input["scaffolding"]
                            .as_bool()
                            .ok_or("scaffolding must be a boolean")?,
                        suppress_sliding_down_ladder: input["suppress_sliding_down_ladder"]
                            .as_bool()
                            .ok_or("suppress_sliding_down_ladder must be a boolean")?,
                        horizontal_collision: input["horizontal_collision"]
                            .as_bool()
                            .ok_or("horizontal_collision must be a boolean")?,
                        jumping: input["jumping"]
                            .as_bool()
                            .ok_or("jumping must be a boolean")?,
                    });
                Ok(json!({
                    "label": string(input, "label")?,
                    "movement": vec3_snapshot(movement),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(json!({"states": states}));
    }
    if operation == "local_player_travel_in_air" {
        let inputs = request["inputs"]
            .as_array()
            .ok_or("inputs must be an array")?;
        let states = inputs
            .iter()
            .map(|input| {
                let levitation_amplifier = input["levitation_amplifier"]
                    .as_i64()
                    .ok_or("levitation_amplifier must be an integer")?;
                let levitation_amplifier = if levitation_amplifier >= 0 {
                    Some(
                        i32::try_from(levitation_amplifier)
                            .map_err(|_| "levitation_amplifier is out of range".to_owned())?,
                    )
                } else {
                    None
                };
                let min_y =
                    i32::try_from(input["min_y"].as_i64().ok_or("min_y must be an integer")?)
                        .map_err(|_| "min_y is out of range".to_owned())?;
                let movement = local_player_travel_in_air(LocalPlayerTravelInAirInput {
                    input: f64_vec3(input.get("input").ok_or("input must be a vector")?)?,
                    delta_movement: f64_vec3(input.get("delta").ok_or("delta must be a vector")?)?,
                    y_rot: f32_field(input, "y_rot")?,
                    on_ground: input["on_ground"]
                        .as_bool()
                        .ok_or("on_ground must be a boolean")?,
                    block_friction: f32_field(input, "block_friction")?,
                    friction_modifier: f32_field(input, "friction_modifier")?,
                    speed: f32_field(input, "speed")?,
                    flying_speed: f32_field(input, "flying_speed")?,
                    flying: input["flying"]
                        .as_bool()
                        .ok_or("flying must be a boolean")?,
                    sprinting: input["sprinting"]
                        .as_bool()
                        .ok_or("sprinting must be a boolean")?,
                    passenger: input["passenger"]
                        .as_bool()
                        .ok_or("passenger must be a boolean")?,
                    on_climbable: input["on_climbable"]
                        .as_bool()
                        .ok_or("on_climbable must be a boolean")?,
                    scaffolding: input["scaffolding"]
                        .as_bool()
                        .ok_or("scaffolding must be a boolean")?,
                    suppress_sliding_down_ladder: input["suppress_sliding_down_ladder"]
                        .as_bool()
                        .ok_or(
                        "suppress_sliding_down_ladder must be a boolean",
                    )?,
                    horizontal_collision: input["horizontal_collision"]
                        .as_bool()
                        .ok_or("horizontal_collision must be a boolean")?,
                    jumping: input["jumping"]
                        .as_bool()
                        .ok_or("jumping must be a boolean")?,
                    levitation_amplifier,
                    gravity: f64_field(input, "gravity")?,
                    client_side: input["client_side"]
                        .as_bool()
                        .ok_or("client_side must be a boolean")?,
                    has_chunk_at: input["has_chunk_at"]
                        .as_bool()
                        .ok_or("has_chunk_at must be a boolean")?,
                    current_y: f64_field(input, "current_y")?,
                    min_y,
                    should_discard_friction: input["should_discard_friction"]
                        .as_bool()
                        .ok_or("should_discard_friction must be a boolean")?,
                    air_drag_modifier: f32_field(input, "air_drag_modifier")?,
                    omnidirectional_air_mover: input["omnidirectional_air_mover"]
                        .as_bool()
                        .ok_or("omnidirectional_air_mover must be a boolean")?,
                });
                Ok(json!({
                    "label": string(input, "label")?,
                    "movement": vec3_snapshot(movement),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(json!({"states": states}));
    }
    if operation == "local_player_update_fall_flying_movement" {
        let inputs = request["inputs"]
            .as_array()
            .ok_or("inputs must be an array")?;
        let states = inputs
            .iter()
            .map(|input| {
                let movement = local_player_update_fall_flying_movement(
                    LocalPlayerUpdateFallFlyingMovementInput {
                        look_angle: f64_vec3(input.get("look").ok_or("look must be a vector")?)?,
                        delta_movement: f64_vec3(
                            input.get("delta").ok_or("delta must be a vector")?,
                        )?,
                        x_rot: f32_field(input, "x_rot")?,
                        gravity: f64_field(input, "gravity")?,
                    },
                );
                Ok(json!({
                    "label": string(input, "label")?,
                    "movement": vec3_snapshot(movement),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(json!({"states": states}));
    }
    if operation == "local_player_travel_dispatch" {
        let inputs = request["inputs"]
            .as_array()
            .ok_or("inputs must be an array")?;
        let states = inputs
            .iter()
            .map(|input| {
                let result = local_player_travel_dispatch(LocalPlayerTravelDispatchInput {
                    parameters: local_player_travel_dispatch_parameters(input)?,
                    delta_movement: f64_vec3(input.get("delta").ok_or("delta must be a vector")?)?,
                });
                Ok(json!({
                    "label": string(input, "label")?,
                    "mode": result.mode.as_str(),
                    "fall_flying_climbable_fallback": result.fall_flying_climbable_fallback,
                    "movement": vec3_snapshot(result.movement),
                }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        return Ok(json!({"states": states}));
    }
    if operation == "character_event" {
        return character_event_result(request);
    }
    if operation == "preedit_event" {
        return preedit_event_result(request);
    }
    if operation == "option_values" {
        return option_values_result(request);
    }
    if operation == "sliderable_enum" {
        return sliderable_enum_result(request);
    }
    if operation == "cycleable_enum" {
        return cycleable_enum_result(request);
    }
    if operation == "alt_enum" {
        return alt_enum_result(request);
    }
    if operation == "lazy_enum" {
        return lazy_enum_result(request);
    }
    if operation == "clamping_lazy_max_int_range" {
        return clamping_lazy_max_int_range_result(request);
    }
    if operation == "mapped_int_range" {
        return mapped_int_range_result(request);
    }
    if operation == "mapped_unit_double" {
        return mapped_unit_double_result(request);
    }
    if operation == "scroll_wheel" {
        return scroll_wheel_result(request);
    }
    if operation == "smooth_double" {
        return smooth_double_result(request);
    }
    if operation == "input_with_modifiers" {
        return input_event_result(request);
    }
    if operation == "key_mapping" {
        return key_mapping_result(request);
    }
    if operation == "toggle_key_mapping" {
        return toggle_key_mapping_result(request);
    }
    if operation == "direction" {
        let mut facing =
            Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        let mut events = vec![direction_snapshot(facing)];
        let steps = request["steps"]
            .as_array()
            .ok_or("steps must be an array")?;
        for step in steps {
            let axis = Axis::from_name(string(step, "axis")?).ok_or("unknown axis")?;
            let clockwise = step["clockwise"]
                .as_bool()
                .ok_or("clockwise must be a boolean")?;
            facing = if clockwise {
                facing.clockwise(axis)
            } else {
                facing.counter_clockwise(axis)
            };
            events.push(direction_snapshot(facing));
        }
        return Ok(json!({"events": events}));
    }
    if operation == "direction_axial_rotation" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        return Ok(json!({
            "clockwise": direction_snapshot(facing.clockwise(axis)),
            "counter_clockwise": direction_snapshot(facing.counter_clockwise(axis)),
        }));
    }
    if operation == "direction_y_rotation" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        let clockwise = match facing.clockwise_y() {
            Ok(direction) => json!({"outcome": "value", "value": direction_snapshot(direction)}),
            Err(DirectionYRotationError::ClockwiseVertical(direction)) => json!({
                "outcome": "error",
                "error": format!("Unable to get Y-rotated facing of {}", direction.name()),
            }),
            Err(DirectionYRotationError::CounterClockwiseVertical(_)) => {
                unreachable!("clockwise_y cannot return a counter-clockwise error")
            }
        };
        let counter_clockwise = match facing.counter_clockwise_y() {
            Ok(direction) => json!({"outcome": "value", "value": direction_snapshot(direction)}),
            Err(DirectionYRotationError::CounterClockwiseVertical(direction)) => json!({
                "outcome": "error",
                "error": format!("Unable to get CCW facing of {}", direction.name()),
            }),
            Err(DirectionYRotationError::ClockwiseVertical(_)) => {
                unreachable!("counter_clockwise_y cannot return a clockwise error")
            }
        };
        return Ok(json!({"clockwise": clockwise, "counter_clockwise": counter_clockwise}));
    }
    if operation == "direction_data" {
        let value = string(request, "value")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let facing = match string(request, "type")? {
            "3d" => Direction::from_3d_data_value(value),
            "2d" => Direction::from_2d_data_value(value),
            _ => return Err("unknown Direction data type".into()),
        };
        return Ok(json!({"value": direction_snapshot(facing)}));
    }
    if operation == "direction_y_rot" {
        let y_rot = f64_field(request, "value")?;
        return Ok(json!({
            "value": direction_snapshot(Direction::from_y_rot(y_rot)),
        }));
    }
    if operation == "direction_axis" {
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        let direction = match string(request, "direction")? {
            "positive" => AxisDirection::Positive,
            "negative" => AxisDirection::Negative,
            _ => return Err("unknown axis direction".into()),
        };
        return Ok(json!({
            "value": direction_snapshot(Direction::from_axis_and_direction(axis, direction)),
        }));
    }
    if operation == "direction_get" {
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        let direction = match string(request, "direction")? {
            "positive" => AxisDirection::Positive,
            "negative" => AxisDirection::Negative,
            _ => return Err("unknown axis direction".into()),
        };
        return Ok(json!({
            "value": direction_snapshot(Direction::get(direction, axis)),
        }));
    }
    if operation == "direction_unit_vectors" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        let (x, y, z) = facing.step();
        return Ok(json!({
            "unit_vec3i": {"x": x, "y": y, "z": z},
            "unit_vec3": vec3_snapshot(facing.unit_vec3()),
            "unit_vec3f": vec3f_snapshot(facing.unit_vec3f()),
        }));
    }
    if operation == "direction_step" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        return Ok(json!({
            "step_x": facing.get_step_x(),
            "step_y": facing.get_step_y(),
            "step_z": facing.get_step_z(),
            "step": vec3f_snapshot(facing.step_vec3f()),
        }));
    }
    if operation == "direction_axis_orientation" {
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        return Ok(json!({
            "vertical": axis.is_vertical(),
            "horizontal": axis.is_horizontal(),
        }));
    }
    if operation == "direction_axis_directions" {
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        let directions = axis.get_directions();
        return Ok(json!({
            "positive": direction_snapshot(axis.get_positive()),
            "negative": direction_snapshot(axis.get_negative()),
            "directions": directions.into_iter().map(direction_snapshot).collect::<Vec<_>>(),
        }));
    }
    if operation == "direction_axis_direction_values" {
        let direction = match string(request, "direction")? {
            "positive" => AxisDirection::Positive,
            "negative" => AxisDirection::Negative,
            _ => return Err("unknown axis direction".into()),
        };
        return Ok(json!({
            "step": direction.get_step(),
            "name": direction.name(),
            "opposite": direction.opposite().name(),
        }));
    }
    if operation == "direction_axis_choose" {
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        let int_value = |name: &str| {
            string(request, name)?
                .parse::<i32>()
                .map_err(|error| error.to_string())
        };
        let double_value = |name: &str| f64_field(request, name);
        let bool_value = |name: &str| {
            request[name]
                .as_bool()
                .ok_or_else(|| format!("{name} must be a boolean"))
        };
        return Ok(json!({
            "int": axis.choose_i32(int_value("int_x")?, int_value("int_y")?, int_value("int_z")?),
            "double": format!("{:016x}", axis.choose_f64(
                double_value("double_x")?, double_value("double_y")?, double_value("double_z")?,
            ).to_bits()),
            "boolean": axis.choose_bool(
                bool_value("bool_x")?, bool_value("bool_y")?, bool_value("bool_z")?,
            ),
        }));
    }
    if operation == "direction_axis_test" {
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        let input = match string(request, "direction")? {
            "none" => None,
            name => Some(Direction::from_name(name).ok_or("unknown direction")?),
        };
        return Ok(json!({"value": axis.test(input)}));
    }
    if operation == "direction_axis_plane" {
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        let plane = match axis.get_plane() {
            Plane::Horizontal => "horizontal",
            Plane::Vertical => "vertical",
        };
        return Ok(json!({"plane": plane}));
    }
    if operation == "direction_plane_test" {
        let plane = match string(request, "plane")? {
            "horizontal" => Plane::Horizontal,
            "vertical" => Plane::Vertical,
            _ => return Err("unknown plane".into()),
        };
        let input = match string(request, "direction")? {
            "none" => None,
            name => Some(Direction::from_name(name).ok_or("unknown direction")?),
        };
        return Ok(json!({"value": plane.test(input)}));
    }
    if operation == "direction_plane_order" {
        let plane = match string(request, "plane")? {
            "horizontal" => Plane::Horizontal,
            "vertical" => Plane::Vertical,
            _ => return Err("unknown plane".into()),
        };
        return Ok(json!({
            "faces": plane.directions().iter().map(|direction| direction.name()).collect::<Vec<_>>(),
            "stream_faces": plane.directions().iter().map(|direction| direction.name()).collect::<Vec<_>>(),
            "axes": plane.axes().iter().map(|axis| axis.name()).collect::<Vec<_>>(),
            "length": plane.length(),
        }));
    }
    if operation == "direction_random" {
        let seed = string(request, "seed")?
            .parse::<i64>()
            .map_err(|error| error.to_string())?;
        let plane = match string(request, "plane")? {
            "horizontal" => Plane::Horizontal,
            "vertical" => Plane::Vertical,
            _ => return Err("unknown plane".into()),
        };
        let mut random = RandomSource::create(seed);
        let direction = Direction::get_random(&mut random);
        let axis = Axis::get_random(&mut random);
        let plane_direction = plane.get_random_direction(&mut random);
        let plane_axis = plane.get_random_axis(&mut random);
        return Ok(json!({
            "direction": direction_snapshot(direction),
            "axis": axis.name(),
            "plane_direction": direction_snapshot(plane_direction),
            "plane_axis": plane_axis.name(),
            "tail": random.next_int().to_string(),
        }));
    }
    if operation == "direction_plane_shuffle" {
        let seed = string(request, "seed")?
            .parse::<i64>()
            .map_err(|error| error.to_string())?;
        let plane = match string(request, "plane")? {
            "horizontal" => Plane::Horizontal,
            "vertical" => Plane::Vertical,
            _ => return Err("unknown plane".into()),
        };
        let mut random = RandomSource::create(seed);
        let faces = plane
            .shuffled_copy(&mut random)
            .into_iter()
            .map(|direction| direction.name())
            .collect::<Vec<_>>();
        return Ok(json!({
            "faces": faces,
            "tail": random.next_int().to_string(),
        }));
    }
    if operation == "direction_all_shuffle" {
        let seed = string(request, "seed")?
            .parse::<i64>()
            .map_err(|error| error.to_string())?;
        let mut random = RandomSource::create(seed);
        let faces = Direction::all_shuffled(&mut random)
            .into_iter()
            .map(|direction| direction.name())
            .collect::<Vec<_>>();
        return Ok(json!({
            "faces": faces,
            "tail": random.next_int().to_string(),
        }));
    }
    if operation == "direction_stream" {
        let faces = Direction::stream()
            .map(|direction| direction.name())
            .collect::<Vec<_>>();
        return Ok(json!({"faces": faces}));
    }
    if operation == "direction_to_y_rot" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        return Ok(json!({
            "bits": format!("{:08x}", facing.to_y_rot().to_bits()),
        }));
    }
    if operation == "direction_get_y_rot" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        return Ok(match Direction::get_y_rot(facing) {
            Ok(value) => json!({"outcome": "value", "bits": format!("{:08x}", value.to_bits())}),
            Err(DirectionYRotError::Vertical(direction)) => json!({
                "outcome": "error",
                "error": format!("No y-Rot for vertical axis: {}", direction.name()),
            }),
        });
    }
    if operation == "direction_rotation" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        return Ok(json!({"value": quaternion_snapshot(facing.get_rotation())}));
    }
    if operation == "direction_name_lookup" {
        let name = string(request, "name")?;
        return match string(request, "kind")? {
            "direction" => Ok(match Direction::from_name(name) {
                Some(direction) => json!({
                    "found": true,
                    "name": direction.name(),
                    "serialized_name": direction.name(),
                    "text": direction.name(),
                }),
                None => json!({"found": false}),
            }),
            "axis" => Ok(match Axis::from_name(name) {
                Some(axis) => json!({
                    "found": true,
                    "name": axis.name(),
                    "serialized_name": axis.name(),
                    "text": axis.name(),
                }),
                None => json!({"found": false}),
            }),
            _ => Err("unknown name lookup kind".into()),
        };
    }
    if operation == "direction_enum_text" {
        let name = string(request, "name")?;
        return match string(request, "kind")? {
            "axis_direction" => {
                let direction = match name {
                    "positive" => AxisDirection::Positive,
                    "negative" => AxisDirection::Negative,
                    _ => return Err("unknown axis direction".into()),
                };
                Ok(json!({
                    "enum_name": direction.enum_name(),
                    "name": direction.name(),
                    "text": direction.to_string_java(),
                }))
            }
            "plane" => {
                let plane = match name {
                    "horizontal" => Plane::Horizontal,
                    "vertical" => Plane::Vertical,
                    _ => return Err("unknown plane".into()),
                };
                Ok(json!({
                    "enum_name": plane.enum_name(),
                    "text": plane.to_string_java(),
                }))
            }
            _ => Err("unknown enum text kind".into()),
        };
    }
    if operation == "direction_enum_name" {
        let name = string(request, "name")?;
        return match string(request, "kind")? {
            "direction" => {
                let direction = Direction::from_name(name).ok_or("unknown direction name")?;
                Ok(json!({
                    "enum_name": direction.enum_name(),
                    "name": direction.name(),
                    "serialized_name": direction.name(),
                    "text": direction.name(),
                }))
            }
            "axis" => {
                let axis = Axis::from_name(name).ok_or("unknown axis name")?;
                Ok(json!({
                    "enum_name": axis.enum_name(),
                    "name": axis.name(),
                    "serialized_name": axis.name(),
                    "text": axis.name(),
                }))
            }
            _ => Err("unknown enum name kind".into()),
        };
    }
    if operation == "direction_approximate_nearest" {
        let facing = Direction::approximate_nearest(
            f64_field(request, "x")?,
            f64_field(request, "y")?,
            f64_field(request, "z")?,
        );
        return Ok(json!({"value": direction_snapshot(facing)}));
    }
    if operation == "direction_approximate_nearest_overloads" {
        let f32_facing = Direction::approximate_nearest_f32(
            f32_field(request, "f32_x")?,
            f32_field(request, "f32_y")?,
            f32_field(request, "f32_z")?,
        );
        let f64_vector = Vec3::new(
            f64_field(request, "x")?,
            f64_field(request, "y")?,
            f64_field(request, "z")?,
        );
        return Ok(json!({
            "f32": direction_snapshot(f32_facing),
            "f64": direction_snapshot(Direction::approximate_nearest(
                f64_vector.x, f64_vector.y, f64_vector.z,
            )),
            "vec3": direction_snapshot(Direction::approximate_nearest_vec3(f64_vector)),
        }));
    }
    if operation == "direction_nearest" {
        let coordinate = |name: &str| {
            string(request, name)?
                .parse::<i32>()
                .map_err(|error| error.to_string())
        };
        let or_else = match string(request, "or_else")? {
            "none" => None,
            name => Some(Direction::from_name(name).ok_or("unknown fallback direction")?),
        };
        let facing = Direction::get_nearest(
            coordinate("x")?,
            coordinate("y")?,
            coordinate("z")?,
            or_else,
        );
        return Ok(match facing {
            Some(facing) => json!({"value": direction_snapshot(facing)}),
            None => json!({}),
        });
    }
    if operation == "direction_axis_step_order" {
        let axes = Direction::axis_step_order(f64_field(request, "x")?, f64_field(request, "z")?);
        return Ok(json!({
            "axes": axes.into_iter().map(Axis::name).collect::<Vec<_>>(),
        }));
    }
    if operation == "direction_is_facing_angle" {
        let facing = Direction::from_name(string(request, "name")?).ok_or("unknown direction")?;
        return Ok(json!({
            "value": facing.is_facing_angle(f32_field(request, "angle")?),
        }));
    }
    if operation == "coord" {
        let value = string(request, "value")?
            .parse::<i32>()
            .map_err(|error| error.to_string())?;
        let out = match string(request, "fn")? {
            "block_to_section" => math::block_to_section_coord(value),
            "section_to_block" => math::section_to_block_coord(value),
            "section_relative" => math::section_relative(value),
            _ => return Err("unknown coord function".into()),
        };
        return Ok(json!({"value": out.to_string()}));
    }
    if operation == "vec3" {
        let mut vector = Vec3::new(
            f64_field(request, "x")?,
            f64_field(request, "y")?,
            f64_field(request, "z")?,
        );
        let mut events = vec![vec3_snapshot(vector)];
        let steps = request["steps"]
            .as_array()
            .ok_or("steps must be an array")?;
        for step in steps {
            vector = match string(step, "action")? {
                "normalize" => vector.normalized(),
                "scale" => vector.scale(f64_field(step, "factor")?),
                "reverse" => vector.reverse(),
                "multiply_vec3" => vector.multiply_vec3(f64_vec3(
                    step.get("scale").ok_or("scale must be an object")?,
                )?),
                "multiply_components" => vector.multiply_components(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                ),
                "horizontal" => vector.horizontal(),
                "add_vec3" => vector.add_vec3(f64_vec3(
                    step.get("delta").ok_or("delta must be an object")?,
                )?),
                "add_scalar" => vector.add_scalar(f64_field(step, "value")?),
                "add_components" => vector.add_components(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                ),
                "subtract_vec3" => vector.subtract_vec3(f64_vec3(
                    step.get("delta").ok_or("delta must be an object")?,
                )?),
                "subtract_scalar" => vector.subtract_scalar(f64_field(step, "value")?),
                "subtract_components" => vector.subtract_components(
                    f64_field(step, "x")?,
                    f64_field(step, "y")?,
                    f64_field(step, "z")?,
                ),
                "vector_to" => vector.vector_to(f64_vec3(
                    step.get("target").ok_or("target must be an object")?,
                )?),
                "cross" => vector.cross(f64_vec3(
                    step.get("other").ok_or("other must be an object")?,
                )?),
                _ => return Err("unknown Vec3 action".into()),
            };
            events.push(vec3_snapshot(vector));
        }
        return Ok(json!({"events": events}));
    }
    if operation == "vec3_dot" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
        return Ok(json!({"value": format!("{:016x}", first.dot(second).to_bits())}));
    }
    if operation == "vec3_distance_to_sqr" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
        return Ok(json!({"value": format!("{:016x}", first.distance_to_sqr(second).to_bits())}));
    }
    if operation == "vec3_distance_to" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
        return Ok(json!({"value": format!("{:016x}", first.distance_to(second).to_bits())}));
    }
    if operation == "vec3_distance_to_sqr_components" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let value = first.distance_to_sqr_components(
            f64_field(request, "x")?,
            f64_field(request, "y")?,
            f64_field(request, "z")?,
        );
        return Ok(json!({"value": format!("{:016x}", value.to_bits())}));
    }
    if operation == "vec3_horizontal_distances" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        return Ok(json!({
            "distance": format!("{:016x}", first.horizontal_distance().to_bits()),
            "distance_squared": format!("{:016x}", first.horizontal_distance_squared().to_bits()),
        }));
    }
    if operation == "vec3_closer_than" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
        return Ok(json!({
            "position": first.closer_than_position(second, f64_field(request, "distance")?),
            "xz_y": first.closer_than_xz_y(
                second,
                f64_field(request, "distance_xz")?,
                f64_field(request, "distance_y")?,
            ),
        }));
    }
    if operation == "vec3_offset_random" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let offset = f32_field(request, "offset")?;
        let seed = random_seed(request)?;
        let mut random = RandomSource::create(seed);
        let offset_random = first.offset_random(&mut random, offset);
        let mut random_xz = RandomSource::create(seed);
        let offset_random_xz = first.offset_random_xz(&mut random_xz, offset);
        return Ok(json!({
            "offset_random": vec3_snapshot(offset_random),
            "offset_random_xz": vec3_snapshot(offset_random_xz),
        }));
    }
    if operation == "vec3_lerp" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
        let alpha = f64_field(request, "alpha")?;
        return Ok(json!({"value": vec3_snapshot(first.lerp(second, alpha))}));
    }
    if operation == "vec3_rotate_clockwise_90" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        return Ok(json!({"value": vec3_snapshot(first.rotate_clockwise_90())}));
    }
    if operation == "vec3_is_finite" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        return Ok(json!({"value": first.is_finite()}));
    }
    if operation == "vec3_get" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        return Ok(json!({"value": format!("{:016x}", first.get(axis).to_bits())}));
    }
    if operation == "vec3_with" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let axis = Axis::from_name(string(request, "axis")?).ok_or("unknown axis")?;
        let value = f64_field(request, "value")?;
        return Ok(json!({"value": vec3_snapshot(first.with(axis, value))}));
    }
    if operation == "vec3_relative" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let direction =
            Direction::from_name(string(request, "direction")?).ok_or("unknown direction")?;
        let distance = f64_field(request, "distance")?;
        return Ok(json!({"value": vec3_snapshot(first.relative(direction, distance))}));
    }
    if operation == "vec3_align" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let axis_values = request["axes"].as_array().ok_or("axes must be an array")?;
        let mut axes = Vec::with_capacity(axis_values.len());
        for axis in axis_values {
            let name = axis.as_str().ok_or("axis must be a string")?;
            axes.push(Axis::from_name(name).ok_or("unknown axis")?);
        }
        return Ok(json!({"value": vec3_snapshot(first.align(&axes))}));
    }
    if operation == "vec3_projected_on" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let onto = f64_vec3(request.get("onto").ok_or("onto must be an object")?)?;
        return Ok(json!({"value": vec3_snapshot(first.projected_on(onto))}));
    }
    if operation == "vec3_to_vector3f" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        return Ok(json!({"value": vec3f_snapshot(first.to_vector3f())}));
    }
    if operation == "vec3_coordinates" {
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
        return Ok(json!({"value": vec3_snapshot(value)}));
    }
    if operation == "vec3_equality" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let second = f64_vec3(request.get("second").ok_or("second must be an object")?)?;
        return Ok(json!({
            "equals": first.equals_java(second),
            "first_hash": first.hash_code_java().to_string(),
            "second_hash": second.hash_code_java().to_string(),
        }));
    }
    if operation == "vec3_to_string" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        return Ok(json!({"value": first.to_string_java()}));
    }
    if operation == "vec3_rotate" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        let radians = f32_field(request, "radians")?;
        let value = match string(request, "axis")? {
            "x" => first.x_rot(radians),
            "y" => first.y_rot(radians),
            "z" => first.z_rot(radians),
            _ => return Err("unknown Vec3 rotation axis".into()),
        };
        return Ok(json!({"value": vec3_snapshot(value)}));
    }
    if operation == "vec3_direction_from_rotation" {
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
        return Ok(json!({"value": vec3_snapshot(value)}));
    }
    if operation == "vec3_local_coordinates" {
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
        return Ok(json!({"value": vec3_snapshot(value)}));
    }
    if operation == "vec3_rotation" {
        let first = f64_vec3(request.get("first").ok_or("first must be an object")?)?;
        return Ok(json!({"value": vec2_snapshot(first.rotation())}));
    }
    if operation == "vec2_rotate" {
        let first = request.get("first").ok_or("first must be an object")?;
        let vector = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
        return Ok(json!({
            "value": vec2_snapshot(vector.rotate(f64_field(request, "angle")?)),
        }));
    }
    if operation == "vec2_dot" {
        let first = request.get("first").ok_or("first must be an object")?;
        let second = request.get("second").ok_or("second must be an object")?;
        let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
        let second = Vec2::new(f32_field(second, "x")?, f32_field(second, "y")?);
        return Ok(json!({
            "value": format!("{:08x}", first.dot(second).to_bits()),
        }));
    }
    if operation == "vec2_add_vec2" {
        let first = request.get("first").ok_or("first must be an object")?;
        let second = request.get("second").ok_or("second must be an object")?;
        let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
        let second = Vec2::new(f32_field(second, "x")?, f32_field(second, "y")?);
        return Ok(json!({
            "value": vec2_snapshot(first.add_vec2(second)),
        }));
    }
    if operation == "vec2_add_scalar" {
        let first = request.get("first").ok_or("first must be an object")?;
        let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
        return Ok(json!({
            "value": vec2_snapshot(first.add_scalar(f32_field(request, "value")?)),
        }));
    }
    if operation == "vec2_distance_to_sqr" {
        let first = request.get("first").ok_or("first must be an object")?;
        let second = request.get("second").ok_or("second must be an object")?;
        let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
        let second = Vec2::new(f32_field(second, "x")?, f32_field(second, "y")?);
        return Ok(json!({
            "value": format!("{:08x}", first.distance_to_sqr(second).to_bits()),
        }));
    }
    if operation == "vec2_negated" {
        let first = request.get("first").ok_or("first must be an object")?;
        let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
        return Ok(json!({
            "value": vec2_snapshot(first.negated()),
        }));
    }
    if operation == "vec2_equality" {
        let first = request.get("first").ok_or("first must be an object")?;
        let second = request.get("second").ok_or("second must be an object")?;
        let first = Vec2::new(f32_field(first, "x")?, f32_field(first, "y")?);
        let second = Vec2::new(f32_field(second, "x")?, f32_field(second, "y")?);
        return Ok(json!({
            "equals": first.equals_java(second),
            "first_hash": first.hash_code_java().to_string(),
            "second_hash": second.hash_code_java().to_string(),
        }));
    }
    if operation == "vec2_constant" {
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
        return Ok(json!({"value": vec2_snapshot(value)}));
    }
    if operation == "vec3_constant" {
        let value = match string(request, "name")? {
            "zero" => Vec3::ZERO,
            "x_axis" => Vec3::X_AXIS,
            "y_axis" => Vec3::Y_AXIS,
            "z_axis" => Vec3::Z_AXIS,
            _ => return Err("unknown Vec3 constant".into()),
        };
        return Ok(json!({"value": vec3_snapshot(value)}));
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

fn nbt_type_name(value: &FieldValue) -> &'static str {
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

fn nbt_value_json(value: &FieldValue) -> Value {
    match value {
        FieldValue::Byte(value) => json!(value.to_string()),
        FieldValue::Short(value) => json!(value.to_string()),
        FieldValue::Int(value) => json!(value.to_string()),
        FieldValue::Long(value) => json!(value.to_string()),
        FieldValue::Float(value) => json!(format!("{:08x}", value.to_bits())),
        FieldValue::Double(value) => json!(format!("{:016x}", value.to_bits())),
        FieldValue::ByteArray(values) => {
            json!(
                values
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            )
        }
        FieldValue::Str(value) => json!(value),
        FieldValue::List(values) => {
            Value::Array(values.iter().map(nbt_description_json).collect::<Vec<_>>())
        }
        FieldValue::Compound(fields) => {
            let mut object = serde_json::Map::new();
            let mut sorted: Vec<&Field> = fields.iter().collect();
            sorted.sort_by(|a, b| a.key.cmp(&b.key));
            for field in sorted {
                object.insert(field.key.clone(), nbt_description_json(&field.value));
            }
            Value::Object(object)
        }
        FieldValue::IntArray(values) => {
            json!(
                values
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            )
        }
        FieldValue::LongArray(values) => {
            json!(
                values
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
            )
        }
    }
}

fn nbt_description_json(value: &FieldValue) -> Value {
    json!({
        "type": nbt_type_name(value),
        "value": nbt_value_json(value),
    })
}

fn nbt_field_json(field: &Field) -> Value {
    json!({
        "key": field.key,
        "type": nbt_type_name(&field.value),
        "value": nbt_value_json(&field.value),
    })
}

fn nbt_tree_values<T>(node: &Value, value_name: &str) -> Result<Vec<T>, String>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    node["value"]
        .as_array()
        .ok_or_else(|| format!("{value_name} value must be an array"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| format!("{value_name} values must be strings"))?
                .parse::<T>()
                .map_err(|error| error.to_string())
        })
        .collect()
}

fn nbt_tree_value(node: &Value) -> Result<FieldValue, String> {
    match string(node, "type")? {
        "byte" => Ok(FieldValue::Byte(
            string(node, "value")?
                .parse::<i8>()
                .map_err(|error| error.to_string())?,
        )),
        "short" => Ok(FieldValue::Short(
            string(node, "value")?
                .parse::<i16>()
                .map_err(|error| error.to_string())?,
        )),
        "int" => Ok(FieldValue::Int(
            string(node, "value")?
                .parse::<i32>()
                .map_err(|error| error.to_string())?,
        )),
        "long" => Ok(FieldValue::Long(
            string(node, "value")?
                .parse::<i64>()
                .map_err(|error| error.to_string())?,
        )),
        "float" => Ok(FieldValue::Float(f32::from_bits(
            u32::from_str_radix(string(node, "value")?, 16).map_err(|error| error.to_string())?,
        ))),
        "double" => Ok(FieldValue::Double(f64::from_bits(
            u64::from_str_radix(string(node, "value")?, 16).map_err(|error| error.to_string())?,
        ))),
        "byte-array" => Ok(FieldValue::ByteArray(nbt_tree_values(node, "byte-array")?)),
        "string" => Ok(FieldValue::Str(string(node, "value")?.to_owned())),
        "list" => Ok(FieldValue::List(
            node["value"]
                .as_array()
                .ok_or("list value must be an array")?
                .iter()
                .map(nbt_tree_value)
                .collect::<Result<Vec<_>, _>>()?,
        )),
        "compound" => {
            let object = node["value"]
                .as_object()
                .ok_or("compound value must be an object")?;
            let fields = object
                .iter()
                .map(|(key, value)| {
                    Ok(Field {
                        key: key.clone(),
                        value: nbt_tree_value(value)?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(FieldValue::Compound(fields))
        }
        "int-array" => Ok(FieldValue::IntArray(nbt_tree_values(node, "int-array")?)),
        "long-array" => Ok(FieldValue::LongArray(nbt_tree_values(node, "long-array")?)),
        _ => Err("unknown NBT field type".into()),
    }
}

fn nbt_result(request: &Value) -> Result<Value, String> {
    fn nbt_error(error: nbt::ReadError) -> &'static str {
        match error {
            nbt::ReadError::BufferUnderflow => "buffer_underflow",
            nbt::ReadError::DecoderError => "decoder_error",
            nbt::ReadError::EncoderError => "encoder_error",
            nbt::ReadError::TooBig => "too_big",
        }
    }

    let prefix = decode_hex(string(request, "prefix_hex")?)?;
    match string(request, "operation")? {
        "read" => {
            let framed = decode_hex(string(request, "hex")?)?;
            let mut remaining = framed.as_slice();
            match nbt::read_nbt(&mut remaining) {
                Ok(None) => Ok(json!({
                    "null": true,
                    "outcome": "value",
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
                Ok(Some(fields)) => {
                    let mut out: Vec<Value> = fields.iter().map(nbt_field_json).collect();
                    out.sort_by(|a, b| a["key"].as_str().cmp(&b["key"].as_str()));
                    Ok(json!({
                        "fields": out,
                        "outcome": "value",
                        "consumed": framed.len() - remaining.len(),
                        "remaining_hex": hex(remaining),
                    }))
                }
                Err(error) => Ok(json!({
                    "outcome": "error",
                    "error": nbt_error(error),
                    "consumed": framed.len() - remaining.len(),
                    "remaining_hex": hex(remaining),
                })),
            }
        }
        "write" => {
            let mut buffer = prefix.clone();
            if request.get("null_root").and_then(Value::as_bool) == Some(true) {
                nbt::write_nbt(None, &mut buffer);
            } else if let Some(tree) = request.get("tree") {
                let value = nbt_tree_value(tree)?;
                let FieldValue::Compound(fields) = value else {
                    return Err("NBT tree root must be a compound".into());
                };
                nbt::write_nbt(Some(&fields), &mut buffer);
            } else {
                let fields = request["fields"]
                    .as_array()
                    .ok_or("fields must be an array")?;
                let mut root = Vec::with_capacity(fields.len());
                for entry in fields {
                    let key = string(entry, "key")?.to_owned();
                    let value = match string(entry, "type")? {
                        "byte" => FieldValue::Byte(
                            string(entry, "value")?
                                .parse::<i8>()
                                .map_err(|e| e.to_string())?,
                        ),
                        "short" => FieldValue::Short(
                            string(entry, "value")?
                                .parse::<i16>()
                                .map_err(|e| e.to_string())?,
                        ),
                        "int" => FieldValue::Int(
                            string(entry, "value")?
                                .parse::<i32>()
                                .map_err(|e| e.to_string())?,
                        ),
                        "long" => FieldValue::Long(
                            string(entry, "value")?
                                .parse::<i64>()
                                .map_err(|e| e.to_string())?,
                        ),
                        "float" => FieldValue::Float(f32::from_bits(
                            u32::from_str_radix(string(entry, "value")?, 16)
                                .map_err(|e| e.to_string())?,
                        )),
                        "double" => FieldValue::Double(f64::from_bits(
                            u64::from_str_radix(string(entry, "value")?, 16)
                                .map_err(|e| e.to_string())?,
                        )),
                        "string" => FieldValue::Str(string(entry, "value")?.to_owned()),
                        _ => return Err("unknown NBT field type".into()),
                    };
                    root.push(Field { key, value });
                }
                nbt::write_nbt(Some(&root), &mut buffer);
            }
            Ok(json!({
                "outcome": "value",
                "written": buffer.len() - prefix.len(),
                "hex": hex(&buffer),
            }))
        }
        _ => Err("unknown NBT operation".into()),
    }
}

/// The same deterministic rendering the oracle writes, so the comparison sees
/// shape, order, number literals and string code units.
fn json_canonical(value: &voxel_core::json::JsonValue, out: &mut String) {
    use voxel_core::json::JsonValue;
    let write_string = |units: &[u16], out: &mut String| {
        out.push('s');
        for unit in units {
            out.push_str(&format!("{unit:04x}"));
        }
        out.push(';');
    };
    match value {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(true) => out.push_str("true"),
        JsonValue::Bool(false) => out.push_str("false"),
        JsonValue::Number(literal) => {
            out.push('n');
            out.push_str(literal);
        }
        JsonValue::String(units) => write_string(units, out),
        JsonValue::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                json_canonical(item, out);
            }
            out.push(']');
        }
        JsonValue::Object(object) => {
            out.push('{');
            for (index, (key, item)) in object.entries().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_string(key, out);
                out.push(':');
                json_canonical(item, out);
            }
            out.push('}');
        }
    }
}

fn json_number_result(request: &Value) -> Result<Value, String> {
    use voxel_core::json::JsonValue;
    let rows = request["rows"]
        .as_array()
        .ok_or("json rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let literal = string(row, "literal")?;
        let Ok(value) = voxel_core::json::parse(literal) else {
            out.push(json!({"label": label, "outcome": "rejected"}));
            continue;
        };
        let JsonValue::Number(kept) = &value else {
            out.push(json!({"label": label, "outcome": "rejected"}));
            continue;
        };
        out.push(json!({
            "label": label,
            "outcome": "converted",
            "literal": kept,
            "as_int": value.as_i32().map_or("error".to_owned(), |v| v.to_string()),
            "as_long": value.as_i64().map_or("error".to_owned(), |v| v.to_string()),
            "as_float": value
                .as_f32()
                .map_or("error".to_owned(), |v| format!("{:08x}", v.to_bits())),
            "as_double": value
                .as_f64()
                .map_or("error".to_owned(), |v| format!("{:016x}", v.to_bits())),
        }));
    }
    Ok(json!({"rows": out}))
}

fn json_result(request: &Value) -> Result<Value, String> {
    if string(request, "operation")? == "number" {
        return json_number_result(request);
    }
    if string(request, "operation")? != "parse" {
        return Err("unknown json operation".into());
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("json rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let hex = string(row, "text")?;
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16))
            .collect::<Result<_, _>>()
            .map_err(|error| format!("text hex: {error}"))?;
        let text = match core::str::from_utf8(&bytes) {
            Ok(text) => text,
            Err(_) => {
                out.push(json!({"label": label, "outcome": "rejected"}));
                continue;
            }
        };
        match voxel_core::json::parse(text) {
            Err(_) => out.push(json!({"label": label, "outcome": "rejected"})),
            Ok(value) => {
                let mut canonical = String::new();
                json_canonical(&value, &mut canonical);
                out.push(json!({
                    "label": label,
                    "outcome": "parsed",
                    "canonical": canonical,
                }));
            }
        }
    }
    Ok(json!({"rows": out}))
}

fn model_geometry_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let read = |key: &str| -> Result<[f32; 3], String> {
            let array = row[key].as_array().ok_or("vector must be an array")?;
            let mut value = [0.0_f32; 3];
            for (slot, item) in value.iter_mut().zip(array) {
                let hex = item.as_str().ok_or("vector element must be a string")?;
                *slot = f32::from_bits(
                    u32::from_str_radix(hex, 16).map_err(|error| format!("vector: {error}"))?,
                );
            }
            Ok(value)
        };
        let from = read("from")?;
        let to = read("to")?;
        let Some(facing) = direction_by_name(string(row, "facing")?) else {
            out.push(json!({"label": label, "outcome": "unknown_facing"}));
            continue;
        };
        let uvs = voxel_render::default_face_uv(from, to, facing);
        let corners: Vec<Value> = voxel_render::face_vertices(from, to, facing)
            .iter()
            .map(|vertex| {
                json!({
                    "x": format!("{:08x}", vertex[0].to_bits()),
                    "y": format!("{:08x}", vertex[1].to_bits()),
                    "z": format!("{:08x}", vertex[2].to_bits()),
                })
            })
            .collect();
        out.push(json!({
            "label": label,
            "outcome": "baked",
            "min_u": format!("{:08x}", uvs.min_u.to_bits()),
            "min_v": format!("{:08x}", uvs.min_v.to_bits()),
            "max_u": format!("{:08x}", uvs.max_u.to_bits()),
            "max_v": format!("{:08x}", uvs.max_v.to_bits()),
            "corners": corners,
        }));
    }
    Ok(json!({"rows": out}))
}

fn direction_by_name(name: &str) -> Option<voxel_core::direction::Direction> {
    use voxel_core::direction::Direction;
    Some(match name {
        "down" => Direction::Down,
        "up" => Direction::Up,
        "north" => Direction::North,
        "south" => Direction::South,
        "west" => Direction::West,
        "east" => Direction::East,
        _ => return None,
    })
}

fn model_rotation_result(request: &Value) -> Result<Value, String> {
    use voxel_render::rotation;
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let read = |key: &str| -> Result<[f32; 3], String> {
            let array = row[key].as_array().ok_or("vector must be an array")?;
            let mut value = [0.0_f32; 3];
            for (slot, item) in value.iter_mut().zip(array) {
                let hex = item.as_str().ok_or("vector element must be a string")?;
                *slot = f32::from_bits(
                    u32::from_str_radix(hex, 16).map_err(|error| format!("vector: {error}"))?,
                );
            }
            Ok(value)
        };
        let from = read("from")?;
        let to = read("to")?;
        let mut origin = read("origin")?;
        // The deserializer scales the origin into block units before use.
        for axis in origin.iter_mut() {
            *axis *= 0.0625;
        }
        let angle = f32::from_bits(
            u32::from_str_radix(string(row, "angle")?, 16)
                .map_err(|error| format!("angle: {error}"))?,
        );
        let axis = match string(row, "axis")? {
            "x" => rotation::RotationAxis::X,
            "y" => rotation::RotationAxis::Y,
            "z" => rotation::RotationAxis::Z,
            _ => {
                out.push(json!({"label": label, "outcome": "unknown_input"}));
                continue;
            }
        };
        let Some(facing) = direction_by_name(string(row, "facing")?) else {
            out.push(json!({"label": label, "outcome": "unknown_input"}));
            continue;
        };
        let rescale = row["rescale"].as_bool().ok_or("rescale must be a bool")?;

        let transform = rotation::element_transform(axis, angle, rescale);
        let mut matrix = Vec::with_capacity(16);
        for column in &transform {
            for value in column {
                matrix.push(json!(format!("{:08x}", value.to_bits())));
            }
        }
        let corners: Vec<Value> = voxel_render::face_vertices(from, to, facing)
            .iter()
            .map(|vertex| {
                let rotated = rotation::rotate_vertex(*vertex, origin, &transform);
                json!({
                    "x": format!("{:08x}", rotated[0].to_bits()),
                    "y": format!("{:08x}", rotated[1].to_bits()),
                    "z": format!("{:08x}", rotated[2].to_bits()),
                })
            })
            .collect();
        out.push(json!({
            "label": label,
            "outcome": "rotated",
            "transform": matrix,
            "corners": corners,
        }));
    }
    Ok(json!({"rows": out}))
}

fn model_winding_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let given = row["corners"]
            .as_array()
            .ok_or("corners must be an array")?;
        let mut positions = [[0.0_f32; 3]; 4];
        for (slot, item) in positions.iter_mut().zip(given) {
            let array = item.as_array().ok_or("corner must be an array")?;
            for (axis, value) in slot.iter_mut().zip(array) {
                let hex = value.as_str().ok_or("corner element must be a string")?;
                *axis = f32::from_bits(
                    u32::from_str_radix(hex, 16).map_err(|error| format!("corner: {error}"))?,
                );
            }
        }

        let normal = voxel_render::triangle_normal(positions[0], positions[1], positions[2]);
        let mut entry = serde_json::Map::new();
        entry.insert("label".into(), json!(label));
        entry.insert("outcome".into(), json!("wound"));
        entry.insert(
            "normal_x".into(),
            json!(format!("{:08x}", normal[0].to_bits())),
        );
        entry.insert(
            "normal_y".into(),
            json!(format!("{:08x}", normal[1].to_bits())),
        );
        entry.insert(
            "normal_z".into(),
            json!(format!("{:08x}", normal[2].to_bits())),
        );

        match voxel_render::closest_direction(normal) {
            None => {
                entry.insert("direction".into(), json!("none"));
            }
            Some(direction) => {
                entry.insert("direction".into(), json!(direction_name(direction)));
                let mut payloads = [10_u64, 20, 30, 40];
                match voxel_render::recalculate_winding(&mut positions, &mut payloads, direction) {
                    Err(_) => {
                        entry.insert("winding".into(), json!("no_matching_vertex"));
                    }
                    Ok(()) => {
                        let wound: Vec<Value> = positions
                            .iter()
                            .map(|vertex| {
                                json!({
                                    "x": format!("{:08x}", vertex[0].to_bits()),
                                    "y": format!("{:08x}", vertex[1].to_bits()),
                                    "z": format!("{:08x}", vertex[2].to_bits()),
                                })
                            })
                            .collect();
                        entry.insert("wound".into(), json!(wound));
                        entry.insert("uv_order".into(), json!(payloads.to_vec()));
                    }
                }
            }
        }
        out.push(Value::Object(entry));
    }
    Ok(json!({"rows": out}))
}

fn direction_name(direction: voxel_core::direction::Direction) -> &'static str {
    use voxel_core::direction::Direction;
    match direction {
        Direction::Down => "down",
        Direction::Up => "up",
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
    }
}

fn model_euler_result(request: &Value) -> Result<Value, String> {
    use voxel_render::rotation;
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let read = |key: &str| -> Result<f32, String> {
            Ok(f32::from_bits(
                u32::from_str_radix(string(row, key)?, 16)
                    .map_err(|error| format!("{key}: {error}"))?,
            ))
        };
        let transform = rotation::euler_element_transform(
            read("x")?,
            read("y")?,
            read("z")?,
            row["rescale"].as_bool().ok_or("rescale must be a bool")?,
        );
        let mut matrix = Vec::with_capacity(16);
        for column in &transform {
            for value in column {
                matrix.push(json!(format!("{:08x}", value.to_bits())));
            }
        }
        out.push(json!({
            "label": string(row, "label")?,
            "outcome": "rotated",
            "transform": matrix,
        }));
    }
    Ok(json!({"rows": out}))
}

fn model_uv_rotation_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let degrees = i32_field(row, "degrees")?;
        let Some(rotation) = voxel_render::Quadrant::from_degrees(degrees) else {
            out.push(json!({"label": label, "outcome": "rejected"}));
            continue;
        };
        let values = row["uvs"].as_array().ok_or("uvs must be an array")?;
        let mut parsed = [0.0_f32; 4];
        for (slot, item) in parsed.iter_mut().zip(values) {
            let hex = item.as_str().ok_or("uv element must be a string")?;
            *slot = f32::from_bits(
                u32::from_str_radix(hex, 16).map_err(|error| format!("uv: {error}"))?,
            );
        }
        let uvs = voxel_render::FaceUvs {
            min_u: parsed[0],
            min_v: parsed[1],
            max_u: parsed[2],
            max_v: parsed[3],
        };
        let pairs: Vec<Value> = (0..4)
            .map(|vertex| {
                let (u, v) = voxel_render::face_vertex_uv(uvs, rotation, vertex);
                json!({
                    "u": format!("{:08x}", u.to_bits()),
                    "v": format!("{:08x}", v.to_bits()),
                })
            })
            .collect();
        out.push(json!({
            "label": label,
            "outcome": "rotated",
            "shift": rotation.shift(),
            "pairs": pairs,
        }));
    }
    Ok(json!({"rows": out}))
}

fn model_parse_result(request: &Value) -> Result<Value, String> {
    use voxel_render::model;
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let hex = string(row, "text")?;
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16))
            .collect::<Result<_, _>>()
            .map_err(|error| format!("text hex: {error}"))?;
        let Ok(text) = core::str::from_utf8(&bytes) else {
            out.push(json!({"label": label, "outcome": "rejected"}));
            continue;
        };
        let parsed = match model::parse_model(text) {
            Err(_) => {
                out.push(json!({"label": label, "outcome": "rejected"}));
                continue;
            }
            Ok(parsed) => parsed,
        };

        let slots: Vec<Value> = parsed
            .texture_slots
            .entries()
            .map(|(slot, contents)| match contents {
                model::SlotContents::Reference(target) => json!({
                    "slot": slot, "kind": "Reference", "target": target,
                }),
                model::SlotContents::Value(material) => json!({
                    "slot": slot,
                    "kind": "Value",
                    "sprite": material.sprite.as_string(),
                    "force_translucent": material.force_translucent,
                }),
            })
            .collect();

        let elements: Vec<Value> = parsed
            .elements
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(|element| {
                let mut faces: Vec<&(voxel_core::direction::Direction, model::ModelFace)> =
                    element.faces.iter().collect();
                // The oracle reads faces out of a map keyed by direction name.
                faces.sort_by_key(|(direction, _)| direction_name(*direction));
                let described_faces: Vec<Value> = faces
                    .iter()
                    .map(|(direction, face)| {
                        json!({
                            "direction": direction_name(*direction),
                            "cullface": face
                                .cull_for_direction
                                .map_or("none", direction_name),
                            "tint_index": face.tint_index,
                            "texture": face.texture,
                            "uv_rotation": face.rotation.shift(),
                            "uvs": match face.uvs {
                                None => json!("none"),
                                Some(uvs) => json!([
                                    format!("{:08x}", uvs.min_u.to_bits()),
                                    format!("{:08x}", uvs.min_v.to_bits()),
                                    format!("{:08x}", uvs.max_u.to_bits()),
                                    format!("{:08x}", uvs.max_v.to_bits()),
                                ]),
                            },
                        })
                    })
                    .collect();
                let bits = |vector: [f32; 3]| {
                    json!(
                        vector
                            .iter()
                            .map(|axis| format!("{:08x}", axis.to_bits()))
                            .collect::<Vec<_>>()
                    )
                };
                json!({
                    "from": bits(element.from),
                    "to": bits(element.to),
                    "shade": element.shade,
                    "light_emission": element.light_emission,
                    "rotation": match &element.rotation {
                        None => json!("none"),
                        Some(rotation) => {
                            let mut matrix = Vec::with_capacity(16);
                            for column in &rotation.transform {
                                for value in column {
                                    matrix.push(json!(format!("{:08x}", value.to_bits())));
                                }
                            }
                            json!({
                                "origin": bits(rotation.origin),
                                "rescale": rotation.rescale,
                                "transform": matrix,
                            })
                        }
                    },
                    "faces": described_faces,
                })
            })
            .collect();

        let mut display = serde_json::Map::new();
        if let Some(transforms) = &parsed.display {
            let bits = |vector: [f32; 3]| {
                json!(
                    vector
                        .iter()
                        .map(|axis| format!("{:08x}", axis.to_bits()))
                        .collect::<Vec<_>>()
                )
            };
            for context in model::DisplayContext::ALL {
                if let Some(transform) = transforms.get(context) {
                    display.insert(
                        context.name().to_owned(),
                        json!({
                            "rotation": bits(transform.rotation),
                            "translation": bits(transform.translation),
                            "scale": bits(transform.scale),
                        }),
                    );
                }
            }
        }
        out.push(json!({
            "label": label,
            "outcome": "parsed",
            "parent": parsed
                .parent
                .as_ref()
                .map_or("none".to_owned(), |name| name.as_string()),
            "ambient_occlusion": parsed
                .ambient_occlusion
                .map_or("unset".to_owned(), |value| value.to_string()),
            "gui_light": match parsed.gui_light {
                None => "unset",
                Some(model::GuiLight::Front) => "FRONT",
                Some(model::GuiLight::Side) => "SIDE",
            },
            "texture_slots": slots,
            "has_geometry": parsed.elements.is_some(),
            "elements": elements,
            "display": display,
        }));
    }
    Ok(json!({"rows": out}))
}

fn model_state_result(request: &Value) -> Result<Value, String> {
    use voxel_render::rotation;
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let Some(transform) = rotation::model_state_transform(
            i32_field(row, "x")?,
            i32_field(row, "y")?,
            i32_field(row, "z")?,
        ) else {
            out.push(json!({"label": label, "outcome": "rejected"}));
            continue;
        };
        let mut matrix = Vec::with_capacity(16);
        for column in &transform {
            for value in column {
                matrix.push(json!(format!("{:08x}", value.to_bits())));
            }
        }
        let uv_lock = row["uvlock"].as_bool().ok_or("uvlock must be a boolean")?;
        let (x, y, z) = (
            i32_field(row, "x")?,
            i32_field(row, "y")?,
            i32_field(row, "z")?,
        );
        let flat = |matrix: rotation::FlatMatrix| -> Vec<Value> {
            matrix
                .iter()
                .map(|value| json!(format!("{:08x}", value.to_bits())))
                .collect()
        };
        let mut faces = Vec::with_capacity(6);
        for facing in [
            voxel_core::direction::Direction::Down,
            voxel_core::direction::Direction::Up,
            voxel_core::direction::Direction::North,
            voxel_core::direction::Direction::South,
            voxel_core::direction::Direction::West,
            voxel_core::direction::Direction::East,
        ] {
            let forward = rotation::face_transform(x, y, z, facing, uv_lock)
                .ok_or("no face transform for that rotation")?;
            let inverse = rotation::inverse_face_transform(x, y, z, facing, uv_lock)
                .ok_or("no inverse face transform for that rotation")?;
            faces.push(json!({
                "face": facing.name(),
                "forward": flat(forward),
                "inverse": flat(inverse),
            }));
        }
        out.push(json!({
            "label": label,
            "outcome": "resolved",
            "transformation": matrix,
            "faces": faces,
        }));
    }
    Ok(json!({"rows": out}))
}

fn model_uv_lock_result(request: &Value) -> Result<Value, String> {
    use voxel_core::direction::Direction;
    use voxel_render::rotation;
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let facing = Direction::from_name(string(row, "facing")?).ok_or("unknown facing")?;
        let matrix = rotation::inverse_face_transform(
            i32_field(row, "x")?,
            i32_field(row, "y")?,
            i32_field(row, "z")?,
            facing,
            row["uvlock"].as_bool().ok_or("uvlock must be a boolean")?,
        )
        .ok_or("no face transform for that rotation")?;
        let read = |name: &str| -> Result<f32, String> {
            Ok(f32::from_bits(
                u32::from_str_radix(string(row, name)?, 16)
                    .map_err(|error| format!("{name}: {error}"))?,
            ))
        };
        let (u, v) = rotation::transform_face_uv(&matrix, read("u")?, read("v")?);
        out.push(json!({
            "label": string(row, "label")?,
            "outcome": "transformed",
            "identity": rotation::is_flat_identity(&matrix),
            "u": format!("{:08x}", u.to_bits()),
            "v": format!("{:08x}", v.to_bits()),
        }));
    }
    Ok(json!({"rows": out}))
}

fn describe_variants(variants: &[voxel_render::Variant]) -> Value {
    Value::Array(
        variants
            .iter()
            .map(|variant| {
                json!({
                    "model": variant.model.as_string(),
                    "x": variant.x.name(),
                    "y": variant.y.name(),
                    "z": variant.z.name(),
                    "uvlock": variant.uvlock,
                    "weight": variant.weight,
                })
            })
            .collect::<Vec<_>>(),
    )
}

fn describe_condition(condition: &voxel_render::Condition) -> Value {
    match condition {
        voxel_render::Condition::KeyValue(tests) => {
            let mut described = serde_json::Map::new();
            for (property, terms) in tests {
                described.insert(
                    property.clone(),
                    Value::Array(
                        terms
                            .iter()
                            .map(|term| json!({"value": term.value, "negated": term.negated}))
                            .collect(),
                    ),
                );
            }
            json!({"kind": "key_value", "tests": Value::Object(described)})
        }
        voxel_render::Condition::Combined { operation, terms } => json!({
            "kind": "combined",
            "operation": operation.name(),
            "terms": terms.iter().map(describe_condition).collect::<Vec<_>>(),
        }),
    }
}

fn model_blockstate_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        match voxel_render::parse_block_state(string(row, "json")?) {
            None => out.push(json!({"label": label, "outcome": "rejected"})),
            Some(file) => {
                let mut described = serde_json::Map::new();
                described.insert("label".into(), json!(label));
                described.insert("outcome".into(), json!("parsed"));
                if let Some(variants) = &file.variants {
                    let mut listed = serde_json::Map::new();
                    for (key, value) in variants {
                        listed.insert(key.clone(), describe_variants(value));
                    }
                    described.insert("variants".into(), Value::Object(listed));
                }
                if let Some(multipart) = &file.multipart {
                    described.insert(
                        "multipart".into(),
                        Value::Array(
                            multipart
                                .iter()
                                .map(|selector| {
                                    let mut piece = serde_json::Map::new();
                                    if let Some(condition) = &selector.condition {
                                        piece.insert("when".into(), describe_condition(condition));
                                    }
                                    piece
                                        .insert("apply".into(), describe_variants(&selector.apply));
                                    Value::Object(piece)
                                })
                                .collect(),
                        ),
                    );
                }
                out.push(Value::Object(described));
            }
        }
    }
    Ok(json!({"rows": out}))
}

fn model_result(request: &Value) -> Result<Value, String> {
    if string(request, "operation")? == "nearest_direction" {
        let rows = request["rows"]
            .as_array()
            .ok_or("model rows must be an array")?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let read = |name: &str| -> Result<f32, String> {
                Ok(f32::from_bits(
                    u32::from_str_radix(string(row, name)?, 16)
                        .map_err(|error| format!("{name}: {error}"))?,
                ))
            };
            let vector = [read("x")?, read("y")?, read("z")?];
            out.push(json!({
                "label": string(row, "label")?,
                "outcome": "resolved",
                "direction": voxel_render::approximate_nearest_direction(vector).name(),
            }));
        }
        return Ok(json!({"rows": out}));
    }
    if string(request, "operation")? == "cull_rotation" {
        use voxel_core::direction::Direction;
        use voxel_render::rotation;
        let rows = request["rows"]
            .as_array()
            .ok_or("model rows must be an array")?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let facing = Direction::from_name(string(row, "facing")?).ok_or("unknown facing")?;
            let rotated = rotation::rotate_cull_direction(
                i32_field(row, "x")?,
                i32_field(row, "y")?,
                i32_field(row, "z")?,
                facing,
            )
            .ok_or("rotation must be a multiple of ninety")?;
            out.push(json!({
                "label": string(row, "label")?,
                "outcome": "resolved",
                "direction": rotated.name(),
            }));
        }
        return Ok(json!({"rows": out}));
    }
    if string(request, "operation")? == "uv_lock" {
        return model_uv_lock_result(request);
    }
    if string(request, "operation")? == "blockstate_parse" {
        return model_blockstate_result(request);
    }
    if string(request, "operation")? == "model_state" {
        return model_state_result(request);
    }
    if string(request, "operation")? == "parse_model" {
        return model_parse_result(request);
    }
    if string(request, "operation")? == "face_uv_rotation" {
        return model_uv_rotation_result(request);
    }
    if string(request, "operation")? == "euler_rotation" {
        return model_euler_result(request);
    }
    if string(request, "operation")? == "quad_winding" {
        return model_winding_result(request);
    }
    if string(request, "operation")? == "element_rotation" {
        return model_rotation_result(request);
    }
    if string(request, "operation")? == "face_geometry" {
        return model_geometry_result(request);
    }
    if string(request, "operation")? != "texture_slots" {
        return Err("unknown model operation".into());
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("model rows must be an array")?;
    let probes = [
        "all", "top", "bottom", "side", "north", "particle", "missing", "#all", "#top",
    ];
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut resolver = voxel_render::TextureSlotsResolver::new();
        let chain = row["chain"].as_array().ok_or("chain must be an array")?;
        for level in chain {
            let slots = level.as_array().ok_or("chain level must be an array")?;
            let mut entries = Vec::with_capacity(slots.len());
            for slot in slots {
                let name = string(slot, "slot")?;
                let contents = if slot.get("raw").is_some() {
                    voxel_render::SlotContents::parse_string(string(slot, "raw")?)
                        .map_err(|error| format!("texture value: {error:?}"))?
                } else {
                    voxel_render::SlotContents::Value(voxel_render::Material {
                        sprite: Identifier::parse(string(slot, "sprite")?)
                            .map_err(|error| format!("sprite: {error:?}"))?,
                        force_translucent: slot["force_translucent"]
                            .as_bool()
                            .ok_or("force_translucent must be a bool")?,
                    })
                };
                entries.push((name.to_owned(), contents));
            }
            resolver.add_last(voxel_render::TextureSlotsData::from_entries(entries));
        }
        let resolved = resolver.resolve();
        let mut lookups = Vec::new();
        for probe in probes {
            if let Some(material) = resolved.material(probe) {
                lookups.push(json!({
                    "probe": probe,
                    "sprite": material.sprite.as_string(),
                    "force_translucent": material.force_translucent,
                }));
            }
        }
        lookups.sort_by(|left, right| left["probe"].as_str().cmp(&right["probe"].as_str()));
        out.push(json!({
            "label": string(row, "label")?,
            "outcome": "resolved",
            "lookups": lookups,
        }));
    }
    Ok(json!({"rows": out}))
}

fn image_tables_result(request: &Value) -> Result<Value, String> {
    let tables = voxel_render::SrgbTables::new();
    let mut forward = String::with_capacity(256 * 4);
    for channel in 0..=255_u8 {
        forward.push_str(&format!("{:04x}", tables.to_linear(channel)));
    }
    let mut backward = String::with_capacity(1024 * 2);
    for step in 0..1024_u16 {
        backward.push_str(&format!("{:02x}", tables.to_srgb(step)));
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let label = string(&rows[0], "label")?;
    Ok(json!({"rows": [{
        "label": label,
        "outcome": "listed",
        "srgb_to_linear": forward,
        "linear_to_srgb": backward,
    }]}))
}

fn image_mip_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let hex = string(row, "png")?;
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16))
            .collect::<Result<_, _>>()
            .map_err(|error| format!("png hex: {error}"))?;
        let levels = i32_field(row, "levels")?;
        let Ok(image) = voxel_render::decode_rgba(&bytes) else {
            out.push(json!({"label": label, "outcome": "failed"}));
            continue;
        };
        let name = string(row, "name")?;
        let path = name.split_once(':').map_or(name, |(_, path)| path);
        let strategy =
            voxel_render::MipmapStrategy::parse(&string(row, "strategy")?.to_lowercase())
                .ok_or("unknown mipmap strategy")?;
        let bias = f32::from_bits(
            u32::from_str_radix(string(row, "bias")?, 16)
                .map_err(|error| format!("mipmap bias: {error}"))?,
        );
        let transparency = voxel_render::Transparency::of(
            row["transparent"]
                .as_bool()
                .ok_or("transparent must be a boolean")?,
            row["translucent"]
                .as_bool()
                .ok_or("translucent must be a boolean")?,
        );
        let generated = voxel_render::generate_mip_levels_with(
            path,
            &image,
            levels.max(0) as u32,
            strategy,
            bias,
            transparency,
        );
        let described: Vec<Value> = generated
            .iter()
            .map(|level| {
                let mut encoded = String::with_capacity(level.rgba.len() * 2);
                for byte in &level.rgba {
                    encoded.push_str(&format!("{byte:02x}"));
                }
                json!({"width": level.width, "height": level.height, "rgba": encoded})
            })
            .collect();
        out.push(json!({
            "label": label,
            "outcome": "generated",
            "levels": described,
        }));
    }
    Ok(json!({"rows": out}))
}

fn image_transparency_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let bytes = hex_bytes(string(row, "png")?)?;
        match voxel_render::decode_rgba(&bytes) {
            Err(_) => out.push(json!({"label": label, "outcome": "failed"})),
            Ok(image) => {
                let measured = voxel_render::compute_transparency(&image);
                out.push(json!({
                    "label": label,
                    "outcome": "measured",
                    "transparent": measured.has_transparent,
                    "translucent": measured.has_translucent,
                    "opaque": measured.is_opaque(),
                }));
            }
        }
    }
    Ok(json!({"rows": out}))
}

fn image_texture_metadata_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let parsed =
            voxel_core::json::parse(string(row, "json")?)
                .ok()
                .and_then(|value| match value {
                    voxel_core::json::JsonValue::Object(object) => {
                        voxel_render::TextureMetadata::parse(&object)
                    }
                    _ => None,
                });
        match parsed {
            None => out.push(json!({"label": label, "outcome": "rejected"})),
            Some(metadata) => out.push(json!({
                "label": label,
                "outcome": "parsed",
                "blur": metadata.blur,
                "clamp": metadata.clamp,
                "strategy": metadata.mipmap_strategy.name(),
                "bias": format!("{:08x}", metadata.alpha_cutoff_bias.to_bits()),
            })),
        }
    }
    Ok(json!({"rows": out}))
}

fn image_sprite_uv_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let width = i32_field(row, "width")?;
        let height = i32_field(row, "height")?;
        let placement = voxel_render::SpritePlacement {
            name: voxel_core::identifier::Identifier::parse("minecraft:block/parity")
                .map_err(|_| "sprite name")?,
            x: i32_field(row, "x")?,
            y: i32_field(row, "y")?,
            padding: i32_field(row, "padding")?,
        };
        let sprite = voxel_render::AtlasSprite::new(
            &placement,
            width,
            height,
            i32_field(row, "atlas_width")?,
            i32_field(row, "atlas_height")?,
        );
        let mut interpolated = Vec::new();
        for offset in row["offsets"]
            .as_array()
            .ok_or("offsets must be an array")?
        {
            let bits = u32::from_str_radix(offset.as_str().ok_or("offset must be a string")?, 16)
                .map_err(|error| format!("offset: {error}"))?;
            let offset = f32::from_bits(bits);
            interpolated.push(json!({
                "u": format!("{:08x}", sprite.u(offset).to_bits()),
                "v": format!("{:08x}", sprite.v(offset).to_bits()),
            }));
        }
        out.push(json!({
            "label": label,
            "outcome": "placed",
            "x": sprite.x,
            "y": sprite.y,
            "u0": format!("{:08x}", sprite.u0.to_bits()),
            "u1": format!("{:08x}", sprite.u1.to_bits()),
            "v0": format!("{:08x}", sprite.v0.to_bits()),
            "v1": format!("{:08x}", sprite.v1.to_bits()),
            "interpolated": interpolated,
        }));
    }
    Ok(json!({"rows": out}))
}

fn image_colour_map_result(request: &Value) -> Result<Value, String> {
    let bytes = hex_bytes(string(request, "colour_map")?)?;
    let map = voxel_render::decode_rgba(&bytes).map_err(|_| "colour map did not decode")?;
    // The reference reads the map as packed colours, not as bytes.
    let pixels: Vec<u32> = map
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|texel| {
            (u32::from(texel[3]) << 24)
                | (u32::from(texel[0]) << 16)
                | (u32::from(texel[1]) << 8)
                | u32::from(texel[2])
        })
        .collect();
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let kind = match string(row, "kind")? {
            "grass" => voxel_render::ColourMap::Grass,
            "foliage" => voxel_render::ColourMap::Foliage,
            "dry_foliage" => voxel_render::ColourMap::DryFoliage,
            other => return Err(format!("unknown colour map kind: {other}")),
        };
        let read = |name: &str| -> Result<f64, String> {
            let bits = u64::from_str_radix(string(row, name)?, 16)
                .map_err(|error| format!("{name}: {error}"))?;
            Ok(f64::from_bits(bits))
        };
        match voxel_render::colour_map_lookup(kind, &pixels, read("temperature")?, read("rain")?) {
            Err(_) => out.push(json!({"label": label, "outcome": "out_of_bounds"})),
            Ok(colour) => out.push(json!({
                "label": label,
                "outcome": "resolved",
                "colour": format!("{colour:08x}"),
            })),
        }
    }
    Ok(json!({"rows": out}))
}

fn image_argb_multiply_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let read = |name: &str| -> Result<u32, String> {
            u32::from_str_radix(string(row, name)?, 16).map_err(|error| format!("{name}: {error}"))
        };
        out.push(json!({
            "label": string(row, "label")?,
            "outcome": "multiplied",
            "colour": format!("{:08x}", voxel_render::multiply(read("lhs")?, read("rhs")?)),
        }));
    }
    Ok(json!({"rows": out}))
}

fn image_tint_constants_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(json!({
            "label": string(row, "label")?,
            "outcome": "listed",
            "values": {
                "foliage_default": format!("{:08x}", voxel_render::ColourMap::Foliage.default_colour()),
                "foliage_evergreen": format!("{:08x}", voxel_render::constant_tints("minecraft:spruce_leaves")[0]),
                "foliage_birch": format!("{:08x}", voxel_render::constant_tints("minecraft:birch_leaves")[0]),
                "foliage_mangrove": format!("{:08x}", voxel_render::FOLIAGE_MANGROVE),
                "dry_foliage_default": format!("{:08x}", voxel_render::ColourMap::DryFoliage.default_colour()),
                "lily_pad_in_hand": format!("{:08x}", voxel_render::constant_tints("minecraft:lily_pad")[0]),
                "lily_pad_in_world": format!("{:08x}", voxel_render::LILY_PAD_IN_WORLD),
            },
        }));
    }
    Ok(json!({"rows": out}))
}

/// `block_declarations`: the declarations as the oracle describes them.
fn block_declarations(label: &str) -> Value {
    use voxel_data::block::classes::BlockClass;
    use voxel_data::block::definition::{Emission, Fluid, PushReaction, Rule};
    use voxel_data::block::properties::FIELDS;
    use voxel_data::block::registry::{self, PropertyKind};
    let bits = |value: f32| format!("{:08x}", value.to_bits());
    let classes: Vec<Value> = BlockClass::ALL
        .iter()
        .map(|class| {
            let parent = class
                .parent()
                .map_or(String::new(), |parent| format!("{parent:?}"));
            let interfaces: Vec<String> = class
                .interfaces()
                .iter()
                .map(|interface| format!("{interface:?}"))
                .collect();
            let declares: Vec<String> = class
                .declares()
                .iter()
                .map(|method| {
                    let name = format!("{method:?}");
                    name[..1].to_lowercase() + &name[1..]
                })
                .collect();
            json!({
                "name": format!("{class:?}"),
                "parent": parent,
                "interfaces": interfaces,
                "declares": declares,
            })
        })
        .collect();
    let interfaces: Vec<Value> = voxel_data::block::classes::BlockInterface::ALL
        .iter()
        .map(|interface| {
            let parents: Vec<String> = interface
                .parents()
                .iter()
                .map(|parent| format!("{parent:?}"))
                .collect();
            json!({"name": format!("{interface:?}"), "parents": parents})
        })
        .collect();
    let properties: Vec<Value> = FIELDS
        .iter()
        .map(|(field, property)| {
            json!({
                "field": field,
                "name": property.name,
                "kind": match property.kind {
                    PropertyKind::Boolean => "boolean",
                    PropertyKind::Integer => "integer",
                    PropertyKind::Enum => "enum",
                },
                "values": property.values,
            })
        })
        .collect();
    let field = |property: &registry::Property| {
        FIELDS
            .iter()
            .find(|(_, listed)| std::ptr::eq(*listed, property))
            .map(|(field, _)| *field)
            .expect("every property is listed")
    };
    let blocks: Vec<Value> = voxel_data::block::registry::BLOCKS
        .iter()
        .map(|block| {
            let count = block.state_count();
            let mut per_state = serde_json::Map::new();
            if block.light_emission != Emission::Constant(0) {
                let levels: Vec<u8> = (0..count).map(|offset| block.light_emission.value(offset)).collect();
                per_state.insert("lightEmission".into(), json!(levels));
            }
            for (key, rule) in [
                ("emissiveRendering", block.emissive_rendering),
                ("isSuffocating", block.suffocating),
                ("isViewBlocking", block.view_blocking),
                ("isRedstoneConductor", block.redstone_conductor),
            ] {
                if rule != Rule::Default {
                    let values: Vec<bool> = (0..count)
                        .map(|offset| rule.value(offset).expect("not the default"))
                        .collect();
                    per_state.insert(key.into(), json!(values));
                }
            }
            let mut described = json!({
                "name": block.name,
                "class": format!("{:?}", block.class),
                "properties": block.properties.iter().map(|property| field(property)).collect::<Vec<_>>(),
                "default": block.default,
                "declared": {
                    "destroyTime": bits(block.destroy_time),
                    "explosionResistance": bits(block.explosion_resistance),
                    "friction": bits(block.friction),
                    "speedFactor": bits(block.speed_factor),
                    "jumpFactor": bits(block.jump_factor),
                    "bounceRestitution": bits(block.bounce_restitution),
                    "hasCollision": block.has_collision,
                    "requiresCorrectToolForDrops": block.requires_correct_tool_for_drops,
                    "isRandomlyTicking": block.randomly_ticking,
                    "canOcclude": block.can_occlude,
                    "isAir": block.air,
                    "ignitedByLava": block.ignited_by_lava,
                    "liquid": block.liquid,
                    "forceSolidOff": block.force_solid_off,
                    "forceSolidOn": block.force_solid_on,
                    "spawnTerrainParticles": block.spawn_terrain_particles,
                    "replaceable": block.replaceable,
                    "dynamicShape": block.dynamic_shape,
                    "offsetType": match block.offset_type {
                    voxel_data::block::definition::OffsetType::None => "NONE",
                    voxel_data::block::definition::OffsetType::Xz => "XZ",
                    voxel_data::block::definition::OffsetType::Xyz => "XYZ",
                },
                "pushReaction": match block.push_reaction {
                        PushReaction::Normal => "NORMAL",
                        PushReaction::Destroy => "DESTROY",
                        PushReaction::Block => "BLOCK",
                        PushReaction::Ignore => "IGNORE",
                        PushReaction::PushOnly => "PUSH_ONLY",
                    },
                },
                "per_state": per_state,
                "item": block.item_name(),
            });
            if block.class.is(BlockClass::LiquidBlock) {
                described["fluid"] = json!(match block.fluid {
                    Fluid::Water => "minecraft:water",
                    Fluid::FlowingWater => "minecraft:flowing_water",
                    Fluid::Lava => "minecraft:lava",
                    Fluid::FlowingLava => "minecraft:flowing_lava",
                    Fluid::Empty => "minecraft:empty",
                });
            }
            described
        })
        .collect();
    json!({
        "label": label,
        "outcome": "listed",
        "classes": classes,
        "interfaces": interfaces,
        "properties": properties,
        "blocks": blocks,
    })
}

/// The vanilla data pack's block tags, which is what the oracle's registry
/// holds.
fn vanilla_tags() -> Result<voxel_world::block::tags::BlockTags, String> {
    voxel_world::block::tags::BlockTags::from_registries(&vanilla_block_tags()?.registries)
        .ok_or_else(|| "the vanilla tags have no block registry".to_owned())
}

/// `face_occlusion`: the state cache's faces and skip rules, numbered and
/// grouped the way the oracle groups them.
fn face_occlusion(label: &str) -> Result<Value, String> {
    use voxel_data::block::registry;
    use voxel_world::block::culling::skip_rendering;
    use voxel_world::block::state::{DIRECTIONS, states};
    let tags = vanilla_tags()?;
    let states = states();
    let mut tuples: Vec<[u16; 6]> = Vec::new();
    let mut blocks = Vec::with_capacity(registry::BLOCKS.len());
    for block in &registry::BLOCKS {
        let first = block.first_state_id();
        let per_state: Vec<usize> = (0..block.state_count())
            .map(|offset| {
                let faces = states.get(first + offset).expect("a registry state").faces;
                tuples
                    .iter()
                    .position(|tuple| *tuple == faces)
                    .unwrap_or_else(|| {
                        tuples.push(faces);
                        tuples.len() - 1
                    })
            })
            .collect();
        let listed = if per_state.iter().all(|tuple| *tuple == per_state[0]) {
            vec![per_state[0]]
        } else {
            per_state
        };
        blocks.push(json!({"name": block.name, "tuples": listed}));
    }
    let count = states.faces().len();
    let matrix = |test: &dyn Fn(u16, u16) -> bool| -> Vec<String> {
        (0..count as u16)
            .map(|shape| {
                (0..count as u16)
                    .map(|occluder| if test(shape, occluder) { '1' } else { '0' })
                    .collect()
            })
            .collect()
    };
    let joins = matrix(&|shape, occluder| states.joins(shape, occluder));
    let covers = matrix(&|shape, occluder| states.covers(shape, occluder));
    let info = |block: &registry::Block, offset: u32| {
        states
            .get(block.first_state_id() + offset)
            .expect("a registry state")
    };
    let mut skips = Vec::new();
    for block in &registry::BLOCKS {
        let own = info(block, block.default_state_id() - block.first_state_id());
        for other in &registry::BLOCKS {
            let neighbour = info(other, other.default_state_id() - other.first_state_id());
            if !DIRECTIONS
                .iter()
                .any(|direction| skip_rendering(own, neighbour, *direction, &tags))
            {
                continue;
            }
            let mut all = true;
            let mut directions = std::collections::BTreeSet::new();
            for offset in 0..block.state_count() {
                for other_offset in 0..other.state_count() {
                    for direction in DIRECTIONS {
                        if skip_rendering(
                            info(block, offset),
                            info(other, other_offset),
                            direction,
                            &tags,
                        ) {
                            directions.insert(direction.name());
                        } else {
                            all = false;
                        }
                    }
                }
            }
            skips.push(json!({
                "block": block.name,
                "neighbour": other.name,
                "always": all,
                "directions": directions,
            }));
        }
    }
    let state_dependent = skips.iter().any(|pair| pair["always"] == json!(false));
    Ok(json!({
        "label": label,
        "outcome": "listed",
        "tuples": tuples,
        "blocks": blocks,
        "shape_count": count,
        "joins": joins,
        "covers": covers,
        "skip_pairs": skips,
        "skip_is_state_dependent": state_dependent,
    }))
}

/// `movement_properties`: the declarations, the state cache and the
/// vanilla tags, as the oracle describes them.
fn movement_properties(label: &str) -> Result<Value, String> {
    use voxel_data::block::classes::BlockClass;
    use voxel_data::block::definition::Fluid;
    let tags = vanilla_tags()?;
    let bits = |value: f32| format!("{:08x}", value.to_bits());
    let fluid_name = |fluid: Fluid| match fluid {
        Fluid::Empty => "minecraft:empty",
        Fluid::Water => "minecraft:water",
        Fluid::FlowingWater => "minecraft:flowing_water",
        Fluid::Lava => "minecraft:lava",
        Fluid::FlowingLava => "minecraft:flowing_lava",
    };
    let blocks: Vec<Value> = voxel_data::block::registry::BLOCKS
        .iter()
        .enumerate()
        .map(|(index, block)| {
            let first = block.first_state_id();
            let mut fluids: Vec<String> = (0..block.state_count())
                .map(|offset| {
                    let state =
                        voxel_world::block::state(first + offset).expect("a registry state");
                    let fluid = state.fluid;
                    format!(
                        "{}|{}|{}|{:08x}|{}|{}",
                        fluid_name(fluid.fluid),
                        fluid.amount,
                        fluid.falling,
                        fluid.own_height().to_bits(),
                        fluid.is_source(),
                        state.suffocating
                    )
                })
                .collect();
            if fluids.iter().all(|fluid| *fluid == fluids[0]) {
                fluids.truncate(1);
            }
            let tagged = |tag: &str| tags.contains(index, tag);
            json!({
                "name": block.name,
                "friction": bits(block.friction),
                "speed_factor": bits(block.speed_factor),
                "jump_factor": bits(block.jump_factor),
                "bounce_restitution": bits(block.bounce_restitution),
                "climbable": tagged("minecraft:climbable"),
                "fence": tagged("minecraft:fences"),
                "wall": tagged("minecraft:walls"),
                "can_glide_through": tagged("minecraft:can_glide_through"),
                "suppresses_bounce": tagged("minecraft:suppresses_bounce"),
                "fence_gate": block.class.is(BlockClass::FenceGateBlock),
                "trapdoor": block.class.is(BlockClass::TrapDoorBlock),
                "ladder": block.class.is(BlockClass::LadderBlock),
                "fluids": fluids,
            })
        })
        .collect();
    Ok(json!({"label": label, "outcome": "listed", "blocks": blocks}))
}

/// A default component value as the component's own codec writes it in JSON.
fn component_json(component: &voxel_data::item::component::Component) -> Value {
    use voxel_data::item::component::{AttributeDisplay, Component, Operation, Rarity, SlotGroup};
    match component {
        Component::ItemName(key) => json!({"translate": key}),
        Component::Rarity(rarity) => json!(match rarity {
            Rarity::Common => "common",
            Rarity::Uncommon => "uncommon",
            Rarity::Rare => "rare",
            Rarity::Epic => "epic",
        }),
        Component::AttributeModifiers(modifiers) => Value::Array(
            modifiers
                .iter()
                .map(|modifier| {
                    let mut entry = json!({
                        "type": modifier.attribute,
                        "id": modifier.id,
                        "amount": modifier.amount,
                        "operation": match modifier.operation {
                            Operation::AddValue => "add_value",
                            Operation::AddMultipliedBase => "add_multiplied_base",
                            Operation::AddMultipliedTotal => "add_multiplied_total",
                        },
                        "slot": match modifier.slot {
                            SlotGroup::Any => "any",
                            SlotGroup::MainHand => "mainhand",
                            SlotGroup::OffHand => "offhand",
                            SlotGroup::Hand => "hand",
                            SlotGroup::Feet => "feet",
                            SlotGroup::Legs => "legs",
                            SlotGroup::Chest => "chest",
                            SlotGroup::Head => "head",
                            SlotGroup::Armor => "armor",
                            SlotGroup::Body => "body",
                            SlotGroup::Saddle => "saddle",
                        },
                    });
                    if modifier.display == AttributeDisplay::Hidden {
                        entry["display"] = json!({"type": "hidden"});
                    }
                    entry
                })
                .collect(),
        ),
        Component::EnchantmentGlintOverride(value) => json!(value),
        Component::MapColor(color) => json!(color),
        Component::PotionContents(None) => json!({}),
        Component::PotionContents(Some(potion)) => json!({"potion": potion}),
        Component::StoredEnchantments(enchantments) => Value::Object(
            enchantments
                .iter()
                .map(|(name, level)| ((*name).to_owned(), json!(level)))
                .collect(),
        ),
        Component::PaintingVariant(name) => json!(name),
        Component::SuspiciousStewEffects(effects) => Value::Array(
            effects
                .iter()
                .map(|(name, duration)| json!({"id": name, "duration": duration}))
                .collect(),
        ),
        Component::BannerPatterns(layers) => Value::Array(
            layers
                .iter()
                .map(|(pattern, color)| json!({"color": color, "pattern": pattern}))
                .collect(),
        ),
        Component::TooltipDisplay(hidden) => json!({"hidden_components": hidden}),
        Component::PotionDurationScale(scale) => json!(f64::from(*scale)),
        Component::Fireworks { flight_duration } => json!({"flight_duration": flight_duration}),
        Component::Instrument(name) | Component::JukeboxPlayable(name) => json!(name),
        Component::BlockState(properties) => Value::Object(
            properties
                .iter()
                .map(|(key, value)| ((*key).to_owned(), json!(value)))
                .collect(),
        ),
        Component::OminousBottleAmplifier(amplifier) => json!(amplifier),
    }
}

/// An item's default components the client reads, as the codec's JSON and
/// as the network's bytes, keyed by component type name.
fn default_components(
    item: &voxel_data::item::definition::Item,
    registries: &voxel_game::registries::Registries,
) -> (Value, Value) {
    use voxel_data::item::component::Component;
    use voxel_data::item::component_types::NAMES;
    let mut json_map = serde_json::Map::new();
    let mut wire_map = serde_json::Map::new();
    let hex = |bytes: &[u8]| {
        bytes
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    };
    let mut put = |id: i32, json: Value, bytes: &[u8]| {
        let name = NAMES[id as usize];
        json_map.insert(name.to_owned(), json);
        wire_map.insert(name.to_owned(), json!(hex(bytes)));
    };
    let declared_name = item
        .components
        .iter()
        .any(|component| matches!(component, Component::ItemName(_)));
    if !declared_name {
        let key = voxel_game::default_components::description_key(item);
        put(
            voxel_data::item::component_types::ITEM_NAME,
            json!({"translate": key}),
            &voxel_game::default_components::translatable(&key),
        );
    }
    for component in item.components {
        if let Some((id, bytes)) =
            voxel_game::default_components::encode(component, Some(registries))
        {
            put(id, component_json(component), &bytes);
        }
    }
    (Value::Object(json_map), Value::Object(wire_map))
}

/// `item_declarations`: the declarations as the oracle describes them.
fn item_declarations(label: &str) -> Value {
    use voxel_data::item::classes::ItemClass;
    use voxel_data::item::definition::RuleBlocks;
    let classes: Vec<Value> = ItemClass::ALL
        .iter()
        .map(|class| {
            let declares: Vec<String> = class
                .declares()
                .iter()
                .map(|method| {
                    let name = format!("{method:?}");
                    name[..1].to_lowercase() + &name[1..]
                })
                .collect();
            json!({
                "name": format!("{class:?}"),
                "parent": class.parent().map_or(String::new(), |parent| format!("{parent:?}")),
                "declares": declares,
            })
        })
        .collect();
    let data_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data");
    let registries = voxel_game::registries::Registries::vanilla(&data_root);
    let items: Vec<Value> = voxel_data::item::registry::ITEMS
        .iter()
        .map(|item| {
            let mut described = json!({
                "name": item.name,
                "class": format!("{:?}", item.class),
                "max_stack_size": item.max_stack_size,
                "max_damage": item.max_damage,
                "consumable": item.consumable,
                "blocks_attacks": item.blocks_attacks,
                "kinetic_weapon": item.kinetic_weapon,
            });
            if let Some(equip) = &item.equippable {
                described["equippable"] = json!({
                    "slot": equip.slot,
                    "player": equip.player,
                    "swappable": equip.swappable,
                });
            }
            if let Some(remainder) = &item.crafting_remainder {
                described["crafting_remainder"] =
                    json!({"item": remainder.item, "count": remainder.count});
            }
            if let Some(tool) = item.tool {
                let rules: Vec<Value> = tool
                    .rules
                    .iter()
                    .map(|rule| {
                        let mut written = json!({});
                        written["blocks"] = match rule.blocks {
                            RuleBlocks::Tag(tag) => json!(format!("#{tag}")),
                            RuleBlocks::Blocks(names) => json!(names),
                        };
                        if let Some(speed) = rule.speed {
                            written["speed"] = json!(format!("{:08x}", speed.to_bits()));
                        }
                        if let Some(correct) = rule.correct_for_drops {
                            written["correct_for_drops"] = json!(correct);
                        }
                        written
                    })
                    .collect();
                described["tool"] = json!({
                    "rules": rules,
                    "default_mining_speed": format!("{:08x}", tool.default_mining_speed.to_bits()),
                    "damage_per_block": tool.damage_per_block,
                    "can_destroy_blocks_in_creative": tool.can_destroy_blocks_in_creative,
                });
            }
            if item.burn_time > 0 {
                described["burn_time"] = json!(item.burn_time);
            }
            if let Some(block) = item.block {
                described["block"] = json!(block);
            }
            let (components, wire) = default_components(item, &registries);
            described["components"] = components;
            described["component_wire"] = wire;
            described
        })
        .collect();
    let component_types: Vec<&str> = voxel_data::item::component_types::NAMES.to_vec();
    json!({
        "label": label,
        "outcome": "listed",
        "classes": classes,
        "items": items,
        "component_types": component_types,
    })
}

/// A parsed JSON value as a `serde_json` one, for the oracle's comparison.
fn serde_of(value: &voxel_core::json::JsonValue) -> Value {
    use voxel_core::json::JsonValue;
    match value {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(flag) => json!(flag),
        JsonValue::Number(text) => text.parse::<i64>().map_or_else(
            |_| json!(text.parse::<f64>().unwrap_or(0.0)),
            |number| json!(number),
        ),
        JsonValue::String(units) => json!(String::from_utf16_lossy(units)),
        JsonValue::Array(items) => Value::Array(items.iter().map(serde_of).collect()),
        JsonValue::Object(object) => Value::Object(
            object
                .entries()
                .map(|(key, value)| (String::from_utf16_lossy(key), serde_of(value)))
                .collect(),
        ),
    }
}

/// `tooltip_registries`: the data-driven registries as the game loads them
/// from the data pack, and the attributes' declarations.
fn tooltip_registries(label: &str) -> Value {
    use voxel_data::item::attributes::ATTRIBUTES;
    use voxel_game::registries::{DescribedEntry, Registries};
    let data_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data");
    let registries = Registries::vanilla(&data_root);
    let component =
        |value: &Option<voxel_core::json::JsonValue>| value.as_ref().map_or(Value::Null, serde_of);
    let enchantments: Vec<Value> = registries
        .enchantments
        .iter()
        .enumerate()
        .map(|(id, entry)| {
            json!({
                "name": entry.name,
                "id": id,
                "description": component(&entry.description),
                "curse": entry.curse,
                "max_level": entry.max_level,
                "tooltip_order": entry.tooltip_order,
            })
        })
        .collect();
    let paintings: Vec<Value> = registries
        .painting_variants
        .iter()
        .enumerate()
        .map(|(id, entry)| {
            let mut described = json!({
                "name": entry.name,
                "id": id,
                "width": entry.width,
                "height": entry.height,
            });
            if entry.title.is_some() {
                described["title"] = component(&entry.title);
            }
            if entry.author.is_some() {
                described["author"] = component(&entry.author);
            }
            described
        })
        .collect();
    let described = |entries: &[DescribedEntry]| -> Vec<Value> {
        entries
            .iter()
            .enumerate()
            .map(|(id, entry)| {
                json!({
                    "name": entry.name,
                    "id": id,
                    "description": component(&entry.description),
                })
            })
            .collect()
    };
    let banners: Vec<Value> = registries
        .banner_patterns
        .iter()
        .enumerate()
        .map(|(id, entry)| {
            json!({"name": entry.name, "id": id, "translation_key": entry.translation_key})
        })
        .collect();
    let attributes: Vec<Value> = ATTRIBUTES
        .iter()
        .map(|entry| {
            let (up, down) = entry.styles();
            json!({
                "name": entry.name,
                "description_id": entry.description_id(),
                "style_up": up,
                "style_down": down,
            })
        })
        .collect();
    json!({
        "label": label,
        "outcome": "listed",
        "enchantments": enchantments,
        "painting_variants": paintings,
        "instruments": described(&registries.instruments),
        "jukebox_songs": described(&registries.jukebox_songs),
        "banner_patterns": banners,
        "trim_materials": described(&registries.trim_materials),
        "trim_patterns": described(&registries.trim_patterns),
        "attributes": attributes,
    })
}

fn block_result(request: &Value) -> Result<Value, String> {
    use voxel_data::block::registry;
    let operation = string(request, "operation")?;
    let rows = request["rows"]
        .as_array()
        .ok_or("block rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        match operation {
            "state_definitions" => {
                let blocks: Vec<Value> = voxel_data::block::registry::BLOCKS
                    .iter()
                    .enumerate()
                    .map(|(id, block)| {
                        let properties: Vec<Value> = block
                            .properties
                            .iter()
                            .map(|property| {
                                json!({
                                    "name": property.name,
                                    "kind": match property.kind {
                                        registry::PropertyKind::Boolean => "boolean",
                                        registry::PropertyKind::Integer => "integer",
                                        registry::PropertyKind::Enum => "enum",
                                    },
                                    "values": property.values,
                                })
                            })
                            .collect();
                        json!({
                            "name": block.name,
                            "id": id,
                            "properties": properties,
                            "default": block.default,
                            "state_count": block.state_count(),
                            "first_state_id": block.first_state_id(),
                            "default_state_id": block.default_state_id(),
                        })
                    })
                    .collect();
                out.push(json!({
                    "label": label,
                    "outcome": "listed",
                    "block_count": registry::BLOCKS.len(),
                    "state_count": registry::BLOCKS
                        .iter()
                        .map(registry::Block::state_count)
                        .sum::<u32>(),
                    "blocks": blocks,
                }));
            }
            "offset_properties" => {
                use voxel_data::block::definition::OffsetType;
                let blocks: Vec<Value> = voxel_data::block::registry::BLOCKS
                    .iter()
                    .filter(|block| block.offset_type != OffsetType::None)
                    .map(|block| {
                        json!({
                            "name": block.name,
                            "kind": if block.offset_type == OffsetType::Xyz { "xyz" } else { "xz" },
                            "max_horizontal": format!(
                                "{:08x}",
                                voxel_world::block::max_horizontal_offset(block).to_bits()
                            ),
                            "max_vertical": format!(
                                "{:08x}",
                                voxel_world::block::max_vertical_offset(block).to_bits()
                            ),
                        })
                    })
                    .collect();
                out.push(json!({"label": label, "outcome": "listed", "blocks": blocks}));
            }
            "fluid_flow_properties" => {
                // `FlowingFluid.isSolidFace`'s sturdy faces (never for ice)
                // and `blocksMotion`, as runs of equal values.
                let count = registry::state_count();
                let value = |id: u32| {
                    let state = voxel_world::block::state(id).expect("a registry state");
                    let ice = state
                        .block()
                        .class
                        .is(voxel_data::block::classes::BlockClass::IceBlock);
                    let sturdy = if ice { 0 } else { state.sturdy_faces };
                    sturdy | u8::from(state.blocks_motion) << 6
                };
                let mut runs = Vec::new();
                let mut last = None;
                for id in 0..count {
                    let current = value(id);
                    if last != Some(current) {
                        runs.push(format!("{id}:{current}"));
                        last = Some(current);
                    }
                }
                out.push(json!({
                    "label": label,
                    "outcome": "listed",
                    "state_count": count,
                    "runs": runs,
                }));
            }
            "motion_blocking" => {
                let count = registry::state_count();
                let mut words = vec![0_u64; count.div_ceil(64) as usize];
                for id in 0..count {
                    if voxel_world::motion_blocking(id as i32) {
                        words[id as usize / 64] |= 1 << (id % 64);
                    }
                }
                let words: Vec<String> = words.iter().map(|word| format!("{word:016x}")).collect();
                out.push(json!({
                    "label": label,
                    "outcome": "listed",
                    "state_count": count,
                    "words": words,
                }));
            }
            "fluid_overlay_blocks" => {
                let blocks: Vec<&str> = registry::BLOCKS
                    .iter()
                    .filter(|block| voxel_world::block::shows_fluid_overlay(block))
                    .map(|block| block.name)
                    .collect();
                out.push(json!({"label": label, "outcome": "listed", "blocks": blocks}));
            }
            "block_declarations" => out.push(block_declarations(label)),
            "item_declarations" => out.push(item_declarations(label)),
            "block_behaviours" => {
                use voxel_data::block::classes::Method;
                // The class whose own method the block runs, as the
                // reference's dispatch resolves it.
                let owner = |block: &registry::Block, method: Method| {
                    block
                        .class
                        .lineage()
                        .find(|class| class.declares().contains(&method))
                        .map_or_else(|| "BlockBehaviour".to_owned(), |class| format!("{class:?}"))
                };
                let blocks: Vec<Value> = voxel_data::block::registry::BLOCKS
                    .iter()
                    .map(|block| {
                        let replaceable = if block.replaceable { "1" } else { "0" };
                        json!({
                            "name": block.name,
                            "class": format!("{:?}", block.class),
                            "placement": owner(block, Method::GetStateForPlacement),
                            "can_be_replaced": owner(block, Method::CanBeReplaced),
                            "can_survive": owner(block, Method::CanSurvive),
                            "use_without_item": owner(block, Method::UseWithoutItem),
                            "use_item_on": owner(block, Method::UseItemOn),
                            "update_shape": owner(block, Method::UpdateShape),
                            "replaceable": replaceable.repeat(block.state_count() as usize),
                            "stairs": block
                                .class
                                .is(voxel_data::block::classes::BlockClass::StairBlock),
                            "item": block.item_name(),
                        })
                    })
                    .collect();
                let items: Vec<Value> = voxel_data::item::registry::ITEMS
                    .iter()
                    .map(|item| {
                        use voxel_data::item::classes::ItemMethod;
                        let owner =
                            |method: ItemMethod| format!("{:?}", item.class.owner_of(method));
                        let mut described = json!({
                            "name": item.name,
                            "class": format!("{:?}", item.class),
                            "use_on": owner(ItemMethod::UseOn),
                            "use": owner(ItemMethod::Use),
                            "default_use_pass": item.use_does_nothing(),
                        });
                        if let Some(block) = item.block {
                            described["block"] = json!(block);
                        }
                        described
                    })
                    .collect();
                out.push(
                    json!({"label": label, "outcome": "listed", "blocks": blocks, "items": items}),
                );
            }
            "destroy_properties" => {
                let blocks: Vec<Value> = voxel_data::block::registry::BLOCKS
                    .iter()
                    .map(|block| {
                        let first = block.first_state_id();
                        let mut states: Vec<String> = (0..block.state_count())
                            .map(|offset| {
                                let state = voxel_world::block::state(first + offset)
                                    .expect("a registry state");
                                format!(
                                    "{:08x}|{}|{}|{}",
                                    block.destroy_time.to_bits(),
                                    block.requires_correct_tool_for_drops,
                                    block.air,
                                    state.fluid.legacy_block()
                                )
                            })
                            .collect();
                        if states.iter().all(|state| *state == states[0]) {
                            states.truncate(1);
                        }
                        json!({
                            "name": block.name,
                            "game_master": block.class.implements(
                                voxel_data::block::classes::BlockInterface::GameMasterBlock,
                            ),
                            "states": states,
                        })
                    })
                    .collect();
                out.push(json!({"label": label, "outcome": "listed", "blocks": blocks}));
            }
            "item_tools" => {
                use voxel_data::item::registry::{ITEMS, RuleBlocks};
                let items: Vec<Value> = ITEMS
                    .iter()
                    .map(|item| {
                        let mut described = json!({"name": item.name});
                        if let Some(tool) = item.tool {
                            let rules: Vec<Value> = tool
                                .rules
                                .iter()
                                .map(|rule| {
                                    let mut written = json!({});
                                    written["blocks"] = match rule.blocks {
                                        RuleBlocks::Tag(tag) => json!(format!("#{tag}")),
                                        RuleBlocks::Blocks(names) => json!(names),
                                    };
                                    if let Some(speed) = rule.speed {
                                        written["speed"] = json!(format!("{:08x}", speed.to_bits()));
                                    }
                                    if let Some(correct) = rule.correct_for_drops {
                                        written["correct_for_drops"] = json!(correct);
                                    }
                                    written
                                })
                                .collect();
                            described["tool"] = json!({
                                "rules": rules,
                                "default_mining_speed": format!("{:08x}", tool.default_mining_speed.to_bits()),
                                "damage_per_block": tool.damage_per_block,
                                "can_destroy_blocks_in_creative": tool.can_destroy_blocks_in_creative,
                            });
                        }
                        described
                    })
                    .collect();
                out.push(json!({"label": label, "outcome": "listed", "items": items}));
            }
            "item_stacking" => {
                use voxel_data::item::classes::ItemMethod;
                let items: Vec<Value> = voxel_data::item::registry::ITEMS
                    .iter()
                    .map(|item| {
                        let owner =
                            |method: ItemMethod| format!("{:?}", item.class.owner_of(method));
                        let mut described = json!({
                            "name": item.name,
                            "max_stack_size": item.max_stack_size,
                            "max_damage": item.max_damage,
                            "stacked_on_other": owner(ItemMethod::OverrideStackedOnOther),
                            "other_stacked_on_me": owner(ItemMethod::OverrideOtherStackedOnMe),
                        });
                        if let Some(equip) = &item.equippable {
                            described["equip_slot"] = json!(equip.slot);
                            described["player_can_equip"] = json!(equip.player);
                        }
                        if let Some(remainder) = &item.crafting_remainder {
                            described["crafting_remainder"] = json!(remainder.item);
                            described["crafting_remainder_count"] = json!(remainder.count);
                        }
                        described
                    })
                    .collect();
                out.push(json!({"label": label, "outcome": "listed", "items": items}));
            }
            "item_components" => {
                let hex = |bytes: &[u8]| {
                    bytes
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>()
                };
                // The reference's whole default patch of each item, kept as a
                // fixture, is read by this port, and both forms are written
                // back from what it read.
                const PATCHES: &str =
                    include_str!("../../../tests/behavior/fixtures/item-default-patches.txt");
                let items: Vec<Value> = PATCHES
                    .lines()
                    .map(|line| {
                        let (name, wire) = line.split_once(' ').expect("a name and its bytes");
                        let bytes: Vec<u8> = (0..wire.len())
                            .step_by(2)
                            .map(|at| u8::from_str_radix(&wire[at..at + 2], 16).expect("hex"))
                            .collect();
                        let mut input = bytes.as_slice();
                        match voxel_game::item_components::read_patch(&mut input) {
                            Ok(patch) if input.is_empty() => json!({
                                "name": name,
                                "wire": hex(&patch.trusted()),
                                "delimited": hex(&patch.delimited()),
                                "components": patch.added.iter().map(|(id, _)| *id).collect::<Vec<_>>(),
                            }),
                            _ => json!({"name": name, "wire": "unreadable"}),
                        }
                    })
                    .collect();
                out.push(json!({"label": label, "outcome": "listed", "items": items}));
            }
            "tooltip_registries" => out.push(tooltip_registries(label)),
            "potions" => {
                use voxel_data::item::effect::{attribute_id, effect_id};
                use voxel_data::item::mob_effects::MOB_EFFECTS;
                use voxel_data::item::potions::POTIONS;
                let effects: Vec<Value> = MOB_EFFECTS
                    .iter()
                    .map(|effect| {
                        json!({
                            "name": effect.name,
                            "color": effect.color,
                            "description_id": effect.description_id(),
                            "style": effect.style(),
                            "modifiers": effect.modifiers.iter().map(|modifier| json!({
                                "attribute": attribute_id(modifier.attribute),
                                "amount": modifier.amount,
                                "operation": modifier.operation,
                            })).collect::<Vec<_>>(),
                        })
                    })
                    .collect();
                let potions: Vec<Value> = POTIONS
                    .iter()
                    .map(|potion| {
                        json!({
                            "name": potion.name,
                            "potion_name": potion.potion_name(),
                            "effects": potion.effects.iter().map(|effect| json!({
                                "effect": effect_id(effect.effect),
                                "amplifier": effect.amplifier,
                                "duration": effect.duration,
                                "visible": effect.visible,
                            })).collect::<Vec<_>>(),
                        })
                    })
                    .collect();
                out.push(json!({
                    "label": label,
                    "outcome": "listed",
                    "effects": effects,
                    "potions": potions,
                }));
            }
            "creative_tabs" => {
                use voxel_data::item::component_types::NAMES;
                use voxel_data::item::creative_tabs::{CREATIVE_TABS, DEFAULT_TAB, TabKind};
                let data_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../.data/minecraft/assets/jar/data");
                let registries = voxel_game::registries::Registries::vanilla(&data_root);
                let stack = |entry: &voxel_data::item::creative::CreativeStack| {
                    let made = voxel_game::creative_menu::table_stack(entry, &registries);
                    let mut wire = Vec::new();
                    voxel_core::wire::write_varint(made.item_count(), &mut wire);
                    voxel_core::wire::write_varint(made.item_registry_id(), &mut wire);
                    wire.extend_from_slice(made.patch_bytes());
                    let mut components = serde_json::Map::new();
                    for component in entry.components {
                        if let Some((id, _)) =
                            voxel_game::default_components::encode(component, Some(&registries))
                        {
                            components
                                .insert(NAMES[id as usize].to_owned(), component_json(component));
                        }
                    }
                    json!({
                        "item": entry.item,
                        "wire": wire.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
                        "positive": made.positive_component_type_ids(),
                        "negative": made.negative_component_type_ids(),
                        "components": components,
                    })
                };
                let tabs: Vec<Value> = CREATIVE_TABS
                    .iter()
                    .map(|tab| {
                        json!({
                            "name": tab.name,
                            "type": match tab.kind {
                                TabKind::Category => "CATEGORY",
                                TabKind::Inventory => "INVENTORY",
                                TabKind::Hotbar => "HOTBAR",
                                TabKind::Search => "SEARCH",
                            },
                            "row": if tab.top { "TOP" } else { "BOTTOM" },
                            "column": tab.column,
                            "aligned_right": tab.aligned_right,
                            "show_title": tab.show_title,
                            "can_scroll": tab.can_scroll,
                            "should_display": tab.should_display,
                            "background": tab.background,
                            "title": tab.title,
                            "icon": stack(&tab.icon),
                            "items": tab.items.iter().map(&stack).collect::<Vec<_>>(),
                            "search_items": if tab.search_items.is_empty() { tab.items } else { tab.search_items }
                                .iter()
                                .map(&stack)
                                .collect::<Vec<_>>(),
                        })
                    })
                    .collect();
                out.push(json!({
                    "label": label,
                    "outcome": "listed",
                    "default_tab": DEFAULT_TAB,
                    "tabs": tabs,
                }));
            }
            "light_properties" => {
                let blocks: Vec<Value> = voxel_data::block::registry::BLOCKS
                    .iter()
                    .map(|block| {
                        let first = block.first_state_id();
                        let mut states: Vec<String> = (0..block.state_count())
                            .map(|offset| {
                                let t = voxel_world::block::state(first + offset)
                                    .expect("a registry state");
                                format!(
                                    "{}|{}|{:08x}|{}|{}|{}|{}|{}|{}",
                                    t.light_emission,
                                    t.emissive_rendering,
                                    t.shade_brightness.to_bits(),
                                    t.view_blocking,
                                    t.light_dampening,
                                    t.solid_render,
                                    t.collision_full_block,
                                    t.use_shape_for_light_occlusion,
                                    t.can_occlude
                                )
                            })
                            .collect();
                        if states.iter().all(|state| *state == states[0]) {
                            states.truncate(1);
                        }
                        json!({"name": block.name, "states": states})
                    })
                    .collect();
                out.push(json!({"label": label, "outcome": "listed", "blocks": blocks}));
            }
            "face_occlusion" => out.push(face_occlusion(label)?),
            "collision_shapes" => out.push(shape_table(label, false)),
            "outline_shapes" => out.push(shape_table(label, true)),
            "movement_properties" => out.push(movement_properties(label)?),
            "block_tints" => {
                let tinted: Vec<Value> = registry::TINTS
                    .iter()
                    .map(|(name, layers)| {
                        json!({
                            "name": name,
                            "layers": layers
                                .iter()
                                .map(|colour| format!("{colour:08x}"))
                                .collect::<Vec<_>>(),
                        })
                    })
                    .collect();
                out.push(json!({"label": label, "outcome": "listed", "tinted": tinted}));
            }
            other => return Err(format!("unknown block operation: {other}")),
        }
    }
    Ok(json!({"rows": out}))
}

/// Every state's collision or outline shape as the reference lists them: the
/// distinct shapes by their boxes in the order they are first met, then for
/// each block the shape of its one state or of each.
///
/// The collision shape of a block that shifts by position is the one at the
/// origin; the outline is listed before the shift and a block is position
/// dependent when its shape is not empty and the shift is not always zero.
fn shape_table(label: &str, outline: bool) -> Value {
    const ELSEWHERE: [[i32; 3]; 4] = [[13, 70, -7], [-200, 5, 999], [1, 1, 1], [-3, 64, 12]];
    let mut ids: std::collections::HashMap<Vec<u64>, usize> = std::collections::HashMap::new();
    let mut shapes: Vec<Value> = Vec::new();
    let states = voxel_world::block::states();
    let blocks: Vec<Value> = voxel_data::block::registry::BLOCKS
        .iter()
        .map(|block| {
            let mut per_state = Vec::new();
            let mut moves = false;
            for offset in 0..block.state_count() {
                let info = voxel_world::block::state(block.first_state_id() + offset)
                    .expect("a registry state");
                let origin_collision;
                let shape = if outline {
                    states.outline(info)
                } else {
                    origin_collision = voxel_world::block_state::collision_shape_at(
                        block.first_state_id() + offset,
                        [0; 3],
                    );
                    &origin_collision
                };
                let boxes: Vec<[u64; 6]> = shape
                    .to_aabbs()
                    .iter()
                    .map(|aabb| {
                        [
                            aabb.min_x, aabb.min_y, aabb.min_z, aabb.max_x, aabb.max_y, aabb.max_z,
                        ]
                        .map(f64::to_bits)
                    })
                    .collect();
                let origin = voxel_world::block::offset(block, [0, 0, 0]);
                moves |= !boxes.is_empty()
                    && voxel_world::block::moves_with_position(block)
                    && ELSEWHERE.iter().any(|at| {
                        let shift = voxel_world::block::offset(block, *at);
                        if outline {
                            shift != [0.0; 3]
                        } else {
                            shift != origin
                        }
                    });
                let key: Vec<u64> = boxes.iter().flatten().copied().collect();
                let id = *ids.entry(key).or_insert_with(|| {
                    shapes.push(Value::Array(
                        boxes
                            .iter()
                            .map(|bits| {
                                Value::Array(
                                    bits.iter()
                                        .map(|bit| json!(format!("{bit:016x}")))
                                        .collect(),
                                )
                            })
                            .collect(),
                    ));
                    shapes.len() - 1
                });
                per_state.push(id);
            }
            let listed = if per_state.iter().all(|id| *id == per_state[0]) {
                vec![per_state[0]]
            } else {
                per_state
            };
            let mut entry = json!({
                "name": block.name,
                "shapes": listed,
                "position_dependent": moves,
                "dynamic": block.dynamic_shape,
            });
            if outline {
                // The shift is explained by the same offset at any position.
                entry["offset_explained"] = json!(true);
            }
            entry
        })
        .collect();
    json!({"label": label, "outcome": "listed", "shapes": shapes, "blocks": blocks})
}

fn image_result(request: &Value) -> Result<Value, String> {
    if string(request, "operation")? == "srgb_tables" {
        return image_tables_result(request);
    }
    if string(request, "operation")? == "mip_levels" {
        return image_mip_result(request);
    }
    if string(request, "operation")? == "compute_transparency" {
        return image_transparency_result(request);
    }
    if string(request, "operation")? == "texture_metadata" {
        return image_texture_metadata_result(request);
    }
    if string(request, "operation")? == "sprite_uv" {
        return image_sprite_uv_result(request);
    }
    if string(request, "operation")? == "colour_map" {
        return image_colour_map_result(request);
    }
    if string(request, "operation")? == "argb_multiply" {
        return image_argb_multiply_result(request);
    }
    if string(request, "operation")? == "item_side_faces" {
        let rows = request["rows"].as_array().ok_or("rows must be an array")?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            // Test glue: the texture from the local copy of the pinned jar.
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../.data/minecraft/assets/jar")
                .join(string(row, "texture")?);
            let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let image = voxel_render::png::decode_rgba(&bytes).map_err(|e| format!("{e:?}"))?;
            let alpha: Vec<u8> = image.rgba.chunks(4).map(|pixel| pixel[3]).collect();
            let faces: Vec<String> = voxel_render::item_model::side_faces(
                &alpha,
                image.width as i32,
                image.height as i32,
            )
            .into_iter()
            .map(|(facing, x, y)| format!("{},{x:02},{y:02}", facing.name()))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
            out.push(json!({
                "label": string(row, "label")?,
                "outcome": "scanned",
                "width": image.width,
                "height": image.height,
                "faces": faces,
            }));
        }
        return Ok(json!({"rows": out}));
    }
    if string(request, "operation")? == "tint_constants" {
        return image_tint_constants_result(request);
    }
    if string(request, "operation")? != "png_decode" {
        return Err("unknown image operation".into());
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let hex = string(row, "png")?;
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .map(|index| u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16))
            .collect::<Result<_, _>>()
            .map_err(|error| format!("png hex: {error}"))?;
        match voxel_render::decode_rgba(&bytes) {
            Err(_) => out.push(json!({"label": label, "outcome": "decode_failed"})),
            Ok(image) => {
                let mut encoded = String::with_capacity(image.rgba.len() * 2);
                for byte in &image.rgba {
                    encoded.push_str(&format!("{byte:02x}"));
                }
                out.push(json!({
                    "label": label,
                    "outcome": "decoded",
                    "width": image.width,
                    "height": image.height,
                    "rgba": encoded,
                }));
            }
        }
    }
    Ok(json!({"rows": out}))
}

fn atlas_result(request: &Value) -> Result<Value, String> {
    if string(request, "operation")? != "stitch" {
        return Err("unknown atlas operation".into());
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("atlas rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut stitcher = voxel_render::Stitcher::new(
            i32_field(row, "max_width")?,
            i32_field(row, "max_height")?,
            i32_field(row, "mip_level")?,
            i32_field(row, "anisotropy_bit")?,
        );
        let sprites = row["sprites"]
            .as_array()
            .ok_or("sprites must be an array")?;
        for sprite in sprites {
            let name = Identifier::parse(string(sprite, "name")?)
                .map_err(|error| format!("sprite name: {error:?}"))?;
            stitcher.register_sprite(voxel_render::SpriteEntry {
                width: i32_field(sprite, "width")?,
                height: i32_field(sprite, "height")?,
                name,
            });
        }
        let label = string(row, "label")?;
        match stitcher.stitch() {
            Err(_) => out.push(json!({"label": label, "outcome": "stitch_failed"})),
            Ok(()) => {
                let placements: Vec<Value> = stitcher
                    .placements()
                    .into_iter()
                    .map(|placement| {
                        json!({
                            "name": placement.name.as_string(),
                            "x": placement.x,
                            "y": placement.y,
                            "padding": placement.padding,
                        })
                    })
                    .collect();
                out.push(json!({
                    "label": label,
                    "outcome": "stitched",
                    "width": stitcher.width(),
                    "height": stitcher.height(),
                    "placements": placements,
                }));
            }
        }
    }
    Ok(json!({"rows": out}))
}

fn window_result(request: &Value) -> Result<Value, String> {
    match string(request, "operation")? {
        "base_size" => Ok(json!({
            "base_width": window::BASE_WIDTH,
            "base_height": window::BASE_HEIGHT,
        })),
        "min_size" => {
            let rows = request["rows"]
                .as_array()
                .ok_or("window rows must be an array")?;
            let mut out = Vec::with_capacity(rows.len());
            for row in rows {
                out.push(json!({
                    "label": string(row, "label")?,
                    "allowed": window::allowed_window_min_size(i32_field(row, "requested")?),
                }));
            }
            Ok(json!({"rows": out}))
        }
        "gui_scale" => {
            let rows = request["rows"]
                .as_array()
                .ok_or("window rows must be an array")?;
            let mut out = Vec::with_capacity(rows.len());
            for row in rows {
                let mut scale = window::WindowScale::new(
                    i32_field(row, "framebuffer_width")?,
                    i32_field(row, "framebuffer_height")?,
                );
                let calculated = scale.calculate_scale(
                    i32_field(row, "max_scale")?,
                    row["enforce_unicode"]
                        .as_bool()
                        .ok_or("enforce_unicode must be a bool")?,
                );
                scale.set_gui_scale(calculated);
                out.push(json!({
                    "label": string(row, "label")?,
                    "calculated_scale": calculated,
                    "gui_scale": scale.gui_scale(),
                    "gui_scaled_width": scale.gui_scaled_width(),
                    "gui_scaled_height": scale.gui_scaled_height(),
                }));
            }
            Ok(json!({"rows": out}))
        }
        _ => Err("unknown window operation".into()),
    }
}

/// A block state named as the scenes name it: a block and its properties.
fn named_state(block: &Value) -> Result<i32, String> {
    let name = string(block, "block")?;
    let definition =
        voxel_data::block::registry::block(name).ok_or_else(|| format!("unknown block {name}"))?;
    let mut chosen = std::collections::BTreeMap::new();
    if let Some(properties) = block["properties"].as_object() {
        for (key, value) in properties {
            chosen.insert(
                key.clone(),
                value
                    .as_str()
                    .ok_or("property values are strings")?
                    .to_owned(),
            );
        }
    }
    definition
        .state_id(&chosen)
        .map(|state| state as i32)
        .ok_or_else(|| format!("bad properties for {name}"))
}

/// The vanilla block tags, resolved from the pinned jar's data pack as the
/// server would send them. Test glue only: the client takes its tags from
/// the server. The files stay local under `.data`.
fn vanilla_block_tags() -> Result<ClientboundUpdateTagsPacket, String> {
    vanilla_tags_of("block", "minecraft:block", &|name| {
        voxel_data::block::registry::BLOCKS
            .iter()
            .position(|block| block.name == name)
            .map(|index| index as i32)
    })
}

/// The item tags of the data pack, as a tag packet would carry them.
fn vanilla_item_tags() -> Result<voxel_world::block::tags::ItemTags, String> {
    let packet = vanilla_tags_of("item", "minecraft:item", &voxel_game::item::id_of)?;
    voxel_world::block::tags::ItemTags::from_registries(&packet.registries)
        .ok_or_else(|| "no item tags".to_owned())
}

fn vanilla_tags_of(
    folder: &str,
    registry_name: &str,
    index_of: &dyn Fn(&str) -> Option<i32>,
) -> Result<ClientboundUpdateTagsPacket, String> {
    fn resolve(
        root: &std::path::Path,
        name: &str,
        seen: &mut BTreeMap<String, Vec<i32>>,
        index_of: &dyn Fn(&str) -> Option<i32>,
    ) -> Result<Vec<i32>, String> {
        if let Some(done) = seen.get(name) {
            return Ok(done.clone());
        }
        let path = root.join(format!("{name}.json"));
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let parsed: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for entry in parsed["values"].as_array().ok_or("tag values")? {
            let (id, required) = match entry {
                Value::String(id) => (id.as_str(), true),
                other => (
                    other["id"].as_str().ok_or("tag entry id")?,
                    other["required"].as_bool().unwrap_or(true),
                ),
            };
            if let Some(tag) = id.strip_prefix('#') {
                let path = tag.strip_prefix("minecraft:").ok_or("foreign tag")?;
                out.extend(resolve(root, path, seen, index_of)?);
            } else {
                match index_of(id) {
                    Some(index) => out.push(index),
                    None if !required => {}
                    None => return Err(format!("unknown entry {id} in tag {name}")),
                }
            }
        }
        seen.insert(name.to_owned(), out.clone());
        Ok(out)
    }
    fn walk(
        dir: &std::path::Path,
        root: &std::path::Path,
        names: &mut Vec<String>,
    ) -> Result<(), String> {
        for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.is_dir() {
                walk(&path, root, names)?;
            } else if let Some(stem) = path.strip_prefix(root).ok().and_then(|p| p.to_str()) {
                names.push(stem.trim_end_matches(".json").replace('\\', "/"));
            }
        }
        Ok(())
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data/minecraft/tags")
        .join(folder);
    let mut names = Vec::new();
    walk(&root, &root, &mut names)?;
    names.sort();
    let mut seen = BTreeMap::new();
    let mut tags = Vec::new();
    for name in names {
        let mut values = resolve(&root, &name, &mut seen, index_of)?;
        values.sort_unstable();
        values.dedup();
        let id = Identifier::parse(&format!("minecraft:{name}")).map_err(|_| "tag id")?;
        tags.push(TagEntry::new(id, values));
    }
    let registry = Identifier::parse(registry_name).map_err(|_| "registry id")?;
    Ok(ClientboundUpdateTagsPacket::new(vec![RegistryTags::new(
        registry, tags,
    )]))
}

/// The scene's blocks in a client world of loaded chunks, as the movement
/// oracle builds its level.
fn movement_world(row: &Value) -> Result<ClientWorld, String> {
    use voxel_data::block::registry;
    use voxel_world::{DimensionBounds, SectionInput};
    // The overworld's bounds, as the reference level is built with.
    let dimension = DimensionBounds::new(-64, 384);
    let mut placed = std::collections::HashMap::new();
    for block in row["blocks"].as_array().ok_or("blocks must be an array")? {
        let name = string(block, "block")?;
        let definition = registry::block(name).ok_or_else(|| format!("unknown block {name}"))?;
        let mut chosen = std::collections::BTreeMap::new();
        if let Some(properties) = block["properties"].as_object() {
            for (key, value) in properties {
                chosen.insert(
                    key.clone(),
                    value
                        .as_str()
                        .ok_or("property values are strings")?
                        .to_owned(),
                );
            }
        }
        let state = definition
            .state_id(&chosen)
            .ok_or_else(|| format!("bad properties for {name}"))?;
        placed.insert(
            (
                i32_field(block, "x")?,
                i32_field(block, "y")?,
                i32_field(block, "z")?,
            ),
            state as i32,
        );
    }
    let mut world = ClientWorld::new(dimension);
    let radius = i32_field(row, "chunk_radius")?;
    let biomes = vec![0_i32; 64];
    for chunk_x in -radius..=radius {
        for chunk_z in -radius..=radius {
            let mut sections = Vec::new();
            for index in 0..dimension.section_count() {
                let min_y = dimension.min_y + index as i32 * 16;
                let mut states = vec![0_i32; 4096];
                let mut non_empty = 0_i16;
                let mut fluids = 0_i16;
                for y in 0..16 {
                    for z in 0..16 {
                        for x in 0..16 {
                            if let Some(&state) =
                                placed.get(&(chunk_x * 16 + x, min_y + y, chunk_z * 16 + z))
                            {
                                states[(y * 256 + z * 16 + x) as usize] = state;
                                let (block, _) = registry::block_for_state(state as u32)
                                    .ok_or("state out of range")?;
                                if !matches!(
                                    block.name,
                                    "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                                ) {
                                    non_empty += 1;
                                }
                                if voxel_world::local_player::state_has_fluid(state) {
                                    fluids += 1;
                                }
                            }
                        }
                    }
                }
                sections.push((non_empty, fluids, states));
            }
            world.replace_chunk(
                chunk_x,
                chunk_z,
                sections
                    .iter()
                    .map(|(non_empty, fluids, states)| SectionInput {
                        non_empty_block_count: *non_empty,
                        fluid_count: *fluids,
                        block_states: states,
                        biomes: &biomes,
                    }),
            );
        }
    }

    // The reference level's registry holds the vanilla tags.
    world.apply_update_tags_packet(&vanilla_block_tags()?);
    Ok(world)
}

/// `light_changes`: the client world lit from nothing, then block updates,
/// each followed by a run of the light engine; a box of light after each.
fn light_changes_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("light rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut world = movement_world(row)?;
        world.light_from_sources();
        world.run_light_updates();
        let box_ = &row["box"];
        let origin = (
            i32_field(box_, "x")?,
            i32_field(box_, "y")?,
            i32_field(box_, "z")?,
        );
        let size = (
            i32_field(box_, "size_x")?,
            i32_field(box_, "size_y")?,
            i32_field(box_, "size_z")?,
        );
        let snapshot = |world: &ClientWorld| {
            let mut block = String::new();
            let mut sky = String::new();
            for y in 0..size.1 {
                for z in 0..size.2 {
                    for x in 0..size.0 {
                        let (x, y, z) = (origin.0 + x, origin.1 + y, origin.2 + z);
                        block.push_str(&format!("{:x}", world.block_brightness(x, y, z)));
                        sky.push_str(&format!("{:x}", world.sky_brightness(x, y, z)));
                    }
                }
            }
            json!({"block": block, "sky": sky})
        };
        let mut snapshots = vec![snapshot(&world)];
        for event in row["events"].as_array().ok_or("events must be an array")? {
            world.apply_server_block_state(
                i32_field(event, "x")?,
                i32_field(event, "y")?,
                i32_field(event, "z")?,
                named_state(event)?,
            );
            world.run_light_updates();
            snapshots.push(snapshot(&world));
        }
        out.push(json!({
            "label": string(row, "label")?,
            "outcome": "lit",
            "snapshots": snapshots,
        }));
    }
    Ok(json!({"rows": out}))
}

/// The count of the stack in an inventory menu slot, zero when empty.
fn hand_count(world: &ClientWorld, slot: i16) -> i32 {
    world
        .local_player()
        .and_then(|state| state.inventory_slot(slot))
        .map_or(0, |stack| stack.item_count())
}

fn movement_result(request: &Value) -> Result<Value, String> {
    use voxel_world::ClientPlayerGameMode;
    use voxel_world::local_player::LocalPlayer;

    fn bits64(row: &Value, key: &str) -> Result<f64, String> {
        let text = string(row, key)?;
        u64::from_str_radix(text, 16)
            .map(f64::from_bits)
            .map_err(|error| format!("{key}: {error}"))
    }
    fn bits32(row: &Value, key: &str) -> Result<f32, String> {
        let text = string(row, key)?;
        u32::from_str_radix(text, 16)
            .map(f32::from_bits)
            .map_err(|error| format!("{key}: {error}"))
    }
    fn flag(row: &Value, key: &str) -> Result<bool, String> {
        row[key]
            .as_bool()
            .ok_or_else(|| format!("{key} must be a bool"))
    }

    if string(request, "operation")? == "light_changes" {
        return light_changes_result(request);
    }
    if string(request, "operation")? == "container_clicks" {
        return container_clicks_result(request);
    }
    if string(request, "operation")? != "local_player_ticks" {
        return Err("unknown movement operation".into());
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("movement rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let mut world = movement_world(row)?;
        let game_mode = match string(row, "game_mode")? {
            "survival" => ClientPlayerGameMode::Survival,
            "creative" => ClientPlayerGameMode::Creative,
            "adventure" => ClientPlayerGameMode::Adventure,
            "spectator" => ClientPlayerGameMode::Spectator,
            other => return Err(format!("unknown game mode {other}")),
        };
        let game_mode_of_row = game_mode;
        let mut player = LocalPlayer::new(
            i32_field(row, "entity_id")?,
            voxel_core::math::Vec3::new(bits64(row, "x")?, bits64(row, "y")?, bits64(row, "z")?),
            bits32(row, "y_rot")?,
            bits32(row, "x_rot")?,
            game_mode,
            flag(row, "auto_jump")?,
        );
        let breaking = row.get("breaking");
        let mut game_mode = voxel_game::game_mode::GameMode::default();
        if let Some(breaking) = breaking {
            world.set_local_player_id(player.entity_id());
            if let Some(name) = breaking["main_hand"].as_str() {
                let item =
                    voxel_game::item::id_of(name).ok_or_else(|| format!("unknown item {name}"))?;
                let count = breaking["main_hand_count"].as_i64().unwrap_or(1) as i32;
                world.apply_local_player_container_slot(
                    0,
                    0,
                    36,
                    Some(&voxel_world::ClientItemStackState::new(count, item)),
                );
                player.set_main_hand_item(item);
            }
            if let Some(name) = breaking["off_hand"].as_str() {
                let item =
                    voxel_game::item::id_of(name).ok_or_else(|| format!("unknown item {name}"))?;
                world.apply_local_player_container_slot(
                    0,
                    0,
                    45,
                    Some(&voxel_world::ClientItemStackState::new(1, item)),
                );
            }
            for effect in breaking["effects"].as_array().into_iter().flatten() {
                let id = match string(effect, "effect")? {
                    "minecraft:haste" => 2,
                    "minecraft:mining_fatigue" => 3,
                    other => return Err(format!("unknown effect {other}")),
                };
                world.apply_local_player_mob_effect(
                    player.entity_id(),
                    id,
                    i32_field(effect, "amplifier")?,
                    1_000_000,
                    0,
                );
            }
        }
        let mut ticks = Vec::new();
        let mut failure = None;
        for tick in row["ticks"].as_array().ok_or("ticks must be an array")? {
            let keys = voxel_world::input::Input::new(
                flag(tick, "forward")?,
                flag(tick, "backward")?,
                flag(tick, "left")?,
                flag(tick, "right")?,
                flag(tick, "jump")?,
                flag(tick, "sneak")?,
                flag(tick, "sprint")?,
            );
            let (mut y_rot, mut x_rot) = player.rotation();
            if tick.get("y_rot").is_some() {
                y_rot = bits32(tick, "y_rot")?;
            }
            if tick.get("x_rot").is_some() {
                x_rot = bits32(tick, "x_rot")?;
            }
            player.set_rotation(y_rot, x_rot);
            let mut actions = Vec::new();
            if let Some(attack) = tick.get("attack") {
                for event in attack["server"].as_array().into_iter().flatten() {
                    if event.get("ack").is_some() {
                        for synced in world.handle_block_changed_ack(i32_field(event, "ack")?) {
                            let pos =
                                voxel_core::binary::BlockPos::new(synced.x, synced.y, synced.z);
                            if let Some(position) = synced.player_position
                                && player.is_colliding(&world, pos, synced.state)
                            {
                                player.abs_snap_to(position);
                            }
                        }
                    } else {
                        world.apply_server_block_state(
                            i32_field(event, "x")?,
                            i32_field(event, "y")?,
                            i32_field(event, "z")?,
                            named_state(event)?,
                        );
                    }
                }
                let aim = match attack.get("target") {
                    Some(target) => {
                        let (x, y, z) = (
                            i32_field(target, "x")?,
                            i32_field(target, "y")?,
                            i32_field(target, "z")?,
                        );
                        let location = match target.get("location") {
                            Some(at) => [bits64(at, "x")?, bits64(at, "y")?, bits64(at, "z")?],
                            None => [f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5],
                        };
                        voxel_game::game_mode::Aim::Block {
                            x,
                            y,
                            z,
                            face: match string(target, "face")? {
                                "down" => 0,
                                "up" => 1,
                                "north" => 2,
                                "south" => 3,
                                "west" => 4,
                                "east" => 5,
                                other => return Err(format!("unknown face {other}")),
                            },
                            location,
                            inside: target["inside"].as_bool().unwrap_or(false),
                        }
                    }
                    None => voxel_game::game_mode::Aim::Miss,
                };
                let count = |value: &Value, key: &str| -> Result<u32, String> {
                    u32::try_from(i32_field(value, key)?).map_err(|e| e.to_string())
                };
                let use_ = attack.get("use");
                let keys = voxel_game::game_mode::Keys {
                    attack_clicks: count(attack, "clicks")?,
                    attack_held: flag(attack, "down")?,
                    use_clicks: use_.map_or(Ok(0), |use_| count(use_, "clicks"))?,
                    use_held: use_.map_or(Ok(false), |use_| flag(use_, "down"))?,
                };
                let miner = voxel_game::game_mode::Miner::of(&world, 0);
                let mut ctx = voxel_game::game_mode::Context {
                    world: &mut world,
                    player: &mut player,
                    miner: &miner,
                    mode: game_mode_of_row,
                    selected_slot: 0,
                };
                actions = game_mode.keys_tick(&mut ctx, keys, aim);
            }
            // As the session does: the main hand the player ticks with is
            // the world's inventory now.
            if breaking.is_some() {
                let main_hand = world
                    .local_player()
                    .and_then(|state| state.inventory_slot(36))
                    .map_or(0, |stack| stack.item_registry_id());
                player.set_main_hand_item(main_hand);
            }
            let sent = match player.tick(&world, keys) {
                Ok(sent) => sent,
                Err(error) => {
                    failure = Some(error.to_string());
                    break;
                }
            };
            let position = player.position();
            let motion = player.delta_movement();
            let (y_rot, x_rot) = player.rotation();
            let mut packets: Vec<Value> = actions
                .iter()
                .map(|action| json!({"type": action.type_name(), "bytes": hex(&action.encode_body())}))
                .collect();
            packets.extend(sent.iter().map(
                |packet| json!({"type": packet.type_name(), "bytes": hex(&packet.encode_body())}),
            ));
            let mut described = json!({
                "x": format!("{:016x}", position.x.to_bits()),
                "y": format!("{:016x}", position.y.to_bits()),
                "z": format!("{:016x}", position.z.to_bits()),
                "motion_x": format!("{:016x}", motion.x.to_bits()),
                "motion_y": format!("{:016x}", motion.y.to_bits()),
                "motion_z": format!("{:016x}", motion.z.to_bits()),
                "y_rot": format!("{:08x}", y_rot.to_bits()),
                "x_rot": format!("{:08x}", x_rot.to_bits()),
                "on_ground": player.on_ground(),
                "horizontal_collision": player.horizontal_collision(),
                "vertical_collision": player.vertical_collision(),
                "minor_horizontal_collision": player.minor_horizontal_collision(),
                "sprinting": player.is_sprinting(),
                "shift_key_down": player.is_shift_key_down(),
                "pose": player.pose().java_name(),
                "fall_distance": format!("{:016x}", player.fall_distance().to_bits()),
                "flying": player.is_flying(),
                "bob_walk": format!("{:08x}", player.view_bob(0.5).0.to_bits()),
                "bob": format!("{:08x}", player.view_bob(0.5).1.to_bits()),
                "body_rot": format!("{:08x}", player.model_pose(0.5).body_rot.to_bits()),
                "head_rot": format!("{:08x}", player.model_pose(0.5).head_rot.to_bits()),
                "walk_position": format!("{:08x}", player.model_pose(0.5).walk_position.to_bits()),
                "walk_speed": format!("{:08x}", player.model_pose(0.5).walk_speed.to_bits()),
                "swim_amount": format!("{:08x}", player.model_pose(0.5).swim_amount.to_bits()),
                "hand_x_bob": format!("{:08x}", player.hand_bob(0.5).0.to_bits()),
                "hand_y_bob": format!("{:08x}", player.hand_bob(0.5).1.to_bits()),
                "sent": packets,
            });
            if let Some(attack) = tick.get("attack") {
                let stage = world
                    .destroying_blocks()
                    .get(&player.entity_id())
                    .map_or(-1, |progress| progress.stage);
                let mut watched = Vec::new();
                for place in attack["watch"].as_array().ok_or("watch must be an array")? {
                    watched.push(
                        world
                            .block_state_at(
                                i32_field(place, "x")?,
                                i32_field(place, "y")?,
                                i32_field(place, "z")?,
                            )
                            .unwrap_or(0),
                    );
                }
                described["breaking"] = json!({
                    "destroying": game_mode.is_destroying(),
                    "progress": format!("{:08x}", game_mode.destroy_progress().to_bits()),
                    "destroy_ticks": format!("{:08x}", game_mode.destroy_ticks().to_bits()),
                    "delay": game_mode.destroy_delay(),
                    "miss_time": game_mode.miss_time(),
                    "stage": stage,
                    "sequence": world.prediction_sequence(),
                    "attack_strength_ticker": player.attack_strength_ticker(),
                    "right_click_delay": game_mode.right_click_delay(),
                    "main_hand_count": hand_count(&world, 36),
                    "off_hand_count": hand_count(&world, 45),
                    "watched_states": watched,
                });
            }
            ticks.push(described);
        }
        match failure {
            None => out.push(json!({"label": label, "outcome": "ticked", "ticks": ticks})),
            Some(message) => out.push(json!({
                "label": label,
                "outcome": "unsupported",
                "message": message,
                "ticks": ticks,
            })),
        }
    }
    Ok(json!({"rows": out}))
}

fn entity_result(request: &Value) -> Result<Value, String> {
    use voxel_render::player_model::{Arm, PlayerModel, PlayerRenderState};
    if string(request, "operation")? != "player_model" {
        return Err("unknown entity operation".into());
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("entity rows must be an array")?;
    let bits = |row: &Value, key: &str| -> Result<f32, String> {
        let text = row[key].as_str().ok_or(format!("{key} must be a string"))?;
        u32::from_str_radix(text, 16)
            .map(f32::from_bits)
            .map_err(|error| error.to_string())
    };
    let flag = |row: &Value, key: &str| row[key].as_bool().unwrap_or(false);
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut model = PlayerModel::new(flag(row, "slim"));
        model.setup_anim(&PlayerRenderState {
            head_y_rot: bits(row, "head_y_rot")?,
            x_rot: bits(row, "x_rot")?,
            walk_position: bits(row, "walk_position")?,
            walk_speed: bits(row, "walk_speed")?,
            age_in_ticks: bits(row, "age_in_ticks")?,
            attack_time: bits(row, "attack_time")?,
            attack_arm: if row["attack_arm"].as_str() == Some("left") {
                Arm::Left
            } else {
                Arm::Right
            },
            swim_amount: bits(row, "swim_amount")?,
            crouching: flag(row, "crouching"),
            spectator: flag(row, "spectator"),
        });
        let hex = |value: f32| format!("{:08x}", value.to_bits());
        let mut quads: Vec<String> = model
            .render(&Default::default())
            .into_iter()
            .map(|(_, corners)| {
                corners
                    .iter()
                    .map(|vertex| {
                        vertex
                            .position
                            .iter()
                            .chain(&vertex.uv)
                            .chain(&vertex.normal)
                            .map(|value| hex(*value))
                            .collect::<String>()
                    })
                    .collect::<String>()
            })
            .collect();
        quads.sort();
        out.push(json!({"label": row["label"], "quads": quads}));
    }
    Ok(json!({"rows": out}))
}

/// The recorded drawing calls as the reference's oracle writes them.
fn gui_ops(ops: &[voxel_render::gui::DrawOp]) -> Vec<Value> {
    use voxel_render::gui::DrawOp;
    // Fills before text, then top to bottom, left to right and by what is
    // left to tell two apart: the reference layers what overlaps, an order
    // for the GPU and not for what is drawn.
    let key = |op: &&DrawOp| match op {
        DrawOp::Fill { rect, colour } => (
            0,
            rect.1,
            rect.0,
            rect.2,
            rect.3,
            String::new(),
            i64::from(*colour as i32),
        ),
        DrawOp::Sprite { rect, .. } => (1, rect.1, rect.0, rect.2, rect.3, String::new(), -1),
        DrawOp::Text {
            x, y, colour, runs, ..
        } => (
            2,
            *y as i32,
            *x as i32,
            0,
            0,
            runs.iter().map(|run| run.0.as_str()).collect::<String>(),
            i64::from(*colour as i32),
        ),
    };
    let mut sorted: Vec<&DrawOp> = ops.iter().collect();
    sorted.sort_by_key(key);
    sorted
        .into_iter()
        .map(|op| match op {
            DrawOp::Fill { rect, colour } => json!({
                "op": "fill", "x0": f64::from(rect.0), "y0": f64::from(rect.1),
                "x1": f64::from(rect.2), "y1": f64::from(rect.3),
                "color": *colour as i32,
            }),
            DrawOp::Sprite { name, rect } => json!({
                "op": "sprite", "name": name, "x0": f64::from(rect.0), "y0": f64::from(rect.1),
                "x1": f64::from(rect.2), "y1": f64::from(rect.3), "color": -1,
            }),
            DrawOp::Text {
                x,
                y,
                scale,
                colour,
                shadow,
                runs,
            } => json!({
                "op": "text", "x": f64::from(*x), "y": f64::from(*y), "scale": f64::from(*scale), "color": *colour as i32,
                "shadow": shadow,
                "runs": runs.iter().map(|(text, rgb, bold, italic)| {
                    let mut run = json!({"text": text, "bold": bold, "italic": italic});
                    if let Some(rgb) = rgb {
                        run["rgb"] = json!(rgb);
                    }
                    run
                }).collect::<Vec<_>>(),
            }),
        })
        .collect()
}

/// A clipboard for a test with nothing to copy.
struct NoClipboard;

impl voxel_render::gui::edit_box::Clipboard for NoClipboard {
    fn get(&mut self) -> String {
        String::new()
    }

    fn set(&mut self, _text: &str) {}
}

/// The screens' drawing, recorded and compared with the reference's own.
fn gui_result(request: &Value) -> Result<Value, String> {
    use voxel_render::gui::chat::ChatLog;
    use voxel_render::gui::font::Font;
    use voxel_render::gui::{GuiAtlas, GuiFrame};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/assets");
    let font = Font::load(&root).ok_or("font")?;
    let alt = Font::load_alt(&root).ok_or("alt font")?;
    let (atlas, _) = GuiAtlas::load(&root, &[&font, &alt]).ok_or("gui atlas")?;
    let rows = request["rows"]
        .as_array()
        .ok_or("gui rows must be an array")?;
    let mut out = Vec::new();
    for row in rows {
        let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
        let mut frame = GuiFrame {
            log: Some(Vec::new()),
            ..GuiFrame::default()
        };
        match string(request, "operation")? {
            "chat" => {
                let mut log = ChatLog::default();
                for message in row["messages"].as_array().ok_or("messages")? {
                    let tick = message["tick"].as_u64().unwrap_or(0);
                    while log.ticks() < tick {
                        log.tick();
                    }
                    let text = message["text"].as_str().ok_or("text")?;
                    log.add(&voxel_render::gui::chat::plain_runs(text), &font, false);
                }
                while log.ticks() < row["tick"].as_u64().unwrap_or(0) {
                    log.tick();
                }
                if let Some(scroll) = row["scroll"].as_i64() {
                    log.scroll_by(scroll as i32);
                }
                let focused = row["focused"].as_bool().unwrap_or(false);
                log.draw(&mut frame, &atlas, &font, int("height"), focused);
            }
            "suggestions" => {
                use voxel_render::gui::chat::ChatScreen;
                let width = int("width");
                let height = int("height");
                let text = row["text"].as_str().ok_or("text")?;
                let mut screen = ChatScreen::new(
                    std::sync::Arc::new(Font::load(&root).ok_or("font")?),
                    (width, height),
                    text,
                    Vec::new(),
                )
                .without_key();
                let matches = row["matches"]
                    .as_array()
                    .ok_or("matches")?
                    .iter()
                    .filter_map(|m| m.as_str().map(str::to_owned))
                    .collect();
                screen.suggested(
                    0,
                    row["start"].as_u64().unwrap_or(0) as usize,
                    row["length"].as_u64().unwrap_or(0) as usize,
                    matches,
                );
                let mut keys = vec![String::new()];
                keys.extend(
                    row["keys"]
                        .as_array()
                        .ok_or("keys")?
                        .iter()
                        .filter_map(|k| k.as_str().map(str::to_owned)),
                );
                let mut steps = Vec::new();
                for key in keys {
                    if !key.is_empty() {
                        let (glfw_key, shift) = match key.as_str() {
                            "tab" => (glfw::Key::Tab, false),
                            "shift+tab" => (glfw::Key::Tab, true),
                            "up" => (glfw::Key::Up, false),
                            "down" => (glfw::Key::Down, false),
                            "escape" => (glfw::Key::Escape, false),
                            _ => return Err("unknown key".into()),
                        };
                        screen.key_pressed(glfw_key, (false, shift), &mut NoClipboard);
                    }
                    let mut drawn = GuiFrame {
                        log: Some(Vec::new()),
                        ..GuiFrame::default()
                    };
                    screen.refocus();
                    screen.draw(&mut drawn, &atlas, (0, 0));
                    steps.push(json!({
                        "key": key,
                        "value": screen.value(),
                        "cursor": screen.cursor(),
                        "ops": gui_ops(&drawn.log.take().unwrap_or_default()),
                    }));
                }
                out.push(json!({"label": row["label"], "steps": steps}));
                continue;
            }
            "titles" => {
                use voxel_render::gui::titles::Titles;
                let mut titles = Titles::default();
                titles.set_times(int("fade_in"), int("stay"), int("fade_out"));
                let runs = |text: &str| voxel_render::gui::chat::plain_runs(text);
                if let Some(title) = row["title"].as_str() {
                    titles.set_title(runs(title));
                    for _ in 0..int("elapsed") {
                        titles.tick();
                    }
                }
                if let Some(subtitle) = row["subtitle"].as_str() {
                    titles.set_subtitle(runs(subtitle));
                }
                if let Some(overlay) = row["overlay"].as_str() {
                    titles.set_overlay(runs(overlay));
                    for _ in 0..int("overlay_elapsed") {
                        titles.tick();
                    }
                }
                let partial = row["partial"].as_f64().unwrap_or(0.0) as f32;
                titles.draw(
                    &mut frame,
                    &atlas,
                    &font,
                    (int("width"), int("height")),
                    partial,
                );
            }
            "tab_list" => {
                use voxel_render::gui::tab_list::{self, TabEntry};
                let runs = |key: &str| {
                    row[key]
                        .as_str()
                        .map(voxel_render::gui::chat::plain_runs)
                        .unwrap_or_default()
                };
                let mut entries: Vec<TabEntry> = row["players"]
                    .as_array()
                    .ok_or("players")?
                    .iter()
                    .map(|player| {
                        TabEntry::new(
                            player["name"].as_str().unwrap_or_default(),
                            player["display"]
                                .as_str()
                                .map(voxel_render::gui::chat::plain_runs),
                            player["mode"].as_i64().unwrap_or(0) as i32,
                            player["latency"].as_i64().unwrap_or(0) as i32,
                            player["order"].as_i64().unwrap_or(0) as i32,
                        )
                    })
                    .collect();
                tab_list::sort(&mut entries);
                tab_list::draw(
                    &mut frame,
                    &atlas,
                    &font,
                    int("width"),
                    &entries,
                    &runs("header"),
                    &runs("footer"),
                );
            }
            _ => return Err("unknown gui operation".into()),
        }
        out.push(json!({
            "label": row["label"],
            "ops": gui_ops(&frame.log.take().unwrap_or_default()),
        }));
    }
    Ok(json!({"rows": out}))
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
        "nbt" => nbt_result(&request)?,
        "resources" => resources_result(&request)?,
        "window" => window_result(&request)?,
        "atlas" => atlas_result(&request)?,
        "block" => block_result(&request)?,
        "image" => image_result(&request)?,
        "model" => model_result(&request)?,
        "json" => json_result(&request)?,
        "movement" => movement_result(&request)?,
        "entity" => entity_result(&request)?,
        "gui" => gui_result(&request)?,
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

/// Clicks in the player's own inventory menu, as the client predicts them
/// and sends them.
fn container_clicks_result(request: &Value) -> Result<Value, String> {
    use voxel_game::container_menu::{ClickType, ContainerMenu, MenuKind, Player, Smelter};
    use voxel_world::ClientItemStackState;
    let stack = |described: &Value| -> Result<ClientItemStackState, String> {
        let name = described["item"].as_str().ok_or("item must be a string")?;
        let id = voxel_game::item::id_of(name).ok_or_else(|| format!("unknown item {name}"))?;
        let count = described["count"]
            .as_i64()
            .ok_or("count must be a number")? as i32;
        Ok(ClientItemStackState::new(count, id))
    };
    let describe = |stack: Option<&ClientItemStackState>| -> Value {
        match stack {
            None => Value::Null,
            Some(stack) => json!({
                "item": voxel_data::item::registry::ITEMS[stack.item_registry_id() as usize].name,
                "count": stack.item_count(),
            }),
        }
    };
    let rows = request["rows"].as_array().ok_or("rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = row["label"].as_str().ok_or("label must be a string")?;
        let mut menu = match row.get("menu").and_then(Value::as_str) {
            None => ContainerMenu::default(),
            Some(name) => {
                let kind = match name.strip_prefix("minecraft:").unwrap_or(name) {
                    "crafting" => MenuKind::Crafting,
                    "anvil" => MenuKind::Anvil,
                    "enchantment" => MenuKind::Enchantment,
                    "grindstone" => MenuKind::Grindstone,
                    "loom" => MenuKind::Loom,
                    "smithing" => MenuKind::Smithing,
                    "cartography_table" => MenuKind::Cartography,
                    "stonecutter" => MenuKind::Stonecutter,
                    "furnace" => MenuKind::Furnace(Smelter::Furnace),
                    "blast_furnace" => MenuKind::Furnace(Smelter::BlastFurnace),
                    "smoker" => MenuKind::Furnace(Smelter::Smoker),
                    "generic_3x3" => MenuKind::Dispenser,
                    "hopper" => MenuKind::Hopper,
                    "shulker_box" => MenuKind::ShulkerBox,
                    chest => MenuKind::Chest(
                        chest
                            .strip_prefix("generic_9x")
                            .and_then(|rows| rows.parse().ok())
                            .ok_or_else(|| format!("unknown menu {name}"))?,
                    ),
                };
                let id = row["container_id"]
                    .as_i64()
                    .ok_or("container_id must be a number")? as i32;
                let mut menu = ContainerMenu::for_menu(kind, id, Vec::new(), None, 0);
                if let Some(sets) = row.get("recipe_sets").and_then(Value::as_object) {
                    // In the order the menu reads its sets.
                    menu.accepted = kind
                        .input_sets()
                        .iter()
                        .map(|name| {
                            std::sync::Arc::new(
                                sets.get(*name)
                                    .and_then(Value::as_array)
                                    .into_iter()
                                    .flatten()
                                    .filter_map(|item| {
                                        let name = item.as_str()?;
                                        voxel_game::item::id_of(&format!(
                                            "minecraft:{}",
                                            name.trim_start_matches("minecraft:")
                                        ))
                                    })
                                    .collect(),
                            )
                        })
                        .collect();
                }
                if let Some(inputs) = row.get("stonecutter_inputs").and_then(Value::as_array) {
                    menu.accepted = vec![std::sync::Arc::new(
                        inputs
                            .iter()
                            .filter_map(|item| {
                                let name = item.as_str()?;
                                voxel_game::item::id_of(&format!(
                                    "minecraft:{}",
                                    name.trim_start_matches("minecraft:")
                                ))
                            })
                            .collect(),
                    )];
                }
                if let Some(data) = row.get("data").and_then(Value::as_array) {
                    menu.data = data
                        .iter()
                        .filter_map(|value| Some(value.as_i64()? as i16))
                        .collect();
                }
                if let Some(level) = row.get("xp_level").and_then(Value::as_i64) {
                    menu.xp_level = level as i32;
                }
                menu.item_tags = std::sync::Arc::new(vanilla_item_tags()?);
                if let Some(inputs) = row.get("furnace_inputs").and_then(Value::as_array) {
                    menu.accepted = vec![std::sync::Arc::new(
                        inputs
                            .iter()
                            .filter_map(|name| voxel_game::item::id_of(name.as_str()?))
                            .collect(),
                    )];
                }
                menu
            }
        };
        for slot in row["slots"].as_array().ok_or("slots must be an array")? {
            let index = slot["slot"].as_u64().ok_or("slot must be a number")? as usize;
            menu.slots[index] = Some(stack(slot)?);
        }
        if row.get("carried").is_some() {
            menu.carried = Some(stack(&row["carried"])?);
        }
        menu.state_id = row["state_id"]
            .as_i64()
            .ok_or("state_id must be a number")? as i32;
        let selected = row["selected"]
            .as_u64()
            .ok_or("selected must be a number")? as usize;
        let player = Player {
            selected,
            infinite_materials: row["game_mode"].as_str() == Some("creative"),
        };
        let mut clicks = Vec::new();
        for click in row["clicks"].as_array().ok_or("clicks must be an array")? {
            let kind = ClickType::from_name(click["type"].as_str().ok_or("type must be a string")?)
                .ok_or("unknown click type")?;
            let (swings, body) = menu.handle_click(
                click["slot"].as_i64().ok_or("slot must be a number")? as i32,
                click["button"].as_i64().ok_or("button must be a number")? as i32,
                kind,
                player,
            );
            let mut sent = Vec::new();
            for _ in 0..swings {
                sent.push(json!({"type": "minecraft:swing", "bytes": "00"}));
            }
            sent.push(json!({"type": "minecraft:container_click", "bytes": hex(&body)}));
            let slots: Vec<Value> = menu
                .slots
                .iter()
                .map(|slot| describe(slot.as_ref()))
                .collect();
            let mut described = json!({"sent": sent, "slots": slots, "selected": selected});
            if menu.carried.is_some() {
                described["carried"] = describe(menu.carried.as_ref());
            }
            clicks.push(described);
        }
        out.push(json!({"label": label, "outcome": "clicked", "clicks": clicks}));
    }
    Ok(json!({"rows": out}))
}
