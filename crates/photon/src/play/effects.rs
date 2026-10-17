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
        route_level_events(
            &mut self.session,
            (&mut self.particles, &mut self.gui, self.sound.as_mut()),
        );
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

/// The level events waiting (the packets', or the ones the level's own tick queued): their particles
/// are made, the sounds they play and the songs they start go to the sound system and the screen.
pub(super) fn route_level_events(
    session: &mut photon_net::connection::PlaySession,
    (particles, gui, sound): (
        &mut crate::particles::Particles,
        &mut crate::gui::Gui,
        Option<&mut crate::sound::SoundSystem>,
    ),
) {
    let events = session.take_level_events();
    let event_sounds = particles.level_events(session, events);
    let songs = particles.take_jukebox_changes();
    for change in &songs {
        match *change {
            JukeboxChange::Play { song, .. } => gui.show_now_playing(song),
            JukeboxChange::Notify { pos, playing } => {
                session.set_record_playing_nearby(pos, playing);
            }
            JukeboxChange::Stop { .. } => {}
        }
    }
    if let Some(sound) = sound {
        sound.handle_level_events(event_sounds, songs);
    }
}

/// Where the local player is for the particles: the block they stand in, and the place the
/// pickups of items head for.
pub(super) fn player_place(
    session: &photon_net::connection::PlaySession,
) -> crate::particles::PlayerPlace {
    let local = session.player().map(|player| {
        let at = player.position();
        photon_world::particles::pickup_target((at.x, at.y, at.z), player.eye_height())
    });
    let block = session.player().map(|player| {
        let at = player.position();
        (
            at.x.floor() as i32,
            at.y.floor() as i32,
            at.z.floor() as i32,
        )
    });
    (block, local)
}

/// The blocks about the player animate, and the particles tick; the sounds the blocks played.
pub(super) fn tick_blocks(
    session: &photon_net::connection::PlaySession,
    (particles, sky): (&mut crate::particles::Particles, &crate::world::sky::Sky),
    sound: Option<&mut crate::sound::SoundSystem>,
) {
    let place = player_place(session);
    let ambience = sky.ambience(session);
    let block_sounds = particles.tick(session, place, &ambience);
    if let Some(sound) = sound {
        sound.handle_local(block_sounds);
    }
}
