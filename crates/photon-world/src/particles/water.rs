//! The particles of water: `BubbleParticle`, `BubbleColumnUpParticle`, `BubblePopParticle`,
//! `WaterDropParticle` (rain) and `SplashParticle`, `WakeParticle` (a fishing bobber's trail),
//! `WaterCurrentDownParticle` and `SquidInkParticle`.

use photon_core::ids::BlockStateId;
use std::sync::Arc;

use photon_core::direction::Axis;
use photon_core::math::{mth_cos_f64, mth_sin_f64};
use photon_core::random::RandomSource;

use super::super::ClientWorld;
use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

/// What a bubble's or a drop's family adds to its kind.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Water {
    /// `BubbleParticle`: counts its life down and goes out of the water.
    Bubble,
    /// `BubbleColumnUpParticle`: the plain tick, then out of the water it goes.
    ColumnUp,
    /// `BubblePopParticle`: a picture for each of its four ticks.
    Pop,
    /// `WaterDropParticle` and `SplashParticle`: falls, may go on landing, and goes inside a
    /// block's shape or a fluid.
    Drop,
    /// `WakeParticle`: grows and turns through four pictures.
    Wake,
    /// `WaterCurrentDownParticle`: spirals down, turning by `angle`.
    CurrentDown { angle: f32 },
    /// `SquidInkParticle` on `SimpleAnimatedParticle`: fades in its second half, sinks in air.
    Ink,
}

/// The block position holding a point (`BlockPos.containing`).
fn containing((x, y, z): (f64, f64, f64)) -> [i32; 3] {
    [x, y, z].map(photon_core::math::floor_f64)
}

fn fluid_at(world: &ClientWorld, [x, y, z]: [i32; 3]) -> crate::block::fluid::FluidState {
    crate::block::state_or_air(i64::from(world.block_state_at(x, y, z).unwrap_or(0))).fluid
}

/// `getFluidState(pos).is(FluidTags.WATER)`.
fn in_water(world: &ClientWorld, at: (f64, f64, f64)) -> bool {
    let fluid = fluid_at(world, containing(at));
    !fluid.is_empty() && fluid.is_same(photon_data::block::definition::Fluid::Water)
}

impl Particle {
    /// The bubbles' shared making: a tiny box, a smaller quad, a fifth of the push and a jitter.
    fn bubble_of(
        random: RandomSource,
        at: (f64, f64, f64),
        push: (f64, f64, f64),
        sprite: (&Arc<SpriteSet>, usize),
        water: Water,
    ) -> Self {
        let mut particle = Self::at(random, at, sprite, Kind::Water(water)).quad();
        if water == Water::ColumnUp {
            particle.gravity = -0.125;
            particle.friction = 0.85;
        }
        particle.set_size(0.02, 0.02);
        particle.quad_size *= particle.random.next_float() * 0.6 + 0.2;
        let jitter = |push: f64, random: &mut RandomSource| {
            push * f64::from(0.2_f32) + f64::from((random.next_float() * 2.0 - 1.0) * 0.02)
        };
        particle.xd = jitter(push.0, &mut particle.random);
        particle.yd = jitter(push.1, &mut particle.random);
        particle.zd = jitter(push.2, &mut particle.random);
        let length = if water == Water::ColumnUp { 40.0 } else { 8.0 };
        particle.lifetime = (length / (f64::from(particle.random.next_float()) * 0.8 + 0.2)) as i32;
        particle
    }

    /// `BubbleParticle.Provider`, its picture from the provider's random.
    pub fn bubble(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
    ) -> Self {
        let sprite = sprites.from_random(provider_random);
        Self::bubble_of(random, at, push, (sprites, sprite), Water::Bubble)
    }

    /// `BubbleColumnUpParticle.Provider`.
    pub fn bubble_column_up(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
    ) -> Self {
        let sprite = sprites.from_random(provider_random);
        Self::bubble_of(random, at, push, (sprites, sprite), Water::ColumnUp)
    }

    /// `BubblePopParticle.Provider`: four ticks, falling a little.
    pub fn bubble_pop(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
    ) -> Self {
        let mut particle = Self::at(
            random,
            at,
            (sprites, sprites.first()),
            Kind::Water(Water::Pop),
        )
        .quad();
        particle.lifetime = 4;
        particle.gravity = 0.008;
        (particle.xd, particle.yd, particle.zd) = push;
        particle.set_sprite_from_age();
        particle
    }

