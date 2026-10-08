//! `Particle` and `SingleQuadParticle`: the state every particle has, and how it moves.

use std::sync::Arc;

use voxel_core::aabb::Aabb;
use voxel_core::math::{Vec3, clamp_f32};
use voxel_core::random::RandomSource;

use super::super::ClientWorld;
use super::super::collision::collide_entity_movement;
use super::super::local_player::Level;
use super::light::{add_smooth_block_emission, particle_light};
use super::sprites::SpriteSet;

/// `Particle.MAXIMUM_COLLISION_VELOCITY_SQUARED`.
const MAXIMUM_COLLISION_VELOCITY_SQUARED: f64 = 10_000.0;

/// What a class built on `SingleQuadParticle` adds: how it is drawn, and what it does each tick.
#[derive(Debug, Clone)]
pub(super) enum Kind {
    /// `BaseAshSmokeParticle`: the picture follows the age, the quad grows in over the first
    /// thirty-second of its life. `DustPlumeParticle` (`decays`) slows its fall and its drift
    /// each tick.
    AshSmoke { decays: bool },
    /// `FlameParticle`: goes through blocks, shrinks and glows as it ages.
    Flame,
    /// `SoulParticle`: rises, follows its pictures, translucent; `sculk_soul`'s glows.
    Soul { glowing: bool },
    /// `LavaParticle`: falls, shrinks, glows, and leaves smoke as it goes.
    Lava,
    /// `CampfireSmokeParticle`: a slow column that wanders and fades out at its end.
    CampfireSmoke,
    /// `PlayerCloudParticle`: grows in like smoke and settles to a player beneath it.
    PlayerCloud,
    /// `TerrainParticle`: a flake of a block, showing a quarter of the block's picture at a
    /// place (`uo`, `vo`, in quarters of it) it draws at random.
    Terrain { uo: f32, vo: f32 },
}

/// Which atlas a particle's picture is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleAtlas {
    /// `particle/`.
    Particles,
    /// The blocks' own.
    Blocks,
}

/// A particle of the client level.
#[derive(Debug, Clone)]
pub struct Particle {
    pub(super) xo: f64,
    pub(super) yo: f64,
    pub(super) zo: f64,
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) z: f64,
    pub(super) xd: f64,
    pub(super) yd: f64,
    pub(super) zd: f64,
    bounding_box: Aabb,
    pub(super) on_ground: bool,
    pub(super) has_physics: bool,
    stopped_by_collision: bool,
    pub(super) removed: bool,
    bb_width: f32,
    bb_height: f32,
    pub(super) random: RandomSource,
    pub(super) age: i32,
    pub(super) lifetime: i32,
    pub(super) gravity: f32,
    pub(super) friction: f32,
    pub(super) speed_up_when_y_motion_is_blocked: bool,
    // `SingleQuadParticle`.
    pub(super) quad_size: f32,
    pub(super) r_col: f32,
    pub(super) g_col: f32,
    pub(super) b_col: f32,
    pub(super) alpha: f32,
    pub(super) roll: f32,
    pub(super) o_roll: f32,
    /// The textures its type lists, and which of them it shows.
    pub(super) sprites: Arc<SpriteSet>,
    pub(super) sprite: usize,
    pub(super) kind: Kind,
}

impl Particle {
    /// `Particle(level, x, y, z)`: the first draw is its life.
    pub(super) fn at(
        random: RandomSource,
        (x, y, z): (f64, f64, f64),
        (sprites, sprite): (&Arc<SpriteSet>, usize),
        kind: Kind,
    ) -> Self {
        let mut particle = Self {
            xo: x,
            yo: y,
            zo: z,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            xd: 0.0,
            yd: 0.0,
            zd: 0.0,
            // `INITIAL_AABB`.
            bounding_box: Aabb::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            on_ground: false,
            has_physics: true,
            stopped_by_collision: false,
            removed: false,
            bb_width: 0.6,
            bb_height: 1.8,
            random,
            age: 0,
            lifetime: 0,
            gravity: 0.0,
            friction: 0.98,
            speed_up_when_y_motion_is_blocked: false,
            quad_size: 0.0,
            r_col: 1.0,
            g_col: 1.0,
            b_col: 1.0,
            alpha: 1.0,
            roll: 0.0,
            o_roll: 0.0,
            sprites: Arc::clone(sprites),
            sprite,
            kind,
        };
        particle.set_size(0.2, 0.2);
        particle.set_pos(x, y, z);
        particle.lifetime = (4.0_f32 / (particle.random.next_float() * 0.9 + 0.1)) as i32;
        particle
    }

