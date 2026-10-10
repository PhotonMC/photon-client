//! `Particle` and `SingleQuadParticle`: the state every particle has, and how it moves.

use std::sync::Arc;

use photon_core::aabb::Aabb;
use photon_core::math::Vec3;
use photon_core::random::RandomSource;

use super::super::ClientWorld;
use super::super::collision::collide_entity_movement;
use super::super::local_player::Level;
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
    /// The bubbles, drops and ink of water, each with its own tick.
    Water(super::water::Water),
    /// `DustParticleBase`: grows in like smoke; a transition's colours to go between.
    Dust { transition: Option<[[f32; 3]; 2]> },
    /// `SpellParticle`: its opacity eases back to the one it was given.
    Spell { original_alpha: f32 },
    /// `TotemParticle` on `SimpleAnimatedParticle`: falls, fades in its second half, full bright.
    Totem,
    /// `EndRodParticle` on `SimpleAnimatedParticle`: goes through blocks, fades in its second half
    /// toward a pale colour, full bright.
    EndRod,
    /// `GlowParticle`: glows as it ages.
    Glow,
    /// The particles drawn to a place, each with its own tick.
    Attract(super::attract::Attract),
    /// `VibrationSignalParticle`: chases a position source, turning to face it.
    Vibration(super::vibration::Vibration),
    /// `TrailParticle`: chases a fixed target.
    Trail { target: [f64; 3] },
    /// `ShriekParticle`: does nothing while its delay counts down, then ages normally.
    Shriek { delay: i32 },
    /// `SuspendedParticle`: hangs in the air, the plain tick.
    Suspended,
    /// `CritParticle`: grows in, its green and blue fading each tick.
    Crit,
    /// `AttackSweepParticle`, `HugeExplosionParticle`, `GustParticle`: still and full bright,
    /// following its pictures.
    Burst,
    /// `ExplodeParticle`: the plain tick, following its pictures.
    Poof,
    /// `HeartParticle`, `NoteParticle`: the plain tick, grows in.
    Bobbing,
    /// `SculkChargeParticle`, `SculkChargePopParticle`: glows, translucent, follows its pictures.
    SculkCharge,
    /// `FallingLeavesParticle`: rides the wind down.
    FallingLeaves(super::leaves::Leaf),
    /// `FallingDustParticle`: drifts down, turning.
    FallingDust { rot_speed: f32 },
    /// `DripParticle`: a drop hanging, falling or landed.
    Drip(super::drip::Drip),
    /// A `NoRenderParticle`: never drawn, leaves others about it.
    Emitter(super::hit::Emitter),
    /// `BreakingItemParticle` of a picture of the particles' own (the sulfur cube's goo): a quarter
    /// of it, at a place drawn at random.
    Goo { uo: f32, vo: f32 },
    /// `FireworkParticles.Starter`: bursts the rocket's explosions into sparks.
    FireworkStarter(super::firework::Starter),
    /// `FireworkParticles.SparkParticle`: fades, twinkles and leaves trails.
    FireworkSpark(super::firework::Spark),
    /// `FireworkParticles.OverlayParticle`: the flash of an explosion.
    Flash,
    /// `DragonBreathParticle`: drifts out, settles to the ground and rises slowly from it.
    DragonBreath { has_hit_ground: bool },
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
    /// `getParticleLimit`: the spore blossom's air is counted against a limit.
    pub(super) limited: bool,
    /// The sound its last tick asked the level to play (a drop landing).
    pub(super) sound: Option<super::super::local_player::LocalSound>,
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
            limited: false,
            sound: None,
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
    pub fn tick(&mut self, world: &ClientWorld) -> Vec<super::engine::Spawn> {
        match self.kind {
            Kind::CampfireSmoke => {
                self.campfire_tick(world);
                return Vec::new();
            }
            Kind::Water(water) => {
                self.water_tick(world, water);
                return Vec::new();
            }
            Kind::Attract(attract) => {
                self.attract_tick(attract);
                return Vec::new();
            }
            Kind::Vibration(vibration) => {
                self.vibration_tick(world, vibration);
                return Vec::new();
            }
            Kind::Trail { target } => {
                self.trail_tick(target);
                return Vec::new();
            }
            Kind::Shriek { delay } if delay > 0 => {
                self.kind = Kind::Shriek { delay: delay - 1 };
                return Vec::new();
            }
            Kind::Burst => {
                self.burst_tick();
                return Vec::new();
            }
            Kind::Emitter(emitter) => return self.emitter_tick(emitter),
            Kind::FireworkStarter(_) => return Vec::new(),
            Kind::DragonBreath { has_hit_ground } => {
                self.dragon_breath_tick(world, has_hit_ground);
                return Vec::new();
            }
            Kind::Drip(drip) => return self.drip_tick(world, drip),
            Kind::FallingLeaves(leaf) => {
                self.falling_leaves_tick(world, leaf);
                return Vec::new();
            }
            Kind::FallingDust { rot_speed } => {
                self.falling_dust_tick(world, rot_speed);
                return Vec::new();
            }
            Kind::AshSmoke { decays: true } => {
                self.gravity *= 0.88;
                self.friction *= 0.92;
            }
            _ => {}
        }
        self.base_tick(world);
        match self.kind {
            Kind::AshSmoke { .. }
            | Kind::Soul { .. }
            | Kind::Dust { .. }
            | Kind::Glow
            | Kind::Poof
            | Kind::SculkCharge => {
                self.set_sprite_from_age();
            }
            Kind::Crit => {
                self.g_col *= 0.96;
                self.b_col *= 0.9;
            }
            Kind::Spell { original_alpha } => {
                self.set_sprite_from_age();
                // `Mth.lerp(0.05F, alpha, originalAlpha)`.
                self.alpha += 0.05 * (original_alpha - self.alpha);
            }
            Kind::Totem => self.animated_fade(),
            Kind::FireworkSpark(spark) => {
                self.set_sprite_from_age();
                return self.firework_spark_tick(spark);
            }
            Kind::EndRod => {
                self.animated_fade();
                if self.age > self.lifetime / 2 {
                    // `setFadeColor(15916745)`, closed on by a fifth each tick.
                    let [r, g, b] = [242.0 / 255.0, 222.0 / 255.0, 201.0 / 255.0];
                    self.r_col += (r - self.r_col) * 0.2;
                    self.g_col += (g - self.g_col) * 0.2;
                    self.b_col += (b - self.b_col) * 0.2;
                }
            }
            Kind::PlayerCloud if !self.removed => self.settle_to_player(world),
            Kind::Lava if !self.removed => return self.lava_smoke().into_iter().collect(),
            _ => {}
        }
        Vec::new()
    }

    /// `Particle.tick`.
    pub(super) fn base_tick(&mut self, world: &ClientWorld) {
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
        if matches!(self.kind, Kind::Flame | Kind::Attract(_) | Kind::EndRod) {
            self.bounding_box_move(xa, ya, za);
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

    /// `FlameParticle.move` and the others that go through blocks: the box moves, the position
    /// follows it.
    pub(super) fn bounding_box_move(&mut self, xa: f64, ya: f64, za: f64) {
        self.bounding_box = self.bounding_box.move_by(xa, ya, za);
        self.set_location_from_bounding_box();
    }

    /// The sound its last tick asked for, once.
    pub fn take_sound(&mut self) -> Option<super::super::local_player::LocalSound> {
        self.sound.take()
    }

    /// Whether it is a `NoRenderParticle`, kept in a group of its own and never drawn.
    pub fn is_emitter(&self) -> bool {
        matches!(self.kind, Kind::Emitter(_) | Kind::FireworkStarter(_))
    }

    /// Whether it counts against the spore blossom's limit.
    pub fn is_limited(&self) -> bool {
        self.limited
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

    /// The tint it is drawn in at a partial tick: a dust transition's colour between its two.
    pub fn colour_at(&self, partial: f32) -> [f32; 4] {
        match self.kind {
            Kind::Dust {
                transition: Some(colours),
            } => {
                let [r, g, b] = self.transition_colour(colours, partial);
                [r, g, b, self.alpha]
            }
            Kind::Attract(super::attract::Attract::FlyTowards { alpha, .. }) => [
                self.r_col,
                self.g_col,
                self.b_col,
                self.lifetime_alpha(alpha, partial),
            ],
            // `OverlayParticle.extract`.
            Kind::Flash => [
                self.r_col,
                self.g_col,
                self.b_col,
                0.6 - (self.age as f32 + partial - 1.0) * 0.25 * 0.5,
            ],
            _ => self.colour(),
        }
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
