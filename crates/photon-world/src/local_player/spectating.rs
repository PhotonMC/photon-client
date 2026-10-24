//! What a player does while another entity is the camera (`isControlledCamera` is false).

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ClientPlayerGameMode;
    use photon_core::math::Vec3;

    fn player(controlled: bool) -> LocalPlayer {
        let mut player = LocalPlayer::new(
            photon_core::ids::EntityId(1),
            Vec3::new(0.0, 100.0, 0.0),
            0.0,
            0.0,
            ClientPlayerGameMode::Survival,
            true,
        );
        player.set_controlled_camera(controlled);
        player
    }

    fn forward() -> Input {
        Input::new(true, false, false, false, false, false, false)
    }

    fn moves(sent: &[ServerboundMovement]) -> usize {
        sent.iter()
            .filter(|packet| {
                matches!(
                    packet,
                    ServerboundMovement::MovePlayerPos { .. }
                        | ServerboundMovement::MovePlayerPosRot { .. }
                        | ServerboundMovement::MovePlayerRot { .. }
                        | ServerboundMovement::MovePlayerStatusOnly { .. }
                )
            })
            .count()
    }

    /// The pressed keys still reach the server, which ends a spectating on the sneak key, but no
    /// position does, and the input no longer drives the body.
    #[test]
    fn a_spectating_player_sends_its_keys_but_no_position_and_takes_no_input() {
        let world = ClientWorld::default();
        let mut own = player(true);
        let sent = own.tick(&world, forward()).expect("tick");
        assert!(moves(&sent) > 0 && own.zza > 0.9);

        let mut watching = player(false);
        watching.zza = 1.0;
        let sent = watching.tick(&world, forward()).expect("tick");
        assert_eq!(moves(&sent), 0);
        assert!(
            sent.iter()
                .any(|packet| matches!(packet, ServerboundMovement::PlayerInput(_)))
        );
        assert!((watching.zza - 0.98).abs() < 1e-6, "{}", watching.zza);
    }
}
