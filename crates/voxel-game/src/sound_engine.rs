//! The sound engine's bookkeeping (`SoundEngine`, `ChannelAccess`): which sounds are playing on
//! which channels, what each channel is asked (pitch, volume, attenuation, place), and how a sound
//! is delayed, ticked, looped, paused, stopped and let go. The channels themselves (what makes the
//! sound) are the caller's, behind [`Channel`].

use std::collections::{BTreeMap, HashMap};

use voxel_core::random::RandomSource;

use crate::sounds::{Sound, SoundHolder, SoundPacket, SoundRegistry, SoundSource};

/// `Library.Pool`: sounds loaded whole share one set of channels, streamed ones another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pool {
    Static,
    Streaming,
}

/// One source of sound the engine drives (`Channel`).
pub trait Channel {
    fn set_pitch(&mut self, pitch: f32);
    fn set_volume(&mut self, volume: f32);
    /// Linear fall-off to silence at this distance, in blocks.
    fn linear_attenuation(&mut self, max_distance: f32);
    fn disable_attenuation(&mut self);
    fn set_looping(&mut self, looping: bool);
    fn set_position(&mut self, position: (f64, f64, f64));
    fn set_relative(&mut self, relative: bool);
    /// Play a sound file loaded whole (`sounds/<name>.ogg` of the assets).
    fn attach_static_buffer(&mut self, path: &str);
    /// Play a sound file as it is read, from the start again when it ends if `looping`.
    fn attach_stream(&mut self, path: &str, looping: bool);
    fn play(&mut self);
    fn pause(&mut self);
    fn unpause(&mut self);
    fn stop(&mut self);
    /// Whether the sound has ended (by itself or by [`Self::stop`]).
    fn stopped(&self) -> bool;
    fn update_stream(&mut self);
    /// Give the channel back for good.
    fn destroy(&mut self);
    /// Which channel this is, for those that count them (a test's recorder).
    fn number(&self) -> Option<usize> {
        None
    }
}

/// The channels there are (`Library`): none once a pool is used up.
pub trait ChannelSource {
    type Channel: Channel;
    fn acquire(&mut self, pool: Pool) -> Option<Self::Channel>;
}

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
            SoundHolder::Registered(id) => voxel_data::sound_event::name(*id)?.to_owned(),
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

impl<S: ChannelSource> SoundEngine<S> {
    pub fn new(registry: SoundRegistry, channels: S) -> Self {
        Self {
            registry,
            volumes: SoundVolumes::default(),
            channels,
            tick_count: 0,
            next_id: 0,
            instances: BTreeMap::new(),
            handles: BTreeMap::new(),
            by_source: Vec::new(),
            gain_by_source: HashMap::new(),
            ticking: Vec::new(),
            queued: Vec::new(),
            delete_time: HashMap::new(),
            queued_ticking: Vec::new(),
        }
    }

    /// Take an instance in, to be played, queued or delayed.
    pub fn add(&mut self, instance: SoundInstance) -> InstanceId {
        let id = InstanceId(self.next_id);
        self.next_id += 1;
        self.instances.insert(id, instance);
        id
    }

    fn gain(&self, source: SoundSource) -> f32 {
        self.gain_by_source.get(&source).copied().unwrap_or(1.0)
    }

    /// `SoundEngine.calculateVolume`.
    fn calculate_volume(&self, volume: f32, source: SoundSource) -> f32 {
        clamp(volume, 0.0, 1.0)
            * clamp(self.volumes.final_volume(source), 0.0, 1.0)
            * self.gain(source)
    }

    fn instance_volume(&mut self, id: InstanceId) -> f32 {
        let Some(instance) = self.instances.get_mut(&id) else {
            return 0.0;
        };
        let (volume, source) = (instance.effective_volume(), instance.source);
        self.calculate_volume(volume, source)
    }

    fn instance_pitch(&mut self, id: InstanceId) -> f32 {
        let pitch = self
            .instances
            .get_mut(&id)
            .map_or(1.0, SoundInstance::effective_pitch);
        clamp(pitch, 0.5, 2.0)
    }

    fn requires_manual_looping(instance: &SoundInstance) -> bool {
        instance.delay > 0
    }

