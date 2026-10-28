//! `FireworkParticles`: the starter that bursts a rocket's explosions into sparks, the sparks
//! (which fade, twinkle and leave trails) and the flash.

use std::sync::Arc;

use photon_core::math::{Vec3, fdlibm_cos, fdlibm_sin, mth_lerp_f64};
use photon_core::random::RandomSource;
use photon_data::particle_type;

use super::super::local_player::{LocalSound, LocalSoundSource};
use super::engine::{ParticleData, ParticleEngine, Spawn};
use super::level_events::LevelEventContext;
use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

pub use photon_data::particle_values::{
    FireworkExplosion as Explosion, FireworkShape as Shape, SparkOptions,
};

/// `SparkParticle`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Spark {
    trail: bool,
    twinkle: bool,
    fade: Option<[f32; 3]>,
}

/// `Starter`: the bursts it still has to make.
#[derive(Debug, Clone)]
pub(super) struct Starter {
    life: i32,
    explosions: Arc<Vec<Explosion>>,
    twinkle_delay: bool,
}

/// `DyeColor.BLACK.getFireworkColor`: the colour of an explosion with none.
const BLACK: i32 = 1_973_019;

const CREEPER_COORDS: [[f64; 2]; 12] = [
    [0.0, 0.2],
    [0.2, 0.2],
    [0.2, 0.6],
    [0.6, 0.6],
    [0.6, 0.2],
    [0.2, 0.2],
    [0.2, 0.0],
    [0.4, 0.0],
    [0.4, -0.6],
    [0.2, -0.6],
    [0.2, -0.4],
    [0.0, -0.4],
];

const STAR_COORDS: [[f64; 2]; 6] = [
    [0.0, 1.0],
    [0.3455, 0.309],
    [0.9511, 0.309],
    [0.379_591_836_734_693_9, -0.126_530_612_244_897_95],
    [0.612_244_897_959_183_7, -0.804_081_632_653_061_2],
    [0.0, -0.359_183_673_469_387_73],
];

/// `ColorParticleOption`'s three floats, of an `0xRRGGBB` colour.
fn channels(rgb: i32) -> [f32; 3] {
    [
        ((rgb & 0xFF0000) >> 16) as f32 / 255.0,
        ((rgb & 0xFF00) >> 8) as f32 / 255.0,
        (rgb & 0xFF) as f32 / 255.0,
    ]
}

impl Particle {
    /// `FireworkParticles.Starter(..)`: it lives two ticks for each explosion, and fifteen more to
    /// twinkle in.
    pub(super) fn firework_starter(
        random: RandomSource,
        (at, motion): ((f64, f64, f64), (f64, f64, f64)),
        explosions: Vec<Explosion>,
    ) -> Self {
        let sprites = Arc::new(SpriteSet::new(Vec::new()));
        let twinkle_delay = explosions.iter().any(|explosion| explosion.twinkle);
        let lifetime = explosions.len() as i32 * 2 - 1 + if twinkle_delay { 15 } else { 0 };
        let mut particle = Self::at(
            random,
            at,
            (&sprites, 0),
            Kind::FireworkStarter(Starter {
                life: 0,
                explosions: Arc::new(explosions),
                twinkle_delay,
            }),
        );
        (particle.xd, particle.yd, particle.zd) = motion;
        particle.lifetime = lifetime;
        particle
    }

    /// `SparkProvider` and the spark a trail leaves: the motion is given as it is.
    pub(super) fn firework_spark(
        random: RandomSource,
        (at, (xa, ya, za)): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        options: SparkOptions,
    ) -> Self {
        let spark = Spark {
            trail: options.trail,
            twinkle: options.twinkle,
            fade: options.fade,
        };
        let mut particle = Self::at(
            random,
            at,
            (sprites, sprites.first()),
            Kind::FireworkSpark(spark),
        )
        .quad();
        particle.friction = 0.91;
        particle.gravity = 0.1;
        (particle.xd, particle.yd, particle.zd) = (xa, ya, za);
        particle.quad_size *= 0.75;
        particle.lifetime = 48 + particle.random.next_int_bound(12).unwrap_or(0);
        particle.set_sprite_from_age();
        particle.alpha = 0.99;
        (particle.r_col, particle.g_col, particle.b_col) =
            (options.color[0], options.color[1], options.color[2]);
        if options.half_aged {
            particle.age = particle.lifetime / 2;
        }
        particle
    }

