//! A one-line text box: `EditBox`, without its drawing and its mouse.
//!
//! The text, where the cursor is, where the selection starts, and how far
//! the text is scrolled so the cursor stays in the box. Indices count
//! characters, where the reference counts UTF-16 units; the two agree for
//! what fits a search box.

use crate::gui::font::Font;
use crate::gui::{GuiAtlas, GuiFrame};

/// A key an edit box acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKey {
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    /// Select all.
    A,
    /// Copy.
    C,
    /// Paste.
    V,
    /// Cut.
    X,
}

/// What the box asks of the clipboard.
pub trait Clipboard {
    fn get(&mut self) -> String;
    fn set(&mut self, text: &str);
}

/// `EditBox`.
#[derive(Debug, Clone)]
pub struct EditBox {
    value: Vec<char>,
    cursor: usize,
    highlight: usize,
    display_pos: usize,
    max_length: usize,
    /// `getInnerWidth`.
    width: i32,
    /// `textColor`: the default the reference gives a box.
    colour: u32,
    /// `setSuggestion`: the grey text after the cursor.
    suggestion: Option<String>,
}

/// `Font.plainSubstrByWidth`: the longest head of the text that fits.
fn head_by_width(font: &Font, text: &[char], width: i32) -> usize {
    let mut used = 0.0_f32;
    for (index, character) in text.iter().enumerate() {
        let advance = font
            .glyph(*character)
            .map_or(0.0, |glyph| glyph.advance(false));
        used += advance;
        if used > width as f32 {
            return index;
        }
    }
    text.len()
}

/// `Font.plainSubstrByWidth` from the end: the longest tail that fits.
fn tail_by_width(font: &Font, text: &[char], width: i32) -> usize {
    let mut used = 0.0_f32;
    for (count, character) in text.iter().rev().enumerate() {
        let advance = font
            .glyph(*character)
            .map_or(0.0, |glyph| glyph.advance(false));
        used += advance;
        if used > width as f32 {
            return count;
        }
    }
    text.len()
}

impl EditBox {
    /// A box of an inner width and a longest text.
    pub fn new(width: i32, max_length: usize) -> Self {
        Self {
            value: Vec::new(),
            cursor: 0,
            highlight: 0,
            display_pos: 0,
            max_length,
            width,
            colour: 0xFFE0_E0E0,
            suggestion: None,
        }
    }

    /// `setTextColor`.
    pub fn with_colour(mut self, colour: u32) -> Self {
        self.colour = colour;
        self
    }

    /// `setSuggestion`.
    pub fn set_suggestion(&mut self, suggestion: Option<String>) {
        self.suggestion = suggestion;
    }

    /// `getCursorPosition`.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// `setCursorPosition` with the selection dropped (`setHighlightPos`).
    pub fn set_cursor_position(&mut self, position: usize, font: &Font) {
        self.move_cursor_to(position, false, font);
    }

    pub fn value(&self) -> String {
        self.value.iter().collect()
    }

