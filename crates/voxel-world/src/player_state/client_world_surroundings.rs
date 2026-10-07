use super::*;

impl ClientWorld {
    /// Where the living entity of an id is and how tall its eyes are, if the world holds one.
    pub(in super::super) fn living_target(&self, id: i32) -> Option<guardian::GuardianTarget> {
        if let Some(player) = self.local_player.as_ref().filter(|p| p.id() == id) {
            let dimensions = player.player_dimensions();
            return Some(guardian::GuardianTarget {
                position: player.position(),
                height: dimensions.height(),
                eye_height: dimensions.eye_height(),
            });
        }
        let target = self.entities.get(&id)?;
        target.living_kind()?;
        let dimensions = target.dimensions().scale(target.scale(), target.scale());
        Some(guardian::GuardianTarget {
            position: target.position(),
            height: dimensions.height(),
            eye_height: dimensions.eye_height(),
        })
    }

    pub(in super::super) fn species_surroundings(&self, entity_id: i32) -> (bool, bool) {
        let sleeper_near_cat = self
            .entities
            .get(&entity_id)
            .is_some_and(|entity| entity.is_lying_cat() && self.sleeping_player_near(entity));
        (sleeper_near_cat, self.is_thundering())
    }
}
