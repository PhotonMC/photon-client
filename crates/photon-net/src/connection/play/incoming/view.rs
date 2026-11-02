use super::*;
use photon_core::math::Vec3;

/// The packets of this family, by packet id.
pub(super) const DECODERS: &[(i32, Decoder)] = &[
    (
        PLAY_CLIENTBOUND_PLAYER_ROTATION_PACKET_ID,
        typed::<PlayerRotationPacket>,
    ),
    (
        PLAY_CLIENTBOUND_PLAYER_LOOK_AT_PACKET_ID,
        typed::<PlayerLookAtPacket>,
    ),
];

/// `handleRotatePlayer`: the server turns the player, who tells it the view it now has.
impl ClientboundPacket for PlayerRotationPacket {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let Some(player) = session.local.player.as_mut() else {
            return Ok(());
        };
        let (y_rot, x_rot) = resolve_player_rotation(*self, player.rotation());
        player.set_view(y_rot, x_rot);
        let (y_rot, x_rot) = player.rotation();
        session.send_movement(ServerboundMovement::MovePlayerRot {
            y_rot,
            x_rot,
            on_ground: false,
            horizontal_collision: false,
        })
    }
}

/// `handleLookAt`: the player turns to face a point, or an entity when the level holds it.
impl ClientboundPacket for PlayerLookAtPacket {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let eyes = |anchor| anchor == EntityAnchor::Eyes;
        let entity = self.entity.and_then(|(id, anchor)| {
            if session.local.player_id == Some(id) {
                let player = session.local.player.as_ref()?;
                let feet = player.position();
                let height = if eyes(anchor) {
                    f64::from(player.eye_height())
                } else {
                    0.0
                };
                Some(Vec3::new(feet.x, feet.y + height, feet.z))
            } else {
                session.world.entity_anchor(id, eyes(anchor))
            }
        });
        let target = entity.unwrap_or(Vec3::new(self.at.0, self.at.1, self.at.2));
        if let Some(player) = session.local.player.as_mut() {
            player.look_at(eyes(self.from), target);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::play::profile_tests::session;
    use crate::connection::transport::Transport;
    use photon_core::ids::EntityId;
    use photon_core::math::Vec3;
    use photon_world::ClientPlayerGameMode;
    use photon_world::local_player::LocalPlayer;

    fn standing(session: &mut PlaySession, y_rot: f32, x_rot: f32) {
        session.local.player = Some(LocalPlayer::new(
            EntityId(1),
            Vec3::new(0.0, 64.0, 0.0),
            y_rot,
            x_rot,
            ClientPlayerGameMode::Survival,
            false,
        ));
    }

    /// The next packet the client sent: its id and body.
    fn sent(server: &std::net::TcpListener) -> (i32, Vec<u8>) {
        let (stream, _) = server.accept().expect("accept");
        let mut stream = Transport::from(stream);
        let packet = crate::connection::read_packet(&mut stream, None).expect("packet");
        let (id, body) = split_packet(&packet).expect("split");
        (id, body.to_vec())
    }

    #[test]
    fn a_rotation_packet_turns_the_view_and_is_answered_with_it() {
        let (mut session, server) = session();
        standing(&mut session, 350.0, 80.0);
        let packet = PlayerRotationPacket {
            y_rot: 30.0,
            relative_y: true,
            x_rot: 30.0,
            relative_x: true,
        };
        Box::new(packet).handle(&mut session).expect("handled");
        let player = session.local.player.as_ref().expect("player");
        assert_eq!(player.rotation(), (380.0, 90.0));
        let (id, body) = sent(&server);
        assert_eq!(id, PLAY_SERVERBOUND_MOVE_PLAYER_ROT_PACKET_ID);
        let mut expected = 380.0_f32.to_be_bytes().to_vec();
        expected.extend_from_slice(&90.0_f32.to_be_bytes());
        // Not on the ground and no collision: the reference sends the answer with both false.
        expected.push(0);
        assert_eq!(body, expected);
    }

    #[test]
    fn a_look_at_turns_the_player_to_a_point_and_to_an_entity_it_knows() {
        let (mut session, _server) = session();
        standing(&mut session, 0.0, 0.0);
        let at_point = PlayerLookAtPacket {
            from: EntityAnchor::Feet,
            at: (10.0, 64.0, 0.0),
            entity: None,
        };
        Box::new(at_point).handle(&mut session).expect("handled");
        // East is a quarter turn to the left of south.
        let (y_rot, x_rot) = session.local.player.as_ref().unwrap().rotation();
        assert!((y_rot + 90.0).abs() < 1e-3 && x_rot.abs() < 1e-3);
        // An entity the level does not hold is looked at where the packet says it is.
        let unknown = PlayerLookAtPacket {
            from: EntityAnchor::Feet,
            at: (0.0, 64.0, 10.0),
            entity: Some((EntityId(99), EntityAnchor::Eyes)),
        };
        Box::new(unknown).handle(&mut session).expect("handled");
        // (The reference's `Mth.atan2` is a table, so the angle is near, not exactly, zero.)
        let (y_rot, x_rot) = session.local.player.as_ref().unwrap().rotation();
        assert!(y_rot.abs() < 1e-3 && x_rot.abs() < 1e-3);
    }
}
