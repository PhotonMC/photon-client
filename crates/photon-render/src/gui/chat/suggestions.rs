//! The list of matches over the chat box (`CommandSuggestions.SuggestionsList`):
//! which one is chosen, how Tab, the arrow keys and the mouse move the choice,
//! and what the list and the ghost text after the box's cursor look like.

use crate::gui::font::Font;
use crate::gui::{GuiAtlas, GuiFrame};

/// `CommandSuggestions` of the chat: lines in the list and a line's height.
const LIMIT: usize = 10;
const LINE: i32 = 12;
/// `ChatScreen`'s fill (`-805306368`), and the colours of a line.
const FILL: u32 = 0xD000_0000;
const CHOSEN: u32 = 0xFFFF_FF00;
const OTHER: u32 = 0xFFAA_AAAA;

/// What a key did to the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListKey {
    Up,
    Down,
    Tab { shift: bool },
    Escape,
}

/// `Suggestions` as the server sent them, sorted, over the text they were
/// asked for.
#[derive(Debug, Clone)]
pub struct SuggestionsList {
    /// The list's corner and size: `Rect2i`.
    rect: (i32, i32, i32, i32),
    /// The box's text when the list was made; a match replaces the range of it.
    original: Vec<char>,
    range: (usize, usize),
    matches: Vec<String>,
    offset: usize,
    current: usize,
    tab_cycles: bool,
    last_mouse: (i32, i32),
}

/// `CommandSuggestions.getLastWordIndex`: where the last word of a text
/// starts, after the last run of whitespace.
fn last_word_index(text: &[char]) -> usize {
    text.iter()
        .rposition(|c| c.is_whitespace())
        .map_or(0, |index| index + 1)
}

/// `CommandSuggestions.sortSuggestions`: those that start with the word being
/// typed, in the order they came, then the rest.
pub fn sort(matches: Vec<String>, typed: &str) -> Vec<String> {
    let chars: Vec<char> = typed.chars().collect();
    let word: String = chars[last_word_index(&chars)..]
        .iter()
        .collect::<String>()
        .to_lowercase();
    let (mut first, rest): (Vec<String>, Vec<String>) = matches.into_iter().partition(|text| {
        text.starts_with(&word) || text.starts_with(&format!("minecraft:{word}"))
    });
    first.extend(rest);
    first
}

impl SuggestionsList {
    /// `showSuggestions`: the matches for the range `start..start + length`
    /// of `original`, the box's unbordered text at `text_x`, the screen
    /// `height` and the `typed` text up to the cursor. `width_of` measures
    /// text. `None` when there are no matches.
    pub fn new(
        matches: Vec<String>,
        (start, length): (usize, usize),
        original: &str,
        typed: &str,
        height: i32,
        font: &Font,
        (text_x, box_width): (i32, i32),
    ) -> Option<Self> {
        if matches.is_empty() {
            return None;
        }
        let matches = sort(matches, typed);
        let widest = matches
            .iter()
            .map(|text| font.width(text))
            .max()
            .unwrap_or(0);
        let chars: Vec<char> = original.chars().collect();
        // `EditBox.getScreenX`.
        let start_x = if start > chars.len() {
            text_x
        } else {
            text_x + font.width(&chars[..start].iter().collect::<String>())
        };
        let x = start_x.clamp(0, (text_x + box_width - widest).max(0));
        let anchor = height - 12;
        let shown = matches.len().min(LIMIT) as i32;
        let list = Self {
            rect: (x - 1, anchor - 3 - shown * LINE, widest + 1, shown * LINE),
            original: chars,
            range: (start, length),
            matches,
            offset: 0,
            current: 0,
            tab_cycles: false,
            last_mouse: (0, 0),
        };
        Some(list)
    }

    /// `Suggestion.apply`: the match in its place in the original text.
    fn applied(&self, index: usize) -> String {
        let (start, length) = self.range;
        let start = start.min(self.original.len());
        let end = (start + length).min(self.original.len());
        let mut text: String = self.original[..start].iter().collect();
        text.push_str(&self.matches[index]);
        text.extend(self.original[end..].iter());
        text
    }

    /// `calculateSuggestionSuffix`: what the chosen match adds to the box's
    /// text, shown after the cursor.
    pub fn ghost(&self, box_text: &str) -> Option<String> {
        self.applied(self.current)
            .strip_prefix(box_text)
            .map(str::to_owned)
    }

    /// `select`: the choice wraps round the list.
    fn select(&mut self, index: i64) {
        let count = self.matches.len() as i64;
        self.current = index.rem_euclid(count) as usize;
    }

    /// `cycle`: the choice moves, and the window of lines follows it.
    fn cycle(&mut self, direction: i64) {
        self.select(self.current as i64 + direction);
        let most = self.matches.len().saturating_sub(LIMIT);
        let last = self.offset + LIMIT - 1;
        if self.current < self.offset {
            self.offset = self.current.min(most);
        } else if self.current > last {
            // `lineStartOffset` is 1 in the chat.
            self.offset = (self.current + 1).saturating_sub(LIMIT).min(most);
        }
    }

