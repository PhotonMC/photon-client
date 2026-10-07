use super::*;

impl PlaySession {
    /// `handleMovePlayer`: where the server puts the player, which it accepts and answers with
    /// where the player now is.
    pub(super) fn handle_player_position(
        &mut self,
        packet: PlayerPositionPacket,
    ) -> Result<(), ConnectionError> {
        self.place_player(&packet);
        self.send(PLAY_SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID, |output| {
            wire::write_varint(packet.teleport_id, output);
            Ok(())
        })?;
        self.send_player_pos_rot()
    }

    /// `handleTeleportEntity` for a vehicle that was removed under its rider: the rider is moved
    /// in its place and tells the server where it is.
    pub(super) fn move_former_rider(
        &mut self,
        packet: &EntityTeleportObservation,
    ) -> Result<(), ConnectionError> {
        self.place_player(&PlayerPositionPacket {
            teleport_id: 0,
            x: packet.x,
            y: packet.y,
            z: packet.z,
            delta_x: packet.delta_x,
            delta_y: packet.delta_y,
            delta_z: packet.delta_z,
            y_rot: packet.y_rot,
            x_rot: packet.x_rot,
            relative_flags: packet.relative_flags,
        });
        self.send_player_pos_rot()
    }

    /// The `PosRot` the player sends after a teleport: where it is now, in the air, free of walls.
    pub(super) fn send_player_pos_rot(&mut self) -> Result<(), ConnectionError> {
        let Some(player) = self.player.as_ref() else {
            return Ok(());
        };
        let position = player.position();
        let (y_rot, x_rot) = player.rotation();
        let absolute = (position.x, position.y, position.z, y_rot, x_rot);
        self.send(PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID, |output| {
            encode_move_player_pos_rot(output, absolute, 0)
        })
    }

    /// The level counts as loaded once the player has a position and the
    /// chunk under it has arrived; then the client says so and starts ticking.
    pub(super) fn maybe_finish_loading(&mut self) -> Result<(), ConnectionError> {
        if self.loaded {
            return Ok(());
        }
        let Some(player) = self.player.as_ref() else {
            return Ok(());
        };
        let position = player.position();
        let chunk_x = voxel_core::math::floor_f64(position.x).div_euclid(16);
        let chunk_z = voxel_core::math::floor_f64(position.z).div_euclid(16);
        if !self.trace.world.contains_chunk(chunk_x, chunk_z) {
            return Ok(());
        }
        self.send(PLAY_SERVERBOUND_PLAYER_LOADED_PACKET_ID, |_| Ok(()))?;
        self.loaded = true;
        Ok(())
    }
}
