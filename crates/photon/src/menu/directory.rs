//! The directory of Photon's backend on the home screen: fetched off the menu's thread, shown as
//! cards, and joined by address. Without `PHOTON_BACKEND` none of this runs and the home screen
//! stays as it is.

use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

use photon_game::text::Lang;
use photon_gui::screen::home::{GameInfo, HomeScreen};
use photon_net::backend::{BackendError, DirectoryServer};

use super::resume::Resume;
use super::signin::SignInFlow;
use super::{Choice, Menu, Screen};
use crate::launch_link::Outcome;
use crate::play::Target;
use photon_game::text::Component;

/// How long the home screen shows a directory before it asks again.
const REFRESH: Duration = Duration::from_secs(30);

type Fetched = Result<Vec<DirectoryServer>, BackendError>;

/// The directory being fetched, and what it said.
pub(super) struct Directory {
    receiver: Option<Receiver<Fetched>>,
    pub(super) servers: Vec<DirectoryServer>,
    /// The fetch was answered with an error and there is no list to keep showing.
    failed: bool,
    /// When the last fetch started.
    started: Instant,
}

/// Whether a new fetch is due: the home screen is showing, none is going on, and the last one
/// is old enough.
fn due(elapsed: Duration, on_home: bool, in_flight: bool) -> bool {
    on_home && !in_flight && elapsed >= REFRESH
}

impl Directory {
    /// The cards of the servers known so far.
    pub(super) fn cards(&self) -> Vec<GameInfo> {
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
        let mut directory = Self {
            receiver: None,
            servers: Vec::new(),
            failed: false,
            started: Instant::now(),
        };
        directory.fetch();
        directory
    }

    fn fetch(&mut self) {
        self.started = Instant::now();
        self.receiver = crate::links_config::backend().map(|backend| {
            let (sender, receiver) = channel();
            let _ = std::thread::Builder::new()
                .name("Directory".to_owned())
                .spawn(move || {
                    let listed = backend.and_then(|backend| backend.servers());
                    let _ = sender.send(listed);
                });
            receiver
        });
    }

    /// Takes an answer: a list replaces the one shown; an error keeps a list that was already
    /// there, and only says "unavailable" when there is nothing else to show. Whether the cards
    /// differ from before.
    fn received(&mut self, answer: Fetched) -> bool {
        let before = self.cards();
        match answer {
            Ok(servers) => {
                self.servers = servers;
                self.failed = false;
            }
            Err(error) => {
                log::warn!("the directory could not be read: {error:?}");
                self.failed = self.servers.is_empty();
            }
        }
        before != self.cards() || self.failed
    }
}

/// What a card of the directory says: the name over the players. Photon games name no version:
/// the client plays the newest.
fn card(server: &DirectoryServer) -> GameInfo {
    let detail = format!("{} online", server.players);
    GameInfo {
        id: server.id.clone(),
        name: server.name.clone(),
        detail,
        description: server.description.clone(),
        videos: server.videos.len(),
        images: server.images.len(),
    }
}

impl Menu {
    /// The answer of the fetch, when it has come: the cards on the home screen; and the next
    /// fetch, once the home screen has shown the last one for a while.
    pub(super) fn directory_polled(&mut self) {
        let Some(receiver) = self.directory.receiver.as_ref() else {
            let on_home = matches!(self.screen, Screen::Home(_));
            if due(self.directory.started.elapsed(), on_home, false) {
                self.directory.fetch();
            }
            return;
        };
        let Ok(answer) = receiver.try_recv() else {
            return;
        };
        self.directory.receiver = None;
        if self.directory.received(answer) {
            self.show_directory();
        }
    }

    /// The cards of the servers known so far, on the home screen when it is open.
    pub(super) fn show_directory(&mut self) {
        let lang = self.gui.lang();
        if let Screen::Home(screen) = &mut self.screen {
            self.directory.apply(screen, lang);
        }
    }

    /// Play on an open game page: the server it was opened for, if the directory still has it.
    pub(super) fn join_open_game(&mut self, size: (i32, i32)) -> Option<Choice> {
        let Screen::Home(screen) = &self.screen else {
            return None;
        };
        let id = screen.open_game_id()?;
        let index = self.directory.servers.iter().position(|s| s.id == id)?;
        self.join_directory(index, size)
    }

    /// A press on the play arrow of the directory's server at `index`: its address is joined.
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
        self.link_outcome(outcome, size);
    }

    /// What a launch link's decision shows: the join, or why there is none.
    pub(super) fn link_outcome(&mut self, outcome: Outcome, size: (i32, i32)) {
        let key = match outcome {
            Outcome::Join(address, name) => {
                self.joined_from_home = true;
                self.connect(
                    Target::Name(address.clone()),
                    Some(&address),
                    (&name, &address),
                    size,
                );
                return;
            }
            Outcome::OtherProfile => "photon.link.other_profile",
            Outcome::NotSignedIn(launch) => {
                self.signin = Some(SignInFlow::Waiting(Resume::Launch(launch)));
                return self.offer_sign_in("photon.link.not_signed_in", size);
            }
            Outcome::Refused => "photon.link.refused",
            Outcome::Unavailable => "photon.link.unavailable",
        };
        self.join_failed(&Component::translatable(key), size, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server(name: &str, players: i32) -> DirectoryServer {
        DirectoryServer {
            id: name.to_owned(),
            name: name.to_owned(),
            address: "h:1".to_owned(),
            description: String::new(),
            version: "1".to_owned(),
            images: Vec::new(),
            videos: Vec::new(),
            players,
        }
    }

    fn empty() -> Directory {
        Directory {
            receiver: None,
            servers: Vec::new(),
            failed: false,
            started: Instant::now(),
        }
    }

    #[test]
    fn a_list_replaces_the_one_shown() {
        let mut directory = empty();
        assert!(directory.received(Ok(vec![server("a", 1)])));
        assert!(directory.received(Ok(vec![server("a", 2)])));
        assert!(!directory.received(Ok(vec![server("a", 2)])));
        assert_eq!(directory.servers[0].players, 2);
    }

    #[test]
    fn an_error_keeps_the_list_that_was_there() {
        let mut directory = empty();
        directory.received(Ok(vec![server("a", 1)]));
        assert!(!directory.received(Err(BackendError::Unavailable)));
        assert_eq!(directory.servers.len(), 1);
        assert!(!directory.failed);
    }

    #[test]
    fn an_error_with_nothing_shown_says_unavailable_until_a_list_comes() {
        let mut directory = empty();
        assert!(directory.received(Err(BackendError::Unavailable)));
        assert!(directory.failed);
        directory.received(Ok(vec![server("a", 1)]));
        assert!(!directory.failed);
    }

    #[test]
    fn a_fetch_is_due_only_on_the_home_screen_when_none_is_going_on_and_the_last_is_old() {
        let old = REFRESH;
        assert!(due(old, true, false));
        assert!(!due(old, false, false));
        assert!(!due(old, true, true));
        assert!(!due(REFRESH - Duration::from_secs(1), true, false));
    }
}
