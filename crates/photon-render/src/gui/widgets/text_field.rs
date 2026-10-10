//! `TextFieldHelper`: the cursor and the selection of a text a screen owns (a sign's line, a book's
//! page), and the keys and characters that edit it. The screen hands in the text and a check the
//! new text must pass. Indices count characters, where the reference counts UTF-16 units; the two
//! agree outside the supplementary planes.

use super::edit_box::{Clipboard, EditKey};

/// `TextFieldHelper`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextField {
    cursor: usize,
    selection: usize,
}

/// `StringUtil.isAllowedChatCharacter`.
fn allowed(character: char) -> bool {
    character != '\u{a7}' && character >= ' ' && character != '\u{7f}'
}

/// `ChatFormatting.stripFormatting`: each `\u{a7}` and the code after it go.
fn strip_formatting(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\u{a7}'
            && chars
                .peek()
                .is_some_and(|code| "0123456789abcdefklmnorABCDEFKLMNOR".contains(*code))
        {
            chars.next();
            continue;
        }
        out.push(character);
    }
    out
}

/// `StringSplitter.getWordPosition` with spaces stripped: `count` words on from `from`, a word
/// ending at a space or a new line.
fn word_position(text: &[char], count: i32, from: usize) -> usize {
    let gap = |c: char| c == ' ' || c == '\n';
    let mut result = from;
    for _ in 0..count.unsigned_abs() {
        if count < 0 {
            while result > 0 && gap(text[result - 1]) {
                result -= 1;
            }
            while result > 0 && !gap(text[result - 1]) {
                result -= 1;
            }
        } else {
            match text[result.min(text.len())..].iter().position(|&c| gap(c)) {
                None => result = text.len(),
                Some(offset) => {
                    result += offset;
                    while result < text.len() && gap(text[result]) {
                        result += 1;
                    }
                }
            }
        }
    }
    result
}

impl TextField {
    /// A helper with its cursor at the end of `text` (`setCursorToEnd`).
    pub fn new(text: &str) -> Self {
        let mut field = Self::default();
        field.set_cursor_to_end(text, false);
        field
    }

    /// `getCursorPos`.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// `getSelectionPos`.
    pub fn selection(&self) -> usize {
        self.selection
    }

    fn reset_selection(&mut self, selecting: bool) {
        if !selecting {
            self.selection = self.cursor;
        }
    }

    /// `setCursorToEnd`.
    pub fn set_cursor_to_end(&mut self, text: &str, selecting: bool) {
        self.cursor = text.chars().count();
        self.reset_selection(selecting);
    }

    /// `deleteSelection`: the text without what is selected, the cursor where it was.
    fn delete_selection(&mut self, text: &[char]) -> Vec<char> {
        if self.selection == self.cursor {
            return text.to_vec();
        }
        let (start, end) = (
            self.cursor.min(self.selection),
            self.cursor.max(self.selection),
        );
        let mut out = text[..start].to_vec();
        out.extend_from_slice(&text[end..]);
        self.cursor = start;
        self.selection = start;
        out
    }

    /// `insertText`: over the selection, kept only when the new text passes `valid`.
    pub fn insert(&mut self, text: &mut String, input: &str, valid: &dyn Fn(&str) -> bool) {
        let mut message: Vec<char> = text.chars().collect();
        if self.selection != self.cursor {
            message = self.delete_selection(&message);
        }
        self.cursor = self.cursor.min(message.len());
        let inserted: Vec<char> = input.chars().collect();
        let mut candidate = message[..self.cursor].to_vec();
        candidate.extend_from_slice(&inserted);
        candidate.extend_from_slice(&message[self.cursor..]);
        let candidate: String = candidate.into_iter().collect();
        if valid(&candidate) {
            let length = candidate.chars().count();
            *text = candidate;
            self.cursor = length.min(self.cursor + inserted.len());
            self.selection = self.cursor;
        }
    }

    /// `charTyped`: a character chat allows goes in.
    pub fn char_typed(&mut self, text: &mut String, character: char, valid: &dyn Fn(&str) -> bool) {
        if allowed(character) {
            self.insert(text, &character.to_string(), valid);
        }
    }

