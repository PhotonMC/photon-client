//! The server taking the connection back to the configuration phase
//! (`ClientPacketListener.handleConfigurationStart`): the world is let go, the configuration is
//! run again and a new play phase starts.

use std::time::Duration;

use super::lifecycle::{self, spawn_reader};
use super::*;
use crate::connection::open::configure_connection;
use crate::connection::record::NoRecorder;

/// How long the configuration may take, the same wait the first one has.
const RECONFIGURATION_TIMEOUT: Duration = Duration::from_secs(30);

impl PlaySession {
    /// `handleResourcePackPush`: an address that is not a web one is answered at once; the rest
    /// wait to be asked about.
    pub(super) fn on_resource_pack_push(
        &mut self,
        request: PackRequest,
    ) -> Result<(), ConnectionError> {
        if request.has_valid_url() {
            self.pack_requests.push(request);
            Ok(())
        } else {
            self.answer_resource_pack(request.id, PackAction::InvalidUrl)
        }
    }

    /// `handleRequestCookie`: what is kept under the key goes back, or nothing.
    pub(super) fn on_cookie_request(
        &mut self,
        request: &photon_protocol::cookie::ClientboundCookieRequestPacket,
    ) -> Result<(), ConnectionError> {
        let answer = self.state.cookies.answer(request);
        self.send(PLAY_SERVERBOUND_COOKIE_RESPONSE_PACKET_ID, |output| {
            answer
                .encode(output)
                .map_err(|_| ConnectionError::Codec("cookie response"))
        })
    }

    /// `handleTransfer`: the connection is let go and another made to the host the server names,
    /// as a transfer, with the cookies kept. The new session starts over like a reconfigured one.
    pub(super) fn transfer(
        &mut self,
        packet: photon_protocol::cookie::ClientboundTransferPacket,
    ) -> Result<(), ConnectionError> {
        let address = lifecycle::resolve_host(&packet.host, packet.port)?;
        let carried = ConnectionState {
            cookies: self.state.cookies.clone(),
            transferring: true,
            ..ConnectionState::default()
        };
        let mut fresh = Self::connect_through(
            (address, Some(packet.host)),
            (&self.username, self.auto_jump),
            RECONFIGURATION_TIMEOUT,
            self.client_information.clone(),
            carried,
        )?;
        fresh.lang = std::sync::Arc::clone(&self.lang);
        fresh.level_reset = true;
        fresh.reconfigured = true;
        *self = fresh;
        Ok(())
    }

