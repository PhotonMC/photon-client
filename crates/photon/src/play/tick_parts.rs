//! The parts of one client tick, each run by a system of the `Tick` schedule in the order the
//! tick always had.

use super::*;

impl Play {
    /// The tick's first part: the sound engine, and the open screen with what it asked for.
    pub(super) fn tick_interface(&mut self) -> Result<()> {
        self.tick_sound();
        if let Some(screen) = self.controls.screen.as_mut() {
            screen.tick(self.session.active_menu());
        }
        // What the screen asked for as it was drawn (a bundle the mouse left).
        self.controls.drain_screen_actions(&mut self.session)
    }

    /// The tick's keys, handled with the aim they point at, the items they used and the hands.
    pub(super) fn tick_input(&mut self, held: &HeldKeys) -> Result<()> {
        // A new level starts without it: the world reads its rain from the block materials.
        if let Some(materials) = self.particles.materials() {
            self.session.set_precipitation(materials);
        }
        let session = &mut self.session;
        // `Minecraft.handleKeybinds`: the attack key is spent before the
        // player moves, aiming the way `GameRenderer.pick` aims.
        let aim = session
            .player()
            .map_or(photon_game::game_mode::Aim::Miss, |player| {
                crate::view::camera_aim(session, player, 1.0)
            });
        // `handleKeybinds` does not run under an overlay: the clicks stay counted.
        let grabbed = self.controls.captured || self.script.is_some();
        if !self.gui.is_loading() {
            session.keys(self.controls.game_keys(grabbed), aim)?;
        }
        for hand in session.take_items_used() {
            self.item_in_hand.item_used(hand);
        }
        self.tutorial_before_tick(&aim, held);
        let session = &mut self.session;
        // After the keys and before the player ticks.
        tick_hands(&mut self.item_in_hand, session, &self.block_renderer);
        Ok(())
    }

    /// The local player's own tick, and what follows from it.
    pub(super) fn tick_local_player(&mut self, held: &HeldKeys) -> Result<()> {
        self.session.tick(held.input())?;
        self.tutorial_tick();
        self.tick_player_sounds();
        Ok(())
    }

    /// What the tick leaves in the world to see and hear: animated textures, the level's own
    /// events, breaking and block particles.
    pub(super) fn tick_effects(&mut self) {
        let session = &mut self.session;
        self.animated_atlas.tick();
        // What the level's own tick queued (a sulfur block's gas) joins the particles in this tick.
        super::effects::route_level_events(
            session,
            (&mut self.particles, &mut self.gui, self.sound.as_mut()),
        );
        let breaking = session.take_breaking_effects();
        self.particles.breaking(session.world(), breaking);
        super::effects::tick_blocks(
            session,
            (&mut self.particles, &self.sky),
            self.sound.as_mut(),
        );
    }

    /// The camera and the screens, the sky, and the tick count.
    pub(super) fn tick_view(&mut self) -> Result<()> {
        super::camera::tick_view(
            (&mut self.camera, &mut self.gui),
            &self.session,
            (&self.sky, &self.controls),
        );
        self.tick_screens()?;
        self.sky.tick(&self.session, &self.controls.options);
        self.ticks += 1;
        Ok(())
    }
}
