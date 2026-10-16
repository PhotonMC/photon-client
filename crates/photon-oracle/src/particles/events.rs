//! The level events of `particle_engine` rows and the sounds they play.

use super::*;
use photon_world::local_player::LocalSound;

/// A `level_event` (the event, the block and the data) or `block_event` (the block) event, queued
/// in the world as the client would; false for an event of another kind.
pub(super) fn queues(event: &Value, world: &mut ClientWorld) -> Result<bool, String> {
    let pos = || -> Result<(i32, i32, i32), String> {
        Ok((
            i32_field(event, "x")?,
            i32_field(event, "y")?,
            i32_field(event, "z")?,
        ))
    };
    match string(event, "type")? {
        "level_event" => world.apply_level_event(
            i32_field(event, "event")?,
            pos()?,
            i32_field(event, "data")?,
        ),
        "explode" => {
            let float = |key: &str| -> Result<f32, String> {
                u32::from_str_radix(string(event, key)?, 16)
                    .map(f32::from_bits)
                    .map_err(|error| format!("{key}: {error}"))
            };
            let kind = |name: &str| {
                photon_data::particle_type::id(name).ok_or(format!("unknown particle {name}"))
            };
            let mut block_particles = Vec::new();
            for info in event["kinds"].as_array().ok_or("kinds must be an array")? {
                let float = |key: &str| -> Result<f32, String> {
                    u32::from_str_radix(string(info, key)?, 16)
                        .map(f32::from_bits)
                        .map_err(|error| format!("{key}: {error}"))
                };
                block_particles.push(photon_world::particles::ExplosionParticle {
                    type_id: kind(string(info, "particle")?)?,
                    data: photon_world::particles::ParticleData::Plain,
                    scaling: float("scaling")?,
                    speed: float("speed")?,
                    weight: i32_field(info, "weight")?,
                });
            }
            world.apply_explosion(photon_world::particles::Explosion {
                center: (
                    parsed64(event, "x")?,
                    parsed64(event, "y")?,
                    parsed64(event, "z")?,
                ),
                radius: float("radius")?,
                block_count: i32_field(event, "block_count")?,
                particle: (
                    kind(string(event, "particle")?)?,
                    photon_world::particles::ParticleData::Plain,
                ),
                sound: Some("minecraft:entity.generic.explode"),
                block_particles,
            });
        }
        "global_level_event" => {
            world.apply_global_level_event(i32_field(event, "event")?, pos()?);
        }
        "block_event" => {
            world.apply_note_block_event(pos()?);
        }
        "fireworks" => {
            use photon_world::particles::{FireworkExplosion, FireworkLaunch, FireworkShape};
            let colors = |list: &Value| -> Result<Vec<i32>, String> {
                list.as_array()
                    .ok_or("colors must be an array")?
                    .iter()
                    .map(|color| {
                        color
                            .as_i64()
                            .and_then(|color| i32::try_from(color).ok())
                            .ok_or_else(|| "a colour".to_owned())
                    })
                    .collect()
            };
            let mut explosions = Vec::new();
            for info in event["explosions"]
                .as_array()
                .ok_or("explosions must be an array")?
            {
                let shape = match string(info, "shape")? {
                    "SMALL_BALL" => FireworkShape::SmallBall,
                    "LARGE_BALL" => FireworkShape::LargeBall,
                    "STAR" => FireworkShape::Star,
                    "CREEPER" => FireworkShape::Creeper,
                    "BURST" => FireworkShape::Burst,
                    other => return Err(format!("unknown shape {other}")),
                };
                explosions.push(FireworkExplosion {
                    shape,
                    colors: colors(&info["colors"])?,
                    fade_colors: colors(&info["fades"])?,
                    trail: info["trail"].as_bool().ok_or("trail")?,
                    twinkle: info["twinkle"].as_bool().ok_or("twinkle")?,
                });
            }
            world.create_fireworks(FireworkLaunch {
                at: (
                    parsed64(event, "x")?,
                    parsed64(event, "y")?,
                    parsed64(event, "z")?,
                ),
                motion: (
                    parsed64(event, "xd")?,
                    parsed64(event, "yd")?,
                    parsed64(event, "zd")?,
                ),
                explosions,
            });
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// A sound as the reference's silent sound manager writes it: the sound, its kind's name, then
/// the volume, pitch and place as bits.
pub(super) fn sound_line(sound: &LocalSound) -> String {
    let source = sound.source.name();
    let (x, y, z) = sound.position;
    format!(
        "{},{source},{},{},{},{},{}{}",
        sound.event,
        bits32(sound.volume),
        bits32(sound.pitch),
        bits64(x),
        bits64(y),
        bits64(z),
        if sound.delay_ticks > 0 {
            format!(",{}", sound.delay_ticks)
        } else {
            String::new()
        }
    )
}

/// An `animate_tick` event: `count` pairs of picks around the block, the blocks' draws from a
/// random seeded as given; the sounds that are not ambient (the reference's sound manager here
/// leaves those out).
pub(super) fn animate_tick<F: FnMut() -> RandomSource>(
    event: &Value,
    engine: &mut photon_world::particles::ParticleEngine,
    context: &mut photon_world::particles::LevelEventContext<'_, F>,
) -> Result<Vec<String>, String> {
    let center = (
        i32_field(event, "x")?,
        i32_field(event, "y")?,
        i32_field(event, "z")?,
    );
    let count = usize::try_from(i32_field(event, "count")?).map_err(|error| error.to_string())?;
    let seed = event["seed"].as_i64().ok_or("seed must be an integer")?;
    let played = engine.animate_tick((center, count), &mut RandomSource::create(seed), context);
    Ok(played
        .iter()
        .filter(|sound| sound.source != photon_world::local_player::LocalSoundSource::Ambient)
        .map(sound_line)
        .collect())
}
