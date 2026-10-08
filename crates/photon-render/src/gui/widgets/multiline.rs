//! `MultilineTextField`: a text of many lines, wrapped to a width, with a cursor and a selection,
//! and the keys that edit it. Indices count characters, where the reference counts UTF-16 units;
//! the two agree outside the supplementary planes.

use crate::gui::font::Font;
use crate::gui::widgets::edit_box::Clipboard;

/// A stretch of the text, `[begin, end)` (`StringView`).
pub type Span = (usize, usize);

fn advance(font: &Font, character: char) -> f32 {
    font.glyph(character)
        .map_or(0.0, |glyph| glyph.advance(false))
}

/// `StringSplitter.splitLines` without the spaces and new lines lines break at: each line of
/// `text` that fits `width`, broken after the last space that fits or, with none, inside the word.
/// A formatting code (`§` and the character after it) takes no room.
pub fn split_lines(font: &Font, text: &[char], width: i32) -> Vec<Span> {
    let max = (width as f32).max(1.0);
    let mut lines = Vec::new();
    let mut start = 0;
    while start < text.len() {
        // `LineBreakFinder` from `start`.
        let (mut used, mut last_space, mut any_width) = (0.0_f32, None, false);
        let mut found = None;
        let mut position = start;
        while position < text.len() {
            let character = text[position];
            if character == '\u{a7}' {
                if position + 1 >= text.len() {
                    break;
                }
                position += 2;
                continue;
            }
            if character == '\n' {
                found = Some(position);
                break;
            }
            if character == ' ' {
                last_space = Some(position);
            }
            let width = advance(font, character);
            used += width;
            if any_width && used > max {
                found = Some(last_space.unwrap_or(position));
                break;
            }
            any_width |= width != 0.0;
            position += 1;
        }
        let Some(split) = found else {
            lines.push((start, text.len()));
            break;
        };
        lines.push((start, split));
        start = if matches!(text.get(split), Some('\n' | ' ')) {
            split + 1
        } else {
            split
        };
    }
    lines
}

/// `Font.plainSubstrByWidth`: how many characters of the head of `text` fit `width`.
pub fn head_by_width(font: &Font, text: &[char], width: i32) -> usize {
    let mut used = 0.0_f32;
    for (index, character) in text.iter().enumerate() {
        used += advance(font, *character);
        if used > width as f32 {
            return index;
        }
    }
    text.len()
}

/// `Character.isWhitespace`.
fn java_whitespace(character: char) -> bool {
    matches!(character, '\t'..='\r' | '\u{1c}'..='\u{1f}')
        || (character.is_whitespace()
            && !matches!(character, '\u{85}' | '\u{a0}' | '\u{2007}' | '\u{202f}'))
}

/// `StringUtil.endsWithNewLine`: a line terminator (`\v`) last.
fn ends_with_new_line(text: &[char]) -> bool {
    matches!(
        text.last(),
        Some('\n' | '\u{b}' | '\u{c}' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}')
    )
}

/// `MultilineTextField`.
#[derive(Debug, Clone)]
pub struct MultilineTextField {
    value: Vec<char>,
    cursor: usize,
    select_cursor: usize,
    selecting: bool,
    character_limit: Option<usize>,
    line_limit: Option<usize>,
    width: i32,
    lines: Vec<Span>,
}

impl MultilineTextField {
    /// An empty text `width` units wide, with no limits.
    pub fn new(font: &Font, width: i32) -> Self {
        let mut field = Self {
            value: Vec::new(),
            cursor: 0,
            select_cursor: 0,
            selecting: false,
            character_limit: None,
            line_limit: None,
            width,
            lines: Vec::new(),
        };
        field.set_value(font, "", false);
        field
    }

    /// `setCharacterLimit`.
    pub fn set_character_limit(&mut self, limit: usize) {
        self.character_limit = Some(limit);
    }

    /// `setLineLimit`.
    pub fn set_line_limit(&mut self, limit: usize) {
        self.line_limit = Some(limit);
    }

    pub fn value(&self) -> String {
        self.value.iter().collect()
    }

