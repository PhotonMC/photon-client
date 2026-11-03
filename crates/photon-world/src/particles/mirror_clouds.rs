//! The clouds an invisible illusioner leaves when its mirror images move (`Illusioner.aiStep`).

use photon_core::random::RandomSource;

use super::engine::ParticleEngine;
use super::level_events::LevelEventContext;

impl ParticleEngine {
    /// Sixteen clouds at the places the illusioner's own random gave, standing still.
    pub(super) fn mirror_clouds<F: FnMut() -> RandomSource>(
        &mut self,
        burst: crate::mobs::burst::EntityBurst,
        context: &mut LevelEventContext<'_, F>,
    ) {
        for at in burst
            .mirror
            .iter()
            .flat_map(|clouds| clouds.iter().copied())
        {
            self.level_add(context, burst.particle, at, (0.0, 0.0, 0.0));
        }
    }
}
