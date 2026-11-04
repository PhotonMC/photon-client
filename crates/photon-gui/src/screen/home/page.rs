//! A game's page on the home screen: the video area, the facts, the description, a way in and the
//! other games. The words come from operators, so they are drawn as plain text, cut to fit.

use crate::kit::{Painter, State, inside};
use crate::layout::Rect;
use photon_game::text::Lang;

use super::BAR_HEIGHT;

const MARGIN: i32 = 12;
const GAP: i32 = 6;
const OTHERS: usize = 3;

/// What the page shows of one game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameInfo {
    pub id: String,
    pub name: String,
    /// The line under the name on a card: the players and the version.
    pub detail: String,
    pub description: String,
    pub videos: usize,
    /// The first video's address, when the game has one.
    pub video: Option<String>,
    pub images: usize,
}

/// The page's words in the player's language.
#[derive(Debug, Clone)]
pub(super) struct Words {
    play: String,
    back: String,
    others: String,
    video_pending: String,
    no_description: String,
    media: String,
}

impl Words {
    pub(super) fn new(lang: &Lang) -> Self {
        let text = |key: &str| lang.text(&format!("photon.game.{key}"));
        Self {
            play: text("play"),
            back: text("back"),
            others: text("others"),
            video_pending: text("video_pending"),
            no_description: text("no_description"),
            media: text("media"),
        }
    }
}

/// What a press on the page chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Hit {
    Back,
    Play,
    /// Another game, by its place among the page's others.
    Other(usize),
}

pub(super) struct Page {
    pub(super) game: GameInfo,
    /// Up to three other games, as they were when the page opened.
    others: Vec<GameInfo>,
    words: Words,
    media: Rect,
    info: Rect,
    hits: Vec<(Rect, Hit)>,
    focus: Option<usize>,
    /// Whether a video picture is on the surface texture, to be drawn over the media box.
    pub(super) live: bool,
}

impl Page {
    /// The page of `game` in a window `size` units big, with up to three of `all`'s other games.
    pub(super) fn new(size: (i32, i32), game: GameInfo, all: &[GameInfo], words: Words) -> Self {
        let others: Vec<GameInfo> = all
            .iter()
            .filter(|other| other.id != game.id)
            .take(OTHERS)
            .cloned()
            .collect();
        let top = BAR_HEIGHT + 8;
        let media_width = (size.0 - 2 * MARGIN) * 11 / 20;
        let media = Rect {
            x: MARGIN,
            y: top,
            width: media_width,
            height: media_width * 9 / 16,
        };
        let info_x = media.x + media.width + 10;
        let info = Rect {
            x: info_x,
            y: top,
            width: size.0 - MARGIN - info_x,
            height: media.height,
        };
        let hits = hit_rects(size.0, (media, info), others.len());
        Self {
            game,
            others,
            words,
            media,
            info,
            hits,
            focus: None,
            live: false,
        }
    }

    /// The game in `slot` among the page's others.
    pub(super) fn other(&self, slot: usize) -> Option<&GameInfo> {
        self.others.get(slot)
    }

    /// The thing under the mouse.
    pub(super) fn hit_at(&self, mouse: (i32, i32)) -> Option<Hit> {
        self.hits
            .iter()
            .find(|(rect, _)| inside(*rect, mouse))
            .map(|&(_, hit)| hit)
    }

    /// Tab: focus moves to the next thing, or the previous one.
    pub(super) fn step(&mut self, forward: bool) {
        let count = self.hits.len();
        self.focus = Some(match (self.focus, forward) {
            (None, true) => 0,
            (None, false) => count - 1,
            (Some(at), true) => (at + 1) % count,
            (Some(at), false) => (at + count - 1) % count,
        });
    }

    /// Enter: the focused thing, or playing when nothing is focused.
    pub(super) fn pressed(&self) -> Hit {
        self.focus.map_or(Hit::Play, |at| self.hits[at].1)
    }

