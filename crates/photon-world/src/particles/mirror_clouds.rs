//! The clouds an invisible illusioner leaves when its mirror images move (`Illusioner.aiStep`).

use photon_core::random::RandomSource;

use super::engine::ParticleEngine;
use super::level_events::LevelEventContext;

impl ParticleEngine {
    /// Sixteen clouds at random places in the entity's box, standing still.
    pub(super) fn mirror_clouds<F: FnMut() -> RandomSource>(
        &mut self,
        burst: crate::mobs::burst::EntityBurst,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let (x, y, z) = burst.at;
        let (width, height) = (f64::from(burst.size.0), f64::from(burst.size.1));
        for _ in 0..16 {
            let random = &mut *context.level_random;
            let at = (
                x + (2.0 * random.next_double() - 1.0) * 0.5 * width,
                y + height * random.next_double(),
                z + (2.0 * random.next_double() - 1.0) * 0.5 * width,
            );
            self.level_add(context, burst.particle, at, (0.0, 0.0, 0.0));
        }
    }
}
