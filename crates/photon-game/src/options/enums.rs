/// `ChatVisiblity`: which chat the client shows, and tells the server it
/// wants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

/// `InactivityFpsLimit`: when the frame rate is lowered for a window nobody is using.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InactivityFpsLimit {
    Minimized,
    Afk,
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

/// `TutorialSteps`: where the tutorial is, which the options keep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialStep {
    Movement,
    FindTree,
    PunchTree,
    OpenInventory,
    CraftPlanks,
    None,
}

impl TutorialStep {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Movement => "movement",
            Self::FindTree => "find_tree",
            Self::PunchTree => "punch_tree",
            Self::OpenInventory => "open_inventory",
            Self::CraftPlanks => "craft_planks",
            Self::None => "none",
        }
    }

    /// `TutorialSteps.getByName`: a name nobody knows is the end of the tutorial.
    pub fn by_name(name: &str) -> Self {
        [
            Self::Movement,
            Self::FindTree,
            Self::PunchTree,
            Self::OpenInventory,
            Self::CraftPlanks,
        ]
        .into_iter()
        .find(|step| step.name() == name)
        .unwrap_or(Self::None)
    }
}

/// `MusicManager.MusicFrequency`: how long the music waits between songs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicFrequency {
    Default,
    Frequent,
    Constant,
}

/// `MusicToastDisplayState`: where the playing song is announced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicToast {
    Never,
    Pause,
    PauseAndToast,
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

    /// `renderInPauseScreen`.
    pub const fn render_in_pause_screen(self) -> bool {
        !matches!(self, Self::Never)
    }

    /// `renderToast`.
    pub const fn render_toast(self) -> bool {
        matches!(self, Self::PauseAndToast)
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

/// `HumanoidArm`: the hand the player uses most, which the server is told.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainHand {
    Left,
    Right,
}

impl MainHand {
    pub const ALL: [Self; 2] = [Self::Left, Self::Right];

    /// `HumanoidArm.id`.
    pub const fn id(self) -> i32 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }

    /// The codec's name, which is what the file holds.
    pub const fn serialized(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
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
            Self::Left => "options.mainHand.left",
            Self::Right => "options.mainHand.right",
        }
    }
}

/// `PlayerModelPart.getId`, by bit: the parts of the skin the player shows.
pub const MODEL_PART_IDS: [&str; 7] = [
    "cape",
    "jacket",
    "left_sleeve",
    "right_sleeve",
    "left_pants_leg",
    "right_pants_leg",
    "hat",
];

/// Every `PlayerModelPart` shown: the mask of the default options.
pub const ALL_MODEL_PARTS: i32 = (1 << MODEL_PART_IDS.len()) - 1;
