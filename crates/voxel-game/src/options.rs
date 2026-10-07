//! `Options` and `options.txt`, for the values the client reads so far: the
//! field of view, the GUI scale, the mouse and the chat. Each keeps the
//! reference's default and range; the file is `key:value` lines in the
//! reference's order, and a key this does not know is kept as it was.

use std::io;
use std::path::Path;

use crate::option::{ClampingLazyMaxIntRange, IntRange, UnitDouble};
use crate::sounds::SoundSource;

/// The data version `options.txt` is written for (`version:` in the file),
/// the pinned client's `world_version`.
const DATA_VERSION: i32 = 4903;

/// `ChatVisiblity`: which chat the client shows, and tells the server it
/// wants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

impl ChatVisibility {
    pub const fn id(self) -> i32 {
        match self {
            Self::Full => 0,
            Self::System => 1,
            Self::Hidden => 2,
        }
    }

    pub const fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Self::Full),
            1 => Some(Self::System),
            2 => Some(Self::Hidden),
            _ => None,
        }
    }
}

/// `InactivityFpsLimit`: when the frame rate is lowered for a window nobody is using.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InactivityFpsLimit {
    Minimized,
    Afk,
}

impl InactivityFpsLimit {
    pub const ALL: [Self; 2] = [Self::Minimized, Self::Afk];

    pub const fn serialized(self) -> &'static str {
        match self {
            Self::Minimized => "minimized",
            Self::Afk => "afk",
        }
    }

    pub fn from_serialized(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|value| value.serialized() == name)
    }

    /// The caption's translation key, which is also the cycle button's value.
    pub const fn caption_key(self) -> &'static str {
        match self {
            Self::Minimized => "options.inactivityFpsLimit.minimized",
            Self::Afk => "options.inactivityFpsLimit.afk",
        }
    }

    pub fn from_caption_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|value| value.caption_key() == key)
    }
}

/// `framerateLimit`'s default, and the value from which the limit is off.
pub const FRAMERATE_LIMIT_DEFAULT: i32 = 120;
pub const UNLIMITED_FRAMERATE_CUTOFF: i32 = 260;

/// `AttackIndicatorStatus`: where the attack strength is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackIndicator {
    Off,
    Crosshair,
    Hotbar,
}

impl AttackIndicator {
    pub const ALL: [Self; 3] = [Self::Off, Self::Crosshair, Self::Hotbar];

    pub const fn id(self) -> i32 {
        match self {
            Self::Off => 0,
            Self::Crosshair => 1,
            Self::Hotbar => 2,
        }
    }

    /// The caption's translation key, which is also the cycle button's value.
    pub const fn caption_key(self) -> &'static str {
        match self {
            Self::Off => "options.off",
            Self::Crosshair => "options.attack.crosshair",
            Self::Hotbar => "options.attack.hotbar",
        }
    }

    pub fn from_caption_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|value| value.caption_key() == key)
    }
}

/// `MusicManager.MusicFrequency`: how long the music waits between songs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicFrequency {
    Default,
    Frequent,
    Constant,
}

impl MusicFrequency {
    pub const ALL: [Self; 3] = [Self::Default, Self::Frequent, Self::Constant];

    /// The codec's name, which is what the file holds.
    pub const fn serialized(self) -> &'static str {
        match self {
            Self::Default => "DEFAULT",
            Self::Frequent => "FREQUENT",
            Self::Constant => "CONSTANT",
        }
    }

    pub fn from_serialized(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|value| value.serialized() == name)
    }

    /// The caption's translation key.
    pub const fn caption_key(self) -> &'static str {
        match self {
            Self::Default => "options.music_frequency.default",
            Self::Frequent => "options.music_frequency.frequent",
            Self::Constant => "options.music_frequency.constant",
        }
    }
}

/// `MusicToastDisplayState`: where the playing song is announced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicToast {
    Never,
    Pause,
    PauseAndToast,
}

impl MusicToast {
    pub const ALL: [Self; 3] = [Self::Never, Self::Pause, Self::PauseAndToast];

    pub const fn serialized(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::Pause => "pause",
            Self::PauseAndToast => "pause_and_toast",
        }
    }

    pub fn from_serialized(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|value| value.serialized() == name)
    }

    /// The label's translation key; the tooltip's is this and `.tooltip`.
    pub const fn text_key(self) -> &'static str {
        match self {
            Self::Never => "options.musicToast.never",
            Self::Pause => "options.musicToast.pauseMenu",
            Self::PauseAndToast => "options.musicToast.pauseMenuAndToast",
        }
    }
}