    pub fn chars(&self) -> &[char] {
        &self.value
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// The display lines (`iterateLines`).
    pub fn lines(&self) -> &[Span] {
        &self.lines
    }

    /// `getSelected`.
    pub fn selected(&self) -> Span {
        (
            self.cursor.min(self.select_cursor),
            self.cursor.max(self.select_cursor),
        )
    }

    /// `hasSelection`.
    pub fn has_selection(&self) -> bool {
        self.select_cursor != self.cursor
    }

    pub fn set_selecting(&mut self, selecting: bool) {
        self.selecting = selecting;
    }

    fn overflows_line_limit(&self, font: &Font, value: &[char]) -> bool {
        self.line_limit.is_some_and(|limit| {
            split_lines(font, value, self.width).len() + usize::from(ends_with_new_line(value))
                > limit
        })
    }

    /// `setValue`: the text, cut to the character limit, the cursor at its end; kept only within
    /// the line limit unless `overflow` lets it past.
    pub fn set_value(&mut self, font: &Font, value: &str, overflow: bool) {
        let mut value: Vec<char> = value.chars().collect();
        if let Some(limit) = self.character_limit {
            value.truncate(limit);
        }
        if overflow || !self.overflows_line_limit(font, &value) {
            self.value = value;
            self.cursor = self.value.len();
            self.select_cursor = self.cursor;
            self.reflow(font);
        }
    }

    /// `insertText`: chat's characters and new lines over the selection, cut to the room the
    /// character limit leaves, kept only within the line limit.
    pub fn insert_text(&mut self, font: &Font, input: &str) {
        if input.is_empty() && !self.has_selection() {
            return;
        }
        let mut text: Vec<char> = input
            .chars()
            .filter(|&c| (c != '\u{a7}' && c >= ' ' && c != '\u{7f}') || c == '\n')
            .collect();
        if let Some(limit) = self.character_limit {
            text.truncate(limit.saturating_sub(self.value.len()));
        }
        let (begin, end) = self.selected();
        let mut value = self.value[..begin].to_vec();
        value.extend_from_slice(&text);
        value.extend_from_slice(&self.value[end..]);
        if !self.overflows_line_limit(font, &value) {
            self.value = value;
            self.cursor = begin + text.len();
            self.select_cursor = self.cursor;
            self.reflow(font);
        }
    }

    /// `deleteText`: the selection, or `dir` characters from the cursor.
    fn delete_text(&mut self, font: &Font, dir: isize) {
        if !self.has_selection() {
            self.select_cursor = self.cursor.saturating_add_signed(dir).min(self.value.len());
        }
        self.insert_text(font, "");
    }

    /// `seekCursor` to a place.
    pub fn seek_cursor(&mut self, at: usize) {
        self.cursor = at.min(self.value.len());
        if !self.selecting {
            self.select_cursor = self.cursor;
        }
    }

    /// `getLineAtCursor`.
    pub fn line_at_cursor(&self) -> Option<usize> {
        self.lines
            .iter()
            .position(|&(begin, end)| self.cursor >= begin && self.cursor <= end)
    }

    /// `getLineView`, clamped to the lines.
    pub fn line_view(&self, index: usize) -> Span {
        self.lines[index.min(self.lines.len() - 1)]
    }

    /// `getCursorLineView` with an offset.
    fn cursor_line_view(&self, offset: isize) -> Span {
        match self.line_at_cursor() {
            None => self.lines[self.lines.len() - 1],
            Some(line) => self.line_view(line.saturating_add_signed(offset)),
        }
    }

    fn width_of(&self, font: &Font, (begin, end): Span) -> i32 {
        font.width(&self.value[begin..end].iter().collect::<String>())
    }

    /// `seekCursorLine`: to the line above or below, as far in as the cursor was (two units to
    /// the right of it).
    fn seek_cursor_line(&mut self, font: &Font, offset: isize) {
        let left = self.width_of(font, (self.cursor_line_view(0).0, self.cursor)) + 2;
        let (begin, end) = self.cursor_line_view(offset);
        let column = head_by_width(font, &self.value[begin..end], left);
        self.seek_cursor(begin + column);
    }

    /// `seekCursorToPoint`: the place under a point of the text, in units from its corner.
    pub fn seek_cursor_to_point(&mut self, font: &Font, (x, y): (f64, f64)) {
        let line = ((y / 9.0).floor() as i64).clamp(0, self.lines.len() as i64 - 1) as usize;
        let (begin, end) = self.lines[line];
        let column = head_by_width(font, &self.value[begin..end], x.floor() as i32);
        self.seek_cursor(begin + column);
    }

    /// `selectWordAtCursor`.
    pub fn select_word_at_cursor(&mut self) {
        let (begin, end) = self.previous_word();
        self.seek_cursor(begin);
        self.selecting = true;
        self.seek_cursor(end);
    }

    fn word_end(&self, from: usize) -> usize {
        let mut end = from;
        while end < self.value.len() && !java_whitespace(self.value[end]) {
            end += 1;
        }
        end
    }

    /// `getPreviousWord`.
    fn previous_word(&self) -> Span {
        if self.value.is_empty() {
            return (0, 0);
        }
        let mut start = self.cursor.min(self.value.len() - 1);
        while start > 0 && java_whitespace(self.value[start - 1]) {
            start -= 1;
        }
        while start > 0 && !java_whitespace(self.value[start - 1]) {
            start -= 1;
        }
        (start, self.word_end(start))
    }

    /// `getNextWord`.
    fn next_word(&self) -> Span {
        if self.value.is_empty() {
            return (0, 0);
        }
        let mut start = self.cursor.min(self.value.len() - 1);
        while start < self.value.len() && !java_whitespace(self.value[start]) {
            start += 1;
        }
        while start < self.value.len() && java_whitespace(self.value[start]) {
            start += 1;
        }
        (start, self.word_end(start))
    }

    /// `keyPressed`: whether the key was taken.
    pub fn key_pressed(
        &mut self,
        font: &Font,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> bool {
        use glfw::Key;
        self.selecting = shift;
        let selected = |field: &Self| {
            let (begin, end) = field.selected();
            field.value[begin..end].iter().collect::<String>()
        };
        match key {
            Key::A if control && !shift => {
                self.cursor = self.value.len();
                self.select_cursor = 0;
            }
            Key::C if control && !shift => clipboard.set(&selected(self)),
            Key::V if control && !shift => {
                let text = clipboard.get();
                self.insert_text(font, &text);
            }
            Key::X if control && !shift => {
                clipboard.set(&selected(self));
                self.insert_text(font, "");
            }
            Key::Enter | Key::KpEnter => self.insert_text(font, "\n"),
            Key::Backspace | Key::Delete => {
                let back = key == Key::Backspace;
                let dir = if control {
                    let (begin, _) = if back {
                        self.previous_word()
                    } else {
                        self.next_word()
                    };
                    begin as isize - self.cursor as isize
                } else if back {
                    -1
                } else {
                    1
                };
                self.delete_text(font, dir);
            }
            Key::Right | Key::Left => {
                let forward = key == Key::Right;
                let at = if control {
                    if forward {
                        self.next_word().0
                    } else {
                        self.previous_word().0
                    }
                } else if forward {
                    self.cursor + 1
                } else {
                    self.cursor.saturating_sub(1)
                };
                self.seek_cursor(at);
            }
            Key::Down | Key::Up => {
                if !control {
                    self.seek_cursor_line(font, if key == Key::Down { 1 } else { -1 });
                }
            }
            Key::PageUp => self.seek_cursor(0),
            Key::PageDown => self.seek_cursor(self.value.len()),
            Key::Home => {
                let at = if control {
                    0
                } else {
                    self.cursor_line_view(0).0
                };
                self.seek_cursor(at);
            }
            Key::End => {
                let at = if control {
                    self.value.len()
                } else {
                    self.cursor_line_view(0).1
                };
                self.seek_cursor(at);
            }
            _ => return false,
        }
        true
    }

    /// `reflowDisplayLines`.
    fn reflow(&mut self, font: &Font) {
        self.lines = if self.value.is_empty() {
            vec![(0, 0)]
        } else {
            let mut lines = split_lines(font, &self.value, self.width);
            if self.value.last() == Some(&'\n') {
                lines.push((self.value.len(), self.value.len()));
            }
            lines
        };
    }
}
