//! `StringReader` and `CommandSyntaxException`, Brigadier's way of reading
//! text and of saying where and why it could not.
//!
//! Positions count characters, as the chat box does (the reference counts
//! UTF-16 units; the two agree below U+10000).

use crate::text::Component;

/// A failure to read: what to tell the player, and where in the input.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandSyntaxException {
    pub message: Component,
    /// `literalIncorrect`, which the usage line counts apart from the rest.
    pub literal_incorrect: bool,
    /// The input and cursor `createWithContext` recorded, if it did.
    pub context: Option<(String, i32)>,
}

impl CommandSyntaxException {
    /// An exception that names no place (`create`).
    pub fn plain(key: &str, args: &[Component]) -> Self {
        Self {
            message: Component::translatable_with(key, args.to_vec()),
            literal_incorrect: false,
            context: None,
        }
    }

    /// `createWithContext`.
    pub fn at(reader: &StringReader, key: &str, args: &[Component]) -> Self {
        Self {
            context: Some((reader.text.iter().collect(), reader.cursor as i32)),
            ..Self::plain(key, args)
        }
    }

    pub fn cursor(&self) -> i32 {
        self.context.as_ref().map_or(-1, |(_, cursor)| *cursor)
    }

    /// `getContext`: up to ten characters before the cursor, then the marker.
    pub fn context_text(&self) -> Option<String> {
        let (input, cursor) = self.context.as_ref()?;
        if *cursor < 0 {
            return None;
        }
        let chars: Vec<char> = input.chars().collect();
        let cursor = (*cursor as usize).min(chars.len());
        let mut text = String::new();
        if cursor > 10 {
            text.push_str("...");
        }
        text.extend(&chars[cursor.saturating_sub(10)..cursor]);
        text.push_str("<--[HERE]");
        Some(text)
    }
}

pub use crate::java_float::{java_double, java_float};

#[derive(Debug, Clone)]
pub struct StringReader {
    text: Vec<char>,
    cursor: usize,
}

fn allowed_number(c: char) -> bool {
    c.is_ascii_digit() || c == '.' || c == '-'
}

fn allowed_unquoted(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+')
}

