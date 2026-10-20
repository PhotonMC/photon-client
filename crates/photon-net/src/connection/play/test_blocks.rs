use super::*;
use photon_game::test_block::{TestBlockContents, TestBlockMode};

impl PlaySession {
    /// `ServerboundSetTestBlockPacket`: the position, the mode by ordinal and the message.
    pub fn set_test_block(
        &mut self,
        pos: (i32, i32, i32),
        mode: TestBlockMode,
        message: &str,
    ) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_SET_TEST_BLOCK_PACKET_ID, |output| {
            output.extend_from_slice(&pack_block_position(pos).to_be_bytes());
            wire::write_varint(mode as i32, output);
            wire::write_varint(message.len() as i32, output);
            output.extend_from_slice(message.as_bytes());
            Ok(())
        })
    }

    /// The test block the player used, whose screen is to open (`Player.openTestBlock`).
    pub fn take_opened_test_block(&mut self) -> Option<(i32, i32, i32)> {
        self.local.game_mode_state.take_opened_test_block()
    }

    /// The test block at a position with the contents its block entity holds, or `None` when the
    /// block or its entity is not there.
    pub fn test_block(&self, pos: (i32, i32, i32)) -> Option<TestBlockContents> {
        let view = self.world.test_block_view(pos)?;
        let state_mode = TestBlockMode::named(view.state_mode).unwrap_or(TestBlockMode::Start);
        Some(TestBlockContents::from_tag(view.tag, state_mode))
    }
}
