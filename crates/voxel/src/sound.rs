//! The game's sound (`SoundManager`, `SoundEngine`): the sound definitions the assets hold, the
//! engine over the device, the sounds the server asks for played, and the listener put where the
//! camera is. With no device, or no `sounds.json`, the game goes on in silence.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use voxel_audio::channel::{MixerChannels, SoundAssets};
use voxel_audio::mixer::ListenerTransform;
use voxel_audio::output::Output;
use voxel_core::random::RandomSource;
use voxel_game::options::Options;
use voxel_game::sound_engine::{SoundEngine, SoundInstance};
use voxel_game::sounds::{
    SoundCommand, SoundEntityPacket, SoundHolder, SoundRegistry, SoundSource,
};
use voxel_world::ClientWorld;
use voxel_world::local_player::LocalSoundSource;

/// The asset index's objects: the sound files and `sounds.json` are kept by the hash of their content.
struct IndexedAssets {
    root: PathBuf,
    /// Index path (`minecraft/sounds/x.ogg`) to hash.
    hashes: HashMap<String, String>,
}

impl IndexedAssets {
    /// The index of the assets that sit above the jar's folder (`<assets>/jar/assets`).
    fn open(jar_assets: &Path) -> Option<Self> {
        let root = jar_assets.parent()?.parent()?.to_path_buf();
        let index = voxel_core::json::read_object(&root.join("indexes/32.json"))?;
        let hashes = index
            .object("objects")?
            .entries()
            .filter_map(|(key, entry)| {
                let JsonValue::Object(entry) = entry else {
                    return None;
                };
                Some((String::from_utf16(key).ok()?, entry.text("hash")?))
            })
            .collect();
        Some(Self { root, hashes })
    }

    fn object(&self, index_path: &str) -> Option<Vec<u8>> {
        let hash = self.hashes.get(index_path)?;
        std::fs::read(self.root.join("objects").join(&hash[..2]).join(hash)).ok()
    }
}

use voxel_core::json::JsonValue;

impl SoundAssets for IndexedAssets {
    /// `namespace:sounds/name.ogg`.
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        let (namespace, rest) = path.split_once(':')?;
        self.object(&format!("{namespace}/{rest}"))
    }
}

/// The positions of the entities that sounds follow (`EntityBoundSoundInstance`), put up to date
/// each tick; none once the entity is gone.
type Followed = Arc<Mutex<HashMap<i32, Option<(f64, f64, f64)>>>>;

pub struct SoundSystem {
    engine: SoundEngine<MixerChannels>,
    mixer: Arc<Mutex<voxel_audio::mixer::Mixer>>,
    _output: Output,
    followed: Followed,
    local_sounds: u32,
}

impl SoundSystem {
    /// Open the device and read the sound definitions; none if either cannot be had.
    pub fn open(jar_assets: &Path, options: &Options) -> Option<Self> {
        let output = match Output::open() {
            Ok(output) => output,
            Err(error) => {
                log::warn!("sound off: {}", error.0);
                return None;
            }
        };
        let assets = Arc::new(IndexedAssets::open(jar_assets)?);
        let text = String::from_utf8(assets.object("minecraft/sounds.json")?).ok()?;
        let mut registry = SoundRegistry::default();
        let exists = {
            let assets = Arc::clone(&assets);
            move |path: &str| {
                path.split_once(':').is_some_and(|(namespace, rest)| {
                    assets.hashes.contains_key(&format!("{namespace}/{rest}"))
                })
            }
        };
        if registry.register_file("minecraft", &text, &exists).is_err() {
            log::warn!("sound off: sounds.json is not valid");
            return None;
        }
        // `Library.init`: a device gives its own count of sources; thirty when it cannot say.
        let channels = MixerChannels::with_total(Arc::clone(&output.mixer), assets, 30);
        let mut engine = SoundEngine::new(registry, channels);
        for source in SoundSource::ALL {
            engine
                .volumes
                .set(source, options.sound_category_volume(source));
        }
        log::info!("sound on");
        Some(Self {
            engine,
            mixer: Arc::clone(&output.mixer),
            _output: output,
            followed: Arc::new(Mutex::new(HashMap::new())),
            local_sounds: 0,
        })
    }