    /// `Particle(level, x, y, z, xa, ya, za)`: a first motion of the given one, spread and
    /// scaled.
    pub(super) fn thrown(
        random: RandomSource,
        at: (f64, f64, f64),
        (xa, ya, za): (f64, f64, f64),
        sprite: (&Arc<SpriteSet>, usize),
        kind: Kind,
    ) -> Self {
        let mut particle = Self::at(random, at, sprite, kind);
        particle.xd = xa + f64::from((particle.random.next_float() * 2.0 - 1.0) * 0.4);
        particle.yd = ya + f64::from((particle.random.next_float() * 2.0 - 1.0) * 0.4);
        particle.zd = za + f64::from((particle.random.next_float() * 2.0 - 1.0) * 0.4);
        let speed =
            f64::from((particle.random.next_float() + particle.random.next_float() + 1.0) * 0.15);
        let dd =
            (particle.xd * particle.xd + particle.yd * particle.yd + particle.zd * particle.zd)
                .sqrt();
        particle.xd = particle.xd / dd * speed * f64::from(0.4_f32);
        particle.yd = particle.yd / dd * speed * f64::from(0.4_f32) + f64::from(0.1_f32);
        particle.zd = particle.zd / dd * speed * f64::from(0.4_f32);
        particle
    }

    /// `SingleQuadParticle`'s size draw, after the particle's own.
    pub(super) fn quad(mut self) -> Self {
        self.quad_size = 0.1 * (self.random.next_float() * 0.5 + 0.5) * 2.0;
        self
    }

    /// `Particle.scale`, with `SingleQuadParticle`'s part of it.
    pub fn scale(&mut self, scale: f32) {
        self.quad_size *= scale;
        self.set_size(0.2 * scale, 0.2 * scale);
    }

    /// `Particle.setSize`: the box keeps its middle and its floor.
    pub(super) fn set_size(&mut self, width: f32, height: f32) {
        if width != self.bb_width || height != self.bb_height {
            self.bb_width = width;
            self.bb_height = height;
            let aabb = self.bounding_box;
            let new_min_x = (aabb.min_x + aabb.max_x - f64::from(width)) / 2.0;
            let new_min_z = (aabb.min_z + aabb.max_z - f64::from(width)) / 2.0;
            self.bounding_box = Aabb::new(
                new_min_x,
                aabb.min_y,
                new_min_z,
                new_min_x + f64::from(self.bb_width),
                aabb.min_y + f64::from(self.bb_height),
                new_min_z + f64::from(self.bb_width),
            );
        }
    }

    /// `Particle.setPos`.
    pub(super) fn set_pos(&mut self, x: f64, y: f64, z: f64) {
        self.x = x;
        self.y = y;
        self.z = z;
        let half = self.bb_width / 2.0;
        let height = self.bb_height;
        self.bounding_box = Aabb::new(
            x - f64::from(half),
            y,
            z - f64::from(half),
            x + f64::from(half),
            y + f64::from(height),
            z + f64::from(half),
        );
    }

    /// `Particle.setLocationFromBoundingbox`.
    fn set_location_from_bounding_box(&mut self) {
        let aabb = self.bounding_box;
        self.x = (aabb.min_x + aabb.max_x) / 2.0;
        self.y = aabb.min_y;
        self.z = (aabb.min_z + aabb.max_z) / 2.0;
    }

    /// `SingleQuadParticle.setSpriteFromAge`.
    pub(super) fn set_sprite_from_age(&mut self) {
        if !self.removed {
            self.sprite = self.sprites.from_age(self.age, self.lifetime);
        }
    }

