//! The game's music (`MusicManager` over `Minecraft.getSituationalMusic`): the music of the
//! dimension and of the biome the player stands in, played through the sound system.

use std::path::Path;

use photon_core::json::JsonObject;
use photon_core::random::RandomSource;
use photon_game::options::MusicFrequency;
use photon_game::sounds::SoundSource;
use photon_game::sounds::music::{BackgroundMusic, Music, MusicHost, MusicManager, Started};
use photon_net::connection::PlaySession;

use crate::sound::SoundSystem;

const BACKGROUND_MUSIC: &str = "minecraft:audio/background_music";
const MUSIC_VOLUME: &str = "minecraft:audio/music_volume";

/// What one dimension or biome says of its music.
#[derive(Debug, Clone, Default)]
struct Attributes {
    music: Option<BackgroundMusic>,
    volume: Option<f32>,
}

fn read_music(object: &JsonObject) -> Option<Music> {
    Some(Music::new(
        &object.text("sound")?,
        object.i32("min_delay")?,
        object.i32("max_delay")?,
        object.bool("replace_current_music").unwrap_or(false),
    ))
}

fn read_attributes(file: Option<&JsonObject>) -> Attributes {
    let Some(attributes) = file.and_then(|file| file.object("attributes")) else {
        return Attributes::default();
    };
    let music = attributes
        .object(BACKGROUND_MUSIC)
        .map(|object| BackgroundMusic {
            default: object.object("default").and_then(read_music),
            creative: object.object("creative").and_then(read_music),
            underwater: object.object("underwater").and_then(read_music),
        });
    Attributes {
        music,
        volume: attributes.f32(MUSIC_VOLUME),
    }
}

pub(crate) fn data_file(root: &Path, folder: &str, name: &str) -> Option<JsonObject> {
    let name = name.strip_prefix("minecraft:").unwrap_or(name);
    photon_core::json::read_object(
        &root
            .join("minecraft")
            .join(folder)
            .join(format!("{name}.json")),
    )
}

fn new_manager() -> MusicManager {
    MusicManager::new(
        RandomSource::create(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos() as i64),
        ),
        MusicFrequency::Default,
    )
}

/// What the menus and the games hand each other, so one engine and one music manager serve the
/// whole run (`Minecraft.soundManager` and `musicManager` outlive the level).
pub struct Carried {
    pub sound: crate::sound::SoundSlot,
    pub manager: MusicManager,
}

impl Carried {
    /// The engine and the game's music manager, to hand to the menus.
    pub fn new(mut sound: crate::sound::SoundSlot, music: GameMusic) -> Self {
        // `Minecraft.disconnect`: leaving the level stops every sound.
        if let Some(system) = sound.as_mut() {
            system.reset_level();
        }
        Self {
            sound,
            manager: music.into_manager(),
        }
    }

    /// The parts, each absent when nothing was carried.
    pub fn split(carried: Option<Self>) -> (Option<crate::sound::SoundSlot>, Option<MusicManager>) {
        match carried {
            Some(Self { sound, manager }) => (Some(sound), Some(manager)),
            None => (None, None),
        }
    }
}

/// `Minecraft.getSituationalMusic` with no level: the credits' own track while their screen is
/// up (`WinScreen.getBackgroundMusic`), else `Musics.MENU`.
fn screen_or_menu(credits: bool) -> Music {
    if credits {
        Music::new("minecraft:music.credits", 0, 0, true)
    } else {
        Music::new("minecraft:music.menu", 20, 600, true)
    }
}

/// The music of the menus: `Musics.MENU` whenever there is no level, ticked at the client's rate.
pub struct MenuMusic {
    manager: MusicManager,
    last: std::time::Instant,
    owed: std::time::Duration,
}

/// `Minecraft.TICKS_PER_SECOND` as a duration, and how many ticks one frame may owe.
const TICK: std::time::Duration = std::time::Duration::from_millis(50);
const MAX_OWED_TICKS: u32 = 10;

impl MenuMusic {
    pub fn new(manager: Option<MusicManager>) -> Self {
        Self {
            manager: manager.unwrap_or_else(new_manager),
            last: std::time::Instant::now(),
            owed: std::time::Duration::ZERO,
        }
    }

    pub fn into_manager(self) -> MusicManager {
        self.manager
    }

    /// The client ticks the real time has owed: the manager asks for `Musics.MENU`; a level
    /// that is loading holds the next song back.
    pub fn tick(
        &mut self,
        sound: &mut SoundSystem,
        options: &photon_game::options::Options,
        (loading, credits): (bool, bool),
    ) {
        let now = std::time::Instant::now();
        self.owed = (self.owed + now.duration_since(self.last)).min(TICK * MAX_OWED_TICKS);
        self.last = now;
        while self.owed >= TICK {
            self.owed -= TICK;
            let mut host = Host {
                sound,
                situation: Some(screen_or_menu(credits)),
                volume: 1.0,
                toast: Vec::new(),
                loading,
            };
            self.manager
                .set_frequency_if_changed(options.music_frequency, &host);
            self.manager.tick(&mut host);
        }
    }
}

pub struct GameMusic {
    manager: MusicManager,
    dimension: Attributes,
    /// Each biome's, by its network number.
    biomes: Vec<Attributes>,
    /// The master and music sliders as they were last tick (`Options.createSoundSliderOptionInstance`
    /// reacts when one changes).
    sliders: Option<(f64, f64)>,
}

