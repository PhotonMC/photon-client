//! The `photon:` custom payloads over a play connection, against a server played by the test
//! (ADR 0024): a server's mod list reaches the app, the client's hello reaches the server.

use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use photon_core::identifier::{Identifier, write_identifier};

use super::lifecycle::spawn_reader;
use super::*;

fn framed(id: i32, body: &[u8]) -> Vec<u8> {
    let mut packet = Vec::new();
    wire::write_varint(id, &mut packet);
    packet.extend_from_slice(body);
    let mut out = Vec::new();
    wire::write_varint(packet.len() as i32, &mut out);
    out.extend_from_slice(&packet);
    out
}

fn payload(channel: &str, data: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    let id = Identifier::parse(channel).expect("channel");
    write_identifier(&id, &mut body).expect("identifier");
    body.extend_from_slice(data);
    framed(
        crate::connection::test_wire::play(PLAY_CLIENTBOUND_CUSTOM_PAYLOAD_PACKET_ID),
        &body,
    )
}

fn play_against(
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
        crate::connection::play::version_tests::registries(),
    );
    (
        PlaySession::assemble(
            stream,
            None,
            incoming,
            ConnectionState::default(),
            ("Tester", false),
        ),
        server,
    )
}

#[test]
fn photon_payloads_reach_the_app_and_other_namespaces_do_not() {
    let (mut session, server) = play_against(|mut stream| {
        stream
            .write_all(&payload("photon:mods", b"{\"v\":1,\"mods\":[]}"))
            .expect("mods");
        stream
            .write_all(&payload("someone:else", b"ignored"))
            .expect("other");
        stream
            .write_all(&payload("photon:more", b"x"))
            .expect("more");
    });
    let started = std::time::Instant::now();
    let mut taken = Vec::new();
    while taken.len() < 2 && started.elapsed() < Duration::from_secs(5) {
        session.wait(Duration::from_millis(50)).ok();
        taken.extend(session.take_photon_payloads());
    }
    server.join().expect("server");
    let channels: Vec<&str> = taken.iter().map(|(channel, _)| channel.as_str()).collect();
    assert_eq!(channels, ["photon:mods", "photon:more"]);
    assert_eq!(taken[0].1, b"{\"v\":1,\"mods\":[]}");
    // Taken once.
    assert!(session.take_photon_payloads().is_empty());
}

#[test]
fn the_hello_goes_to_the_server_as_a_serverbound_custom_payload() {
    let (mut session, server) = play_against(|mut stream| {
        let packet = read_packet(&mut stream, None).expect("hello");
        let (id, body) = split_packet(&packet).expect("split");
        assert_eq!(id, PLAY_SERVERBOUND_CUSTOM_PAYLOAD_PACKET_ID);
        let mut input = body;
        let channel = photon_core::identifier::read_identifier(&mut input).expect("channel");
        assert_eq!(channel.as_string(), "photon:hello");
        assert_eq!(input, b"{\"v\":1}");
    });
    session
        .send_photon_payload("photon:hello", b"{\"v\":1}")
        .expect("send");
    server.join().expect("server");
}
