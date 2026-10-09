//! `FallingLeavesParticle` and `LeavesBlock.animateTick`: leaves that fall from a block with room
//! below, drifting on the wind or swirling as they go.

use std::sync::Arc;

use photon_core::math::{fdlibm_cos, fdlibm_pow, fdlibm_sin};
use photon_core::random::RandomSource;
use photon_core::shape::{BooleanOp, Shapes};

use super::super::ClientWorld;
use super::engine::{ParticleData, ParticleEngine};
use super::level_events::LevelEventContext;
use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;
use super::terrain::TerrainMaterials;

/// How a leaf falls: its spin, and the wind it rides.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Leaf {
    rot_speed: f32,
    spin_acceleration: f32,
    wind_big: f32,
    swirl: bool,
    flow_away: bool,
    xa_flow_scale: f64,
    za_flow_scale: f64,
    swirl_period: f64,
}

/// `FallingLeavesParticle.INITIAL_LIFETIME`, and the age its curve ends at.
const INITIAL_LIFETIME: i32 = 300;

impl Particle {
    /// The falling leaves' providers: a cherry petal, a pale oak leaf, and a tinted leaf (with the
    /// colour its options carry); nothing for another type.
    pub fn falling_leaves_provider(
        name: &str,
        random: RandomSource,
        at: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
        (provider_random, data): (&mut RandomSource, ParticleData),
    ) -> Option<Self> {
        // Fall and side acceleration, swirl, flow away, scale, start velocity.
        let shape = match name {
            "cherry_leaves" => (0.25, 2.0, false, true, 1.0, 0.0),
            "pale_oak_leaves" | "tinted_leaves" => (0.07, 10.0, true, false, 2.0, 0.021),
            _ => return None,
        };
        let colour = match (name, data) {
            ("tinted_leaves", ParticleData::Color(color)) => Some(color),
            ("tinted_leaves", _) => return None,
            _ => None,
        };
        let sprite = sprites.from_random(provider_random);
        let mut particle = Self::falling_leaves(random, at, (sprites, sprite), shape);
        if let Some(color) = colour {
            let channel = |shift: u32| ((color >> shift) & 0xFF) as f32 / 255.0;
            (particle.r_col, particle.g_col, particle.b_col) =
                (channel(16), channel(8), channel(0));
        }
        Some(particle)
    }

    /// `FallingLeavesParticle(..)`.
    fn falling_leaves(
        random: RandomSource,
        at: (f64, f64, f64),
        sprite: (&Arc<SpriteSet>, usize),
        (fall, side, swirl, flow_away, scale, start): (f32, f32, bool, bool, f32, f32),
    ) -> Self {
        let placeholder = Kind::Suspended;
        let mut particle = Self::at(random, at, sprite, placeholder).quad();
        // The fields' own draws, after the particle's.
        let turn = |random: &mut RandomSource, degrees: f64| {
            let degrees = if random.next_boolean() {
                -degrees
            } else {
                degrees
            };
            degrees.to_radians() as f32
        };
        let rot_speed = turn(&mut particle.random, 30.0);
        let spin_acceleration = turn(&mut particle.random, 5.0);
        particle.lifetime = INITIAL_LIFETIME;
        particle.gravity = fall * 1.2 * 0.0025;
        let size = scale
            * if particle.random.next_boolean() {
                0.05
            } else {
                0.075
            };
        particle.quad_size = size;
        particle.set_size(size, size);
        particle.friction = 1.0;
        particle.yd = f64::from(-start);
        let angle = particle.random.next_float();
        let wind = f64::from(side);
        particle.kind = Kind::FallingLeaves(Leaf {
            rot_speed,
            spin_acceleration,
            wind_big: side,
            swirl,
            flow_away,
            xa_flow_scale: fdlibm_cos(f64::from(angle * 60.0).to_radians()) * wind,
            za_flow_scale: fdlibm_sin(f64::from(angle * 60.0).to_radians()) * wind,
            swirl_period: f64::from(1000.0 + angle * 3000.0).to_radians(),
        });
        particle
    }

