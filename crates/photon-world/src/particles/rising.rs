//! `SoulParticle` (on `RisingParticle`), `LavaParticle`, `CampfireSmokeParticle` and
//! `PlayerCloudParticle`.

use std::sync::Arc;

use photon_core::random::RandomSource;

use super::super::ClientWorld;
use super::engine::Spawn;
use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

impl Particle {
    /// `RisingParticle(..)`: a slow rise from a spot a little off where it was asked for.
    fn rising(
        random: RandomSource,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        sprite: (&Arc<SpriteSet>, usize),
        kind: Kind,
    ) -> Self {
        let mut particle = Self::thrown(random, at, push, sprite, kind).quad();
        particle.friction = 0.96;
        particle.xd = particle.xd * f64::from(0.01_f32) + push.0;
        particle.yd = particle.yd * f64::from(0.01_f32) + push.1;
        particle.zd = particle.zd * f64::from(0.01_f32) + push.2;
        particle.x +=
            f64::from((particle.random.next_float() - particle.random.next_float()) * 0.05);
        particle.y +=
            f64::from((particle.random.next_float() - particle.random.next_float()) * 0.05);
        particle.z +=
            f64::from((particle.random.next_float() - particle.random.next_float()) * 0.05);
        particle.lifetime =
            (8.0 / (f64::from(particle.random.next_float()) * 0.8 + 0.2)) as i32 + 4;
        particle
    }

    /// `RisingParticle(..)` as `FlameParticle` makes it.
    pub(super) fn rising_flame(
        random: RandomSource,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        sprite: (&Arc<SpriteSet>, usize),
    ) -> Self {
        Self::rising(random, at, push, sprite, Kind::Flame)
    }

    /// `SoulParticle.Provider` and `EmissiveProvider` (`glowing`): half again as big, its
    /// picture following its age.
    pub fn soul(
        random: RandomSource,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
        glowing: bool,
    ) -> Self {
        let mut particle = Self::rising(
            random,
            at,
            push,
            (sprites, sprites.first()),
            Kind::Soul { glowing },
        );
        particle.scale(1.5);
        particle.set_sprite_from_age();
        particle.alpha = 1.0;
        particle
    }

    /// `LavaParticle.Provider`: a spark thrown up, its picture drawn from the engine's random.
    pub fn lava(
        random: RandomSource,
        at: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
        engine_random: &mut RandomSource,
    ) -> Self {
        let sprite = sprites.from_random(engine_random);
        let mut particle =
            Self::thrown(random, at, (0.0, 0.0, 0.0), (sprites, sprite), Kind::Lava).quad();
        particle.gravity = 0.75;
        particle.friction = 0.999;
        particle.xd *= f64::from(0.8_f32);
        particle.yd *= f64::from(0.8_f32);
        particle.zd *= f64::from(0.8_f32);
        particle.yd = f64::from(particle.random.next_float() * 0.4 + 0.05);
        particle.quad_size *= particle.random.next_float() * 2.0 + 0.2;
        particle.lifetime = (16.0 / (f64::from(particle.random.next_float()) * 0.8 + 0.2)) as i32;
        particle
    }

    /// `LavaParticle.tick`'s smoke: the likelier the younger it is.
    pub(super) fn lava_smoke(&mut self) -> Option<Spawn> {
        let odds = self.age as f32 / self.lifetime as f32;
        (self.random.next_float() > odds).then(|| Spawn {
            type_id: photon_data::particle_type::id("smoke").unwrap_or(-1),
            data: super::engine::ParticleData::Plain,
            override_limiter: false,
            always_show: false,
            at: (self.x, self.y, self.z),
            push: (self.xd, self.yd, self.zd),
        })
    }

    /// `CampfireSmokeParticle.CosyProvider` and `SignalProvider` (`signal`): three times the size,
    /// living four or fourteen seconds and more, a little transparent.
    pub fn campfire_smoke(
        random: RandomSource,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        (sprites, signal): (&Arc<SpriteSet>, bool),
        engine_random: &mut RandomSource,
    ) -> Self {
        let sprite = sprites.from_random(engine_random);
        let mut particle = Self::at(random, at, (sprites, sprite), Kind::CampfireSmoke).quad();
        particle.scale(3.0);
        particle.set_size(0.25, 0.25);
        let base = if signal { 280 } else { 80 };
        particle.lifetime = particle.random.next_int_bound(50).unwrap_or(0) + base;
        particle.gravity = 3.0e-6;
        particle.xd = push.0;
        particle.yd = push.1 + f64::from(particle.random.next_float() / 500.0);
        particle.zd = push.2;
        particle.alpha = if signal { 0.95 } else { 0.9 };
        particle
    }