    /// `WaterDropParticle(..)`: thrown up a little, a speck.
    fn drop_of(
        random: RandomSource,
        at: (f64, f64, f64),
        sprite: (&Arc<SpriteSet>, usize),
    ) -> Self {
        let mut particle = Self::thrown(
            random,
            at,
            (0.0, 0.0, 0.0),
            sprite,
            Kind::Water(Water::Drop),
        )
        .quad();
        particle.xd *= f64::from(0.3_f32);
        particle.yd = f64::from(particle.random.next_float() * 0.2 + 0.1);
        particle.zd *= f64::from(0.3_f32);
        particle.set_size(0.01, 0.01);
        particle.gravity = 0.06;
        particle.lifetime = (8.0 / (f64::from(particle.random.next_float()) * 0.8 + 0.2)) as i32;
        particle
    }

    /// `WaterDropParticle.Provider` (`rain`).
    pub fn rain(
        random: RandomSource,
        at: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
    ) -> Self {
        let sprite = sprites.from_random(provider_random);
        Self::drop_of(random, at, (sprites, sprite))
    }

    /// `SplashParticle.Provider`: a drop that falls slower, thrown sideways when pushed only so.
    pub fn splash(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
    ) -> Self {
        let sprite = sprites.from_random(provider_random);
        let mut particle = Self::drop_of(random, at, (sprites, sprite));
        particle.gravity = 0.04;
        if push.1 == 0.0 && (push.0 != 0.0 || push.2 != 0.0) {
            (particle.xd, particle.yd, particle.zd) = (push.0, 0.1, push.2);
        }
        particle
    }

    /// `WakeParticle.Provider` (`fishing`): its push as it was given.
    pub fn wake(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
    ) -> Self {
        let mut particle = Self::thrown(
            random,
            at,
            (0.0, 0.0, 0.0),
            (sprites, sprites.first()),
            Kind::Water(Water::Wake),
        )
        .quad();
        particle.xd *= f64::from(0.3_f32);
        particle.yd = f64::from(particle.random.next_float() * 0.2 + 0.1);
        particle.zd *= f64::from(0.3_f32);
        particle.set_size(0.01, 0.01);
        particle.lifetime = (8.0 / (f64::from(particle.random.next_float()) * 0.8 + 0.2)) as i32;
        particle.set_sprite_from_age();
        particle.gravity = 0.0;
        (particle.xd, particle.yd, particle.zd) = push;
        particle
    }

    /// `WaterCurrentDownParticle.Provider`: a speck that spirals down through the water.
    pub fn current_down(
        random: RandomSource,
        at: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
    ) -> Self {
        let sprite = sprites.from_random(provider_random);
        let mut particle = Self::at(
            random,
            at,
            (sprites, sprite),
            Kind::Water(Water::CurrentDown { angle: 0.0 }),
        )
        .quad();
        particle.lifetime = (particle.random.next_float() * 60.0) as i32 + 30;
        particle.has_physics = false;
        (particle.xd, particle.yd, particle.zd) = (0.0, -0.05, 0.0);
        particle.set_size(0.02, 0.02);
        particle.quad_size *= particle.random.next_float() * 0.6 + 0.2;
        particle.gravity = 0.002;
        particle
    }

    /// `SquidInkParticle.Provider` (black) and `GlowInkProvider` (`glow`: a pale teal).
    pub fn squid_ink(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        sprites: &Arc<SpriteSet>,
        glow: bool,
    ) -> Self {
        let mut particle = Self::at(
            random,
            at,
            (sprites, sprites.first()),
            Kind::Water(Water::Ink),
        )
        .quad();
        particle.friction = 0.92;
        particle.quad_size = 0.5;
        particle.alpha = 1.0;
        // `ARGB.colorFromFloat(1, 0.2, 0.8, 0.6)` back to floats: each channel cut to eight bits.
        (particle.r_col, particle.g_col, particle.b_col) = if glow {
            (51.0 / 255.0, 204.0 / 255.0, 153.0 / 255.0)
        } else {
            (0.0, 0.0, 0.0)
        };
        particle.lifetime =
            (particle.quad_size * 12.0 / (particle.random.next_float() * 0.8 + 0.2)) as i32;
        particle.set_sprite_from_age();
        particle.has_physics = false;
        (particle.xd, particle.yd, particle.zd) = push;
        particle
    }

