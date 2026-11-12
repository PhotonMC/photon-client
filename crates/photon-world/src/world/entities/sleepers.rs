//! `LivingEntity.baseTick` on an entity's first tick: one that sleeps is put over its bed again,
//! and (26.3) stands up where there is no bed to lie in.

use crate::*;

impl ClientWorld {
    /// The first tick of a living entity the server told of a bed. `sleeping_position` answers
    /// nothing where the block is no bed to lie in, which only the newer version says; that
    /// entity stops sleeping (its pose is standing, its bed forgotten) and stays where it is. The
    /// bed block's own change and the stand-up place of a bed that gives no sleep height are not
    /// ported.
    pub(super) fn first_tick_in_bed(&mut self, entity_id: EntityId) {
        let Some(entity) = self.entities.at(entity_id) else {
            return;
        };
        if entity.tick_count != 0 || entity.living_kind().is_none() {
            return;
        }
        let Some(bed) = entity.sleeping_pos() else {
            return;
        };
        let lying = crate::block::bed::sleeping_position(self, bed);
        let Some(mut entity) = self.entities.at_mut(entity_id) else {
            return;
        };
        if let Some(at) = lying {
            entity.set_position(at);
            return;
        }
        entity.vitals.sleeping_pos = None;
        entity.status.pose = ClientEntityPose::Standing;
    }
}
