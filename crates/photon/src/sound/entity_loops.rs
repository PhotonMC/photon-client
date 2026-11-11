//! The sounds that follow an entity and change with it: a minecart's rolling and a bee's buzz
//! (started when the entity is added, `postAddEntitySoundInstance`), a guardian's beam and a
//! sniffer's digging (started by entity events 21 and 63), and what the local player hears of the
//! vehicle it sits in (`LocalPlayer.startRiding`). What each does every tick is
//! `photon_game::sounds::instances`; this reads the entity for it.

use std::collections::HashSet;
use std::sync::atomic::AtomicBool;

use photon_game::sounds::engine::{TickState, Ticker};
use photon_game::sounds::instances::{
    BEE_AGGRESSIVE, BEE_FLYING, BeeView, Buzz, GUARDIAN_ATTACK, MINECART, MinecartView,
    RIDING_GHAST, RIDING_MINECART, RIDING_MINECART_UNDERWATER, RIDING_NAUTILUS, RidingView,
    RidingVolume, SNIFFER_DIGGING, Setup, Voice, horizontal_speed, speed, tick_guardian_attack,
    tick_minecart, tick_riding, tick_sniffer_digging,
};

use super::*;

/// What the sounds read of an entity, taken once a tick.
#[derive(Clone, Copy)]
struct Snapshot {
    position: (f64, f64, f64),
    motion: (f64, f64, f64),
    off_rail: bool,
    baby: bool,
    angry: bool,
    attack_scale: f32,
    digging: bool,
    under_water: bool,
}

/// An entity some sound follows: what it is now (none once it is gone) and whether it is silent.
struct Watch {
    snapshot: Option<Snapshot>,
    silent: Arc<AtomicBool>,
}

type Watched = Arc<Mutex<HashMap<EntityId, Watch>>>;

/// The entity types the sounds are made for.
struct Kinds {
    minecarts: Vec<photon_core::ids::EntityTypeId>,
    bee: Option<photon_core::ids::EntityTypeId>,
    guardians: Vec<photon_core::ids::EntityTypeId>,
    sniffer: Option<photon_core::ids::EntityTypeId>,
    happy_ghast: Option<photon_core::ids::EntityTypeId>,
    nautiluses: Vec<photon_core::ids::EntityTypeId>,
}

fn kinds() -> &'static Kinds {
    static KINDS: std::sync::OnceLock<Kinds> = std::sync::OnceLock::new();
    KINDS.get_or_init(|| {
        let id = |name: &str| photon_data::entity_type::id(&format!("{NAMESPACE}{name}"));
        let all = |names: &[&str]| names.iter().filter_map(|name| id(name)).collect();
        Kinds {
            minecarts: all(&[
                "minecart",
                "chest_minecart",
                "furnace_minecart",
                "hopper_minecart",
                "tnt_minecart",
                "spawner_minecart",
                "command_block_minecart",
            ]),
            bee: id("bee"),
            guardians: all(&["guardian", "elder_guardian"]),
            sniffer: id("sniffer"),
            happy_ghast: id("happy_ghast"),
            nautiluses: all(&["nautilus", "zombie_nautilus"]),
        }
    })
}

/// The entities that have a sound, the vehicle the player has sounds for, and the buzzes to hand
/// over.
#[derive(Default)]
pub(super) struct Loops {
    started: HashSet<EntityId>,
    riding: Option<EntityId>,
    watched: Watched,
    /// The vehicle the local player sits in now.
    rides: Arc<Mutex<Option<EntityId>>>,
    /// The buzzes that were handed over this tick: the bee and the sound it now makes.
    handovers: Arc<Mutex<Vec<(EntityId, bool)>>>,
}

impl Loops {
    pub(super) fn clear(&mut self) {
        self.started.clear();
        self.riding = None;
        if let Ok(mut watched) = self.watched.lock() {
            watched.clear();
        }
        if let Ok(mut rides) = self.rides.lock() {
            *rides = None;
        }
        if let Ok(mut handovers) = self.handovers.lock() {
            handovers.clear();
        }
    }
}

/// The instance a setup describes.
fn instance_of(setup: Setup) -> SoundInstance {
    let mut instance = SoundInstance::new(
        &format!("{NAMESPACE}{}", setup.event),
        setup.source,
        unseeded(),
    );
    instance.volume = setup.volume;
    instance.looping = setup.looping;
    instance.can_start_silent = setup.can_start_silent;
    instance.attenuation = setup.attenuation;
    instance
}

