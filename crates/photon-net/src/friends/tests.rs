use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

use photon_core::binary::Uuid;

use super::*;

/// Answers written by the test, in order, and the requests the service made.
struct Script {
    replies: RefCell<VecDeque<Result<Reply, TransportError>>>,
    seen: RefCell<Vec<Request>>,
}

impl Transport for &Script {
    fn send(&self, request: &Request) -> Result<Reply, TransportError> {
        self.seen.borrow_mut().push(request.clone());
        self.replies
            .borrow_mut()
            .pop_front()
            .expect("a reply for every request")
    }
}

fn script(replies: Vec<Result<Reply, TransportError>>) -> Script {
    Script {
        replies: RefCell::new(replies.into()),
        seen: RefCell::new(Vec::new()),
    }
}

fn reply(status: u16, body: &str) -> Result<Reply, TransportError> {
    Ok(Reply {
        status,
        body: body.to_owned(),
        ..Reply::default()
    })
}

const ALEX: &str = "0a0b0c0d-0000-4000-8000-000000000001";
const LIST: &str = r#"{"friends":[{"profileId":"0a0b0c0d-0000-4000-8000-000000000001","name":"Alex"}],
    "incomingRequests":[{"profileId":"0a0b0c0d000040008000000000000002","name":"Sam"},{"name":"NoId"}],
    "outgoingRequests":null}"#;

fn alex() -> Friend {
    Friend {
        id: Uuid::new(0x0a0b_0c0d_0000_4000, 0x8000_0000_0000_0001),
        name: "Alex".to_owned(),
    }
}

#[test]
fn a_list_reads_with_missing_lists_empty_and_broken_entries_left_out() {
    let data = parse_friends(LIST).expect("an object");
    assert_eq!(data.friends, vec![alex()]);
    assert_eq!(data.incoming.len(), 1);
    assert_eq!(data.incoming[0].name, "Sam");
    assert!(data.outgoing.is_empty());
    assert_eq!(parse_friends("[1]"), None);
}

#[test]
fn statuses_map_to_codes_in_the_references_order() {
    let code = ResultCode::for_status;
    assert_eq!(code(429), ResultCode::TooManyRequests);
    assert_eq!(code(500), ResultCode::ServiceNotAvailable);
    assert_eq!(code(503), ResultCode::ServiceNotAvailable);
    assert_eq!(code(403), ResultCode::Forbidden);
    assert_eq!(code(400), ResultCode::UnknownProfile);
    assert_eq!(code(401), ResultCode::Unauthorized);
    assert_eq!(code(404), ResultCode::Error);
}

#[test]
fn the_list_is_fetched_cached_for_ten_seconds_and_revalidated_by_etag() {
    let now = Instant::now();
    let script = script(vec![
        Ok(Reply {
            etag: Some("\"v1\"".to_owned()),
            retry_after: Some(Duration::from_secs(30)),
            ..reply(200, LIST).unwrap()
        }),
        reply(304, ""),
    ]);
    let mut service = FriendsService::new(&script);
    let (code, data) = service.friend_data(now);
    assert_eq!((code, data.friends.len()), (ResultCode::Success, 1));
    assert_eq!(
        service.friends_poll_interval(),
        Some(Duration::from_secs(30))
    );
    // Inside the cooldown no request is made.
    let (code, data) = service.friend_data(now + Duration::from_secs(5));
    assert_eq!((code, data.friends.len()), (ResultCode::Success, 1));
    assert_eq!(script.seen.borrow().len(), 1);
    // After it the etag is sent back, and a 304 keeps the cache.
    let (code, data) = service.friend_data(now + Duration::from_secs(11));
    assert_eq!((code, data.friends.len()), (ResultCode::Success, 1));
    let seen = script.seen.borrow();
    assert_eq!(seen[1].etag.as_deref(), Some("\"v1\""));
    assert_eq!((seen[1].method, seen[1].path), (Method::Get, "/friends"));
}

#[test]
fn a_failed_list_request_reports_its_code_and_keeps_the_cache() {
    let now = Instant::now();
    let script = script(vec![
        reply(200, LIST),
        reply(401, "{}"),
        Err(TransportError),
        reply(200, "not json"),
    ]);
    let mut service = FriendsService::new(&script);
    service.friend_data(now);
    let later = |seconds| now + Duration::from_secs(seconds);
    let (code, data) = service.friend_data(later(11));
    assert_eq!((code, data.friends.len()), (ResultCode::Unauthorized, 1));
    // The service could not be reached, and an unreadable answer: both are plain errors.
    assert_eq!(service.friend_data(later(22)).0, ResultCode::Error);
    assert_eq!(service.friend_data(later(33)).0, ResultCode::Error);
}

