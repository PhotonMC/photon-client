//! The game's music (`MusicManager` over `Minecraft.getSituationalMusic`): the music of the
//! dimension and of the biome the player stands in, played through the sound system.

use std::path::Path;

use photon_core::json::JsonObject;
use photon_core::random::RandomSource;
use photon_game::options::MusicFrequency;
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

fn data_file(root: &Path, folder: &str, name: &str) -> Option<JsonObject> {
    let name = name.strip_prefix("minecraft:").unwrap_or(name);
    photon_core::json::read_object(
        &root
            .join("minecraft")
            .join(folder)
            .join(format!("{name}.json")),
    )
}

pub struct GameMusic {
    manager: MusicManager,
    dimension: Attributes,
    /// Each biome's, by its network number.
    biomes: Vec<Attributes>,
}

/// What the manager asks of the game, for one tick.
struct Host<'a> {
    sound: &'a mut SoundSystem,
    situation: Option<Music>,
    volume: f32,
}

impl MusicHost for Host<'_> {
    fn volume(&self) -> f32 {
        self.volume
    }
    fn situational_music(&self) -> Option<Music> {
        self.situation.clone()
    }
    fn loading_screen(&self) -> bool {
        false
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
}

impl GameMusic {
    /// The dimension's and the biomes' music from the data pack.
    pub fn load(data_root: &Path, dimension_type: &str, biome_names: &[String]) -> Self {
        Self {
            manager: MusicManager::new(
                RandomSource::create(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(0, |elapsed| elapsed.as_nanos() as i64),
                ),
                MusicFrequency::Default,
            ),
            dimension: read_attributes(
                data_file(data_root, "dimension_type", dimension_type).as_ref(),
            ),
            biomes: biome_names
                .iter()
                .map(|name| read_attributes(data_file(data_root, "worldgen/biome", name).as_ref()))
                .collect(),
        }
    }

    /// `Minecraft.getSituationalMusic` and `getMusicVolume` for the player: the biome at the
    /// player's feet over the dimension.
    fn situation(&self, session: &PlaySession) -> (Option<Music>, f32) {
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
        (selected.cloned(), volume)
    }

    /// One client tick of music.
    pub fn tick(
        &mut self,
        session: &PlaySession,
        sound: &mut SoundSystem,
        frequency: MusicFrequency,
    ) {
        let (situation, volume) = self.situation(session);
        let mut host = Host {
            sound,
            situation,
            volume,
        };
        self.manager.set_frequency_if_changed(frequency, &host);
        self.manager.tick(&mut host);
    }
}
