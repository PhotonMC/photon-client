//! `DripParticle` and its kinds: a drop that hangs, then falls, then lands, each stage asking
//! the level for the next (`DripHangParticle`, `FallAndLandParticle`, `DripLandParticle`).

use std::sync::Arc;

use voxel_core::math::floor_f64;
use voxel_core::random::RandomSource;
use voxel_data::block::definition::Fluid;

use super::super::ClientWorld;
use super::super::local_player::{LocalSound, LocalSoundSource};
use super::engine::{ParticleData, Spawn};
use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

/// What a drop does as it goes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Stage {
    /// `DripHangParticle`: clings, then lets the falling one go (a cooling one reddens first).
    Hang {
        falling: &'static str,
        cooling: bool,
    },
    /// `FallingParticle` and the ones that land: gone on the ground, leaving what lands there
    /// and the sound it makes.
    Fall {
        land: Option<&'static str>,
        sound: Option<&'static str>,
    },
    /// `DripLandParticle`: lies where it fell.
    Land,
}

/// A drop: the fluid it vanishes into, its stage, and whether it glows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Drip {
    fluid: Fluid,
    pub(super) stage: Stage,
    pub(super) glowing: bool,
}

/// The drop colours the providers set.
const WATER: [f32; 3] = [0.2, 0.3, 1.0];
const LAVA: [f32; 3] = [1.0, 0.285_714_3, 0.083_333_336];
const TEAR: [f32; 3] = [0.511_718_75, 0.031_25, 0.890_625];

impl Particle {
    /// The drip providers, by type; nothing for another type.
    pub fn drip_provider(
        name: &str,
        random: RandomSource,
        at: (f64, f64, f64),
        sprites: &Arc<SpriteSet>,
        provider_random: &mut RandomSource,
    ) -> Option<Self> {
        let (fluid, stage, colour, glowing) = drip_of(name)?;
        let sprite = sprites.from_random(provider_random);
        let drip = Drip {
            fluid,
            stage,
            glowing,
        };
        // `DripParticle(..)`: a speck that falls at its own pace.
        let mut particle = Self::at(random, at, (sprites, sprite), Kind::Drip(drip)).quad();
        particle.set_size(0.01, 0.01);
        particle.gravity = 0.06;
        match stage {
            Stage::Hang { .. } => {
                particle.gravity *= 0.02;
                particle.lifetime = 40;
            }
            Stage::Fall { land: Some(_), .. } => {
                particle.lifetime =
                    (64.0 / (f64::from(particle.random.next_float()) * 0.8 + 0.2)) as i32;
            }
            Stage::Land => {
                particle.lifetime =
                    (16.0 / (f64::from(particle.random.next_float()) * 0.8 + 0.2)) as i32;
            }
            Stage::Fall { land: None, .. } => {}
        }
        // What each provider sets after.
        match name {
            "dripping_honey" | "dripping_obsidian_tear" => {
                particle.gravity *= 0.01;
                particle.lifetime = 100;
            }
            "falling_honey" | "falling_obsidian_tear" => particle.gravity = 0.01,
            "landing_honey" => {
                particle.lifetime =
                    (128.0 / (f64::from(provider_random.next_float()) * 0.8 + 0.2)) as i32;
            }
            "landing_obsidian_tear" => {
                particle.lifetime =
                    (28.0 / (f64::from(provider_random.next_float()) * 0.8 + 0.2)) as i32;
            }
            "falling_nectar" => {
                particle.lifetime =
                    (16.0 / (f64::from(provider_random.next_float()) * 0.8 + 0.2)) as i32;
                particle.gravity = 0.007;
            }
            "falling_spore_blossom" => {
                // `Mth.randomBetween(particle.random, 0.1F, 0.9F)`.
                let between = particle.random.next_float() * 0.8 + 0.1;
                particle.lifetime = (64.0 / between) as i32;
                particle.gravity = 0.005;
            }
            _ => {}
        }
        if let Some([r, g, b]) = colour {
            (particle.r_col, particle.g_col, particle.b_col) = (r, g, b);
        }
        Some(particle)
    }

