//! The level events of `particle_engine` rows and the sounds they play.

use super::*;
use photon_world::local_player::LocalSound;

/// A `block_event`: the block at the place takes an event and its data, each a byte.
fn queue_block_event(event: &Value, world: &mut ClientWorld) -> Result<(), String> {
    let byte =
        |key: &str| u8::try_from(i32_field(event, key)?).map_err(|_| format!("{key} is a byte"));
    let pos = (
        i32_field(event, "x")?,
        i32_field(event, "y")?,
        i32_field(event, "z")?,
    );
    world.apply_block_event(pos, byte("b0")?, byte("b1")?);
    Ok(())
}

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
        "block_event" => queue_block_event(event, world)?,
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

/// The registries of the vanilla data pack, entries named in the order a vanilla server
/// synchronises them.
fn vanilla_registries() -> &'static photon_game::registries::Registries {
    static REGISTRIES: std::sync::OnceLock<photon_game::registries::Registries> =
        std::sync::OnceLock::new();
    REGISTRIES.get_or_init(|| {
        let data_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.data/minecraft/assets/jar/data");
        photon_game::registries::Registries::vanilla(&data_root)
    })
}

/// An engine whose jukebox songs are the ones the vanilla data pack holds.
pub(super) fn engine_with_songs(
    random: RandomSource,
    resources: photon_world::particles::ParticleResources,
    row: &Value,
) -> photon_world::particles::ParticleEngine {
    let mut engine = photon_world::particles::ParticleEngine::new(random, resources);
    let songs = vanilla_registries().jukebox_songs.iter();
    engine.set_jukebox_songs(songs.map(|song| song.sound_event.clone()).collect());
    // The block a creative player's marker item stands for, when the row has one.
    let marker = row["marker_block"]
        .as_str()
        .and_then(photon_data::block::registry::block_index);
    engine.set_marker_block(marker);
    engine
}

/// The key of the name a song shows on the hud.
fn song_name_key(song: usize) -> String {
    let description = vanilla_registries().jukebox_songs[song]
        .description
        .as_ref();
    match photon_game::text::Component::from_json(description.expect("a song has a name")) {
        Some(photon_game::text::Component {
            content: photon_game::text::Content::Translatable { key, .. },
            ..
        }) => key,
        _ => String::new(),
    }
}

/// What an event played, as the reference's sound manager and hud are asked for it: the sounds,
/// then the songs jukeboxes started and stopped.
pub(super) fn heard(
    played: &[LocalSound],
    (engine, world): (
        &mut photon_world::particles::ParticleEngine,
        &mut photon_world::ClientWorld,
    ),
) -> Vec<String> {
    use photon_world::particles::JukeboxChange;
    let mut lines: Vec<String> = played.iter().map(sound_line).collect();
    let place =
        |pos: (i32, i32, i32)| [pos.0, pos.1, pos.2].map(|block| bits64(f64::from(block) + 0.5));
    for change in engine.take_jukebox_changes() {
        match change {
            JukeboxChange::Play { pos, sound, song } => {
                let [x, y, z] = place(pos);
                let (volume, pitch) = (bits32(4.0), bits32(1.0));
                lines.push(format!("{sound},record,{volume},{pitch},{x},{y},{z}"));
                lines.push(format!("now_playing,{}", song_name_key(song)));
            }
            JukeboxChange::Stop { pos, sound } => {
                let [x, y, z] = place(pos);
                lines.push(format!("stop,{sound},{x},{y},{z}"));
            }
            JukeboxChange::Notify { pos, playing } => world.set_record_playing_nearby(pos, playing),
        }
    }
    lines
}

/// The block entities of the level tick, for the rows that ask for it, as they do after the
/// entities and before the level's own tick; then how many particles wait to join the engine.
pub(super) fn tick_block_entities(
    world: &mut ClientWorld,
    row: &Value,
    engine: &photon_world::particles::ParticleEngine,
) -> usize {
    if row["tick_block_entities"].as_bool() == Some(true) {
        world.tick_block_entities();
    }
    engine.pending_count()
}

/// What a tick played, then whether each parrot of the scene dances and what the block entities
/// the row watches hold.
pub(super) fn ticked_lines(
    ticked: &[LocalSound],
    (world, row): (&photon_world::ClientWorld, &Value),
) -> Vec<String> {
    let mut lines: Vec<String> = ticked.iter().map(sound_line).collect();
    lines.extend(super::extras::party_lines(world));
    lines.extend(super::extras::watch_lines(world, row));
    lines.extend(super::extras::region_lines(world, row));
    lines
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
