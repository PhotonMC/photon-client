use super::*;

impl PlaySession {
    pub(super) fn on_set_simulation_distance(
        &mut self,
        payload: &[u8],
    ) -> Result<(), ConnectionError> {
        if let Some(chunks) = parsed(parse_entity_id_prefix(payload), "entity_id_prefix") {
            self.trace.world.set_server_simulation_distance(chunks);
        }
        Ok(())
    }

    pub(super) fn on_keep_alive(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let mut input = payload;
        let packet = crate::common::ClientboundKeepAlivePacket::decode(&mut input)
            .map_err(|_| ConnectionError::Codec("play keep alive"))?;
        self.send(PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID, |output| {
            crate::common::ServerboundKeepAlivePacket::new(packet.id).encode(output);
            Ok(())
        })?;
        Ok(())
    }

    pub(super) fn on_ping(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let mut input = payload;
        let packet = crate::common::ClientboundPingPacket::decode(&mut input)
            .map_err(|_| ConnectionError::Codec("play ping"))?;
        self.send(PLAY_SERVERBOUND_PONG_PACKET_ID, |output| {
            crate::common::ServerboundPongPacket::new(packet.id).encode(output);
            Ok(())
        })?;
        Ok(())
    }

    pub(super) fn on_update_tags(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let mut input = payload;
        let packet = voxel_core::tags::ClientboundUpdateTagsPacket::decode(&mut input)
            .map_err(|_| ConnectionError::Codec("update tags"))?;
        self.trace.world.apply_update_tags_packet(&packet);
        Ok(())
    }

    pub(super) fn on_disconnect(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let mut input = payload;
        self.disconnect = Some(read_disconnect_reason(&mut input)?);
        Ok(())
    }
}
