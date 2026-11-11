//! The play packets of the wire oracle that carry particles, an explosion or where the player
//! spawns: what they read from bytes.

use photon_data::particle_values::{Explosion, LevelParticles};
use photon_protocol::play::{
    Decode, Explode, LevelParticlesPacket, Login, Respawn, SpawnInfoObservation,
};
use serde_json::{Value, json};

fn bits32(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

fn bits64(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

fn particle_name(type_id: i32) -> &'static str {
    photon_data::particle_type::name(type_id).unwrap_or("unknown")
}

fn particles_fields(packet: &LevelParticles) -> Value {
    json!({
        "particle": particle_name(packet.type_id),
        "override": packet.override_limiter, "always_show": packet.always_show,
        "x": bits64(packet.at.0), "y": bits64(packet.at.1), "z": bits64(packet.at.2),
        "x_dist": bits32(packet.spread.0), "y_dist": bits32(packet.spread.1),
        "z_dist": bits32(packet.spread.2),
        "x_speed": bits32(packet.max_speed.0), "y_speed": bits32(packet.max_speed.1),
        "z_speed": bits32(packet.max_speed.2),
        "count": packet.count, "randomization": packet.randomization.id(),
    })
}

fn explosion_fields(explosion: &Explosion, knockback: Option<(f64, f64, f64)>) -> Value {
    let blocks: Vec<Value> = explosion
        .block_particles
        .iter()
        .map(|row| {
            json!({
                "particle": particle_name(row.type_id), "scaling": bits32(row.scaling),
                "speed": bits32(row.speed), "weight": row.weight,
            })
        })
        .collect();
    let mut fields = json!({
        "x": bits64(explosion.center.0), "y": bits64(explosion.center.1),
        "z": bits64(explosion.center.2), "radius": bits32(explosion.radius),
        "block_count": explosion.block_count, "particle": particle_name(explosion.particle.0),
        "block_particles": blocks, "play_sound": explosion.play_sound,
    });
    if let Some((x, y, z)) = knockback {
        fields["knockback"] = json!({"x": bits64(x), "y": bits64(y), "z": bits64(z)});
    }
    if let Some(sound) = explosion.sound {
        fields["sound"] = json!(sound);
    }
    fields
}

fn spawn_fields(spawn: &SpawnInfoObservation) -> Value {
    let mut fields = json!({
        "dimension_type": spawn.dimension_type_id, "dimension": spawn.dimension,
        "seed": spawn.seed.to_string(), "game_type": spawn.game_type,
        "previous_game_type": spawn.previous_game_type.map_or(-1, i32::from),
        "is_debug": spawn.is_debug, "is_flat": spawn.is_flat,
        "portal_cooldown": spawn.portal_cooldown, "sea_level": spawn.sea_level,
    });
    if let Some(death) = &spawn.last_death_location {
        fields["death_dimension"] = json!(death.dimension);
        fields["death_position"] = json!(death.position.to_string());
    }
    fields
}

/// The fields of a packet of this file, or none when the bytes are refused (or the packet is of
/// another kind: `Err`).
pub(super) fn fields(packet: &str, bytes: &[u8]) -> Result<Option<Value>, String> {
    let registries = crate::case_ids::registries();
    Ok(match packet {
        "level_particles" => LevelParticlesPacket::decode(bytes, &registries)
            .ok()
            .and_then(|decoded| decoded.0)
            .map(|packet| particles_fields(&packet)),
        "explode" => Explode::decode(bytes, &registries)
            .ok()
            .and_then(|decoded| decoded.0)
            .map(|(explosion, knockback)| explosion_fields(&explosion, knockback)),
        "respawn" => Respawn::decode(bytes, &registries)
            .ok()
            .map(|Respawn((spawn, keep))| {
                let mut fields = spawn_fields(&spawn);
                fields["data_to_keep"] = json!(i32::from(keep as i8));
                fields
            }),
        "login" => Login::decode(bytes, &registries).ok().map(|Login(login)| {
            let mut fields = spawn_fields(&login.spawn_info);
            fields["player_id"] = json!(login.player_id.0);
            fields["hardcore"] = json!(login.hardcore);
            fields["level_count"] = json!(login.level_count);
            fields["max_players"] = json!(login.max_players);
            fields["chunk_radius"] = json!(login.chunk_radius);
            fields["simulation_distance"] = json!(login.simulation_distance);
            fields["reduced_debug_info"] = json!(login.reduced_debug_info);
            fields["show_death_screen"] = json!(login.show_death_screen);
            fields["limited_crafting"] = json!(login.do_limited_crafting);
            fields["online_mode"] = json!(login.online_mode);
            fields["enforces_secure_chat"] = json!(login.enforces_secure_chat);
            fields
        }),
        other => return Err(format!("unknown play packet {other}")),
    })
}
