//! The system toast (`SystemToast`): a title and a message in a plain box, kept for a while that
//! its kind sets.

use super::{Toast, ToastEnv, Visibility};
use crate::{FontRuns, FrameRuns};
use photon_game::text::{Component, Run, Style};
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `SystemToast.SystemToastId`: what a notice is about, and how long it stays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemToastId {
    NarratorToggle,
    WorldBackup,
    PackLoadFailure,
    WorldAccessFailure,
    PackCopyFailure,
    FileDropFailure,
    PeriodicNotification,
    LowDiskSpace,
    ChunkLoadFailure,
    ChunkSaveFailure,
    UnsecureServerWarning,
    FriendSystemNotification,
}

impl SystemToastId {
    /// `displayTime`, in milliseconds.
    pub fn display_time(self) -> i64 {
        match self {
            Self::LowDiskSpace | Self::UnsecureServerWarning => 10_000,
            _ => 5_000,
        }
    }
}

/// `SystemToast`.
pub struct SystemToast {
    id: SystemToastId,
    title_lines: Vec<Vec<Run>>,
    message_lines: Vec<Vec<Run>>,
    last_changed: i64,
    changed: bool,
    width: i32,
    force_hide: bool,
    wanted: Visibility,
}

/// `Font.split(text, 200)`.
fn split_to_length(env: &ToastEnv<'_>, text: &Component) -> Vec<Vec<Run>> {
    let runs = env.lang.runs(text, &Style::default());
    crate::chat::split(&runs, 200, env.font)
}

impl SystemToast {
    pub fn new(
        id: SystemToastId,
        env: &ToastEnv<'_>,
        title: &Component,
        message: Option<&Component>,
    ) -> Self {
        let mut toast = Self {
            id,
            title_lines: Vec::new(),
            message_lines: Vec::new(),
            last_changed: 0,
            changed: false,
            width: 0,
            force_hide: false,
            wanted: Visibility::Hide,
        };
        toast.set_text(env, title, message);
        toast
    }

    pub fn id(&self) -> SystemToastId {
        self.id
    }

    /// `forceHide`.
    pub fn force_hide(&mut self) {
        self.force_hide = true;
    }

    /// `reset`: new words, which keep the toast shown from now on.
    pub fn reset(&mut self, env: &ToastEnv<'_>, title: &Component, message: Option<&Component>) {
        self.set_text(env, title, message);
        self.changed = true;
    }

    /// The private `update(title, message)` and `recalculateWidth`.
    fn set_text(&mut self, env: &ToastEnv<'_>, title: &Component, message: Option<&Component>) {
        self.title_lines = split_to_length(env, title);
        self.message_lines = message.map_or_else(Vec::new, |text| split_to_length(env, text));
        let widest = self
            .title_lines
            .iter()
            .chain(&self.message_lines)
            .map(|line| env.font.width_of_runs(line))
            .max()
            .unwrap_or(200);
        self.width = widest.max(160) + 30;
    }

    fn draw_lines(
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        env: &ToastEnv<'_>,
        lines: &[Vec<Run>],
        (first_y, colour): (i32, u32),
    ) {
        for (index, line) in lines.iter().enumerate() {
            frame.styled_text(
                atlas,
                env.font,
                line,
                18,
                first_y + index as i32 * 12,
                colour,
                false,
            );
        }
    }
}

impl Toast for SystemToast {
    fn wanted_visibility(&self) -> Visibility {
        self.wanted
    }

    fn update(&mut self, env: &ToastEnv<'_>, fully_visible_for_ms: i64) {
        if self.changed {
            self.last_changed = fully_visible_for_ms;
            self.changed = false;
        }
        let time_to_display = self.id.display_time() as f64 * env.display_time;
        let since_update = fully_visible_for_ms - self.last_changed;
        self.wanted = if !self.force_hide && (since_update as f64) < time_to_display {
            Visibility::Show
        } else {
            Visibility::Hide
        };
    }

    fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, env: &ToastEnv<'_>) {
        frame.sprite(atlas, "toast/system", 0, 0, self.width, self.height());
        if self.message_lines.is_empty() {
            Self::draw_lines(frame, atlas, env, &self.title_lines, (12, 0xFFFF_FF00));
        } else {
            Self::draw_lines(frame, atlas, env, &self.title_lines, (7, 0xFFFF_FF00));
            let below = 7 + self.title_lines.len() as i32 * 12;
            Self::draw_lines(frame, atlas, env, &self.message_lines, (below, 0xFFFF_FFFF));
        }
    }

    fn width(&self, _env: &ToastEnv<'_>) -> i32 {
        self.width
    }

    fn height(&self) -> i32 {
        let title = (self.title_lines.len() as i32 - 1) * 12;
        let message = (self.message_lines.len().max(1) as i32) * 12;
        20 + title + message
    }

    fn as_system_mut(&mut self) -> Option<&mut SystemToast> {
        Some(self)
    }
}