    /// `FireworkParticles.FlashProvider`: a pale burst over the screen's world, of the colour it
    /// is given.
    pub(super) fn firework_flash(
        random: RandomSource,
        at: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
        data: ParticleData,
    ) -> Option<Self> {
        let ParticleData::Color(argb) = data else {
            return None;
        };
        let sprite = sprites.from_random(provider_random);
        let mut particle = Self::at(random, at, (sprites, sprite), Kind::Flash).quad();
        particle.lifetime = 4;
        let argb = argb as i32;
        (particle.r_col, particle.g_col, particle.b_col) = (
            ((argb >> 16) & 0xFF) as f32 / 255.0,
            ((argb >> 8) & 0xFF) as f32 / 255.0,
            (argb & 0xFF) as f32 / 255.0,
        );
        particle.alpha = ((argb >> 24) & 0xFF) as f32 / 255.0;
        Some(particle)
    }

    /// `SparkParticle.tick` after `SimpleAnimatedParticle.tick`: the picture and the fade, and the
    /// spark a trail leaves every other tick of its first half.
    pub(super) fn firework_spark_tick(&mut self, spark: Spark) -> Vec<Spawn> {
        if self.age > self.lifetime / 2 {
            self.alpha =
                1.0 - (self.age as f32 - (self.lifetime / 2) as f32) / self.lifetime as f32;
            if let Some(fade) = spark.fade {
                self.r_col += (fade[0] - self.r_col) * 0.2;
                self.g_col += (fade[1] - self.g_col) * 0.2;
                self.b_col += (fade[2] - self.b_col) * 0.2;
            }
        }
        if spark.trail && self.age < self.lifetime / 2 && (self.age + self.lifetime) % 2 == 0 {
            return vec![Spawn {
                type_id: particle_type::id("firework").expect("a carried type"),
                data: ParticleData::FireworkSpark(SparkOptions {
                    trail: false,
                    twinkle: spark.twinkle,
                    color: [self.r_col, self.g_col, self.b_col],
                    fade: spark.fade,
                    half_aged: true,
                }),
                override_limiter: true,
                always_show: false,
                at: (self.x, self.y, self.z),
                push: (0.0, 0.0, 0.0),
            }];
        }
        Vec::new()
    }

    pub(super) fn is_firework_starter(&self) -> bool {
        matches!(self.kind, Kind::FireworkStarter(_))
    }

    /// Whether `SparkParticle.extract` draws it now: a twinkling spark blinks after a third of its
    /// life.
    pub fn is_drawn(&self) -> bool {
        match self.kind {
            Kind::FireworkSpark(Spark { twinkle: true, .. }) => {
                self.age < self.lifetime / 3 || (self.age + self.lifetime) / 3 % 2 == 0
            }
            _ => true,
        }
    }

