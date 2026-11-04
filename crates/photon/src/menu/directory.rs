//! The directory of Photon's backend on the home screen: fetched off the menu's thread, shown as
//! cards, and joined by address. Without `PHOTON_BACKEND` none of this runs and the home screen
//! stays as it is.

use std::sync::mpsc::{Receiver, channel};

use photon_game::text::Lang;
use photon_gui::screen::home::HomeScreen;
use photon_net::backend::{BackendError, DirectoryServer};

use super::{Choice, Menu, Screen};
use crate::launch_link::Outcome;
use crate::play::Target;
use photon_game::text::Component;

/// The directory being fetched, and what it said.
pub(super) struct Directory {
    receiver: Option<Receiver<Result<Vec<DirectoryServer>, BackendError>>>,
    pub(super) servers: Vec<DirectoryServer>,
    /// The fetch was answered with an error.
    failed: bool,
}

impl Directory {
    /// The cards of the servers known so far.
    pub(super) fn cards(&self) -> Vec<(String, String)> {
        self.servers.iter().map(card).collect()
    }

    /// What is known so far on the home screen: the servers' cards, or the failure on the
    /// placeholder.
    pub(super) fn apply(&self, screen: &mut HomeScreen, lang: &Lang) {
        screen.set_directory(self.cards());
        if self.failed {
            let key = "photon.home.games.unavailable";
            screen.set_directory_unavailable(lang.get(key).unwrap_or(key));
        }
    }

    /// Starts the fetch when a backend is named; nothing otherwise.
    pub(super) fn look_up() -> Self {
        let receiver = crate::links_config::backend().map(|backend| {
            let (sender, receiver) = channel();
            let _ = std::thread::Builder::new()
                .name("Directory".to_owned())
                .spawn(move || {
                    let listed = backend.and_then(|backend| backend.servers());
                    let _ = sender.send(listed);
                });
            receiver
        });
        Self {
            receiver,
            servers: Vec::new(),
            failed: false,
        }
    }
}

/// What a card of the directory says: the name over the players and the version.
fn card(server: &DirectoryServer) -> (String, String) {
    let detail = format!("{} online  {}", server.players, server.version);
    (server.name.clone(), detail.trim_end().to_owned())
}

impl Menu {
    /// The answer of the fetch, when it has come: the cards on the home screen.
    pub(super) fn directory_polled(&mut self) {
        let Some(receiver) = self.directory.receiver.as_ref() else {
            return;
        };
        let Ok(answer) = receiver.try_recv() else {
            return;
        };
        self.directory.receiver = None;
        match answer {
            Ok(servers) => self.directory.servers = servers,
            Err(error) => {
                log::warn!("the directory could not be read: {error:?}");
                self.directory.failed = true;
            }
        }
        self.show_directory();
    }

    /// The cards of the servers known so far, on the home screen when it is open.
    pub(super) fn show_directory(&mut self) {
        let lang = self.gui.lang();
        if let Screen::Home(screen) = &mut self.screen {
            self.directory.apply(screen, lang);
        }
    }

    /// A press on the card of the directory's server at `index`: its address is joined.
    pub(super) fn join_directory(&mut self, index: usize, size: (i32, i32)) -> Option<Choice> {
        let server = self.directory.servers.get(index)?.clone();
        self.joined_from_home = true;
        self.connect(
            Target::Name(server.address.clone()),
            Some(&server.address),
            (&server.name, &server.address),
            size,
        );
        None
    }
}

impl Menu {
    /// The decision about the launch link, when it has come: the join, or why there is none.
    pub(super) fn link_polled(&mut self, size: (i32, i32)) {
        let Some(outcome) = self.link.as_ref().and_then(|task| task.poll()) else {
            return;
        };
        self.link = None;
        let key = match outcome {
            Outcome::Join(address) => {
                self.joined_from_home = true;
                self.connect(
                    Target::Name(address.clone()),
                    Some(&address),
                    (&address, &address),
                    size,
                );
                return;
            }
            Outcome::OtherProfile => "photon.link.other_profile",
            Outcome::NotSignedIn => "photon.link.not_signed_in",
            Outcome::Refused => "photon.link.refused",
            Outcome::Unavailable => "photon.link.unavailable",
        };
        self.join_failed(&Component::translatable(key), size, true);
    }
}
