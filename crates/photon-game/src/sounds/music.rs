//! `MusicManager`: which song plays, how long the quiet between songs is, and how a song is faded
//! with the music volume. What the game is doing (which music fits, whether a song is still
//! playing) is the host's.

use photon_core::random::RandomSource;

use crate::options::MusicFrequency;

/// `MusicManager.STARTING_DELAY`.
const STARTING_DELAY: i32 = 100;

/// `net.minecraft.sounds.Music`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Music {
    /// The sound event, `minecraft:music.game`.
    pub sound: String,
    pub min_delay: i32,
    pub max_delay: i32,
    pub replace_current_music: bool,
}

impl Music {
    pub fn new(sound: &str, min_delay: i32, max_delay: i32, replace_current_music: bool) -> Self {
        Self {
            sound: sound.to_owned(),
            min_delay,
            max_delay,
            replace_current_music,
        }
    }

    /// `Musics.createGameMusic`.
    pub fn game(sound: &str) -> Self {
        Self::new(sound, 12000, 24000, false)
    }
}

/// `BackgroundMusic`: the music of a place, by the player's circumstances.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BackgroundMusic {
    pub default: Option<Music>,
    pub creative: Option<Music>,
    pub underwater: Option<Music>,
}

impl BackgroundMusic {
    /// `BackgroundMusic.select`.
    pub fn select(&self, creative: bool, underwater: bool) -> Option<&Music> {
        if underwater && self.underwater.is_some() {
            self.underwater.as_ref()
        } else if creative && self.creative.is_some() {
            self.creative.as_ref()
        } else {
            self.default.as_ref()
        }
    }
}

/// What a request to play asks of the sound engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Started {
    Yes,
    Silently,
    No,
}

/// What the music manager asks of the game.
pub trait MusicHost {
    /// `Minecraft.getMusicVolume`.
    fn volume(&self) -> f32;
    /// `Minecraft.getSituationalMusic`.
    fn situational_music(&self) -> Option<Music>;
    /// Whether the level loading screen is up.
    fn loading_screen(&self) -> bool;
    /// `SoundManager.isActive` of the song the manager started, by its event.
    fn is_active(&self) -> bool;
    /// `SoundManager.play(SimpleSoundInstance.forMusic(..))`.
    fn play(&mut self, music: &Music) -> Started;
    /// `SoundManager.stop` of the song playing.
    fn stop(&mut self, event: &str);
    /// `SoundManager.updateCategoryVolume(MUSIC, gain)`.
    fn update_music_volume(&mut self, gain: f32);
}

pub struct MusicManager {
    random: RandomSource,
    /// The event of the song playing.
    current: Option<String>,
    frequency: MusicFrequency,
    current_gain: f32,
    next_song_delay: i32,
}

/// `MusicFrequency.maxFrequency`, in ticks.
fn max_frequency(frequency: MusicFrequency) -> i32 {
    match frequency {
        MusicFrequency::Default => 20 * 1200,
        MusicFrequency::Frequent => 10 * 1200,
        MusicFrequency::Constant => 0,
    }
}

/// `Mth.nextInt`.
fn next_int(random: &mut RandomSource, min: i32, max: i32) -> i32 {
    if min >= max {
        min
    } else {
        random.next_int_bound(max - min + 1).unwrap_or(0) + min
    }
}

impl MusicManager {
    pub fn new(random: RandomSource, frequency: MusicFrequency) -> Self {
        Self {
            random,
            current: None,
            frequency,
            current_gain: 1.0,
            next_song_delay: STARTING_DELAY,
        }
    }

    pub fn next_song_delay(&self) -> i32 {
        self.next_song_delay
    }

    pub fn gain(&self) -> f32 {
        self.current_gain
    }

    pub fn current(&self) -> Option<&str> {
        self.current.as_deref()
    }

    /// `MusicFrequency.getNextSongDelay`.
    fn next_delay(&mut self, music: Option<&Music>) -> i32 {
        let Some(music) = music else {
            return max_frequency(self.frequency);
        };
        if self.frequency == MusicFrequency::Constant {
            return 100;
        }
        let limit = max_frequency(self.frequency);
        let min = music.min_delay.min(limit);
        let max = music.max_delay.min(limit);
        next_int(&mut self.random, min, max)
    }

    /// `setMinutesBetweenSongs`, when the option is not the one the manager has.
    pub fn set_frequency_if_changed(&mut self, frequency: MusicFrequency, host: &impl MusicHost) {
        if frequency != self.frequency {
            self.set_frequency(frequency, host);
        }
    }