    /// The tick of a particle of water.
    pub(super) fn water_tick(&mut self, world: &ClientWorld, water: Water) {
        match water {
            Water::ColumnUp => {
                self.base_tick(world);
                if !self.removed && !in_water(world, (self.x, self.y, self.z)) {
                    self.removed = true;
                }
                return;
            }
            Water::Ink => {
                self.base_tick(world);
                self.animated_fade();
                if !self.removed {
                    self.animated_fade();
                    let [x, y, z] = containing((self.x, self.y, self.z));
                    let state = world.block_state_at(x, y, z).unwrap_or(0);
                    if super::terrain::is_air(super::terrain::name_of(state)) {
                        self.yd -= f64::from(0.0074_f32);
                    }
                }
                return;
            }
            _ => {}
        }
        self.xo = self.x;
        self.yo = self.y;
        self.zo = self.z;
        match water {
            Water::Bubble => {
                let life = self.lifetime;
                self.lifetime -= 1;
                if life <= 0 {
                    self.removed = true;
                    return;
                }
                self.yd += 0.002;
                self.move_by(world, self.xd, self.yd, self.zd);
                self.scale_motion(0.85);
                if !in_water(world, (self.x, self.y, self.z)) {
                    self.removed = true;
                }
            }
            Water::Pop => {
                let age = self.age;
                self.age += 1;
                if age >= self.lifetime {
                    self.removed = true;
                    return;
                }
                self.yd -= f64::from(self.gravity);
                self.move_by(world, self.xd, self.yd, self.zd);
                self.set_sprite_from_age();
            }
            Water::Drop => self.drop_tick(world),
            Water::Wake => {
                let life = 60 - self.lifetime;
                let left = self.lifetime;
                self.lifetime -= 1;
                if left <= 0 {
                    self.removed = true;
                    return;
                }
                self.yd -= f64::from(self.gravity);
                self.move_by(world, self.xd, self.yd, self.zd);
                self.scale_motion(0.98);
                let size = life as f32 * 0.001;
                self.set_size(size, size);
                self.sprite = self.sprites.from_age(life % 4, 4);
            }
            Water::CurrentDown { angle } => {
                let age = self.age;
                self.age += 1;
                if age >= self.lifetime {
                    self.removed = true;
                    return;
                }
                self.xd += f64::from(0.6 * mth_cos_f64(f64::from(angle)));
                self.zd += f64::from(0.6 * mth_sin_f64(f64::from(angle)));
                self.xd *= 0.07;
                self.zd *= 0.07;
                self.move_by(world, self.xd, self.yd, self.zd);
                if !in_water(world, (self.x, self.y, self.z)) || self.on_ground {
                    self.removed = true;
                }
                self.kind = Kind::Water(Water::CurrentDown {
                    angle: angle + 0.08,
                });
            }
            Water::ColumnUp | Water::Ink => {}
        }
    }

    /// `WaterDropParticle.tick`.
    fn drop_tick(&mut self, world: &ClientWorld) {
        let left = self.lifetime;
        self.lifetime -= 1;
        if left <= 0 {
            self.removed = true;
            return;
        }
        self.yd -= f64::from(self.gravity);
        self.move_by(world, self.xd, self.yd, self.zd);
        self.scale_motion(0.98);
        if self.on_ground {
            if self.random.next_float() < 0.5 {
                self.removed = true;
            }
            self.xd *= f64::from(0.7_f32);
            self.zd *= f64::from(0.7_f32);
        }
        let pos = containing((self.x, self.y, self.z));
        let [bx, by, bz] = pos;
        let state = world.block_state_at(bx, by, bz).unwrap_or(0);
        let shape = crate::block::registry::collision_shape_at(BlockStateId(state.max(0)), pos);
        let shape_top = shape.max_at(Axis::Y, self.x - f64::from(bx), self.z - f64::from(bz));
        // `FluidState.getHeight`: the whole block under the same fluid, else its own height.
        let fluid = fluid_at(world, pos);
        let fluid_height = if fluid.is_empty() {
            0.0
        } else if fluid_at(world, [bx, by + 1, bz]).is_same(fluid.fluid) {
            1.0
        } else {
            f64::from(fluid.own_height())
        };
        let offset = shape_top.max(fluid_height);
        if offset > 0.0 && self.y < f64::from(by) + offset {
            self.removed = true;
        }
    }

    /// The motion times `factor` (an `f32` the reference writes as such) on each axis.
    fn scale_motion(&mut self, factor: f32) {
        self.xd *= f64::from(factor);
        self.yd *= f64::from(factor);
        self.zd *= f64::from(factor);
    }

    /// `SimpleAnimatedParticle.tick`'s picture and fade: in its second half it grows clearer.
    pub(super) fn animated_fade(&mut self) {
        self.set_sprite_from_age();
        if self.age > self.lifetime / 2 {
            self.alpha =
                1.0 - (self.age as f32 - (self.lifetime / 2) as f32) / self.lifetime as f32;
        }
    }
}
