//! Reading one argument of each kind (`ArgumentType.parse`).

use super::reader::{CommandSyntaxException, StringReader, java_double, java_float};
use super::{ArgumentKind, StringType};
use crate::text::Component;

type Read = Result<(), CommandSyntaxException>;

fn literal(text: impl Into<String>) -> Component {
    Component::literal(text)
}

/// `WorldCoordinate.isRelative`: a leading `~`.
fn relative(reader: &mut StringReader) -> bool {
    if reader.peek() == '~' {
        reader.skip();
        true
    } else {
        false
    }
}

/// `WorldCoordinate.parseDouble` (the centring changes only the value).
fn coordinate_double(reader: &mut StringReader) -> Read {
    if reader.can_read() && reader.peek() == '^' {
        return Err(CommandSyntaxException::at(
            reader,
            "argument.pos.mixed",
            &[],
        ));
    }
    if !reader.can_read() {
        return Err(CommandSyntaxException::at(
            reader,
            "argument.pos.missing.double",
            &[],
        ));
    }
    let is_relative = relative(reader);
    if reader.can_read() && reader.peek() != ' ' {
        reader.read_double()?;
    }
    let _ = is_relative;
    Ok(())
}

/// `WorldCoordinate.parseInt`: `~` takes a double, an absolute value an int.
fn coordinate_int(reader: &mut StringReader) -> Read {
    if reader.can_read() && reader.peek() == '^' {
        return Err(CommandSyntaxException::at(
            reader,
            "argument.pos.mixed",
            &[],
        ));
    }
    if !reader.can_read() {
        return Err(CommandSyntaxException::at(
            reader,
            "argument.pos.missing.int",
            &[],
        ));
    }
    let is_relative = relative(reader);
    if reader.can_read() && reader.peek() != ' ' {
        if is_relative {
            reader.read_double()?;
        } else {
            reader.read_int()?;
        }
    }
    Ok(())
}

/// Three coordinates apart by spaces, or the error naming what is missing.
fn coordinates(
    reader: &mut StringReader,
    count: usize,
    incomplete: &str,
    one: fn(&mut StringReader) -> Read,
) -> Read {
    let start = reader.cursor();
    for index in 0..count {
        if index > 0 {
            if reader.can_read() && reader.peek() == ' ' {
                reader.skip();
            } else {
                reader.set_cursor(start);
                return Err(CommandSyntaxException::at(reader, incomplete, &[]));
            }
        }
        one(reader)?;
    }
    Ok(())
}

/// `LocalCoordinates.parse`.
fn local_coordinates(reader: &mut StringReader) -> Read {
    let start = reader.cursor();
    for index in 0..3 {
        if index > 0 {
            if reader.can_read() && reader.peek() == ' ' {
                reader.skip();
            } else {
                reader.set_cursor(start);
                return Err(CommandSyntaxException::at(
                    reader,
                    "argument.pos3d.incomplete",
                    &[],
                ));
            }
        }
        if !reader.can_read() {
            return Err(CommandSyntaxException::at(
                reader,
                "argument.pos.missing.double",
                &[],
            ));
        }
        if reader.peek() != '^' {
            reader.set_cursor(start);
            return Err(CommandSyntaxException::at(
                reader,
                "argument.pos.mixed",
                &[],
            ));
        }
        reader.skip();
        if reader.can_read() && reader.peek() != ' ' {
            reader.read_double()?;
        }
    }
    Ok(())
}

/// `ParserUtils.readWhile(reader, c != ' ')`.
fn word(reader: &mut StringReader) {
    while reader.can_read() && reader.peek() != ' ' {
        reader.skip();
    }
}

/// A stretch up to the next space outside quotes and brackets.
fn balanced(reader: &mut StringReader) {
    let mut depth = 0_i32;
    let mut quote: Option<char> = None;
    while reader.can_read() {
        let c = reader.peek();
        match quote {
            Some(q) => {
                if c == '\\' && reader.can_read_n(2) {
                    reader.skip();
                } else if c == q {
                    quote = None;
                }
            }
            None => match c {
                ' ' if depth <= 0 => return,
                '"' | '\'' => quote = Some(c),
                '[' | '{' | '(' => depth += 1,
                ']' | '}' | ')' => depth -= 1,
                _ => {}
            },
        }
        reader.skip();
    }
}