/// What the manager asks of the game, for one tick.
struct Host<'a> {
    sound: &'a mut SoundSystem,
    situation: Option<Music>,
    volume: f32,
    /// What the manager asked of the now-playing toast: shown (true) or hidden.
    toast: Vec<bool>,
    /// `LevelLoadingScreen` is up.
    loading: bool,
}

impl MusicHost for Host<'_> {
    fn volume(&self) -> f32 {
        self.volume
    }
    fn situational_music(&self) -> Option<Music> {
        self.situation.clone()
    }
    fn loading_screen(&self) -> bool {
        self.loading
    }
    fn is_active(&self) -> bool {
        self.sound.music_active()
    }
    fn play(&mut self, music: &Music) -> Started {
        self.sound.play_music(&music.sound)
    }
    fn stop(&mut self, _event: &str) {
        self.sound.stop_music();
    }
    fn update_music_volume(&mut self, gain: f32) {
        self.sound.update_music_volume(gain);
    }
    fn show_now_playing_toast(&mut self) {
        self.toast.push(true);
    }
    fn hide_now_playing_toast(&mut self) {
        self.toast.push(false);
    }
}

impl GameMusic {
    /// The dimension's and the biomes' music from the data pack.
    pub fn load(
        data_root: &Path,
        dimension_type: &str,
        biome_names: &[String],
        manager: Option<MusicManager>,
    ) -> Self {
        Self {
            manager: manager.unwrap_or_else(new_manager),
            dimension: read_attributes(
                data_file(data_root, "dimension_type", dimension_type).as_ref(),
            ),
            biomes: biome_names
                .iter()
                .map(|name| read_attributes(data_file(data_root, "worldgen/biome", name).as_ref()))
                .collect(),
            sliders: None,
        }
    }

    /// The music manager, which the menus take over when the game ends.
    pub fn into_manager(self) -> MusicManager {
        self.manager
    }

    /// `Minecraft.getSituationalMusic` and `getMusicVolume` for the player: the biome at the
    /// player's feet over the dimension.
    fn situation(&self, session: &PlaySession, screen: (bool, bool)) -> (Option<Music>, f32) {
        let (boss_music, credits) = screen;
        // A screen's own music comes first, at full volume (`getMusicVolume`).
        if credits {
            return (Some(screen_or_menu(true)), 1.0);
        }
        let Some(player) = session.player() else {
            return (None, 1.0);
        };
        let at = player.position();
        let biome = session
            .world()
            .biome_at(
                at.x.floor() as i32,
                at.y.floor() as i32,
                at.z.floor() as i32,
            )
            .and_then(|id| usize::try_from(id).ok())
            .and_then(|id| self.biomes.get(id));
        let music = biome
            .and_then(|biome| biome.music.as_ref())
            .or(self.dimension.music.as_ref());
        let volume = biome
            .and_then(|biome| biome.volume)
            .or(self.dimension.volume)
            .unwrap_or(1.0);
        let abilities = player.abilities();
        let creative = abilities.instabuild() && abilities.may_fly();
        let selected = music.and_then(|music| music.select(creative, player.is_under_water()));
        // `Musics.END_BOSS`: the dragon fight's track while a boss bar asks for music in the End.
        if boss_music && session.dimension() == "minecraft:the_end" {
            return (
                Some(Music::new("minecraft:music.dragon", 0, 0, true)),
                volume,
            );
        }
        (selected.cloned(), volume)
    }

    /// One client tick of music; what the manager asks of the now-playing toast goes to the GUI.
    pub fn tick(
        &mut self,
        (session, sound): (&PlaySession, &mut SoundSystem),
        options: &photon_game::options::Options,
        (gui, credits): (&mut crate::gui::Gui, bool),
    ) {
        let frequency = options.music_frequency;
        let (situation, volume) = self.situation(session, (gui.boss_bars.plays_music(), credits));
        let mut host = Host {
            sound,
            situation,
            volume,
            toast: Vec::new(),
            loading: false,
        };
        self.manager.set_frequency_if_changed(frequency, &host);
        // A change of the master or music slider that leaves the music audible shows the toast of
        // a song that began silently.
        let sliders = (
            options.sound_category_volume(SoundSource::Master),
            options.sound_category_volume(SoundSource::Music),
        );
        let changed = self
            .sliders
            .replace(sliders)
            .is_some_and(|before| before != sliders);
        if changed && sliders.0 * sliders.1 > 0.0 {
            self.manager.show_now_playing_toast_if_needed(&mut host);
        }
        self.manager.tick(&mut host);
        gui.music_toast_requests(host.toast, options.notification_display_time);
    }

    /// `MusicManager.getCurrentMusicTranslationKey`: the song the manager has playing, as its
    /// language key.
    pub fn current_song(&self, sound: &SoundSystem) -> Option<String> {
        self.manager.current()?;
        sound.music_song_key()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_credits_screen_asks_for_its_own_track_over_the_menu_music() {
        let credits = screen_or_menu(true);
        assert_eq!(credits.sound, "minecraft:music.credits");
        assert_eq!((credits.min_delay, credits.max_delay), (0, 0));
        assert!(credits.replace_current_music);
        assert_eq!(screen_or_menu(false).sound, "minecraft:music.menu");
    }
}
