//! The menus' sound: the engine and the music they share with the games.

use std::path::Path;

use photon_game::options::Options;

use super::{Menu, Screen};
use crate::music::{Carried, MenuMusic};
use crate::play::Client;
use crate::sound::SoundSlot;

/// The engine and the music manager the last game handed back, or new ones.
pub(super) fn take(client: &mut Client, root: &Path, options: &Options) -> (SoundSlot, MenuMusic) {
    let (sound, manager) = Carried::split(client.audio.take());
    (
        sound.unwrap_or_else(|| SoundSlot::open(root, options)),
        MenuMusic::new(manager),
    )
}

impl Menu {
    /// The engine and the music, which go to the game and come back from it.
    pub(super) fn into_audio(self) -> Carried {
        Carried {
            sound: self.sound,
            manager: self.music.into_manager(),
        }
    }

    /// The options as they stand now: while an options screen is open its sliders change them at
    /// once, and they are the menu's own only once saved.
    pub(super) fn live_options(&self) -> &photon_game::options::Options {
        if self.host.host_open() {
            &self.host.options
        } else {
            &self.options
        }
    }

    /// One frame of the menus' sound: the engine, then the menu music's ticks.
    pub(super) fn frame_audio(&mut self) {
        let options = self.live_options().clone();
        self.sound.frame_in_menu(&options);
        let loading =
            matches!(&self.screen, Screen::Connecting(_, task, _) if task.status.loading());
        if let Some(sound) = self.sound.as_mut() {
            self.music.tick(sound, &options, loading);
        }
    }
}