    pub(super) fn draw(&self, painter: &mut Painter, mouse: (i32, i32)) {
        let skin = *painter.skin;
        self.draw_media(painter);
        let info = self.info;
        let name = painter.fitted(&self.game.name, info.width);
        painter.label(&name, (info.x, info.y), skin.ink);
        let detail = painter.fitted(&self.game.detail, info.width);
        painter.label(&detail, (info.x, info.y + 11), skin.muted);
        let media = self
            .words
            .media
            .replacen("%s", &self.game.videos.to_string(), 1)
            .replacen("%s", &self.game.images.to_string(), 1);
        painter.label(
            &painter.fitted(&media, info.width),
            (info.x, info.y + 22),
            skin.muted,
        );
        let hovered = self.hit_at(mouse);
        for (index, (rect, hit)) in self.hits.iter().enumerate() {
            let state = if self.focus == Some(index) || hovered == Some(*hit) {
                State::Hover
            } else {
                State::Normal
            };
            match hit {
                Hit::Play => painter.button(*rect, &self.words.play, state),
                Hit::Back => painter.button(*rect, &self.words.back, state),
                Hit::Other(slot) => {
                    if let Some(other) = self.others.get(*slot) {
                        painter.card(*rect, (&other.name, &other.detail), state);
                    }
                }
            }
        }
        self.draw_description(painter);
        if !self.others.is_empty() {
            let y = self.media.y + self.media.height + 8;
            painter.label(&self.words.others, (MARGIN, y), skin.ink);
        }
    }

    /// The video's place: until the client plays videos it is a flat box with the game's initial
    /// and, when the game has a video, a line saying so.
    fn draw_media(&self, painter: &mut Painter) {
        let skin = *painter.skin;
        if self.live {
            let m = self.media;
            painter.frame.surface_quad((m.x, m.y, m.width, m.height));
            return;
        }
        painter.fill_rect(self.media, skin.bg | 0xFF00_0000);
        let initial: String = self.game.name.chars().take(1).collect();
        painter.centred(&initial, self.media, skin.ink);
        if self.game.videos > 0 {
            let line = Rect {
                y: self.media.y + self.media.height - 12,
                height: 10,
                ..self.media
            };
            painter.centred(&self.words.video_pending, line, skin.muted);
        }
    }

    fn draw_description(&self, painter: &mut Painter) {
        let muted = painter.skin.muted;
        let text = if self.game.description.trim().is_empty() {
            self.words.no_description.clone()
        } else {
            self.game.description.clone()
        };
        let top = self.info.y + 58;
        let rows = ((self.info.y + self.info.height - top) / 10).max(1) as usize;
        for (row, line) in wrapped(&|t| painter.font.width(t), &text, self.info.width, rows)
            .iter()
            .enumerate()
        {
            painter.label(line, (self.info.x, top + 10 * row as i32), muted);
        }
    }
}

/// Where each thing on the page can be pressed: Play and Back under the facts, then the others.
fn hit_rects(width: i32, (media, info): (Rect, Rect), others: usize) -> Vec<(Rect, Hit)> {
    let button = |x: i32, width: i32| Rect {
        x,
        y: info.y + 36,
        width,
        height: 16,
    };
    let mut hits = vec![
        (button(info.x, 70), Hit::Play),
        (button(info.x + 74, 50), Hit::Back),
    ];
    let cell = (width - 2 * MARGIN - GAP * (OTHERS as i32 - 1)) / OTHERS as i32;
    for slot in 0..others {
        let rect = Rect {
            x: MARGIN + slot as i32 * (cell + GAP),
            y: media.y + media.height + 20,
            width: cell,
            height: 32,
        };
        hits.push((rect, Hit::Other(slot)));
    }
    hits
}

/// `text` as lines no wider than `width`, at most `rows` of them, the last ending in `...` when
/// the text goes on. Control characters count as spaces.
fn wrapped(width_of: &dyn Fn(&str) -> i32, text: &str, width: i32, rows: usize) -> Vec<String> {
    let mut lines = break_words(width_of, text, width);
    if lines.len() > rows {
        lines.truncate(rows);
        if let Some(last) = lines.last_mut() {
            last.push_str("...");
        }
    }
    lines
        .into_iter()
        .map(|line| cut_to(width_of, line, width))
        .collect()
}

