//! The advancement toast (`AdvancementToast`): the sprite, the kind of advancement and its title,
//! with the icon over it.

use super::{Toast, ToastEnv, Visibility};
use crate::FrameRuns;
use photon_game::advancements::{AdvancementType, DisplayInfo};
use photon_game::text::{Component, Run, Style};
use photon_render::gui::{GuiAtlas, GuiFrame, ItemDrawer};

/// How long it stays fully shown, in milliseconds (`DISPLAY_TIME`).
const DISPLAY_TIME: f64 = 5000.0;

/// How long the kind is shown over a title of several lines (`unlockTextTime`), and how long the
/// words take to change over (`unlockFadeTime`).
const UNLOCK_TEXT_MS: i64 = 1500;
const UNLOCK_FADE_MS: f32 = 300.0;

/// `AdvancementToast`, holding what it shows as it was when the toast was made.
pub struct AdvancementToast {
    kind: AdvancementType,
    kind_name: Vec<Run>,
    title_lines: Vec<Vec<Run>>,
    icon: photon_world::ClientItemStackState,
    wanted: Visibility,
    /// How long it has been fully shown, as the last update saw it.
    shown_for: i64,
}

/// Where a line is drawn and in what colour.
struct Line<'a> {
    runs: &'a [Run],
    y: i32,
    colour: u32,
}

impl AdvancementToast {
    pub fn new(display: &DisplayInfo, env: &ToastEnv<'_>) -> Self {
        let key = match display.frame {
            AdvancementType::Task => "advancements.toast.task",
            AdvancementType::Challenge => "advancements.toast.challenge",
            AdvancementType::Goal => "advancements.toast.goal",
        };
        let title = env.lang.runs(&display.title, &Style::default());
        Self {
            kind: display.frame,
            kind_name: env
                .lang
                .runs(&Component::translatable(key), &Style::default()),
            title_lines: crate::chat::split(&title, 125, env.font),
            icon: photon_world::ClientItemStackState::new(1, display.icon.0),
            wanted: Visibility::Hide,
            shown_for: 0,
        }
    }

    fn kind_colour(&self) -> u32 {
        match self.kind {
            AdvancementType::Challenge => 0xFFFF_88FF,
            _ => 0xFFFF_FF00,
        }
    }

    /// `Graphics.text`, which draws nothing in a colour with no alpha.
    fn text(frame: &mut GuiFrame, atlas: &GuiAtlas, env: &ToastEnv<'_>, line: &Line<'_>) {
        if line.colour >> 24 != 0 {
            frame.styled_text(atlas, env.font, line.runs, 30, line.y, line.colour, false);
        }
    }

    /// A title of several lines: the kind fades out, then the words fade in.
    fn draw_lines(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, env: &ToastEnv<'_>) {
        if self.shown_for < UNLOCK_TEXT_MS {
            let fade = ((UNLOCK_TEXT_MS - self.shown_for) as f32 / UNLOCK_FADE_MS).clamp(0.0, 1.0);
            let alpha = (fade * 255.0).floor() as u32;
            let colour = alpha << 24 | self.kind_colour() & 0x00FF_FFFF;
            let line = Line {
                runs: &self.kind_name,
                y: 11,
                colour,
            };
            Self::text(frame, atlas, env, &line);
        } else {
            let fade = ((self.shown_for - UNLOCK_TEXT_MS) as f32 / UNLOCK_FADE_MS).clamp(0.0, 1.0);
            let alpha = (fade * 252.0).floor() as u32;
            let mut y = self.height() / 2 - self.title_lines.len() as i32 * 9 / 2;
            for runs in &self.title_lines {
                let colour = alpha << 24 | 0x00FF_FFFF;
                Self::text(frame, atlas, env, &Line { runs, y, colour });
                y += 9;
            }
        }
    }
}

impl Toast for AdvancementToast {
    fn wanted_visibility(&self) -> Visibility {
        self.wanted
    }

    fn update(&mut self, env: &ToastEnv<'_>, fully_visible_for_ms: i64) {
        self.shown_for = fully_visible_for_ms;
        self.wanted = if fully_visible_for_ms as f64 >= DISPLAY_TIME * env.display_time {
            Visibility::Hide
        } else {
            Visibility::Show
        };
    }

    fn sound_event(&self) -> Option<&'static str> {
        (self.kind == AdvancementType::Challenge).then_some("minecraft:ui.toast.challenge_complete")
    }

    fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, env: &ToastEnv<'_>) {
        frame.sprite(
            atlas,
            "toast/advancement",
            0,
            0,
            self.width(env),
            self.height(),
        );
        if let [runs] = self.title_lines.as_slice() {
            let kind = Line {
                runs: &self.kind_name,
                y: 7,
                colour: self.kind_colour(),
            };
            Self::text(frame, atlas, env, &kind);
            Self::text(
                frame,
                atlas,
                env,
                &Line {
                    runs,
                    y: 18,
                    colour: 0xFFFF_FFFF,
                },
            );
        } else {
            self.draw_lines(frame, atlas, env);
        }
    }

    fn draw_items(&self, frame: &mut GuiFrame, item: &mut ItemDrawer<'_>, origin: (f32, f32)) {
        frame.item_at(item, &self.icon, origin.0 + 8.0, origin.1 + 8.0);
    }
}
