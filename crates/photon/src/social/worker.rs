//! The thread that talks to the friends service (`Friends List` in the reference): it takes
//! commands in order, answers each with events, and never touches the game's state.

use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use photon_net::friends::{
    FriendData, FriendsService, Preferences, PresenceEntry, PresenceStatus, ResultCode, Transport,
};

/// An action on the lists (`PlayerSocialManager`'s calls).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Action {
    Send(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Command {
    /// The friends flags of the account.
    Preferences,
    /// An update of the lists, which the service's own cooldown may answer from its cache.
    Refresh,
    Action(Action),
    /// `updateFriendSettings`: the friends list, then the friend invites.
    Settings(bool, bool),
    Presence(PresenceStatus),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Event {
    Preferences(Preferences),
    List {
        code: ResultCode,
        data: FriendData,
        /// The service's poll hint for the lists.
        hint: Option<Duration>,
    },
    Action(ResultCode),
    Settings {
        code: ResultCode,
        friends_list: bool,
        invites: bool,
    },
    Presence {
        entries: Vec<PresenceEntry>,
        hint: Option<Duration>,
    },
}

/// Serves commands until the sender is dropped or the receiver of the events is.
pub(crate) fn run<T: Transport>(
    mut service: FriendsService<T>,
    commands: &Receiver<Command>,
    events: &Sender<Event>,
) {
    while let Ok(command) = commands.recv() {
        if !serve(&mut service, command, events) {
            return;
        }
    }
}

/// One command; false once nobody is listening for the answers.
fn serve<T: Transport>(
    service: &mut FriendsService<T>,
    command: Command,
    events: &Sender<Event>,
) -> bool {
    let now = Instant::now();
    let answers = match command {
        Command::Preferences => vec![Event::Preferences(service.preferences())],
        Command::Refresh => vec![list(service, now)],
        Command::Action(action) => {
            let code = act(service, action, now);
            // `runAction`: a call that worked is followed by a forced update of the lists.
            let mut answers = vec![Event::Action(code)];
            if code == ResultCode::Success {
                answers.push(list(service, now));
            }
            answers
        }
        Command::Settings(friends_list, invites) => vec![Event::Settings {
            code: service.update_settings(friends_list, invites),
            friends_list,
            invites,
        }],
        Command::Presence(status) => {
            let entries = service.presence(status);
            vec![Event::Presence {
                entries,
                hint: service.presence_poll_interval(),
            }]
        }
    };
    answers.into_iter().all(|event| events.send(event).is_ok())
}

fn list<T: Transport>(service: &mut FriendsService<T>, now: Instant) -> Event {
    let (code, data) = service.friend_data(now);
    Event::List {
        code,
        data,
        hint: service.friends_poll_interval(),
    }
}

fn act<T: Transport>(service: &mut FriendsService<T>, action: Action, now: Instant) -> ResultCode {
    match action {
        Action::Send(name) => service.send_request(&name, now),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::sync::{Arc, Mutex};

    use photon_net::friends::{Reply, Request, TransportError};

    use super::*;

    struct Fake(Arc<Mutex<Vec<Request>>>);

    impl Transport for Fake {
        fn send(&self, request: &Request) -> Result<Reply, TransportError> {
            self.0.lock().unwrap().push(request.clone());
            Ok(Reply {
                status: 200,
                body: r#"{"friends":[{"profileId":"0a0b0c0d-0000-4000-8000-000000000001","name":"Alex"}]}"#
                    .to_owned(),
                ..Reply::default()
            })
        }
    }

    #[test]
    fn a_working_action_is_followed_by_the_lists() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        command_tx
            .send(Command::Action(Action::Send("Alex".to_owned())))
            .unwrap();
        command_tx
            .send(Command::Presence(PresenceStatus::Online))
            .unwrap();
        drop(command_tx);
        run(
            FriendsService::new(Fake(seen.clone())),
            &command_rx,
            &event_tx,
        );
        drop(event_tx);
        let events: Vec<Event> = event_rx.iter().collect();
        assert_eq!(events[0], Event::Action(ResultCode::Success));
        let Event::List { code, data, .. } = &events[1] else {
            panic!("the lists follow a working action");
        };
        assert_eq!((*code, data.friends.len()), (ResultCode::Success, 1));
        assert!(matches!(events[2], Event::Presence { .. }));
        let paths: Vec<&str> = seen.lock().unwrap().iter().map(|r| r.path).collect();
        assert_eq!(paths, ["/friends", "/presence"]);
    }
}
