//! `ParticleEngine`: the particles of the level, and how they get in.

use photon_core::ids::BlockStateId;
use photon_core::ids::{EntityId, ItemId};
use std::collections::VecDeque;

use photon_core::binary::BlockPos;
use photon_core::direction::Direction;
use photon_core::math::Vec3;
use photon_core::random::RandomSource;
use photon_data::particle_type;

use super::super::ClientWorld;
use super::group::ParticleGroup;
use super::particle::Particle;
use super::resources::ParticleResources;

/// `ParticleStatus`: the particles option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleStatus {
    All,
    Decreased,
    Minimal,
}

/// What `ClientLevel.doAddParticle` reads besides the level's random source.
#[derive(Debug, Clone, Copy)]
pub struct AddView {
    /// Where the camera is.
    pub camera: Vec3,
    pub status: ParticleStatus,
}

/// What a particle type carries after its id (`ParticleOptions`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ParticleData {
    /// `SimpleParticleType`: nothing.
    #[default]
    Plain,
    /// `BlockParticleOption`: a block state.
    Block(BlockStateId),
    /// `ColorParticleOption`: an ARGB colour.
    Color(u32),
    /// `SpellParticleOption`: a colour and how fast it goes.
    Spell { color: u32, power: f32 },
    /// `DustParticleOptions`: an RGB colour and a size (held between 0.01 and 4).
    Dust { color: u32, scale: f32 },
    /// `DustColorTransitionOptions`: from one colour to another, and a size.
    DustTransition { from: u32, to: u32, scale: f32 },
    /// `SculkChargeParticleOptions`: how far the quad is turned.
    Roll(f32),
    /// `ItemParticleOption`: the item, by its registry id (its components are not read).
    Item(ItemId),
    /// `VibrationParticleOption`: where it is headed, and how many ticks until it arrives.
    Vibration {
        destination: PositionSource,
        arrival_in_ticks: i32,
    },
    /// `TrailParticleOption`: where it is headed, its colour, and how long it takes.
    Trail {
        target: (f64, f64, f64),
        color: u32,
        duration: i32,
    },
    /// `ShriekParticleOption`: how many ticks before it starts ageing.
    Shriek { delay: i32 },
    /// What a firework's starter (or a spark with a trail) makes a `firework` spark with.
    FireworkSpark(super::firework::SparkOptions),
    /// `PowerParticleOption`: how hard it is thrown.
    Power(f32),
    /// `GeyserParticleOptions`: how many blocks of water the geyser rises through.
    Geyser { water_blocks: i32 },
    /// `GeyserBaseParticleOptions`: the same, and how hard the spray at its foot is thrown.
    GeyserBase { water_blocks: i32, burst: f32 },
}

/// `world.level.gameevent.PositionSource`: a fixed block (`BlockPositionSource`), or an entity
/// tracked by id with a vertical offset (`EntityPositionSource`, resolved each tick the way
/// `ClientLevel.getEntity` does).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PositionSource {
    Block(BlockPos),
    Entity { id: EntityId, y_offset: f32 },
}

impl PositionSource {
    /// `PositionSource.getPosition`: a block's centre, or a tracked entity's position (gone once
    /// it cannot be found), with the entity source's vertical offset added.
    pub fn position(self, world: &ClientWorld) -> Option<(f64, f64, f64)> {
        match self {
            Self::Block(pos) => Some((
                f64::from(pos.x) + 0.5,
                f64::from(pos.y) + 0.5,
                f64::from(pos.z) + 0.5,
            )),
            Self::Entity { id, y_offset } => {
                let (x, y, z) = world.entities().at(id)?.position();
                Some((x, y + f64::from(y_offset), z))
            }
        }
    }
}

impl ParticleData {
    /// The block state of a type that carries one.
    pub fn block_state(self) -> Option<BlockStateId> {
        match self {
            Self::Block(state) => Some(state),
            _ => None,
        }
    }
}

/// One particle asked for: `Level.addParticle`.
#[derive(Debug, Clone, Copy)]
pub struct Spawn {
    pub type_id: i32,
    /// What the type carries.
    pub data: ParticleData,
    pub override_limiter: bool,
    pub always_show: bool,
    pub at: (f64, f64, f64),
    pub push: (f64, f64, f64),
}