    /// `setMinutesBetweenSongs`.
    pub fn set_frequency(&mut self, frequency: MusicFrequency, host: &impl MusicHost) {
        self.frequency = frequency;
        let music = host.situational_music();
        self.next_song_delay = self.next_delay(music.as_ref());
    }

    /// `MusicManager.tick`.
    pub fn tick(&mut self, host: &mut impl MusicHost) {
        let volume = host.volume();
        if self.current.is_some() && self.current_gain != volume && !self.fade(volume, host) {
            return;
        }
        let Some(music) = host.situational_music() else {
            self.next_song_delay = self.next_song_delay.max(STARTING_DELAY);
            return;
        };
        if let Some(current) = self.current.clone() {
            if music.replace_current_music && music.sound != current {
                host.stop(&current);
                self.next_song_delay = next_int(&mut self.random, 0, music.min_delay / 2);
            }
            if !host.is_active() {
                self.current = None;
                let delay = self.next_delay(Some(&music));
                self.next_song_delay = self.next_song_delay.min(delay);
            }
        }
        self.next_song_delay = self.next_song_delay.min(music.max_delay);
        if self.current.is_none() && !host.loading_screen() {
            self.next_song_delay = self.next_song_delay.wrapping_sub(1);
            if self.next_song_delay <= 0 {
                self.start_playing(&music, host);
            }
        }
    }

    /// `startPlaying`.
    pub fn start_playing(&mut self, music: &Music, host: &mut impl MusicHost) {
        self.current = Some(music.sound.clone());
        host.play(music);
        self.next_song_delay = i32::MAX;
    }

    /// `stopPlaying()`.
    pub fn stop_playing(&mut self, host: &mut impl MusicHost) {
        if let Some(current) = self.current.take() {
            host.stop(&current);
        }
        let music = host.situational_music();
        self.next_song_delay = self.next_delay(music.as_ref()) + 100;
    }

    /// `fadePlaying`: whether the song goes on.
    fn fade(&mut self, volume: f32, host: &mut impl MusicHost) -> bool {
        if self.current_gain < volume {
            self.current_gain += self.current_gain.clamp(5.0e-4, 0.005);
            if self.current_gain > volume {
                self.current_gain = volume;
            }
        } else {
            self.current_gain = 0.03 * volume + 0.97 * self.current_gain;
            if (self.current_gain - volume).abs() < 1.0e-4 || self.current_gain < volume {
                self.current_gain = volume;
            }
        }
        self.current_gain = self.current_gain.clamp(0.0, 1.0);
        if self.current_gain <= 1.0e-4 {
            self.stop_playing(host);
            false
        } else {
            host.update_music_volume(self.current_gain);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Host {
        music: Option<Music>,
        active: bool,
        played: Vec<String>,
        stops: i32,
    }

    impl MusicHost for Host {
        fn volume(&self) -> f32 {
            1.0
        }
        fn situational_music(&self) -> Option<Music> {
            self.music.clone()
        }
        fn loading_screen(&self) -> bool {
            false
        }
        fn is_active(&self) -> bool {
            self.active
        }
        fn play(&mut self, music: &Music) -> Started {
            self.played.push(music.sound.clone());
            self.active = true;
            Started::Yes
        }
        fn stop(&mut self, _: &str) {
            self.stops += 1;
            self.active = false;
        }
        fn update_music_volume(&mut self, _: f32) {}
    }

    #[test]
    fn a_song_starts_after_the_starting_delay_and_the_next_waits() {
        let mut host = Host {
            music: Some(Music::new("minecraft:music.game", 600, 1200, false)),
            active: false,
            played: Vec::new(),
            stops: 0,
        };
        let mut manager = MusicManager::new(RandomSource::create(1), MusicFrequency::Default);
        for _ in 0..99 {
            manager.tick(&mut host);
        }
        assert!(host.played.is_empty());
        manager.tick(&mut host);
        assert_eq!(host.played, ["minecraft:music.game"]);
        // The song ends: the next one waits a delay within the music's own.
        host.active = false;
        manager.tick(&mut host);
        assert!((600..=1200).contains(&manager.next_song_delay()));
    }

    #[test]
    fn music_that_replaces_stops_the_song_playing() {
        let mut host = Host {
            music: Some(Music::new("minecraft:music.game", 600, 1200, false)),
            active: false,
            played: Vec::new(),
            stops: 0,
        };
        let mut manager = MusicManager::new(RandomSource::create(1), MusicFrequency::Constant);
        for _ in 0..100 {
            manager.tick(&mut host);
        }
        host.music = Some(Music::new("minecraft:music.end", 600, 1200, true));
        manager.tick(&mut host);
        assert_eq!(host.stops, 1);
    }
}
