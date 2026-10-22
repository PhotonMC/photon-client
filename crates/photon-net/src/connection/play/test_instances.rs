use super::*;
use photon_game::test_instance::{InstanceRequest, TestInstanceContents};
use photon_protocol::play::TestInstanceBlockStatus;

impl PlaySession {
    /// `ServerboundTestInstanceBlockActionPacket`: the position, the action and the block's data
    /// as the screen holds it (the status is always cleared, and no error goes along).
    pub fn test_instance_action(
        &mut self,
        pos: (i32, i32, i32),
        request: &InstanceRequest,
    ) -> Result<(), ConnectionError> {
        self.send(
            PLAY_SERVERBOUND_TEST_INSTANCE_BLOCK_ACTION_PACKET_ID,
            |output| {
                output.extend_from_slice(&pack_block_position(pos).to_be_bytes());
                wire::write_varint(request.action as i32, output);
                output.push(u8::from(request.test.is_some()));
                if let Some(test) = &request.test {
                    wire::write_varint(test.len() as i32, output);
                    output.extend_from_slice(test.as_bytes());
                }
                for axis in [request.size.0, request.size.1, request.size.2] {
                    wire::write_varint(axis, output);
                }
                let rotation = photon_game::structure_block::Rotation::ALL
                    .iter()
                    .position(|rotation| *rotation == request.rotation)
                    .unwrap_or(0);
                wire::write_varint(rotation as i32, output);
                output.push(u8::from(request.ignore_entities));
                wire::write_varint(0, output);
                output.push(0);
                Ok(())
            },
        )
    }

    /// The test instance block the player used, whose screen is to open.
    pub fn take_opened_test_instance_block(&mut self) -> Option<(i32, i32, i32)> {
        self.local.game_mode_state.take_opened_test_instance_block()
    }

    /// The test instance block at a position with the contents and error its block entity holds,
    /// or `None` when the block is not there.
    pub fn test_instance_block(
        &self,
        pos: (i32, i32, i32),
    ) -> Option<(TestInstanceContents, Option<photon_game::text::Component>)> {
        let tag = self.world.test_instance_block_view(pos)?;
        Some((
            TestInstanceContents::from_tag(tag),
            TestInstanceContents::error_from_tag(tag),
        ))
    }

    /// The status lines that came in since the last call.
    pub fn take_test_instance_statuses(&mut self) -> Vec<TestInstanceBlockStatus> {
        std::mem::take(&mut self.menus.test_instance_statuses)
    }
}
