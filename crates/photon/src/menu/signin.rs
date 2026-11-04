//! A website sign-in link on the menu (ADR 0023): the name of the signed-in player is looked up,
//! the player is asked, and a yes signs in and opens the browser, each off the menu's thread.

use std::sync::mpsc::{Receiver, channel};

use photon_game::text::Component;
use photon_gui::screen::notice::{Answer, NoticeScreen};

use super::input::{ask_event, failed_event};
use super::{Menu, Screen};
use crate::launch_link::{self, Link, LinkTask};
use crate::play::Client;
use crate::signin_link::{self, Outcome};

/// Where a sign-in link has got to.
pub(super) enum SignInFlow {
    /// Looking up who is signed in, for the question; the browser's state waits.
    Looking(Receiver<Option<String>>, String),
    /// The browser's state, signed in and opened on a thread.
    Working(Receiver<Outcome>),
}

impl SignInFlow {
    pub(super) fn look_up(state: String) -> Self {
        let (sender, receiver) = channel();
        let _ = std::thread::Builder::new()
            .name("Sign-in Link".to_owned())
            .spawn(move || {
                let _ = sender.send(signin_link::signed_in_name());
            });
        Self::Looking(receiver, state)
    }
}

/// The link the game was started with, as far as the menu takes it: a launch link starts spending
/// its code, a sign-in link starts looking up who is signed in.
pub(super) fn pending_link() -> (Option<LinkTask>, Option<SignInFlow>) {
    match launch_link::take_pending() {
        Some(Link::Launch(code)) => (Some(LinkTask::start(code)), None),
        Some(Link::SignIn(state)) => (None, Some(SignInFlow::look_up(state))),
        None => (None, None),
    }
}

impl Menu {
    /// A link handed over by a second start (`photon open-link` while this client runs) raises the
    /// window once, and is taken on
    /// the home screen or a notice and when nothing else is going on.
    pub(super) fn link_arrived(&mut self, client: &mut Client) {
        if crate::link_socket::arrived() {
            client.surface.raise();
        }
        let idle = matches!(self.screen, Screen::Home(_) | Screen::Failed(..));
        if !idle || self.link.is_some() || self.signin.is_some() {
            return;
        }
        (self.link, self.signin) = pending_link();
    }

    /// One window event for a notice or a question: a button leaves it, or answers it.
    pub(super) fn notice_event(&mut self, event: glfw::WindowEvent, size: (i32, i32)) {
        let mouse = self.mouse;
        match &mut self.screen {
            Screen::Failed(screen, home) => {
                if failed_event(screen, event, mouse) {
                    self.screen = if *home {
                        self.title(size)
                    } else {
                        self.server_list(size)
                    };
                }
            }
            Screen::Ask(screen, state) => {
                if let Some(answer) = ask_event(screen, event, mouse) {
                    let state = state.clone();
                    self.signin_answered(answer, &state, size);
                }
            }
            _ => {}
        }
    }

    /// The next step of a sign-in link, when its thread has answered.
    pub(super) fn signin_polled(&mut self, size: (i32, i32)) {
        match self.signin.take() {
            Some(SignInFlow::Looking(receiver, state)) => match receiver.try_recv() {
                Ok(Some(name)) => self.ask_to_sign_in(&name, state, size),
                Ok(None) => self.notice("photon.signin.not_signed_in", size),
                Err(_) => self.signin = Some(SignInFlow::Looking(receiver, state)),
            },
            Some(SignInFlow::Working(receiver)) => match receiver.try_recv() {
                Ok(outcome) => self.signin_done(outcome, size),
                Err(_) => self.signin = Some(SignInFlow::Working(receiver)),
            },
            None => {}
        }
    }

    /// The question: sign the website's browser in as this player?
    fn ask_to_sign_in(&mut self, name: &str, state: String, size: (i32, i32)) {
        let (font, lang) = (self.gui.text_assets().0, self.gui.lang());
        let question = NoticeScreen::asking(
            font,
            lang,
            size,
            (
                &Component::translatable("photon.signin.heading"),
                &Component::translatable_with("photon.signin.ask", vec![Component::literal(name)]),
                &Component::translatable("photon.signin.cancel"),
                &Component::translatable("photon.signin.confirm"),
            ),
        );
        self.screen = Screen::Ask(question, state);
    }

    /// The answer to the question: a no goes home; a yes starts the sign-in.
    pub(super) fn signin_answered(&mut self, answer: Answer, state: &str, size: (i32, i32)) {
        match answer {
            Answer::Back => self.screen = self.title(size),
            Answer::Yes => {
                let state = state.to_owned();
                let (sender, receiver) = channel();
                let _ = std::thread::Builder::new()
                    .name("Sign-in".to_owned())
                    .spawn(move || {
                        let _ = sender.send(signin_link::run(&state));
                    });
                self.signin = Some(SignInFlow::Working(receiver));
                self.screen = self.title(size);
            }
        }
    }

    fn signin_done(&mut self, outcome: Outcome, size: (i32, i32)) {
        let key = signin_link::message_key(&outcome);
        self.notice(key, size);
    }

    /// A notice about the sign-in, back to the home screen.
    fn notice(&mut self, reason: &str, size: (i32, i32)) {
        let heading = Component::translatable("photon.signin.heading");
        self.notice_screen(&heading, &Component::translatable(reason), size, true);
    }
}
