use super::*;

/// The scoreboard packets, by id.
pub(crate) fn parse_scoreboard_packet(
    id: i32,
    input: &[u8],
) -> Result<Option<photon_game::scoreboard::ScoreboardEvent>, ConnectionError> {
    use photon_game::scoreboard::{ScoreboardEvent, TeamParameters};
    let mut input = input;
    let input = &mut input;
    let string = |input: &mut &[u8], name| read_chat_string(input, 32767, name);
    Ok(Some(match id {
        PLAY_CLIENTBOUND_SET_OBJECTIVE_PACKET_ID => {
            let name = string(input, "objective name")?;
            let method = read_byte(input, "objective method")?;
            if method == 0 || method == 2 {
                ScoreboardEvent::Objective {
                    name,
                    method,
                    display_name: read_chat_component(input, "objective display name")?,
                    hearts: read_varint(input, "objective render type")? == 1,
                    number_format: read_number_format(input)?,
                }
            } else {
                ScoreboardEvent::Objective {
                    name,
                    method,
                    display_name: photon_game::text::Component::empty(),
                    hearts: false,
                    number_format: None,
                }
            }
        }
        PLAY_CLIENTBOUND_SET_SCORE_PACKET_ID => ScoreboardEvent::Score {
            owner: string(input, "score owner")?,
            objective: string(input, "score objective")?,
            value: read_varint(input, "score value")?,
            display: if read_bool(input, "score display present")? {
                Some(read_chat_component(input, "score display")?)
            } else {
                None
            },
            number_format: read_number_format(input)?,
        },
        PLAY_CLIENTBOUND_RESET_SCORE_PACKET_ID => ScoreboardEvent::ResetScore {
            owner: string(input, "reset score owner")?,
            objective: if read_bool(input, "reset score objective present")? {
                Some(string(input, "reset score objective")?)
            } else {
                None
            },
        },
        PLAY_CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID => {
            let slot = read_varint(input, "display slot")?;
            let name = string(input, "display objective")?;
            ScoreboardEvent::Display {
                slot: u8::try_from(slot).unwrap_or(u8::MAX),
                objective: (!name.is_empty()).then_some(name),
            }
        }
        PLAY_CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID => {
            let name = string(input, "team name")?;
            let method = read_byte(input, "team method")?;
            let parameters = if method == 0 || method == 2 {
                let display_name = read_chat_component(input, "team display name")?;
                let prefix = read_chat_component(input, "team prefix")?;
                let suffix = read_chat_component(input, "team suffix")?;
                read_varint(input, "team name tag visibility")?;
                read_varint(input, "team collision rule")?;
                let colour = if read_bool(input, "team colour present")? {
                    Some(read_varint(input, "team colour")?.clamp(0, 15) as u8)
                } else {
                    None
                };
                read_byte(input, "team options")?;
                Some(TeamParameters {
                    display_name,
                    prefix,
                    suffix,
                    colour,
                })
            } else {
                None
            };
            let mut players = Vec::new();
            if matches!(method, 0 | 3 | 4) {
                for _ in 0..read_count(input, "team player count")? {
                    players.push(string(input, "team player")?);
                }
            }
            ScoreboardEvent::Team {
                name,
                method,
                parameters,
                players,
            }
        }
        _ => return Ok(None),
    }))
}

/// `NumberFormatTypes.OPTIONAL_STREAM_CODEC`: absent, or a type (blank,
/// styled, fixed) and what it holds.
pub(super) fn read_number_format(
    input: &mut &[u8],
) -> Result<Option<photon_game::scoreboard::NumberFormat>, ConnectionError> {
    use photon_game::scoreboard::NumberFormat;
    if !read_bool(input, "number format present")? {
        return Ok(None);
    }
    Ok(Some(match read_varint(input, "number format type")? {
        0 => NumberFormat::Blank,
        1 => NumberFormat::Styled(
            photon_core::nbt::read_any_nbt(input)
                .ok()
                .flatten()
                .and_then(|tag| photon_game::text::Style::from_nbt(&tag))
                .ok_or(ConnectionError::Codec("number format style"))?,
        ),
        2 => NumberFormat::Fixed(read_chat_component(input, "number format value")?),
        _ => return Err(ConnectionError::Codec("number format type")),
    }))
}
