//! The sound engine's bookkeeping (`SoundEngine`, `ChannelAccess`): which sounds are playing on
//! which channels, what each channel is asked (pitch, volume, attenuation, place), and how a sound
//! is delayed, ticked, looped, paused, stopped and let go. The channels themselves (what makes the
//! sound) are the caller's, behind [`Channel`].

use std::collections::{BTreeMap, HashMap};

use photon_audio::library::{Channel, ChannelSource, Pool};
use photon_core::random::RandomSource;

use crate::sounds::{Sound, SoundHolder, SoundPacket, SoundRegistry, SoundSource};

/// `SoundInstance.Attenuation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attenuation {
    None,
    Linear,
}

/// A sound being asked for (`SoundInstance`, `AbstractSoundInstance`).
pub struct SoundInstance {
    pub event: String,
    pub source: SoundSource,
    pub volume: f32,
    pub pitch: f32,
    pub position: (f64, f64, f64),
    pub looping: bool,
    /// Ticks between a looping sound's end and its start again, when it loops by hand.
    pub delay: i32,
    pub attenuation: Attenuation,
    pub relative: bool,
    pub can_start_silent: bool,
    pub can_play: bool,
    random: RandomSource,
    sound: Option<Sound>,
    /// What a sound that ticks does each tick (`TickableSoundInstance.tick`): it may move, change its
    /// volume and pitch, and say it is over by setting `stopped`.
    pub ticker: Option<Ticker>,
    stopped: bool,
}

/// What a ticking sound does each tick.
pub type Ticker = Box<dyn FnMut(&mut TickState)>;

/// What a ticking sound may change of itself.
pub struct TickState<'a> {
    pub position: &'a mut (f64, f64, f64),
    pub volume: &'a mut f32,
    pub pitch: &'a mut f32,
    pub looping: &'a mut bool,
    pub stopped: &'a mut bool,
}

impl SoundInstance {
    pub fn new(event: &str, source: SoundSource, random: RandomSource) -> Self {
        Self {
            event: event.to_owned(),
            source,
            volume: 1.0,
            pitch: 1.0,
            position: (0.0, 0.0, 0.0),
            looping: false,
            delay: 0,
            attenuation: Attenuation::Linear,
            relative: false,
            can_start_silent: false,
            can_play: true,
            random,
            sound: None,
            ticker: None,
            stopped: false,
        }
    }

    /// What `ClientPacketListener.handleSoundEvent` plays (`SimpleSoundInstance`): the sound the
    /// packet names, at its place, with its volume and pitch, picked with its seed. Nothing for a
    /// registry number the game does not have.
    pub fn at(packet: &SoundPacket) -> Option<Self> {
        let event = match &packet.sound {
            SoundHolder::Registered(id) => photon_data::sound_event::name(*id)?.to_owned(),
            SoundHolder::Direct { location, .. } => location.clone(),
        };
        let mut instance = Self::new(&event, packet.source, RandomSource::create(packet.seed));
        instance.volume = packet.volume;
        instance.pitch = packet.pitch;
        instance.position = packet.position;
        Some(instance)
    }

    /// `AbstractSoundInstance.resolve`: the sound this instance is, picked from its event. Returns
    /// whether the registry has the event.
    fn resolve(&mut self, registry: &SoundRegistry) -> bool {
        if self.event == "minecraft:intentionally_empty" {
            self.sound = Some(Sound::intentionally_empty());
            return true;
        }
        match registry.event(&self.event) {
            None => {
                self.sound = Some(Sound::empty());
                false
            }
            Some(event) => {
                self.sound = Some(registry.pick(event, &mut self.random));
                true
            }
        }
    }

    /// `AbstractSoundInstance.getVolume`: this instance's, times its sound's.
    fn effective_volume(&mut self) -> f32 {
        let sound_volume = self
            .sound
            .as_ref()
            .map_or(1.0, |sound| sound.volume.sample());
        self.volume * sound_volume
    }

