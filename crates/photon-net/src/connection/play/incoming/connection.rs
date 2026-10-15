use super::*;

/// The packets of this family, by packet id.
pub(super) const DECODERS: &[(i32, Decoder)] = &[
    (
        PLAY_CLIENTBOUND_SET_SIMULATION_DISTANCE_PACKET_ID,
        typed::<SetSimulationDistance>,
    ),
    (
        PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID,
        typed::<photon_protocol::common::ClientboundKeepAlivePacket>,
    ),
    (
        PLAY_CLIENTBOUND_PING_PACKET_ID,
        typed::<photon_protocol::common::ClientboundPingPacket>,
    ),
    (PLAY_CLIENTBOUND_UPDATE_TAGS_PACKET_ID, typed::<UpdateTags>),
    (PLAY_CLIENTBOUND_DISCONNECT_PACKET_ID, typed::<Disconnect>),
];

/// `handleSetChunkCacheRadius`'s sibling: the simulation distance the server runs with.
impl ClientboundPacket for SetSimulationDistance {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.trace.world.set_server_simulation_distance(self.0);
        Ok(())
    }
}

/// `handleKeepAlive`: answered at once with the same number.
impl ClientboundPacket for photon_protocol::common::ClientboundKeepAlivePacket {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.send(PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID, |output| {
            photon_protocol::common::ServerboundKeepAlivePacket::new(self.id).encode(output);
            Ok(())
        })?;
        Ok(())
    }
}

/// `handlePing`: answered at once with the same number.
impl ClientboundPacket for photon_protocol::common::ClientboundPingPacket {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.send(PLAY_SERVERBOUND_PONG_PACKET_ID, |output| {
            photon_protocol::common::ServerboundPongPacket::new(self.id).encode(output);
            Ok(())
        })?;
        Ok(())
    }
}

/// `handleUpdateTags`.
impl ClientboundPacket for UpdateTags {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.trace.world.apply_update_tags_packet(&self.0);
        Ok(())
    }
}

/// `handleDisconnect`.
impl ClientboundPacket for Disconnect {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.disconnect = Some(self.0);
        Ok(())
    }
}