    /// Acknowledge the server, run the configuration phase on the connection and start over in
    /// the play phase with a world of nothing yet known. The reader thread has stopped by the
    /// terminal packet, so the connection is this call's alone until it ends.
    pub(super) fn reconfigure(&mut self) -> Result<(), ConnectionError> {
        self.send(
            PLAY_SERVERBOUND_CONFIGURATION_ACKNOWLEDGED_PACKET_ID,
            |_| Ok(()),
        )?;
        let mut stream = self.writer.try_clone()?;
        stream.set_read_timeout(Some(RECONFIGURATION_TIMEOUT))?;
        // What the connection learned at login stays; the registries, tags and features are the
        // new configuration's.
        let mut state = ConnectionState {
            profile_id: self.state.profile_id,
            cookies: self.state.cookies.clone(),
            server_brand: self.state.server_brand.clone(),
            custom_report_details: self.state.custom_report_details.clone(),
            ..ConnectionState::default()
        };
        configure_connection(
            &mut stream,
            self.compression_threshold,
            (RECONFIGURATION_TIMEOUT, true),
            (&mut state, &mut NoRecorder),
        )?;
        stream.set_read_timeout(None)?;
        let incoming = spawn_reader(stream, self.compression_threshold);
        let mut fresh = Self::assemble(
            self.writer.try_clone()?,
            self.compression_threshold,
            incoming,
            state,
            (&self.username, self.auto_jump),
        );
        fresh.client_information = self.client_information.clone();
        fresh.lang = std::sync::Arc::clone(&self.lang);
        // Nothing of the old level stays.
        fresh.level_reset = true;
        fresh.reconfigured = true;
        *self = fresh;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;

    use super::*;

    fn frame(id: i32) -> Vec<u8> {
        let mut body = Vec::new();
        wire::write_varint(id, &mut body);
        let mut framed = Vec::new();
        wire::write_varint(body.len() as i32, &mut framed);
        framed.extend_from_slice(&body);
        framed
    }

    #[test]
    fn the_server_taking_the_connection_back_to_configuration_starts_a_new_world() {
        use std::io::Write;
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let address = listener.local_addr().expect("address");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            // The terminal play packet, then the configuration's finish.
            stream
                .write_all(&frame(PLAY_CLIENTBOUND_START_CONFIGURATION_PACKET_ID))
                .expect("start configuration");
            let acknowledged = read_packet(&mut stream, None).expect("acknowledgement");
            let (id, _) = split_packet(&acknowledged).expect("split");
            assert_eq!(id, PLAY_SERVERBOUND_CONFIGURATION_ACKNOWLEDGED_PACKET_ID);
            stream
                .write_all(&frame(CONFIG_FINISH_PACKET_ID))
                .expect("finish");
            let finished = read_packet(&mut stream, None).expect("finish answer");
            let (id, _) = split_packet(&finished).expect("split");
            assert_eq!(id, CONFIG_FINISH_PACKET_ID);
        });
        let stream = TcpStream::connect(address).expect("connect");
        let incoming = spawn_reader(stream.try_clone().expect("clone"), None);
        let mut session = PlaySession::assemble(
            stream,
            None,
            incoming,
            ConnectionState::default(),
            ("Tester", false),
        );
        session.wait(Duration::from_secs(5)).expect("handled");
        assert!(session.take_level_reset());
        server.join().expect("server");
    }

    #[test]
    fn a_cookie_the_server_stored_is_given_back_when_it_asks() {
        use std::io::Write;
        let key = photon_core::identifier::Identifier::parse("example:token").expect("key");
        let mut stored = Vec::new();
        photon_core::identifier::write_identifier(&key, &mut stored).expect("key");
        wire::write_varint(2, &mut stored);
        stored.extend_from_slice(&[7, 9]);
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let address = listener.local_addr().expect("address");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut with_id = |id: i32, body: &[u8]| {
                let mut packet = Vec::new();
                wire::write_varint(id, &mut packet);
                packet.extend_from_slice(body);
                let mut framed = Vec::new();
                wire::write_varint(packet.len() as i32, &mut framed);
                framed.extend_from_slice(&packet);
                stream.write_all(&framed).expect("write");
            };
            let mut request = Vec::new();
            photon_core::identifier::write_identifier(
                &photon_core::identifier::Identifier::parse("example:token").expect("key"),
                &mut request,
            )
            .expect("key");
            with_id(PLAY_CLIENTBOUND_STORE_COOKIE_PACKET_ID, &stored);
            with_id(PLAY_CLIENTBOUND_COOKIE_REQUEST_PACKET_ID, &request);
            let answer = read_packet(&mut stream, None).expect("response");
            let (id, body) = split_packet(&answer).expect("split");
            assert_eq!(id, PLAY_SERVERBOUND_COOKIE_RESPONSE_PACKET_ID);
            assert!(body.ends_with(&[1, 2, 7, 9]));
        });
        let stream = TcpStream::connect(address).expect("connect");
        let incoming = spawn_reader(stream.try_clone().expect("clone"), None);
        let mut session = PlaySession::assemble(
            stream,
            None,
            incoming,
            ConnectionState::default(),
            ("Tester", false),
        );
        session.wait(Duration::from_secs(5)).expect("store");
        session.wait(Duration::from_secs(5)).expect("request");
        server.join().expect("server");
        assert!(session.state.cookies.get(&key).is_some());
    }

    #[test]
    fn a_transfer_connects_to_the_named_host_with_the_transfer_intention() {
        use std::io::Write;
        let target = TcpListener::bind("127.0.0.1:0").expect("target");
        let target_port = target.local_addr().expect("address").port();
        let origin = TcpListener::bind("127.0.0.1:0").expect("origin");
        let origin_address = origin.local_addr().expect("address");
        let destination = std::thread::spawn(move || {
            let (mut stream, _) = target.accept().expect("accept");
            let handshake = read_packet(&mut stream, None).expect("handshake");
            let (id, mut body) = split_packet(&handshake).expect("split");
            assert_eq!(id, LOGIN_INTENTION_PACKET_ID);
            let intention = photon_protocol::handshake::ClientIntentionPacket::decode(&mut body)
                .expect("decode");
            assert_eq!(
                String::from_utf16(&intention.host_name).expect("host"),
                "127.0.0.1"
            );
            assert_eq!(intention.port, i32::from(target_port));
            assert_eq!(
                intention.intention,
                photon_protocol::handshake::ClientIntent::Transfer
            );
        });
        let origin = std::thread::spawn(move || {
            let (mut stream, _) = origin.accept().expect("accept");
            let mut body = Vec::new();
            photon_core::utf8::write_utf8(
                &mut body,
                &"127.0.0.1".encode_utf16().collect::<Vec<_>>(),
                32_767,
            )
            .expect("host");
            wire::write_varint(i32::from(target_port), &mut body);
            let mut packet = Vec::new();
            wire::write_varint(PLAY_CLIENTBOUND_TRANSFER_PACKET_ID, &mut packet);
            packet.extend_from_slice(&body);
            let mut framed = Vec::new();
            wire::write_varint(packet.len() as i32, &mut framed);
            framed.extend_from_slice(&packet);
            stream.write_all(&framed).expect("transfer");
            std::thread::sleep(Duration::from_millis(300));
        });
        let stream = TcpStream::connect(origin_address).expect("connect");
        let incoming = spawn_reader(stream.try_clone().expect("clone"), None);
        let mut session = PlaySession::assemble(
            stream,
            None,
            incoming,
            ConnectionState::default(),
            ("Tester", false),
        );
        // The destination hangs up before it logs the player in, so the transfer fails there.
        assert!(session.wait(Duration::from_secs(5)).is_err());
        destination.join().expect("destination");
        origin.join().expect("origin");
    }

    fn pack_push_body(url: &str) -> Vec<u8> {
        let mut body = Vec::new();
        photon_core::binary::write_uuid(photon_core::binary::Uuid::new(1, 2), &mut body);
        for text in [url, "0123456789abcdef0123456789abcdef01234567"] {
            photon_core::utf8::write_utf8(
                &mut body,
                &text.encode_utf16().collect::<Vec<_>>(),
                32_767,
            )
            .expect("string");
        }
        body.extend_from_slice(&[0, 0]);
        body
    }

    #[test]
    fn a_pack_the_server_pushes_waits_for_an_answer_and_a_bad_address_is_refused_at_once() {
        use std::io::Write;
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let address = listener.local_addr().expect("address");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            for url in ["https://example.com/pack.zip", "gopher://example.com/pack"] {
                let mut packet = Vec::new();
                wire::write_varint(PLAY_CLIENTBOUND_RESOURCE_PACK_PUSH_PACKET_ID, &mut packet);
                packet.extend_from_slice(&pack_push_body(url));
                let mut framed = Vec::new();
                wire::write_varint(packet.len() as i32, &mut framed);
                framed.extend_from_slice(&packet);
                stream.write_all(&framed).expect("push");
            }
            // Only the bad address is answered by the client alone.
            let answer = read_packet(&mut stream, None).expect("answer");
            let (id, body) = split_packet(&answer).expect("split");
            assert_eq!(id, PLAY_SERVERBOUND_RESOURCE_PACK_PACKET_ID);
            assert_eq!(body[16], PackAction::InvalidUrl as u8);
            let answer = read_packet(&mut stream, None).expect("declined");
            let (_, body) = split_packet(&answer).expect("split");
            assert_eq!(body[16], PackAction::Declined as u8);
        });
        let stream = TcpStream::connect(address).expect("connect");
        let incoming = spawn_reader(stream.try_clone().expect("clone"), None);
        let mut session = PlaySession::assemble(
            stream,
            None,
            incoming,
            ConnectionState::default(),
            ("Tester", false),
        );
        session.wait(Duration::from_secs(5)).expect("first push");
        session.wait(Duration::from_secs(5)).expect("second push");
        let requests = session.take_pack_requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].url, "https://example.com/pack.zip");
        session
            .answer_resource_pack(requests[0].id, PackAction::Declined)
            .expect("answer");
        server.join().expect("server");
    }

    #[test]
    fn a_pack_pushed_during_configuration_is_declined() {
        use std::io::Write;
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let address = listener.local_addr().expect("address");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut packet = Vec::new();
            wire::write_varint(CONFIG_CLIENTBOUND_RESOURCE_PACK_PUSH_PACKET_ID, &mut packet);
            packet.extend_from_slice(&pack_push_body("https://example.com/pack.zip"));
            let mut framed = Vec::new();
            wire::write_varint(packet.len() as i32, &mut framed);
            framed.extend_from_slice(&packet);
            stream.write_all(&framed).expect("push");
            stream
                .write_all(&frame(CONFIG_FINISH_PACKET_ID))
                .expect("finish");
            let answer = read_packet(&mut stream, None).expect("answer");
            let (id, body) = split_packet(&answer).expect("split");
            assert_eq!(id, CONFIG_SERVERBOUND_RESOURCE_PACK_PACKET_ID);
            assert_eq!(body[16], PackAction::Declined as u8);
        });
        let mut stream = TcpStream::connect(address).expect("connect");
        let mut state = ConnectionState::default();
        configure_connection(
            &mut stream,
            None,
            (Duration::from_secs(5), true),
            (&mut state, &mut NoRecorder),
        )
        .expect("configured");
        server.join().expect("server");
    }
}