    /// `FallingLeavesParticle.tick`: blown along the curve of its age, gone on the ground or when a
    /// wall stops it.
    pub(super) fn falling_leaves_tick(&mut self, world: &ClientWorld, mut leaf: Leaf) {
        self.xo = self.x;
        self.yo = self.y;
        self.zo = self.z;
        let expired = self.lifetime <= 0;
        self.lifetime -= 1;
        if expired {
            self.removed = true;
            return;
        }
        let alive = (INITIAL_LIFETIME - self.lifetime) as f32;
        let relative_age = (alive / INITIAL_LIFETIME as f32).min(1.0);
        let (mut xa, mut za) = (0.0, 0.0);
        if leaf.flow_away {
            let curve = fdlibm_pow(f64::from(relative_age), 1.25);
            xa += leaf.xa_flow_scale * curve;
            za += leaf.za_flow_scale * curve;
        }
        if leaf.swirl {
            let turn = f64::from(relative_age) * leaf.swirl_period;
            xa += f64::from(relative_age) * fdlibm_cos(turn) * f64::from(leaf.wind_big);
            za += f64::from(relative_age) * fdlibm_sin(turn) * f64::from(leaf.wind_big);
        }
        self.xd += xa * f64::from(0.0025_f32);
        self.zd += za * f64::from(0.0025_f32);
        self.yd -= f64::from(self.gravity);
        leaf.rot_speed += leaf.spin_acceleration / 20.0;
        self.o_roll = self.roll;
        self.roll += leaf.rot_speed / 20.0;
        self.kind = Kind::FallingLeaves(leaf);
        self.move_by(world, self.xd, self.yd, self.zd);
        if self.on_ground || self.lifetime < 299 && (self.xd == 0.0 || self.zd == 0.0) {
            self.removed = true;
            return;
        }
        self.xd *= f64::from(self.friction);
        self.yd *= f64::from(self.friction);
        self.zd *= f64::from(self.friction);
    }
}

impl ParticleEngine {
    /// `LeavesBlock.animateTick`: a leaf falls by the block's chance where the block below has no
    /// full top. (A drip in the rain, `isRainingAt`, is not carried.)
    pub(super) fn leaves_animate_tick<F: FnMut() -> RandomSource>(
        &mut self,
        (state, name): (i32, &str),
        (x, y, z): (i32, i32, i32),
        random: &mut RandomSource,
        (context, materials): (&mut LevelEventContext<'_, F>, Option<&dyn TerrainMaterials>),
    ) {
        let Some((chance, particle, colour)) = photon_data::leaves::of(name) else {
            return;
        };
        if random.next_float() >= chance {
            return;
        }
        let world = context.world;
        let below = world.block_state_at(x, y - 1, z).unwrap_or(0);
        let shape = crate::block::registry::collision_shape_at(below as u32, [x, y - 1, z]);
        let top = shape.face_shape(photon_core::direction::Direction::Up);
        if !Shapes::join_is_not_empty(&Shapes::block(), &top, BooleanOp::NotSame) {
            return;
        }
        let (particle, data) = if particle == "tinted" {
            // `ClientLevel.getClientLeafTintColor`: the block's tint where it is, white without.
            let pos = photon_core::binary::BlockPos::new(x, y, z);
            let tint = materials
                .and_then(|materials| materials.tint(world, state, pos))
                .unwrap_or(-1);
            ("tinted_leaves", ParticleData::Color(tint as u32))
        } else {
            let name = particle.strip_prefix("minecraft:").unwrap_or(particle);
            let data = colour.map_or(ParticleData::Plain, |color| {
                ParticleData::Color(color as u32)
            });
            (name, data)
        };
        // `ParticleUtils.spawnParticleBelow`.
        let at = (
            f64::from(x) + random.next_double(),
            f64::from(y) - 0.05,
            f64::from(z) + random.next_double(),
        );
        self.level_add_data(context, particle, data, at);
    }
}
