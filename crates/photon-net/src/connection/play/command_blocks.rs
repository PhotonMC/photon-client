use super::*;
use photon_world::CommandBlockMode;

/// What the command block's screen sends back (`ServerboundSetCommandBlockPacket`).
pub struct CommandBlockSettings<'a> {
    pub command: &'a str,
    pub mode: CommandBlockMode,
    pub track_output: bool,
    pub conditional: bool,
    pub automatic: bool,
}

impl PlaySession {
    /// `ServerboundSetCommandBlockPacket`: the position, the command, the mode by its ordinal and
    /// the flags track output (1), conditional (2) and automatic (4).
    pub fn set_command_block(
        &mut self,
        pos: (i32, i32, i32),
        settings: &CommandBlockSettings<'_>,
    ) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_SET_COMMAND_BLOCK_PACKET_ID, |output| {
            output.extend_from_slice(&pack_block_position(pos).to_be_bytes());
            wire::write_varint(settings.command.len() as i32, output);
            output.extend_from_slice(settings.command.as_bytes());
            wire::write_varint(settings.mode as i32, output);
            output.push(
                u8::from(settings.track_output)
                    | u8::from(settings.conditional) << 1
                    | u8::from(settings.automatic) << 2,
            );
            Ok(())
        })
    }
}

impl PlaySession {
    /// The command block the player used, whose screen is to open (`Player.openCommandBlock`).
    pub fn take_opened_command_block(&mut self) -> Option<(i32, i32, i32)> {
        self.local.game_mode_state.take_opened_command_block()
    }
}
