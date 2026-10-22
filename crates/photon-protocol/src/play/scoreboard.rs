use super::*;

/// The scoreboard packets, by id.
pub fn parse_scoreboard_packet(
    id: i32,
    input: &[u8],
) -> Result<Option<photon_game::scoreboard::ScoreboardEvent>, ConnectionError> {
    use photon_game::scoreboard::ScoreboardEvent;
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
                Some(read_team_parameters(input)?)
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

/// A team's display name, dressing, name tag visibility, colour and options.
fn read_team_parameters(
    input: &mut &[u8],
) -> Result<photon_game::scoreboard::TeamParameters, ConnectionError> {
    use photon_game::scoreboard::{NameTagVisibility, TeamParameters};
    let display_name = read_chat_component(input, "team display name")?;
    let prefix = read_chat_component(input, "team prefix")?;
    let suffix = read_chat_component(input, "team suffix")?;
    let visibility = read_varint(input, "team name tag visibility")?;
    read_varint(input, "team collision rule")?;
    let colour = if read_bool(input, "team colour present")? {
        Some(read_varint(input, "team colour")?.clamp(0, 15) as u8)
    } else {
        None
    };
    let options = read_byte(input, "team options")?;
    Ok(TeamParameters {
        display_name,
        prefix,
        suffix,
        colour,
        name_tag_visibility: NameTagVisibility::from_id(visibility),
        // `PlayerTeam.unpackOptions`: bit 1.
        sees_friendly_invisibles: options & 2 != 0,
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use photon_game::scoreboard::NameTagVisibility;

    /// A team's parameters on the wire: three plain texts, then the rest.
    fn parameters(visibility: u8, options: u8) -> Vec<u8> {
        let mut bytes = Vec::new();
        for _ in 0..3 {
            // A network text component that is a plain string: a string tag.
            bytes.extend([8, 0, 1, b'x']);
        }
        // Visibility, collision rule, no colour, options.
        bytes.extend([visibility, 0, 0, options]);
        bytes
    }

    fn read(visibility: u8, options: u8) -> photon_game::scoreboard::TeamParameters {
        let bytes = parameters(visibility, options);
        read_team_parameters(&mut bytes.as_slice()).expect("parameters")
    }

    #[test]
    fn the_visibility_and_the_friendly_invisibles_option_are_read() {
        assert_eq!(
            read(2, 0).name_tag_visibility,
            NameTagVisibility::HideForOtherTeams
        );
        assert_eq!(
            read(3, 0).name_tag_visibility,
            NameTagVisibility::HideForOwnTeam
        );
        // An id out of range is the first (`OutOfBoundsStrategy.ZERO`).
        assert_eq!(read(7, 0).name_tag_visibility, NameTagVisibility::Always);
        // Option bit 1 (value 2) is the flag; bit 0 (friendly fire) is not.
        assert!(read(0, 2).sees_friendly_invisibles);
        assert!(!read(0, 1).sees_friendly_invisibles);
    }
}
