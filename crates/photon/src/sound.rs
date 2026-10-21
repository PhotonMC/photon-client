//! The game's sound (`SoundManager`, `SoundEngine`): the sound definitions the assets hold, the
//! engine over the device, the sounds the server asks for played, and the listener put where the
//! camera is. With no device, or no `sounds.json`, the game goes on in silence.

use photon_core::ids::EntityId;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use photon_audio::channel::{MixerChannels, SoundAssets};
use photon_audio::mixer::ListenerTransform;
use photon_audio::output::Output;
use photon_core::random::RandomSource;
use photon_game::options::Options;
use photon_game::sounds::engine::{SoundEngine, SoundInstance, TickState};
use photon_game::sounds::{
    SoundCommand, SoundEntityPacket, SoundHolder, SoundRegistry, SoundSource,
};
use photon_world::ClientWorld;
use photon_world::local_player::{LocalSoundSource, UnderwaterSound};

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
        let index = photon_core::json::read_object(&root.join("indexes/32.json"))?;
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

use photon_core::json::JsonValue;

impl SoundAssets for IndexedAssets {
    /// `namespace:sounds/name.ogg`.
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        let (namespace, rest) = path.split_once(':')?;
        self.object(&format!("{namespace}/{rest}"))
    }
}

/// The positions of the entities that sounds follow (`EntityBoundSoundInstance`), put up to date
/// each tick; none once the entity is gone.
type Followed = Arc<Mutex<HashMap<EntityId, Option<(f64, f64, f64)>>>>;

pub struct SoundSystem {
    engine: SoundEngine<MixerChannels>,
    mixer: Arc<Mutex<photon_audio::mixer::Mixer>>,
    _output: Output,
    followed: Followed,
    /// Where the listener is, which the sounds that hang at an angle from it follow.
    camera: Arc<Mutex<Vec3>>,
    local_sounds: u32,
    /// The song the music manager has playing.
    music: Option<photon_game::sounds::engine::InstanceId>,
    /// The record each jukebox is playing.
    jukeboxes: HashMap<(i32, i32, i32), photon_game::sounds::engine::InstanceId>,
    /// Whether the local player is under water, for the sounds that end when the water is left.
    underwater: Arc<AtomicBool>,
    /// Where the local player is, how they move and whether they glide.
    glide: Arc<Mutex<(Vec3, [f64; 3], bool)>>,
    /// The listener as last placed: where it is, which way it looks and which way is up.
    listener: Mutex<(Vec3, [f64; 3], [f64; 3])>,
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
            camera: Arc::new(Mutex::new(Vec3::ZERO)),
            local_sounds: 0,
            music: None,
            jukeboxes: HashMap::new(),
            underwater: Arc::new(AtomicBool::new(false)),
            glide: Arc::new(Mutex::new((Vec3::ZERO, [0.0; 3], false))),
            listener: Mutex::new((Vec3::ZERO, [0.0, 0.0, -1.0], [0.0, 1.0, 0.0])),
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

    /// The channels in use in each pool and the pools' sizes (`Library.getChannelDebugString`).
    pub fn channel_usage(&self) -> [(usize, usize); 2] {
        self.engine.channels().usage()
    }

    /// How many sounds the local player has made.
    pub fn local_sound_count(&self) -> u32 {
        self.local_sounds
    }

