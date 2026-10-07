use super::*;

impl ClientEntity {
    /// `ElytraAnimationState.getRotX/Y/Z`: how far the wings are spread.
    pub fn elytra_rotations(&self, partial_tick: f32) -> [f32; 3] {
        self.living.elytra.rotations(partial_tick)
    }

    /// `LivingEntity.getTicksUsingItem(partialTick)`: how long the item in use has been used.
    pub fn ticks_using_item(&self, partial_tick: f32) -> f32 {
        if self.is_using_item() {
            self.living.using_item_ticks as f32 + partial_tick
        } else {
            0.0
        }
    }

    /// `LivingEntity.onKineticHit` (entity event 2): the kick of a spear's hit starts, unless one
    /// began in the last ten ticks.
    pub(crate) fn on_kinetic_hit(&mut self, game_time: u64) {
        if self
            .living
            .kinetic_hit_time
            .is_none_or(|last| game_time.saturating_sub(last) > 10)
        {
            self.living.kinetic_hit_time = Some(game_time);
        }
    }
}