    /// `Starter.tick`: the sound of the blast, the sparks and flash of the explosion due, and the
    /// twinkle's sound at the end. `camera` is where the eye is.
    pub(super) fn firework_starter_tick(&mut self, camera: Vec3) -> Vec<Spawn> {
        let Kind::FireworkStarter(starter) = &self.kind else {
            return Vec::new();
        };
        let (explosions, twinkle_delay, mut life) = (
            Arc::clone(&starter.explosions),
            starter.twinkle_delay,
            starter.life,
        );
        let far = camera.distance_to_sqr_components(self.x, self.y, self.z) >= 256.0;
        let mut spawns = Vec::new();
        if life == 0 {
            let large = explosions.len() >= 3
                || explosions
                    .iter()
                    .any(|explosion| explosion.shape == Shape::LargeBall);
            let event = match (large, far) {
                (true, true) => "minecraft:entity.firework_rocket.large_blast_far",
                (true, false) => "minecraft:entity.firework_rocket.large_blast",
                (false, true) => "minecraft:entity.firework_rocket.blast_far",
                (false, false) => "minecraft:entity.firework_rocket.blast",
            };
            let pitch = 0.95 + self.random.next_float() * 0.1;
            self.firework_sound(event, pitch, camera);
        }
        if life % 2 == 0 && (life / 2) < explosions.len() as i32 {
            let explosion = &explosions[(life / 2) as usize];
            let colors = if explosion.colors.is_empty() {
                vec![BLACK]
            } else {
                explosion.colors.clone()
            };
            let shape = SparkShape {
                colors: &colors,
                fades: &explosion.fade_colors,
                trail: explosion.trail,
                twinkle: explosion.twinkle,
            };
            match explosion.shape {
                Shape::SmallBall => self.firework_ball(0.25, 2, &shape, &mut spawns),
                Shape::LargeBall => self.firework_ball(0.5, 4, &shape, &mut spawns),
                Shape::Star => self.firework_shape(0.5, &STAR_COORDS, (&shape, false), &mut spawns),
                Shape::Creeper => {
                    self.firework_shape(0.5, &CREEPER_COORDS, (&shape, true), &mut spawns);
                }
                Shape::Burst => self.firework_burst(&shape, &mut spawns),
            }
            spawns.push(Spawn {
                type_id: particle_type::id("flash").expect("a carried type"),
                data: ParticleData::Color(colors[0] as u32),
                override_limiter: true,
                always_show: false,
                at: (self.x, self.y, self.z),
                push: (0.0, 0.0, 0.0),
            });
        }
        life += 1;
        if life > self.lifetime {
            if twinkle_delay {
                let event = if far {
                    "minecraft:entity.firework_rocket.twinkle_far"
                } else {
                    "minecraft:entity.firework_rocket.twinkle"
                };
                let pitch = 0.9 + self.random.next_float() * 0.15;
                self.firework_sound(event, pitch, camera);
            }
            self.removed = true;
        }
        if let Kind::FireworkStarter(starter) = &mut self.kind {
            starter.life = life;
        }
        spawns
    }

    /// `level.playLocalSound(x, y, z, sound, AMBIENT, 20.0F, pitch, true)`.
    fn firework_sound(&mut self, event: &'static str, pitch: f32, camera: Vec3) {
        let place = (self.x, self.y, self.z);
        let distance_sqr = camera.distance_to_sqr_components(place.0, place.1, place.2);
        self.sound = Some(
            LocalSound::at(event, LocalSoundSource::Ambient, (20.0, pitch), place)
                .delayed_by(distance_sqr),
        );
    }

    /// `Starter.createParticle`: a spark thrown from the starter, tinted at random from the
    /// colours the explosion lists.
    fn firework_make(
        &mut self,
        motion: (f64, f64, f64),
        shape: &SparkShape<'_>,
        spawns: &mut Vec<Spawn>,
    ) {
        let pick = |random: &mut RandomSource, list: &[i32]| {
            list[random.next_int_bound(list.len() as i32).unwrap_or(0) as usize]
        };
        let color = channels(pick(&mut self.random, shape.colors));
        let fade = (!shape.fades.is_empty()).then(|| channels(pick(&mut self.random, shape.fades)));
        spawns.push(Spawn {
            type_id: particle_type::id("firework").expect("a carried type"),
            data: ParticleData::FireworkSpark(SparkOptions {
                trail: shape.trail,
                twinkle: shape.twinkle,
                color,
                fade,
                half_aged: false,
            }),
            override_limiter: true,
            always_show: false,
            at: (self.x, self.y, self.z),
            push: motion,
        });
    }

