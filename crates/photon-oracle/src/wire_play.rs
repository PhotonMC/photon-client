//! The play packet operations of the wire oracle: what a packet reads, and the view the
//! rotation and look-at packets leave the player.

use photon_core::ids::EntityId;
use photon_core::math::Vec3;
use photon_protocol::play::{
    ChunksBiomes, CustomChatCompletions, DebugSamplePacket, Decode, EntityAnchor,
    GameTestHighlightPos, LowDiskSpaceWarning, MoveVehiclePacket, PlayerCombatEnd,
    PlayerCombatEnter, PlayerLookAtPacket, PlayerRotationPacket, PongResponsePacket,
    ProjectilePowerPacket, ServerDataPacket, TagQueryPacket, resolve_player_rotation,
};
use photon_world::ClientPlayerGameMode;
use photon_world::local_player::LocalPlayer;
use serde_json::{Value, json};

use crate::enum_ops::f64_field;
use crate::error::f32_field;
use crate::hex::{decode_hex, string};

fn bits32(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

fn bits64(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn flag(request: &Value, name: &str) -> Result<bool, String> {
    request[name]
        .as_bool()
        .ok_or_else(|| format!("{name} must be a boolean"))
}

fn anchor_index(anchor: EntityAnchor) -> i32 {
    match anchor {
        EntityAnchor::Feet => 0,
        EntityAnchor::Eyes => 1,
    }
}

fn fields_of(packet: &str, bytes: &[u8]) -> Result<Option<Value>, String> {
    let registries = crate::case_ids::registries();
    Ok(match packet {
        "player_rotation" => PlayerRotationPacket::decode(bytes, &registries)
            .ok()
            .map(|p| {
                json!({
                    "y_rot": bits32(p.y_rot), "relative_y": p.relative_y,
                    "x_rot": bits32(p.x_rot), "relative_x": p.relative_x,
                })
            }),
        "player_look_at" => PlayerLookAtPacket::decode(bytes, &registries)
            .ok()
            .map(|p| {
                let mut fields = json!({
                    "from": anchor_index(p.from),
                    "x": bits64(p.at.0), "y": bits64(p.at.1), "z": bits64(p.at.2),
                    "at_entity": p.entity.is_some(),
                });
                if let Some((id, anchor)) = p.entity {
                    fields["entity"] = json!(id.0);
                    fields["to"] = json!(anchor_index(anchor));
                }
                fields
            }),
        "move_vehicle" => MoveVehiclePacket::decode(bytes, &registries).ok().map(|p| {
            json!({
                "x": bits64(p.position.0), "y": bits64(p.position.1), "z": bits64(p.position.2),
                "y_rot": bits32(p.y_rot), "x_rot": bits32(p.x_rot),
            })
        }),
        "projectile_power" => ProjectilePowerPacket::decode(bytes, &registries)
            .ok()
            .map(|p| json!({"entity": p.entity.0, "power": bits64(p.acceleration_power)})),
        other => return notice_fields(other, bytes),
    })
}

/// The fields of the packets that tell the client something rather than move the player.
fn notice_fields(packet: &str, bytes: &[u8]) -> Result<Option<Value>, String> {
    let registries = crate::case_ids::registries();
    Ok(match packet {
        "chunks_biomes" => ChunksBiomes::decode(bytes, &registries).ok().map(|p| {
            let chunks: Vec<Value> =
                p.0.iter()
                    .map(|c| json!({"x": c.x, "z": c.z, "buffer": c.buffer.iter().map(|b| format!("{b:02x}")).collect::<String>()}))
                    .collect();
            json!({"chunks": chunks})
        }),
        "server_data" => ServerDataPacket::decode(bytes, &registries).ok().map(|p| {
            let motd = match &p.motd.content {
                photon_core::text::Content::Literal(text) => text.clone(),
                photon_core::text::Content::Translatable { key, .. } => key.clone(),
            };
            let mut fields = json!({"motd": motd});
            if let Some(icon) = &p.icon {
                fields["icon"] = json!(icon.iter().map(|b| format!("{b:02x}")).collect::<String>());
            }
            fields
        }),
        "tag_query" => TagQueryPacket::decode(bytes, &registries).ok().map(|p| {
            let mut fields = json!({"transaction": p.transaction_id});
            if let Some(tag) = p.tag {
                let root = photon_core::nbt::FieldValue::Compound(tag.clone());
                fields["compact"] = json!(photon_core::snbt::compact(&root));
                fields["pretty"] = json!(photon_core::snbt::pretty(&root));
                fields["entity"] =
                    json!(photon_core::snbt::pretty_compound_without(&tag, &["UUID", "Pos"]));
            }
            fields
        }),
        "custom_chat_completions" => CustomChatCompletions::decode(bytes, &registries).ok().map(|p| {
            json!({"action": p.action as i32, "entries": p.entries})
        }),
        other => return combat_fields(other, bytes),
    })
}

/// The fields of the packets that carry no more than a notice of a fight or of the disk.
fn combat_fields(packet: &str, bytes: &[u8]) -> Result<Option<Value>, String> {
    let registries = crate::case_ids::registries();
    Ok(match packet {
        "pong_response" => PongResponsePacket::decode(bytes, &registries)
            .ok()
            .map(|p| json!({"time": p.time.to_string()})),
        "debug_sample" => DebugSamplePacket::decode(bytes, &registries).ok().map(|p| {
            let sample: Vec<String> = p.sample.iter().map(i64::to_string).collect();
            json!({"sample": sample, "type": 0})
        }),
        "game_test_highlight" => GameTestHighlightPos::decode(bytes, &registries)
            .ok()
            .map(|p| {
                let short = |(x, y, z): (i32, i32, i32)| format!("{x}, {y}, {z}");
                json!({"absolute": short(p.absolute), "relative": short(p.relative)})
            }),
        "low_disk_space" => LowDiskSpaceWarning::decode(bytes, &registries)
            .ok()
            .map(|_| json!({})),
        "combat_enter" => PlayerCombatEnter::decode(bytes, &registries)
            .ok()
            .map(|_| json!({})),
        "combat_end" => PlayerCombatEnd::decode(bytes, &registries)
            .ok()
            .map(|p| json!({"duration": p.duration})),
        other => return Err(format!("unknown play packet {other}")),
    })
}

/// `play_packet`: the fields a packet reads from bytes.
pub(crate) fn play_packet_result(request: &Value) -> Result<Value, String> {
    let bytes = decode_hex(string(request, "hex")?)?;
    Ok(match fields_of(string(request, "packet")?, &bytes)? {
        Some(fields) => json!({"outcome": "value", "fields": fields}),
        None => json!({"outcome": "error"}),
    })
}

fn player_at(position: Vec3, y_rot: f32, x_rot: f32) -> LocalPlayer {
    LocalPlayer::new(
        EntityId(1),
        position,
        y_rot,
        x_rot,
        ClientPlayerGameMode::Survival,
        false,
    )
}

/// `rotate_player`: the view the server's rotation packet leaves.
pub(crate) fn rotate_player_result(request: &Value) -> Result<Value, String> {
    let mut player = player_at(
        Vec3::ZERO,
        f32_field(request, "current_y")?,
        f32_field(request, "current_x")?,
    );
    // The row's start goes through the same setters the reference's does.
    let (y, x) = player.rotation();
    player.set_view(y, x);
    let packet = PlayerRotationPacket {
        y_rot: f32_field(request, "y_rot")?,
        relative_y: flag(request, "relative_y")?,
        x_rot: f32_field(request, "x_rot")?,
        relative_x: flag(request, "relative_x")?,
    };
    let (y_rot, x_rot) = resolve_player_rotation(packet, player.rotation());
    player.set_view(y_rot, x_rot);
    let (y_rot, x_rot) = player.rotation();
    Ok(json!({"y_rot": bits32(y_rot), "x_rot": bits32(x_rot)}))
}

/// `look_at`: the view `Entity.lookAt` leaves.
pub(crate) fn look_at_result(request: &Value) -> Result<Value, String> {
    let at = |name: &str| f64_field(request, name);
    let mut player = player_at(Vec3::new(at("x")?, at("y")?, at("z")?), 0.0, 0.0);
    player.look_at(
        flag(request, "eyes")?,
        Vec3::new(at("at_x")?, at("at_y")?, at("at_z")?),
    );
    let (y_rot, x_rot) = player.rotation();
    Ok(json!({"y_rot": bits32(y_rot), "x_rot": bits32(x_rot)}))
}