    /// `DripParticle.tick`, with its stage's `preMoveUpdate` and `postMoveUpdate`: what it asks
    /// the level for (the next stage), and the sound a landing makes.
    pub(super) fn drip_tick(&mut self, world: &ClientWorld, drip: Drip) -> Vec<Spawn> {
        self.xo = self.x;
        self.yo = self.y;
        self.zo = self.z;
        let mut spawns = Vec::new();
        if let Stage::Hang { cooling: true, .. } = drip.stage {
            // `CoolingDripHangParticle.preMoveUpdate`.
            self.r_col = 1.0;
            self.g_col = 16.0 / (40 - self.lifetime + 16) as f32;
            self.b_col = 4.0 / (40 - self.lifetime + 8) as f32;
        }
        let expired = self.lifetime <= 0;
        self.lifetime -= 1;
        if expired {
            self.removed = true;
            if let Stage::Hang { falling, .. } = drip.stage {
                spawns.push(plain(
                    falling,
                    (self.x, self.y, self.z),
                    (self.xd, self.yd, self.zd),
                ));
            }
        }
        if self.removed {
            return spawns;
        }
        self.yd -= f64::from(self.gravity);
        self.move_by(world, self.xd, self.yd, self.zd);
        match drip.stage {
            Stage::Hang { .. } => {
                self.xd *= 0.02;
                self.yd *= 0.02;
                self.zd *= 0.02;
            }
            Stage::Fall { land, sound } if self.on_ground => {
                self.removed = true;
                if let Some(land) = land {
                    spawns.push(plain(land, (self.x, self.y, self.z), (0.0, 0.0, 0.0)));
                }
                if let Some(sound) = sound {
                    // `Mth.randomBetween(this.random, 0.3F, 1.0F)`.
                    let volume = self.random.next_float() * 0.7 + 0.3;
                    self.sound = Some(LocalSound {
                        event: sound,
                        source: LocalSoundSource::Blocks,
                        volume,
                        pitch: 1.0,
                        position: (self.x, self.y, self.z),
                    });
                }
            }
            _ => {}
        }
        if self.removed {
            return spawns;
        }
        self.xd *= f64::from(0.98_f32);
        self.yd *= f64::from(0.98_f32);
        self.zd *= f64::from(0.98_f32);
        if drip.fluid != Fluid::Empty {
            let pos = [self.x, self.y, self.z].map(floor_f64);
            let state = world.block_state_at(pos[0], pos[1], pos[2]).unwrap_or(0);
            let fluid = crate::block::state_or_air(i64::from(state)).fluid;
            if !fluid.is_empty() && fluid.is_same(drip.fluid) {
                // `FluidState.getHeight`: full under the same fluid.
                let above = world
                    .block_state_at(pos[0], pos[1] + 1, pos[2])
                    .unwrap_or(0);
                let above = crate::block::state_or_air(i64::from(above)).fluid;
                let height = if !above.is_empty() && above.is_same(drip.fluid) {
                    1.0
                } else {
                    fluid.own_height()
                };
                if self.y < f64::from(pos[1]) + f64::from(height) {
                    self.removed = true;
                }
            }
        }
        spawns
    }
}

/// `Level.addParticle` of a plain type.
fn plain(name: &str, at: (f64, f64, f64), push: (f64, f64, f64)) -> Spawn {
    Spawn {
        type_id: voxel_data::particle_type::id(name).expect("a carried type"),
        data: ParticleData::Plain,
        override_limiter: false,
        always_show: false,
        at,
        push,
    }
}

/// A drip type's fluid, stage, colour (where its provider sets one) and glow.
fn drip_of(name: &str) -> Option<(Fluid, Stage, Option<[f32; 3]>, bool)> {
    use Fluid::{Empty, Lava, Water};
    let hang = |falling, cooling| Stage::Hang { falling, cooling };
    let fall = |land, sound| Stage::Fall { land, sound };
    Some(match name {
        "dripping_lava" => (Lava, hang("falling_lava", true), None, false),
        "falling_lava" => (Lava, fall(Some("landing_lava"), None), Some(LAVA), false),
        "landing_lava" => (Lava, Stage::Land, Some(LAVA), false),
        "dripping_water" => (Water, hang("falling_water", false), Some(WATER), false),
        "falling_water" => (Water, fall(Some("splash"), None), Some(WATER), false),
        "dripping_honey" => (
            Empty,
            hang("falling_honey", false),
            Some([0.622, 0.508, 0.082]),
            false,
        ),
        "falling_honey" => (
            Empty,
            fall(Some("landing_honey"), Some("minecraft:block.beehive.drip")),
            Some([0.582, 0.448, 0.082]),
            false,
        ),
        "landing_honey" => (Empty, Stage::Land, Some([0.522, 0.408, 0.082]), false),
        "falling_nectar" => (Empty, fall(None, None), Some([0.92, 0.782, 0.72]), false),
        "falling_spore_blossom" => (Empty, fall(None, None), Some([0.32, 0.5, 0.22]), false),
        "dripping_obsidian_tear" => (
            Empty,
            hang("falling_obsidian_tear", false),
            Some(TEAR),
            true,
        ),
        "falling_obsidian_tear" => (
            Empty,
            fall(Some("landing_obsidian_tear"), None),
            Some(TEAR),
            true,
        ),
        "landing_obsidian_tear" => (Empty, Stage::Land, Some(TEAR), true),
        "dripping_dripstone_water" => (
            Water,
            hang("falling_dripstone_water", false),
            Some(WATER),
            false,
        ),
        "falling_dripstone_water" => (
            Water,
            fall(
                Some("splash"),
                Some("minecraft:block.pointed_dripstone.drip_water"),
            ),
            Some(WATER),
            false,
        ),
        "dripping_dripstone_lava" => (Lava, hang("falling_dripstone_lava", true), None, false),
        "falling_dripstone_lava" => (
            Lava,
            fall(
                Some("landing_lava"),
                Some("minecraft:block.pointed_dripstone.drip_lava"),
            ),
            Some(LAVA),
            false,
        ),
        _ => return None,
    })
}
