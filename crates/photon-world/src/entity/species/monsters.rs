use super::super::*;

impl ClientEntity {
    /// Whether the type is a `Zombie` or one of its subclasses, which share
    /// `DATA_BABY_ID`.
    pub fn is_zombie_type(&self) -> bool {
        photon_data::entity_type::name(self.type_id).is_some_and(|name| {
            matches!(
                name,
                "minecraft:zombie"
                    | "minecraft:husk"
                    | "minecraft:drowned"
                    | "minecraft:zombie_villager"
                    | "minecraft:zombified_piglin"
            )
        })
    }
}

impl EntityView<'_> {
    /// A copper golem's animation states.
    pub fn copper_golem(&self) -> Option<&crate::mobs::copper_golem::CopperGolemAnimations> {
        self.kind.copper_golem()
    }
}

impl EntityView<'_> {
    /// `CopperGolem.getState`.
    pub fn copper_golem_action(&self) -> crate::mobs::copper_golem::GolemAction {
        self.living.copper_golem_action
    }

    /// `CopperGolem.getWeatherState`: unaffected, exposed, weathered, oxidized (0 to 3).
    pub fn copper_golem_weather(&self) -> u8 {
        self.living.copper_golem_weather
    }

    /// `Goat.hasLeftHorn` and `hasRightHorn`.
    pub fn goat_horns(&self) -> (bool, bool) {
        (!self.living.lost_left_horn, !self.living.lost_right_horn)
    }

    /// `Goat.isScreamingGoat`.
    pub fn is_screaming_goat(&self) -> bool {
        self.living.screaming_goat
    }

    /// `Goat.getRammingXHeadRot`: how far the head is lowered, in radians.
    pub fn goat_ramming_x_head_rot(&self) -> f32 {
        let maximum = if self.status.baby { 52.5 } else { 30.0 };
        self.living.animal.goat_lower_head_tick as f32 / 20.0
            * maximum
            * (std::f64::consts::PI / 180.0) as f32
    }

    /// `Creeper.isPowered`.
    pub fn is_powered(&self) -> bool {
        self.living.powered
    }

    /// `SnowGolem.hasPumpkin`.
    pub fn has_pumpkin(&self) -> bool {
        !self.living.bare_head
    }

    /// `EnderMan.isCreepy`.
    pub fn is_creepy(&self) -> bool {
        self.living.creepy
    }

    /// `Hoglin.getAttackAnimationRemainingTicks` (a zoglin's too).
    pub fn hoglin_attack_ticks(&self) -> i32 {
        self.living.animal.hoglin_attack_ticks
    }

    /// An iron golem's `getAttackAnimationTick` and `getOfferFlowerTick`.
    pub fn iron_golem_ticks(&self) -> (i32, i32) {
        let animal = &self.living.animal;
        (animal.golem_attack_tick, animal.golem_offer_flower_tick)
    }

    /// `Raider.isCelebrating`.
    pub fn is_celebrating(&self) -> bool {
        self.living.celebrating
    }

    /// `SpellcasterIllager.isCastingSpell`.
    pub fn is_casting_spell(&self) -> bool {
        self.living.casting_spell
    }

    /// `Creeper.getSwelling`: how far the fuse has burned, between two ticks.
    pub fn creeper_swelling(&self, partial: f32) -> f32 {
        self.living.animal.creeper_swelling(partial)
    }
}

impl crate::EntityViewMut<'_> {
    /// `EnderMan.onSyncedDataUpdated` of `DATA_CREEPY`: every update of it plays the stare while
    /// the enderman has been stared at.
    pub(crate) fn set_creepy(&mut self, creepy: bool) {
        self.living.creepy = creepy;
        if self.living.stared_at {
            self.stare_sound();
        }
    }

    /// `EnderMan.playStareSound`: the sound is asked for once in four hundred ticks, and not by a
    /// silent enderman (which still counts the time).
    pub(crate) fn stare_sound(&mut self) {
        let tick_count = self.tick_count;
        if self
            .living
            .last_stare_sound
            .is_some_and(|last| tick_count < last.saturating_add(400))
        {
            return;
        }
        self.living.last_stare_sound = Some(tick_count);
        if !self.status.silent {
            self.sound_requests
                .push(crate::entity::TickSound::EndermanStare);
        }
    }
}
