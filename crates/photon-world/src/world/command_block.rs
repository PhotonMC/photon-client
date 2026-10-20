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

/// `AbstractMinecart`'s data ids, then `MinecartCommandBlock`'s: the command and the last output.
const MINECART_COMMAND_DATA_ID: u8 = 13;
const MINECART_LAST_OUTPUT_DATA_ID: u8 = 14;
/// The serializers of a string and of a component.
const STRING_SERIALIZER: i32 = 4;
const COMPONENT_SERIALIZER: i32 = 5;

/// A command block minecart as the client has it (`MinecartCommandBlock.getCommandBlock`).
#[derive(Debug, Clone, PartialEq)]
pub struct CommandMinecartView<'a> {
    /// `BaseCommandBlock.getCommand`, from the synched data.
    pub command: String,
    /// `getLastOutput` as the network tag the data carries, from its first byte on.
    pub last_output: Option<&'a [u8]>,
    /// `BaseCommandBlock.isTrackOutput` of the client's copy.
    pub track_output: bool,
}

impl ClientWorld {
    /// The command block minecart with an id, or `None` when no such entity is there.
    pub fn command_minecart_view(&self, entity_id: EntityId) -> Option<CommandMinecartView<'_>> {
        let entity = self.entities.at(entity_id)?;
        if photon_data::entity_type::name(entity.type_id)
            != Some("minecraft:command_block_minecart")
        {
            return None;
        }
        let metadata = entity.metadata().unwrap_or_default();
        let command = match find_metadata_field(metadata, MINECART_COMMAND_DATA_ID) {
            Some((STRING_SERIALIZER, mut input)) => {
                let length =
                    usize::try_from(photon_core::wire::read_varint(&mut input).ok()?).ok()?;
                String::from_utf8_lossy(input.get(..length)?).into_owned()
            }
            _ => String::new(),
        };
        let last_output = match find_metadata_field(metadata, MINECART_LAST_OUTPUT_DATA_ID) {
            Some((COMPONENT_SERIALIZER, input)) => Some(input),
            _ => None,
        };
        Some(CommandMinecartView {
            command,
            last_output,
            track_output: self
                .minecart_track_output
                .get(&entity_id)
                .copied()
                .unwrap_or(true),
        })
    }

    /// `BaseCommandBlock.setTrackOutput` of a command block minecart, which the entity keeps until
    /// the client drops it.
    pub fn set_command_minecart_track_output(&mut self, entity_id: EntityId, track_output: bool) {
        if self.command_minecart_view(entity_id).is_some() {
            self.minecart_track_output.insert(entity_id, track_output);
        }
    }
}