    /// `Starter.createParticleBall`.
    fn firework_ball(
        &mut self,
        base_speed: f64,
        steps: i32,
        shape: &SparkShape<'_>,
        spawns: &mut Vec<Spawn>,
    ) {
        let mut y_step = -steps;
        while y_step <= steps {
            let mut x_step = -steps;
            while x_step <= steps {
                let mut z_step = -steps;
                while z_step <= steps {
                    let spread = |step: i32, random: &mut RandomSource| {
                        f64::from(step) + (random.next_double() - random.next_double()) * 0.5
                    };
                    let xa = spread(x_step, &mut self.random);
                    let ya = spread(y_step, &mut self.random);
                    let za = spread(z_step, &mut self.random);
                    let len = (xa * xa + ya * ya + za * za).sqrt() / base_speed
                        + self.random.next_gaussian() * 0.05;
                    self.firework_make((xa / len, ya / len, za / len), shape, spawns);
                    if y_step != -steps && y_step != steps && x_step != -steps && x_step != steps {
                        z_step += steps * 2 - 1;
                    }
                    z_step += 1;
                }
                x_step += 1;
            }
            y_step += 1;
        }
    }

    /// `Starter.createParticleShape`: the outline of a shape, turned about the vertical in three
    /// directions.
    fn firework_shape(
        &mut self,
        base_speed: f64,
        coords: &[[f64; 2]],
        (shape, flat): (&SparkShape<'_>, bool),
        spawns: &mut Vec<Spawn>,
    ) {
        let (sx, sy) = (coords[0][0], coords[0][1]);
        self.firework_make((sx * base_speed, sy * base_speed, 0.0), shape, spawns);
        let base_angle = self.random.next_float() * std::f32::consts::PI;
        let angle_mod = if flat { 0.034 } else { 0.34 };
        for angle_step in 0..3 {
            let angle = f64::from(base_angle)
                + f64::from(angle_step as f32 * std::f32::consts::PI) * angle_mod;
            let (mut ox, mut oy) = (sx, sy);
            for coord in &coords[1..] {
                let (tx, ty) = (coord[0], coord[1]);
                let mut sub_step = 0.25;
                while sub_step <= 1.0 {
                    let mut xa = mth_lerp_f64(sub_step, ox, tx) * base_speed;
                    let ya = mth_lerp_f64(sub_step, oy, ty) * base_speed;
                    let za = xa * fdlibm_sin(angle);
                    xa *= fdlibm_cos(angle);
                    let mut flip = -1.0;
                    while flip <= 1.0 {
                        self.firework_make((xa * flip, ya, za * flip), shape, spawns);
                        flip += 2.0;
                    }
                    sub_step += 0.25;
                }
                ox = tx;
                oy = ty;
            }
        }
    }

    /// `Starter.createParticleBurst`: seventy sparks thrown up and about, with the rocket's drift.
    fn firework_burst(&mut self, shape: &SparkShape<'_>, spawns: &mut Vec<Spawn>) {
        let base_off_x = self.random.next_gaussian() * 0.05;
        let base_off_z = self.random.next_gaussian() * 0.05;
        for _ in 0..70 {
            let xa = self.xd * 0.5 + self.random.next_gaussian() * 0.15 + base_off_x;
            let za = self.zd * 0.5 + self.random.next_gaussian() * 0.15 + base_off_z;
            let ya = self.yd * 0.5 + self.random.next_double() * 0.5;
            self.firework_make((xa, ya, za), shape, spawns);
        }
    }
}

/// A rocket that went off: `ClientLevel.createFireworks`.
#[derive(Debug, Clone, PartialEq)]
pub struct FireworkLaunch {
    pub at: (f64, f64, f64),
    pub motion: (f64, f64, f64),
    pub explosions: Vec<Explosion>,
}

impl ParticleEngine {
    /// `ClientLevel.createFireworks`: a rocket with no explosion leaves a few puffs of smoke, one
    /// with some a starter that bursts them.
    pub(super) fn create_fireworks<F: FnMut() -> RandomSource>(
        &mut self,
        launch: FireworkLaunch,
        context: &mut LevelEventContext<'_, F>,
    ) {
        if launch.explosions.is_empty() {
            // The bound is drawn again for each puff, as the loop asks for it.
            let mut puffs = 0;
            while puffs < context.level_random.next_int_bound(3).unwrap_or(0) + 2 {
                let sideways = context.level_random.next_gaussian() * 0.05;
                let forwards = context.level_random.next_gaussian() * 0.05;
                self.level_add(context, "poof", launch.at, (sideways, 0.005, forwards));
                puffs += 1;
            }
        } else {
            let random = (context.particle_random)();
            self.add(Particle::firework_starter(
                random,
                (launch.at, launch.motion),
                launch.explosions,
            ));
        }
    }
}

/// What an explosion's sparks are made of.
struct SparkShape<'a> {
    colors: &'a [i32],
    fades: &'a [i32],
    trail: bool,
    twinkle: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spark(trail: bool) -> Particle {
        let sprites = Arc::new(SpriteSet::new(vec!["spark_0".into(), "spark_1".into()]));
        Particle::firework_spark(
            RandomSource::create(5),
            ((0.0, 64.0, 0.0), (0.0, 0.0, 0.0)),
            &sprites,
            SparkOptions {
                trail,
                twinkle: true,
                color: [0.5, 0.25, 0.125],
                fade: Some([1.0, 1.0, 1.0]),
                half_aged: false,
            },
        )
    }

