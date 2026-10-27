//! `Level.isRainingAt` and `Entity.isInRain`: where the rain falls on something.

use std::sync::Arc;

use photon_core::binary::BlockPos;

use crate::ClientWorld;
use crate::particles::TerrainMaterials;

/// What the biomes say of where rain falls (the client's biome tints carry it). The source is no
/// part of what a world is equal to.
#[derive(Debug, Clone, Default)]
pub(crate) struct Precipitation(Option<Arc<dyn TerrainMaterials + Send + Sync>>);

impl PartialEq for Precipitation {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for Precipitation {}

impl ClientWorld {
    /// Where the world learns what falls from the sky at a place.
    pub fn set_precipitation(&mut self, materials: Arc<dyn TerrainMaterials + Send + Sync>) {
        self.precipitation = Precipitation(Some(materials));
    }

    /// `Level.isRainingAt`, with the materials' precipitation (none: never).
    pub fn raining_at_with(&self, materials: Option<&dyn TerrainMaterials>, pos: BlockPos) -> bool {
        self.is_raining()
            && self.sky_brightness(pos.x, pos.y, pos.z) >= 15
            && self.motion_blocking_height(pos.x, pos.z) <= pos.y
            && materials.is_some_and(|materials| materials.rains_at(self, pos))
    }

    /// `Level.isRainingAt`.
    pub fn is_raining_at(&self, pos: BlockPos) -> bool {
        self.raining_at_with(
            self.precipitation
                .0
                .as_deref()
                .map(|materials| materials as &dyn TerrainMaterials),
            pos,
        )
    }

    /// `Entity.isInRain` for a box standing at `position` with its top at `top`: rain at its
    /// block, or at the block of its top.
    pub fn is_in_rain(&self, position: (f64, f64, f64), top: f64) -> bool {
        let at = |y: f64| {
            BlockPos::new(
                photon_core::math::floor_f64(position.0),
                photon_core::math::floor_f64(y),
                photon_core::math::floor_f64(position.2),
            )
        };
        self.is_raining_at(at(position.1)) || self.is_raining_at(at(top))
    }
}