#[test]
fn actions_put_the_target_and_take_the_new_lists() {
    let now = Instant::now();
    let script = script(vec![reply(200, LIST), reply(200, r#"{"friends":[]}"#)]);
    let mut service = FriendsService::new(&script);
    assert_eq!(service.send_request("Al\"ex", now), ResultCode::Success);
    assert_eq!(service.remove_friend(alex().id, now), ResultCode::Success);
    let seen = script.seen.borrow();
    assert_eq!(seen[0].method, Method::Put);
    assert_eq!(
        seen[0].body.as_deref(),
        Some(r#"{"name":"Al\"ex","updateType":"ADD"}"#)
    );
    assert_eq!(
        seen[1].body.as_deref(),
        Some(&format!(r#"{{"profileId":"{ALEX}","updateType":"REMOVE"}}"#)[..])
    );
    // The answer replaced the cache, and the cooldown means the next fetch is the cache.
    drop(seen);
    let (_, data) = service.friend_data(now);
    assert!(data.friends.is_empty());
}

#[test]
fn an_action_that_fails_maps_its_cause() {
    let now = Instant::now();
    let script = script(vec![
        reply(429, ""),
        reply(400, "{}"),
        Err(TransportError),
        reply(200, "<html>"),
    ]);
    let mut service = FriendsService::new(&script);
    let id = alex().id;
    assert_eq!(
        service.accept_incoming(id, now),
        ResultCode::TooManyRequests
    );
    assert_eq!(
        service.decline_incoming(id, now),
        ResultCode::UnknownProfile
    );
    assert_eq!(
        service.revoke_outgoing(id, now),
        ResultCode::ServiceNotAvailable
    );
    assert_eq!(service.send_request_to(id, now), ResultCode::Error);
}

#[test]
fn settings_and_presence_bodies_are_the_references() {
    let script = script(vec![
        reply(200, "{}"),
        Ok(Reply {
            etag: Some("\"p1\"".to_owned()),
            retry_after: Some(Duration::from_secs(60)),
            ..reply(
                200,
                r#"{"presence":[{"profileId":"0a0b0c0d-0000-4000-8000-000000000001","status":"PLAYING_SERVER","lastUpdated":"2026-10-31T12:00:00Z"}]}"#,
            )
            .unwrap()
        }),
        reply(304, ""),
        Err(TransportError),
    ]);
    let mut service = FriendsService::new(&script);
    assert_eq!(service.update_settings(true, false), ResultCode::Success);
    let first = service.presence(PresenceStatus::Online);
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].status, PresenceStatus::PlayingServer);
    assert_eq!(
        service.presence_poll_interval(),
        Some(Duration::from_secs(60))
    );
    // An unchanged answer and an unreachable service both give the cached presence.
    assert_eq!(service.presence(PresenceStatus::Offline), first);
    assert_eq!(service.presence(PresenceStatus::Offline), first);
    let seen = script.seen.borrow();
    assert_eq!(
        seen[0].body.as_deref(),
        Some(r#"{"friendsPreferences":{"friends":"ENABLED","acceptInvites":"DISABLED"}}"#)
    );
    assert_eq!(
        (seen[0].method, seen[0].path),
        (Method::Post, "/player/attributes")
    );
    assert_eq!(seen[1].body.as_deref(), Some(r#"{"status":"ONLINE"}"#));
    assert_eq!(seen[2].etag.as_deref(), Some("\"p1\""));
}

#[test]
fn retry_after_is_whole_seconds_above_zero() {
    use super::transport::parse_retry_after as parse;
    assert_eq!(parse(Some(" 30 ")), Some(Duration::from_secs(30)));
    assert_eq!(parse(Some("0")), None);
    assert_eq!(parse(Some("soon")), None);
    assert_eq!(parse(None), None);
}

#[test]
fn a_host_that_is_not_https_or_this_machine_is_never_asked() {
    let transport = HttpTransport::new(
        "http://example.com".to_owned(),
        crate::account::AccessToken::new("secret".to_owned()),
    );
    let request = Request {
        method: Method::Get,
        path: "/friends",
        body: None,
        etag: None,
    };
    assert_eq!(transport.send(&request), Err(TransportError));
}

mod updates {
    use super::*;
    use crate::friends::{FriendData, FriendListUpdates, Notice, State};

    fn friend(n: u64, name: &str) -> Friend {
        Friend {
            id: Uuid::new(1, n),
            name: name.to_owned(),
        }
    }

    fn lists(friends: Vec<Friend>, incoming: Vec<Friend>, outgoing: Vec<Friend>) -> FriendData {
        FriendData {
            friends,
            incoming,
            outgoing,
        }
    }

    #[test]
    fn codes_map_to_the_states_the_screens_show() {
        let state = State::for_code;
        assert_eq!(state(ResultCode::Forbidden), State::TemporaryUnavailable);
        assert_eq!(
            state(ResultCode::TooManyRequests),
            State::TemporaryUnavailable
        );
        assert_eq!(
            state(ResultCode::UnknownProfile),
            State::UserMayLackActiveProfile
        );
        assert_eq!(state(ResultCode::Error), State::GenericError);
        assert_eq!(state(ResultCode::Unauthorized), State::Unauthorized);
        assert_eq!(state(ResultCode::Success), State::Success);
    }

    #[test]
    fn the_first_answer_sets_the_lists_without_toasts() {
        let now = Instant::now();
        let mut updates = FriendListUpdates::new();
        assert_eq!(updates.state(), State::Loading);
        let applied = updates.apply(
            (
                ResultCode::Success,
                lists(vec![friend(1, "A")], vec![], vec![]),
            ),
            now,
            true,
        );
        assert!(applied.notify && applied.notices.is_empty());
        assert_eq!(updates.state(), State::Success);
        assert_eq!(updates.latest().friends.len(), 1);
    }

    #[test]
    fn a_change_toasts_by_where_the_player_came_from() {
        let now = Instant::now();
        let (a, b, c, d) = (
            friend(1, "A"),
            friend(2, "B"),
            friend(3, "C"),
            friend(4, "D"),
        );
        let mut updates = FriendListUpdates::new();
        let first = lists(vec![], vec![b.clone()], vec![c.clone()]);
        updates.apply((ResultCode::Success, first), now, true);
        let second = lists(vec![a.clone(), b.clone()], vec![d.clone()], vec![c.clone()]);
        let applied = updates.apply((ResultCode::Success, second), now, true);
        assert!(applied.notify);
        assert_eq!(
            applied.notices,
            vec![
                Notice::FriendAdded(a),
                Notice::RequestAccepted(b),
                Notice::RequestReceived(d),
            ]
        );
        // The same lists again change nothing and tell nobody.
        let again = lists(
            vec![friend(1, "A"), friend(2, "B")],
            vec![friend(4, "D")],
            vec![c],
        );
        let applied = updates.apply((ResultCode::Success, again), now, true);
        assert_eq!(applied, Applied::default());
    }

    #[test]
    fn toasts_off_still_notifies_the_screens() {
        let now = Instant::now();
        let mut updates = FriendListUpdates::new();
        updates.apply((ResultCode::Success, FriendData::default()), now, false);
        let data = lists(vec![], vec![], vec![friend(5, "E")]);
        let applied = updates.apply((ResultCode::Success, data), now, false);
        assert!(applied.notify && applied.notices.is_empty());
    }

    #[test]
    fn a_failure_notifies_and_the_next_success_starts_over_quietly() {
        let now = Instant::now();
        let mut updates = FriendListUpdates::new();
        updates.apply((ResultCode::Success, FriendData::default()), now, true);
        let failed = updates.apply((ResultCode::Unauthorized, FriendData::default()), now, true);
        assert!(failed.notify);
        assert_eq!(updates.state(), State::Unauthorized);
        let data = lists(vec![friend(1, "A")], vec![], vec![]);
        let back = updates.apply((ResultCode::Success, data), now, true);
        assert!(back.notify && back.notices.is_empty());
    }

    #[test]
    fn updates_are_due_by_the_hint_and_slower_behind_a_screen() {
        let now = Instant::now();
        let mut updates = FriendListUpdates::new();
        assert!(updates.is_due(now, None, false));
        updates.apply((ResultCode::Success, FriendData::default()), now, true);
        let after = |seconds| now + Duration::from_secs(seconds);
        assert!(!updates.is_due(after(59), None, true));
        assert!(updates.is_due(after(60), None, true));
        assert!(!updates.is_due(after(299), None, false));
        assert!(updates.is_due(after(300), None, false));
        let hint = Some(Duration::from_secs(10));
        assert!(updates.is_due(after(10), hint, true));
        assert!(!updates.is_due(after(49), hint, false));
    }
}

#[test]
fn preferences_read_the_friends_flags_and_default_to_off() {
    use crate::friends::{Preferences, parse_preferences};
    let on = r#"{"privileges":{},"friendsPreferences":{"friends":"ENABLED","acceptInvites":"DISABLED"}}"#;
    assert_eq!(
        parse_preferences(on),
        Some(Preferences {
            friends_list: true,
            accept_invites: false
        })
    );
    assert_eq!(parse_preferences("{}"), Some(Preferences::default()));
    assert_eq!(parse_preferences("nope"), None);
    let script = script(vec![reply(200, on), reply(500, "")]);
    let mut service = FriendsService::new(&script);
    assert!(service.preferences().friends_list);
    assert_eq!(service.preferences(), Preferences::default());
    assert_eq!(script.seen.borrow()[0].path, "/player/attributes");
}