/// Runs one of the ticks of `photon_game::sounds::instances` on what the engine holds of the
/// instance, and puts the result back.
fn with_voice(state: &mut TickState, tick: impl FnOnce(&mut Voice)) {
    let mut voice = Voice {
        position: *state.position,
        volume: *state.volume,
        pitch: *state.pitch,
        stopped: false,
    };
    tick(&mut voice);
    *state.position = voice.position;
    *state.volume = voice.volume;
    *state.pitch = voice.pitch;
    if voice.stopped {
        *state.stopped = true;
        *state.looping = false;
    }
}

fn snapshot_of(watched: &Watched, id: EntityId) -> Option<Snapshot> {
    watched.lock().ok()?.get(&id)?.snapshot
}

fn snapshot_view(entity: photon_world::EntityView<'_>, world: &ClientWorld) -> Snapshot {
    let kinds = kinds();
    let is_bee = Some(entity.type_id) == kinds.bee;
    Snapshot {
        position: entity.position(),
        motion: entity.motion(),
        off_rail: world.minecart_off_rail(entity),
        baby: is_bee && entity.is_baby(),
        angry: is_bee && entity.is_angry(world.game_time()),
        attack_scale: entity.guardian().map_or(0.0, |guardian| {
            guardian.attack_animation_scale(0.0, entity.guardian_attack_duration())
        }),
        digging: entity
            .sniffer()
            .is_some_and(|sniffer| sniffer.can_play_digging_sound()),
        under_water: world.entity_under_water(entity),
    }
}

impl SoundSystem {
    /// Gives an instance the silent flag of the entity it follows, and plays it.
    fn follow_entity(&mut self, id: EntityId, mut instance: SoundInstance, queued: bool) {
        if let Ok(mut watched) = self.loops.watched.lock() {
            let watch = watched.entry(id).or_insert_with(|| Watch {
                snapshot: None,
                silent: Arc::new(AtomicBool::new(false)),
            });
            instance.silenced = Some(Arc::clone(&watch.silent));
        }
        let instance_id = self.engine.add(instance);
        if queued {
            self.engine.queue_ticking_sound(instance_id);
        } else {
            self.engine.play(instance_id);
        }
    }

    fn minecart_ticker(&self, id: EntityId) -> Ticker {
        let watched = Arc::clone(&self.loops.watched);
        Box::new(move |state: &mut TickState| {
            let snapshot = snapshot_of(&watched, id);
            with_voice(state, |voice| match snapshot {
                None => tick_minecart(
                    voice,
                    &MinecartView {
                        removed: true,
                        position: voice.position,
                        speed: 0.0,
                        off_rail: false,
                        runs_normally: true,
                    },
                ),
                Some(now) => tick_minecart(
                    voice,
                    &MinecartView {
                        removed: false,
                        position: now.position,
                        speed: horizontal_speed(now.motion.0, now.motion.2),
                        off_rail: now.off_rail,
                        runs_normally: true,
                    },
                ),
            });
        })
    }

    fn bee_ticker(&self, id: EntityId, aggressive: bool) -> Ticker {
        let watched = Arc::clone(&self.loops.watched);
        let handovers = Arc::clone(&self.loops.handovers);
        let mut buzz = Buzz::new(aggressive);
        Box::new(move |state: &mut TickState| {
            let snapshot = snapshot_of(&watched, id);
            with_voice(state, |voice| {
                let view = BeeView {
                    removed: snapshot.is_none(),
                    position: snapshot.map_or(voice.position, |now| now.position),
                    speed: snapshot.map_or(0.0, |now| horizontal_speed(now.motion.0, now.motion.2)),
                    baby: snapshot.is_some_and(|now| now.baby),
                    angry: snapshot.is_some_and(|now| now.angry),
                };
                if buzz.tick(voice, &view)
                    && let Ok(mut handovers) = handovers.lock()
                {
                    handovers.push((id, !buzz.aggressive));
                }
            });
        })
    }

    fn start_minecart(&mut self, id: EntityId, position: (f64, f64, f64)) {
        let mut instance = instance_of(MINECART);
        instance.position = narrowed(position);
        instance.ticker = Some(self.minecart_ticker(id));
        self.follow_entity(id, instance, false);
    }

    fn start_bee(&mut self, id: EntityId, aggressive: bool, position: (f64, f64, f64)) {
        let mut instance = instance_of(if aggressive {
            BEE_AGGRESSIVE
        } else {
            BEE_FLYING
        });
        instance.position = narrowed(position);
        instance.ticker = Some(self.bee_ticker(id, aggressive));
        // `queueTickingSound`: it starts with the next tick.
        self.follow_entity(id, instance, true);
    }