/// `EntityArgument.parse`: a selector (`@p`, `@e[...]`), a name of up to
/// sixteen characters, or a UUID. Options are read by shape, so a `limit` or
/// `type` they carry does not change what the selector may return.
fn entity(reader: &mut StringReader, single: bool, players_only: bool, selectors: bool) -> Read {
    let start = reader.cursor();
    let (many, entities, own) = if reader.can_read() && reader.peek() == '@' {
        if !selectors {
            return Err(CommandSyntaxException::at(
                reader,
                "argument.entity.selector.not_allowed",
                &[],
            ));
        }
        reader.skip();
        if !reader.can_read() {
            return Err(CommandSyntaxException::at(
                reader,
                "argument.entity.selector.missing",
                &[],
            ));
        }
        let selector = reader.cursor();
        let kind = reader.read();
        let (many, entities, own) = match kind {
            'a' => (true, false, false),
            'e' => (true, true, false),
            'n' => (false, true, false),
            'p' | 'r' => (false, false, false),
            's' => (false, true, true),
            _ => {
                reader.set_cursor(selector);
                return Err(CommandSyntaxException::at(
                    reader,
                    "argument.entity.selector.unknown",
                    &[literal(format!("@{kind}"))],
                ));
            }
        };
        let mut options = false;
        if reader.can_read() && reader.peek() == '[' {
            options = true;
            let from = reader.cursor();
            balanced(reader);
            if reader.cursor() == from || !reader.slice(from, reader.cursor()).ends_with(']') {
                return Err(CommandSyntaxException::at(
                    reader,
                    "argument.entity.options.unterminated",
                    &[],
                ));
            }
        }
        if options {
            return Ok(());
        }
        (many, entities, own)
    } else {
        let name = reader.read_string()?;
        if !uuid_valid(&name) && (name.is_empty() || name.chars().count() > 16) {
            reader.set_cursor(start);
            return Err(CommandSyntaxException::at(
                reader,
                "argument.entity.invalid",
                &[],
            ));
        }
        (false, uuid_valid(&name), false)
    };
    if many && single {
        reader.set_cursor(0);
        let key = if players_only {
            "argument.player.toomany"
        } else {
            "argument.entity.toomany"
        };
        return Err(CommandSyntaxException::at(reader, key, &[]));
    }
    if entities && players_only && !own {
        reader.set_cursor(0);
        return Err(CommandSyntaxException::at(
            reader,
            "argument.player.entities",
            &[],
        ));
    }
    Ok(())
}

fn identifier_valid(text: &str) -> bool {
    let (namespace, path) = match text.split_once(':') {
        Some(("", path)) => ("minecraft", path),
        Some((namespace, path)) => (namespace, path),
        None => ("minecraft", text),
    };
    namespace
        .chars()
        .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase() || matches!(c, '_' | '.' | '-'))
        && path.chars().all(|c| {
            c.is_ascii_digit() || c.is_ascii_lowercase() || matches!(c, '_' | '.' | '-' | '/')
        })
}

/// `Identifier.read`.
fn identifier(reader: &mut StringReader) -> Read {
    let start = reader.cursor();
    while reader.can_read()
        && (reader.peek().is_ascii_digit()
            || reader.peek().is_ascii_lowercase()
            || matches!(reader.peek(), '_' | ':' | '/' | '.' | '-'))
    {
        reader.skip();
    }
    let raw = reader.slice(start, reader.cursor());
    if identifier_valid(&raw) {
        Ok(())
    } else {
        reader.set_cursor(start);
        Err(CommandSyntaxException::at(
            reader,
            "argument.id.invalid",
            &[],
        ))
    }
}

/// An unquoted word that must be one of a set.
fn one_of(reader: &mut StringReader, values: &[&str], key: &str, rewind: bool) -> Read {
    let start = reader.cursor();
    let name = reader.read_unquoted();
    if values.contains(&name.as_str()) {
        return Ok(());
    }
    if rewind {
        reader.set_cursor(start);
    }
    Err(CommandSyntaxException::at(reader, key, &[literal(name)]))
}

fn range_error<T: std::fmt::Display>(
    reader: &StringReader,
    kind: &str,
    side: &str,
    bound: T,
    found: String,
) -> CommandSyntaxException {
    CommandSyntaxException::at(
        reader,
        &format!("argument.{kind}.{side}"),
        &[literal(bound.to_string()), literal(found)],
    )
}

impl ArgumentKind {
    /// `ArgumentType.parse`: read the argument off the reader, for a source
    /// that may use entity selectors.
    pub fn read(&self, reader: &mut StringReader) -> Read {
        self.read_as(reader, true)
    }

