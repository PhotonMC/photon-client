//! The server taking the connection back to the configuration phase
//! (`ClientPacketListener.handleConfigurationStart`): the world is let go, the configuration is
//! run again and a new play phase starts.

use std::sync::mpsc;
use std::time::Duration;

use super::lifecycle::{self, spawn_reader};
use super::*;
use crate::connection::open::configure_connection;
use crate::connection::record::NoRecorder;

/// How long the configuration may take, the same wait the first one has.
const RECONFIGURATION_TIMEOUT: Duration = Duration::from_secs(30);

impl PlaySession {
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
    /// as a transfer, with the cookies kept, on its own thread so the window goes on. The new
    /// session starts over like a reconfigured one when it arrives ([`Self::poll_switch`]).
    pub(super) fn transfer(
        &mut self,
        packet: photon_protocol::cookie::ClientboundTransferPacket,
    ) -> Result<(), ConnectionError> {
        let stage = crate::connection::LoginStage::default();
        let carried = ConnectionState {
            version: self.registries.version(),
            account: self.state.account.clone(),
            cookies: self.state.cookies.clone(),
            packs: self.state.packs.clone(),
            login_stage: Some(stage.clone()),
            transferring: true,
            seen_players: self.social.seen_players.clone(),
            seen_insecure_warning: self.effects.seen_unsecure_warning,
            ..ConnectionState::default()
        };
        // `Connection.disconnect`: the old connection ends before the new one is made.
        self.link.writer.flush()?;
        self.link.writer.stream()?.shutdown();
        let username = self.link.username.clone();
        let auto_jump = self.link.auto_jump;
        let information = self.link.client_information.clone();
        let (send, result) = mpsc::channel();
        std::thread::Builder::new()
            .name("Server Connector".to_owned())
            .spawn(move || {
                let _ = send.send(transfer_to(
                    (packet.host, packet.port),
                    (&username, auto_jump),
                    information,
                    carried,
                ));
            })?;
        self.link.switching = Some(Switch {
            kind: SwitchKind::Transferring,
            result,
            abort: None,
            stage: Some(stage),
        });
        Ok(())
    }

    /// Acknowledge the server and run the configuration phase on the connection on its own
    /// thread, so the window goes on; a new play phase with a world of nothing yet known starts
    /// when it is done ([`Self::poll_switch`]). The reader thread has stopped by the terminal
    /// packet, so the connection is the thread's alone until it ends.
    pub(super) fn reconfigure(&mut self) -> Result<(), ConnectionError> {
        self.send(
            PLAY_SERVERBOUND_CONFIGURATION_ACKNOWLEDGED_PACKET_ID,
            |_| Ok(()),
        )?;
        // The acknowledgement is on the wire before the connection is used directly.
        self.link.writer.flush()?;
        let stream = self.link.writer.stream()?;
        let abort = stream.try_clone()?;
        // What the connection learned at login stays; the registries, tags and features are the
        // new configuration's.
        let state = ConnectionState {
            version: self.registries.version(),
            account: self.state.account.clone(),
            profile_id: self.state.profile_id,
            profile_name: self.state.profile_name.clone(),
            cookies: self.state.cookies.clone(),
            packs: self.state.packs.clone(),
            server_brand: self.state.server_brand.clone(),
            custom_report_details: self.state.custom_report_details.clone(),
            // `CommonListenerCookie.serverLinks` goes on to the configuration listener.
            server_links: self.state.server_links.clone(),
            seen_players: self.social.seen_players.clone(),
            seen_insecure_warning: self.effects.seen_unsecure_warning,
            ..ConnectionState::default()
        };
        let writer = self.link.writer.stream()?;
        let threshold = self.link.compression_threshold;
        let username = self.link.username.clone();
        let auto_jump = self.link.auto_jump;
        let information = self.link.client_information.clone();
        let (send, result) = mpsc::channel();
        std::thread::Builder::new()
            .name("Reconfiguration".to_owned())
            .spawn(move || {
                let _ = send.send(configure_again(
                    (stream, writer, threshold),
                    state,
                    (&username, auto_jump),
                    information,
                ));
            })?;
        self.link.switching = Some(Switch {
            kind: SwitchKind::Reconfiguring,
            result,
            abort: Some(abort),
            stage: None,
        });
        Ok(())
    }

    /// What the connection is doing besides playing, while it is.
    pub fn switching(&self) -> Option<SwitchKind> {
        self.link.switching.as_ref().map(|switch| switch.kind)
    }

    /// The login stage (`connection::stage`) a transfer's new connection has come to.
    pub fn switch_stage(&self) -> Option<u8> {
        let stage = self.link.switching.as_ref()?.stage.as_ref()?;
        Some(stage.0.load(std::sync::atomic::Ordering::Relaxed))
    }

