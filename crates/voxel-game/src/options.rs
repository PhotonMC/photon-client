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

/// `Options.logMouse`.
pub fn log_mouse(value: i32) -> f64 {
    10.0_f64.powf(f64::from(value) / 100.0)
}

/// `Options.unlogMouse`.
pub fn unlog_mouse(value: f64) -> i32 {
    (value.log10() * 100.0).floor() as i32
}

/// The wheel's value set, `IntRange(-200, 100).xmap(logMouse, unlogMouse)`:
/// a value takes the nearest step at or below it.
fn mouse_wheel_step(value: f64) -> Option<f64> {
    IntRange::new(-200, 100)
        .xmap(log_mouse, |value: &f64| unlog_mouse(*value), false)
        .validate_value(&value)
}

/// What the client holds of `Options`, set to the reference's defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct Options {
    /// `fov`: 30 to 110 degrees.
    pub fov: i32,
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
    Fov,
    GuiScale,
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
    SoundDevice,
    ShowSubtitles,
    DirectionalAudio,
    MusicToast,
    MusicFrequency,
    SoundCategory(SoundSource),
}

const KEYS: [(&str, Key); 32] = [
    ("fov", Key::Fov),
    ("guiScale", Key::GuiScale),
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

fn parse_json(text: &str) -> Json<'_> {
    let text = text.trim();
    match text {
        "true" => Json::Bool(true),
        "false" => Json::Bool(false),
        _ if is_json_number(text) => Json::Number(text),
        _ => json_string(text).map_or(Json::Other, Json::Text),
    }
}

/// A JSON string with the escapes a name can carry.
fn json_string(text: &str) -> Option<String> {
    let inner = text.strip_prefix('"')?.strip_suffix('"')?;
    let mut out = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return None,
            '\\' => match chars.next()? {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                _ => return None,
            },
            _ => out.push(c),
        }
    }
    Some(out)
}

/// A string as the file writes it.
fn write_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The JSON number grammar.
fn is_json_number(text: &str) -> bool {
    let bytes = text.as_bytes();
    let digits = |from: usize| {
        bytes[from..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count()
    };
    let mut at = usize::from(bytes.first() == Some(&b'-'));
    match digits(at) {
        0 => return false,
        1 => {}
        _ if bytes[at] == b'0' => return false,
        _ => {}
    }
    at += digits(at);
    if bytes.get(at) == Some(&b'.') {
        let fraction = digits(at + 1);
        if fraction == 0 {
            return false;
        }
        at += 1 + fraction;
    }
    if matches!(bytes.get(at), Some(b'e' | b'E')) {
        at += 1;
        if matches!(bytes.get(at), Some(b'+' | b'-')) {
            at += 1;
        }
        let exponent = digits(at);
        if exponent == 0 {
            return false;
        }
        at += exponent;
    }
    at == bytes.len()
}

/// `Number.doubleValue` of a parsed number.
fn double_value(number: &str) -> f64 {
    number.parse().unwrap_or(f64::NAN)
}

/// `Number.intValue` of a parsed number (`LazilyParsedNumber`): an integer
/// as it reads, else the decimal cut towards zero.
fn int_value(number: &str) -> i32 {
    if let Ok(value) = number.parse::<i64>() {
        return value as i32;
    }
    double_value(number) as i128 as i32
}

/// `Double.toString`, as `Gson` writes a double.
pub fn java_double(value: f64) -> String {
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0.0"
        } else {
            "0.0"
        }
        .to_owned();
    }
    if (1e-3..1e7).contains(&value.abs()) {
        return format!("{value:?}");
    }
    let scientific = format!("{value:e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let dot = if mantissa.contains('.') { "" } else { ".0" };
    format!("{mantissa}{dot}E{exponent}")
}

