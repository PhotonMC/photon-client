//! The sounds of a place (`BiomeAmbientSoundsHandler` over the `audio/ambient_sounds`
//! attribute): a loop that fades between places, additions that come by chance, and the mood
//! sound of the dark.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use photon_core::json::{JsonObject, JsonValue};
use photon_core::random::RandomSource;

/// Ticks a loop takes to fade in or out (`LOOP_SOUND_CROSS_FADE_TIME`).
const CROSS_FADE_TIME: i32 = 40;
/// How much brighter sky light lowers the mood per tick (`SKY_MOOD_RECOVERY_RATE`).
const SKY_MOOD_RECOVERY_RATE: f32 = 0.001;

/// `AmbientMoodSettings`.
#[derive(Debug, Clone, PartialEq)]
pub struct Mood {
    pub sound: String,
    pub tick_delay: i32,
    pub block_search_extent: i32,
    pub sound_position_offset: f64,
}

/// `AmbientAdditionsSettings`.
#[derive(Debug, Clone, PartialEq)]
pub struct Addition {
    pub sound: String,
    pub tick_chance: f64,
}

/// `AmbientSounds`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AmbientSounds {
    pub loop_sound: Option<String>,
    pub mood: Option<Mood>,
    pub additions: Vec<Addition>,
}