/// `fov`'s range, in degrees.
const FOV: IntRange = IntRange::new(30, 110);
const FOV_DEFAULT: i32 = 70;
/// `ClampingLazyMaxIntRange`'s encodable maximum for `guiScale`, which is
/// also the largest the option takes before a window exists.
const GUI_SCALE_MAX: i32 = 2_147_483_646;
/// `ChatComponent.getHeight(1.0) - 20`, the span the unfocused height's
/// default (70 pixels) is a share of.
const CHAT_HEIGHT_SPAN: f64 = 160.0;

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
    /// `lang`.
    pub language_code: String,
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
    lines: Vec<(String, String)>,
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
            language_code: "en_us".to_owned(),
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

/// An option of the file, in the order `processOptions` goes through them.
#[derive(Debug, Clone, Copy)]
enum Key {
    EnableVsync,
    Fov,
    GuiScale,
    MaxFps,
    InactivityFpsLimit,
    InvertXMouse,
    InvertYMouse,
    MouseSensitivity,
    Lang,
    ChatVisibility,
    ChatOpacity,
    ChatLineSpacing,
    TextBackgroundOpacity,
    ChatHeightFocused,
    ChatHeightUnfocused,
    ChatScale,
    ChatWidth,
    MouseWheelSensitivity,
    SaveChatDrafts,
    AttackIndicator,
    HideLightningFlashes,
    SoundDevice,
    ShowSubtitles,
    DirectionalAudio,
    MusicToast,
    MusicFrequency,
    SoundCategory(SoundSource),
}

const KEYS: [(&str, Key); 37] = [
    ("enableVsync", Key::EnableVsync),
    ("fov", Key::Fov),
    ("guiScale", Key::GuiScale),
    ("maxFps", Key::MaxFps),
    ("inactivityFpsLimit", Key::InactivityFpsLimit),
    ("invertXMouse", Key::InvertXMouse),
    ("invertYMouse", Key::InvertYMouse),
    ("mouseSensitivity", Key::MouseSensitivity),
    ("lang", Key::Lang),
    ("chatVisibility", Key::ChatVisibility),
    ("chatOpacity", Key::ChatOpacity),
    ("chatLineSpacing", Key::ChatLineSpacing),
    ("textBackgroundOpacity", Key::TextBackgroundOpacity),
    ("chatHeightFocused", Key::ChatHeightFocused),
    ("chatHeightUnfocused", Key::ChatHeightUnfocused),
    ("chatScale", Key::ChatScale),
    ("chatWidth", Key::ChatWidth),
    ("mouseWheelSensitivity", Key::MouseWheelSensitivity),
    ("saveChatDrafts", Key::SaveChatDrafts),
    ("attackIndicator", Key::AttackIndicator),
    ("hideLightningFlashes", Key::HideLightningFlashes),
    ("soundDevice", Key::SoundDevice),
    ("showSubtitles", Key::ShowSubtitles),
    ("directionalAudio", Key::DirectionalAudio),
    ("musicToast", Key::MusicToast),
    ("musicFrequency", Key::MusicFrequency),
    (
        "soundCategory_master",
        Key::SoundCategory(SoundSource::Master),
    ),
    (
        "soundCategory_music",
        Key::SoundCategory(SoundSource::Music),
    ),
    (
        "soundCategory_record",
        Key::SoundCategory(SoundSource::Records),
    ),
    (
        "soundCategory_weather",
        Key::SoundCategory(SoundSource::Weather),
    ),
    (
        "soundCategory_block",
        Key::SoundCategory(SoundSource::Blocks),
    ),
    (
        "soundCategory_hostile",
        Key::SoundCategory(SoundSource::Hostile),
    ),
    (
        "soundCategory_neutral",
        Key::SoundCategory(SoundSource::Neutral),
    ),
    (
        "soundCategory_player",
        Key::SoundCategory(SoundSource::Players),
    ),
    (
        "soundCategory_ambient",
        Key::SoundCategory(SoundSource::Ambient),
    ),
    (
        "soundCategory_voice",
        Key::SoundCategory(SoundSource::Voice),
    ),
    ("soundCategory_ui", Key::SoundCategory(SoundSource::Ui)),
];

