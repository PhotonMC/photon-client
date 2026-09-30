//! The chat: its lines as `ChatComponent` keeps, wraps, fades and draws them
//! over the HUD, and the box a line is typed into (`ChatScreen`).
//!
//! Not drawn: the scroll bar, the message trust icon and the chat
//! visibility, width, scale and opacity options, which stay at their
//! defaults.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

use voxel_game::text::{Run, Style};

use crate::gui::GuiAtlas;
use crate::gui::GuiFrame;
use crate::gui::edit_box::{Clipboard, EditBox, edit_key};
use crate::gui::font::Font;

/// `ChatComponent`'s width at the default option, and the lines it keeps.
const WIDTH: i32 = 320;
const MAX_LINES: usize = 100;
/// A line's height, and the room under the chat for the hotbar.
const LINE_HEIGHT: i32 = 9;
const BOTTOM_MARGIN: i32 = 40;
/// The lines a closed and an open chat show (`getHeight` / 9).
const LINES_CLOSED: usize = 10;
const LINES_OPEN: usize = 20;
/// `GuiMessageTag.system()`'s indicator colour.
const TAG_COLOUR: u32 = 0x00D0_D0D0;

/// One wrapped line and the tick it was added on.
#[derive(Debug, Clone)]
struct ChatLine {
    runs: Vec<Run>,
    added: u64,
}

/// `StringSplitter.splitLines`: the runs broken into lines of at most `max`
/// units, at the last space that fits or inside a word when there is none.
/// The space a line breaks at goes; a line that follows another starts with
/// a space (`ComponentRenderUtils.wrapComponents`).
pub fn wrap(runs: &[Run], max: i32, font: &Font) -> Vec<Vec<Run>> {
    let chars: Vec<(char, usize)> = runs
        .iter()
        .enumerate()
        .flat_map(|(index, run)| run.text.chars().map(move |c| (c, index)))
        .collect();
    let advance = |(c, index): (char, usize)| {
        let bold = runs[index].style.bold == Some(true);
        font.glyph(c).map_or(0.0, |glyph| glyph.advance(bold))
    };
    let mut lines: Vec<Vec<(char, usize)>> = Vec::new();
    let mut current: Vec<(char, usize)> = Vec::new();
    let mut width = 0.0_f32;
    let mut last_space: Option<usize> = None;
    for &entry in &chars {
        if entry.0 == '\n' {
            lines.push(std::mem::take(&mut current));
            width = 0.0;
            last_space = None;
            continue;
        }
        current.push(entry);
        width += advance(entry);
        if entry.0 == ' ' {
            last_space = Some(current.len() - 1);
        }
        if width > max as f32 && current.len() > 1 {
            let (line, rest) = match last_space {
                Some(space) => {
                    let rest = current.split_off(space + 1);
                    current.pop();
                    (std::mem::take(&mut current), rest)
                }
                None => {
                    let rest = current.split_off(current.len() - 1);
                    (std::mem::take(&mut current), rest)
                }
            };
            lines.push(line);
            current = rest;
            width = current.iter().map(|entry| advance(*entry)).sum();
            last_space = current.iter().rposition(|entry| entry.0 == ' ');
        }
    }
    lines.push(current);
    lines
        .into_iter()
        .enumerate()
        .map(|(number, line)| {
            let mut out: Vec<Run> = Vec::new();
            let indent = (number > 0).then(|| (' ', line.first().map_or(0, |entry| entry.1)));
            for (c, index) in indent.into_iter().chain(line) {
                match out.last_mut() {
                    Some(run) if run.style == runs[index].style => run.text.push(c),
                    _ => out.push(Run {
                        text: c.to_string(),
                        style: runs[index].style.clone(),
                    }),
                }
            }
            out
        })
        .collect()
}

/// `AlphaCalculator.timeBased`: fully shown for 180 ticks, then faded out by
/// the 200th.
fn fade(age: u64) -> f32 {
    let t = ((1.0 - age as f32 / 200.0) * 10.0).clamp(0.0, 1.0);
    t * t
}