    /// `SoundEngine.play`.
    pub fn play(&mut self, id: InstanceId) -> PlayResult {
        let Some(instance) = self.instances.get_mut(&id) else {
            return PlayResult::NotStarted;
        };
        if !instance.can_play {
            return PlayResult::NotStarted;
        }
        if !instance.resolve(&self.registry) {
            return PlayResult::NotStarted;
        }
        let sound = instance.sound.clone().expect("resolved");
        if sound == Sound::intentionally_empty() {
            return PlayResult::NotStarted;
        }
        if sound == Sound::empty() {
            return PlayResult::NotStarted;
        }
        let instance_volume = instance.effective_volume();
        let attenuation_distance = instance_volume.max(1.0) * sound.attenuation_distance as f32;
        let source = instance.source;
        let volume = self.calculate_volume(instance_volume, source);
        let pitch = self.instance_pitch(id);
        let instance = self.instances.get(&id).expect("added");
        let (attenuation, relative, position) =
            (instance.attenuation, instance.relative, instance.position);
        let mut started_silently = false;
        if volume == 0.0 {
            if !instance.can_start_silent && source != SoundSource::Music {
                return PlayResult::NotStarted;
            }
            started_silently = true;
        }
        let is_looping = instance.looping && !Self::requires_manual_looping(instance);
        let is_streaming = sound.stream;
        let pool = if is_streaming {
            Pool::Streaming
        } else {
            Pool::Static
        };
        let Some(mut channel) = self.channels.acquire(pool) else {
            return PlayResult::NotStarted;
        };
        self.delete_time.insert(id, self.tick_count + MIN_LIFETIME);
        channel.set_pitch(pitch);
        channel.set_volume(volume);
        if attenuation == Attenuation::Linear {
            channel.linear_attenuation(attenuation_distance);
        } else {
            channel.disable_attenuation();
        }
        channel.set_looping(is_looping && !is_streaming);
        channel.set_position(position);
        channel.set_relative(relative);
        if is_streaming {
            channel.attach_stream(&sound.path(), is_looping);
        } else {
            channel.attach_static_buffer(&sound.path());
        }
        channel.play();
        self.handles.insert(
            id,
            Handle {
                channel: Some(channel),
                stopped: false,
            },
        );
        if !self.by_source.contains(&(source, id)) {
            self.by_source.push((source, id));
        }
        if self
            .instances
            .get(&id)
            .is_some_and(SoundInstance::is_ticking)
        {
            self.ticking.push(id);
        }
        if started_silently {
            PlayResult::StartedSilently
        } else {
            PlayResult::Started
        }
    }

    /// `SoundEngine.queueTickingSound`.
    pub fn queue_ticking_sound(&mut self, id: InstanceId) {
        self.queued_ticking.push(id);
    }

    /// `SoundEngine.playDelayed`.
    pub fn play_delayed(&mut self, id: InstanceId, delay: i32) {
        self.queue(id, self.tick_count + delay);
    }

    /// One entry of `queuedSounds`: a sound has one time to be played at.
    fn queue(&mut self, id: InstanceId, at: i32) {
        self.queued.retain(|(queued, _)| *queued != id);
        self.queued.push((id, at));
    }

    /// `SoundEngine.stop(SoundInstance)`.
    pub fn stop(&mut self, id: InstanceId) {
        if let Some(channel) = self
            .handles
            .get_mut(&id)
            .and_then(|handle| handle.channel.as_mut())
        {
            channel.stop();
        }
    }

    /// `SoundEngine.stopAll`.
    pub fn stop_all(&mut self) {
        for (_, mut handle) in std::mem::take(&mut self.handles) {
            Self::release(&mut handle);
        }
        self.queued.clear();
        self.ticking.clear();
        self.by_source.clear();
        self.delete_time.clear();
        self.queued_ticking.clear();
        self.gain_by_source.clear();
    }

    /// `ChannelHandle.release`: the channel goes back to its pool.
    fn release(handle: &mut Handle<S::Channel>) {
        handle.stopped = true;
        if let Some(mut channel) = handle.channel.take() {
            channel.destroy();
        }
    }

    /// `SoundEngine.stop(Identifier, SoundSource)`.
    pub fn stop_matching(&mut self, event: Option<&str>, source: Option<SoundSource>) {
        let matching = |instance: &SoundInstance| event.is_none_or(|event| instance.event == event);
        if let Some(source) = source {
            let ids: Vec<InstanceId> = self
                .by_source
                .iter()
                .filter(|(by, _)| *by == source)
                .map(|(_, id)| *id)
                .collect();
            for id in ids {
                if self.instances.get(&id).is_some_and(matching) {
                    self.stop(id);
                }
            }
        } else if event.is_none() {
            self.stop_all();
        } else {
            let ids: Vec<InstanceId> = self.handles.keys().copied().collect();
            for id in ids {
                if self.instances.get(&id).is_some_and(matching) {
                    self.stop(id);
                }
            }
        }
    }

    /// `SoundEngine.refreshCategoryVolume`.
    pub fn refresh_category_volume(&mut self, source: SoundSource) {
        let ids: Vec<InstanceId> = self.handles.keys().copied().collect();
        for id in ids {
            let Some(instance_source) = self.instances.get(&id).map(|instance| instance.source)
            else {
                continue;
            };
            if source == instance_source || source == SoundSource::Master {
                let volume = self.instance_volume(id);
                if let Some(channel) = self.handles.get_mut(&id).and_then(|h| h.channel.as_mut()) {
                    channel.set_volume(volume);
                }
            }
        }
    }

    /// `SoundEngine.updateCategoryVolume`.
    pub fn update_category_volume(&mut self, source: SoundSource, gain: f32) {
        self.gain_by_source.insert(source, clamp(gain, 0.0, 1.0));
        self.refresh_category_volume(source);
    }