    /// Whether the connection ended because a transfer could not be made.
    pub fn transfer_failed(&self) -> bool {
        self.link.transfer_failed
    }

    /// The player ends a reconfiguration or a transfer (`ABORT_CONNECTION`): the thread's
    /// connection is cut and its outcome is no longer waited for.
    pub fn abort_switch(&mut self) {
        let Some(switch) = self.link.switching.take() else {
            return;
        };
        if let Some(stream) = switch.abort {
            stream.shutdown();
        }
        self.link.transfer_failed = switch.kind == SwitchKind::Transferring;
        self.disconnect(photon_game::text::Component::translatable(
            "connect.aborted",
        ));
    }

    /// Wait up to `limit` for the switch to end; a new session replaces this one when it has, and
    /// a failure ends the connection with what went wrong.
    pub(super) fn poll_switch(&mut self, limit: Duration) -> Result<(), ConnectionError> {
        let Some(switch) = self.link.switching.as_ref() else {
            return Ok(());
        };
        let kind = switch.kind;
        match switch.result.recv_timeout(limit) {
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Ok(Ok(mut fresh)) => {
                fresh.social.lang = std::sync::Arc::clone(&self.social.lang);
                // Nothing of the old level stays.
                fresh.level_info.level_reset = true;
                fresh.level_info.level_swapped = true;
                fresh.level_info.reconfigured = true;
                fresh.level_info.chat_reset =
                    kind == SwitchKind::Transferring || fresh.state.chat_reset;
                *self = fresh;
            }
            Ok(Err(error)) => self.end_switch(kind, &error),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                self.end_switch(kind, &ConnectionError::ServerDisconnected);
            }
        }
        Ok(())
    }

    /// `ClientCommonPacketListenerImpl.onDisconnect` for a switch that failed.
    fn end_switch(&mut self, kind: SwitchKind, error: &ConnectionError) {
        use photon_game::text::Component;
        self.link.switching = None;
        self.link.transfer_failed = kind == SwitchKind::Transferring;
        let message = match (kind, error) {
            (_, ConnectionError::Disconnected(reason)) => {
                self.link.disconnect = Some((**reason).clone());
                return;
            }
            (_, ConnectionError::ServerDisconnected) => {
                self.link.disconnect = Some(Component::translatable("disconnect.endOfStream"));
                return;
            }
            (SwitchKind::Transferring, error) => error.to_string(),
            (SwitchKind::Reconfiguring, error) => format!("Internal Exception: {error}"),
        };
        self.link.disconnect = Some(Component::translatable_with(
            "disconnect.genericReason",
            vec![Component::literal(message)],
        ));
    }
}

/// What a play connection in a switch is called.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchKind {
    /// `ServerReconfigScreen`: the configuration phase runs again.
    Reconfiguring,
    /// `ConnectScreen` with `connect.transferring`: another server is being joined.
    Transferring,
}

/// A reconfiguration or a transfer on its own thread.
pub(super) struct Switch {
    kind: SwitchKind,
    result: mpsc::Receiver<Result<PlaySession, ConnectionError>>,
    /// The connection the thread reads, to be cut where the player gives up.
    abort: Option<Transport>,
    /// The login stage of a transfer's new connection.
    stage: Option<crate::connection::LoginStage>,
}

/// The configuration phase again on a connection, then a play phase on it.
fn configure_again(
    (mut stream, writer, compression_threshold): (Transport, Transport, Option<usize>),
    mut state: ConnectionState,
    (username, auto_jump): (&str, bool),
    information: photon_protocol::configuration::ClientInformation,
) -> Result<PlaySession, ConnectionError> {
    stream.set_read_timeout(Some(RECONFIGURATION_TIMEOUT))?;
    let configured = configure_connection(
        &mut stream,
        compression_threshold,
        (RECONFIGURATION_TIMEOUT, true),
        (&mut state, &mut NoRecorder),
    );
    match configured {
        // A transfer in the configuration phase is followed like one in the play phase.
        Err(ConnectionError::Transfer { host, port }) => {
            let carried = ConnectionState {
                version: state.version,
                account: state.account,
                cookies: state.cookies,
                packs: state.packs,
                login_stage: state.login_stage,
                transferring: true,
                seen_players: state.seen_players,
                seen_insecure_warning: state.seen_insecure_warning,
                ..ConnectionState::default()
            };
            transfer_to((host, port), (username, auto_jump), information, carried)
        }
        Err(error) => Err(error),
        Ok(_) => {
            stream.set_read_timeout(None)?;
            let incoming = spawn_reader(
                stream,
                compression_threshold,
                VersionRegistries::of(state.version),
            );
            let mut fresh = PlaySession::assemble(
                writer,
                compression_threshold,
                incoming,
                state,
                (username, auto_jump),
            );
            fresh.link.client_information = information;
            Ok(fresh)
        }
    }
}