/// The lines the player has been told, newest first.
#[derive(Debug, Default)]
pub struct ChatLog {
    lines: VecDeque<ChatLine>,
    ticks: u64,
    scroll: usize,
}

impl ChatLog {
    pub fn tick(&mut self) {
        self.ticks += 1;
    }

    /// `ChatComponent.addMessage`: the message wrapped and added, newest
    /// first. A chat that is scrolled back keeps what it shows.
    pub fn add(&mut self, runs: &[Run], font: &Font, focused: bool) {
        for line in wrap(runs, WIDTH, font) {
            if focused && self.scroll > 0 {
                self.scroll += 1;
            }
            self.lines.push_front(ChatLine {
                runs: line,
                added: self.ticks,
            });
        }
        self.lines.truncate(MAX_LINES);
    }

    /// `scrollChat`, in lines.
    pub fn scroll_by(&mut self, amount: i32) {
        let limit = self.lines.len().saturating_sub(LINES_OPEN);
        self.scroll = (self.scroll as i32 + amount).clamp(0, limit as i32) as usize;
    }

    pub fn reset_scroll(&mut self) {
        self.scroll = 0;
    }

    /// `ChatComponent.extractRenderState`: the lines over the hotbar.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        gui_height: i32,
        focused: bool,
    ) {
        let per_page = if focused { LINES_OPEN } else { LINES_CLOSED };
        let shown = self.lines.len().saturating_sub(self.scroll).min(per_page);
        let bottom = gui_height - BOTTOM_MARGIN;
        for i in 0..shown {
            let line = &self.lines[i + self.scroll];
            let alpha = if focused {
                1.0
            } else {
                fade(self.ticks - line.added)
            };
            if alpha <= 1.0e-5 {
                continue;
            }
            let entry_bottom = bottom - i as i32 * LINE_HEIGHT;
            let top = entry_bottom - LINE_HEIGHT;
            let background = ((255.0 * 0.5 * alpha) as u32) << 24;
            frame.fill(atlas, (0, top, WIDTH + 12, entry_bottom), background);
            let tag = ((255.0 * alpha) as u32) << 24 | TAG_COLOUR;
            frame.fill(atlas, (0, top, 2, entry_bottom), tag);
            let text = ((255.0 * alpha) as u32) << 24 | 0x00FF_FFFF;
            frame.styled_text(atlas, font, &line.runs, 4, entry_bottom - 8, text, true);
        }
    }
}

/// What a key did to the chat box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatKey {
    None,
    Close,
    /// The line to send, and whether the screen closes (it always does).
    Send(String),
    /// Scroll the log by this many lines.
    Scroll(i32),
}

/// `ChatScreen`: the box at the bottom of the screen and the history of what
/// was sent.
pub struct ChatScreen {
    edit: EditBox,
    font: Arc<Font>,
    size: (i32, i32),
    history: Vec<String>,
    history_pos: usize,
    history_buffer: String,
    focused_at: Instant,
    /// The key that opened the box also types its character: dropped.
    skip_char: bool,
    /// What the server suggested for the last request, the range of the text
    /// it replaces and the one Tab last took.
    suggestions: Vec<String>,
    range: (usize, usize),
    selected: Option<usize>,
    /// The request to send: its id and text, and the id of the last sent.
    request: Option<(i32, String)>,
    request_id: i32,
    /// A Tab just changed the text: that edit asks for nothing.
    applied: bool,
}

impl ChatScreen {
    /// `ChatScreen.init`: the box holds `initial` (empty, or a slash), and
    /// the character of the key that opened it is to be dropped.
    pub fn new(font: Arc<Font>, size: (i32, i32), initial: &str, history: Vec<String>) -> Self {
        let mut edit = EditBox::new(size.0 - 4, 256);
        edit.set_value(initial, &font);
        let history_pos = history.len();
        Self {
            edit,
            font,
            size,
            history,
            history_pos,
            history_buffer: String::new(),
            focused_at: Instant::now(),
            skip_char: true,
            suggestions: Vec::new(),
            range: (0, 0),
            selected: None,
            request: None,
            request_id: 0,
            applied: false,
        }
    }