impl Options {
    /// `Options.load`: `path` read over the defaults; a missing file leaves
    /// them, a line or a value that does not read is skipped.
    pub fn load(path: &Path) -> Self {
        let mut options = Self::default();
        let Ok(bytes) = std::fs::read(path) else {
            return options;
        };
        for line in String::from_utf8_lossy(&bytes).lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            match options.lines.iter_mut().find(|(known, _)| known == key) {
                Some(entry) => value.clone_into(&mut entry.1),
                None => options.lines.push((key.to_owned(), value.to_owned())),
            }
        }
        for (name, key) in KEYS {
            let value = options
                .lines
                .iter()
                .find(|(known, _)| known == name)
                .map(|(_, value)| value.clone());
            if let Some(value) = value {
                options.read(key, &value);
            }
        }
        options
    }

    /// `read` for the sound options.
    fn read_sound(&mut self, key: Key, json: &Json) {
        match (key, json) {
            (Key::ShowSubtitles, Json::Bool(value)) => self.show_subtitles = *value,
            (Key::DirectionalAudio, Json::Bool(value)) => self.directional_audio = *value,
            (Key::SoundDevice, Json::Text(device)) => self.sound_device.clone_from(device),
            (Key::MusicFrequency, Json::Text(name)) => {
                if let Some(value) = MusicFrequency::from_serialized(name) {
                    self.music_frequency = value;
                }
            }
            (Key::MusicToast, Json::Text(name)) => {
                if let Some(value) = MusicToast::from_serialized(name) {
                    self.music_toast = value;
                }
            }
            (Key::SoundCategory(source), Json::Number(number)) => {
                if let Some(value) = UnitDouble::validate_value(double_value(number)) {
                    self.sound_volumes[source.ordinal() as usize] = value;
                }
            }
            (Key::SoundCategory(source), Json::Bool(value)) => {
                self.sound_volumes[source.ordinal() as usize] = if *value { 1.0 } else { 0.0 };
            }
            _ => {}
        }
    }

    /// `option.codec().parse` and `set`: a value its codec refuses leaves
    /// the option as it was; one its value set refuses falls to the default.
    fn read(&mut self, key: Key, text: &str) {
        let json = parse_json(text);
        let unit = || match json {
            Json::Number(number) => UnitDouble::validate_value(double_value(number)),
            Json::Bool(value) => Some(if value { 1.0 } else { 0.0 }),
            Json::Text(_) | Json::Other => None,
        };
        let boolean = || match json {
            Json::Bool(value) => Some(value),
            _ => None,
        };
        match key {
            Key::Fov => {
                if let Json::Number(number) = json {
                    let degrees = (double_value(number) * 40.0 + 70.0) as i32;
                    self.fov = FOV.validate_value(degrees).unwrap_or(FOV_DEFAULT);
                }
            }
            Key::GuiScale => {
                if let Json::Number(number) = json {
                    let scale = int_value(number);
                    if (0..=GUI_SCALE_MAX + 1).contains(&scale) {
                        self.gui_scale =
                            ClampingLazyMaxIntRange::new(0, || GUI_SCALE_MAX, GUI_SCALE_MAX)
                                .validate_value(scale);
                    }
                }
            }
            Key::InvertXMouse => self.invert_mouse_x = boolean().unwrap_or(self.invert_mouse_x),
            Key::SaveChatDrafts => {
                self.save_chat_drafts = boolean().unwrap_or(self.save_chat_drafts);
            }
            Key::InvertYMouse => self.invert_mouse_y = boolean().unwrap_or(self.invert_mouse_y),
            Key::ShowSubtitles
            | Key::DirectionalAudio
            | Key::SoundDevice
            | Key::MusicFrequency
            | Key::MusicToast
            | Key::SoundCategory(_) => self.read_sound(key, &json),
            Key::MouseSensitivity => self.sensitivity = unit().unwrap_or(self.sensitivity),
            Key::Lang => text.clone_into(&mut self.language_code),
            Key::ChatVisibility => {
                if let Json::Number(number) = json {
                    // `ByIdMap` wraps an id out of range.
                    let id = int_value(number).rem_euclid(3);
                    self.chat_visibility =
                        ChatVisibility::from_id(id).unwrap_or(self.chat_visibility);
                }
            }
            Key::ChatOpacity => self.chat_opacity = unit().unwrap_or(self.chat_opacity),
            Key::ChatLineSpacing => {
                self.chat_line_spacing = unit().unwrap_or(self.chat_line_spacing)
            }
            Key::TextBackgroundOpacity => {
                self.text_background_opacity = unit().unwrap_or(self.text_background_opacity);
            }
            Key::ChatHeightFocused => {
                self.chat_height_focused = unit().unwrap_or(self.chat_height_focused);
            }
            Key::ChatHeightUnfocused => {
                self.chat_height_unfocused = unit().unwrap_or(self.chat_height_unfocused);
            }
            Key::ChatScale => self.chat_scale = unit().unwrap_or(self.chat_scale),
            Key::ChatWidth => self.chat_width = unit().unwrap_or(self.chat_width),
            Key::MouseWheelSensitivity => {
                if let Json::Number(number) = json {
                    let value = double_value(number);
                    if (log_mouse(-200)..=log_mouse(100)).contains(&value) {
                        self.mouse_wheel_sensitivity =
                            mouse_wheel_step(value).unwrap_or(log_mouse(0));
                    }
                }
            }
        }
    }

    /// The value the file holds for an option.
    fn write(&self, key: Key) -> String {
        match key {
            Key::Fov => java_double((f64::from(self.fov) - 70.0) / 40.0),
            Key::GuiScale => self.gui_scale.to_string(),
            Key::InvertXMouse => self.invert_mouse_x.to_string(),
            Key::SaveChatDrafts => self.save_chat_drafts.to_string(),
            Key::InvertYMouse => self.invert_mouse_y.to_string(),
            Key::MouseSensitivity => java_double(self.sensitivity),
            Key::Lang => self.language_code.clone(),
            Key::ChatVisibility => self.chat_visibility.id().to_string(),
            Key::ChatOpacity => java_double(self.chat_opacity),
            Key::ChatLineSpacing => java_double(self.chat_line_spacing),
            Key::TextBackgroundOpacity => java_double(self.text_background_opacity),
            Key::ChatHeightFocused => java_double(self.chat_height_focused),
            Key::ChatHeightUnfocused => java_double(self.chat_height_unfocused),
            Key::ChatScale => java_double(self.chat_scale),
            Key::ChatWidth => java_double(self.chat_width),
            Key::MouseWheelSensitivity => java_double(self.mouse_wheel_sensitivity),
            Key::ShowSubtitles => self.show_subtitles.to_string(),
            Key::DirectionalAudio => self.directional_audio.to_string(),
            Key::SoundDevice => write_string(&self.sound_device),
            Key::MusicFrequency => write_string(self.music_frequency.serialized()),
            Key::MusicToast => write_string(self.music_toast.serialized()),
            Key::SoundCategory(source) => {
                java_double(self.sound_volumes[source.ordinal() as usize])
            }
        }
    }

    /// `Options.getSoundSourceVolume`: what the player set a sound category to.
    pub fn sound_category_volume(&self, source: SoundSource) -> f64 {
        self.sound_volumes[source.ordinal() as usize]
    }

    /// `Options.save`: the version, then the lines as they were read with
    /// the values this holds, then the options the file did not have.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut text = format!("version:{DATA_VERSION}\n");
        let mut written = Vec::new();
        for (name, value) in &self.lines {
            if name == "version" {
                continue;
            }
            let known = KEYS.iter().find(|(known, _)| known == name);
            let value = known.map_or_else(|| value.clone(), |(_, key)| self.write(*key));
            text.push_str(&format!("{name}:{value}\n"));
            written.push(name.as_str());
        }
        for (name, key) in KEYS {
            if !written.contains(&name) {
                text.push_str(&format!("{name}:{}\n", self.write(key)));
            }
        }
        if let Some(directory) = path.parent() {
            std::fs::create_dir_all(directory)?;
        }
        std::fs::write(path, text)
    }
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
}