    /// `GuardianAttackSoundInstance.tick` for the guardian `id`.
    fn guardian_ticker(&self, id: EntityId) -> Ticker {
        let watched = Arc::clone(&self.loops.watched);
        Box::new(move |state: &mut TickState| {
            let snapshot = snapshot_of(&watched, id);
            with_voice(state, |voice| {
                tick_guardian_attack(
                    voice,
                    snapshot.is_none(),
                    snapshot.map_or(voice.position, |now| now.position),
                    snapshot.map_or(0.0, |now| now.attack_scale),
                );
            });
        })
    }

    /// `SnifferSoundInstance.tick` for the sniffer `id`.
    fn sniffer_ticker(&self, id: EntityId) -> Ticker {
        let watched = Arc::clone(&self.loops.watched);
        Box::new(move |state: &mut TickState| {
            let snapshot = snapshot_of(&watched, id);
            with_voice(state, |voice| {
                tick_sniffer_digging(
                    voice,
                    snapshot.is_none(),
                    snapshot.map_or(voice.position, |now| now.position),
                    snapshot.is_some_and(|now| now.digging),
                );
            });
        })
    }

    /// `ClientPacketListener.handleEntityEvent` 21 and 63.
    pub fn start_entity_sounds(&mut self, events: Vec<(EntityId, i8)>, world: &ClientWorld) {
        for (id, event) in events {
            let Some(entity) = world.entities().at(id) else {
                continue;
            };
            let kinds = kinds();
            let ticker = match event {
                21 if kinds.guardians.contains(&entity.type_id) => self.guardian_ticker(id),
                63 if Some(entity.type_id) == kinds.sniffer => self.sniffer_ticker(id),
                _ => continue,
            };
            let mut instance = instance_of(if event == 21 {
                GUARDIAN_ATTACK
            } else {
                SNIFFER_DIGGING
            });
            instance.ticker = Some(ticker);
            self.follow_entity(id, instance, false);
        }
    }

    /// `RidingEntitySoundInstance.tick` for the vehicle `id`.
    fn riding_ticker(&self, id: EntityId, kind: Riding) -> Ticker {
        let watched = Arc::clone(&self.loops.watched);
        let rides = Arc::clone(&self.loops.rides);
        let player_under_water = Arc::clone(&self.underwater);
        let volume = RidingVolume {
            min: 0.0,
            max: kind.max_volume,
            amplifier: 1.0,
        };
        let follows = photon_core::versions::rules()
            .world
            .riding_sound_follows_vehicle;
        Box::new(move |state: &mut TickState| {
            let snapshot = snapshot_of(&watched, id);
            let rides_it = rides.lock().is_ok_and(|rides| *rides == Some(id));
            let water = if kind.minecart {
                player_under_water.load(Ordering::Relaxed)
            } else {
                snapshot.is_some_and(|now| now.under_water)
            };
            let view = RidingView {
                removed: snapshot.is_none(),
                rides_it,
                wrong_water: kind.underwater != water,
                speed: snapshot.map_or(0.0, |now| {
                    if kind.minecart {
                        horizontal_speed(now.motion.0, now.motion.2)
                    } else {
                        speed(now.motion)
                    }
                }),
                allowed: !kind.minecart || snapshot.is_some_and(|now| !now.off_rail),
                follow: follows.then(|| snapshot.map(|now| now.position)).flatten(),
            };
            with_voice(state, |voice| tick_riding(voice, &view, volume));
        })
    }

    fn start_riding(&mut self, (id, at): (EntityId, (f64, f64, f64)), kind: Riding, setup: Setup) {
        let mut instance = instance_of(setup);
        if photon_core::versions::rules()
            .world
            .riding_sound_follows_vehicle
        {
            instance.position = at;
        }
        instance.ticker = Some(self.riding_ticker(id, kind));
        self.follow_entity(id, instance, false);
    }

    /// `LocalPlayer.startRiding`: the sounds of a minecart, a happy ghast or a nautilus.
    fn start_riding_sounds(
        &mut self,
        vehicle: (EntityId, (f64, f64, f64)),
        type_id: photon_core::ids::EntityTypeId,
    ) {
        let kinds = kinds();
        if kinds.minecarts.contains(&type_id) {
            let kind = Riding::MINECART;
            self.start_riding(
                vehicle,
                Riding {
                    underwater: true,
                    ..kind
                },
                RIDING_MINECART_UNDERWATER,
            );
            self.start_riding(vehicle, kind, RIDING_MINECART);
        } else if Some(type_id) == kinds.happy_ghast {
            self.start_riding(vehicle, Riding::AFLOAT, RIDING_GHAST);
        } else if kinds.nautiluses.contains(&type_id) {
            self.start_riding(vehicle, Riding::UNDER, RIDING_NAUTILUS);
        }
    }

