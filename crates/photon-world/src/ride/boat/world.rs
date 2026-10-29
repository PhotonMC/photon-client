use super::*;
use photon_core::ids::EntityId;

/// Whether a paddle at this turn makes its stroke sound this tick (`AbstractBoat.tick`'s test of
/// the paddle's position against the start of its sweep).
pub(super) fn paddle_makes_sound(position: f32) -> bool {
    let turn = (std::f64::consts::PI * 2.0) as f32;
    let quarter = (std::f64::consts::PI / 4.0) as f32;
    let step = (std::f64::consts::PI / 8.0) as f32;
    position % turn <= quarter && (position + step) % turn >= quarter
}

impl ClientWorld {
    /// `AbstractBoat.tick` for the boat the local player rides, between this tick's start and its
    /// movement. Its paddles are queued to be told to the server.
    pub(crate) fn tick_ridden_boat(&mut self, entity_id: EntityId) {
        let Some(entity) = self.entities.at(entity_id) else {
            return;
        };
        let Some(pick) = photon_data::entity_type::pick(entity.type_id) else {
            return;
        };
        let (width, height, _) = pick.size;
        let dimensions =
            ClientEntityDimensions::scalable(width, height).scale(entity.scale(), entity.scale());
        let (x, y, z) = entity.position();
        let (ox, oy, oz) = entity.previous_position();
        let mut body = BoatBody::new(
            Vec3::new(x, y, z),
            Vec3::new(ox, oy, oz),
            dimensions,
            entity.boat_ride,
        );
        let (mx, my, mz) = entity.motion();
        body.delta = Vec3::new(mx, my, mz);
        body.y_rot = entity.rotation.y_rot.get();
        body.delta_rotation = entity.delta_rotation.get();
        body.on_ground = entity.on_ground;
        body.supporting_block = entity.tracking.support.main_block;
        body.on_ground_no_blocks = entity.tracking.support.no_blocks;
        body.no_gravity = entity.no_gravity;
        body.input = entity.boat_input;
        body.has_passengers = !entity.passenger_ids.is_empty();
        body.player_aboard = entity.passenger_ids.first().is_some_and(|&id| {
            self.local_player
                .as_ref()
                .is_some_and(|player| player.id == id)
                || self
                    .entities
                    .get(&id)
                    .is_some_and(|passenger| passenger.is_player())
        });
        body.paddles = (entity.paddle_left, entity.paddle_right);
        let level = Level::new(self);
        let Ok(paddles) = body.tick(&level) else {
            return;
        };
        // The paddles row, and a stroke at the start of its sweep asks which sound the water or
        // the land makes, which finds the status again (and what finding it leaves behind).
        let silent = entity.silent;
        let paddling = [paddles.0, paddles.1].map(|rows| rows && body.has_passengers);
        let positions = [0, 1].map(|side| {
            entity
                .kind
                .boat()
                .map_or(0.0, |boat| boat.paddle_position(side))
        });
        for (rows, position) in paddling.iter().zip(positions) {
            if *rows && !silent && paddle_makes_sound(position) {
                let _ = body.status(&level);
            }
        }
        let mut entity = self
            .entities
            .at_mut(entity_id)
            .expect("the boat was read above");
        entity.set_position((body.position.x, body.position.y, body.position.z));
        entity.vel.motion.x = JavaF64::new(body.delta.x);
        entity.vel.motion.y = JavaF64::new(body.delta.y);
        entity.vel.motion.z = JavaF64::new(body.delta.z);
        entity.rotation.y_rot = JavaF32::new(body.y_rot);
        entity.delta_rotation = JavaF32::new(body.delta_rotation);
        entity.on_ground = body.on_ground;
        entity.tracking.support = Support {
            main_block: body.supporting_block,
            no_blocks: body.on_ground_no_blocks,
        };
        entity.paddle_left = body.paddles.0;
        entity.paddle_right = body.paddles.1;
        entity.boat_ride = body.ride;
        entity.tick_boat_end(paddling);
        if let Some(state) = entity.kind.boat_mut() {
            state.is_under_water = body.ride.status.is_some_and(Status::is_under_water);
        }
        self.paddle_packets.push(paddles);
    }

    /// The paddles the ridden boat's ticks told of since last asked, each to be sent to the server.
    pub fn take_paddle_packets(&mut self) -> Vec<(bool, bool)> {
        std::mem::take(&mut self.paddle_packets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `AbstractBoat.controlBoat` against the numbers the reference gives from the same start
    /// (`docs/records/client-world-entity-boat-control-conformance.md`): input flags, `deltaRotation`, yaw,
    /// the motion's x and z, and the paddles.
    #[test]
    fn control_applies_java_input_delta_and_paddles() {
        let rows = [
            (
                "left",
                BoatInput::new(true, false, false, false),
                0xbf80_0000_u32,
                0x4110_0000_u32,
                0x3fbf_ccc0_8100_0000_u64,
                0xbfcf_5e2d_1900_0000_u64,
                (false, true),
            ),
            (
                "right",
                BoatInput::new(false, true, false, false),
                0x3f80_0000,
                0x4130_0000,
                0x3fbf_c17d_a440_0000,
                0xbfcf_5f2b_3300_0000,
                (true, false),
            ),
            (
                "forward",
                BoatInput::new(false, false, true, false),
                0x0000_0000,
                0x4120_0000,
                0x3fbe_38e6_d800_0000,
                0xbfca_f52e_e000_0000,
                (true, true),
            ),
            (
                "backward",
                BoatInput::new(false, false, false, true),
                0x0000_0000,
                0x4120_0000,
                0x3fc0_1c71_9280_0000,
                0xbfd0_50ad_1200_0000,
                (false, false),
            ),
            (
                "both_turn",
                BoatInput::new(true, true, false, false),
                0x0000_0000,
                0x4120_0000,
                0x3fc0_0000_0000_0000,
                0xbfd0_0000_0000_0000,
                (false, false),
            ),
            (
                "left_forward",
                BoatInput::new(true, false, true, false),
                0xbf80_0000,
                0x4110_0000,
                0x3fbe_6604_0800_0000,
                0xbfca_f168_c800_0000,
                (true, true),
            ),
        ];
        for (name, input, delta, yaw, motion_x, motion_z, paddles) in rows {
            let mut body = BoatBody::new(
                Vec3::new(2.0, 12.0, 2.0),
                Vec3::new(2.0, 12.0, 2.0),
                ClientEntityDimensions::scalable(1.375, 0.5625),
                RideState::default(),
            );
            body.delta = Vec3::new(0.125, 0.0, -0.25);
            body.y_rot = 10.0;
            body.input = input;
            body.has_passengers = true;
            body.control_boat();
            assert_eq!(body.delta_rotation.to_bits(), delta, "{name}");
            assert_eq!(body.y_rot.to_bits(), yaw, "{name}");
            assert_eq!(body.delta.x.to_bits(), motion_x, "{name}");
            assert_eq!(body.delta.z.to_bits(), motion_z, "{name}");
            assert_eq!(body.paddles, paddles, "{name}");
        }
    }
}
