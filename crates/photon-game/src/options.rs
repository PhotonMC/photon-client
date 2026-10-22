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
    /// `ao`: smooth lighting of block faces.
    pub ambient_occlusion: bool,
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
    /// `chatColors`: formatting codes of chat text are applied; the server is told.
    pub chat_colors: bool,
    /// `allowServerListing`: the server may list the player; the server is told.
    pub allow_server_listing: bool,
    /// `particles`.
    pub particles: ParticleStatus,
    /// `invertYMouse`.
    pub invert_mouse_y: bool,
    /// `mouseSensitivity`, 0 to 1.
    pub sensitivity: f64,
    /// `mouseWheelSensitivity`, 0.01 to 10.
    pub mouse_wheel_sensitivity: f64,
    /// `renderDistance`, in chunks.
    pub render_distance: i32,
    /// `menuBackgroundBlurriness`, from 0 (none) to 10.
    pub menu_background_blurriness: i32,
    /// `lang`.
    pub language_code: String,
    /// `lastServer` (`lastMpIp`): the address the direct connection last had.
    pub last_server: String,
    /// `Options.joinedFirstServer`: the social interactions toast has been shown.
    pub joined_first_server: bool,
    /// `Options.advancedItemTooltips`: F3+H.
    pub advanced_item_tooltips: bool,
    /// `Options.pauseOnLostFocus`: F3+P.
    pub pause_on_lost_focus: bool,
    pub chat_visibility: ChatVisibility,
    pub chat_opacity: f64,
    /// `gamma`, 0 to 1: the brightness option, how far dark places are lifted.
    pub gamma: f64,
    /// `screenEffectScale`, 0 to 1: how far nausea and portals distort the view.
    pub screen_effect_scale: f64,
    pub chat_line_spacing: f64,
    pub text_background_opacity: f64,
    /// `backgroundForChatOnly`: the text background opacity applies to the chat alone.
    pub background_for_chat_only: bool,
    /// `notificationDisplayTime`: how long the subtitles and toasts stay, as a multiplier.
    pub notification_display_time: f64,
    /// `chatDelay`: seconds between the chat messages shown.
    pub chat_delay: f64,
    pub chat_height_focused: f64,
    pub chat_height_unfocused: f64,
    pub chat_scale: f64,
    pub chat_width: f64,
    pub attack_indicator: AttackIndicator,
    pub tutorial_step: TutorialStep,
    /// `hideLightningFlashes`: the sky does not flash with lightning.
    pub hide_lightning_flashes: bool,
    /// `soundCategory_<name>` for each `SoundSource`, in ordinal order, 0 to 1.
    pub sound_volumes: [f64; 11],
    /// `soundDevice`: "" is the default device.
    pub sound_device: String,
    pub show_subtitles: bool,
    pub directional_audio: bool,
    /// `bobView`: the view and the hand sway with the walk.
    pub view_bobbing: bool,
    /// `highContrastBlockOutline`: the aimed block's outline is bright with a black backing.
    pub high_contrast_block_outline: bool,
    /// `fovEffectScale`, 0 to 1: how far speed and effects change the field of view.
    pub fov_effect_scale: f64,
    /// `darknessEffectScale`, 0 to 1: how far the darkness effect dims the view.
    pub darkness_effect_scale: f64,
    /// `glintSpeed`, 0 to 1: how fast the enchantment glint moves.
    pub glint_speed: f64,
    /// `glintStrength`, 0 to 1: how strongly the enchantment glint shows.
    pub glint_strength: f64,
    /// `damageTiltStrength`, 0 to 1: how far a hit tilts the view.
    pub damage_tilt_strength: f64,
    pub music_frequency: MusicFrequency,
    pub music_toast: MusicToast,
    /// `modelParts`: the bits (`PlayerModelPart.getMask`) of the parts of the skin shown.
    pub model_parts: i32,
    pub main_hand: MainHand,
    /// The key each mapping is bound to, in `keyMappings` order.
    pub(super) key_bindings: Vec<photon_data::keys::InputKey>,
    /// The file's lines as last read, so what is not known is kept.
    pub(super) lines: Vec<(String, String)>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            fov: FOV_DEFAULT,
            ambient_occlusion: true,
            enable_vsync: true,
            framerate_limit: FRAMERATE_LIMIT_DEFAULT,
            inactivity_fps_limit: InactivityFpsLimit::Afk,
            gui_scale: 0,
            invert_mouse_x: false,
            save_chat_drafts: false,
            chat_colors: true,
            allow_server_listing: true,
            particles: ParticleStatus::All,
            invert_mouse_y: false,
            sensitivity: 0.5,
            mouse_wheel_sensitivity: log_mouse(0),
            render_distance: 12,
            menu_background_blurriness: 5,
            language_code: "en_us".to_owned(),
            last_server: String::new(),
            joined_first_server: false,
            advanced_item_tooltips: false,
            pause_on_lost_focus: true,
            chat_visibility: ChatVisibility::Full,
            chat_opacity: 1.0,
            gamma: 0.5,
            screen_effect_scale: 1.0,
            chat_line_spacing: 0.0,
            text_background_opacity: 0.5,
            background_for_chat_only: true,
            notification_display_time: 1.0,
            chat_delay: 0.0,
            chat_height_focused: 1.0,
            // `ChatComponent.defaultUnfocusedPct`.
            chat_height_unfocused: 70.0 / CHAT_HEIGHT_SPAN,
            chat_scale: 1.0,
            chat_width: 1.0,
            attack_indicator: AttackIndicator::Crosshair,
            tutorial_step: TutorialStep::Movement,
            hide_lightning_flashes: false,
            sound_volumes: [1.0; 11],
            sound_device: String::new(),
            show_subtitles: false,
            directional_audio: false,
            view_bobbing: true,
            high_contrast_block_outline: false,
            fov_effect_scale: 1.0,
            darkness_effect_scale: 1.0,
            glint_speed: 0.5,
            glint_strength: 0.75,
            damage_tilt_strength: 1.0,
            music_frequency: MusicFrequency::Default,
            music_toast: MusicToast::Never,
            model_parts: ALL_MODEL_PARTS,
            main_hand: MainHand::Right,
            key_bindings: bindings::default_bindings(),
            lines: Vec::new(),
        }
    }
}

mod bindings;
pub use bindings::{default_bindings, key_display_name};
mod enums;
pub use enums::*;
mod file;
mod json;
pub use json::*;
mod keys;
pub use keys::DATA_VERSION;
use keys::*;
mod sliders;
pub use sliders::*;
