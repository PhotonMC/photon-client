//! The server taking the connection back to the configuration phase
//! (`ClientPacketListener.handleConfigurationStart`): the world is let go, the configuration is
//! run again and a new play phase starts.

use std::time::Duration;

use super::lifecycle::spawn_reader;
use super::*;
use crate::connection::probe::configure_connection;

/// How long the configuration may take, the same wait the first one has.
const RECONFIGURATION_TIMEOUT: Duration = Duration::from_secs(30);

impl PlaySession {
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
}
