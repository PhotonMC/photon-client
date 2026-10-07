use super::*;

impl PlaySession {
    /// `ClientPacketListener.handleMovePlayer`.
    /// `setValuesFromPositionPacket` for the local player: a rider stays where its vehicle puts it.
    pub(in super::super) fn place_player(&mut self, packet: &PlayerPositionPacket) {
        let passenger = self.trace.world.local_player_is_passenger();
        let current = self.player.as_ref().map(|player| {
            let position = player.position();
            let (y_rot, x_rot) = player.rotation();
            (position.x, position.y, position.z, y_rot, x_rot)
        });
        let absolute = resolve_player_position(*packet, current);
        let position = Vec3::new(absolute.0, absolute.1, absolute.2);
        let known_health = self.health_received.then_some(self.status.health);
        let player = self.player.get_or_insert_with(|| {
            let mut player = LocalPlayer::new(
                self.player_id.unwrap_or(0),
                position,
                absolute.3,
                absolute.4,
                self.game_mode,
                self.auto_jump,
            );
            // A health update that came before the player did is its first.
            if let Some(health) = known_health {
                player.hurt_to(health);
            }
            player
        });
        if let Some(abilities) = self.abilities {
            player.set_abilities(abilities);
        }
        let current_delta = player.delta_movement();
        let delta = Vec3::new(
            packet.delta_x
                + if packet.relative_flags & (1 << 5) != 0 {
                    current_delta.x
                } else {
                    0.0
                },
            packet.delta_y
                + if packet.relative_flags & (1 << 6) != 0 {
                    current_delta.y
                } else {
                    0.0
                },
            packet.delta_z
                + if packet.relative_flags & (1 << 7) != 0 {
                    current_delta.z
                } else {
                    0.0
                },
        );
        if !passenger {
            player.apply_teleport(position, delta, absolute.3, absolute.4);
        }
    }
}