/// What a particle's tick asks the level for, in order: a particle, or a sound.
#[derive(Debug, Clone)]
pub enum Emitted {
    Particle(Spawn),
    Sound(crate::local_player::LocalSound),
    /// A gas cloud asks the level for a place near the cell it hangs in to send gas to.
    GasProbe([i32; 3]),
}

/// `ClientboundLevelParticlesPacket`, of a type that is only its id or a block state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelParticles {
    pub type_id: i32,
    /// What the type carries.
    pub data: ParticleData,
    pub override_limiter: bool,
    pub always_show: bool,
    pub at: (f64, f64, f64),
    pub spread: (f32, f32, f32),
    pub max_speed: f32,
    pub count: i32,
}

#[derive(Debug, Clone)]
pub struct ParticleEngine {
    group: ParticleGroup,
    /// `ParticleEngine.trackingEmitters`, ticked after the groups.
    trackers: Vec<super::TrackingEmitter>,
    /// The local player's entity id and body, which the level's entities do not hold.
    local_body: Option<(EntityId, super::EntityBody)>,
    /// The `NO_RENDER` group: the emitters, never drawn.
    emitters: ParticleGroup,
    /// `ClientExplosionTracker`: the explosions whose block particles the next level tick scatters.
    pub(super) explosions: Vec<super::Explosion>,
    /// The `ITEM_PICKUP` group, and the ones waiting to join it.
    pickups: ParticleGroup<super::ItemPickup>,
    pickups_to_add: VecDeque<super::ItemPickup>,
    /// The `ELDER_GUARDIANS` group, and the ones waiting to join it.
    elder_guardians: ParticleGroup<super::ElderGuardian>,
    elder_guardians_to_add: VecDeque<super::ElderGuardian>,
    to_add: VecDeque<Particle>,
    random: RandomSource,
    resources: ParticleResources,
    /// `trackedParticleCounts` of `ParticleLimit.SPORE_BLOSSOM`: the spore blossom's particles
    /// alive or waiting.
    spore_blossoms: i32,
    pub(super) jukeboxes: super::jukebox::Jukeboxes,
    /// The block a creative player's marker item stands for (`markerParticleTarget`).
    pub(super) marker_block: Option<usize>,
    /// `ClientLevel.rainSoundTime`.
    rain_sound_time: i32,
}

/// `ParticleLimit.SPORE_BLOSSOM`.
const SPORE_BLOSSOM_LIMIT: i32 = 1000;

impl ParticleEngine {
    /// An engine whose own random source (the one it picks a flame's picture and lets
    /// particles in past the reservoir's start with) is this one.
    pub fn new(random: RandomSource, resources: ParticleResources) -> Self {
        Self {
            group: ParticleGroup::default(),
            emitters: ParticleGroup::default(),
            explosions: Vec::new(),
            pickups: ParticleGroup::default(),
            trackers: Vec::new(),
            local_body: None,
            pickups_to_add: VecDeque::new(),
            elder_guardians: ParticleGroup::default(),
            elder_guardians_to_add: VecDeque::new(),
            to_add: VecDeque::new(),
            random,
            resources,
            spore_blossoms: 0,
            jukeboxes: super::jukebox::Jukeboxes::default(),
            marker_block: None,
            rain_sound_time: 0,
        }
    }

    pub(super) fn rain_sound_time(&mut self) -> &mut i32 {
        &mut self.rain_sound_time
    }

    /// The block the marker particles show (`ClientLevel.getMarkerParticleTarget`): a barrier or a
    /// light in the hand of a creative player.
    pub fn set_marker_block(&mut self, block: Option<usize>) {
        self.marker_block = block;
    }

    /// A new level: no record plays (`LevelEventHandler.playingJukeboxSongs` is the level's own).
    pub fn forget_jukeboxes(&mut self) {
        self.jukeboxes.forget_playing();
    }

