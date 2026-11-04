//! A link handed over while a game is played: the window comes forward, a launch code is spent at
//! once (it lasts a minute), and the answer is asked about over the pause menu.

use super::*;
use crate::launch_link::{self, LinkTask};

impl Play {
    /// One frame of it. A sign-in link is left for the menu; a newer launch link waits while one
    /// is being spent or asked about.
    pub(super) fn game_link_frame(&mut self) {
        if crate::link_socket::arrived() {
            self.surface.raise();
        }
        let held = &mut self.controls.game_link;
        if held.task.is_none()
            && held.ready.is_none()
            && let Some(code) = launch_link::take_pending_launch()
        {
            held.task = Some(LinkTask::start(code));
        }
        if let Some(outcome) = held.task.as_ref().and_then(LinkTask::poll) {
            held.task = None;
            held.ready = Some(outcome);
        }
        if self.controls.link_confirm.is_some() {
            return;
        }
        if let Some(outcome) = self.controls.game_link.ready.take() {
            let prompt = launch_link::in_play(outcome);
            self.controls
                .ask_about_game(prompt, &mut self.surface, &self.screen_assets);
        }
    }
}
