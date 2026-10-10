use super::*;

pub(crate) fn parse_command_suggestions(
    input: &[u8],
) -> Result<CommandSuggestions, ConnectionError> {
    let mut input = input;
    let id = read_varint(&mut input, "suggestion id")?;
    let start = read_varint(&mut input, "suggestion start")?.max(0) as usize;
    let length = read_varint(&mut input, "suggestion length")?.max(0) as usize;
    let count = read_varint(&mut input, "suggestion count")?;
    let mut matches = Vec::new();
    for _ in 0..count {
        matches.push(read_chat_string(&mut input, 32767, "suggestion text")?);
        // The tooltip is read and dropped.
        if read_bool(&mut input, "suggestion tooltip flag")? {
            read_chat_component(&mut input, "suggestion tooltip")?;
        }
    }
    Ok(CommandSuggestions {
        id,
        start,
        length,
        matches,
    })
}

/// `ClientboundCommandsPacket`: the entries, linked into a tree.
pub fn parse_commands_packet(
    input: &[u8],
) -> Result<photon_game::commands::CommandTree, ConnectionError> {
    use photon_game::commands::{CommandTree, NodeKind, NodeStub};
    let mut input = input;
    let input = &mut input;
    let count = read_count(input, "command node count")?;
    let mut stubs = Vec::with_capacity(count.min(4096));
    for _ in 0..count {
        let flags = read_byte(input, "command node flags")?;
        let children = (0..read_count(input, "command children count")?)
            .map(|_| read_count(input, "command child"))
            .collect::<Result<Vec<_>, _>>()?;
        let redirect = if flags & 8 != 0 {
            Some(read_count(input, "command redirect")?)
        } else {
            None
        };
        let mut custom = false;
        let mut type_id = 0;
        let kind = match flags & 3 {
            1 => NodeKind::Literal(read_chat_string(input, 32767, "command literal")?),
            2 => {
                let name = read_chat_string(input, 32767, "command argument name")?;
                let id = read_varint(input, "command argument type")?;
                let kind = read_argument_kind(input, id)?;
                type_id = u8::try_from(id).unwrap_or(u8::MAX);
                if flags & 16 != 0 {
                    read_chat_string(input, 32767, "command suggestion id")?;
                    custom = true;
                }
                NodeKind::Argument { name, kind }
            }
            _ => NodeKind::Root,
        };
        stubs.push(NodeStub {
            kind,
            children,
            redirect,
            executable: flags & 4 != 0,
            custom_suggestions: custom,
            type_id,
        });
    }
    let root = read_count(input, "command root index")?;
    CommandTree::build(stubs, root).ok_or(ConnectionError::Codec("command tree"))
}

/// An argument type by registry id with its parameters
/// (`ArgumentTypeInfo.deserializeFromNetwork`).
pub(super) fn read_argument_kind(
    input: &mut &[u8],
    id: i32,
) -> Result<photon_game::commands::ArgumentKind, ConnectionError> {
    use photon_game::commands::{ArgumentKind as K, StringType};
    let name = "command argument parameters";
    Ok(match id {
        0 => K::Bool,
        1 => {
            let flags = read_byte(input, name)?;
            let min = if flags & 1 != 0 {
                read_f32(input, name)?
            } else {
                -f32::MAX
            };
            let max = if flags & 2 != 0 {
                read_f32(input, name)?
            } else {
                f32::MAX
            };
            K::Float { min, max }
        }
        2 => {
            let flags = read_byte(input, name)?;
            let min = if flags & 1 != 0 {
                read_f64(input, name)?
            } else {
                -f64::MAX
            };
            let max = if flags & 2 != 0 {
                read_f64(input, name)?
            } else {
                f64::MAX
            };
            K::Double { min, max }
        }
        3 => {
            let flags = read_byte(input, name)?;
            let min = if flags & 1 != 0 {
                read_i32(input, name)?
            } else {
                i32::MIN
            };
            let max = if flags & 2 != 0 {
                read_i32(input, name)?
            } else {
                i32::MAX
            };
            K::Integer { min, max }
        }
        4 => {
            let flags = read_byte(input, name)?;
            let min = if flags & 1 != 0 {
                read_i64(input, name)?
            } else {
                i64::MIN
            };
            let max = if flags & 2 != 0 {
                read_i64(input, name)?
            } else {
                i64::MAX
            };
            K::Long { min, max }
        }
        5 => K::String(match read_varint(input, name)? {
            0 => StringType::Word,
            1 => StringType::Phrase,
            _ => StringType::Greedy,
        }),
        6 => {
            let flags = read_byte(input, name)?;
            K::Entity {
                single: flags & 1 != 0,
                players_only: flags & 2 != 0,
            }
        }
        // Game profiles are players; score holders also take a bare name.
        7 => K::Entity {
            single: false,
            players_only: true,
        },
        31 => {
            read_byte(input, name)?;
            K::ScoreHolder
        }
        12..=15 | 18 | 19 | 21..=23 | 37 | 52..=55 => K::Balanced,
        8 => K::BlockPos,
        9 => K::ColumnPos,
        10 => K::Vec3,
        11 => K::Vec2,
        16 => K::TeamColor,
        17 => K::HexColor,
        20 => K::Greedy,
        24 | 33 => K::Unquoted,
        25 | 27 | 34 | 35 | 39 | 40 | 49..=51 => K::Word,
        26 => K::Operation,
        28 => K::Angle,
        29 => K::Rotation,
        30 => K::ScoreboardSlot,
        32 => K::Swizzle,
        36 | 41 => K::Identifier,
        38 => K::EntityAnchor,
        42 => K::GameMode,
        43 => K::Time {
            min: read_i32(input, name)?,
        },
        // Resources name the registry they come from.
        44..=48 => {
            read_chat_string(input, 32767, name)?;
            K::Word
        }
        56 => K::Uuid,
        _ => return Err(ConnectionError::Codec("command argument type")),
    })
}

/// `ClientboundCommandSuggestionsPacket`: the request it answers, the range
/// of the typed text the matches replace, and the matches.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandSuggestions {
    pub id: i32,
    pub start: usize,
    pub length: usize,
    pub matches: Vec<String>,
}
