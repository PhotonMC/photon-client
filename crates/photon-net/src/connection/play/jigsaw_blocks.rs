use super::*;
use photon_game::jigsaw::{JigsawContents, Joint};

/// What the jigsaw block's screen sends back, but for the block's position
/// (`ServerboundSetJigsawBlockPacket`).
pub struct JigsawBlockSettings<'a> {
    pub name: &'a str,
    pub target: &'a str,
    pub pool: &'a str,
    pub final_state: &'a str,
    pub joint: Joint,
    pub selection_priority: i32,
    pub placement_priority: i32,
}

fn write_text(text: &str, output: &mut Vec<u8>) {
    wire::write_varint(text.len() as i32, output);
    output.extend_from_slice(text.as_bytes());
}

impl PlaySession {
    /// `ServerboundSetJigsawBlockPacket`: the position, the three identifiers and the final state
    /// as text, the joint's name, and the two priorities as varints.
    pub fn set_jigsaw_block(
        &mut self,
        pos: (i32, i32, i32),
        settings: &JigsawBlockSettings<'_>,
    ) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_SET_JIGSAW_BLOCK_PACKET_ID, |output| {
            output.extend_from_slice(&pack_block_position(pos).to_be_bytes());
            for text in [
                settings.name,
                settings.target,
                settings.pool,
                settings.final_state,
                settings.joint.name(),
            ] {
                write_text(text, output);
            }
            wire::write_varint(settings.selection_priority, output);
            wire::write_varint(settings.placement_priority, output);
            Ok(())
        })
    }

    /// `ServerboundJigsawGeneratePacket`: the position, how many levels to generate, and whether
    /// to keep the jigsaws.
    pub fn generate_jigsaw(
        &mut self,
        pos: (i32, i32, i32),
        levels: i32,
        keep_jigsaws: bool,
    ) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_JIGSAW_GENERATE_PACKET_ID, |output| {
            output.extend_from_slice(&pack_block_position(pos).to_be_bytes());
            wire::write_varint(levels, output);
            output.push(u8::from(keep_jigsaws));
            Ok(())
        })
    }

    /// The jigsaw block the player used, whose screen is to open (`Player.openJigsawBlock`).
    pub fn take_opened_jigsaw_block(&mut self) -> Option<(i32, i32, i32)> {
        self.local.game_mode_state.take_opened_jigsaw_block()
    }

    /// The jigsaw block at a position with the contents its block entity holds, or `None` when the
    /// block or its entity is not there.
    pub fn jigsaw_block(&self, pos: (i32, i32, i32)) -> Option<JigsawContents> {
        let view = self.world.jigsaw_block_view(pos)?;
        Some(JigsawContents::from_tag(view.tag, view.vertical))
    }
}