    /// `CampfireSmokeParticle.tick`: it wanders a little, sinks a hair, and fades in its last
    /// three seconds; it goes when its life or its opacity runs out.
    pub(super) fn campfire_tick(&mut self, world: &ClientWorld) {
        self.xo = self.x;
        self.yo = self.y;
        self.zo = self.z;
        let age = self.age;
        self.age = self.age.wrapping_add(1);
        if age < self.lifetime && self.alpha > 0.0 {
            let side = |random: &mut RandomSource| {
                let amount = random.next_float() / 5000.0;
                if random.next_boolean() {
                    amount
                } else {
                    -amount
                }
            };
            self.xd += f64::from(side(&mut self.random));
            self.zd += f64::from(side(&mut self.random));
            self.yd -= f64::from(self.gravity);
            self.move_by(world, self.xd, self.yd, self.zd);
            if self.age >= self.lifetime - 60 && self.alpha > 0.01 {
                self.alpha -= 0.015;
            }
        } else {
            self.removed = true;
        }
    }

    /// `PlayerCloudParticle.Provider` and `SneezeProvider` (`sneeze`: green and faint).
    pub fn player_cloud(
        random: RandomSource,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
        sneeze: bool,
    ) -> Self {
        let mut particle = Self::thrown(
            random,
            at,
            (0.0, 0.0, 0.0),
            (sprites, sprites.first()),
            Kind::PlayerCloud,
        )
        .quad();
        particle.friction = 0.96;
        particle.xd = particle.xd * f64::from(0.1_f32) + push.0;
        particle.yd = particle.yd * f64::from(0.1_f32) + push.1;
        particle.zd = particle.zd * f64::from(0.1_f32) + push.2;
        let colour = 1.0 - particle.random.next_float() * 0.3;
        particle.r_col = colour;
        particle.g_col = colour;
        particle.b_col = colour;
        particle.quad_size *= 1.875;
        let base = (8.0 / (f64::from(particle.random.next_float()) * 0.8 + 0.3)) as i32;
        particle.lifetime = (base as f32 * 2.5).max(1.0) as i32;
        particle.has_physics = false;
        particle.set_sprite_from_age();
        if sneeze {
            (particle.r_col, particle.g_col, particle.b_col) = (0.22, 1.0, 0.53);
            particle.alpha = 0.4;
        }
        particle
    }

    /// `PlayerCloudParticle.tick` after the move: its picture follows its age, and above a player
    /// within two blocks it settles a fifth of the way to the player's feet and their fall.
    pub(super) fn settle_to_player(&mut self, world: &ClientWorld) {
        self.set_sprite_from_age();
        if let Some((player_y, player_dy)) =
            world.nearest_player_feet((self.x, self.y, self.z), 2.0)
            && self.y > player_y
        {
            self.y += (player_y - self.y) * 0.2;
            self.yd += (player_dy - self.yd) * 0.2;
            self.set_pos(self.x, self.y, self.z);
        }
    }
}

impl ClientWorld {
    /// `EntityGetter.getNearestPlayer(x, y, z, range, false)`: the nearest player that is not a
    /// spectator, nearer than `range` (the local player first, then the others the server added);
    /// its feet's height and its vertical motion.
    pub(super) fn nearest_player_feet(
        &self,
        (x, y, z): (f64, f64, f64),
        range: f64,
    ) -> Option<(f64, f64)> {
        let local = self
            .local_player()
            .filter(|player| !player.game_mode().is_spectator());
        let local_id = self.local_player().map(|player| player.id);
        let players = local
            .map(|player| (player.position(), player.motion().1))
            .into_iter()
            .chain(
                self.entities()
                    .players()
                    .filter(|entity| Some(entity.id()) != local_id)
                    .filter(|entity| !entity.is_spectator())
                    .map(|entity| (entity.position(), entity.motion().1)),
            );
        let mut best: Option<(f64, (f64, f64))> = None;
        for ((px, py, pz), dy) in players {
            let (dx, dy_, dz) = (px - x, py - y, pz - z);
            let distance = dx * dx + dy_ * dy_ + dz * dz;
            if distance < range * range && best.is_none_or(|(nearest, _)| distance < nearest) {
                best = Some((distance, (py, dy)));
            }
        }
        best.map(|(_, feet)| feet)
    }
}
