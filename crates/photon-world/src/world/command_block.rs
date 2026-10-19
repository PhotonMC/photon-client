//! What a command block's screen reads of the block: its mode and condition from the block state,
//! and its command from the block entity's tag.

use crate::*;

/// A command block as the client has it.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandBlockView<'a> {
    /// `CommandBlockEntity.getMode`, by the block: `Redstone` for the plain one.
    pub mode: CommandBlockMode,
    /// `CommandBlockEntity.isConditional`: the block state's `conditional`.
    pub conditional: bool,
    /// The block entity's tag, which carries the command.
    pub tag: Option<&'a [u8]>,
}

/// `CommandBlockEntity.Mode`, in the order the wire counts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBlockMode {
    Sequence,
    Auto,
    Redstone,
}

impl ClientWorld {
    /// The command block at a position, with its block entity's data when the client has any (Java
    /// makes the entity with the block; here it is kept once the server sends it).
    pub fn command_block_view(&self, pos: (i32, i32, i32)) -> Option<CommandBlockView<'_>> {
        let state = self.block_state_at(pos.0, pos.1, pos.2)?;
        let mode = match crate::block::state_or_air(state).block().name {
            "minecraft:command_block" => CommandBlockMode::Redstone,
            "minecraft:repeating_command_block" => CommandBlockMode::Auto,
            "minecraft:chain_command_block" => CommandBlockMode::Sequence,
            _ => return None,
        };
        let tag = self
            .block_entity_at(pos.0, pos.1, pos.2)
            .filter(|entity| {
                photon_data::block_entity_type::name(entity.type_id)
                    == Some("minecraft:command_block")
            })
            .and_then(|entity| entity.tag.as_deref());
        Some(CommandBlockView {
            mode,
            conditional: crate::local_player::property(Some(state), "conditional") == Some("true"),
            tag,
        })
    }
}
