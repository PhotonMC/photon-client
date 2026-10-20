use super::*;
use photon_core::ids::EntityId;
use photon_game::command_block::CommandBlockContents;
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
    /// `ServerboundSetCommandMinecartPacket`: the entity id, the command and whether the output is
    /// tracked.
    pub fn set_command_minecart(
        &mut self,
        entity_id: EntityId,
        command: &str,
        track_output: bool,
    ) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_SET_COMMAND_MINECART_PACKET_ID, |output| {
            wire::write_varint(entity_id.0, output);
            wire::write_varint(command.len() as i32, output);
            output.extend_from_slice(command.as_bytes());
            output.push(u8::from(track_output));
            Ok(())
        })
    }

    /// The command block minecart the player used, whose screen is to open
    /// (`Player.openMinecartCommandBlock`).
    pub fn take_opened_command_minecart(&mut self) -> Option<EntityId> {
        self.local.game_mode_state.take_opened_command_minecart()
    }

    /// `BaseCommandBlock.setTrackOutput` of a minecart: what its screen last left.
    pub fn set_minecart_track_output(&mut self, entity_id: EntityId, track_output: bool) {
        self.world
            .set_command_minecart_track_output(entity_id, track_output);
    }

    /// The command block minecart with an id, as its synched data and the client's copy of its
    /// output tracking have it, or `None` once it is gone.
    pub fn command_minecart(&self, entity_id: EntityId) -> Option<CommandBlockContents> {
        let view = self.world.command_minecart_view(entity_id)?;
        Some(CommandBlockContents::from_minecart(
            view.command,
            view.last_output,
            view.track_output,
        ))
    }
}

impl PlaySession {
    /// The command block the player used, whose screen is to open (`Player.openCommandBlock`).
    pub fn take_opened_command_block(&mut self) -> Option<(i32, i32, i32)> {
        self.local.game_mode_state.take_opened_command_block()
    }
}

/// What a command block's screen shows of the block, as the client has it.
pub struct CommandBlockState {
    pub mode: CommandBlockMode,
    pub conditional: bool,
    pub contents: photon_game::command_block::CommandBlockContents,
}

impl PlaySession {
    /// The command block at a position, with the contents its block entity holds
    /// (`BaseCommandBlock`), or `None` when the block is no command block.
    pub fn command_block(&self, pos: (i32, i32, i32)) -> Option<CommandBlockState> {
        let view = self.world.command_block_view(pos)?;
        Some(CommandBlockState {
            mode: view.mode,
            conditional: view.conditional,
            contents: photon_game::command_block::CommandBlockContents::from_tag(view.tag),
        })
    }

    /// Whether a command block's data arrived since the last call.
    pub fn take_command_block_data(&mut self) -> bool {
        std::mem::take(&mut self.menus.command_block_data)
    }
}
