use std::sync::mpsc::channel;

use photon_core::binary::Uuid;
use photon_net::friends::{Friend, FriendData};

use super::*;

fn social() -> (Social, Sender<Event>, Receiver<Command>) {
    let (commands, command_rx) = channel();
    let (event_tx, events) = channel();
    (
        Social::with_channels(commands, events, Instant::now()),
        event_tx,
        command_rx,
    )
}

fn friend() -> Friend {
    Friend {
        id: Uuid::new(1, 1),
        name: "Alex".to_owned(),
    }
}

#[test]
fn nothing_is_asked_while_the_friends_list_is_off() {
    let (mut social, events, commands) = social();
    let now = Instant::now();
    events
        .send(Event::Preferences(Preferences {
            friends_list: false,
            accept_invites: true,
        }))
        .unwrap();
    social.tick(now, (false, true), PresenceStatus::Online);
    assert!(!social.list_enabled());
    assert!(commands.try_recv().is_err());
}

#[test]
fn an_enabled_list_is_asked_once_until_the_answer_comes() {
    let (mut social, events, commands) = social();
    let now = Instant::now();
    events
        .send(Event::Preferences(Preferences {
            friends_list: true,
            accept_invites: true,
        }))
        .unwrap();
    social.tick(now, (false, true), PresenceStatus::Online);
    assert_eq!(commands.try_recv(), Ok(Command::Refresh));
    social.tick(now, (false, true), PresenceStatus::Online);
    assert!(commands.try_recv().is_err());
    let data = FriendData {
        friends: vec![friend()],
        ..FriendData::default()
    };
    events
        .send(Event::List {
            code: ResultCode::Success,
            data,
            hint: None,
        })
        .unwrap();
    social.tick(now, (false, true), PresenceStatus::Online);
    assert_eq!(social.state(), State::Success);
    assert_eq!(social.lists().friends.len(), 1);
    assert!(social.take_changed() && !social.take_changed());
}

#[test]
fn presence_is_posted_only_for_an_open_overlay_with_friends() {
    let (mut social, events, commands) = social();
    let now = Instant::now();
    let later = now + Duration::from_secs(61);
    events
        .send(Event::Preferences(Preferences {
            friends_list: true,
            accept_invites: true,
        }))
        .unwrap();
    events
        .send(Event::List {
            code: ResultCode::Success,
            data: FriendData {
                friends: vec![friend()],
                ..FriendData::default()
            },
            hint: None,
        })
        .unwrap();
    social.refreshing = true;
    social.tick(later, (false, true), PresenceStatus::Online);
    assert!(commands.try_recv().is_err());
    social.tick(later, (true, true), PresenceStatus::Online);
    assert_eq!(
        commands.try_recv(),
        Ok(Command::Presence(PresenceStatus::Online))
    );
    // Not again until the interval has passed.
    social.tick(later, (true, true), PresenceStatus::Online);
    assert!(commands.try_recv().is_err());
}