    /// One tick of the entity sounds, before the engine's: the entities that have one are looked
    /// at again, the ones that have none yet get theirs, and the buzzes that changed are handed on.
    pub(super) fn tick_entity_loops(&mut self, world: &ClientWorld) {
        let kinds = kinds();
        let mut present = HashSet::new();
        let mut added = Vec::new();
        for entity in world.entities().all() {
            let is_bee = Some(entity.type_id) == kinds.bee;
            if !is_bee && !kinds.minecarts.contains(&entity.type_id) {
                continue;
            }
            present.insert(entity.id());
            if self.loops.started.insert(entity.id()) {
                added.push((entity.id(), is_bee, entity.position()));
            }
        }
        self.loops.started.retain(|id| present.contains(id));
        for (id, is_bee, position) in added {
            if is_bee {
                // `postAddEntitySoundInstance` reads whether the bee is angry as it is added.
                let angry = world
                    .entities()
                    .at(id)
                    .is_some_and(|bee| bee.is_angry(world.game_time()));
                self.start_bee(id, angry, position);
            } else {
                self.start_minecart(id, position);
            }
        }
        self.follow_vehicle(world);
        self.refresh_watched(world);
    }

    /// The vehicle the local player sits in: its sounds start when the player mounts.
    fn follow_vehicle(&mut self, world: &ClientWorld) {
        let vehicle = world.ridden_vehicle_of_local_player();
        if vehicle != self.loops.riding {
            self.loops.riding = vehicle;
            if let Ok(mut rides) = self.loops.rides.lock() {
                *rides = vehicle;
            }
            if let Some(id) = vehicle
                && let Some((type_id, at)) = world
                    .entities()
                    .at(id)
                    .map(|entity| (entity.type_id, entity.position()))
            {
                self.start_riding_sounds((id, at), type_id);
            }
        }
    }

    fn refresh_watched(&self, world: &ClientWorld) {
        let Ok(mut watched) = self.loops.watched.lock() else {
            return;
        };
        for (id, watch) in watched.iter_mut() {
            let entity = world.entities().at(*id);
            watch.snapshot = entity.map(|entity| snapshot_view(entity, world));
            watch.silent.store(
                entity.is_some_and(|entity| entity.is_silent()),
                Ordering::Relaxed,
            );
        }
    }

    /// After the engine's tick: the entities that are gone are let go, and the bees that changed
    /// their buzz have the other one queued.
    pub(super) fn finish_entity_loops(&mut self, world: &ClientWorld) {
        if let Ok(mut watched) = self.loops.watched.lock() {
            watched.retain(|_, watch| watch.snapshot.is_some());
        }
        let handovers = self
            .loops
            .handovers
            .lock()
            .map(|mut handovers| std::mem::take(&mut *handovers))
            .unwrap_or_default();
        for (id, aggressive) in handovers {
            if let Some(bee) = world.entities().at(id) {
                self.start_bee(id, aggressive, bee.position());
            }
        }
    }
}

/// How a riding sound reads the vehicle.
#[derive(Clone, Copy)]
struct Riding {
    /// The sound is for the player being under water, not the vehicle.
    minecart: bool,
    /// The sound is the one under water.
    underwater: bool,
    max_volume: f32,
}

impl Riding {
    const MINECART: Self = Self {
        minecart: true,
        underwater: false,
        max_volume: 0.75,
    };
    const AFLOAT: Self = Self {
        minecart: false,
        underwater: false,
        max_volume: 1.0,
    };
    const UNDER: Self = Self {
        minecart: false,
        underwater: true,
        max_volume: 1.0,
    };
}

impl SoundSystem {
    /// `Hud.playAirBubblePoppedSound`: `Player.playSound`, heard from where the player is.
    pub fn handle_bubble_pops(&mut self, pops: Vec<(f32, f32)>, at: (f64, f64, f64)) {
        for (volume, pitch) in pops {
            self.local_sounds += 1;
            let seed = i64::from(self.local_sounds).wrapping_mul(0x5DEE_CE66_D1CE_4E5B);
            let name = format!("{NAMESPACE}ui.hud.bubble_pop");
            let mut instance =
                SoundInstance::new(&name, SoundSource::Players, RandomSource::create(seed));
            instance.volume = volume;
            instance.pitch = pitch;
            instance.position = at;
            let id = self.engine.add(instance);
            self.engine.play(id);
        }
    }
}
