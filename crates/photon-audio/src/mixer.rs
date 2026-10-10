//! The mixer: the sounds that play, each at its place, pitch and volume, into stereo frames.
//!
//! The reference hands this to OpenAL Soft; what is written here is the model the reference asks
//! of it (`Channel`, `Listener`): a source with a position, a pitch that scales how fast it plays,
//! a gain, a linear fall-off to silence at a distance (`AL_LINEAR_DISTANCE_CLAMPED`, reference
//! distance 0, roll-off 1), looping, and a position that is relative to the listener or not. How
//! OpenAL Soft pans and filters is its own; this pans by the angle to the sound, constant power.
//! Nothing here claims to sound the same as OpenAL Soft.

use std::sync::Arc;

use crate::decode::Pcm;

/// Where the listener is and which way it faces (`Listener.setTransform`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ListenerTransform {
    pub position: [f64; 3],
    pub forward: [f64; 3],
    pub up: [f64; 3],
}

impl Default for ListenerTransform {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            forward: [0.0, 0.0, -1.0],
            up: [0.0, 1.0, 0.0],
        }
    }
}

/// Where a source's samples come from.
pub enum Samples {
    /// A file decoded whole.
    Whole(Arc<Pcm>),
    /// A file read as it plays: what has been read, a chunk at a time.
    Streamed(Box<dyn StreamFeed + Send>),
}

/// A stream the mixer pulls decoded samples from (`AudioStream.read`).
pub trait StreamFeed {
    fn channels(&self) -> u8;
    fn rate(&self) -> u32;
    /// Move up to `max` more samples (all channels) into `out`; false once the stream has ended
    /// and nothing is left.
    fn pull(&mut self, out: &mut Vec<i16>, max: usize) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Initial,
    Playing,
    Paused,
    Stopped,
}

/// One sound in the mixer (an OpenAL source).
pub struct Source {
    samples: Option<Samples>,
    /// What a streamed source has buffered, from `buffer_start` (in frames).
    window: Vec<i16>,
    buffer_start: u64,
    ended: bool,
    /// The frame being played, with its fraction.
    cursor: f64,
    pub state: State,
    pub pitch: f32,
    pub gain: f32,
    pub looping: bool,
    /// Distance at which the sound is silent, or none for no fall-off.
    pub max_distance: Option<f32>,
    pub relative: bool,
    pub position: [f64; 3],
}

impl Default for Source {
    fn default() -> Self {
        Self {
            samples: None,
            window: Vec::new(),
            buffer_start: 0,
            ended: false,
            cursor: 0.0,
            state: State::Initial,
            pitch: 1.0,
            gain: 1.0,
            looping: false,
            max_distance: None,
            relative: false,
            position: [0.0; 3],
        }
    }
}

/// How many frames a streamed source keeps ahead of its play position.
const STREAM_AHEAD: usize = 1 << 15;

impl Source {
    pub fn attach(&mut self, samples: Samples) {
        self.samples = Some(samples);
        self.cursor = 0.0;
        self.buffer_start = 0;
        self.window.clear();
        self.ended = false;
    }

    pub fn play(&mut self) {
        if matches!(self.state, State::Initial | State::Stopped | State::Paused)
            && self.samples.is_some()
        {
            self.state = State::Playing;
        }
    }

    fn channels(&self) -> usize {
        match &self.samples {
            Some(Samples::Whole(pcm)) => usize::from(pcm.channels.max(1)),
            Some(Samples::Streamed(feed)) => usize::from(feed.channels().max(1)),
            None => 1,
        }
    }

    fn rate(&self) -> u32 {
        match &self.samples {
            Some(Samples::Whole(pcm)) => pcm.rate,
            Some(Samples::Streamed(feed)) => feed.rate(),
            None => 44100,
        }
    }

    /// The frame `index` of the source, channel by channel (zero beyond what there is).
    fn frame(&self, index: i64, channel: usize) -> f32 {
        if index < 0 {
            return 0.0;
        }
        let channels = self.channels();
        let at = match &self.samples {
            Some(Samples::Whole(pcm)) => pcm.samples.get(index as usize * channels + channel),
            Some(Samples::Streamed(_)) => {
                let relative = (index as u64).checked_sub(self.buffer_start);
                relative
                    .and_then(|relative| self.window.get(relative as usize * channels + channel))
            }
            None => None,
        };
        at.map_or(0.0, |sample| f32::from(*sample) / 32768.0)
    }

    /// How many frames the source has in all, if that is known (not for a stream yet going).
    fn length(&self) -> Option<u64> {
        match &self.samples {
            Some(Samples::Whole(pcm)) => {
                Some((pcm.samples.len() / usize::from(pcm.channels.max(1))) as u64)
            }
            _ => None,
        }
    }

