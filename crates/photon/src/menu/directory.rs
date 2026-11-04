//! The directory of Photon's backend on the home screen: fetched off the menu's thread, shown as
//! cards, and joined by address. Without `PHOTON_BACKEND` none of this runs and the home screen
//! stays as it is.

use std::sync::mpsc::{Receiver, channel};

use photon_net::backend::{Backend, BackendError, DirectoryServer};

use super::{Choice, Menu, Screen};
use crate::play::Target;

/// The directory being fetched, and what it said.
pub(super) struct Directory {
    receiver: Option<Receiver<Result<Vec<DirectoryServer>, BackendError>>>,
    pub(super) servers: Vec<DirectoryServer>,
}

impl Directory {
    /// The cards of the servers known so far.
    pub(super) fn cards(&self) -> Vec<(String, String)> {
        self.servers.iter().map(card).collect()
    }

    /// Starts the fetch when a backend is named; nothing otherwise.
    pub(super) fn look_up() -> Self {
        let receiver = Backend::from_environment().map(|backend| {
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
            Err(error) => log::warn!("the directory could not be read: {error:?}"),
        }
        self.show_directory();
    }

    /// The cards of the servers known so far, on the home screen when it is open.
    pub(super) fn show_directory(&mut self) {
        let cards = self.directory.cards();
        if let Screen::Home(screen) = &mut self.screen {
            screen.set_directory(cards);
        }
    }

    /// A press on the card of the directory's server at `index`: its address is joined.
    pub(super) fn join_directory(&mut self, index: usize, size: (i32, i32)) -> Option<Choice> {
        let server = self.directory.servers.get(index)?.clone();
        self.connect(
            Target::Name(server.address.clone()),
            Some(&server.address),
            (&server.name, &server.address),
            size,
        );
        None
    }
}