    #[test]
    fn a_trail_leaves_a_spark_every_other_tick_of_the_first_half() {
        let mut particle = spark(true);
        let Kind::FireworkSpark(state) = particle.kind else {
            panic!("a spark");
        };
        let mut left = 0;
        while particle.age < particle.lifetime / 2 {
            let asked = particle.firework_spark_tick(state);
            if (particle.age + particle.lifetime) % 2 == 0 {
                assert_eq!(asked.len(), 1);
                let ParticleData::FireworkSpark(child) = asked[0].data else {
                    panic!("a spark");
                };
                // It is born half through its life, in the colour the parent has now, and leaves none.
                assert!(child.half_aged && !child.trail && child.twinkle);
                assert_eq!(
                    child.color,
                    [particle.r_col, particle.g_col, particle.b_col]
                );
                left += 1;
            } else {
                assert!(asked.is_empty());
            }
            particle.age += 1;
        }
        assert!(left > 0);
        // Without a trail it leaves none.
        let mut plain = spark(false);
        let Kind::FireworkSpark(state) = plain.kind else {
            panic!("a spark");
        };
        assert!(plain.firework_spark_tick(state).is_empty());
    }

    #[test]
    fn a_twinkling_spark_blinks_after_a_third_of_its_life() {
        let mut particle = spark(false);
        particle.lifetime = 60;
        particle.age = 19;
        assert!(particle.is_drawn());
        // (20 + 60) / 3 = 26, even: drawn; (21 + 60) / 3 = 27, odd: not.
        particle.age = 20;
        assert!(particle.is_drawn());
        particle.age = 21;
        assert!(!particle.is_drawn());
    }

    #[test]
    fn the_spark_keeps_the_colour_it_is_given_and_fades_toward_the_other() {
        let mut particle = spark(false);
        let Kind::FireworkSpark(state) = particle.kind else {
            panic!("a spark");
        };
        assert_eq!(
            [particle.r_col, particle.g_col, particle.b_col],
            [0.5, 0.25, 0.125]
        );
        particle.age = particle.lifetime / 2 + 1;
        particle.firework_spark_tick(state);
        assert!(particle.r_col > 0.5 && particle.b_col > 0.125);
    }
}