    /// Keep a streamed source's window filled ahead of the cursor.
    fn feed(&mut self) {
        let channels = self.channels();
        let cursor_frame = self.cursor as u64;
        let Some(Samples::Streamed(feed)) = &mut self.samples else {
            return;
        };
        // Drop what has been played.
        let played = (cursor_frame.saturating_sub(self.buffer_start) as usize)
            .min(self.window.len() / channels);
        if played > 1024 {
            self.window.drain(..played * channels);
            self.buffer_start += played as u64;
        }
        let want = (cursor_frame + STREAM_AHEAD as u64)
            .saturating_sub(self.buffer_start + (self.window.len() / channels) as u64);
        if want > 0 && !self.ended && !feed.pull(&mut self.window, want as usize * channels) {
            self.ended = true;
        }
    }
}

/// The sources and the listener, mixed into stereo.
pub struct Mixer {
    sources: Vec<Option<Source>>,
    listener: ListenerTransform,
    /// The output's samples per second.
    rate: u32,
}

impl Mixer {
    pub fn new(rate: u32) -> Self {
        Self {
            sources: Vec::new(),
            listener: ListenerTransform::default(),
            rate,
        }
    }

    pub fn set_listener(&mut self, listener: ListenerTransform) {
        self.listener = listener;
    }

    /// A new source; its number.
    pub fn add(&mut self) -> usize {
        if let Some(free) = self.sources.iter().position(Option::is_none) {
            self.sources[free] = Some(Source::default());
            return free;
        }
        self.sources.push(Some(Source::default()));
        self.sources.len() - 1
    }

    pub fn remove(&mut self, id: usize) {
        if let Some(slot) = self.sources.get_mut(id) {
            *slot = None;
        }
    }

    pub fn source(&mut self, id: usize) -> Option<&mut Source> {
        self.sources.get_mut(id)?.as_mut()
    }

    pub fn source_ref(&self, id: usize) -> Option<&Source> {
        self.sources.get(id)?.as_ref()
    }

