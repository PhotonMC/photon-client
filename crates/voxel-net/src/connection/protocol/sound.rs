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

/// The title, action bar and tab list packets, by id.
pub(crate) fn parse_hud_packet(id: i32, input: &[u8]) -> Result<Option<HudEvent>, ConnectionError> {
    let mut input = input;
    let input = &mut input;
    Ok(Some(match id {
        PLAY_CLIENTBOUND_SET_TITLE_TEXT_PACKET_ID => {
            HudEvent::Title(read_chat_component(input, "title")?)
        }
        PLAY_CLIENTBOUND_SET_SUBTITLE_TEXT_PACKET_ID => {
            HudEvent::Subtitle(read_chat_component(input, "subtitle")?)
        }
        PLAY_CLIENTBOUND_SET_ACTION_BAR_TEXT_PACKET_ID => {
            HudEvent::ActionBar(read_chat_component(input, "action bar")?)
        }
        PLAY_CLIENTBOUND_SET_TITLES_ANIMATION_PACKET_ID => HudEvent::Times {
            fade_in: read_i32(input, "title fade in")?,
            stay: read_i32(input, "title stay")?,
            fade_out: read_i32(input, "title fade out")?,
        },
        PLAY_CLIENTBOUND_CLEAR_TITLES_PACKET_ID => HudEvent::Clear {
            reset_times: read_bool(input, "clear titles reset")?,
        },
        PLAY_CLIENTBOUND_TAB_LIST_PACKET_ID => HudEvent::TabList {
            header: read_chat_component(input, "tab list header")?,
            footer: read_chat_component(input, "tab list footer")?,
        },
        PLAY_CLIENTBOUND_BOSS_EVENT_PACKET_ID => parse_boss_event(input)?,
        _ => return Ok(None),
    }))
}

/// `ClientboundCommandsPacket`: the entries, linked into a tree.
pub fn parse_commands_packet(
    input: &[u8],
) -> Result<voxel_game::commands::CommandTree, ConnectionError> {
    use voxel_game::commands::{CommandTree, NodeKind, NodeStub};
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

#[expect(clippy::too_many_lines, reason = "one arm for each argument type id")]
/// An argument type by registry id with its parameters
/// (`ArgumentTypeInfo.deserializeFromNetwork`).
pub(super) fn read_argument_kind(
    input: &mut &[u8],
    id: i32,
) -> Result<voxel_game::commands::ArgumentKind, ConnectionError> {
    use voxel_game::commands::{ArgumentKind as K, StringType};
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

/// `ClientboundDisconnectPacket`: the reason the server gives.
pub(crate) fn read_disconnect_reason(
    input: &mut &[u8],
) -> Result<voxel_game::text::Component, ConnectionError> {
    read_chat_component(input, "disconnect reason")
}

/// `ClientboundPlayerCombatKillPacket`: who died and the message for them.
pub(crate) fn parse_player_combat_kill_packet(
    input: &[u8],
) -> Result<(i32, voxel_game::text::Component), ConnectionError> {
    let mut input = input;
    let id = read_varint(&mut input, "combat kill player id")?;
    Ok((id, read_chat_component(&mut input, "combat kill message")?))
}

/// `ClientboundLevelEventPacket`, read from the front of `input`.
pub fn read_level_event_packet(input: &mut &[u8]) -> Result<LevelEvent, ConnectionError> {
    let event = read_i32(input, "level event")?;
    let pos = voxel_core::binary::BlockPos::unpack(read_i64(input, "level event position")? as u64);
    let data = read_i32(input, "level event data")?;
    let global = read_bool(input, "level event global")?;
    Ok(LevelEvent {
        event,
        pos: (pos.x, pos.y, pos.z),
        data,
        global,
    })
}

/// `SoundEvent.STREAM_CODEC`: a registry place plus one, or zero and the event whole.
pub(super) fn read_sound_holder(
    input: &mut &[u8],
) -> Result<voxel_game::sounds::SoundHolder, ConnectionError> {
    use voxel_game::sounds::SoundHolder;
    let holder = read_varint(input, "sound holder")?;
    if holder == 0 {
        let location = read_identifier_string(input, "sound location")?;
        let fixed_range = if read_bool(input, "sound range present")? {
            Some(read_f32(input, "sound range")?)
        } else {
            None
        };
        return Ok(SoundHolder::Direct {
            location,
            fixed_range,
        });
    }
    let id = holder - 1;
    if voxel_data::sound_event::name(id).is_none() {
        return Err(ConnectionError::Codec("unknown sound event id"));
    }
    Ok(SoundHolder::Registered(id))
}

/// `FriendlyByteBuf.readEnum(SoundSource.class)`.
pub(super) fn read_sound_source(
    input: &mut &[u8],
) -> Result<voxel_game::sounds::SoundSource, ConnectionError> {
    let ordinal = read_varint(input, "sound source")?;
    voxel_game::sounds::SoundSource::from_ordinal(ordinal)
        .ok_or(ConnectionError::Codec("unknown sound source"))
}

/// `ClientboundSoundPacket`, read from the front of `input` (what follows is the caller's).
pub fn read_sound_packet(
    input: &mut &[u8],
) -> Result<voxel_game::sounds::SoundPacket, ConnectionError> {
    let sound = read_sound_holder(input)?;
    let source = read_sound_source(input)?;
    // The packet carries eighths of a block as ints; the reference divides as a float.
    let place = |input: &mut &[u8], name| -> Result<f64, ConnectionError> {
        Ok(f64::from(read_i32(input, name)? as f32 / 8.0))
    };
    let position = (
        place(input, "sound x")?,
        place(input, "sound y")?,
        place(input, "sound z")?,
    );
    Ok(voxel_game::sounds::SoundPacket {
        sound,
        source,
        position,
        volume: read_f32(input, "sound volume")?,
        pitch: read_f32(input, "sound pitch")?,
        seed: read_i64(input, "sound seed")?,
    })
}

/// `ClientboundSoundEntityPacket`, read from the front of `input`.
pub fn read_sound_entity_packet(
    input: &mut &[u8],
) -> Result<voxel_game::sounds::SoundEntityPacket, ConnectionError> {
    Ok(voxel_game::sounds::SoundEntityPacket {
        sound: read_sound_holder(input)?,
        source: read_sound_source(input)?,
        entity_id: read_varint(input, "sound entity id")?,
        volume: read_f32(input, "sound volume")?,
        pitch: read_f32(input, "sound pitch")?,
        seed: read_i64(input, "sound seed")?,
    })
}

/// `ClientboundStopSoundPacket`, read from the front of `input`.
pub fn read_stop_sound_packet(
    input: &mut &[u8],
) -> Result<voxel_game::sounds::StopSoundPacket, ConnectionError> {
    let flags = read_byte(input, "stop sound flags")?;
    let source = if flags & 1 > 0 {
        Some(read_sound_source(input)?)
    } else {
        None
    };
    let name = if flags & 2 > 0 {
        Some(read_identifier_string(input, "stop sound name")?)
    } else {
        None
    };
    Ok(voxel_game::sounds::StopSoundPacket { name, source })
}