    /// The sounds the local player made by moving (`Level.playLocalSound`): played where they were made.
    pub fn handle_local(&mut self, sounds: Vec<photon_world::local_player::LocalSound>) {
        for sound in sounds {
            self.local_sounds += 1;
            let source = match sound.source {
                LocalSoundSource::Players => SoundSource::Players,
                LocalSoundSource::Blocks => SoundSource::Blocks,
                LocalSoundSource::Hostile => SoundSource::Hostile,
                LocalSoundSource::Neutral => SoundSource::Neutral,
                LocalSoundSource::Records => SoundSource::Records,
                LocalSoundSource::Ambient => SoundSource::Ambient,
            };
            // The reference draws the seed from the level's own random source.
            let seed = i64::from(self.local_sounds).wrapping_mul(0x5DEE_CE66_D1CE_4E5B);
            let mut instance = SoundInstance::new(sound.event, source, RandomSource::create(seed));
            if sound.event == photon_world::local_player::GLIDE_SOUND {
                self.glide_loop(&mut instance);
            }
            instance.volume = sound.volume;
            instance.pitch = sound.pitch;
            instance.position = sound.position;
            if sound.relative {
                instance.relative = true;
                instance.attenuation = photon_game::sounds::engine::Attenuation::None;
            }
            let id = self.engine.add(instance);
            if sound.delay_ticks > 0 {
                self.engine.play_delayed(id, sound.delay_ticks);
            } else {
                self.engine.play(id);
            }
        }
    }

    /// The player's place and motion for the glide loop, and whether they glide.
    pub fn set_glide(&self, position: Vec3, motion: [f64; 3], gliding: bool) {
        if let Ok(mut glide) = self.glide.lock() {
            *glide = (position, motion, gliding);
        }
    }

    /// `ElytraOnPlayerSoundInstance`: silent for a second, then as loud as the speed, until the
    /// glide has ended.
    fn glide_loop(&self, instance: &mut SoundInstance) {
        let glide = Arc::clone(&self.glide);
        instance.looping = true;
        let mut time = 0_i32;
        instance.ticker = Some(Box::new(move |state: &mut TickState| {
            time += 1;
            let Ok((position, motion, gliding)) = glide.lock().map(|glide| *glide) else {
                return;
            };
            if time > 20 && !gliding {
                *state.stopped = true;
                *state.looping = false;
                return;
            }
            // The reference places the sound at the player's position narrowed to floats.
            *state.position = (
                f64::from(position.x as f32),
                f64::from(position.y as f32),
                f64::from(position.z as f32),
            );
            let speed =
                (motion[0] * motion[0] + motion[1] * motion[1] + motion[2] * motion[2]) as f32;
            let mut volume = if speed >= 1.0e-7 {
                (speed / 4.0).clamp(0.0, 1.0)
            } else {
                0.0
            };
            if time < 20 {
                volume = 0.0;
            } else if time < 40 {
                volume *= (time - 20) as f32 / 20.0;
            }
            *state.volume = volume;
            *state.pitch = if volume > 0.8 {
                1.0 + (volume - 0.8)
            } else {
                1.0
            };
        }));
    }

    /// `Minecraft.setLevel`: a new level starts in silence (`SoundManager.stop`).
    pub fn reset_level(&mut self) {
        self.engine.stop_all();
        self.jukeboxes.clear();
    }

    /// What the level events brought: the sounds they play and the songs jukeboxes start and stop.
    pub fn handle_level_events(
        &mut self,
        sounds: Vec<photon_world::local_player::LocalSound>,
        songs: Vec<photon_world::particles::JukeboxChange>,
    ) {
        self.handle_local(sounds);
        self.handle_jukebox(songs);
    }

    /// `LevelEventHandler.playJukeboxSong` and `stopJukeboxSong`: a record sounds from the middle
    /// of its jukebox, over everything near, until it is stopped.
    pub fn handle_jukebox(&mut self, changes: Vec<photon_world::particles::JukeboxChange>) {
        use photon_world::particles::JukeboxChange;
        for change in changes {
            match change {
                JukeboxChange::Play { pos, sound, .. } => {
                    let mut instance =
                        SoundInstance::new(&sound, SoundSource::Records, RandomSource::create(0));
                    instance.volume = 4.0;
                    instance.position = (
                        f64::from(pos.0) + 0.5,
                        f64::from(pos.1) + 0.5,
                        f64::from(pos.2) + 0.5,
                    );
                    let id = self.engine.add(instance);
                    self.jukeboxes.insert(pos, id);
                    self.engine.play(id);
                }
                JukeboxChange::Notify { .. } => {}
                JukeboxChange::Stop { pos, .. } => {
                    if let Some(id) = self.jukeboxes.remove(&pos) {
                        self.engine.stop(id);
                    }
                }
            }
        }
    }