    /// `SoundEngine.pauseAllExcept`.
    pub fn pause_all_except(&mut self, ignored: &[SoundSource]) {
        for (id, handle) in &mut self.handles {
            let source = self.instances.get(id).map(|instance| instance.source);
            if source.is_some_and(|source| !ignored.contains(&source))
                && let Some(channel) = handle.channel.as_mut()
            {
                channel.pause();
            }
        }
    }

    /// `SoundEngine.resume`.
    pub fn resume(&mut self) {
        for handle in self.handles.values_mut() {
            if let Some(channel) = handle.channel.as_mut() {
                channel.unpause();
            }
        }
    }

    /// Which channel an instance is on, if it has one that counts them.
    pub fn channel_number(&self, id: InstanceId) -> Option<usize> {
        self.handles.get(&id)?.channel.as_ref()?.number()
    }

    /// `SoundEngine.isActive`.
    pub fn is_active(&self, id: InstanceId) -> bool {
        self.delete_time
            .get(&id)
            .is_some_and(|delete| *delete <= self.tick_count)
            || self.handles.contains_key(&id)
    }

    /// `SoundEngine.tick`'s sound part, then `ChannelAccess.scheduleTick`.
    pub fn tick(&mut self, paused: bool) {
        if paused {
            self.tick_music_when_paused();
        } else {
            self.tick_in_game_sound();
        }
        self.schedule_tick();
    }

    /// `ChannelAccess.scheduleTick`: streams are fed, and channels that have ended are let go.
    fn schedule_tick(&mut self) {
        for handle in self.handles.values_mut() {
            if handle.stopped {
                continue;
            }
            let Some(channel) = handle.channel.as_mut() else {
                continue;
            };
            channel.update_stream();
            if channel.stopped() {
                Self::release(handle);
            }
        }
    }

    fn tick_music_when_paused(&mut self) {
        let finished: Vec<InstanceId> = self
            .handles
            .iter()
            .filter(|(id, handle)| {
                handle.stopped
                    && self
                        .instances
                        .get(id)
                        .is_some_and(|instance| instance.source == SoundSource::Music)
            })
            .map(|(id, _)| *id)
            .collect();
        for id in finished {
            self.forget(id);
        }
    }

    /// An instance's channel is gone: it is forgotten (but for what looping by hand queues).
    fn forget(&mut self, id: InstanceId) {
        self.handles.remove(&id);
        self.delete_time.remove(&id);
        if let Some(source) = self.instances.get(&id).map(|instance| instance.source)
            && let Some(at) = self
                .by_source
                .iter()
                .position(|entry| *entry == (source, id))
        {
            self.by_source.remove(at);
        }
    }

    fn tick_in_game_sound(&mut self) {
        self.tick_count += 1;
        for id in std::mem::take(&mut self.queued_ticking) {
            if self
                .instances
                .get(&id)
                .is_some_and(|instance| instance.can_play)
            {
                self.play(id);
            }
        }
        for id in self.ticking.clone() {
            if !self
                .instances
                .get(&id)
                .is_some_and(|instance| instance.can_play)
            {
                self.stop(id);
            }
            if let Some(instance) = self.instances.get_mut(&id) {
                instance.tick();
            }
            if self
                .instances
                .get(&id)
                .is_none_or(|instance| instance.stopped)
            {
                self.stop(id);
            } else {
                let volume = self.instance_volume(id);
                let pitch = self.instance_pitch(id);
                let position = self.instances.get(&id).map(|instance| instance.position);
                if let (Some(position), Some(channel)) = (
                    position,
                    self.handles
                        .get_mut(&id)
                        .and_then(|handle| handle.channel.as_mut()),
                ) {
                    channel.set_volume(volume);
                    channel.set_pitch(pitch);
                    channel.set_position(position);
                }
            }
        }
        let released: Vec<InstanceId> = self
            .handles
            .iter()
            .filter(|(id, handle)| {
                handle.stopped
                    && self
                        .delete_time
                        .get(id)
                        .is_none_or(|delete| *delete <= self.tick_count)
            })
            .map(|(id, _)| *id)
            .collect();
        for id in released {
            if let Some(instance) = self.instances.get(&id)
                && instance.looping
                && Self::requires_manual_looping(instance)
            {
                let (delay, tick_count) = (instance.delay, self.tick_count);
                self.queue(id, tick_count + delay);
            }
            self.forget(id);
            self.ticking.retain(|ticking| *ticking != id);
        }
        let due: Vec<InstanceId> = self
            .queued
            .iter()
            .filter(|(_, at)| self.tick_count >= *at)
            .map(|(id, _)| *id)
            .collect();
        self.queued.retain(|(_, at)| self.tick_count < *at);
        for id in due {
            if let Some(instance) = self.instances.get_mut(&id) {
                instance.tick();
            }
            self.play(id);
        }
    }
}

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
            voxel_data::sound_event::name(0)
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
