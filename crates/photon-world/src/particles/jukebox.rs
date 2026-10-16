//! Level events 1010 and 1011 (`LevelEventHandler.playJukeboxSong`, `stopJukeboxSong`): a record
//! played from a jukebox keeps sounding from its place until the jukebox says it is over.

use std::collections::HashMap;

use photon_core::random::RandomSource;

use super::engine::ParticleEngine;
use super::level_events::LevelEventContext;

/// A change to the songs playing from jukeboxes, for the sound system to carry out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JukeboxChange {
    /// A song starts at the middle of the block.
    Play { pos: (i32, i32, i32), sound: String },
    /// The song of the block ends.
    Stop { pos: (i32, i32, i32), sound: String },
}

/// The jukebox songs of the level's registry and the ones playing.
#[derive(Clone, Debug, Default)]
pub(super) struct Jukeboxes {
    /// The sound event of each song, by the registry's number for it.
    songs: Vec<Option<String>>,
    /// `playingJukeboxSongs`: the sound each place plays.
    playing: HashMap<(i32, i32, i32), String>,
    changes: Vec<JukeboxChange>,
}

impl ParticleEngine {
    /// The sound event of each `minecraft:jukebox_song` (none where it names no event), in the
    /// registry's order.
    pub fn set_jukebox_songs(&mut self, songs: Vec<Option<String>>) {
        self.jukeboxes.songs = songs;
    }

    /// The songs started and stopped since the last call, in order.
    pub fn take_jukebox_changes(&mut self) -> Vec<JukeboxChange> {
        std::mem::take(&mut self.jukeboxes.changes)
    }

    /// The events that need the level's blocks or a place to keep: 1504 and the jukebox's 1010
    /// and 1011. False for another.
    pub(super) fn block_level_event<F: FnMut() -> RandomSource>(
        &mut self,
        (event, pos, data): (i32, (i32, i32, i32), i32),
        context: &mut LevelEventContext<'_, F>,
    ) -> bool {
        match event {
            1504 => self.dripstone_drip(pos, context),
            1010 => self.play_jukebox_song(pos, data),
            1011 => self.stop_jukebox_song(pos),
            _ => return false,
        }
        true
    }

    /// `playJukeboxSong`: nothing where the registry has no song of that number.
    fn play_jukebox_song(&mut self, pos: (i32, i32, i32), song: i32) {
        let Some(sound) = usize::try_from(song)
            .ok()
            .and_then(|index| self.jukeboxes.songs.get(index))
            .cloned()
            .flatten()
        else {
            return;
        };
        self.stop_jukebox_song(pos);
        self.jukeboxes.playing.insert(pos, sound.clone());
        self.jukeboxes
            .changes
            .push(JukeboxChange::Play { pos, sound });
    }

    /// `stopJukeboxSong`.
    fn stop_jukebox_song(&mut self, pos: (i32, i32, i32)) {
        if let Some(sound) = self.jukeboxes.playing.remove(&pos) {
            self.jukeboxes
                .changes
                .push(JukeboxChange::Stop { pos, sound });
        }
    }
}
