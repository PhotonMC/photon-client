//! The credits screen (`WinScreen`): the end poem and the credits, rolled up the screen over the
//! menu's background, shown when the player wins the game.

use crate::{FontRuns, FrameRuns};
use std::path::Path;

use photon_core::json::{self, JsonValue};
use photon_core::random::RandomSource;
use photon_game::text::{Run, Style};

use crate::chat::split;
use photon_render::gui::font::Font;
use photon_render::gui::{DrawOp, GuiAtlas, GuiFrame};

use super::title::draw_logo;

/// The speed of the roll in screen units a frame, for the credits alone and with the poem.
const CREDITS_SPEED: f32 = 0.75;
const POEM_SPEED: f32 = 0.5;
/// `SPEEDUP_FACTOR` and the extra a held Ctrl adds to it.
const SPEEDUP: f32 = 5.0;
const SPEEDUP_PER_MODIFIER: f32 = 15.0;
/// The poem's lines break at this width; each line of the roll is this tall.
const POEM_WIDTH: i32 = 256;
const LINE_HEIGHT: i32 = 12;
/// The seed that picks how many characters each scrambled word of the poem has.
const POEM_SEED: i64 = 8_124_371;
const KEY_SPACE: i32 = 32;
const KEY_UP: i32 = 265;
const KEY_LEFT_CONTROL: i32 = 341;
const KEY_RIGHT_CONTROL: i32 = 345;
const SECTION: char = '\u{a7}';

/// What the poem marks to be replaced by a scrambled word (`OBFUSCATE_TOKEN`).
fn obfuscate_token() -> String {
    format!("{SECTION}f{SECTION}k{SECTION}a{SECTION}b")
}

/// `StringDecomposer.iterateFormatted` over a plain string: the `\u{a7}` codes become styles.
fn legacy_runs(text: &str) -> Vec<Run> {
    const COLOURS: [&str; 16] = [
        "black",
        "dark_blue",
        "dark_green",
        "dark_aqua",
        "dark_red",
        "dark_purple",
        "gold",
        "gray",
        "dark_gray",
        "blue",
        "green",
        "aqua",
        "red",
        "light_purple",
        "yellow",
        "white",
    ];
    let mut runs: Vec<Run> = Vec::new();
    let mut style = Style::default();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == SECTION {
            // A lone `\u{a7}` at the end is dropped; an unknown code is skipped with its letter.
            let Some(code) = chars.next() else { break };
            let code = code.to_ascii_lowercase();
            if let Some(index) = code.to_digit(16) {
                style = Style {
                    obfuscated: Some(false),
                    bold: Some(false),
                    strikethrough: Some(false),
                    underlined: Some(false),
                    italic: Some(false),
                    ..Style::coloured(COLOURS[index as usize])
                };
            } else {
                match code {
                    'k' => style.obfuscated = Some(true),
                    'l' => style.bold = Some(true),
                    'm' => style.strikethrough = Some(true),
                    'n' => style.underlined = Some(true),
                    'o' => style.italic = Some(true),
                    'r' => style = Style::default(),
                    _ => {}
                }
            }
            continue;
        }
        match runs.last_mut() {
            Some(last) if last.style == style => last.text.push(c),
            _ => runs.push(Run {
                text: c.to_string(),
                style: style.clone(),
            }),
        }
    }
    runs
}

fn plain(text: &str, colour: &str) -> Vec<Run> {
    vec![Run {
        text: text.to_owned(),
        style: Style::coloured(colour),
    }]
}

fn text_of(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::String(units) => String::from_utf16(units).ok(),
        _ => None,
    }
}

pub struct CreditsScreen {
    poem: bool,
    lines: Vec<Vec<Run>>,
    centered: Vec<bool>,
    total_scroll_length: i32,
    scroll: f32,
    speedup_active: bool,
    speedup_modifiers: Vec<i32>,
    scroll_speed: f32,
    unmodified_scroll_speed: f32,
    direction: i32,
    size: (i32, i32),
}