    /// The block models and tints the flakes of blocks are made with.
    pub fn set_terrain(&mut self, materials: std::sync::Arc<dyn super::TerrainMaterials>) {
        self.resources.set_terrain(materials);
    }

    /// `ParticleEngine.createParticle`: whether the type's provider made one. The particle's own
    /// random source is asked for only when it is.
    pub fn create_particle(
        &mut self,
        world: &ClientWorld,
        (type_id, data): (i32, ParticleData),
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        (random, level_random): (impl FnOnce() -> RandomSource, &mut RandomSource),
    ) -> bool {
        // `ElderGuardianParticle.Provider`: a particle with no picture of the atlas.
        if particle_type::id("elder_guardian") == Some(type_id) {
            self.elder_guardians_to_add
                .push_back(super::ElderGuardian::new());
            return true;
        }
        match self.resources.create(
            world,
            (type_id, data),
            (at, push),
            random,
            (&mut self.random, level_random),
        ) {
            Some(particle) => {
                self.add(particle);
                true
            }
            None => false,
        }
    }

    /// `ClientLevel.addDestroyBlockEffect`: the flakes of a block that broke join the group.
    pub fn add_destroy_block_effect(
        &mut self,
        world: &ClientWorld,
        (pos, state): (BlockPos, BlockStateId),
        particle_random: impl FnMut() -> RandomSource,
    ) {
        let Some(materials) = self.resources.terrain().cloned() else {
            return;
        };
        for flake in super::terrain::destroy_block_effect(
            (pos, state),
            (world, materials.as_ref()),
            particle_random,
        ) {
            self.add(flake);
        }
    }

    /// `ClientLevel.addBreakingBlockEffect`: a flake from the face of the block being broken.
    pub fn add_breaking_block_effect(
        &mut self,
        world: &ClientWorld,
        (pos, direction): (BlockPos, Direction),
        level_random: &mut RandomSource,
        particle_random: impl FnOnce() -> RandomSource,
    ) {
        let Some(materials) = self.resources.terrain().cloned() else {
            return;
        };
        if let Some(flake) = super::terrain::breaking_block_effect(
            (pos, direction),
            (world, materials.as_ref()),
            level_random,
            particle_random,
        ) {
            self.add(flake);
        }
    }

    /// `ParticleEngine.add`: it joins the group at the end of the next tick, unless its limit
    /// (the spore blossom's) is reached.
    pub fn add(&mut self, particle: Particle) {
        if particle.is_limited() {
            if self.spore_blossoms >= SPORE_BLOSSOM_LIMIT {
                return;
            }
            self.spore_blossoms += 1;
        }
        self.to_add.push_back(particle);
    }

    /// `ParticleEngine.tick`: the group ticks, then the particles added meanwhile join it. What a
    /// particle asks the level for (lava's smoke) goes through `ClientLevel.addParticle`, with
    /// the level's random source and each new particle's own.
    pub fn tick(
        &mut self,
        world: &ClientWorld,
        view: &AddView,
        level_random: &mut RandomSource,
        mut particle_random: impl FnMut() -> RandomSource,
    ) -> Vec<crate::local_player::LocalSound> {
        // The reference ticks its groups in no fixed order; the emitters' particles go through
        // the level after the quads have moved.
        let (mut emitted, limited_gone) = self.group.tick(world, view.camera);
        self.spore_blossoms -= limited_gone;
        emitted.append(&mut self.emitters.tick(world, view.camera).0);
        emitted.extend(self.tick_trackers(world));
        let mut sounds = Vec::new();
        for request in emitted {
            match request {
                Emitted::Particle(spawn) => {
                    self.add_particle(world, &spawn, view, level_random, &mut particle_random);
                }
                Emitted::Sound(sound) => {
                    // `playLocalSound`'s seed.
                    level_random.next_long();
                    sounds.push(sound);
                }
                Emitted::GasProbe(cell) => {
                    self.send_gas(world, cell, view, level_random, &mut particle_random);
                }
            }
        }
        self.elder_guardians.retain_mut(super::ElderGuardian::tick);
        while let Some(particle) = self.elder_guardians_to_add.pop_front() {
            self.elder_guardians.add(particle, &mut self.random);
        }
        while let Some(particle) = self.to_add.pop_front() {
            let limited = particle.is_limited();
            let group = if particle.is_emitter() {
                &mut self.emitters
            } else {
                &mut self.group
            };
            if !group.add(particle, &mut self.random) && limited {
                self.spore_blossoms -= 1;
            }
        }
        sounds
    }

