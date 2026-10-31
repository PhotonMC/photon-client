use photon_game::spectator::Action;

use super::*;

/// The time `SpectatorGui` fades by (`Util.getMillis`), counted from the first call.
pub(crate) fn now_ms() -> i64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_millis() as i64
}

impl Controls {
    /// A number key of a spectator picks a slot of the spectator menu (`onHotbarSelected`).
    pub(super) fn spectator_hotbar(&mut self, name: &str, session: &mut PlaySession) {
        let Some(slot) = name["key.hotbar.".len()..]
            .parse::<i32>()
            .ok()
            .and_then(|number| number.checked_sub(1))
        else {
            return;
        };
        let sources = session.spectator_sources();
        let action = self.spectator.on_hotbar_selected(slot, now_ms(), &sources);
        self.run_spectator_action(action, session);
    }

    /// `key.spectatorHotbar` (`onHotbarActionKeyPressed`).
    pub(super) fn spectator_action_key(&mut self, session: &mut PlaySession) {
        let sources = session.spectator_sources();
        let action = self.spectator.on_action_key(now_ms(), &sources);
        self.run_spectator_action(action, session);
    }

    /// `MouseHandler.onScroll` of a spectator with the menu open: the wheel moves the selection.
    pub(super) fn spectator_scrolled(&mut self, (x, y): (f64, f64)) {
        if let Some(wheel) = self.scroll_wheel.wheel(x, y) {
            self.spectator.on_mouse_scrolled(-wheel, now_ms());
        }
    }

    /// `PlayerMenuItem.selectItem`: the server is asked to teleport the spectator.
    fn run_spectator_action(&mut self, action: Option<Action>, session: &mut PlaySession) {
        if let Some(Action::Teleport(id)) = action
            && let Err(error) = session.teleport_to_entity(id)
        {
            log::warn!("could not teleport to the player: {error:?}");
        }
    }
}
