//! `TerrainParticle`: a flake of a block, and the effects that spawn them.

use std::sync::Arc;

use photon_core::binary::BlockPos;
use photon_core::direction::Direction;
use photon_core::math::ceil_f64;
use photon_core::random::RandomSource;

use super::super::ClientWorld;
use super::particle::{Kind, Particle};
use super::sprites::SpriteSet;

/// What the pictures and the colours of a block's flakes come from: the block models and tint
/// sources the renderer owns.
pub trait TerrainMaterials: std::fmt::Debug {
    /// `BlockStateModelSet.getParticleMaterial(state).sprite()`: the texture (under `textures/`) of
    /// the block atlas the model names as its particle.
    fn particle_texture(&self, state: i32) -> String;

    /// `BlockTintSource.colorAsTerrainParticle` of the state's first tint source, if it has one.
    fn tint(&self, world: &ClientWorld, state: i32, pos: BlockPos) -> Option<i32>;

    /// The particle textures of an item's model layers as it lies on the ground
    /// (`ItemStackRenderState` layers' `particleMaterial`), in layer order.
    fn item_particle_textures(&self, _item: i32) -> Vec<String> {
        Vec::new()
    }
}

/// `BlockBehaviour.BlockStateBase.isAir`.
pub(super) fn is_air(name: &str) -> bool {
    matches!(
        name,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
}

/// `BlockBehaviour.Properties.noTerrainParticles`: the blocks that set it.
fn spawns_terrain_particles(name: &str) -> bool {
    !matches!(name, "minecraft:barrier" | "minecraft:structure_void")
}

pub(super) fn name_of(state: i32) -> &'static str {
    crate::block::state_or_air(i64::from(state)).block().name
}

/// `TerrainParticle.createTerrainParticle`'s test of the state.
pub(super) fn makes_terrain_particles(state: i32) -> bool {
    let name = name_of(state);
    !is_air(name) && name != "minecraft:moving_piston" && spawns_terrain_particles(name)
}

impl Particle {
    /// `TerrainParticle(level, x, y, z, xa, ya, za, state, pos)`.
    pub fn terrain(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        (state, pos): (i32, BlockPos),
        (world, materials): (&ClientWorld, &dyn TerrainMaterials),
    ) -> Self {
        let sprites = Arc::new(SpriteSet::new(vec![materials.particle_texture(state)]));
        let mut particle = Self::thrown(
            random,
            at,
            push,
            (&sprites, 0),
            Kind::Terrain { uo: 0.0, vo: 0.0 },
        )
        .quad();
        particle.gravity = 1.0;
        particle.r_col = 0.6;
        particle.g_col = 0.6;
        particle.b_col = 0.6;
        if let Some(colour) = materials.tint(world, state, pos) {
            particle.r_col *= ((colour >> 16) & 0xFF) as f32 / 255.0;
            particle.g_col *= ((colour >> 8) & 0xFF) as f32 / 255.0;
            particle.b_col *= (colour & 0xFF) as f32 / 255.0;
        }
        particle.quad_size /= 2.0;
        let uo = particle.random.next_float() * 3.0;
        let vo = particle.random.next_float() * 3.0;
        particle.kind = Kind::Terrain { uo, vo };
        particle
    }

    /// `TerrainParticle.Provider`: nothing for air, a piston in motion or a block with no flakes.
    pub fn terrain_provider(
        random: RandomSource,
        (at, push): ((f64, f64, f64), (f64, f64, f64)),
        state: i32,
        env: (&ClientWorld, &dyn TerrainMaterials),
    ) -> Option<Self> {
        makes_terrain_particles(state).then(|| {
            let pos = BlockPos::new(
                photon_core::math::floor_f64(at.0),
                photon_core::math::floor_f64(at.1),
                photon_core::math::floor_f64(at.2),
            );
            Self::terrain(random, (at, push), (state, pos), env)
        })
    }

    /// `TerrainParticle.CrumblingProvider`: still, and gone within ten ticks.
    pub fn crumbling_provider(
        random: RandomSource,
        at: ((f64, f64, f64), (f64, f64, f64)),
        state: i32,
        env: (&ClientWorld, &dyn TerrainMaterials),
        engine_random: &mut RandomSource,
    ) -> Option<Self> {
        let mut particle = Self::terrain_provider(random, at, state, env)?;
        particle.xd = 0.0;
        particle.yd = 0.0;
        particle.zd = 0.0;
        particle.lifetime = engine_random
            .next_int_bound(10)
            .expect("the bound is positive")
            + 1;
        Some(particle)
    }

    /// `TerrainParticle.DustPillarProvider`: thrown up, with a sideways spread, for a second or two.
    pub fn dust_pillar_provider(
        random: RandomSource,
        at: ((f64, f64, f64), (f64, f64, f64)),
        state: i32,
        env: (&ClientWorld, &dyn TerrainMaterials),
        engine_random: &mut RandomSource,
    ) -> Option<Self> {
        let y_aux = at.1.1;
        let mut particle = Self::terrain_provider(random, at, state, env)?;
        particle.xd = engine_random.next_gaussian() / 30.0;
        particle.yd = y_aux + engine_random.next_gaussian() / 2.0;
        particle.zd = engine_random.next_gaussian() / 30.0;
        particle.lifetime = engine_random
            .next_int_bound(20)
            .expect("the bound is positive")
            + 20;
        Some(particle)
    }
}