    /// `onClick`: the cursor goes to the place `offset` units into the text
    /// that is shown.
    pub fn click_at(&mut self, offset: i32, font: &Font) {
        let offset = offset.min(self.width);
        let rest = &self.value[self.display_pos.min(self.value.len())..];
        let position = head_by_width(font, rest, offset) + self.display_pos;
        self.move_cursor_to(position, false, font);
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// `setValue`: the text, the cursor at its end.
    pub fn set_value(&mut self, text: &str, font: &Font) {
        self.value = text.chars().take(self.max_length).collect();
        self.move_cursor_to(self.value.len(), false, font);
    }

    /// `getHighlighted`.
    pub fn highlighted(&self) -> String {
        let (start, end) = self.selection();
        self.value[start..end].iter().collect()
    }

    fn selection(&self) -> (usize, usize) {
        (
            self.cursor.min(self.highlight),
            self.cursor.max(self.highlight),
        )
    }

    /// `insertText`: over the selection, cut to what fits, filtered as chat
    /// is.
    pub fn insert_text(&mut self, input: &str, font: &Font) {
        let (start, end) = self.selection();
        let room = (self.max_length + end).saturating_sub(self.value.len() + start);
        if room == 0 {
            return;
        }
        let text: Vec<char> = input
            .chars()
            .filter(|&c| c != '\u{a7}' && c >= ' ' && c != '\u{7f}')
            .take(room)
            .collect();
        let length = text.len();
        self.value.splice(start..end, text);
        self.set_cursor(start + length, font);
        self.highlight = self.cursor;
    }

    fn delete_to(&mut self, position: usize, font: &Font) {
        if self.value.is_empty() {
            return;
        }
        if self.highlight != self.cursor {
            self.insert_text("", font);
            return;
        }
        let (start, end) = (position.min(self.cursor), position.max(self.cursor));
        if start != end {
            self.value.drain(start..end);
            self.set_cursor(start, font);
            self.move_cursor_to(start, false, font);
        }
    }

    /// `getWordPosition`: a word away, forwards or back.
    fn word_position(&self, direction: i32, from: usize) -> usize {
        let mut result = from;
        for _ in 0..direction.unsigned_abs() {
            if direction > 0 {
                let length = self.value.len();
                match self.value[result..].iter().position(|&c| c == ' ') {
                    None => result = length,
                    Some(offset) => {
                        result += offset;
                        while result < length && self.value[result] == ' ' {
                            result += 1;
                        }
                    }
                }
            } else {
                while result > 0 && self.value[result - 1] == ' ' {
                    result -= 1;
                }
                while result > 0 && self.value[result - 1] != ' ' {
                    result -= 1;
                }
            }
        }
        result
    }

    fn set_cursor(&mut self, position: usize, font: &Font) {
        self.cursor = position.min(self.value.len());
        self.scroll_to(self.cursor, font);
    }

    fn move_cursor_to(&mut self, position: usize, extend: bool, font: &Font) {
        self.set_cursor(position, font);
        if !extend {
            self.highlight = self.cursor;
        }
    }

    /// `scrollTo`: the text is shifted so the position shows.
    fn scroll_to(&mut self, position: usize, font: &Font) {
        let length = self.value.len();
        self.display_pos = self.display_pos.min(length);
        let visible = head_by_width(font, &self.value[self.display_pos..], self.width);
        let last_visible = self.display_pos + visible;
        if position == self.display_pos {
            self.display_pos =
                self.display_pos
                    .saturating_sub(tail_by_width(font, &self.value, self.width));
        }
        if position > last_visible {
            self.display_pos += position - last_visible;
        } else if position <= self.display_pos {
            self.display_pos = position;
        }
        self.display_pos = self.display_pos.min(length);
    }

    /// `keyPressed` for a key the box knows. Whether the key was taken.
    pub fn key_pressed(
        &mut self,
        key: EditKey,
        control: bool,
        shift: bool,
        font: &Font,
        clipboard: &mut dyn Clipboard,
    ) -> bool {
        match key {
            EditKey::Backspace => {
                if control && self.highlight == self.cursor {
                    let position = self.word_position(-1, self.cursor);
                    self.delete_to(position, font);
                } else if self.highlight != self.cursor {
                    self.insert_text("", font);
                } else {
                    self.delete_to(self.cursor.saturating_sub(1), font);
                }
            }
            EditKey::Delete => {
                if control && self.highlight == self.cursor {
                    let position = self.word_position(1, self.cursor);
                    self.delete_to(position, font);
                } else if self.highlight != self.cursor {
                    self.insert_text("", font);
                } else {
                    self.delete_to((self.cursor + 1).min(self.value.len()), font);
                }
            }
            EditKey::Left => {
                let target = if control {
                    self.word_position(-1, self.cursor)
                } else {
                    self.cursor.saturating_sub(1)
                };
                self.move_cursor_to(target, shift, font);
            }
            EditKey::Right => {
                let target = if control {
                    self.word_position(1, self.cursor)
                } else {
                    (self.cursor + 1).min(self.value.len())
                };
                self.move_cursor_to(target, shift, font);
            }
            EditKey::Home => self.move_cursor_to(0, shift, font),
            EditKey::End => self.move_cursor_to(self.value.len(), shift, font),
            EditKey::A if control => {
                self.move_cursor_to(self.value.len(), false, font);
                self.highlight = 0;
            }
            EditKey::C if control => clipboard.set(&self.highlighted()),
            EditKey::V if control => {
                let text = clipboard.get();
                self.insert_text(&text, font);
            }
            EditKey::X if control => {
                clipboard.set(&self.highlighted());
                self.insert_text("", font);
            }
            _ => return false,
        }
        true
    }

    /// `charTyped`. Whether the character was taken.
    pub fn char_typed(&mut self, character: char, font: &Font) -> bool {
        if character == '\u{a7}' || character < ' ' || character == '\u{7f}' {
            return false;
        }
        self.insert_text(&character.to_string(), font);
        true
    }

    /// What is shown: the text from the scroll on, as much as fits, with
    /// where the cursor and the selection are in it, in characters.
    pub fn shown(&self, font: &Font) -> Shown {
        let rest = &self.value[self.display_pos..];
        let fits = head_by_width(font, rest, self.width);
        let text: String = rest[..fits].iter().collect();
        let relative = |position: usize| position as isize - self.display_pos as isize;
        Shown {
            text,
            cursor: relative(self.cursor),
            highlight: relative(self.highlight).clamp(0, fits as isize) as usize,
            insert: self.cursor < self.value.len() || self.value.len() >= self.max_length,
        }
    }
}

/// A text formatter: a stretch of text and where it starts in the whole,
/// cut into parts that each have a colour and may be in italics.
pub type Formatter<'a> = dyn Fn(&str, usize) -> Option<Vec<(String, u32, bool)>> + 'a;

/// The part of the text the box shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shown {
    pub text: String,
    /// The cursor's place in `text`, negative or past its end when it is
    /// out of view.
    pub cursor: isize,
    /// The selection's other end in `text`.
    pub highlight: usize,
    /// The cursor overwrites (a line) rather than follows the text (`_`).
    pub insert: bool,
}