    /// `useSuggestion`: the text and the cursor the box takes.
    pub fn use_current(&mut self) -> (String, usize) {
        self.tab_cycles = true;
        let end = self.range.0 + self.matches[self.current].chars().count();
        (self.applied(self.current), end)
    }

    /// `SuggestionsList.keyPressed`. Whether the list took the key, and the
    /// text the box takes when it used a match; `Escape` asks to hide.
    pub fn key(&mut self, key: ListKey) -> (bool, Option<(String, usize)>) {
        match key {
            ListKey::Up => {
                self.cycle(-1);
                self.tab_cycles = false;
                (true, None)
            }
            ListKey::Down => {
                self.cycle(1);
                self.tab_cycles = false;
                (true, None)
            }
            ListKey::Tab { shift } => {
                if self.tab_cycles {
                    self.cycle(if shift { -1 } else { 1 });
                }
                (true, Some(self.use_current()))
            }
            ListKey::Escape => (false, None),
        }
    }

    fn contains(&self, (x, y): (i32, i32)) -> bool {
        let (rx, ry, w, h) = self.rect;
        x >= rx && x < rx + w && y >= ry && y < ry + h
    }

    /// `mouseClicked`: a click on a line chooses it and uses it. `None` when
    /// the click was not in the list.
    pub fn clicked(&mut self, mouse: (i32, i32)) -> Option<Option<(String, usize)>> {
        if !self.contains(mouse) {
            return None;
        }
        let line = (mouse.1 - self.rect.1) / LINE + self.offset as i32;
        if line >= 0 && (line as usize) < self.matches.len() {
            self.select(i64::from(line));
            return Some(Some(self.use_current()));
        }
        Some(None)
    }

    /// `mouseScrolled`: the window of lines moves when the mouse is over it.
    pub fn scrolled(&mut self, scroll: f64, mouse: (i32, i32)) -> bool {
        if !self.contains(mouse) {
            return false;
        }
        let most = self.matches.len().saturating_sub(LIMIT) as f64;
        self.offset = (self.offset as f64 - scroll).clamp(0.0, most) as usize;
        true
    }

    /// `extractRenderState`: the lines, each filled, the chosen one yellow;
    /// dotted edges when more lines lie above or below. A mouse that moved
    /// over a line chooses it.
    pub fn draw(&mut self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (rx, ry, width, height) = self.rect;
        let limit = self.matches.len().min(LIMIT);
        let has_previous = self.offset > 0;
        let has_next = self.matches.len() > self.offset + limit;
        let moved = self.last_mouse != mouse;
        if moved {
            self.last_mouse = mouse;
        }
        if has_previous || has_next {
            frame.fill(atlas, (rx, ry - 1, rx + width, ry), FILL);
            frame.fill(atlas, (rx, ry + height, rx + width, ry + height + 1), FILL);
            for (shown, y) in [(has_previous, ry - 1), (has_next, ry + height)] {
                if shown {
                    for x in (0..width).step_by(2) {
                        frame.fill(atlas, (rx + x, y, rx + x + 1, y + 1), 0xFFFF_FFFF);
                    }
                }
            }
        }
        for row in 0..limit {
            let top = ry + LINE * row as i32;
            frame.fill(atlas, (rx, top, rx + width, top + LINE), FILL);
            if mouse.0 > rx
                && mouse.0 < rx + width
                && mouse.1 > top
                && mouse.1 < top + LINE
                && moved
            {
                self.select((row + self.offset) as i64);
            }
            let colour = if row + self.offset == self.current {
                CHOSEN
            } else {
                OTHER
            };
            let runs = [photon_game::text::Run {
                text: self.matches[row + self.offset].clone(),
                style: photon_game::text::Style::default(),
            }];
            frame.styled_text(atlas, font, &runs, rx + 1, top + 2, colour, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_that_start_with_the_word_come_first() {
        let sorted = sort(
            vec![
                "zgame".into(),
                "gamemode".into(),
                "minecraft:gamerule".into(),
            ],
            "/say ga",
        );
        assert_eq!(sorted, ["gamemode", "minecraft:gamerule", "zgame"]);
        // The first word keeps its slash, so nothing starts with it and the
        // server's order stands.
        let kept = sort(vec!["zgame".into(), "gamemode".into()], "/ga");
        assert_eq!(kept, ["zgame", "gamemode"]);
    }

    #[test]
    fn the_last_word_starts_after_the_last_blank() {
        let text: Vec<char> = "say  hi".chars().collect();
        assert_eq!(last_word_index(&text), 5);
        assert_eq!(last_word_index(&[]), 0);
    }
}
