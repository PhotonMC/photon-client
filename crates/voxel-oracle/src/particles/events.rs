//! The level events of `particle_engine` rows and the sounds they play.

use super::*;
use voxel_world::local_player::LocalSound;

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
        "block_event" => {
            world.apply_note_block_event(pos()?);
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
        "{},{source},{},{},{},{},{}",
        sound.event,
        bits32(sound.volume),
        bits32(sound.pitch),
        bits64(x),
        bits64(y),
        bits64(z)
    )
}

/// An `animate_tick` event: `count` pairs of picks around the block, the blocks' draws from a
/// random seeded as given; the sounds that are not ambient (the reference's sound manager here
/// leaves those out).
pub(super) fn animate_tick<F: FnMut() -> RandomSource>(
    event: &Value,
    engine: &mut voxel_world::particles::ParticleEngine,
    context: &mut voxel_world::particles::LevelEventContext<'_, F>,
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
        .filter(|sound| sound.source != voxel_world::local_player::LocalSoundSource::Ambient)
        .map(sound_line)
        .collect())
}