impl StringReader {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.chars().collect(),
            cursor: 0,
        }
    }

    pub fn string(&self) -> String {
        self.text.iter().collect()
    }

    pub fn total_length(&self) -> usize {
        self.text.len()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    pub fn remaining_length(&self) -> usize {
        self.text.len() - self.cursor
    }

    pub fn can_read_n(&self, length: usize) -> bool {
        self.cursor + length <= self.text.len()
    }

    pub fn can_read(&self) -> bool {
        self.can_read_n(1)
    }

    pub fn peek(&self) -> char {
        self.text[self.cursor]
    }

    pub fn skip(&mut self) {
        self.cursor += 1;
    }

    pub fn read(&mut self) -> char {
        let c = self.text[self.cursor];
        self.cursor += 1;
        c
    }

    pub fn slice(&self, start: usize, end: usize) -> String {
        self.text[start..end].iter().collect()
    }

    pub fn remaining(&self) -> String {
        self.slice(self.cursor, self.text.len())
    }

    /// `readUnquotedString`.
    pub fn read_unquoted(&mut self) -> String {
        let start = self.cursor;
        while self.can_read() && allowed_unquoted(self.peek()) {
            self.skip();
        }
        self.slice(start, self.cursor)
    }

    /// `readString`: quoted, or unquoted.
    pub fn read_string(&mut self) -> Result<String, CommandSyntaxException> {
        if !self.can_read() {
            return Ok(String::new());
        }
        let next = self.peek();
        if next == '"' || next == '\'' {
            self.skip();
            return self.read_string_until(next);
        }
        Ok(self.read_unquoted())
    }

    /// `readStringUntil`.
    pub fn read_string_until(
        &mut self,
        terminator: char,
    ) -> Result<String, CommandSyntaxException> {
        let mut result = String::new();
        let mut escaped = false;
        while self.can_read() {
            let c = self.read();
            if escaped {
                if c == terminator || c == '\\' {
                    result.push(c);
                    escaped = false;
                } else {
                    self.cursor -= 1;
                    return Err(CommandSyntaxException::at(
                        self,
                        "parsing.quote.escape",
                        &[Component::literal(c.to_string())],
                    ));
                }
            } else if c == '\\' {
                escaped = true;
            } else if c == terminator {
                return Ok(result);
            } else {
                result.push(c);
            }
        }
        Err(CommandSyntaxException::at(
            self,
            "parsing.quote.expected.end",
            &[],
        ))
    }

    /// The characters a number may be made of, read as text.
    fn read_number_text(&mut self) -> String {
        let start = self.cursor;
        while self.can_read() && allowed_number(self.peek()) {
            self.skip();
        }
        self.slice(start, self.cursor)
    }

    fn read_number<T>(
        &mut self,
        parse: impl Fn(&str) -> Option<T>,
        expected: &str,
        invalid: &str,
    ) -> Result<T, CommandSyntaxException> {
        let start = self.cursor;
        let number = self.read_number_text();
        if number.is_empty() {
            return Err(CommandSyntaxException::at(self, expected, &[]));
        }
        parse(&number).ok_or_else(|| {
            self.cursor = start;
            CommandSyntaxException::at(self, invalid, &[Component::literal(number.clone())])
        })
    }

    pub fn read_int(&mut self) -> Result<i32, CommandSyntaxException> {
        self.read_number(
            |text| text.parse().ok(),
            "parsing.int.expected",
            "parsing.int.invalid",
        )
    }

    pub fn read_long(&mut self) -> Result<i64, CommandSyntaxException> {
        self.read_number(
            |text| text.parse().ok(),
            "parsing.long.expected",
            "parsing.long.invalid",
        )
    }

    pub fn read_float(&mut self) -> Result<f32, CommandSyntaxException> {
        self.read_number(
            |text| text.parse().ok(),
            "parsing.float.expected",
            "parsing.float.invalid",
        )
    }

    pub fn read_double(&mut self) -> Result<f64, CommandSyntaxException> {
        self.read_number(
            |text| text.parse().ok(),
            "parsing.double.expected",
            "parsing.double.invalid",
        )
    }

    /// `readBoolean`.
    pub fn read_boolean(&mut self) -> Result<bool, CommandSyntaxException> {
        let start = self.cursor;
        let value = self.read_string()?;
        if value.is_empty() {
            return Err(CommandSyntaxException::at(
                self,
                "parsing.bool.expected",
                &[],
            ));
        }
        match value.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => {
                self.cursor = start;
                Err(CommandSyntaxException::at(
                    self,
                    "parsing.bool.invalid",
                    &[Component::literal(value)],
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_prints_numbers_its_own_way() {
        assert_eq!(java_double(1.0), "1.0");
        assert_eq!(java_double(-2.5), "-2.5");
        assert_eq!(java_double(1.0e7), "1.0E7");
        assert_eq!(java_double(0.001), "0.001");
        assert_eq!(java_double(0.0001), "1.0E-4");
        assert_eq!(java_double(1234567.0), "1234567.0");
        assert_eq!(java_float(3.4028235e38), "3.4028235E38");
        assert_eq!(java_float(0.5), "0.5");
        assert_eq!(java_double(100.0), "100.0");
    }

    #[test]
    fn numbers_stop_at_what_a_number_cannot_hold() {
        let mut reader = StringReader::new("12abc");
        assert_eq!(reader.read_int().unwrap(), 12);
        assert_eq!(reader.cursor(), 2);
        let mut reader = StringReader::new("1.2.3");
        let error = reader.read_double().unwrap_err();
        assert_eq!(error.cursor(), 0);
        assert_eq!(reader.cursor(), 0);
    }

    #[test]
    fn quoted_strings_unescape() {
        let mut reader = StringReader::new("\"a\\\"b\" rest");
        assert_eq!(reader.read_string().unwrap(), "a\"b");
        assert_eq!(reader.remaining(), " rest");
        let mut reader = StringReader::new("\"open");
        assert!(reader.read_string().is_err());
    }

    #[test]
    fn the_context_shows_ten_characters_before_the_cursor() {
        let mut reader = StringReader::new("give @p minecraft:stone 99999999999");
        reader.set_cursor(23);
        let error = CommandSyntaxException::at(&reader, "x", &[]);
        assert_eq!(error.context_text().unwrap(), "...raft:stone<--[HERE]");
    }
}