    /// `removeCharsFromCursor`: the selection, or `count` characters before or after the cursor.
    fn remove_chars(&mut self, text: &mut String, count: isize) {
        let message: Vec<char> = text.chars().collect();
        if message.is_empty() {
            return;
        }
        let updated = if self.selection != self.cursor {
            self.delete_selection(&message)
        } else {
            let other = self.cursor.saturating_add_signed(count).min(message.len());
            let (start, end) = (other.min(self.cursor), other.max(self.cursor));
            let mut out = message[..start].to_vec();
            out.extend_from_slice(&message[end..]);
            if count < 0 {
                self.cursor = start;
                self.selection = start;
            }
            out
        };
        *text = updated.into_iter().collect();
    }

    /// `keyPressed`: whether the key was taken. Delete edits but is not taken, as in the
    /// reference.
    pub fn key_pressed(
        &mut self,
        text: &mut String,
        key: EditKey,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
        valid: &dyn Fn(&str) -> bool,
    ) -> bool {
        let message: Vec<char> = text.chars().collect();
        let selected = || {
            let (start, end) = (
                self.cursor.min(self.selection),
                self.cursor.max(self.selection),
            );
            message[start.min(message.len())..end.min(message.len())]
                .iter()
                .collect::<String>()
        };
        match key {
            EditKey::A if control && !shift => {
                self.selection = 0;
                self.cursor = message.len();
            }
            EditKey::C if control && !shift => clipboard.set(&selected()),
            EditKey::V if control && !shift => {
                let pasted = strip_formatting(&clipboard.get().replace('\r', ""));
                self.insert(text, &pasted, valid);
                self.selection = self.cursor;
            }
            EditKey::X if control && !shift => {
                clipboard.set(&selected());
                *text = self.delete_selection(&message).into_iter().collect();
            }
            EditKey::Backspace => {
                let count = if control {
                    word_position(&message, -1, self.cursor) as isize - self.cursor as isize
                } else {
                    -1
                };
                self.remove_chars(text, count);
            }
            EditKey::Delete => {
                let count = if control {
                    word_position(&message, 1, self.cursor) as isize - self.cursor as isize
                } else {
                    1
                };
                self.remove_chars(text, count);
                return false;
            }
            EditKey::Left | EditKey::Right => {
                let forward = key == EditKey::Right;
                self.cursor = if control {
                    word_position(&message, if forward { 1 } else { -1 }, self.cursor)
                } else if forward {
                    (self.cursor + 1).min(message.len())
                } else {
                    self.cursor.saturating_sub(1)
                };
                self.reset_selection(shift);
            }
            EditKey::Home => {
                self.cursor = 0;
                self.reset_selection(shift);
            }
            EditKey::End => self.set_cursor_to_end(text, shift),
            _ => return false,
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Held(String);

    impl Clipboard for Held {
        fn get(&mut self) -> String {
            self.0.clone()
        }

        fn set(&mut self, text: &str) {
            self.0 = text.to_owned();
        }
    }

    #[test]
    fn edits_by_words_and_keeps_only_valid_text() {
        let short = |text: &str| text.chars().count() <= 8;
        let mut clipboard = Held("\u{a7}cab\rc".into());
        let mut text = "one two".to_owned();
        let mut field = TextField::new(&text);
        field.key_pressed(
            &mut text,
            EditKey::Backspace,
            (true, false),
            &mut clipboard,
            &short,
        );
        assert_eq!((text.as_str(), field.cursor()), ("one ", 4));
        field.key_pressed(&mut text, EditKey::V, (true, false), &mut clipboard, &short);
        assert_eq!(text, "one abc");
        // Too long: refused, the cursor stays.
        field.insert(&mut text, "xyz", &short);
        assert_eq!((text.as_str(), field.cursor()), ("one abc", 7));
        field.key_pressed(
            &mut text,
            EditKey::Left,
            (true, true),
            &mut clipboard,
            &short,
        );
        assert_eq!((field.cursor(), field.selection()), (4, 7));
        field.char_typed(&mut text, 'Z', &short);
        assert_eq!((text.as_str(), field.cursor()), ("one Z", 5));
        field.char_typed(&mut text, '\u{a7}', &short);
        assert_eq!(text, "one Z");
    }
}
