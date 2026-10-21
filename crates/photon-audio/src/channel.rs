//! The engine's channels over the mixer (`Channel`, `Library`): a channel is a source in the
//! mixer, loaded from the files the assets hold.

use std::sync::{Arc, Mutex};

use crate::library::{Channel, ChannelSource, Pool};

use crate::decode::{LoopingOggStream, OggStream, decode_ogg};
use crate::mixer::{Mixer, Samples, State, StreamFeed};

/// Where the sound files are (`sounds/<name>.ogg` of the assets): their bytes, by the path
/// `namespace:sounds/name.ogg`.
pub trait SoundAssets: Send + Sync {
    fn read(&self, path: &str) -> Option<Vec<u8>>;
}

/// A decoder that keeps ahead of the mixer: what it has decoded, a chunk at a time.
struct Feeder {
    stream: Streams,
    channels: u8,
    rate: u32,
}

enum Streams {
    Once(OggStream),
    Looping(LoopingOggStream),
}

impl StreamFeed for Feeder {
    fn channels(&self) -> u8 {
        self.channels
    }

    fn rate(&self) -> u32 {
        self.rate
    }

    fn pull(&mut self, out: &mut Vec<i16>, max: usize) -> bool {
        let chunk = match &mut self.stream {
            Streams::Once(stream) => stream.read(max * 2),
            Streams::Looping(stream) => stream.read(max * 2),
        };
        match chunk {
            Ok(samples) if !samples.is_empty() => {
                out.extend(samples);
                true
            }
            _ => false,
        }
    }
}

/// One channel of the engine, a source of the mixer.
pub struct MixerChannel {
    mixer: Arc<Mutex<Mixer>>,
    assets: Arc<dyn SoundAssets>,
    id: usize,
    pool: usize,
    used: Arc<Mutex<[usize; 2]>>,
}

impl MixerChannel {
    fn with<T>(&self, action: impl FnOnce(&mut crate::mixer::Source) -> T) -> Option<T> {
        let mut mixer = self.mixer.lock().ok()?;
        mixer.source(self.id).map(action)
    }
}

impl Channel for MixerChannel {
    fn set_pitch(&mut self, pitch: f32) {
        self.with(|source| source.pitch = pitch);
    }

    fn set_volume(&mut self, volume: f32) {
        self.with(|source| source.gain = volume);
    }

    fn linear_attenuation(&mut self, max_distance: f32) {
        self.with(|source| source.max_distance = Some(max_distance));
    }

    fn disable_attenuation(&mut self) {
        self.with(|source| source.max_distance = None);
    }

    fn set_looping(&mut self, looping: bool) {
        self.with(|source| source.looping = looping);
    }

    fn set_position(&mut self, position: (f64, f64, f64)) {
        self.with(|source| source.position = [position.0, position.1, position.2]);
    }

    fn set_relative(&mut self, relative: bool) {
        self.with(|source| source.relative = relative);
    }

    fn attach_static_buffer(&mut self, path: &str) {
        let Some(pcm) = self
            .assets
            .read(path)
            .and_then(|bytes| decode_ogg(&bytes).ok())
        else {
            return;
        };
        self.with(|source| source.attach(Samples::Whole(Arc::new(pcm))));
    }

    fn attach_stream(&mut self, path: &str, looping: bool) {
        let Some(bytes) = self.assets.read(path) else {
            return;
        };
        let feeder = if looping {
            LoopingOggStream::new(bytes).ok().map(|stream| Feeder {
                channels: stream.channels(),
                rate: stream.rate(),
                stream: Streams::Looping(stream),
            })
        } else {
            OggStream::new(bytes).ok().map(|stream| Feeder {
                channels: stream.channels(),
                rate: stream.rate(),
                stream: Streams::Once(stream),
            })
        };
        if let Some(feeder) = feeder {
            self.with(|source| source.attach(Samples::Streamed(Box::new(feeder))));
        }
    }

    fn play(&mut self) {
        self.with(|source| source.play());
    }

