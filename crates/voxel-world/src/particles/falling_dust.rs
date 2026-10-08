//! `FallingDustParticle`: a speck of a block that drifts down and turns, coloured by the block.

use std::sync::Arc;

use voxel_core::binary::BlockPos;
use voxel_core::random::RandomSource;

use super::super::ClientWorld;
use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;
use super::terrain::TerrainMaterials;

/// The blocks drawn as nothing (`RenderShape.INVISIBLE`), bar air.
const INVISIBLE: [&str; 8] = [
    "minecraft:barrier",
    "minecraft:bubble_column",
    "minecraft:end_gateway",
    "minecraft:end_portal",
    "minecraft:light",
    "minecraft:water",
    "minecraft:lava",
    "minecraft:structure_void",
];

impl Particle {
    /// `FallingDustParticle.Provider`: nothing for a block drawn as nothing; otherwise coloured by
    /// a falling block's dust, the block's tint as a flake would take it, or its map colour.
    pub fn falling_dust_provider(
        random: impl FnOnce() -> RandomSource,
        at: (f64, f64, f64),
        (state, sprites): (i32, &Arc<SpriteSet>),
        (world, materials): (&ClientWorld, Option<&dyn TerrainMaterials>),
    ) -> Option<Self> {
        let block = crate::block::state_or_air(i64::from(state)).block();
        if !block.air && INVISIBLE.contains(&block.name) {
            return None;
        }
        let pos = BlockPos::new(
            voxel_core::math::floor_f64(at.0),
            voxel_core::math::floor_f64(at.1),
            voxel_core::math::floor_f64(at.2),
        );
        let map_color = || {
            let id = voxel_data::map::state_map_color(block.name, state as u32);
            voxel_data::map::base_color(id)
        };
        let tint = voxel_data::map::dust_color(block.name)
            .or_else(|| materials.and_then(|materials| materials.tint(world, state, pos)))
            .unwrap_or_else(map_color);
        let channel = |shift: i32| ((tint >> shift) & 0xFF) as f32 / 255.0;
        let mut particle = Self::at(
            random(),
            at,
            (sprites, sprites.first()),
            Kind::FallingDust { rot_speed: 0.0 },
        )
        .quad();
        (particle.r_col, particle.g_col, particle.b_col) = (channel(16), channel(8), channel(0));
        particle.quad_size *= 0.674_999_95;
        let base = (32.0 / (f64::from(particle.random.next_float()) * 0.8 + 0.2)) as i32;
        particle.lifetime = (base as f32 * 0.9).max(1.0) as i32;
        particle.set_sprite_from_age();
        let rot_speed = (particle.random.next_float() - 0.5) * 0.1;
        particle.kind = Kind::FallingDust { rot_speed };
        particle.roll = particle.random.next_float() * std::f64::consts::TAU as f32;
        Some(particle)
    }

    /// `FallingDustParticle.tick`: turns as it drifts down, still on the ground, no faster than a
    /// limit.
    pub(super) fn falling_dust_tick(&mut self, world: &ClientWorld, rot_speed: f32) {
        self.xo = self.x;
        self.yo = self.y;
        self.zo = self.z;
        let aged = self.age >= self.lifetime;
        self.age += 1;
        if aged {
            self.removed = true;
            return;
        }
        self.set_sprite_from_age();
        self.o_roll = self.roll;
        self.roll += std::f32::consts::PI * rot_speed * 2.0;
        if self.on_ground {
            self.o_roll = 0.0;
            self.roll = 0.0;
        }
        self.move_by(world, self.xd, self.yd, self.zd);
        self.yd -= f64::from(0.003_f32);
        self.yd = self.yd.max(f64::from(-0.14_f32));
    }
}
