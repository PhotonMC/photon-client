use super::*;

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
            listening: false,
            heard: Vec::new(),
        }
    }

    /// `addEventListener` and `removeEventListener` for the one listener there is.
    pub fn set_listening(&mut self, listening: bool) {
        self.listening = listening;
    }

    /// What the listener was told since it last asked.
    pub fn take_heard(&mut self) -> Vec<Heard> {
        std::mem::take(&mut self.heard)
    }

    /// Take an instance in, to be played, queued or delayed.
    pub fn add(&mut self, instance: SoundInstance) -> InstanceId {
        let id = InstanceId(self.next_id);
        self.next_id += 1;
        self.instances.insert(id, instance);
        id
    }

    pub(super) fn gain(&self, source: SoundSource) -> f32 {
        self.gain_by_source.get(&source).copied().unwrap_or(1.0)
    }

    /// `SoundEngine.calculateVolume`.
    pub(super) fn calculate_volume(&self, volume: f32, source: SoundSource) -> f32 {
        clamp(volume, 0.0, 1.0)
            * clamp(self.volumes.final_volume(source), 0.0, 1.0)
            * self.gain(source)
    }

    pub(super) fn instance_volume(&mut self, id: InstanceId) -> f32 {
        let Some(instance) = self.instances.get_mut(&id) else {
            return 0.0;
        };
        let (volume, source) = (instance.effective_volume(), instance.source);
        self.calculate_volume(volume, source)
    }

    pub(super) fn instance_pitch(&mut self, id: InstanceId) -> f32 {
        let pitch = self
            .instances
            .get_mut(&id)
            .map_or(1.0, SoundInstance::effective_pitch);
        clamp(pitch, 0.5, 2.0)
    }

    pub(super) fn requires_manual_looping(instance: &SoundInstance) -> bool {
        instance.delay > 0
    }

    /// What `SoundEngine.play` tells the listeners (`onPlaySound`) of a sound about to start.
    fn tell_listeners(&mut self, id: InstanceId, attenuation_distance: f32) {
        let Some(instance) = self.instances.get(&id).filter(|_| self.listening) else {
            return;
        };
        let range = if !instance.relative && instance.attenuation != Attenuation::None {
            attenuation_distance
        } else {
            f32::INFINITY
        };
        let subtitle = self
            .registry
            .event(&instance.event)
            .and_then(|event| event.subtitle.clone());
        let position = instance.position;
        self.heard.push(Heard {
            subtitle,
            position,
            range,
        });
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
        self.tell_listeners(id, attenuation_distance);
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
        let pool = [Pool::Static, Pool::Streaming][usize::from(is_streaming)];
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
    pub(super) fn queue(&mut self, id: InstanceId, at: i32) {
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
    pub(super) fn release(handle: &mut Handle<S::Channel>) {
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
    pub(super) fn schedule_tick(&mut self) {
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

    pub(super) fn tick_music_when_paused(&mut self) {
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
    pub(super) fn forget(&mut self, id: InstanceId) {
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

    pub(super) fn tick_in_game_sound(&mut self) {
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
