//! The sounds of the place the player is in (`BiomeAmbientSoundsHandler`): the biome's
//! `audio/ambient_sounds` over the dimension's, played through the sound system.

use std::path::Path;
use std::sync::Arc;

use photon_core::random::RandomSource;
use photon_game::sounds::ambient::{
    AmbientHost, AmbientSounds, BiomeAmbient, LoopFade, Surroundings,
};
use photon_net::connection::PlaySession;

use crate::music::data_file;
use crate::sound::SoundSystem;

const AMBIENT_SOUNDS: &str = "minecraft:audio/ambient_sounds";

fn read(file: Option<&photon_core::json::JsonObject>) -> Option<AmbientSounds> {
    file?
        .object("attributes")?
        .object(AMBIENT_SOUNDS)
        .map(AmbientSounds::read)
}

pub struct GameAmbient {
    handler: BiomeAmbient,
    dimension: Option<AmbientSounds>,
    /// Each biome's, by its network number.
    biomes: Vec<Option<AmbientSounds>>,
}

struct Host<'a>(&'a mut SoundSystem);

impl AmbientHost for Host<'_> {
    fn start_loop(&mut self, event: &str, fade: Arc<LoopFade>) {
        self.0.play_ambient_loop(event, fade);
    }
    fn play_addition(&mut self, event: &str) {
        self.0.play_ambient_addition(event);
    }
    fn play_mood(&mut self, event: &str, position: (f64, f64, f64), random: RandomSource) {
        self.0.play_ambient_mood(event, position, random);
    }
}

impl GameAmbient {
    pub fn load(data_root: &Path, dimension_type: &str, biome_names: &[String]) -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos() as i64);
        Self {
            handler: BiomeAmbient::new(RandomSource::create(seed)),
            dimension: read(data_file(data_root, "dimension_type", dimension_type).as_ref()),
            biomes: biome_names
                .iter()
                .map(|name| read(data_file(data_root, "worldgen/biome", name).as_ref()))
                .collect(),
        }
    }

    /// `LocalPlayer.getCurrentMood`.
    pub fn moodiness(&self) -> f32 {
        self.handler.moodiness()
    }

    /// One client tick of the player's ambient sounds.
    pub fn tick(&mut self, session: &PlaySession, sound: &mut SoundSystem) {
        let Some(player) = session.player() else {
            return;
        };
        let world = session.world();
        let at = player.position();
        let biome = world
            .biome_at(
                at.x.floor() as i32,
                at.y.floor() as i32,
                at.z.floor() as i32,
            )
            .and_then(|id| usize::try_from(id).ok())
            .and_then(|id| self.biomes.get(id))
            .and_then(Option::as_ref);
        let empty = AmbientSounds::default();
        let sounds = biome.or(self.dimension.as_ref()).unwrap_or(&empty);
        let light = |x, y, z| {
            (
                world.sky_brightness(x, y, z),
                world.block_brightness(x, y, z),
            )
        };
        let around = Surroundings {
            position: (at.x, at.y, at.z),
            eye_y: at.y + f64::from(player.eye_height()),
            light: &light,
        };
        self.handler.tick(sounds, &around, &mut Host(sound));
    }
}