    /// The sounds the server asked for (`handleSoundEvent`, `handleSoundEntityEvent`,
    /// `handleStopSoundEvent`), played or stopped.
    pub fn handle(&mut self, commands: Vec<SoundCommand>, world: &ClientWorld) {
        for command in commands {
            match command {
                SoundCommand::At(packet) => {
                    if let Some(instance) = SoundInstance::at(&packet) {
                        let id = self.engine.add(instance);
                        self.engine.play(id);
                    }
                }
                SoundCommand::FollowingEntity(packet) => self.follow(&packet, world),
                SoundCommand::Stop(packet) => self
                    .engine
                    .stop_matching(packet.name.as_deref(), packet.source),
            }
        }
    }

    /// How many sounds the local player has made.
    pub fn local_sound_count(&self) -> u32 {
        self.local_sounds
    }

    /// The sounds the local player made by moving (`Level.playLocalSound`): played where they were made.
    pub fn handle_local(&mut self, sounds: Vec<voxel_world::local_player::LocalSound>) {
        for sound in sounds {
            self.local_sounds += 1;
            let source = match sound.source {
                LocalSoundSource::Players => SoundSource::Players,
                LocalSoundSource::Blocks => SoundSource::Blocks,
            };
            // The reference draws the seed from the level's own random source.
            let seed = i64::from(self.local_sounds).wrapping_mul(0x5DEE_CE66_D1CE_4E5B);
            let mut instance = SoundInstance::new(sound.event, source, RandomSource::create(seed));
            instance.volume = sound.volume;
            instance.pitch = sound.pitch;
            instance.position = sound.position;
            let id = self.engine.add(instance);
            self.engine.play(id);
        }
    }

    /// `EntityBoundSoundInstance`: a sound that goes where an entity does, and ends with it.
    fn follow(&mut self, packet: &SoundEntityPacket, world: &ClientWorld) {
        let event = match &packet.sound {
            SoundHolder::Registered(id) => match voxel_data::sound_event::name(*id) {
                Some(name) => name.to_owned(),
                None => return,
            },
            SoundHolder::Direct { location, .. } => location.clone(),
        };
        let mut instance =
            SoundInstance::new(&event, packet.source, RandomSource::create(packet.seed));
        instance.volume = packet.volume;
        instance.pitch = packet.pitch;
        let entity = packet.entity_id;
        // `EntityBoundSoundInstance`: it starts where the entity is; a sound for an entity that is not there is not played.
        let Some(position) = world
            .entities()
            .find(|found| found.id() == entity)
            .map(|found| found.position())
        else {
            return;
        };
        instance.position = position;
        if let Ok(mut map) = self.followed.lock() {
            map.insert(entity, Some(position));
        }
        let followed = Arc::clone(&self.followed);
        instance.ticker = Some(Box::new(move |state| {
            match followed
                .lock()
                .ok()
                .and_then(|map| map.get(&entity).copied())
            {
                Some(Some(position)) => *state.position = position,
                _ => {
                    *state.stopped = true;
                    *state.looping = false;
                }
            }
        }));
        let id = self.engine.add(instance);
        self.engine.play(id);
    }

    /// `Options`' sound sliders, as the engine plays by them: a category that changed is applied
    /// to the sounds already playing (`SoundManager.refreshCategoryVolume`).
    pub fn apply_volumes(&mut self, options: &Options) {
        for source in SoundSource::ALL {
            let volume = options.sound_category_volume(source);
            if self.engine.volumes.slider_value(source) != volume {
                self.engine.volumes.set(source, volume);
                self.engine.refresh_category_volume(source);
            }
        }
    }

    /// One client tick of sound: the entities that sounds follow are looked up, then the engine ticks.
    pub fn tick(&mut self, world: &ClientWorld, paused: bool) {
        if let Ok(mut followed) = self.followed.lock() {
            for (id, position) in followed.iter_mut() {
                *position = world
                    .entities()
                    .find(|entity| entity.id() == *id)
                    .map(|entity| entity.position());
            }
        }
        self.engine.tick(paused);
    }

    /// `SoundEngine.updateSource`: the listener is where the camera is, facing where it faces.
    pub fn set_listener(&self, position: Vec3, forward: [f32; 3], up: [f32; 3]) {
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.set_listener(ListenerTransform {
                position: [position.x, position.y, position.z],
                forward: forward.map(f64::from),
                up: up.map(f64::from),
            });
        }
    }
}

use voxel_core::math::Vec3;