    /// The gains of the left and right ears for a source at this place (`AL_LINEAR_DISTANCE_CLAMPED`
    /// and a constant-power pan by the angle to the sound).
    fn ear_gains(&self, source: &Source) -> (f32, f32) {
        let relative = if source.relative {
            source.position
        } else {
            std::array::from_fn(|axis| source.position[axis] - self.listener.position[axis])
        };
        let distance = (relative[0].powi(2) + relative[1].powi(2) + relative[2].powi(2)).sqrt();
        let mut gain = f64::from(source.gain);
        if let Some(max) = source.max_distance {
            let max = f64::from(max);
            gain *= if max > 0.0 {
                (1.0 - distance.min(max) / max).max(0.0)
            } else {
                0.0
            };
        }
        let gain = gain as f32;
        if source.channels() > 1 {
            return (gain, gain);
        }
        // To the listener's own axes: right is forward x up.
        let forward = self.listener.forward;
        let up = self.listener.up;
        let right = [
            forward[1] * up[2] - forward[2] * up[1],
            forward[2] * up[0] - forward[0] * up[2],
            forward[0] * up[1] - forward[1] * up[0],
        ];
        let pan = if distance > 1e-6 {
            let (axis, relative) = if source.relative {
                // A relative source is already in the listener's axes: x to the right.
                ([1.0, 0.0, 0.0], relative)
            } else {
                (right, relative)
            };
            ((relative[0] * axis[0] + relative[1] * axis[1] + relative[2] * axis[2]) / distance)
                .clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let angle = (pan + 1.0) * std::f64::consts::FRAC_PI_4;
        (gain * angle.cos() as f32, gain * angle.sin() as f32)
    }

    /// Mix the next `out.len() / 2` frames of stereo into `out` (interleaved left, right).
    pub fn render(&mut self, out: &mut [f32]) {
        out.fill(0.0);
        let frames = out.len() / 2;
        let ears: Vec<Option<(f32, f32)>> = self
            .sources
            .iter()
            .map(|slot| slot.as_ref().map(|source| self.ear_gains(source)))
            .collect();
        for (slot, ears) in self.sources.iter_mut().zip(ears) {
            let (Some(source), Some((left, right))) = (slot, ears) else {
                continue;
            };
            if source.state != State::Playing {
                continue;
            }
            let channels = source.channels();
            let step = f64::from(source.pitch) * f64::from(source.rate()) / f64::from(self.rate);
            for frame in 0..frames {
                source.feed();
                let index = source.cursor.floor() as i64;
                let fraction = (source.cursor - index as f64) as f32;
                let sample = |channel: usize| {
                    let a = source.frame(index, channel);
                    let b = source.frame(index + 1, channel);
                    a + (b - a) * fraction
                };
                let (mono_left, mono_right) = if channels == 1 {
                    let value = sample(0);
                    (value, value)
                } else {
                    (sample(0), sample(1))
                };
                out[frame * 2] += mono_left * left;
                out[frame * 2 + 1] += mono_right * right;
                source.cursor += step;
                let end = source.length();
                let past_end = match end {
                    Some(length) => source.cursor >= length as f64,
                    None => {
                        source.ended
                            && source.cursor
                                >= (source.buffer_start + (source.window.len() / channels) as u64)
                                    as f64
                    }
                };
                if past_end {
                    if source.looping && end.is_some() {
                        source.cursor -= end.unwrap_or(1) as f64;
                    } else {
                        source.state = State::Stopped;
                        break;
                    }
                }
            }
        }
        for sample in out.iter_mut() {
            *sample = sample.clamp(-1.0, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(frames: usize, rate: u32) -> Arc<Pcm> {
        Arc::new(Pcm {
            channels: 1,
            rate,
            samples: vec![16384; frames],
        })
    }

    fn playing(mixer: &mut Mixer, samples: Arc<Pcm>) -> usize {
        let id = mixer.add();
        let source = mixer.source(id).expect("added");
        source.attach(Samples::Whole(samples));
        source.play();
        id
    }

    #[test]
    fn a_sound_at_the_listener_is_heard_in_both_ears_and_fades_linearly_to_its_max_distance() {
        let mut mixer = Mixer::new(1000);
        let id = playing(&mut mixer, tone(100, 1000));
        mixer.source(id).expect("source").max_distance = Some(10.0);
        let mut out = vec![0.0; 2];
        mixer.render(&mut out);
        // Centre, at the listener: full gain, half in each ear (constant power).
        let centre = 0.5 * std::f32::consts::FRAC_1_SQRT_2;
        assert!((out[0] - centre).abs() < 1e-5 && (out[1] - centre).abs() < 1e-5);
        // Five of ten blocks away, straight ahead: half the gain.
        mixer.source(id).expect("source").position = [0.0, 0.0, -5.0];
        mixer.source(id).expect("source").cursor = 0.0;
        let mut out = vec![0.0; 2];
        mixer.render(&mut out);
        assert!((out[0] - centre * 0.5).abs() < 1e-5, "{}", out[0]);
        // Beyond the maximum: silent.
        mixer.source(id).expect("source").position = [0.0, 0.0, -50.0];
        mixer.source(id).expect("source").cursor = 0.0;
        let mut out = vec![0.0; 2];
        mixer.render(&mut out);
        assert_eq!(out, vec![0.0, 0.0]);
    }

    #[test]
    fn a_sound_to_the_right_is_heard_in_the_right_ear() {
        let mut mixer = Mixer::new(1000);
        let id = playing(&mut mixer, tone(100, 1000));
        // Facing -z with y up, +x is to the right.
        mixer.source(id).expect("source").position = [4.0, 0.0, 0.0];
        let mut out = vec![0.0; 2];
        mixer.render(&mut out);
        assert!(out[1] > 0.49 && out[0] < 1e-4, "{out:?}");
    }

    #[test]
    fn a_higher_pitch_plays_faster_and_ends_sooner() {
        let mut mixer = Mixer::new(1000);
        let id = playing(&mut mixer, tone(100, 1000));
        mixer.source(id).expect("source").pitch = 2.0;
        let mut out = vec![0.0; 2 * 60];
        mixer.render(&mut out);
        assert_eq!(mixer.source_ref(id).expect("source").state, State::Stopped);
        // Fifty frames' worth of sound, then silence.
        assert!(out[2 * 40] > 0.0 && out[2 * 55] == 0.0);
    }

    #[test]
    fn a_looping_sound_starts_over_and_a_paused_one_stays_where_it_is() {
        let mut mixer = Mixer::new(1000);
        let id = playing(&mut mixer, tone(10, 1000));
        mixer.source(id).expect("source").looping = true;
        let mut out = vec![0.0; 2 * 25];
        mixer.render(&mut out);
        assert_eq!(mixer.source_ref(id).expect("source").state, State::Playing);
        assert!(out[2 * 24] > 0.0);
        mixer.source(id).expect("source").state = State::Paused;
        let mut out = vec![0.0; 2 * 5];
        mixer.render(&mut out);
        assert!(out.iter().all(|sample| *sample == 0.0));
    }

    #[test]
    fn a_relative_sound_is_placed_from_the_listener_not_the_world() {
        let mut mixer = Mixer::new(1000);
        mixer.set_listener(ListenerTransform {
            position: [100.0, 50.0, 100.0],
            ..ListenerTransform::default()
        });
        let id = playing(&mut mixer, tone(100, 1000));
        let source = mixer.source(id).expect("source");
        source.relative = true;
        source.max_distance = Some(10.0);
        source.position = [0.0, 0.0, 0.0];
        let mut out = vec![0.0; 2];
        mixer.render(&mut out);
        assert!(out[0] > 0.3);
    }
}