impl AmbientSounds {
    /// The attribute's value as a data file writes it (`AmbientSounds.CODEC`: `additions` may be
    /// one object or a list).
    pub fn read(object: &JsonObject) -> Self {
        let mood = object.object("mood").and_then(|mood| {
            Some(Mood {
                sound: mood.text("sound")?,
                tick_delay: mood.i32("tick_delay")?,
                block_search_extent: mood.i32("block_search_extent")?,
                sound_position_offset: mood.f64("offset")?,
            })
        });
        let read_addition = |addition: &JsonObject| {
            Some(Addition {
                sound: addition.text("sound")?,
                tick_chance: addition.f64("tick_chance")?,
            })
        };
        let mut additions: Vec<Addition> = object
            .array("additions")
            .map(|list| {
                list.iter()
                    .filter_map(|value| match value {
                        JsonValue::Object(addition) => read_addition(addition),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        additions.extend(object.object("additions").and_then(read_addition));
        Self {
            loop_sound: object.text("loop"),
            mood,
            additions,
        }
    }
}

/// `BiomeAmbientSoundsHandler.LoopSoundInstance`'s fade, shared with the sound that plays it.
#[derive(Debug, Default)]
pub struct LoopFade {
    fade: AtomicI32,
    direction: AtomicI32,
    stopped: AtomicBool,
}

impl LoopFade {
    /// `LoopSoundInstance.tick`: the volume the loop has now.
    pub fn tick(&self) -> f32 {
        let mut fade = self.fade.load(Ordering::Relaxed);
        if fade < 0 {
            self.stopped.store(true, Ordering::Relaxed);
        }
        fade += self.direction.load(Ordering::Relaxed);
        self.fade.store(fade, Ordering::Relaxed);
        (fade as f32 / CROSS_FADE_TIME as f32).clamp(0.0, 1.0)
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::Relaxed)
    }

    fn fade_out(&self) {
        let fade = self.fade.load(Ordering::Relaxed).min(CROSS_FADE_TIME);
        self.fade.store(fade, Ordering::Relaxed);
        self.direction.store(-1, Ordering::Relaxed);
    }

    fn fade_in(&self) {
        let fade = self.fade.load(Ordering::Relaxed).max(0);
        self.fade.store(fade, Ordering::Relaxed);
        self.direction.store(1, Ordering::Relaxed);
    }
}

/// What the handler asks of the sound engine.
pub trait AmbientHost {
    /// Play a loop that fades by the given state.
    fn start_loop(&mut self, event: &str, fade: Arc<LoopFade>);
    /// `SimpleSoundInstance.forAmbientAddition`.
    fn play_addition(&mut self, event: &str);
    /// `SimpleSoundInstance.forAmbientMood`: placed in the world.
    fn play_mood(&mut self, event: &str, position: (f64, f64, f64), random: RandomSource);
}

/// Where the player is and what light is around it.
pub struct Surroundings<'a> {
    pub position: (f64, f64, f64),
    pub eye_y: f64,
    /// Sky and block light at a block.
    pub light: &'a dyn Fn(i32, i32, i32) -> (u8, u8),
}

/// `BiomeAmbientSoundsHandler`.
pub struct BiomeAmbient {
    random: RandomSource,
    loops: Vec<(String, Arc<LoopFade>)>,
    moodiness: f32,
    previous_loop: Option<String>,
}

impl BiomeAmbient {
    pub fn new(random: RandomSource) -> Self {
        Self {
            random,
            loops: Vec::new(),
            moodiness: 0.0,
            previous_loop: None,
        }
    }

    /// `getMoodiness`.
    pub fn moodiness(&self) -> f32 {
        self.moodiness
    }

    pub fn tick(
        &mut self,
        sounds: &AmbientSounds,
        around: &Surroundings<'_>,
        host: &mut impl AmbientHost,
    ) {
        self.loops.retain(|(_, fade)| !fade.is_stopped());
        if sounds.loop_sound != self.previous_loop {
            self.previous_loop.clone_from(&sounds.loop_sound);
            for (_, fade) in &self.loops {
                fade.fade_out();
            }
            if let Some(event) = &sounds.loop_sound {
                let existing = self.loops.iter().find(|(name, _)| name == event);
                let fade = if let Some((_, fade)) = existing {
                    Arc::clone(fade)
                } else {
                    let fade = Arc::new(LoopFade::default());
                    host.start_loop(event, Arc::clone(&fade));
                    self.loops.push((event.clone(), Arc::clone(&fade)));
                    fade
                };
                fade.fade_in();
            }
        }
        for addition in &sounds.additions {
            if self.random.next_double() < addition.tick_chance {
                host.play_addition(&addition.sound);
            }
        }
        if let Some(mood) = &sounds.mood {
            self.tick_mood(mood, around, host);
        }
    }

    fn tick_mood(&mut self, mood: &Mood, around: &Surroundings<'_>, host: &mut impl AmbientHost) {
        let extent = mood.block_search_extent;
        let span = extent * 2 + 1;
        let (x, _, z) = around.position;
        let offset = |random: &mut RandomSource| {
            f64::from(random.next_int_bound(span).unwrap_or(0)) - f64::from(extent)
        };
        let sample_x = (x + offset(&mut self.random)).floor() as i32;
        let sample_y = (around.eye_y + offset(&mut self.random)).floor() as i32;
        let sample_z = (z + offset(&mut self.random)).floor() as i32;
        let (sky, block) = (around.light)(sample_x, sample_y, sample_z);
        if sky > 0 {
            self.moodiness -= f32::from(sky) / 15.0 * SKY_MOOD_RECOVERY_RATE;
        } else {
            self.moodiness -= (f32::from(block) - 1.0) / mood.tick_delay as f32;
        }
        if self.moodiness >= 1.0 {
            let direction = (
                f64::from(sample_x) + 0.5 - x,
                f64::from(sample_y) + 0.5 - around.eye_y,
                f64::from(sample_z) + 0.5 - z,
            );
            let distance =
                (direction.0 * direction.0 + direction.1 * direction.1 + direction.2 * direction.2)
                    .sqrt();
            let source_distance = distance + mood.sound_position_offset;
            let random = RandomSource::create(self.random.next_long());
            host.play_mood(
                &mood.sound,
                (
                    x + direction.0 / distance * source_distance,
                    around.eye_y + direction.1 / distance * source_distance,
                    z + direction.2 / distance * source_distance,
                ),
                random,
            );
            self.moodiness = 0.0;
        } else {
            self.moodiness = self.moodiness.max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Recorder {
        loops: Vec<String>,
        additions: Vec<String>,
        moods: Vec<(String, (f64, f64, f64))>,
    }

    impl AmbientHost for Recorder {
        fn start_loop(&mut self, event: &str, _: Arc<LoopFade>) {
            self.loops.push(event.to_owned());
        }
        fn play_addition(&mut self, event: &str) {
            self.additions.push(event.to_owned());
        }
        fn play_mood(&mut self, event: &str, position: (f64, f64, f64), _: RandomSource) {
            self.moods.push((event.to_owned(), position));
        }
    }

    fn cave() -> AmbientSounds {
        AmbientSounds {
            loop_sound: None,
            mood: Some(Mood {
                sound: "minecraft:ambient.cave".to_owned(),
                tick_delay: 6000,
                block_search_extent: 8,
                sound_position_offset: 2.0,
            }),
            additions: Vec::new(),
        }
    }

    #[test]
    fn the_dark_builds_the_mood_until_the_sound_plays() {
        let mut handler = BiomeAmbient::new(RandomSource::create(1));
        let mut host = Recorder::default();
        let dark = |_: i32, _: i32, _: i32| (0, 0);
        let around = Surroundings {
            position: (0.5, 10.0, 0.5),
            eye_y: 11.62,
            light: &dark,
        };
        // Darkness with no light at all gives 1/6000 a tick.
        for _ in 0..6010 {
            handler.tick(&cave(), &around, &mut host);
        }
        assert_eq!(host.moods.len(), 1);
        assert_eq!(host.moods[0].0, "minecraft:ambient.cave");
    }

    #[test]
    fn the_sky_wears_the_mood_away() {
        let mut handler = BiomeAmbient::new(RandomSource::create(1));
        let mut host = Recorder::default();
        let bright = |_: i32, _: i32, _: i32| (15, 0);
        let around = Surroundings {
            position: (0.5, 64.0, 0.5),
            eye_y: 65.62,
            light: &bright,
        };
        for _ in 0..100 {
            handler.tick(&cave(), &around, &mut host);
        }
        assert!(host.moods.is_empty());
        assert_eq!(handler.moodiness(), 0.0);
    }

    #[test]
    fn a_new_loop_starts_once_and_the_old_one_fades_out() {
        let mut handler = BiomeAmbient::new(RandomSource::create(1));
        let mut host = Recorder::default();
        let lit = |_: i32, _: i32, _: i32| (15, 0);
        let around = Surroundings {
            position: (0.0, 64.0, 0.0),
            eye_y: 65.62,
            light: &lit,
        };
        let mut sounds = AmbientSounds {
            loop_sound: Some("minecraft:ambient.a".to_owned()),
            ..AmbientSounds::default()
        };
        handler.tick(&sounds, &around, &mut host);
        handler.tick(&sounds, &around, &mut host);
        assert_eq!(host.loops, ["minecraft:ambient.a"]);
        sounds.loop_sound = Some("minecraft:ambient.b".to_owned());
        handler.tick(&sounds, &around, &mut host);
        assert_eq!(host.loops.len(), 2);
    }
}
