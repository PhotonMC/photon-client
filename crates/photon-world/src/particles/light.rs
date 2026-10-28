//! The light a particle is drawn with: `LightCoordsUtil`.

use photon_core::binary::BlockPos;

use super::super::ClientWorld;
use super::super::block::state_or_air;

/// `LightCoordsUtil.FULL_BRIGHT`.
const FULL_BRIGHT: i32 = 15_728_880;

/// `LightCoordsUtil.pack`.
fn pack(block: i32, sky: i32) -> i32 {
    (block << 4) | (sky << 20)
}

/// `LightCoordsUtil.getLightCoords(level, pos)`: the block and sky light at the position, the
/// block's own glow where it is brighter, or full where the block is drawn glowing.
pub(super) fn light_coords(world: &ClientWorld, pos: BlockPos) -> i32 {
    let state = state_or_air(
        world
            .block_state_at(pos.x, pos.y, pos.z)
            .unwrap_or(crate::constants::VANILLA_AIR_BLOCK_STATE_ID),
    );
    if state.emissive_rendering {
        return FULL_BRIGHT;
    }
    let block = i32::from(world.block_brightness(pos.x, pos.y, pos.z));
    let sky = i32::from(world.sky_brightness(pos.x, pos.y, pos.z));
    let emission = i32::from(state.light_emission);
    let packed = pack(block, sky);
    if block < emission {
        (packed & 0xFF_0000) | (emission << 4)
    } else {
        packed
    }
}

impl ClientWorld {
    /// `LightCoordsUtil.getLightCoords(level, pos)`, which block entities are lit by too.
    pub fn block_light_coords(&self, x: i32, y: i32, z: i32) -> i32 {
        light_coords(self, BlockPos::new(x, y, z))
    }
}

/// What `Particle.getLightCoords` answers for a position: `ClientLevel.hasChunk` says yes for
/// every chunk, so it is always the level's light.
pub(super) fn particle_light(world: &ClientWorld, x: f64, y: f64, z: f64) -> i32 {
    use photon_core::math::floor_f64;
    let pos = BlockPos::new(floor_f64(x), floor_f64(y), floor_f64(z));
    light_coords(world, pos)
}

/// `LightCoordsUtil.addSmoothBlockEmission`.
pub(super) fn add_smooth_block_emission(light_coords: i32, emission: f32) -> i32 {
    let emission = photon_core::math::clamp_f32(emission, 0.0, 1.0);
    let emitted = (photon_core::math::clamp_f32(emission, 0.0, 1.0) * 240.0) as i32;
    let block = ((light_coords & 0xFF) + emitted).min(240);
    block & 0xFF | ((light_coords >> 16) & 0xFF) << 16
}
