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
