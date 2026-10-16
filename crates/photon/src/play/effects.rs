//! What the packets brought since the last frame: particles, level events and sounds.

use super::*;
use photon_world::particles::JukeboxChange;

impl Play {
    /// The level's particles, events and sounds the packets brought since the last frame.
    pub(super) fn route_effects(&mut self) {
        // The particles option is not kept yet: all of them.
        let level_particles = self.session.take_level_particles();
        let pickups = self.session.take_item_pickups();
        self.particles.add_pickups(pickups);
        let events = self.session.take_level_events();
        let event_sounds = self.particles.level_events(&self.session, events);
        let songs = self.particles.take_jukebox_changes();
        for change in &songs {
            match *change {
                JukeboxChange::Play { song, .. } => self.gui.show_now_playing(song),
                JukeboxChange::Notify { pos, playing } => {
                    self.session.set_record_playing_nearby(pos, playing);
                }
                JukeboxChange::Stop { .. } => {}
            }
        }
        if let Some(sound) = self.sound.as_mut() {
            sound.handle_level_events(event_sounds, songs);
        }
        self.particles.receive(
            level_particles,
            self.session.world(),
            photon_world::particles::ParticleStatus::All,
        );
        let commands = self.session.take_sounds();
        self.sounds_heard += commands.len();
        if let Some(sound) = self.sound.as_mut() {
            sound.handle(commands, self.session.world());
        }
    }
}
