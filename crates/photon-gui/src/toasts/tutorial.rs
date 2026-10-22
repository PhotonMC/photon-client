//! The tutorial toast (`TutorialToast`): an icon and lines of words, with a bar of progress for
//! the steps that have one.

use super::{Toast, ToastEnv, Visibility};
use crate::FrameRuns;
use photon_game::text::{Component, Run, Style};
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `TutorialToast.Icons`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialIcon {
    MovementKeys,
    Mouse,
    Tree,
    RecipeBook,
    WoodenPlanks,
    SocialInteractions,
    RightClick,
}

impl TutorialIcon {
    fn sprite(self) -> &'static str {
        match self {
            Self::MovementKeys => "toast/movement_keys",
            Self::Mouse => "toast/mouse",
            Self::Tree => "toast/tree",
            Self::RecipeBook => "toast/recipe_book",
            Self::WoodenPlanks => "toast/wooden_planks",
            Self::SocialInteractions => "toast/social_interactions",
            Self::RightClick => "toast/right_click",
        }
    }
}

/// The width the words are split at (`font.split(.., 126)`).
const TEXT_WIDTH: i32 = 126;

/// The colour of the title (`withColor(-11534256)`).
const TITLE_COLOUR: u32 = 0x50_0050;

/// How a tutorial toast is made: its words, whether it has a bar of progress and how long it
/// stays (zero: until it is hidden).
pub struct TutorialWords<'a> {
    pub title: &'a Component,
    pub message: Option<&'a Component>,
    pub progressable: bool,
    pub time_to_display_ms: i32,
}

pub struct TutorialToast {
    icon: TutorialIcon,
    lines: Vec<Vec<Run>>,
    visibility: Visibility,
    last_smoothing_time: i64,
    smoothed_progress: f32,
    progress: f32,
    progressable: bool,
    time_to_display_ms: i32,
}

impl TutorialToast {
    pub fn new(env: &ToastEnv<'_>, icon: TutorialIcon, words: &TutorialWords<'_>) -> Self {
        let title_style = Style {
            colour: Some(TITLE_COLOUR),
            ..Style::default()
        };
        let mut lines = crate::chat::split(
            &env.lang.runs(words.title, &title_style),
            TEXT_WIDTH,
            env.font,
        );
        if let Some(message) = words.message {
            let runs = env.lang.runs(message, &Style::default());
            lines.extend(crate::chat::split(&runs, TEXT_WIDTH, env.font));
        }
        Self {
            icon,
            lines,
            visibility: Visibility::Show,
            last_smoothing_time: 0,
            smoothed_progress: 0.0,
            progress: 0.0,
            progressable: words.progressable,
            time_to_display_ms: words.time_to_display_ms,
        }
    }

    /// `hide`.
    pub fn hide(&mut self) {
        self.visibility = Visibility::Hide;
    }

    /// `updateProgress`.
    pub fn update_progress(&mut self, progress: f32) {
        self.progress = progress;
    }

    pub fn icon(&self) -> TutorialIcon {
        self.icon
    }

    /// What `updateProgress` last set.
    pub fn progress(&self) -> f32 {
        self.progress
    }

    pub fn progressable(&self) -> bool {
        self.progressable
    }

    fn content_height(&self) -> i32 {
        self.lines.len().max(2) as i32 * 11
    }
}

impl Toast for TutorialToast {
    fn wanted_visibility(&self) -> Visibility {
        self.visibility
    }

    fn update(&mut self, _env: &ToastEnv<'_>, fully_visible_for_ms: i64) {
        if self.time_to_display_ms > 0 {
            self.progress = (fully_visible_for_ms as f32 / self.time_to_display_ms as f32).min(1.0);
            self.smoothed_progress = self.progress;
            self.last_smoothing_time = fully_visible_for_ms;
            if fully_visible_for_ms > i64::from(self.time_to_display_ms) {
                self.hide();
            }
        } else if self.progressable {
            let delta = (fully_visible_for_ms - self.last_smoothing_time) as f32 / 100.0;
            self.smoothed_progress = clamped_lerp(delta, self.smoothed_progress, self.progress);
            self.last_smoothing_time = fully_visible_for_ms;
        }
    }

    fn height(&self) -> i32 {
        7 + self.content_height() + 3
    }

    fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, env: &ToastEnv<'_>) {
        let height = self.height();
        frame.sprite(atlas, "toast/tutorial", 0, 0, self.width(env), height);
        frame.sprite(atlas, self.icon.sprite(), 6, 6, 20, 20);
        let text_height = self.lines.len() as i32 * 11;
        let top = 7 + (self.content_height() - text_height) / 2;
        for (index, runs) in self.lines.iter().enumerate() {
            let y = top + index as i32 * 11;
            frame.styled_text(atlas, env.font, runs, 30, y, 0xFF00_0000, false);
        }
        if self.progressable {
            let y = height - 4;
            frame.fill(atlas, (3, y, 157, y + 1), 0xFFFF_FFFF);
            let colour = if self.progress >= self.smoothed_progress {
                0xFF00_5500
            } else {
                0xFF55_0000
            };
            let end = (3.0 + 154.0 * self.smoothed_progress) as i32;
            frame.fill(atlas, (3, y, end, y + 1), colour);
        }
    }

    fn as_tutorial_mut(&mut self) -> Option<&mut TutorialToast> {
        Some(self)
    }

    fn as_tutorial(&self) -> Option<&TutorialToast> {
        Some(self)
    }
}

/// `Mth.clampedLerp`.
fn clamped_lerp(delta: f32, start: f32, end: f32) -> f32 {
    if delta < 0.0 {
        start
    } else if delta > 1.0 {
        end
    } else {
        start + delta * (end - start)
    }
}
