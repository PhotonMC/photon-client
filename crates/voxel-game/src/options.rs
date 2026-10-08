//! `Options` and `options.txt`, for the values the client reads so far: the
//! field of view, the GUI scale, the mouse and the chat. Each keeps the
//! reference's default and range; the file is `key:value` lines in the
//! reference's order, and a key this does not know is kept as it was.

pub mod values;

use std::io;
use std::path::Path;

use crate::options::values::{ClampingLazyMaxIntRange, IntRange, UnitDouble};
use crate::sounds::SoundSource;

/// `renderDistance`'s range: `IntRange(2, 32)` on a 64-bit machine with a gigabyte to spare.
pub const RENDER_DISTANCE_MIN: i32 = 2;
pub const RENDER_DISTANCE_MAX: i32 = 32;

/// What the client holds of `Options`, set to the reference's defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    /// `fov`: 30 to 110 degrees.
    pub fov: i32,
    /// `enableVsync`: wait for the display to refresh before showing a frame.
    pub enable_vsync: bool,
    /// `framerateLimit` (`maxFps`): 10 to 260, from 260 on no limit.
    pub framerate_limit: i32,
    pub inactivity_fps_limit: InactivityFpsLimit,
    /// `guiScale`: 0 picks the largest that fits.
    pub gui_scale: i32,
    /// `invertXMouse`.
    pub invert_mouse_x: bool,
    /// `saveChatDrafts`: a chat box closed with Escape keeps its text.
    pub save_chat_drafts: bool,
    /// `invertYMouse`.
    pub invert_mouse_y: bool,
    /// `mouseSensitivity`, 0 to 1.
    pub sensitivity: f64,
    /// `mouseWheelSensitivity`, 0.01 to 10.
    pub mouse_wheel_sensitivity: f64,
    /// `renderDistance`, in chunks.
    pub render_distance: i32,
    /// `lang`.
    pub language_code: String,
    /// `lastServer` (`lastMpIp`): the address the direct connection last had.
    pub last_server: String,
    pub chat_visibility: ChatVisibility,
    pub chat_opacity: f64,
    pub chat_line_spacing: f64,
    pub text_background_opacity: f64,
    pub chat_height_focused: f64,
    pub chat_height_unfocused: f64,
    pub chat_scale: f64,
    pub chat_width: f64,
    pub attack_indicator: AttackIndicator,
    /// `hideLightningFlashes`: the sky does not flash with lightning.
    pub hide_lightning_flashes: bool,
    /// `soundCategory_<name>` for each `SoundSource`, in ordinal order, 0 to 1.
    pub sound_volumes: [f64; 11],
    /// `soundDevice`: "" is the default device.
    pub sound_device: String,
    pub show_subtitles: bool,
    pub directional_audio: bool,
    pub music_frequency: MusicFrequency,
    pub music_toast: MusicToast,
    /// The file's lines as last read, so what is not known is kept.
    pub(super) lines: Vec<(String, String)>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            fov: FOV_DEFAULT,
            enable_vsync: true,
            framerate_limit: FRAMERATE_LIMIT_DEFAULT,
            inactivity_fps_limit: InactivityFpsLimit::Afk,
            gui_scale: 0,
            invert_mouse_x: false,
            save_chat_drafts: false,
            invert_mouse_y: false,
            sensitivity: 0.5,
            mouse_wheel_sensitivity: log_mouse(0),
            render_distance: 12,
            language_code: "en_us".to_owned(),
            last_server: String::new(),
            chat_visibility: ChatVisibility::Full,
            chat_opacity: 1.0,
            chat_line_spacing: 0.0,
            text_background_opacity: 0.5,
            chat_height_focused: 1.0,
            // `ChatComponent.defaultUnfocusedPct`.
            chat_height_unfocused: 70.0 / CHAT_HEIGHT_SPAN,
            chat_scale: 1.0,
            chat_width: 1.0,
            attack_indicator: AttackIndicator::Crosshair,
            hide_lightning_flashes: false,
            sound_volumes: [1.0; 11],
            sound_device: String::new(),
            show_subtitles: false,
            directional_audio: false,
            music_frequency: MusicFrequency::Default,
            music_toast: MusicToast::Never,
            lines: Vec::new(),
        }
    }
}

mod enums;
pub use enums::*;
mod file;
mod json;
pub use json::*;
mod keys;
use keys::*;
mod sliders;
pub use sliders::*;
