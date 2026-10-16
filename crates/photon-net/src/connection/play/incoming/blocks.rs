use super::*;

/// The packets of this family, by packet id.
pub(super) const DECODERS: &[(i32, Decoder)] = &[
    (
        PLAY_CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID,
        typed::<ForgetLevelChunk>,
    ),
    (
        PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID,
        typed::<BlockUpdate>,
    ),
    (
        PLAY_CLIENTBOUND_SECTION_BLOCKS_UPDATE_PACKET_ID,
        typed::<SectionBlocksUpdate>,
    ),
    (
        PLAY_CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID,
        typed::<SetChunkCacheRadius>,
    ),
    (
        PLAY_CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID,
        typed::<SetChunkCacheCenter>,
    ),
    (
        PLAY_CLIENTBOUND_CHUNK_BATCH_FINISHED_PACKET_ID,
        typed::<ChunkBatchFinished>,
    ),
    (
        PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID,
        typed::<BlockChangedAck>,
    ),
    (
        PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID,
        typed::<BlockDestruction>,
    ),
    (
        PLAY_CLIENTBOUND_BLOCK_ENTITY_DATA_PACKET_ID,
        typed::<BlockEntityData>,
    ),
    (
        PLAY_CLIENTBOUND_BLOCK_EVENT_PACKET_ID,
        typed::<BlockEventPacket>,
    ),
];

impl ClientboundPacket for ForgetLevelChunk {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let (x, z) = self.0;
        session.world.unload_chunk(x, z);
        session.effects.forgotten_chunks.push((x, z));
        session.mark_columns_around(x, z);
        Ok(())
    }
}

impl ClientboundPacket for BlockUpdate {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let update = self.0;
        session.mark_block(update.x, update.y, update.z);
        session
            .world
            .apply_server_block_state(update.x, update.y, update.z, update.state_id);
        Ok(())
    }
}

impl ClientboundPacket for SectionBlocksUpdate {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        for ([x, y, z], state) in self.blocks {
            session.mark_block(x, y, z);
            session.world.apply_server_block_state(x, y, z, state);
        }
        Ok(())
    }
}

impl ClientboundPacket for SetChunkCacheRadius {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.world.update_chunk_cache_radius(self.0);
        // `Options.setServerRenderDistance`.
        session.level_info.view_distance = self.0;
        Ok(())
    }
}

impl ClientboundPacket for SetChunkCacheCenter {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let (x, z) = self.0;
        session.world.update_chunk_cache_center(x, z);
        Ok(())
    }
}

impl ClientboundPacket for ChunkBatchFinished {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.send(PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID, |output| {
            // The chunk batch calculator's starting estimate.
            output.extend_from_slice(&3.5_f32.to_be_bytes());
            Ok(())
        })?;
        Ok(())
    }
}

impl ClientboundPacket for BlockChangedAck {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        for synced in session.world.handle_block_changed_ack(self.0) {
            session.mark_block(synced.x, synced.y, synced.z);
            // `syncBlockState`: a block put back into the player
            // moves it back to where it stood when it predicted.
            let pos = photon_core::binary::BlockPos::new(synced.x, synced.y, synced.z);
            if let Some(position) = synced.player_position
                && let Some(player) = session.local.player.as_mut()
                && player.is_colliding(&session.world, pos, synced.state)
            {
                player.abs_snap_to(position);
            }
        }
        Ok(())
    }
}

impl ClientboundPacket for BlockDestruction {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let packet = self.0;
        session.world.destroy_block_progress(
            packet.entity_id,
            packet.x,
            packet.y,
            packet.z,
            i32::from(packet.progress),
        );
        Ok(())
    }
}

impl ClientboundPacket for BlockEntityData {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let entity = self.0;
        session.world.apply_block_entity_update(BlockEntityInput {
            x: entity.x,
            y: entity.y,
            z: entity.z,
            type_id: entity.type_id,
            tag: entity.tag.as_deref(),
        });
        Ok(())
    }
}

impl ClientboundPacket for BlockEventPacket {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let (pos, event, data) = self.0;
        session.world.apply_block_event(pos, event, data);
        Ok(())
    }
}
