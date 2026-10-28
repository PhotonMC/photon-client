//! What a player does while the server has it in a bed.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::ClientPlayerGameMode;
    use photon_core::binary::BlockPos;
    use photon_core::math::Vec3;

    fn player() -> LocalPlayer {
        LocalPlayer::new(
            photon_core::ids::EntityId(1),
            Vec3::new(10.0, 64.0, 10.0),
            0.0,
            0.0,
            ClientPlayerGameMode::Survival,
            true,
        )
    }

    /// `LivingEntity.setPosToBed` and `Player.getDesiredPose`: told of a bed, the player lies over
    /// its middle with the low eyes of the sleeping pose, and standing up restores them.
    #[test]
    fn a_player_told_of_a_bed_lies_in_it_with_low_eyes() {
        let world = ClientWorld::default();
        let mut own = player();
        own.set_sleeping_pos(Some(BlockPos { x: 3, y: 70, z: -4 }));
        let at = own.position();
        assert_eq!((at.x, at.y, at.z), (3.5, 70.6875, -3.5));
        own.tick(
            &world,
            Input::new((false, false, false, false), false, false, false),
        )
        .expect("tick");
        assert_eq!(own.pose(), ClientEntityPose::Sleeping);
        assert_eq!(own.eye_height(), 0.2);

        own.set_sleeping_pos(None);
        own.tick(
            &world,
            Input::new((false, false, false, false), false, false, false),
        )
        .expect("tick");
        assert_eq!(own.pose(), ClientEntityPose::Standing);
    }
}
