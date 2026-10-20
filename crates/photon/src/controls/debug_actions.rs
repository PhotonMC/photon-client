//! The debug key combinations that act on the client or the player (`KeyboardHandler.handleDebugKeys`):
//! F3+C copies the location, F3+N switches between spectator and the mode before it.

use super::game_mode_switcher::debug_feedback;
use super::*;
use photon_gui::chat::Tag;
use photon_world::ClientPlayerGameMode;

/// F3+C's clipboard text: a `/tp` in the player's dimension, each number with two decimals.
fn location_command(dimension: &str, position: (f64, f64, f64), rotation: (f32, f32)) -> String {
    let fixed = photon_gui::hud::debug::fixed;
    format!(
        "/execute in {dimension} run tp @s {} {} {} {} {}",
        fixed(position.0, 2),
        fixed(position.1, 2),
        fixed(position.2, 2),
        fixed(f64::from(rotation.0), 2),
        fixed(f64::from(rotation.1), 2),
    )
}

/// `Minecraft.pauseIfInactive` for one frame: whether the pause menu opens, and whether the window
/// counts as active now. A window without focus pauses once it has been so for more than half a
/// second; a screen that is up stops that, without counting as activity.
fn pause_due(
    (focused, option_on): (bool, bool),
    (since, now): (Instant, Instant),
    screen_open: bool,
) -> (bool, bool) {
    if focused || !option_on {
        return (false, true);
    }
    (
        !screen_open && now - since > Duration::from_millis(500),
        false,
    )
}

/// The key a script's letter or digit stands for.
fn scripted_key(letter: char) -> Option<glfw::Key> {
    use glfw::Key::*;
    let letters = [
        A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    ];
    let digits = [Num1, Num2, Num3, Num4];
    match letter {
        'a'..='z' => letters.get(letter as usize - 'a' as usize).copied(),
        '1'..='4' => digits.get(letter as usize - '1' as usize).copied(),
        _ => None,
    }
}

