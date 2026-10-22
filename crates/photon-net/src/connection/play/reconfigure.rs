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
            self.social.pack_requests.push(request);
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
            account: self.state.account.clone(),
            cookies: self.state.cookies.clone(),
            transferring: true,
            ..ConnectionState::default()
        };
        let mut fresh = Self::connect_through(
            (address, Some(packet.host)),
            (&self.link.username, self.link.auto_jump),
            RECONFIGURATION_TIMEOUT,
            self.link.client_information.clone(),
            carried,
        )?;
        fresh.social.lang = std::sync::Arc::clone(&self.social.lang);
        fresh.level_info.level_reset = true;
        fresh.level_info.level_swapped = true;
        fresh.level_info.reconfigured = true;
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
        // The acknowledgement is on the wire before the connection is used directly.
        self.link.writer.flush()?;
        let mut stream = self.link.writer.stream()?;
        stream.set_read_timeout(Some(RECONFIGURATION_TIMEOUT))?;
        // What the connection learned at login stays; the registries, tags and features are the
        // new configuration's.
        let mut state = ConnectionState {
            profile_id: self.state.profile_id,
            profile_name: self.state.profile_name.clone(),
            cookies: self.state.cookies.clone(),
            server_brand: self.state.server_brand.clone(),
            custom_report_details: self.state.custom_report_details.clone(),
            // `CommonListenerCookie.serverLinks` goes on to the configuration listener.
            server_links: self.state.server_links.clone(),
            ..ConnectionState::default()
        };
        configure_connection(
            &mut stream,
            self.link.compression_threshold,
            (RECONFIGURATION_TIMEOUT, true),
            (&mut state, &mut NoRecorder),
        )?;
        stream.set_read_timeout(None)?;
        let incoming = spawn_reader(stream, self.link.compression_threshold);
        let mut fresh = Self::assemble(
            self.link.writer.stream()?,
            self.link.compression_threshold,
            incoming,
            state,
            (&self.link.username, self.link.auto_jump),
        );
        fresh.link.client_information = self.link.client_information.clone();
        fresh.social.lang = std::sync::Arc::clone(&self.social.lang);
        // Nothing of the old level stays.
        fresh.level_info.level_reset = true;
        fresh.level_info.level_swapped = true;
        fresh.level_info.reconfigured = true;
        *self = fresh;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{TcpListener, TcpStream};

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
            let mut stream = Transport::from(listener.accept().expect("accept").0);
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
        let stream = Transport::from(TcpStream::connect(address).expect("connect"));
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

    fn framed(id: i32, body: &[u8]) -> Vec<u8> {
        let mut packet = Vec::new();
        wire::write_varint(id, &mut packet);
        packet.extend_from_slice(body);
        let mut out = Vec::new();
        wire::write_varint(packet.len() as i32, &mut out);
        out.extend_from_slice(&packet);
        out
    }

    /// A dimension type entry with the fields the client reads.
    fn dimension_type_registry(min_y: i32, height: i32) -> Vec<u8> {
        use photon_core::nbt::{Field, FieldValue};
        let field = |key: &str, value| Field {
            key: key.to_owned(),
            value,
        };
        let entry = photon_protocol::registry_data::PackedRegistryEntry::new(
            photon_core::identifier::Identifier::parse("example:lobby").expect("id"),
            Some(FieldValue::Compound(vec![
                field("min_y", FieldValue::Int(min_y)),
                field("height", FieldValue::Int(height)),
                field("has_skylight", FieldValue::Byte(1)),
                field("has_ceiling", FieldValue::Byte(0)),
                field("has_ender_dragon_fight", FieldValue::Byte(0)),
                field("coordinate_scale", FieldValue::Double(1.0)),
                field("logical_height", FieldValue::Int(height)),
                field("ambient_light", FieldValue::Float(0.0)),
            ])),
        );
        let mut body = Vec::new();
        photon_protocol::registry_data::ClientboundRegistryDataPacket::new(
            photon_core::identifier::Identifier::parse("minecraft:dimension_type").expect("id"),
            vec![entry],
        )
        .encode(&mut body)
        .expect("registry");
        framed(CONFIG_REGISTRY_DATA_PACKET_ID, &body)
    }

    fn login_into(dimension_type_id: i32) -> Vec<u8> {
        let identifier = |body: &mut Vec<u8>, text: &str| {
            photon_core::identifier::write_identifier(
                &photon_core::identifier::Identifier::parse(text).expect("id"),
                body,
            )
            .expect("identifier");
        };
        let mut body = Vec::new();
        body.extend_from_slice(&7_i32.to_be_bytes());
        body.push(0);
        wire::write_varint(1, &mut body);
        identifier(&mut body, "example:lobby");
        for value in [20, 8, 8] {
            wire::write_varint(value, &mut body);
        }
        body.extend_from_slice(&[0, 1, 0]);
        wire::write_varint(dimension_type_id, &mut body);
        identifier(&mut body, "example:lobby");
        body.extend_from_slice(&1_i64.to_be_bytes());
        body.extend_from_slice(&[1, u8::MAX, 0, 0, 0]);
        wire::write_varint(0, &mut body);
        wire::write_varint(63, &mut body);
        body.extend_from_slice(&[0, 0]);
        framed(PLAY_CLIENTBOUND_LOGIN_PACKET_ID, &body)
    }

    /// A session that has the first server's two dimension types logs in again after the
    /// second server's configuration, which has one type of another height.
    fn log_in_again_into(dimension_type_id: i32) -> PlaySession {
        use std::io::Write;
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let address = listener.local_addr().expect("address");
        std::thread::spawn(move || {
            let mut stream = Transport::from(listener.accept().expect("accept").0);
            stream
                .write_all(&frame(PLAY_CLIENTBOUND_START_CONFIGURATION_PACKET_ID))
                .expect("start configuration");
            read_packet(&mut stream, None).expect("acknowledgement");
            stream
                .write_all(&dimension_type_registry(0, 128))
                .expect("registry");
            stream
                .write_all(&frame(CONFIG_FINISH_PACKET_ID))
                .expect("finish");
            read_packet(&mut stream, None).expect("finish answer");
            stream
                .write_all(&login_into(dimension_type_id))
                .expect("login");
            std::thread::sleep(Duration::from_millis(500));
        });
        let stream = Transport::from(TcpStream::connect(address).expect("connect"));
        let incoming = spawn_reader(stream.try_clone().expect("clone"), None);
        let first = ConnectionState {
            dimension_type_registry_bounds: vec![
                Some(photon_world::DimensionBounds::new(-64, 384)),
                Some(photon_world::DimensionBounds::new(0, 256)),
            ],
            ..ConnectionState::default()
        };
        let mut session = PlaySession::assemble(stream, None, incoming, first, ("Tester", false));
        session.wait(Duration::from_secs(5)).expect("reconfigured");
        session.wait(Duration::from_secs(5)).expect("logged in");
        session
    }

    #[test]
    fn a_second_login_uses_the_new_configurations_dimension_types() {
        let mut session = log_in_again_into(0);
        assert_eq!(session.state.dimension_type_registry_bounds.len(), 1);
        let bounds = session.world().dimension();
        assert_eq!((bounds.min_y, bounds.height), (0, 128));
        assert!(session.take_reconfigured());
        assert!(session.take_level_reset());
    }

    #[test]
    fn a_second_login_naming_a_type_the_first_configuration_had_does_not_panic() {
        // Id 1 existed in the first configuration only; the built-in extent stands in.
        let session = log_in_again_into(1);
        assert_eq!(session.state.dimension_type_registry_bounds.len(), 1);
        assert!(session.world().dimension().height > 0);
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
            let mut stream = Transport::from(listener.accept().expect("accept").0);
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
        let stream = Transport::from(TcpStream::connect(address).expect("connect"));
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
            let mut stream = Transport::from(target.accept().expect("accept").0);
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
            let mut stream = Transport::from(origin.accept().expect("accept").0);
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
        let stream = Transport::from(TcpStream::connect(origin_address).expect("connect"));
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
            let mut stream = Transport::from(listener.accept().expect("accept").0);
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
        let stream = Transport::from(TcpStream::connect(address).expect("connect"));
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
            let mut stream = Transport::from(listener.accept().expect("accept").0);
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
        let mut stream = Transport::from(TcpStream::connect(address).expect("connect"));
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
