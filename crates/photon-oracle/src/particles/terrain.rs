//! The block materials a particle row's engine is given: one blank terrain sprite, the tints of
//! the few blocks the rows use, and the plains' rain.

use super::*;
use photon_world::particles::TerrainMaterials;

/// What a row's blocks look like to the particles made of them.
#[derive(Debug)]
pub(super) struct OracleTerrain;

impl TerrainMaterials for OracleTerrain {
    /// The scene's biome is the plains: warm enough for rain at any height of the scene.
    fn rains_at(&self, _world: &ClientWorld, _pos: photon_core::binary::BlockPos) -> bool {
        true
    }

    fn particle_texture(&self, _state: BlockStateId) -> String {
        "block/stone".into()
    }

    fn tint(
        &self,
        _world: &ClientWorld,
        state: BlockStateId,
        _pos: photon_core::binary::BlockPos,
    ) -> Option<i32> {
        // The tint sources of these blocks do not read the biome (`BlockColors`).
        match photon_world::block::state_or_air(state).block().name {
            "minecraft:spruce_leaves" => Some(-10_380_959),
            "minecraft:birch_leaves" => Some(-8_345_771),
            // `grassBlock().colorAsTerrainParticle`: none.
            "minecraft:grass_block" => Some(-1),
            _ => None,
        }
    }
}