impl EditBox {
    /// `EditBox.extractWidgetRenderState` without a border, its text's corner
    /// at `(x, y)`: the text with the cursor blinking in it, the selection
    /// over it. `focused_at` is when the box took the focus, which the
    /// cursor blinks from.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        at: (i32, i32),
        focused_at: std::time::Instant,
    ) {
        self.draw_formatted(frame, atlas, font, at, Some(focused_at), None);
    }

    /// `draw` with a formatter (`addFormatter`): given a stretch of the text
    /// and where it starts in the whole, it cuts it into parts in colours of
    /// their own, or says nothing of it.
    pub fn draw_formatted(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        (text_x, text_y): (i32, i32),
        focused_at: Option<std::time::Instant>,
        formatter: Option<&Formatter<'_>>,
    ) {
        // `applyFormat`: the parts, drawn one after another; their width.
        let draw_text =
            |frame: &mut GuiFrame, text: &str, offset: usize, x: i32, colour: u32| -> i32 {
                match formatter.and_then(|format| format(text, offset)) {
                    Some(parts) => {
                        let mut at = x;
                        for (part, rgb, italic) in parts {
                            if italic {
                                let runs = [photon_game::text::Run {
                                    text: part,
                                    style: photon_game::text::Style {
                                        colour: Some(rgb),
                                        italic: Some(true),
                                        ..photon_game::text::Style::default()
                                    },
                                }];
                                frame.styled_text(atlas, font, &runs, at, text_y, colour, true);
                                at += font.width_of_runs(&runs);
                            } else {
                                frame.text_with_shadow(
                                    atlas,
                                    font,
                                    &part,
                                    at,
                                    text_y,
                                    0xFF00_0000 | rgb,
                                );
                                at += font.width(&part);
                            }
                        }
                        at - x
                    }
                    None => {
                        frame.text_with_shadow(atlas, font, text, x, text_y, colour);
                        font.width(text)
                    }
                }
            };
        let shown = self.shown(font);
        let chars: Vec<char> = shown.text.chars().collect();
        let cursor_on_screen = shown.cursor >= 0 && shown.cursor as usize <= chars.len();
        let blink = focused_at.is_some_and(|at| (at.elapsed().as_millis() / 300).is_multiple_of(2));
        let colour = self.colour;
        let right = text_x + self.width;
        let mut draw_x = text_x;
        if !chars.is_empty() {
            let half: String = if cursor_on_screen {
                chars[..shown.cursor as usize].iter().collect()
            } else {
                shown.text.clone()
            };
            draw_x += draw_text(frame, &half, self.display_pos, draw_x, colour) + 1;
        }
        let mut cursor_x = draw_x;
        if !cursor_on_screen {
            cursor_x = if shown.cursor > 0 { right } else { text_x };
        } else if shown.insert {
            cursor_x -= 1;
            draw_x -= 1;
        }
        if !chars.is_empty() && cursor_on_screen && (shown.cursor as usize) < chars.len() {
            let rest: String = chars[shown.cursor as usize..].iter().collect();
            draw_text(frame, &rest, self.cursor, draw_x, colour);
        }
        if !shown.insert
            && let Some(suggestion) = &self.suggestion
        {
            frame.text_with_shadow(atlas, font, suggestion, cursor_x - 1, text_y, 0xFF80_8080);
        }
        if shown.highlight as isize != shown.cursor {
            let before: String = chars[..shown.highlight].iter().collect();
            let highlight_x = text_x + font.width(&before);
            frame.fill(
                atlas,
                (
                    cursor_x.min(right),
                    text_y - 1,
                    (highlight_x - 1).min(right),
                    text_y + 1 + 9,
                ),
                0x800000FF,
            );
        }
        if blink && cursor_on_screen {
            if shown.insert {
                frame.fill(
                    atlas,
                    (cursor_x, text_y - 1, cursor_x + 1, text_y + 10),
                    colour,
                );
            } else {
                frame.text_with_shadow(atlas, font, "_", cursor_x, text_y, colour);
            }
        }
    }
}