impl CreditsScreen {
    /// `WinScreen` and `init`: the poem (when `poem`), the credits and the post-credits text read
    /// from the assets' `texts` folder, `PLAYERNAME` replaced by the player's name.
    pub fn new(
        font: &Font,
        assets_root: &Path,
        size: (i32, i32),
        poem: bool,
        player: &str,
    ) -> Self {
        let speed = if poem { POEM_SPEED } else { CREDITS_SPEED };
        let mut screen = Self {
            poem,
            lines: Vec::new(),
            centered: Vec::new(),
            total_scroll_length: 0,
            scroll: 0.0,
            speedup_active: false,
            speedup_modifiers: Vec::new(),
            scroll_speed: speed,
            unmodified_scroll_speed: speed,
            direction: 1,
            size,
        };
        let read =
            |name: &str| std::fs::read_to_string(assets_root.join("minecraft/texts").join(name));
        if poem && let Ok(text) = read("end.txt") {
            screen.add_poem_file(font, &text, player);
        }
        if let Ok(text) = read("credits.json") {
            // A document that fails part-way keeps the lines it had added.
            let _ = screen.add_credits_file(&text);
        }
        if poem && let Ok(text) = read("postcredits.txt") {
            screen.add_poem_file(font, &text, player);
        }
        screen.total_scroll_length = screen.lines.len() as i32 * LINE_HEIGHT;
        screen
    }

    fn add_empty_line(&mut self) {
        self.lines.push(Vec::new());
        self.centered.push(false);
    }

    fn add_credits_line(&mut self, runs: Vec<Run>, centered: bool) {
        self.lines.push(runs);
        self.centered.push(centered);
    }

    fn add_poem_file(&mut self, font: &Font, text: &str, player: &str) {
        let mut random = RandomSource::create(POEM_SEED);
        let token = obfuscate_token();
        for line in text.lines() {
            let mut line = line.replace("PLAYERNAME", player);
            while let Some(at) = line.find(&token) {
                let count = random.next_int_bound(4).unwrap_or(0) + 3;
                let scrambled = &"XXXXXXXX"[..count as usize];
                let (before, after) = (&line[..at], &line[at + token.len()..]);
                line = format!("{before}{SECTION}f{SECTION}k{scrambled}{after}");
            }
            // An empty line splits into no lines at all; the empty one follows it.
            if !line.is_empty() {
                for split_line in split(&legacy_runs(&line), POEM_WIDTH, font) {
                    self.add_credits_line(split_line, false);
                }
            }
            self.add_empty_line();
        }
        for _ in 0..8 {
            self.add_empty_line();
        }
    }

