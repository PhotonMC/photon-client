//! The server taking the connection back to the configuration phase
//! (`ClientPacketListener.handleConfigurationStart`): the world is let go, the configuration is
//! run again and a new play phase starts.

use std::time::Duration;

use super::lifecycle::{self, spawn_reader};
use super::*;
use crate::connection::probe::configure_connection;

/// How long the configuration may take, the same wait the first one has.
const RECONFIGURATION_TIMEOUT: Duration = Duration::from_secs(30);

impl PlaySession {
    /// `handleRequestCookie`: what is kept under the key goes back, or nothing.
    pub(super) fn on_cookie_request(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let request = crate::cookie::ClientboundCookieRequestPacket::decode(&mut &payload[..])
            .map_err(|_| ConnectionError::Codec("cookie request"))?;
        let answer = self.trace.cookies.answer(&request);
        self.send(PLAY_SERVERBOUND_COOKIE_RESPONSE_PACKET_ID, |output| {
            answer
                .encode(output)
                .map_err(|_| ConnectionError::Codec("cookie response"))
        })
    }

    /// `handleTransfer`: the connection is let go and another made to the host the server names,
    /// as a transfer, with the cookies kept. The new session starts over like a reconfigured one.
    pub(super) fn transfer(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let packet = crate::cookie::ClientboundTransferPacket::decode(&mut &payload[..])
            .map_err(|_| ConnectionError::Codec("transfer"))?;
        let address = lifecycle::resolve_host(&packet.host, packet.port)?;
        let carried = PlayConnectionTrace {
            cookies: self.trace.cookies.clone(),
            transferring: true,
            ..PlayConnectionTrace::default()
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
        let mut trace = PlayConnectionTrace {
            compression_threshold: self.compression_threshold,
            login_finished: true,
            profile_id: self.trace.profile_id,
            cookies: self.trace.cookies.clone(),
            server_brand: self.trace.server_brand.clone(),
            custom_report_details: self.trace.custom_report_details.clone(),
            ..PlayConnectionTrace::default()
        };
        configure_connection(
            &mut stream,
            self.compression_threshold,
            (RECONFIGURATION_TIMEOUT, true),
            &mut trace,
        )?;
        stream.set_read_timeout(None)?;
        let incoming = spawn_reader(stream, self.compression_threshold);
        let mut fresh = Self::assemble(
            self.writer.try_clone()?,
            self.compression_threshold,
            incoming,
            trace,
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
            PlayConnectionTrace::default(),
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
            PlayConnectionTrace::default(),
            ("Tester", false),
        );
        session.wait(Duration::from_secs(5)).expect("store");
        session.wait(Duration::from_secs(5)).expect("request");
        server.join().expect("server");
        assert!(session.trace.cookies.get(&key).is_some());
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
            let intention =
                crate::handshake::ClientIntentionPacket::decode(&mut body).expect("decode");
            assert_eq!(
                String::from_utf16(&intention.host_name).expect("host"),
                "127.0.0.1"
            );
            assert_eq!(intention.port, i32::from(target_port));
            assert_eq!(
                intention.intention,
                crate::handshake::ClientIntent::Transfer
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
            PlayConnectionTrace::default(),
            ("Tester", false),
        );
        // The destination hangs up before it logs the player in, so the transfer fails there.
        assert!(session.wait(Duration::from_secs(5)).is_err());
        destination.join().expect("destination");
        origin.join().expect("origin");
    }
}
