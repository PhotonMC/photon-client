//! The sound packets' values: what a packet names, before the sound engine plays it.

/// `SoundSource`: what kind of sound it is, which the player's volume sliders are per.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundSource {
    Master,
    Music,
    Records,
    Weather,
    Blocks,
    Hostile,
    Neutral,
    Players,
    Ambient,
    Voice,
    Ui,
}

impl SoundSource {
    /// In ordinal order, which is what the packets carry.
    pub const ALL: [Self; 11] = [
        Self::Master,
        Self::Music,
        Self::Records,
        Self::Weather,
        Self::Blocks,
        Self::Hostile,
        Self::Neutral,
        Self::Players,
        Self::Ambient,
        Self::Voice,
        Self::Ui,
    ];

    /// The source with this ordinal.
    pub fn from_ordinal(ordinal: i32) -> Option<Self> {
        usize::try_from(ordinal)
            .ok()
            .and_then(|index| Self::ALL.get(index))
            .copied()
    }

    pub fn ordinal(self) -> i32 {
        Self::ALL
            .iter()
            .position(|candidate| *candidate == self)
            .unwrap_or_default() as i32
    }

    /// `SoundSource.getName`: the name the options file and commands use.
    pub fn name(self) -> &'static str {
        match self {
            Self::Master => "master",
            Self::Music => "music",
            Self::Records => "record",
            Self::Weather => "weather",
            Self::Blocks => "block",
            Self::Hostile => "hostile",
            Self::Neutral => "neutral",
            Self::Players => "player",
            Self::Ambient => "ambient",
            Self::Voice => "voice",
            Self::Ui => "ui",
        }
    }
}

/// A sound event as a packet names it (`Holder<SoundEvent>`): by its place in the registry,
/// or sent whole.
#[derive(Debug, Clone, PartialEq)]
pub enum SoundHolder {
    Registered(i32),
    Direct {
        location: String,
        fixed_range: Option<f32>,
    },
}

/// `ClientboundSoundPacket`: a sound at a place.
#[derive(Debug, Clone, PartialEq)]
pub struct SoundPacket {
    pub sound: SoundHolder,
    pub source: SoundSource,
    /// The place, in blocks (the packet carries eighths of a block).
    pub position: (f64, f64, f64),
    pub volume: f32,
    pub pitch: f32,
    /// What the sound is picked from the event with.
    pub seed: i64,
}

/// `ClientboundSoundEntityPacket`: a sound that follows an entity.
#[derive(Debug, Clone, PartialEq)]
pub struct SoundEntityPacket {
    pub sound: SoundHolder,
    pub source: SoundSource,
    pub entity_id: crate::ids::EntityId,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}

/// `ClientboundStopSoundPacket`: stop the sounds of an event, of a source, or both (none: all).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopSoundPacket {
    pub name: Option<String>,
    pub source: Option<SoundSource>,
}

/// What a sound packet asks of the sound engine, in the order they arrived.
#[derive(Debug, Clone, PartialEq)]
pub enum SoundCommand {
    At(SoundPacket),
    FollowingEntity(SoundEntityPacket),
    Stop(StopSoundPacket),
}
