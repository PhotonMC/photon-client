//! What the packets brought since the last frame: particles, level events and sounds.

use super::*;
use photon_world::particles::JukeboxChange;

impl Play {
    /// `handlePortalTransitionEffect`: a screen that is not allowed in a portal (a pause
    /// screen is, as are the chat, the death screen and the credits) closes, a container with its
    /// close packet. The screens that carry packets of their own when they close (signs, books,
    /// dialogs that do not pause) are not closed yet.
    fn close_screens_in_portal(&mut self) {
        if !self.session.take_portal_screen_close() {
            return;
        }
        if self.controls.screen.is_some() {
            let creative_tab = &mut self.controls.creative_tab;
            let closed = crate::screens::close_inventory(
                &mut self.session,
                &mut self.surface,
                &mut self.controls.screen,
                creative_tab,
            );
            if closed.is_err() {
                self.controls.screen = None;
            }
        }
        if self.controls.close_menus() {
            self.surface.set_cursor_captured(true);
        }
    }
    /// The totem animation's triangles, in the hand pass's space, while it shows
    /// (`ScreenEffectRenderer.submit`, hidden with the HUD).
    pub(super) fn activation_vertices(&mut self) -> Vec<photon_render::scene::Vertex> {
        let partial = (self.accumulated.as_secs_f64() / TICK_LENGTH.as_secs_f64()) as f32;
        let shown = self.camera.activation.shown(partial, self.aspect);
        match shown {
            Some((stack, pose)) if !self.controls.hud_hidden => {
                self.item_layers
                    .activation(stack, pose, (&self.block_renderer, &self.biome_tints))
            }
            _ => Vec::new(),
        }
    }

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
        if let Some(stack) = self.session.take_item_activation() {
            // `GameRenderer.random`: unseeded.
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos() as i64);
            let mut random = photon_core::random::RandomSource::create(nanos);
            let offsets = (random.next_float(), random.next_float());
            self.camera.activation.display(stack, offsets);
        }
        self.close_screens_in_portal();
        // `ClientLevel.tick`: the End's flash sounds, but not over the credits.
        if let Some(angles) = self.sky.take_flash_start()
            && self.controls.credits.is_none()
            && let Some(sound) = self.sound.as_mut()
        {
            sound.play_end_flash(angles);
        }
        let tracked = self.session.take_tracking_emitters();
        let tracked_sounds = self.particles.tracking(&self.session, tracked);
        if let Some(sound) = self.sound.as_mut() {
            sound.handle_local(tracked_sounds);
        }
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