/// A value as the file's JSON holds it (`LenientJsonParser`): a number, a
/// boolean, a string, or anything else, which no option's codec takes.
enum Json<'a> {
    Number(&'a str),
    Bool(bool),
    Text(String),
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loaded(text: &str) -> Options {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let number = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!("voxel-options-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join(format!("{number}.txt"));
        std::fs::write(&path, text).unwrap();
        Options::load(&path)
    }

    #[test]
    fn the_defaults_are_the_references() {
        let options = Options::default();
        assert_eq!((options.fov, options.gui_scale), (70, 0));
        assert_eq!(
            (options.sensitivity, options.mouse_wheel_sensitivity),
            (0.5, 1.0)
        );
        assert_eq!(
            (options.chat_opacity, options.chat_scale, options.chat_width),
            (1.0, 1.0, 1.0)
        );
        assert_eq!(options.chat_height_unfocused, 0.4375);
        assert_eq!(options.text_background_opacity, 0.5);
        assert_eq!(options.language_code, "en_us");
    }

    #[test]
    fn the_field_of_view_is_stored_as_a_share_of_its_range() {
        // `(int)(value * 40.0 + 70.0)`; outside 30 to 110 the default holds.
        assert_eq!(loaded("fov:0.5").fov, 90);
        assert_eq!(loaded("fov:-1.0").fov, 30);
        assert_eq!(loaded("fov:1.0").fov, 110);
        assert_eq!(loaded("fov:2.0").fov, 70);
        assert_eq!(loaded("fov:-2.0").fov, 70);
        assert_eq!(loaded("fov:true").fov, 70);
    }

    #[test]
    fn a_unit_value_outside_its_range_leaves_the_option() {
        assert_eq!(loaded("mouseSensitivity:0.25").sensitivity, 0.25);
        assert_eq!(loaded("mouseSensitivity:1.5").sensitivity, 0.5);
        assert_eq!(loaded("mouseSensitivity:true").sensitivity, 1.0);
        assert_eq!(loaded("chatOpacity:false").chat_opacity, 0.0);
        assert_eq!(loaded("chatOpacity:nonsense").chat_opacity, 1.0);
    }

    #[test]
    fn the_gui_scale_and_the_visibility_read_as_integers() {
        assert_eq!(loaded("guiScale:3").gui_scale, 3);
        assert_eq!(loaded("guiScale:-1").gui_scale, 0);
        assert_eq!(loaded("guiScale:2.9").gui_scale, 2);
        assert_eq!(
            loaded("chatVisibility:1").chat_visibility,
            ChatVisibility::System
        );
        // The id wraps round.
        assert_eq!(
            loaded("chatVisibility:3").chat_visibility,
            ChatVisibility::Full
        );
        assert_eq!(
            loaded("chatVisibility:-1").chat_visibility,
            ChatVisibility::Hidden
        );
    }

    #[test]
    fn the_wheel_takes_the_step_at_or_below_its_value() {
        assert_eq!(
            loaded("mouseWheelSensitivity:1.0").mouse_wheel_sensitivity,
            1.0
        );
        assert_eq!(
            loaded("mouseWheelSensitivity:10.0").mouse_wheel_sensitivity,
            10.0
        );
        assert_eq!(
            loaded("mouseWheelSensitivity:11.0").mouse_wheel_sensitivity,
            1.0
        );
        let stepped = loaded("mouseWheelSensitivity:0.5").mouse_wheel_sensitivity;
        assert!((stepped - 10.0_f64.powf(-0.31)).abs() < 1e-12);
    }

    #[test]
    fn booleans_and_the_language_read_as_they_are() {
        let options = loaded("invertYMouse:true\ninvertXMouse:1\nlang:tr_tr");
        assert!(options.invert_mouse_y);
        assert!(!options.invert_mouse_x);
        assert_eq!(options.language_code, "tr_tr");
    }

    #[test]
    fn a_double_is_written_as_java_writes_it() {
        assert_eq!(java_double(0.0), "0.0");
        assert_eq!(java_double(1.0), "1.0");
        assert_eq!(java_double(0.4375), "0.4375");
        assert_eq!(java_double(-0.5), "-0.5");
        assert_eq!(java_double(1.0e-4), "1.0E-4");
        assert_eq!(java_double(1.5e-5), "1.5E-5");
        assert_eq!(java_double(12_345_678.0), "1.2345678E7");
        assert_eq!(java_double(0.001), "0.001");
    }

    #[test]
    fn what_is_saved_reads_back_and_what_is_not_known_stays() {
        let directory = std::env::temp_dir().join("voxel-options-roundtrip");
        let path = directory.join("options.txt");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            &path,
            "version:1\nkey_key.jump:key.keyboard.space\nfov:0.5\nbad line\nsoundCategory_master:0.8\n",
        )
        .unwrap();
        let mut options = Options::load(&path);
        assert_eq!(options.fov, 90);
        options.fov = 100;
        options.chat_width = 0.25;
        options.save(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "version:4903");
        assert_eq!(lines[1], "key_key.jump:key.keyboard.space");
        assert_eq!(lines[2], "fov:0.75");
        assert_eq!(lines[3], "soundCategory_master:0.8");
        assert!(lines.contains(&"chatWidth:0.25"));
        assert!(lines.contains(&"guiScale:0"));
        let again = Options::load(&path);
        assert_eq!((again.fov, again.chat_width), (100, 0.25));
    }

    #[test]
    fn the_sound_options_read_and_write_as_the_file_holds_them() {
        let directory = std::env::temp_dir().join("voxel-options-sound");
        let path = directory.join("options.txt");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            &path,
            "soundCategory_music:0.25\nsoundCategory_ui:2.0\nshowSubtitles:true\nmusicFrequency:\"CONSTANT\"\nmusicToast:\"pause_and_toast\"\nsoundDevice:\"Speakers\"\n",
        )
        .unwrap();
        let options = Options::load(&path);
        assert_eq!(options.sound_category_volume(SoundSource::Music), 0.25);
        // Out of range: the option keeps its default.
        assert_eq!(options.sound_category_volume(SoundSource::Ui), 1.0);
        assert!(options.show_subtitles);
        assert_eq!(options.music_frequency, MusicFrequency::Constant);
        assert_eq!(options.music_toast, MusicToast::PauseAndToast);
        assert_eq!(options.sound_device, "Speakers");
        options.save(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("musicFrequency:\"CONSTANT\"\n"));
        assert!(text.contains("soundCategory_music:0.25\n"));
        assert!(text.contains("soundDevice:\"Speakers\"\n"));
    }

    #[test]
    fn the_attack_indicator_reads_by_id_and_wraps() {
        let options = loaded("attackIndicator:2");
        assert_eq!(options.attack_indicator, AttackIndicator::Hotbar);
        assert_eq!(
            loaded("attackIndicator:5").attack_indicator,
            AttackIndicator::Hotbar
        );
        assert_eq!(
            loaded("attackIndicator:3").attack_indicator,
            AttackIndicator::Off
        );
        assert_eq!(
            loaded("attackIndicator:true").attack_indicator,
            AttackIndicator::Crosshair
        );
    }

    #[test]
    fn the_display_options_read_and_write_as_the_game_does() {
        let mut options = loaded("enableVsync:false\nmaxFps:30\ninactivityFpsLimit:\"minimized\"");
        assert!(!options.enable_vsync);
        assert_eq!(options.framerate_limit, 30);
        assert_eq!(options.inactivity_fps_limit, InactivityFpsLimit::Minimized);
        // A limit outside `intRange(10, 260)` and a value that is not a name leave the option.
        let refused = loaded("maxFps:5\ninactivityFpsLimit:\"never\"");
        assert_eq!(refused.framerate_limit, FRAMERATE_LIMIT_DEFAULT);
        assert_eq!(refused.inactivity_fps_limit, InactivityFpsLimit::Afk);
        assert_eq!(loaded("maxFps:260").framerate_limit, 260);
        options.framerate_limit = UNLIMITED_FRAMERATE_CUTOFF;
        let path = std::env::temp_dir().join(format!("voxel-display-{}.txt", std::process::id()));
        options.save(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("enableVsync:false\n"));
        assert!(text.contains("maxFps:260\n"));
        assert!(text.contains("inactivityFpsLimit:\"minimized\"\n"));
    }
}

mod options_sound;

mod mouse;
pub use mouse::*;