    /// `ArgumentType.parse(reader, source)`: the source's permission to use
    /// entity selectors is the one thing a kind reads from it.
    #[expect(clippy::too_many_lines, reason = "one arm for each kind of argument")]
    pub fn read_as(&self, reader: &mut StringReader, selectors: bool) -> Read {
        match self {
            Self::Bool => reader.read_boolean().map(drop),
            Self::Integer { min, max } => {
                let start = reader.cursor();
                let value = reader.read_int()?;
                if value < *min {
                    reader.set_cursor(start);
                    return Err(range_error(
                        reader,
                        "integer",
                        "low",
                        min,
                        value.to_string(),
                    ));
                }
                if value > *max {
                    reader.set_cursor(start);
                    return Err(range_error(
                        reader,
                        "integer",
                        "big",
                        max,
                        value.to_string(),
                    ));
                }
                Ok(())
            }
            Self::Long { min, max } => {
                let start = reader.cursor();
                let value = reader.read_long()?;
                if value < *min {
                    reader.set_cursor(start);
                    return Err(range_error(reader, "long", "low", min, value.to_string()));
                }
                if value > *max {
                    reader.set_cursor(start);
                    return Err(range_error(reader, "long", "big", max, value.to_string()));
                }
                Ok(())
            }
            Self::Float { min, max } => {
                let start = reader.cursor();
                let value = reader.read_float()?;
                if value < *min {
                    reader.set_cursor(start);
                    return Err(range_error(
                        reader,
                        "float",
                        "low",
                        java_float(*min),
                        java_float(value),
                    ));
                }
                if value > *max {
                    reader.set_cursor(start);
                    return Err(range_error(
                        reader,
                        "float",
                        "big",
                        java_float(*max),
                        java_float(value),
                    ));
                }
                Ok(())
            }
            Self::Double { min, max } => {
                let start = reader.cursor();
                let value = reader.read_double()?;
                if value < *min {
                    reader.set_cursor(start);
                    return Err(range_error(
                        reader,
                        "double",
                        "low",
                        java_double(*min),
                        java_double(value),
                    ));
                }
                if value > *max {
                    reader.set_cursor(start);
                    return Err(range_error(
                        reader,
                        "double",
                        "big",
                        java_double(*max),
                        java_double(value),
                    ));
                }
                Ok(())
            }
            Self::String(StringType::Word) | Self::Unquoted => {
                reader.read_unquoted();
                Ok(())
            }
            Self::String(StringType::Phrase) => reader.read_string().map(drop),
            Self::String(StringType::Greedy) | Self::Greedy => {
                reader.set_cursor(reader.total_length());
                Ok(())
            }
            Self::BlockPos => {
                if reader.can_read() && reader.peek() == '^' {
                    local_coordinates(reader)
                } else {
                    coordinates(reader, 3, "argument.pos3d.incomplete", coordinate_int)
                }
            }
            Self::Vec3 => {
                if reader.can_read() && reader.peek() == '^' {
                    local_coordinates(reader)
                } else {
                    coordinates(reader, 3, "argument.pos3d.incomplete", coordinate_double)
                }
            }
            Self::ColumnPos => {
                if !reader.can_read() {
                    return Err(CommandSyntaxException::at(
                        reader,
                        "argument.pos2d.incomplete",
                        &[],
                    ));
                }
                coordinates(reader, 2, "argument.pos2d.incomplete", coordinate_int)
            }
            Self::Vec2 | Self::Rotation => {
                if !reader.can_read() {
                    return Err(CommandSyntaxException::at(
                        reader,
                        "argument.pos2d.incomplete",
                        &[],
                    ));
                }
                coordinates(reader, 2, "argument.pos2d.incomplete", coordinate_double)
            }
            Self::Angle => {
                if !reader.can_read() {
                    return Err(CommandSyntaxException::at(
                        reader,
                        "argument.angle.incomplete",
                        &[],
                    ));
                }
                relative(reader);
                if reader.can_read() && reader.peek() != ' ' {
                    let value = reader.read_float()?;
                    if !value.is_finite() {
                        return Err(CommandSyntaxException::at(
                            reader,
                            "argument.angle.invalid",
                            &[],
                        ));
                    }
                }
                Ok(())
            }
            Self::Identifier => identifier(reader),
            Self::GameMode => one_of(
                reader,
                &["survival", "creative", "adventure", "spectator"],
                "argument.gamemode.invalid",
                false,
            ),
            Self::EntityAnchor => {
                one_of(reader, &["feet", "eyes"], "argument.anchor.invalid", true)
            }
            Self::ScoreboardSlot => {
                // The slot names hold dots, which `readUnquotedString` takes.
                one_of(
                    reader,
                    &crate::scoreboard::DISPLAY_SLOT_NAMES,
                    "argument.scoreboardDisplaySlot.invalid",
                    false,
                )
            }
            Self::TeamColor => one_of(
                reader,
                &crate::scoreboard::TEAM_COLOURS,
                "argument.color.invalid",
                false,
            ),
            Self::HexColor => {
                let text = reader.read_unquoted();
                if matches!(text.chars().count(), 3 | 6) {
                    Ok(())
                } else {
                    Err(CommandSyntaxException::at(
                        reader,
                        "argument.hexcolor.invalid",
                        &[literal(text)],
                    ))
                }
            }
            Self::Operation => {
                if !reader.can_read() {
                    return Err(CommandSyntaxException::at(
                        reader,
                        "arguments.operation.invalid",
                        &[],
                    ));
                }
                let start = reader.cursor();
                word(reader);
                let text = reader.slice(start, reader.cursor());
                if ["=", "+=", "-=", "*=", "/=", "%=", "<", ">", "><"].contains(&text.as_str()) {
                    Ok(())
                } else {
                    Err(CommandSyntaxException::plain(
                        "arguments.operation.invalid",
                        &[],
                    ))
                }
            }
            Self::Swizzle => {
                let mut seen = String::new();
                while reader.can_read() && reader.peek() != ' ' {
                    let c = reader.read();
                    if !matches!(c, 'x' | 'y' | 'z') || seen.contains(c) {
                        return Err(CommandSyntaxException::at(
                            reader,
                            "arguments.swizzle.invalid",
                            &[],
                        ));
                    }
                    seen.push(c);
                }
                Ok(())
            }
            Self::Uuid => {
                let rest = reader.remaining();
                let end = rest
                    .chars()
                    .take_while(|c| c.is_ascii_hexdigit() || *c == '-')
                    .count();
                let candidate: String = rest.chars().take(end).collect();
                if uuid_valid(&candidate) {
                    reader.set_cursor(reader.cursor() + end);
                    Ok(())
                } else {
                    Err(CommandSyntaxException::at(
                        reader,
                        "argument.uuid.invalid",
                        &[],
                    ))
                }
            }
            Self::Time { min } => {
                let value = reader.read_float()?;
                let unit = reader.read_unquoted();
                let factor = match unit.as_str() {
                    "d" => 24000,
                    "s" => 20,
                    "t" | "" => 1,
                    _ => {
                        return Err(CommandSyntaxException::at(
                            reader,
                            "argument.time.invalid_unit",
                            &[],
                        ));
                    }
                };
                let ticks = (value * factor as f32 + 0.5).floor() as i32;
                if ticks < *min {
                    Err(CommandSyntaxException::at(
                        reader,
                        "argument.time.tick_count_too_low",
                        &[literal(min.to_string()), literal(ticks.to_string())],
                    ))
                } else {
                    Ok(())
                }
            }
            Self::Entity {
                single,
                players_only,
            } => entity(reader, *single, *players_only, selectors),
            Self::Word => {
                word(reader);
                Ok(())
            }
            Self::Balanced => {
                balanced(reader);
                Ok(())
            }
            Self::ScoreHolder => {
                if !selectors && reader.can_read() && reader.peek() == '@' {
                    return Err(CommandSyntaxException::at(
                        reader,
                        "argument.entity.selector.not_allowed",
                        &[],
                    ));
                }
                balanced(reader);
                Ok(())
            }
        }
    }
}

