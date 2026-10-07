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

/// One particle asked for: `Level.addParticle`.
#[derive(Debug, Clone, Copy)]
pub struct Spawn {
    pub type_id: i32,
    /// The block state of a type that carries one (`BlockParticleOption`).
    pub block_state: Option<i32>,
    pub override_limiter: bool,
    pub always_show: bool,
    pub at: (f64, f64, f64),
    pub push: (f64, f64, f64),
}

/// `ClientboundLevelParticlesPacket`, of a type that is only its id or a block state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevelParticles {
    pub type_id: i32,
    /// The block state of a type that carries one.
    pub block_state: Option<i32>,
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
    to_add: VecDeque<Particle>,
    random: RandomSource,
    resources: ParticleResources,
}

impl ParticleEngine {
    /// An engine whose own random source (the one it picks a flame's picture and lets
    /// particles in past the reservoir's start with) is this one.
    pub fn new(random: RandomSource, resources: ParticleResources) -> Self {
        Self {
            group: ParticleGroup::default(),
            to_add: VecDeque::new(),
            random,
            resources,
        }
    }

    /// `ParticleEngine.createParticle`: whether the type's provider made one. The particle's own
    /// random source is asked for only when it is.
    pub fn create_particle(
        &mut self,
        world: &ClientWorld,
        (type_id, block_state): (i32, Option<i32>),
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        random: impl FnOnce() -> RandomSource,
    ) -> bool {
        match self.resources.create(
            world,
            (type_id, block_state),
            (at, push),
            random,
            &mut self.random,
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

    /// `ParticleEngine.add`: it joins the group at the end of the next tick.
    pub fn add(&mut self, particle: Particle) {
        self.to_add.push_back(particle);
    }

    /// `ParticleEngine.tick`: the group ticks, then the particles added meanwhile join it.
    pub fn tick(&mut self, world: &ClientWorld) {
        self.group.tick(world);
        while let Some(particle) = self.to_add.pop_front() {
            self.group.add(particle, &mut self.random);
        }
    }

    /// `ParticleEngine.clearParticles`.
    pub fn clear(&mut self) {
        self.group.clear();
        self.to_add.clear();
    }

    /// The particles in the group, in the order they tick.
    pub fn particles(&self) -> impl Iterator<Item = &Particle> {
        self.group.iter()
    }

    pub fn particle_count(&self) -> usize {
        self.group.len()
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
                (spawn.type_id, spawn.block_state),
                (spawn.at, spawn.push),
                &mut particle_random,
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
            block_state: packet.block_state,
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
