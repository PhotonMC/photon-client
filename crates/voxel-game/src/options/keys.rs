use super::*;

/// The data version `options.txt` is written for (`version:` in the file),
/// the pinned client's `world_version`.
pub(super) const DATA_VERSION: i32 = 4903;

/// An option of the file, in the order `processOptions` goes through them.
#[derive(Debug, Clone, Copy)]
pub(super) enum Key {
    EnableVsync,
    Fov,
    GuiScale,
    MaxFps,
    InactivityFpsLimit,
    InvertXMouse,
    InvertYMouse,
    MouseSensitivity,
    Lang,
    LastServer,
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

pub(super) const KEYS: [(&str, Key); 38] = [
    ("enableVsync", Key::EnableVsync),
    ("fov", Key::Fov),
    ("guiScale", Key::GuiScale),
    ("maxFps", Key::MaxFps),
    ("inactivityFpsLimit", Key::InactivityFpsLimit),
    ("invertXMouse", Key::InvertXMouse),
    ("invertYMouse", Key::InvertYMouse),
    ("mouseSensitivity", Key::MouseSensitivity),
    ("lang", Key::Lang),
    ("lastServer", Key::LastServer),
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