    fn pause(&mut self) {
        self.with(|source| {
            if source.state == State::Playing {
                source.state = State::Paused;
            }
        });
    }

    fn unpause(&mut self) {
        self.with(|source| {
            if source.state == State::Paused {
                source.state = State::Playing;
            }
        });
    }

    fn stop(&mut self) {
        self.with(|source| source.state = State::Stopped);
    }

    fn stopped(&self) -> bool {
        self.with(|source| source.state == State::Stopped)
            .unwrap_or(true)
    }

    fn update_stream(&mut self) {}

    fn destroy(&mut self) {
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.remove(self.id);
        }
        if let Ok(mut used) = self.used.lock() {
            used[self.pool] = used[self.pool].saturating_sub(1);
        }
    }
}

/// The channel pools over one mixer (`Library.CountingChannelPool`): so many for sounds loaded whole
/// and so many for streams.
pub struct MixerChannels {
    mixer: Arc<Mutex<Mixer>>,
    assets: Arc<dyn SoundAssets>,
    limits: [usize; 2],
    used: Arc<Mutex<[usize; 2]>>,
}

impl MixerChannels {
    /// The pools of `Library.init` for a device with this many sources to give (`getChannelCount`;
    /// thirty if it cannot say): the streams take the square root, between two and eight.
    pub fn with_total(
        mixer: Arc<Mutex<Mixer>>,
        assets: Arc<dyn SoundAssets>,
        total: usize,
    ) -> Self {
        let streaming = (total as f64).sqrt() as usize;
        let streaming = streaming.clamp(2, 8);
        let static_count = total.saturating_sub(streaming).clamp(8, 255);
        Self {
            mixer,
            assets,
            limits: [static_count, streaming],
            used: Arc::new(Mutex::new([0, 0])),
        }
    }
}

impl MixerChannels {
    /// `Library.getChannelDebugString`: how many of each pool's channels are in use and how many
    /// it has, the sounds loaded whole first, then the streams.
    pub fn usage(&self) -> [(usize, usize); 2] {
        let used = self.used.lock().map_or([0, 0], |used| *used);
        [(used[0], self.limits[0]), (used[1], self.limits[1])]
    }
}

impl ChannelSource for MixerChannels {
    type Channel = MixerChannel;

    fn acquire(&mut self, pool: Pool) -> Option<MixerChannel> {
        let slot = usize::from(pool == Pool::Streaming);
        let mut used = self.used.lock().ok()?;
        if used[slot] >= self.limits[slot] {
            return None;
        }
        let id = self.mixer.lock().ok()?.add();
        used[slot] += 1;
        Some(MixerChannel {
            mixer: Arc::clone(&self.mixer),
            assets: Arc::clone(&self.assets),
            id,
            pool: slot,
            used: Arc::clone(&self.used),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoAssets;

    impl SoundAssets for NoAssets {
        fn read(&self, _: &str) -> Option<Vec<u8>> {
            None
        }
    }

    #[test]
    fn the_pools_are_as_the_library_sizes_them() {
        let mixer = Arc::new(Mutex::new(Mixer::new(48000)));
        let channels = MixerChannels::with_total(Arc::clone(&mixer), Arc::new(NoAssets), 30);
        assert_eq!(channels.limits, [25, 5]);
        let channels = MixerChannels::with_total(mixer, Arc::new(NoAssets), 255);
        assert_eq!(channels.limits, [247, 8]);
    }

    #[test]
    fn a_pool_runs_out_and_a_destroyed_channel_goes_back() {
        let mixer = Arc::new(Mutex::new(Mixer::new(48000)));
        let mut channels = MixerChannels::with_total(mixer, Arc::new(NoAssets), 10);
        // Ten sources: three streaming (the root of ten), seven static, but at least eight static.
        let mut held: Vec<_> = (0..8)
            .map(|_| channels.acquire(Pool::Static).expect("room"))
            .collect();
        assert!(channels.acquire(Pool::Static).is_none());
        held.pop().expect("one").destroy();
        assert!(channels.acquire(Pool::Static).is_some());
    }
}