/// `text` split at spaces into lines that fit `width`, where a single long word may not.
fn break_words(width_of: &dyn Fn(&str) -> i32, text: &str, width: i32) -> Vec<String> {
    let clean: String = text
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in clean.split_whitespace() {
        let tried = format!("{current} {word}");
        if current.is_empty() || width_of(tried.trim_start()) <= width {
            current = tried.trim_start().to_owned();
        } else {
            lines.push(std::mem::replace(&mut current, word.to_owned()));
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// `line` cut, ending in `...`, when it is still wider than `width`.
fn cut_to(width_of: &dyn Fn(&str) -> i32, mut line: String, width: i32) -> String {
    if width_of(&line) <= width {
        return line;
    }
    while !line.is_empty() && width_of(&format!("{line}...")) > width {
        line.pop();
    }
    format!("{line}...")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(id: &str) -> GameInfo {
        GameInfo {
            id: id.to_owned(),
            name: id.to_owned(),
            detail: String::new(),
            description: String::new(),
            videos: 0,
            video: None,
            images: 0,
        }
    }

    fn page(open: &str, all: &[&str]) -> Page {
        let all: Vec<GameInfo> = all.iter().map(|id| game(id)).collect();
        let current = all.iter().find(|g| g.id == open).cloned().expect("game");
        Page::new((427, 240), current, &all, Words::new(&Lang::default()))
    }

    fn centre(rect: Rect) -> (i32, i32) {
        (rect.x + rect.width / 2, rect.y + rect.height / 2)
    }

    #[test]
    fn the_other_games_leave_out_the_open_one_and_stop_at_three() {
        let page = page("a", &["a", "b", "c", "d", "e"]);
        let ids: Vec<&str> = page.others.iter().map(|g| g.id.as_str()).collect();
        assert_eq!(ids, ["b", "c", "d"]);
        assert_eq!(page.other(0).map(|g| g.id.as_str()), Some("b"));
    }

    #[test]
    fn play_back_and_the_others_are_pressable_and_apart() {
        let page = page("a", &["a", "b"]);
        let rects: Vec<Rect> = page.hits.iter().map(|(rect, _)| *rect).collect();
        assert_eq!(page.hit_at(centre(rects[0])), Some(Hit::Play));
        assert_eq!(page.hit_at(centre(rects[1])), Some(Hit::Back));
        assert_eq!(page.hit_at(centre(rects[2])), Some(Hit::Other(0)));
        assert_eq!(page.hit_at((0, 0)), None);
        for (at, a) in rects.iter().enumerate() {
            for b in &rects[at + 1..] {
                let apart = a.x + a.width <= b.x
                    || b.x + b.width <= a.x
                    || a.y + a.height <= b.y
                    || b.y + b.height <= a.y;
                assert!(apart);
            }
        }
        // Everything is inside the window.
        assert!(
            rects
                .iter()
                .all(|r| r.x + r.width <= 427 && r.y + r.height <= 240)
        );
    }

    #[test]
    fn enter_plays_unless_something_else_is_focused() {
        let mut page = page("a", &["a", "b"]);
        assert_eq!(page.pressed(), Hit::Play);
        page.step(true);
        page.step(true);
        assert_eq!(page.pressed(), Hit::Back);
        page.step(false);
        page.step(false);
        assert_eq!(page.pressed(), Hit::Other(0));
    }

    #[test]
    fn a_description_wraps_to_its_rows_and_marks_the_cut() {
        let font = |text: &str| text.chars().count() as i32 * 6;
        let long = "word ".repeat(200);
        let lines = wrapped(&font, &long, 100, 3);
        assert_eq!(lines.len(), 3);
        assert!(lines[2].ends_with("..."));
        assert!(lines.iter().all(|line| font(line) <= 100));
        assert!(wrapped(&font, "short\ntext", 100, 3).len() == 1);
        assert!(wrapped(&font, "   ", 100, 3).is_empty());
    }
}
