use super::*;

impl ClientWorld {
    /// Apply the trapdoor and ladder block context used by LivingEntity.onClimbable.
    pub fn apply_entity_living_climbable_trapdoor_state(
        &mut self,
        entity_id: i32,
        trapdoor_open: bool,
        trapdoor_facing: &str,
        below_is_ladder: bool,
        ladder_facing: &str,
    ) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.on_climbable = trapdoor_open && below_is_ladder && trapdoor_facing == ladder_facing;
        true
    }
}