    /// `Particle.tick`, with what the class does before and after it; a particle that leaves
    /// another (lava its smoke) asks for it.
    pub fn tick(&mut self, world: &ClientWorld) -> Option<super::engine::Spawn> {
        match self.kind {
            Kind::CampfireSmoke => {
                self.campfire_tick(world);
                return None;
            }
            Kind::AshSmoke { decays: true } => {
                self.gravity *= 0.88;
                self.friction *= 0.92;
            }
            _ => {}
        }
        self.base_tick(world);
        match self.kind {
            Kind::AshSmoke { .. } | Kind::Soul { .. } => self.set_sprite_from_age(),
            Kind::PlayerCloud if !self.removed => self.settle_to_player(world),
            Kind::Lava if !self.removed => return self.lava_smoke(),
            _ => {}
        }
        None
    }

    /// `Particle.tick`.
    fn base_tick(&mut self, world: &ClientWorld) {
        self.xo = self.x;
        self.yo = self.y;
        self.zo = self.z;
        let aged = self.age >= self.lifetime;
        self.age = self.age.wrapping_add(1);
        if aged {
            self.removed = true;
        } else {
            self.yd -= 0.04 * f64::from(self.gravity);
            self.move_by(world, self.xd, self.yd, self.zd);
            if self.speed_up_when_y_motion_is_blocked && self.y == self.yo {
                self.xd *= 1.1;
                self.zd *= 1.1;
            }
            self.xd *= f64::from(self.friction);
            self.yd *= f64::from(self.friction);
            self.zd *= f64::from(self.friction);
            if self.on_ground {
                self.xd *= f64::from(0.7_f32);
                self.zd *= f64::from(0.7_f32);
            }
        }
    }

    /// `Particle.move`, or `FlameParticle.move`, which does not collide.
    pub(super) fn move_by(&mut self, world: &ClientWorld, xa: f64, ya: f64, za: f64) {
        if matches!(self.kind, Kind::Flame) {
            self.bounding_box = self.bounding_box.move_by(xa, ya, za);
            self.set_location_from_bounding_box();
            return;
        }
        if self.stopped_by_collision {
            return;
        }
        let (original_xa, original_ya, original_za) = (xa, ya, za);
        let (mut xa, mut ya, mut za) = (xa, ya, za);
        if self.has_physics
            && (xa != 0.0 || ya != 0.0 || za != 0.0)
            && xa * xa + ya * ya + za * za < MAXIMUM_COLLISION_VELOCITY_SQUARED
        {
            let movement = Vec3::new(xa, ya, za);
            // A shape that depends on who asks is not carried yet: the particle stays where it is.
            let colliders = Level::new(world)
                .block_collisions(self.bounding_box.expand_towards_vec3(movement), 0.0);
            let Ok(colliders) = colliders else {
                return;
            };
            let shapes: Vec<_> = colliders.into_iter().map(|(_, shape)| shape).collect();
            let resolved = collide_entity_movement(movement, self.bounding_box, &shapes);
            xa = resolved.x;
            ya = resolved.y;
            za = resolved.z;
        }
        if xa != 0.0 || ya != 0.0 || za != 0.0 {
            self.bounding_box = self.bounding_box.move_by(xa, ya, za);
            self.set_location_from_bounding_box();
        }
        if original_ya.abs() >= f64::from(1.0e-5_f32) && ya.abs() < f64::from(1.0e-5_f32) {
            self.stopped_by_collision = true;
        }
        self.on_ground = original_ya != ya && original_ya < 0.0;
        if original_xa != xa {
            self.xd = 0.0;
        }
        if original_za != za {
            self.zd = 0.0;
        }
    }

    /// The size of the quad at a partial tick: `getQuadSize`.
    pub fn quad_size_at(&self, partial: f32) -> f32 {
        let s = (self.age as f32 + partial) / self.lifetime as f32;
        match self.kind {
            Kind::AshSmoke { .. } | Kind::PlayerCloud => {
                self.quad_size * clamp_f32(s * 32.0, 0.0, 1.0)
            }
            Kind::Flame => self.quad_size * (1.0 - s * s * 0.5),
            Kind::Lava => self.quad_size * (1.0 - s * s),
            _ => self.quad_size,
        }
    }