/// The connection to the host a transfer names, with what is carried over.
fn transfer_to(
    (host, port): (String, i32),
    (username, auto_jump): (&str, bool),
    information: photon_protocol::configuration::ClientInformation,
    carried: ConnectionState,
) -> Result<PlaySession, ConnectionError> {
    let address = lifecycle::resolve_host(&host, port)?;
    PlaySession::connect_through(
        (address, Some(host)),
        (username, auto_jump),
        RECONFIGURATION_TIMEOUT,
        information,
        carried,
    )
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
        let incoming = spawn_reader(
            stream.try_clone().expect("clone"),
            None,
            crate::connection::play::version_tests::registries(),
        );
        let mut session = PlaySession::assemble(
            stream,
            None,
            incoming,
            ConnectionState::default(),
            ("Tester", false),
        );
        session.wait(Duration::from_secs(5)).expect("handled");
        assert_eq!(session.switching(), Some(SwitchKind::Reconfiguring));
        session.wait(Duration::from_secs(5)).expect("reconfigured");
        assert_eq!(session.switching(), None);
        assert!(session.take_level_reset());
        server.join().expect("server");
    }

    /// A session whose server asks for the configuration phase and then does `then` with the
    /// connection after the acknowledgement.
    fn reconfiguring_against(then: fn(Transport)) -> PlaySession {
        reconfiguring_from(ConnectionState::default(), then)
    }

    fn reconfiguring_from(state: ConnectionState, then: fn(Transport)) -> PlaySession {
        use std::io::Write;
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let address = listener.local_addr().expect("address");
        std::thread::spawn(move || {
            let mut stream = Transport::from(listener.accept().expect("accept").0);
            stream
                .write_all(&frame(PLAY_CLIENTBOUND_START_CONFIGURATION_PACKET_ID))
                .expect("start configuration");
            read_packet(&mut stream, None).expect("acknowledgement");
            then(stream);
        });
        let stream = Transport::from(TcpStream::connect(address).expect("connect"));
        let incoming = spawn_reader(
            stream.try_clone().expect("clone"),
            None,
            crate::connection::play::version_tests::registries(),
        );
        let mut session = PlaySession::assemble(stream, None, incoming, state, ("Tester", false));
        session.wait(Duration::from_secs(5)).expect("started");
        assert_eq!(session.switching(), Some(SwitchKind::Reconfiguring));
        session
    }

    fn finish_configuration(mut stream: Transport) {
        use std::io::Write;
        stream
            .write_all(&frame(CONFIG_FINISH_PACKET_ID))
            .expect("finish");
        read_packet(&mut stream, None).expect("finish answer");
    }

    #[test]
    fn the_chat_stays_through_a_reconfiguration_the_server_did_not_reset() {
        let mut session = reconfiguring_against(finish_configuration);
        session.wait(Duration::from_secs(5)).expect("reconfigured");
        assert!(!session.take_chat_reset());
    }

    #[test]
    fn a_reset_chat_packet_in_the_configuration_does_not_keep_the_chat() {
        let mut session = reconfiguring_against(|mut stream| {
            use std::io::Write;
            stream
                .write_all(&frame(CONFIG_CLIENTBOUND_RESET_CHAT_PACKET_ID))
                .expect("reset chat");
            finish_configuration(stream);
        });
        session.wait(Duration::from_secs(5)).expect("reconfigured");
        assert!(session.take_chat_reset());
        assert!(!session.take_chat_reset());
    }

    #[test]
    fn the_seen_players_and_the_warning_go_through_a_reconfiguration() {
        let mut state = ConnectionState::default();
        state.seen_players.insert((1, 2), "Ann".to_owned());
        state.seen_insecure_warning = true;
        let mut session = reconfiguring_from(state, finish_configuration);
        session.wait(Duration::from_secs(5)).expect("reconfigured");
        assert_eq!(
            session.social.seen_players.get(&(1, 2)).map(String::as_str),
            Some("Ann")
        );
        assert!(session.effects.seen_unsecure_warning);
    }

    #[test]
    fn the_frame_goes_on_while_the_configuration_waits_for_the_server() {
        let mut session = reconfiguring_against(|stream| {
            std::thread::sleep(Duration::from_millis(400));
            drop(stream);
        });
        // Draining does not wait: the caller is back at once with the switch still going.
        let before = std::time::Instant::now();
        session.drain().expect("drained");
        assert!(before.elapsed() < Duration::from_millis(200));
        assert_eq!(session.switching(), Some(SwitchKind::Reconfiguring));
    }

    #[test]
    fn a_server_hanging_up_during_the_configuration_ends_the_connection_with_a_reason() {
        let mut session = reconfiguring_against(drop);
        session.wait(Duration::from_secs(5)).expect("ended");
        assert_eq!(session.switching(), None);
        assert!(session.disconnected().is_some());
        assert!(!session.transfer_failed());
        // Nothing is read or sent over it any more.
        session.drain().expect("drained");
    }

    #[test]
    fn the_reason_a_server_disconnects_with_during_the_configuration_is_kept() {
        use std::io::Write;
        let mut session = reconfiguring_against(|mut stream| {
            // A plain string component: the string tag with no name.
            let mut body = vec![8, 0, 11];
            body.extend_from_slice(b"Server full");
            stream
                .write_all(&framed(CONFIG_CLIENTBOUND_DISCONNECT_PACKET_ID, &body))
                .expect("disconnect");
        });
        session.wait(Duration::from_secs(5)).expect("ended");
        assert_eq!(session.switching(), None);
        assert_eq!(
            session.disconnected(),
            Some(&photon_game::text::Component::literal("Server full"))
        );
    }

    #[test]
    fn the_reason_a_server_disconnects_with_during_the_login_is_kept() {
        use std::io::Write;
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let address = listener.local_addr().expect("address");
        std::thread::spawn(move || {
            let mut stream = Transport::from(listener.accept().expect("accept").0);
            read_packet(&mut stream, None).expect("handshake");
            read_packet(&mut stream, None).expect("hello");
            let mut body = Vec::new();
            let json = br#"{"text":"You are banned"}"#;
            wire::write_varint(json.len() as i32, &mut body);
            body.extend_from_slice(json);
            stream
                .write_all(&framed(LOGIN_CLIENTBOUND_DISCONNECT_PACKET_ID, &body))
                .expect("disconnect");
        });
        let first = ConnectionState {
            protocol_hint: Some(crate::connection::play::version_tests::protocol()),
            ..ConnectionState::default()
        };
        let error = match PlaySession::connect_with(
            address,
            ("Tester", false),
            Duration::from_secs(5),
            photon_protocol::configuration::ClientInformation::default(),
            first,
        ) {
            Err(error) => error,
            Ok(_) => panic!("the login should have ended"),
        };
        assert!(matches!(
            error,
            ConnectionError::Disconnected(reason)
                if *reason == photon_game::text::Component::literal("You are banned")
        ));
    }

    #[test]
    fn the_player_giving_up_on_the_configuration_ends_the_connection_aborted() {
        let mut session = reconfiguring_against(|stream| {
            std::thread::sleep(Duration::from_secs(2));
            drop(stream);
        });
        session.abort_switch();
        assert_eq!(session.switching(), None);
        assert_eq!(
            session.disconnected(),
            Some(&photon_game::text::Component::translatable(
                "connect.aborted"
            ))
        );
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
        let incoming = spawn_reader(
            stream.try_clone().expect("clone"),
            None,
            crate::connection::play::version_tests::registries(),
        );
        let first = ConnectionState {
            dimension_type_registry_bounds: vec![
                Some(photon_world::DimensionBounds::new(-64, 384)),
                Some(photon_world::DimensionBounds::new(0, 256)),
            ],
            ..ConnectionState::default()
        };
        let mut session = PlaySession::assemble(stream, None, incoming, first, ("Tester", false));
        session.wait(Duration::from_secs(5)).expect("started");
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
        let incoming = spawn_reader(
            stream.try_clone().expect("clone"),
            None,
            crate::connection::play::version_tests::registries(),
        );
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
        let incoming = spawn_reader(
            stream.try_clone().expect("clone"),
            None,
            crate::connection::play::version_tests::registries(),
        );
        let mut session = PlaySession::assemble(
            stream,
            None,
            incoming,
            ConnectionState::default(),
            ("Tester", false),
        );
        // The destination hangs up before it logs the player in, so the transfer fails there.
        session.wait(Duration::from_secs(5)).expect("started");
        assert_eq!(session.switching(), Some(SwitchKind::Transferring));
        session.wait(Duration::from_secs(5)).expect("ended");
        assert_eq!(session.switching(), None);
        assert!(session.disconnected().is_some());
        assert!(session.transfer_failed());
        destination.join().expect("destination");
        origin.join().expect("origin");
    }
}