    /// `UnderwaterAmbientSoundInstance` and `SubSound`: the sounds of being under water, which
    /// follow whether the player still is.
    pub fn handle_underwater(&mut self, sounds: Vec<UnderwaterSound>, under_water: bool) {
        self.underwater.store(under_water, Ordering::Relaxed);
        for sound in sounds {
            let event = match sound {
                UnderwaterSound::Loop => "minecraft:ambient.underwater.loop",
                UnderwaterSound::Addition(event) => event,
            };
            let mut instance =
                SoundInstance::new(event, SoundSource::Ambient, RandomSource::create(0));
            instance.relative = true;
            let underwater = Arc::clone(&self.underwater);
            instance.ticker = Some(match sound {
                UnderwaterSound::Loop => {
                    instance.looping = true;
                    let mut fade = 0_i32;
                    Box::new(move |state: &mut TickState| {
                        if fade < 0 {
                            *state.stopped = true;
                            *state.looping = false;
                            return;
                        }
                        fade = if underwater.load(Ordering::Relaxed) {
                            fade + 1
                        } else {
                            fade - 2
                        };
                        fade = fade.min(40);
                        *state.volume = (fade as f32 / 40.0).clamp(0.0, 1.0);
                    })
                }
                UnderwaterSound::Addition(_) => Box::new(move |state: &mut TickState| {
                    if !underwater.load(Ordering::Relaxed) {
                        *state.stopped = true;
                    }
                }),
            });
            let id = self.engine.add(instance);
            self.engine.play(id);
        }
    }

    /// `BiomeAmbientSoundsHandler.LoopSoundInstance`: a loop over everything that fades by `fade`.
    pub fn play_ambient_loop(
        &mut self,
        event: &str,
        fade: Arc<photon_game::sounds::ambient::LoopFade>,
    ) {
        let mut instance = SoundInstance::new(event, SoundSource::Ambient, RandomSource::create(0));
        instance.relative = true;
        instance.looping = true;
        instance.ticker = Some(Box::new(move |state: &mut TickState| {
            *state.volume = fade.tick();
            if fade.is_stopped() {
                *state.stopped = true;
                *state.looping = false;
            }
        }));
        let id = self.engine.add(instance);
        self.engine.play(id);
    }

    /// `SimpleSoundInstance.forAmbientAddition`: not placed anywhere.
    pub fn play_ambient_addition(&mut self, event: &str) {
        use photon_game::sounds::engine::Attenuation;
        let mut instance = SoundInstance::new(event, SoundSource::Ambient, RandomSource::create(0));
        instance.relative = true;
        instance.attenuation = Attenuation::None;
        let id = self.engine.add(instance);
        self.engine.play(id);
    }

    /// `SimpleSoundInstance.forAmbientMood`: placed in the world.
    pub fn play_ambient_mood(
        &mut self,
        event: &str,
        position: (f64, f64, f64),
        random: RandomSource,
    ) {
        let mut instance = SoundInstance::new(event, SoundSource::Ambient, random);
        instance.position = position;
        let id = self.engine.add(instance);
        self.engine.play(id);
    }

