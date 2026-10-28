//! What a resource reload does to the particles.

use super::Particles;

impl Particles {
    /// A resource reload keeps the particles alive: this one, freshly loaded, takes the live
    /// state of the one it replaces and keeps its own sprite sets.
    pub(crate) fn carry_over(&mut self, old: &mut Self) {
        self.engine.carry_over(&mut old.engine);
        for (new, old) in [
            (&mut self.listener_random, &mut old.listener_random),
            (&mut self.level_random, &mut old.level_random),
            (&mut self.animate_random, &mut old.animate_random),
        ] {
            std::mem::swap(new, old);
        }
        std::mem::swap(&mut self.uniquifier, &mut old.uniquifier);
        std::mem::swap(&mut self.dripstone, &mut old.dripstone);
        std::mem::swap(&mut self.local_player, &mut old.local_player);
        std::mem::swap(&mut self.packets_heard, &mut old.packets_heard);
        std::mem::swap(&mut self.most_alive, &mut old.most_alive);
    }
}
