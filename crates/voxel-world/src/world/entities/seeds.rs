use crate::*;

impl ClientWorld {
    /// Fixes the source a horse's tail chance is drawn from; false for an
    /// entity that is not there.
    pub fn fix_horse_seed(&mut self, entity_id: i32, seed: i64) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.fix_horse_seed(seed);
        true
    }

    /// Fixes the source a panda's chance to eat is drawn from; false for an
    /// entity that is not there.
    pub fn fix_panda_seed(&mut self, entity_id: i32, seed: i64) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.fix_panda_seed(seed);
        true
    }

    /// Fixes a rabbit's idle head tilt wait and random source (see
    /// [`rabbit::RabbitState::fix_idle`]); false for an entity that is not there.
    pub fn fix_rabbit_idle(&mut self, entity_id: i32, timeout: i32, seed: i64) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.fix_rabbit_idle(timeout, seed);
        true
    }

    /// Fixes where a guardian's tail starts and the source its spikes are drawn
    /// from out of water (see [`guardian::GuardianState::fix_random`]); false
    /// for an entity that is not there.
    pub fn fix_guardian_random(&mut self, entity_id: i32, tail: f32, seed: i64) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity
            .kind
            .guardian_or_insert_with(|| Box::new(mobs::guardian::GuardianState::new()))
            .fix_random(tail, seed);
        true
    }

    /// Fixes a squid's tentacle speed (see [`squid::SquidState::fix_tentacle_speed`]);
    /// false for an entity that is not there.
    pub fn fix_squid_tentacle_speed(&mut self, entity_id: i32, speed: f32) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity
            .kind
            .squid_or_insert_with(|| Box::new(mobs::squid::SquidState::new()))
            .fix_tentacle_speed(speed);
        true
    }

    /// Fixes the source a copper golem's idle wait is drawn from (see
    /// [`copper_golem::CopperGolemAnimations::fix_seed`]); false for an entity that is not there.
    pub fn fix_copper_golem_seed(&mut self, entity_id: i32, seed: i64) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity
            .kind
            .copper_golem_or_insert_with(|| {
                Box::new(mobs::copper_golem::CopperGolemAnimations::new())
            })
            .fix_seed(seed);
        true
    }
}
