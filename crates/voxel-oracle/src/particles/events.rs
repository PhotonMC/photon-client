//! The level events of `particle_engine` rows and the sounds they play.

use super::*;
use voxel_world::local_player::LocalSound;

/// A `level_event` event: the event, the block and the data.
pub(super) fn level_event_of(event: &Value) -> Result<voxel_world::LevelEvent, String> {
    Ok(voxel_world::LevelEvent {
        event: i32_field(event, "event")?,
        pos: (
            i32_field(event, "x")?,
            i32_field(event, "y")?,
            i32_field(event, "z")?,
        ),
        data: i32_field(event, "data")?,
    })
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