/// `UUID.fromString`: five groups of hex, or the short forms it accepts.
fn uuid_valid(text: &str) -> bool {
    let parts: Vec<&str> = text.split('-').collect();
    parts.len() == 5
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.len() <= 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(kind: &ArgumentKind, text: &str) -> (Result<(), CommandSyntaxException>, usize) {
        let mut reader = StringReader::new(text);
        let result = kind.read(&mut reader);
        (result, reader.cursor())
    }

    #[test]
    fn a_block_position_takes_three_values() {
        assert_eq!(read(&ArgumentKind::BlockPos, "1 ~ ~-2 tail").1, 7);
        assert!(read(&ArgumentKind::BlockPos, "1 2").0.is_err());
        assert_eq!(read(&ArgumentKind::BlockPos, "^ ^1 ^ x").1, 6);
        assert!(read(&ArgumentKind::BlockPos, "^ 1 ^").0.is_err());
    }

    #[test]
    fn integers_keep_to_their_bounds() {
        let kind = ArgumentKind::Integer { min: 0, max: 10 };
        assert!(read(&kind, "5").0.is_ok());
        let (result, cursor) = read(&kind, "11");
        assert_eq!(cursor, 0);
        assert!(result.unwrap_err().message != Component::empty());
    }

    #[test]
    fn selectors_read_to_the_space_outside_brackets() {
        assert_eq!(
            read(&ArgumentKind::Balanced, "@e[type=pig, distance=..5] x").1,
            26
        );
        assert_eq!(read(&ArgumentKind::Balanced, "{a:\"b c\"} x").1, 9);
    }
}
