use super::super::*;

impl ClientEntity {
    /// `Rabbit.getVariant`'s id: zero is brown (an id the variants lack is brown).
    pub fn rabbit_variant(&self) -> i32 {
        self.living.rabbit_type
    }

    /// `MushroomCow.getVariant`'s id: zero is red, one brown (an id past the
    /// last takes the last).
    pub fn mooshroom_variant(&self) -> i32 {
        self.living.mooshroom_type.clamp(0, 1)
    }

    /// `Sheep.getColor`: the dye's id (`DyeColor.getId`).
    pub fn sheep_dye_id(&self) -> u8 {
        self.living.animal.wool & 15
    }

    /// `Sheep.isSheared`.
    pub fn is_sheared(&self) -> bool {
        self.living.animal.wool & 16 != 0
    }

    /// `Sheep.getHeadEatPositionScale` and `getHeadEatAngleScale` at a point
    /// between two ticks, with the pitch the sheep is drawn at, in degrees.
    pub fn sheep_head_eat_scales(&self, partial: f32, pitch: f32) -> (f32, f32) {
        let tick = self.living.animal.eat_tick;
        (
            crate::mobs::animal::head_eat_position_scale(tick, partial),
            crate::mobs::animal::head_eat_angle_scale(tick, partial, pitch),
        )
    }

    /// The registry id of the variant (`PigVariant`, `CowVariant`) the server
    /// set, or none while it wears the default.
    pub fn variant(&self) -> Option<i32> {
        self.living.variant
    }

    /// Whether a piglin dances (`Piglin.isDancing`).
    pub fn is_dancing(&self) -> bool {
        self.living.dancing
    }

    /// `Villager.getVillagerData`: the registry ids of its type and profession
    /// and its level; a plains villager without a profession, at level one,
    /// until the server says otherwise (`createDefaultVillagerData`).
    pub fn villager_data(&self) -> [i32; 3] {
        self.living.villager_data.unwrap_or([2, 0, 1])
    }

    /// Whether a villager or a wandering trader shakes its head
    /// (`getUnhappyCounter() > 0`).
    pub fn is_unhappy(&self) -> bool {
        self.living.unhappy_counter > 0
    }

    /// Whether an enderman carries a block (`getCarriedBlock() != null`).
    pub fn is_carrying_block(&self) -> bool {
        self.living.carried_state != 0
    }

    /// A rabbit's hop and animation states, if it has ticked.
    pub fn rabbit(&self) -> Option<&crate::mobs::rabbit::RabbitState> {
        self.kind.rabbit()
    }

    /// Fixes the idle head tilt's wait and random source of a rabbit (see
    /// [`crate::mobs::rabbit::RabbitState::fix_idle`]).
    pub fn fix_rabbit_idle(&mut self, timeout: i32, seed: i64) {
        self.kind
            .rabbit_or_insert_with(|| Box::new(crate::mobs::rabbit::RabbitState::new()))
            .fix_idle(timeout, seed);
    }
}
