//! A link handed over while a game is played: the window comes forward, a launch code is spent at
//! once (it lasts a minute), and the answer is asked about over the pause menu. A sign-in link is
//! taken only when nothing but the pause menu is open, since the browser's state lasts two minutes.

use super::*;
use crate::launch_link::{self, InPlay, LinkTask, SignInStep};
use crate::signin_link;
use std::sync::mpsc::channel;

impl Play {
    /// One frame of it.
    pub(super) fn game_link_frame(&mut self) {
        if crate::link_socket::arrived() {
            self.surface.raise();
        }
        self.launch_link_frame();
        self.signin_link_frame();
    }

    /// A launch link: its code is spent at once, and a newer one waits while one is being spent or
    /// asked about. The answer waits for the player to close what is open (a spent code lasts).
    fn launch_link_frame(&mut self) {
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
        if self.controls.link_confirm.is_some() || self.controls.screen_open_besides_pause() {
            return;
        }
        if let Some(outcome) = self.controls.game_link.ready.take() {
            self.ask(launch_link::in_play(outcome));
        }
    }

    /// A sign-in link: looked up, asked about, signed in and opened in the browser.
    fn signin_link_frame(&mut self) {
        let idle =
            self.controls.link_confirm.is_none() && !self.controls.screen_open_besides_pause();
        let held = &mut self.controls.game_link;
        if held.signin.is_none()
            && idle
            && let Some(state) = launch_link::take_pending_signin()
        {
            let (sender, receiver) = channel();
            let _ = std::thread::Builder::new()
                .name("Sign-in Link".to_owned())
                .spawn(move || {
                    let _ = sender.send(signin_link::signed_in_name());
                });
            held.signin = Some(SignInStep::Looking(receiver, state));
        }
        if let Some(state) = held.signin_agreed.take() {
            let (sender, receiver) = channel();
            let _ = std::thread::Builder::new()
                .name("Sign-in".to_owned())
                .spawn(move || {
                    let _ = sender.send(signin_link::run(&state));
                });
            held.signin = Some(SignInStep::Working(receiver));
        }
        if let Some(prompt) = held
            .signin
            .take()
            .and_then(|step| Self::signin_step(step, held))
        {
            self.ask(prompt);
        }
    }

    /// The next of a sign-in link's steps: what to ask or say, or none while a thread works.
    fn signin_step(step: SignInStep, held: &mut launch_link::GameLink) -> Option<InPlay> {
        const HEADING: &str = "photon.signin.heading";
        match step {
            SignInStep::Looking(receiver, state) => match receiver.try_recv() {
                Ok(Some(name)) => Some(InPlay::AskSignIn(state, name)),
                Ok(None) => Some(InPlay::Say(HEADING, "photon.signin.not_signed_in")),
                Err(_) => {
                    held.signin = Some(SignInStep::Looking(receiver, state));
                    None
                }
            },
            SignInStep::Working(receiver) => {
                let Ok(outcome) = receiver.try_recv() else {
                    held.signin = Some(SignInStep::Working(receiver));
                    return None;
                };
                Some(InPlay::Say(HEADING, signin_link::message_key(&outcome)))
            }
        }
    }

    fn ask(&mut self, prompt: InPlay) {
        self.controls
            .ask_about_game(prompt, &mut self.surface, &self.screen_assets);
    }
}