    /// Where the local player is, for the emitters that follow it.
    pub fn set_local_body(&mut self, body: Option<(EntityId, super::EntityBody)>) {
        self.local_body = body;
    }

    fn body_of(&self, world: &ClientWorld, id: EntityId) -> Option<super::EntityBody> {
        match self.local_body {
            Some((local, body)) if local == id => Some(body),
            _ => world
                .entities()
                .at(id)
                .map(crate::ClientEntity::tracking_body),
        }
    }

    /// `ParticleEngine.tick`'s emitters: each sprays once and those out of time are dropped.
    fn tick_trackers(&mut self, world: &ClientWorld) -> Vec<Emitted> {
        let mut trackers = std::mem::take(&mut self.trackers);
        let mut emitted = Vec::new();
        for tracker in &mut trackers {
            emitted.extend(tracker.tick(self.body_of(world, tracker.entity())));
        }
        trackers.retain(super::TrackingEmitter::is_alive);
        self.trackers = trackers;
        emitted
    }

    /// `ParticleEngine.createTrackingEmitter`: the emitter sprays at once, and goes on spraying at
    /// each tick until its time is out.
    pub fn create_tracking_emitter(
        &mut self,
        world: &ClientWorld,
        (entity, type_id, life_time): (EntityId, i32, i32),
        body: super::EntityBody,
        view: &AddView,
        level_random: &mut RandomSource,
        mut particle_random: impl FnMut() -> RandomSource,
    ) {
        let (emitter, emitted) =
            super::TrackingEmitter::new(entity, type_id, life_time, particle_random(), body);
        for request in emitted {
            if let Emitted::Particle(spawn) = request {
                self.add_particle(world, &spawn, view, level_random, &mut particle_random);
            }
        }
        if emitter.is_alive() {
            self.trackers.push(emitter);
        }
    }

    /// `ParticleEngine.clearParticles`.
    pub fn clear(&mut self) {
        self.group.clear();
        self.emitters.clear();
        self.trackers.clear();
        self.pickups.clear();
        self.pickups_to_add.clear();
        self.elder_guardians.clear();
        self.elder_guardians_to_add.clear();
        self.explosions.clear();
        self.to_add.clear();
        self.spore_blossoms = 0;
    }

    /// The particles in the group, in the order they tick.
    pub fn particles(&self) -> impl Iterator<Item = &Particle> {
        self.group.iter()
    }

    pub fn particle_count(&self) -> usize {
        self.group.len()
    }

    /// `ParticleEngine.add` of an `ItemPickupParticle`: it joins its group at the end of the
    /// next tick.
    pub fn add_pickup(&mut self, pickup: super::ItemPickup) {
        self.pickups_to_add.push_back(pickup);
    }

    /// The `ITEM_PICKUP` group's part of `ParticleEngine.tick`: each follows where its taker is
    /// now (`target`, by who took it; one gone is followed where it was last), then the ones
    /// added meanwhile join.
    pub fn tick_pickups(&mut self, target: impl Fn(Option<EntityId>) -> Option<(f64, f64, f64)>) {
        self.pickups.retain_mut(|pickup| {
            let at = target(pickup.target).unwrap_or_else(|| pickup.target_at());
            pickup.tick(at);
            pickup.is_alive()
        });
        while let Some(pickup) = self.pickups_to_add.pop_front() {
            self.pickups.add(pickup, &mut self.random);
        }
    }

    /// The elder guardians flashing over the screen, in the order they tick.
    pub fn elder_guardians(&self) -> impl Iterator<Item = &super::ElderGuardian> {
        self.elder_guardians.iter()
    }

    /// The entities flying to whoever took them.
    pub fn pickups(&self) -> impl Iterator<Item = &super::ItemPickup> {
        self.pickups.iter()
    }