impl Controls {
    /// A script's `debug/K` step: F3 pressed, the key pressed and let go, F3 let go, each as an
    /// event of the window, so the routing is the real one.
    pub(super) fn scripted_debug_key(
        &mut self,
        letter: char,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Key, Modifiers, WindowEvent};
        let Some(key) = scripted_key(letter) else {
            return Ok(());
        };
        for (key, action) in [
            (Key::F3, Action::Press),
            (key, Action::Press),
            (key, Action::Release),
            (Key::F3, Action::Release),
        ] {
            let event = WindowEvent::Key(key, 0, action, Modifiers::empty());
            self.handle(event, session, surface, assets)?;
        }
        Ok(())
    }

    /// A key pressed with F3 held and no screen up: the ones that act here, and whether one did.
    pub(super) fn debug_world_key(
        &mut self,
        event: &glfw::WindowEvent,
        (session, surface, assets): (&mut PlaySession, &mut WindowSurface, &ScreenAssets),
    ) -> std::result::Result<bool, Box<dyn std::error::Error>> {
        let glfw::WindowEvent::Key(key, _, glfw::Action::Press, _) = *event else {
            return Ok(false);
        };
        if !self.debug.down {
            return Ok(false);
        }
        if self.run_debug_key(key, (session, surface, assets))? {
            return Ok(true);
        }
        // Escape still pauses; the other keys are the debug keys' and not the game's.
        Ok(key != glfw::Key::Escape && DebugKeys::acts(key, session.reduced_debug_info()))
    }

    /// The debug keys that act in the world, and whether `key` was one.
    fn run_debug_key(
        &mut self,
        key: glfw::Key,
        (session, surface, assets): (&mut PlaySession, &mut WindowSurface, &ScreenAssets),
    ) -> std::result::Result<bool, Box<dyn std::error::Error>> {
        match key {
            glfw::Key::F4 => self.debug_switch_game_mode(session, surface, assets),
            glfw::Key::C => self.debug_copy_location(session, surface, assets),
            glfw::Key::A => {
                self.reload_chunks = true;
                self.debug_say("debug.reload_chunks.message", assets);
            }
            glfw::Key::B => self.debug_hitboxes(session, assets),
            glfw::Key::G => self.debug_chunk_borders(session, assets),
            glfw::Key::H => self.debug_advanced_tooltips(assets),
            glfw::Key::P => self.debug_pause_on_focus_loss(assets),
            glfw::Key::N => self.debug_creative_spectator(session, assets)?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// The `[Debug]:` line in the chat.
    pub(super) fn debug_say(&mut self, key: &str, assets: &ScreenAssets) {
        self.chat_log.add_component(
            &debug_feedback(key),
            &assets.lang,
            (None, Some(Tag::SystemSinglePlayer)),
            &assets.creative.font,
            false,
        );
    }

    /// `Minecraft.pauseIfInactive`: a window that has been without focus for half a second opens
    /// the pause menu, unless a screen is up or the option is off.
    pub fn pause_if_inactive(
        &mut self,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
        now: Instant,
    ) {
        let since = *self.last_active.get_or_insert(now);
        let (pause, active) = pause_due(
            (surface.is_focused(), self.options.pause_on_lost_focus),
            (since, now),
            self.any_screen_open(),
        );
        if active {
            self.last_active = Some(now);
        }
        if pause {
            self.open_pause(surface, assets);
        }
    }

    /// F3+G: the chunk borders entry is toggled, unless the player is absent or has
    /// reduced debug info.
    fn debug_chunk_borders(&mut self, session: &PlaySession, assets: &ScreenAssets) {
        if session.player().is_none() || session.reduced_debug_info() {
            return;
        }
        let on = self.debug.entries.toggle_status("chunk_borders");
        let key = format!("debug.chunk_boundaries.{}", if on { "on" } else { "off" });
        self.debug_say(&key, assets);
    }

    /// F3+B: the entity hitboxes entry is toggled, unless the player is absent or has reduced
    /// debug info.
    fn debug_hitboxes(&mut self, session: &PlaySession, assets: &ScreenAssets) {
        if session.player().is_none() || session.reduced_debug_info() {
            return;
        }
        let on = self.debug.entries.toggle_status("entity_hitboxes");
        let key = format!("debug.show_hitboxes.{}", if on { "on" } else { "off" });
        self.debug_say(&key, assets);
    }

    /// F3+H: the advanced tooltips are toggled and saved.
    fn debug_advanced_tooltips(&mut self, assets: &ScreenAssets) {
        self.options.advanced_item_tooltips = !self.options.advanced_item_tooltips;
        let key = if self.options.advanced_item_tooltips {
            "debug.advanced_tooltips.on"
        } else {
            "debug.advanced_tooltips.off"
        };
        self.debug_say(key, assets);
        self.save_options();
    }

    /// F3+P: the focus pause is toggled and saved.
    fn debug_pause_on_focus_loss(&mut self, assets: &ScreenAssets) {
        self.options.pause_on_lost_focus = !self.options.pause_on_lost_focus;
        self.save_options();
        let key = if self.options.pause_on_lost_focus {
            "debug.pause_focus.on"
        } else {
            "debug.pause_focus.off"
        };
        self.debug_say(key, assets);
    }

    /// F3+C, with the full debug info: the location goes to the clipboard.
    pub(super) fn debug_copy_location(
        &mut self,
        session: &PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let Some(player) = session.player() else {
            return;
        };
        if session.reduced_debug_info() {
            return;
        }
        self.debug_say("debug.copy_location.message", assets);
        let position = player.position();
        let command = location_command(
            session.world().dimension_key(),
            (position.x, position.y, position.z),
            player.rotation(),
        );
        surface.set_clipboard(&command);
    }

    /// F3+N: a game master goes to spectator, and a spectator back to the mode before it
    /// (creative when there is none).
    pub(super) fn debug_creative_spectator(
        &mut self,
        session: &mut PlaySession,
        assets: &ScreenAssets,
    ) -> Result {
        if session.player().is_none() || !session.may_change_game_mode() {
            self.debug_say("debug.creative_spectator.error", assets);
        } else if session.game_mode() != ClientPlayerGameMode::Spectator {
            session.change_game_mode(ClientPlayerGameMode::Spectator)?;
        } else {
            let back = session
                .previous_game_mode()
                .unwrap_or(ClientPlayerGameMode::Creative);
            session.change_game_mode(back)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{location_command, pause_due};
    use std::time::{Duration, Instant};

    #[test]
    fn a_window_pauses_after_half_a_second_without_focus() {
        let start = Instant::now();
        let at = |ms| start + Duration::from_millis(ms);
        // Focus, or the option being off, keeps the clock running and never pauses.
        assert_eq!(
            pause_due((true, true), (start, at(9000)), false),
            (false, true)
        );
        assert_eq!(
            pause_due((false, false), (start, at(9000)), false),
            (false, true)
        );
        // Exactly 500 ms is not yet over; 501 is.
        assert_eq!(
            pause_due((false, true), (start, at(500)), false),
            (false, false)
        );
        assert_eq!(
            pause_due((false, true), (start, at(501)), false),
            (true, false)
        );
        // A screen holds the pause back and does not restart the clock.
        assert_eq!(
            pause_due((false, true), (start, at(900)), true),
            (false, false)
        );
    }

    #[test]
    fn the_location_is_a_tp_with_two_decimals() {
        assert_eq!(
            location_command("minecraft:the_nether", (1.5, 68.0, -2.125), (-180.0, 29.0)),
            "/execute in minecraft:the_nether run tp @s 1.50 68.00 -2.13 -180.00 29.00"
        );
        // The yaw is copied as it is, not wrapped.
        assert!(location_command("a", (0.0, 0.0, 0.0), (540.5, -90.0)).ends_with(" 540.50 -90.00"));
    }
}
