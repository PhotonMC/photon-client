use super::*;

/// `ClientboundPlayerInfoUpdatePacket`: for each player (by its id), the
/// actions the packet's set names, in order. Chat sessions and hat flags are
/// read and dropped.
pub(crate) fn parse_player_info_update(
    input: &[u8],
) -> Result<Vec<PlayerActions>, ConnectionError> {
    let mut input = input;
    let actions = read_byte(&mut input, "player info actions")?;
    let count = read_varint(&mut input, "player info count")?;
    let mut entries = Vec::new();
    for _ in 0..count {
        let uuid = voxel_core::binary::read_uuid(&mut input)
            .map_err(|_| ConnectionError::Codec("player info uuid"))?;
        let mut list = Vec::new();
        for bit in 0..8 {
            if actions & (1 << bit) == 0 {
                continue;
            }
            match bit {
                0 => {
                    let name = read_chat_string(&mut input, 16, "player info name")?;
                    let properties = read_varint(&mut input, "player info properties")?;
                    for _ in 0..properties {
                        read_chat_string(&mut input, 64, "property name")?;
                        read_chat_string(&mut input, 32767, "property value")?;
                        if read_bool(&mut input, "property signature flag")? {
                            read_chat_string(&mut input, 1024, "property signature")?;
                        }
                    }
                    list.push(InfoAction::Add { name });
                }
                1 => {
                    if read_bool(&mut input, "player info chat flag")? {
                        skip_bytes(&mut input, 16 + 8, "chat session")?;
                        for _ in 0..2 {
                            let length = read_varint(&mut input, "chat key length")?;
                            skip_bytes(&mut input, length.max(0) as usize, "chat key")?;
                        }
                    }
                }
                2 => list.push(InfoAction::GameMode(read_varint(&mut input, "game mode")?)),
                3 => list.push(InfoAction::Listed(read_bool(&mut input, "listed")?)),
                4 => list.push(InfoAction::Latency(read_varint(&mut input, "latency")?)),
                5 => {
                    let name = if read_bool(&mut input, "display name flag")? {
                        Some(read_chat_component(&mut input, "display name")?)
                    } else {
                        None
                    };
                    list.push(InfoAction::DisplayName(name));
                }
                6 => list.push(InfoAction::ListOrder(read_varint(
                    &mut input,
                    "list order",
                )?)),
                _ => {
                    read_bool(&mut input, "hat")?;
                }
            }
        }
        entries.push(((uuid.most, uuid.least), list));
    }
    Ok(entries)
}

/// `ClientboundPlayerInfoRemovePacket`.
pub(crate) fn parse_player_info_remove(input: &[u8]) -> Result<Vec<(u64, u64)>, ConnectionError> {
    let mut input = input;
    let count = read_varint(&mut input, "player info remove count")?;
    let mut ids = Vec::new();
    for _ in 0..count {
        let uuid = voxel_core::binary::read_uuid(&mut input)
            .map_err(|_| ConnectionError::Codec("player info remove uuid"))?;
        ids.push((uuid.most, uuid.least));
    }
    Ok(ids)
}

/// A `PlayerInfo` of the tab list: what the client keeps of a player.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerInfo {
    pub name: String,
    /// 0 survival, 1 creative, 2 adventure, 3 spectator.
    pub game_mode: i32,
    pub latency: i32,
    pub listed: bool,
    pub display_name: Option<voxel_game::text::Component>,
    pub list_order: i32,
}

/// One `ClientboundPlayerInfoUpdatePacket` action of a player.
#[derive(Debug, Clone, PartialEq)]
pub enum InfoAction {
    Add { name: String },
    GameMode(i32),
    Listed(bool),
    Latency(i32),
    DisplayName(Option<voxel_game::text::Component>),
    ListOrder(i32),
}

/// A player's id and what a player info packet does to it.
pub type PlayerActions = ((u64, u64), Vec<InfoAction>);