/// The edit key a glfw key is, if any.
pub fn edit_key(key: glfw::Key) -> Option<EditKey> {
    Some(match key {
        glfw::Key::Backspace => EditKey::Backspace,
        glfw::Key::Delete => EditKey::Delete,
        glfw::Key::Left => EditKey::Left,
        glfw::Key::Right => EditKey::Right,
        glfw::Key::Home => EditKey::Home,
        glfw::Key::End => EditKey::End,
        glfw::Key::A => EditKey::A,
        glfw::Key::C => EditKey::C,
        glfw::Key::V => EditKey::V,
        glfw::Key::X => EditKey::X,
        _ => return None,
    })
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

    fn font() -> Option<Font> {
        Font::load(std::path::Path::new(
            &std::env::var("PHOTON_ASSETS").unwrap_or_default(),
        ))
    }

    #[test]
    fn typing_moves_the_cursor_and_the_keys_edit() {
        let Some(font) = font() else { return };
        let mut clipboard = Held(String::new());
        let mut edit = EditBox::new(80, 50);
        for character in "hello world".chars() {
            assert!(edit.char_typed(character, &font));
        }
        assert_eq!(edit.value(), "hello world");
        edit.key_pressed(EditKey::Backspace, false, false, &font, &mut clipboard);
        assert_eq!(edit.value(), "hello worl");
        // A word back, then a word deleted forwards.
        edit.key_pressed(EditKey::Left, true, false, &font, &mut clipboard);
        edit.key_pressed(EditKey::Delete, true, false, &font, &mut clipboard);
        assert_eq!(edit.value(), "hello ");
        edit.key_pressed(EditKey::A, true, false, &font, &mut clipboard);
        edit.key_pressed(EditKey::X, true, false, &font, &mut clipboard);
        assert_eq!(edit.value(), "");
        assert_eq!(clipboard.0, "hello ");
        edit.key_pressed(EditKey::V, true, false, &font, &mut clipboard);
        assert_eq!(edit.value(), "hello ");
    }

    #[test]
    fn the_text_scrolls_to_keep_the_cursor_in_view() {
        let Some(font) = font() else { return };
        let mut edit = EditBox::new(30, 50);
        edit.set_value("abcdefghijklmnopqrstuvwxyz", &font);
        let shown = edit.shown(&font);
        assert!(shown.text.len() < 26);
        assert!(shown.text.ends_with('z'));
        assert!(!shown.insert);
        assert_eq!(shown.cursor as usize, shown.text.len());
    }

    #[test]
    fn the_longest_text_is_kept() {
        let Some(font) = font() else { return };
        let mut edit = EditBox::new(80, 5);
        edit.insert_text("abcdefgh", &font);
        assert_eq!(edit.value(), "abcde");
        assert!(edit.shown(&font).insert);
        assert!(!edit.char_typed('\u{a7}', &font));
    }
}
