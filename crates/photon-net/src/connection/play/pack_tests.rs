//! The server resource packs over a connection: what the client tells the server in the
//! configuration and play phases, against a server that is played by the test.

use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use photon_core::binary::Uuid;
use photon_game::servers::PackStatus;
use photon_protocol::phase::ConnectionProtocol;

use super::lifecycle::spawn_reader;
use super::*;
use crate::connection::open::configure_connection;
use crate::server_packs::{DownloadRequest, PackFetcher, PackSink, PackSource, PackUser};

const PACK: Uuid = Uuid::new(1, 2);
const HASH: &str = "0123456789abcdef0123456789abcdef01234567";

struct Fetch(Result<PathBuf, String>);

impl PackFetcher for Fetch {
    fn fetch(&self, _request: &DownloadRequest, _user: &PackUser) -> Result<PathBuf, String> {
        self.0.clone()
    }
}

fn source(fetch: Result<PathBuf, String>, setting: PackStatus) -> PackSource {
    let source = PackSource::new(Arc::new(Fetch(fetch)), true);
    source.set_setting(setting);
    // The connection is made: the setting decides what the machine does.
    source.configure_for_server(PackSink::new(|_, _| {}));
    source
}

fn framed(id: i32, body: &[u8]) -> Vec<u8> {
    let mut packet = Vec::new();
    wire::write_varint(id, &mut packet);
    packet.extend_from_slice(body);
    let mut out = Vec::new();
    wire::write_varint(packet.len() as i32, &mut out);
    out.extend_from_slice(&packet);
    out
}

fn push_body(url: &str, required: bool) -> Vec<u8> {
    let mut body = Vec::new();
    photon_core::binary::write_uuid(PACK, &mut body);
    for text in [url, HASH] {
        let units = text.encode_utf16().collect::<Vec<_>>();
        photon_core::utf8::write_utf8(&mut body, &units, 32_767).expect("string");
    }
    body.extend_from_slice(&[u8::from(required), 0]);
    body
}

/// The statuses the client sends, as the action numbers, until `count` have arrived.
fn statuses(stream: &mut Transport, packet_id: i32, count: usize) -> Vec<u8> {
    (0..count)
        .map(|_| {
            let answer = read_packet(stream, None).expect("status");
            let (id, body) = split_packet(&answer).expect("split");
            assert_eq!(id, packet_id);
            let mut expected = Vec::new();
            photon_core::binary::write_uuid(PACK, &mut expected);
            assert_eq!(&body[..16], &expected[..]);
            body[16]
        })
        .collect()
}

/// A play session over a connection to a server run by `serve`; the pack source is the
/// session's.
fn play_against(
    packs: &PackSource,
    serve: impl FnOnce(Transport) + Send + 'static,
) -> (PlaySession, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let address = listener.local_addr().expect("address");
    let server = std::thread::spawn(move || {
        serve(Transport::from(listener.accept().expect("accept").0));
    });
    let stream = Transport::from(TcpStream::connect(address).expect("connect"));
    let incoming = spawn_reader(
        stream.try_clone().expect("clone"),
        None,
        VersionRegistries::newest(),
    );
    let state = ConnectionState {
        packs: packs.clone(),
        ..ConnectionState::default()
    };
    packs.configure_for_server(
        crate::connection::pack_sink::sink_for(&stream, ConnectionProtocol::Play, None)
            .expect("sink"),
    );
    (
        PlaySession::assemble(stream, None, incoming, state, ("Tester", false)),
        server,
    )
}

#[test]
fn a_pack_asked_about_is_answered_by_the_player() {
    let packs = source(Err("never".into()), PackStatus::Prompt);
    let (mut session, server) = play_against(&packs, |mut stream| {
        let push = framed(
            PLAY_CLIENTBOUND_RESOURCE_PACK_PUSH_PACKET_ID,
            &push_body("https://example.com/pack.zip", false),
        );
        stream.write_all(&push).expect("push");
        let seen = statuses(&mut stream, PLAY_SERVERBOUND_RESOURCE_PACK_PACKET_ID, 1);
        assert_eq!(seen, [PackAction::Declined as u8]);
    });
    session.wait(Duration::from_secs(5)).expect("push");
    // Nothing was said to the server while the player has not answered.
    let prompt = packs.take_prompt().expect("a question");
    assert_eq!(prompt.requests.len(), 1);
    assert!(!prompt.required);
    let outcome = packs.answer_prompt(false);
    assert_eq!(outcome.remember, Some(PackStatus::Disabled));
    assert_eq!(outcome.disconnect, None);
    server.join().expect("server");
}

#[test]
fn a_bad_address_is_refused_at_once() {
    let packs = source(Err("never".into()), PackStatus::Prompt);
    let (mut session, server) = play_against(&packs, |mut stream| {
        let push = framed(
            PLAY_CLIENTBOUND_RESOURCE_PACK_PUSH_PACKET_ID,
            &push_body("gopher://example.com/pack", false),
        );
        stream.write_all(&push).expect("push");
        let seen = statuses(&mut stream, PLAY_SERVERBOUND_RESOURCE_PACK_PACKET_ID, 1);
        assert_eq!(seen, [PackAction::InvalidUrl as u8]);
    });
    session.wait(Duration::from_secs(5)).expect("push");
    assert!(packs.take_prompt().is_none());
    server.join().expect("server");
}

