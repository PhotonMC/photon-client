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

impl Controls {
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
        match key {
            glfw::Key::F4 => self.debug_switch_game_mode(session, surface, assets),
            glfw::Key::C => self.debug_copy_location(session, surface, assets),
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
    use super::location_command;

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