    /// A script's chat box: no key opened it.
    pub fn without_key(mut self) -> Self {
        self.skip_char = false;
        self
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        self.size = size;
    }

    /// The request the last edit asked for, taken.
    pub fn take_request(&mut self) -> Option<(i32, String)> {
        self.request.take()
    }

    /// `CommandSuggestions.updateCommandInfo`: a command's text asks for its
    /// matches; text that is no command has none.
    fn edited(&mut self) {
        if std::mem::take(&mut self.applied) {
            return;
        }
        self.suggestions.clear();
        self.selected = None;
        let text = self.edit.value();
        if text.starts_with('/') {
            self.request_id += 1;
            self.request = Some((self.request_id, text));
        }
    }

    /// The server's answer, when it is to the last request.
    pub fn suggested(&mut self, id: i32, start: usize, length: usize, matches: Vec<String>) {
        if id == self.request_id {
            self.range = (start, length);
            self.suggestions = matches;
            self.selected = None;
        }
    }

    /// `CommandSuggestions.keyPressed` for Tab: the next match replaces the
    /// range it was offered for.
    fn apply_next(&mut self) {
        if self.suggestions.is_empty() {
            return;
        }
        let next = self
            .selected
            .map_or(0, |index| (index + 1) % self.suggestions.len());
        let chars: Vec<char> = self.edit.value().chars().collect();
        let (start, length) = self.range;
        let start = start.min(chars.len());
        let end = (start + length).min(chars.len());
        let mut text: String = chars[..start].iter().collect();
        text.push_str(&self.suggestions[next]);
        // The text after the range the last Tab replaced is kept.
        text.extend(chars[end..].iter());
        self.selected = Some(next);
        self.range = (start, self.suggestions[next].chars().count());
        self.applied = true;
        let font = self.font.clone();
        self.edit.set_value(&text, &font);
    }

    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// `ChatScreen.keyPressed`.
    pub fn key_pressed(
        &mut self,
        key: glfw::Key,
        (control, shift): (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> ChatKey {
        use glfw::Key;
        match key {
            Key::Escape => return ChatKey::Close,
            Key::Enter | Key::KpEnter => {
                let text = normalize(&self.edit.value());
                if !text.is_empty() {
                    if self.history.last() != Some(&text) {
                        self.history.push(text.clone());
                        if self.history.len() > MAX_LINES {
                            self.history.remove(0);
                        }
                    }
                    return ChatKey::Send(text);
                }
                return ChatKey::Close;
            }
            Key::Tab => self.apply_next(),
            Key::Up => self.move_in_history(-1),
            Key::Down => self.move_in_history(1),
            Key::PageUp => return ChatKey::Scroll(LINES_OPEN as i32 - 1),
            Key::PageDown => return ChatKey::Scroll(-(LINES_OPEN as i32) + 1),
            other => {
                if let Some(edit) = edit_key(other) {
                    let font = self.font.clone();
                    let before = self.edit.value();
                    self.edit
                        .key_pressed(edit, control, shift, &font, clipboard);
                    if self.edit.value() != before {
                        self.edited();
                    }
                }
            }
        }
        ChatKey::None
    }

    /// `charTyped`: a character the chat allows.
    pub fn char_typed(&mut self, character: char) {
        if std::mem::take(&mut self.skip_char) {
            return;
        }
        if character != '\u{a7}' && character >= ' ' && character != '\u{7f}' {
            let font = self.font.clone();
            if self.edit.char_typed(character, &font) {
                self.edited();
            }
        }
    }

    /// `ChatScreen.moveInHistory`: the older or newer line, the line being
    /// typed coming back at the end.
    fn move_in_history(&mut self, direction: i32) {
        let new_pos = (self.history_pos as i32 + direction).clamp(0, self.history.len() as i32);
        let new_pos = new_pos as usize;
        if new_pos == self.history_pos {
            return;
        }
        if new_pos == self.history.len() {
            self.history_pos = new_pos;
            let buffer = self.history_buffer.clone();
            let font = self.font.clone();
            self.edit.set_value(&buffer, &font);
        } else {
            if self.history_pos == self.history.len() {
                self.history_buffer = self.edit.value();
            }
            self.history_pos = new_pos;
            let line = self.history[new_pos].clone();
            let font = self.font.clone();
            self.edit.set_value(&line, &font);
        }
    }

    /// The box over the chat: a dark band, the text in it.
    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas) {
        let (width, height) = self.size;
        frame.fill(atlas, (2, height - 14, width - 2, height - 2), 0x8000_0000);
        self.edit
            .draw(frame, atlas, &self.font, (4, height - 12), self.focused_at);
        // `CommandSuggestions.SuggestionsList`: the matches over the box, at
        // the text they replace.
        let shown = self.suggestions.len().min(10);
        if shown > 0 {
            let typed: String = self.edit.value().chars().take(self.range.0).collect();
            let width = self
                .suggestions
                .iter()
                .map(|text| self.font.width(text))
                .max()
                .unwrap_or(0);
            let x = (4 + self.font.width(&typed) - 1)
                .clamp(0, (width_limit(self.size.0) - width).max(0));
            let top = height - 12 - shown as i32 * 12;
            frame.fill(atlas, (x, top, x + width + 1, height - 12), 0xD000_0000);
            for (row, text) in self.suggestions.iter().take(shown).enumerate() {
                let colour = if self.selected == Some(row) {
                    0xFFFF_FF00
                } else {
                    0xFFAA_AAAA
                };
                let runs = plain_runs(text);
                frame.styled_text(
                    atlas,
                    &self.font,
                    &runs,
                    x + 1,
                    top + row as i32 * 12 + 2,
                    colour,
                    true,
                );
            }
        }
    }
}

/// The right edge suggestions stay inside.
fn width_limit(screen_width: i32) -> i32 {
    screen_width
}

/// `ChatScreen.normalizeChatMessage`: trimmed, runs of blanks made one,
/// cut to 256 characters.
pub fn normalize(text: &str) -> String {
    let joined = text.split_whitespace().collect::<Vec<_>>().join(" ");
    joined.chars().take(256).collect()
}

/// Styled runs of a plain string, for a line the client writes itself.
pub fn plain_runs(text: &str) -> Vec<Run> {
    vec![Run {
        text: text.to_owned(),
        style: Style::default(),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_message_fades_between_180_and_200_ticks() {
        assert_eq!(fade(0), 1.0);
        assert_eq!(fade(180), 1.0);
        assert!((fade(190) - 0.25).abs() < 1.0e-6);
        assert_eq!(fade(200), 0.0);
    }

    #[test]
    fn a_newline_breaks_a_line_and_the_next_starts_with_a_space() {
        let lines = wrap(&plain_runs("ab\ncd"), WIDTH, &Font::default());
        let texts: Vec<String> = lines
            .iter()
            .map(|line| line.iter().map(|run| run.text.as_str()).collect())
            .collect();
        assert_eq!(texts, ["ab", " cd"]);
    }

    #[test]
    fn normalizing_trims_and_joins_blanks() {
        assert_eq!(normalize("  a   b \t c "), "a b c");
        assert_eq!(normalize("   "), "");
    }

    #[test]
    fn history_walks_back_and_returns_the_line_being_typed() {
        let font = Arc::new(Font::default());
        let mut screen =
            ChatScreen::new(font, (427, 240), "", vec!["a".into(), "b".into()]).without_key();
        screen.char_typed('x');
        screen.move_in_history(-1);
        assert_eq!(screen.edit.value(), "b");
        screen.move_in_history(-1);
        assert_eq!(screen.edit.value(), "a");
        screen.move_in_history(1);
        screen.move_in_history(1);
        assert_eq!(screen.edit.value(), "x");
    }
}