#[test]
fn an_enabled_pack_is_accepted_downloaded_and_loaded() {
    let packs = source(Ok(PathBuf::from("/cache/pack")), PackStatus::Enabled);
    let (mut session, server) = play_against(&packs, |mut stream| {
        let push = framed(
            PLAY_CLIENTBOUND_RESOURCE_PACK_PUSH_PACKET_ID,
            &push_body("https://example.com/pack.zip", false),
        );
        stream.write_all(&push).expect("push");
        let seen = statuses(&mut stream, PLAY_SERVERBOUND_RESOURCE_PACK_PACKET_ID, 3);
        assert_eq!(
            seen,
            [
                PackAction::Accepted as u8,
                PackAction::Downloaded as u8,
                PackAction::SuccessfullyLoaded as u8
            ]
        );
    });
    session.wait(Duration::from_secs(5)).expect("push");
    let loaded = wait_for_reload(&packs);
    assert_eq!(loaded, [(PACK, PathBuf::from("/cache/pack"))]);
    packs.reload_succeeded();
    server.join().expect("server");
}

#[test]
fn a_pack_that_cannot_be_fetched_is_reported() {
    let packs = source(Err("no route".into()), PackStatus::Enabled);
    let (mut session, server) = play_against(&packs, |mut stream| {
        let push = framed(
            PLAY_CLIENTBOUND_RESOURCE_PACK_PUSH_PACKET_ID,
            &push_body("https://example.com/pack.zip", false),
        );
        stream.write_all(&push).expect("push");
        let seen = statuses(&mut stream, PLAY_SERVERBOUND_RESOURCE_PACK_PACKET_ID, 2);
        assert_eq!(
            seen,
            [PackAction::Accepted as u8, PackAction::FailedDownload as u8]
        );
    });
    session.wait(Duration::from_secs(5)).expect("push");
    server.join().expect("server");
}

fn wait_for_reload(packs: &PackSource) -> Vec<(Uuid, PathBuf)> {
    let started = std::time::Instant::now();
    loop {
        if let Some(packs) = packs.take_reload() {
            return packs;
        }
        assert!(started.elapsed() < Duration::from_secs(5), "no reload");
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The configuration phase run against a server played by `serve`, on a thread of its own.
fn configure_against(
    packs: &PackSource,
    serve: impl FnOnce(Transport) + Send + 'static,
) -> (
    std::thread::JoinHandle<Result<(), ConnectionError>>,
    std::thread::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let address = listener.local_addr().expect("address");
    let server = std::thread::spawn(move || {
        serve(Transport::from(listener.accept().expect("accept").0));
    });
    let mut stream = Transport::from(TcpStream::connect(address).expect("connect"));
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("timeout");
    let mut state = ConnectionState {
        packs: packs.clone(),
        ..ConnectionState::default()
    };
    let client = std::thread::spawn(move || {
        configure_connection(
            &mut stream,
            None,
            (Duration::from_secs(5), true),
            (&mut state, &mut NoRecorder),
        )
    });
    (client, server)
}

#[test]
fn a_pack_is_loaded_during_configuration_before_the_server_finishes_it() {
    let packs = source(Ok(PathBuf::from("/cache/pack")), PackStatus::Enabled);
    let (client, server) = configure_against(&packs, |mut stream| {
        let push = framed(
            CONFIG_CLIENTBOUND_RESOURCE_PACK_PUSH_PACKET_ID,
            &push_body("https://example.com/pack.zip", true),
        );
        stream.write_all(&push).expect("push");
        // The server waits for the pack's last word before it finishes.
        let seen = statuses(&mut stream, CONFIG_SERVERBOUND_RESOURCE_PACK_PACKET_ID, 3);
        assert_eq!(seen[2], PackAction::SuccessfullyLoaded as u8);
        stream
            .write_all(&framed(CONFIG_FINISH_PACKET_ID, &[]))
            .expect("finish");
    });
    wait_for_reload(&packs);
    packs.reload_succeeded();
    client.join().expect("client").expect("configured");
    server.join().expect("server");
}

#[test]
fn refusing_a_required_pack_during_configuration_ends_the_connection() {
    let packs = source(Err("never".into()), PackStatus::Prompt);
    let (client, server) = configure_against(&packs, |mut stream| {
        let push = framed(
            CONFIG_CLIENTBOUND_RESOURCE_PACK_PUSH_PACKET_ID,
            &push_body("https://example.com/pack.zip", true),
        );
        stream.write_all(&push).expect("push");
        let seen = statuses(&mut stream, CONFIG_SERVERBOUND_RESOURCE_PACK_PACKET_ID, 1);
        assert_eq!(seen, [PackAction::Declined as u8]);
    });
    let started = std::time::Instant::now();
    let prompt = loop {
        if let Some(prompt) = packs.take_prompt() {
            break prompt;
        }
        assert!(started.elapsed() < Duration::from_secs(5), "no question");
        std::thread::sleep(Duration::from_millis(5));
    };
    assert!(prompt.required);
    let outcome = packs.answer_prompt(false);
    assert_eq!(outcome.remember, None);
    let error = client.join().expect("client").expect_err("ended");
    assert!(matches!(
        error,
        ConnectionError::Disconnected(reason)
            if *reason == photon_game::text::Component::translatable(
                "multiplayer.requiredTexturePrompt.disconnect"
            )
    ));
    server.join().expect("server");
}