    /// The light the quad is drawn with at a partial tick: `getLightCoords`.
    pub fn light_coords_at(&self, world: &ClientWorld, partial: f32) -> i32 {
        let light = particle_light(world, self.x, self.y, self.z);
        match self.kind {
            Kind::Flame => {
                add_smooth_block_emission(light, (self.age as f32 + partial) / self.lifetime as f32)
            }
            // `LightCoordsUtil.withBlock(light, 15)`.
            Kind::Lava | Kind::Soul { glowing: true } => (light & 0xFF_0000) | (15 << 4),
            _ => light,
        }
    }

    /// `Particle.setPower`: the motion it started with, scaled (the rise less).
    pub fn set_power(&mut self, power: f32) {
        self.xd *= f64::from(power);
        self.yd = (self.yd - f64::from(0.1_f32)) * f64::from(power) + f64::from(0.1_f32);
        self.zd *= f64::from(power);
    }

    /// `SingleQuadParticle.Layer.translucent`: drawn blended rather than cut out.
    pub fn translucent(&self) -> bool {
        matches!(
            self.kind,
            Kind::Soul { .. } | Kind::CampfireSmoke | Kind::PlayerCloud
        )
    }

    /// The atlas the picture is in.
    pub fn atlas(&self) -> ParticleAtlas {
        match self.kind {
            Kind::Terrain { .. } => ParticleAtlas::Blocks,
            _ => ParticleAtlas::Particles,
        }
    }

    /// The part of its picture a particle shows, as fractions (`u0`, `u1`, `v0`, `v1`), where it is
    /// not the whole: `getU0` and the rest ask the sprite for a place within it.
    pub fn sprite_window(&self) -> Option<[f32; 4]> {
        match self.kind {
            Kind::Terrain { uo, vo } => {
                Some([(uo + 1.0) / 4.0, uo / 4.0, vo / 4.0, (vo + 1.0) / 4.0])
            }
            _ => None,
        }
    }

    pub fn is_alive(&self) -> bool {
        !self.removed
    }

    pub fn age(&self) -> i32 {
        self.age
    }

    pub fn lifetime(&self) -> i32 {
        self.lifetime
    }

    pub fn position(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    /// Where it was a tick ago, for drawing between the two.
    pub fn previous_position(&self) -> Vec3 {
        Vec3::new(self.xo, self.yo, self.zo)
    }

    pub fn delta(&self) -> Vec3 {
        Vec3::new(self.xd, self.yd, self.zd)
    }

    pub fn on_ground(&self) -> bool {
        self.on_ground
    }

    pub fn size(&self) -> (f32, f32) {
        (self.bb_width, self.bb_height)
    }

    pub fn bounding_box(&self) -> Aabb {
        self.bounding_box
    }

    pub fn quad_size(&self) -> f32 {
        self.quad_size
    }

    /// The tint and opacity: red, green, blue, alpha.
    pub fn colour(&self) -> [f32; 4] {
        [self.r_col, self.g_col, self.b_col, self.alpha]
    }

    /// Turns the quad about its own axis: the roll of the last tick and of this one.
    pub fn set_roll(&mut self, previous: f32, roll: f32) {
        self.o_roll = previous;
        self.roll = roll;
    }

    /// The roll of the last tick and of this one.
    pub fn roll(&self) -> (f32, f32) {
        (self.o_roll, self.roll)
    }

    pub fn gravity(&self) -> f32 {
        self.gravity
    }

    pub fn friction(&self) -> f32 {
        self.friction
    }

    /// Where in its sprite set the picture it has now is.
    pub fn sprite(&self) -> usize {
        self.sprite
    }

    /// The texture it shows now: one of the names its type lists.
    pub fn sprite_name(&self) -> &str {
        &self.sprites.names()[self.sprite]
    }
}