    /// What the flakes of blocks are made with, if anything.
    pub(super) fn terrain_materials(&self) -> Option<std::sync::Arc<dyn super::TerrainMaterials>> {
        self.resources.terrain().cloned()
    }

    /// How many emitters are alive.
    pub fn emitter_count(&self) -> usize {
        self.emitters.len()
    }

    /// How many wait to join the group.
    pub fn pending_count(&self) -> usize {
        self.to_add.len()
    }

    /// `ParticleStatus` the next particle is made at: `ClientLevel.calculateParticleLevel`.
    fn particle_level(
        status: ParticleStatus,
        always_show: bool,
        level_random: &mut RandomSource,
    ) -> ParticleStatus {
        let mut level = status;
        if always_show
            && level == ParticleStatus::Minimal
            && level_random.next_int_bound(10) == Ok(0)
        {
            level = ParticleStatus::Decreased;
        }
        if level == ParticleStatus::Decreased && level_random.next_int_bound(3) == Ok(0) {
            level = ParticleStatus::Minimal;
        }
        level
    }

    /// `ClientLevel.doAddParticle`: a particle asked for is made when it ignores the limiter, or
    /// is within 32 blocks of the camera and the option is not at its least.
    pub fn add_particle(
        &mut self,
        world: &ClientWorld,
        spawn: &Spawn,
        view: &AddView,
        level_random: &mut RandomSource,
        mut particle_random: impl FnMut() -> RandomSource,
    ) {
        let override_limiter =
            particle_type::overrides_limiter(spawn.type_id) || spawn.override_limiter;
        let level = Self::particle_level(view.status, spawn.always_show, level_random);
        let (x, y, z) = spawn.at;
        let dx = view.camera.x - x;
        let dy = view.camera.y - y;
        let dz = view.camera.z - z;
        // Not "within": a position that is not a number counts as near, as the reference's does.
        let too_far = dx * dx + dy * dy + dz * dz > 1024.0;
        let near = !too_far;
        if override_limiter || (near && level != ParticleStatus::Minimal) {
            self.create_particle(
                world,
                (spawn.type_id, spawn.data),
                (spawn.at, spawn.push),
                (&mut particle_random, level_random),
            );
        }
    }

    /// `ClientPacketListener.handleParticleEvent`: no count is one particle moving as told; a
    /// count is that many, each spread and sped by the listener's random source.
    pub fn level_particles(
        &mut self,
        world: &ClientWorld,
        packet: &LevelParticles,
        view: &AddView,
        (listener_random, level_random): (&mut RandomSource, &mut RandomSource),
        mut particle_random: impl FnMut() -> RandomSource,
    ) {
        let spawn = |at: (f64, f64, f64), push: (f64, f64, f64)| Spawn {
            type_id: packet.type_id,
            data: packet.data,
            override_limiter: packet.override_limiter,
            always_show: packet.always_show,
            at,
            push,
        };
        if packet.count == 0 {
            let push = (
                f64::from(packet.max_speed * packet.spread.0),
                f64::from(packet.max_speed * packet.spread.1),
                f64::from(packet.max_speed * packet.spread.2),
            );
            self.add_particle(
                world,
                &spawn(packet.at, push),
                view,
                level_random,
                &mut particle_random,
            );
            return;
        }
        for _ in 0..packet.count {
            let x_variance = listener_random.next_gaussian() * f64::from(packet.spread.0);
            let y_variance = listener_random.next_gaussian() * f64::from(packet.spread.1);
            let z_variance = listener_random.next_gaussian() * f64::from(packet.spread.2);
            let xa = listener_random.next_gaussian() * f64::from(packet.max_speed);
            let ya = listener_random.next_gaussian() * f64::from(packet.max_speed);
            let za = listener_random.next_gaussian() * f64::from(packet.max_speed);
            self.add_particle(
                world,
                &spawn(
                    (
                        packet.at.0 + x_variance,
                        packet.at.1 + y_variance,
                        packet.at.2 + z_variance,
                    ),
                    (xa, ya, za),
                ),
                view,
                level_random,
                &mut particle_random,
            );
        }
    }
}