    /// `AbstractSoundInstance.getPitch`.
    fn effective_pitch(&mut self) -> f32 {
        let sound_pitch = self
            .sound
            .as_ref()
            .map_or(1.0, |sound| sound.pitch.sample());
        self.pitch * sound_pitch
    }

    fn tick(&mut self) {
        let Some(mut ticker) = self.ticker.take() else {
            return;
        };
        ticker(&mut TickState {
            position: &mut self.position,
            volume: &mut self.volume,
            pitch: &mut self.pitch,
            looping: &mut self.looping,
            stopped: &mut self.stopped,
        });
        self.ticker = Some(ticker);
    }

    fn is_ticking(&self) -> bool {
        self.ticker.is_some()
    }
}

/// Which instance, to ask the engine about it afterwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceId(u64);

/// `SoundEngine.PlayResult`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayResult {
    Started,
    StartedSilently,
    NotStarted,
}

/// What the player has set the volume sliders to (`Options.getFinalSoundSourceVolume`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundVolumes([f64; 11]);

impl Default for SoundVolumes {
    fn default() -> Self {
        Self([1.0; 11])
    }
}

impl SoundVolumes {
    pub fn set(&mut self, source: SoundSource, volume: f64) {
        self.0[source.ordinal() as usize] = volume;
    }

    /// What the slider of a category is set to.
    pub fn slider_value(&self, source: SoundSource) -> f64 {
        self.0[source.ordinal() as usize]
    }

    /// `Options.getSoundSourceVolume`.
    fn slider(&self, source: SoundSource) -> f32 {
        self.0[source.ordinal() as usize] as f32
    }

    /// `Options.getFinalSoundSourceVolume`: a category's slider times the master's.
    pub fn final_volume(&self, source: SoundSource) -> f32 {
        if source == SoundSource::Master {
            self.slider(source)
        } else {
            self.slider(source) * self.slider(SoundSource::Master)
        }
    }
}

/// `Mth.clamp(float, float, float)`.
fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// `ChannelAccess.ChannelHandle`: a channel, until it is let go.
struct Handle<C> {
    channel: Option<C>,
    stopped: bool,
}

/// The minimum a sound lives for after it starts, in ticks (`MIN_SOURCE_LIFETIME`).
const MIN_LIFETIME: i32 = 20;

pub struct SoundEngine<S: ChannelSource> {
    registry: SoundRegistry,
    pub volumes: SoundVolumes,
    channels: S,
    tick_count: i32,
    next_id: u64,
    instances: BTreeMap<InstanceId, SoundInstance>,
    /// The channels of the sounds in hand (`instanceToChannel`, with the handles of `ChannelAccess`).
    handles: BTreeMap<InstanceId, Handle<S::Channel>>,
    by_source: Vec<(SoundSource, InstanceId)>,
    gain_by_source: HashMap<SoundSource, f32>,
    ticking: Vec<InstanceId>,
    queued: Vec<(InstanceId, i32)>,
    delete_time: HashMap<InstanceId, i32>,
    queued_ticking: Vec<InstanceId>,
}

mod volume;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sound_packet_makes_a_sound_at_its_place_with_its_volume_pitch_and_seed() {
        let packet = SoundPacket {
            sound: SoundHolder::Registered(0),
            source: SoundSource::Blocks,
            position: (1.5, 2.0, -3.25),
            volume: 0.75,
            pitch: 1.25,
            seed: 99,
        };
        let instance = SoundInstance::at(&packet).expect("a registered event");
        assert_eq!(
            Some(instance.event.as_str()),
            photon_data::sound_event::name(0)
        );
        assert_eq!((instance.volume, instance.pitch), (0.75, 1.25));
        assert_eq!(instance.position, (1.5, 2.0, -3.25));
        let unknown = SoundPacket {
            sound: SoundHolder::Registered(1 << 20),
            ..packet
        };
        assert!(SoundInstance::at(&unknown).is_none());
    }
}
