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

/// `UUID.fromString`: five groups of hex, or the short forms it accepts.
fn uuid_valid(text: &str) -> bool {
    let parts: Vec<&str> = text.split('-').collect();
    parts.len() == 5
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.len() <= 16)
}

mod argument_kind_items;

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