/// `ClientLevel.addDestroyBlockEffect`: the flakes of a block that broke, one in each cell of a
/// grid over each box of its shape, moving away from the middle of the box.
pub(super) fn destroy_block_effect(
    (pos, state): (BlockPos, i32),
    (world, materials): (&ClientWorld, &dyn TerrainMaterials),
    mut particle_random: impl FnMut() -> RandomSource,
) -> Vec<Particle> {
    let mut flakes = Vec::new();
    if !makes_terrain_particles_of_destroyed(state) {
        return flakes;
    }
    let shape = crate::outline_shape(state as u32, [pos.x, pos.y, pos.z]);
    shape.for_all_boxes(|x1, y1, z1, x2, y2, z2| {
        let width_x = (x2 - x1).min(1.0);
        let width_y = (y2 - y1).min(1.0);
        let width_z = (z2 - z1).min(1.0);
        let count_x = 2.max(ceil_f64(width_x / 0.25));
        let count_y = 2.max(ceil_f64(width_y / 0.25));
        let count_z = 2.max(ceil_f64(width_z / 0.25));
        for xx in 0..count_x {
            for yy in 0..count_y {
                for zz in 0..count_z {
                    let rel_x = (f64::from(xx) + 0.5) / f64::from(count_x);
                    let rel_y = (f64::from(yy) + 0.5) / f64::from(count_y);
                    let rel_z = (f64::from(zz) + 0.5) / f64::from(count_z);
                    let at = (
                        f64::from(pos.x) + (rel_x * width_x + x1),
                        f64::from(pos.y) + (rel_y * width_y + y1),
                        f64::from(pos.z) + (rel_z * width_z + z1),
                    );
                    flakes.push(Particle::terrain(
                        particle_random(),
                        (at, (rel_x - 0.5, rel_y - 0.5, rel_z - 0.5)),
                        (state, pos),
                        (world, materials),
                    ));
                }
            }
        }
    });
    flakes
}

/// `!blockState.isAir() && blockState.shouldSpawnTerrainParticles()`.
fn makes_terrain_particles_of_destroyed(state: i32) -> bool {
    let name = name_of(state);
    !is_air(name) && spawns_terrain_particles(name)
}

/// `ClientLevel.addBreakingBlockEffect`: one flake on the face of the block being broken that the
/// player looks at, thrown softly and small.
pub(super) fn breaking_block_effect(
    (pos, direction): (BlockPos, Direction),
    (world, materials): (&ClientWorld, &dyn TerrainMaterials),
    level_random: &mut RandomSource,
    particle_random: impl FnOnce() -> RandomSource,
) -> Option<Particle> {
    let state = world.block_state_at(pos.x, pos.y, pos.z).unwrap_or(0);
    let name = name_of(state);
    if crate::shadow::is_invisible_render_shape(name) || !spawns_terrain_particles(name) {
        return None;
    }
    let shape = crate::outline_shape(state as u32, [pos.x, pos.y, pos.z]);
    let bounds = shape.bounds()?;
    let (x, y, z) = (f64::from(pos.x), f64::from(pos.y), f64::from(pos.z));
    let inset = f64::from(0.2_f32);
    let margin = f64::from(0.1_f32);
    let mut xp = x
        + level_random.next_double() * (bounds.max_x - bounds.min_x - inset)
        + margin
        + bounds.min_x;
    let mut yp = y
        + level_random.next_double() * (bounds.max_y - bounds.min_y - inset)
        + margin
        + bounds.min_y;
    let mut zp = z
        + level_random.next_double() * (bounds.max_z - bounds.min_z - inset)
        + margin
        + bounds.min_z;
    match direction {
        Direction::Down => yp = y + bounds.min_y - margin,
        Direction::Up => yp = y + bounds.max_y + margin,
        Direction::North => zp = z + bounds.min_z - margin,
        Direction::South => zp = z + bounds.max_z + margin,
        Direction::West => xp = x + bounds.min_x - margin,
        Direction::East => xp = x + bounds.max_x + margin,
    }
    let mut particle = Particle::terrain(
        particle_random(),
        ((xp, yp, zp), (0.0, 0.0, 0.0)),
        (state, pos),
        (world, materials),
    );
    particle.set_power(0.2);
    particle.scale(0.6);
    Some(particle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_makes_no_flakes() {
        for name in [
            "minecraft:air",
            "minecraft:cave_air",
            "minecraft:moving_piston",
            "minecraft:barrier",
            "minecraft:structure_void",
        ] {
            let state = photon_data::block::registry::block(name)
                .unwrap()
                .default_state_id() as i32;
            assert!(!makes_terrain_particles(state), "{name}");
        }
        let stone = photon_data::block::registry::block("minecraft:stone")
            .unwrap()
            .default_state_id() as i32;
        assert!(makes_terrain_particles(stone));
    }
}