    fn add_credits_file(&mut self, text: &str) -> Option<()> {
        let JsonValue::Array(sections) = json::parse(text).ok()? else {
            return None;
        };
        for section in &sections {
            let JsonValue::Object(section) = section else {
                return None;
            };
            let heading = plain("============", "white");
            self.add_credits_line(heading.clone(), true);
            self.add_credits_line(plain(&section.text("section")?, "yellow"), true);
            self.add_credits_line(heading, true);
            self.add_empty_line();
            self.add_empty_line();
            for discipline in section.array("disciplines")? {
                let JsonValue::Object(discipline) = discipline else {
                    return None;
                };
                let name = discipline.text("discipline")?;
                if !name.is_empty() {
                    self.add_credits_line(plain(&name, "yellow"), true);
                    self.add_empty_line();
                    self.add_empty_line();
                }
                for title in discipline.array("titles")? {
                    let JsonValue::Object(title) = title else {
                        return None;
                    };
                    let name = title.text("title")?;
                    let names = title.array("names")?;
                    self.add_credits_line(plain(&name, "gray"), false);
                    for name in names {
                        let name = format!("           {}", text_of(name)?);
                        self.add_credits_line(plain(&name, "white"), false);
                    }
                    self.add_empty_line();
                    self.add_empty_line();
                }
            }
        }
        Some(())
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// How many lines the roll has.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// How far the roll has gone.
    pub fn scroll(&self) -> f32 {
        self.scroll
    }

    /// Put the roll at `scroll`, as a screen that has been shown for a while is.
    pub fn set_scroll(&mut self, scroll: f32) {
        self.scroll = scroll;
    }

    fn calculate_scroll_speed(&self) -> f32 {
        let direction = self.direction as f32;
        if self.speedup_active {
            let factor = SPEEDUP + self.speedup_modifiers.len() as f32 * SPEEDUP_PER_MODIFIER;
            self.unmodified_scroll_speed * factor * direction
        } else {
            self.unmodified_scroll_speed * direction
        }
    }

    /// `keyPressed`: Up runs the roll backwards, Space speeds it up, and each Ctrl speeds it up
    /// more.
    pub fn key_pressed(&mut self, key: i32) {
        if key == KEY_UP {
            self.direction = -1;
        } else if key == KEY_LEFT_CONTROL || key == KEY_RIGHT_CONTROL {
            if !self.speedup_modifiers.contains(&key) {
                self.speedup_modifiers.push(key);
            }
        } else if key == KEY_SPACE {
            self.speedup_active = true;
        }
        self.scroll_speed = self.calculate_scroll_speed();
    }

    /// `keyReleased`.
    pub fn key_released(&mut self, key: i32) {
        if key == KEY_UP {
            self.direction = 1;
        }
        if key == KEY_SPACE {
            self.speedup_active = false;
        } else if key == KEY_LEFT_CONTROL || key == KEY_RIGHT_CONTROL {
            self.speedup_modifiers.retain(|&held| held != key);
        }
        self.scroll_speed = self.calculate_scroll_speed();
    }

    /// `tick`: whether the roll has run past its end, when the game asks to respawn.
    pub fn tick(&self) -> bool {
        let max_scroll = (self.total_scroll_length + self.size.1 + self.size.1 + 24) as f32;
        self.scroll > max_scroll
    }

    /// `extractBackground` and `extractRenderState` of one frame, `a` ticks after the last: the
    /// menu's background (a poem has the end sky instead, which is not drawn), the vignette, then
    /// the logo and the lines at their place in the roll.
    pub fn draw(&mut self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, a: f32) {
        let (width, height) = self.size;
        // The background is drawn before this frame's step, as the reference's order has it.
        if self.poem {
            // The reference fills the screen with the end portal's two sky layers, which are not
            // drawn: the GPU gets black, and the log keeps the reference's white call.
            frame.fill(atlas, (0, 0, width, height), 0xFF00_0000);
            if let Some(DrawOp::Fill { colour, .. }) =
                frame.log.as_mut().and_then(|log| log.last_mut())
            {
                *colour = 0xFFFF_FFFF;
            }
        } else {
            frame.tile_texture_scrolled(
                atlas,
                "menu_background",
                (32, 32),
                self.scroll * 0.5,
                (0, 0, width, height),
            );
        }
        frame.vignette(
            atlas,
            "textures/misc/credits_vignette",
            (256, 256),
            (0, 0, width, height),
        );
        self.scroll = (self.scroll + a * self.scroll_speed).max(0.0);
        let logo_x = width / 2 - 128;
        let logo_y = height + 50;
        let y_offs = -self.scroll;
        let mark = frame.mark();
        draw_logo(frame, atlas, width, logo_y);
        frame.transform_from(mark, [1.0, 0.0, 0.0, 1.0, 0.0, y_offs]);
        let mut y_pos = logo_y + 100;
        let mut shift = y_offs;
        for (index, line) in self.lines.iter().enumerate() {
            if index == self.lines.len() - 1 {
                let diff = y_pos as f32 + y_offs - (height / 2 - 6) as f32;
                if diff < 0.0 {
                    shift += -diff;
                }
            }
            if y_pos as f32 + y_offs + 12.0 + 8.0 > 0.0 && (y_pos as f32 + y_offs) < height as f32 {
                let mark = frame.mark();
                let x = if self.centered[index] {
                    logo_x + 128 - font.width_of_runs(line) / 2
                } else {
                    logo_x
                };
                frame.styled_text(atlas, font, line, x, y_pos, 0xFFFF_FFFF, true);
                frame.transform_from(mark, [1.0, 0.0, 0.0, 1.0, 0.0, shift]);
            }
            y_pos += LINE_HEIGHT;
        }
    }
}
