//! `ParticleEngine`: the particles of the level, and how they get in.

use std::collections::VecDeque;

use voxel_core::binary::BlockPos;
use voxel_core::direction::Direction;
use voxel_core::math::Vec3;
use voxel_core::random::RandomSource;
use voxel_data::particle_type;

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
    Block(i32),
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
}

impl ParticleData {
    /// The block state of a type that carries one.
    pub fn block_state(self) -> Option<i32> {
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
    /// The `NO_RENDER` group: the emitters, never drawn.
    emitters: ParticleGroup,
    /// The `ITEM_PICKUP` group, and the ones waiting to join it.
    pickups: ParticleGroup<super::ItemPickup>,
    pickups_to_add: VecDeque<super::ItemPickup>,
    to_add: VecDeque<Particle>,
    random: RandomSource,
    resources: ParticleResources,
    /// `trackedParticleCounts` of `ParticleLimit.SPORE_BLOSSOM`: the spore blossom's particles
    /// alive or waiting.
    spore_blossoms: i32,
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
            pickups: ParticleGroup::default(),
            pickups_to_add: VecDeque::new(),
            to_add: VecDeque::new(),
            random,
            resources,
            spore_blossoms: 0,
        }
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
        (pos, state): (BlockPos, i32),
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
        let (mut emitted, limited_gone) = self.group.tick(world);
        self.spore_blossoms -= limited_gone;
        emitted.append(&mut self.emitters.tick(world).0);
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
            }
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

    /// `ParticleEngine.clearParticles`.
    pub fn clear(&mut self) {
        self.group.clear();
        self.emitters.clear();
        self.pickups.clear();
        self.pickups_to_add.clear();
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
    pub fn tick_pickups(&mut self, target: impl Fn(Option<i32>) -> Option<(f64, f64, f64)>) {
        self.pickups.retain_mut(|pickup| {
            let at = target(pickup.target).unwrap_or_else(|| pickup.target_at());
            pickup.tick(at);
            pickup.is_alive()
        });
        while let Some(pickup) = self.pickups_to_add.pop_front() {
            self.pickups.add(pickup, &mut self.random);
        }
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