    /// `EntityBoundSoundInstance`: a sound that goes where an entity does, and ends with it.
    fn follow(&mut self, packet: &SoundEntityPacket, world: &ClientWorld) {
        let event = match &packet.sound {
            SoundHolder::Registered(id) => match photon_data::sound_event::name(*id) {
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
            .all()
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
            if let Some(Some(position)) = followed
                .lock()
                .ok()
                .and_then(|map| map.get(&entity).copied())
            {
                *state.position = position;
            } else {
                *state.stopped = true;
                *state.looping = false;
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

    /// `SimpleSoundInstance.forMusic`: a song over everything, not placed anywhere.
    pub fn play_music(&mut self, event: &str) -> photon_game::sounds::music::Started {
        use photon_game::sounds::engine::{Attenuation, PlayResult};
        use photon_game::sounds::music::Started;
        let mut instance = SoundInstance::new(event, SoundSource::Music, RandomSource::create(0));
        instance.attenuation = Attenuation::None;
        instance.relative = true;
        let id = self.engine.add(instance);
        self.music = Some(id);
        log::info!("music: {event}");
        match self.engine.play(id) {
            PlayResult::Started => Started::Yes,
            PlayResult::StartedSilently => Started::Silently,
            PlayResult::NotStarted => Started::No,
        }
    }

    /// `MusicManager.getCurrentMusicTranslationKey`: the language key of the song that plays, its
    /// file's name without the `minecraft` namespace (`Identifier.toShortLanguageKey`).
    pub fn music_song_key(&self) -> Option<String> {
        let sound = self.music.and_then(|id| self.engine.picked_sound(id))?;
        let location = &sound.location;
        Some(if location.namespace() == "minecraft" {
            location.path().to_owned()
        } else {
            format!("{}.{}", location.namespace(), location.path())
        })
    }

    /// `SimpleSoundInstance.forUI`: a sound of the interface, heard at the same volume wherever
    /// the listener is.
    pub fn play_ui(&mut self, event: &str, pitch: f32, volume: f32) {
        use photon_game::sounds::engine::Attenuation;
        let mut instance = SoundInstance::new(event, SoundSource::Ui, RandomSource::create(0));
        instance.attenuation = Attenuation::None;
        instance.relative = true;
        instance.pitch = pitch;
        instance.volume = volume;
        let id = self.engine.add(instance);
        self.engine.play(id);
    }

    pub fn stop_music(&mut self) {
        if let Some(id) = self.music.take() {
            self.engine.stop(id);
        }
    }

    pub fn music_active(&self) -> bool {
        self.music.is_some_and(|id| self.engine.is_active(id))
    }

    pub fn update_music_volume(&mut self, gain: f32) {
        self.engine.update_category_volume(SoundSource::Music, gain);
    }

    /// One client tick of sound: the entities that sounds follow are looked up, then the engine ticks.
    pub fn tick(&mut self, world: &ClientWorld, paused: bool) {
        if let Ok(mut followed) = self.followed.lock() {
            for (id, position) in followed.iter_mut() {
                *position = world
                    .entities()
                    .all()
                    .find(|entity| entity.id() == *id)
                    .map(|entity| entity.position());
            }
        }
        self.engine.tick(paused);
    }

    /// Whether the subtitles listen (`SoundEngine.addEventListener`).
    pub fn set_listening(&mut self, listening: bool) {
        self.engine.set_listening(listening);
    }

    /// The sounds that started since the subtitles last asked.
    pub fn take_heard(&mut self) -> Vec<photon_game::sounds::engine::Heard> {
        self.engine.take_heard()
    }

    /// Where the listener is, which way it looks and which way is up
    /// (`SoundManager.getListenerTransform`).
    pub fn listener_view(&self) -> (Vec3, [f64; 3], [f64; 3]) {
        self.listener.lock().map_or_else(
            |_| (Vec3::ZERO, [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]),
            |view| *view,
        )
    }

    /// `SoundEngine.updateSource`: the listener is where the camera is, facing where it faces.
    pub fn set_listener(&self, position: Vec3, forward: [f32; 3], up: [f32; 3]) {
        if let Ok(mut view) = self.listener.lock() {
            *view = (position, forward.map(f64::from), up.map(f64::from));
        }
        if let Ok(mut camera) = self.camera.lock() {
            *camera = position;
        }
        if let Ok(mut mixer) = self.mixer.lock() {
            mixer.set_listener(ListenerTransform {
                position: [position.x, position.y, position.z],
                forward: forward.map(f64::from),
                up: up.map(f64::from),
            });
        }
    }
}

use photon_core::math::Vec3;

mod flash;
