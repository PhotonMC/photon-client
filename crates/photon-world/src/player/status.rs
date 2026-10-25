use super::*;

impl ClientPlayerState {
    pub fn entity_data(&self) -> Option<&[u8]> {
        self.entity_data.as_deref()
    }

    pub fn abilities(&self) -> ClientPlayerAbilities {
        self.abilities
    }

    pub fn game_mode(&self) -> ClientPlayerGameMode {
        self.game_mode
    }

    pub fn shared_flags(&self) -> u8 {
        self.shared_flags
    }

    pub fn living_flags(&self) -> u8 {
        self.living_flags
    }

    pub fn is_using_item(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_USING_ITEM)
    }

    pub fn used_item_is_off_hand(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_OFF_HAND)
    }

    pub fn is_auto_spin_attack(&self) -> bool {
        shared_flag_is_set(self.living_flags, LIVING_FLAG_SPIN_ATTACK)
    }

    pub fn air_supply(&self) -> i32 {
        self.air_supply
    }

    pub fn has_custom_name(&self) -> bool {
        self.custom_name_present
    }

    pub fn is_custom_name_visible(&self) -> bool {
        self.custom_name_visible
    }

    pub fn should_show_name(&self) -> bool {
        true
    }

    pub fn is_no_gravity(&self) -> bool {
        self.no_gravity
    }

    pub fn is_silent(&self) -> bool {
        self.silent
    }

    pub fn ticks_frozen(&self) -> i32 {
        self.ticks_frozen
    }

    pub fn absorption(&self) -> f32 {
        self.absorption.get()
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    pub fn model_customisation(&self) -> u8 {
        self.model_customisation
    }

    pub fn percent_frozen(&self) -> f32 {
        (self.ticks_frozen.min(DEFAULT_TICKS_REQUIRED_TO_FREEZE) as f32)
            / DEFAULT_TICKS_REQUIRED_TO_FREEZE as f32
    }

    pub fn is_fully_frozen(&self) -> bool {
        self.ticks_frozen >= DEFAULT_TICKS_REQUIRED_TO_FREEZE
    }

    pub fn is_freezing(&self) -> bool {
        self.ticks_frozen > 0
    }

    pub fn is_on_fire(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_ON_FIRE)
    }

    pub fn is_shift_key_down(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SHIFT_KEY_DOWN)
    }

    pub fn is_sprinting(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SPRINTING)
    }

    pub fn is_swimming(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_SWIMMING)
    }

    pub fn is_invisible(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_INVISIBLE)
    }

    pub fn is_glowing(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_GLOWING)
    }

    pub fn is_fall_flying(&self) -> bool {
        shared_flag_is_set(self.shared_flags, SHARED_FLAG_FALL_FLYING)
    }

    /// `LivingEntity.getAttributeValue` of the local player.
    pub fn attribute_value(&self, attribute: &str) -> Option<f64> {
        crate::effective_attribute(&self.attributes, "minecraft:player", attribute)
    }

    pub fn attributes(&self) -> &[ClientAttributeState] {
        &self.attributes
    }

    pub fn effects(&self) -> &[ClientMobEffectState] {
        &self.effects
    }

    /// The tick count moves on and each effect ticks, as `LivingEntity.tickEffects` does on a
    /// client, and the sleep counter with them (`Player.tick`).
    pub(crate) fn tick_effects(&mut self) {
        self.tick_sleep_timer();
        self.tick_count = self.tick_count.wrapping_add(1);
        for effect in &mut self.effects {
            effect.tick_client();
        }
    }

    pub fn health(&self) -> f32 {
        self.health.get()
    }

    pub fn arrow_count(&self) -> i32 {
        self.arrow_count
    }

    pub fn stinger_count(&self) -> i32 {
        self.stinger_count
    }

    pub fn is_effect_ambient(&self) -> bool {
        self.effect_ambience
    }

    pub fn effect_particles(&self) -> &[crate::EffectParticle] {
        &self.effect_particles
    }

    pub fn sleeping_pos(&self) -> Option<photon_core::binary::BlockPos> {
        self.sleep.pos
    }

    pub fn is_sleeping(&self) -> bool {
        self.sleep.pos.is_some()
    }

    /// `Player.getSleepTimer`.
    pub fn sleep_timer(&self) -> i32 {
        self.sleep.timer
    }

    /// The sleep counter's part of `Player.tick`: it counts up to a hundred while asleep, and on
    /// from there to a hundred and ten after waking, then starts over.
    pub(crate) fn tick_sleep_timer(&mut self) {
        if self.is_sleeping() {
            self.sleep.timer = (self.sleep.timer + 1).min(100);
        } else if self.sleep.timer > 0 {
            self.sleep.timer += 1;
            if self.sleep.timer >= 110 {
                self.sleep.timer = 0;
            }
        }
    }

    /// `Player.stopSleepInBed` as the client runs it: the entity data's sleeping position is
    /// emptied and the counter goes to a hundred, or to zero for a forceful wake-up.
    pub fn stop_sleep_in_bed(&mut self, forceful: bool) {
        self.sleep.pos = None;
        self.sleep.timer = if forceful { 0 } else { 100 };
    }
}

#[cfg(test)]
mod sleep_timer_tests {
    use super::*;

    #[test]
    fn the_sleep_timer_climbs_to_a_hundred_then_runs_out_in_ten_ticks_after_waking() {
        let mut player = ClientPlayerState::new(photon_core::ids::EntityId(1));
        player.sleep.pos = Some(photon_core::binary::BlockPos { x: 0, y: 64, z: 0 });
        for _ in 0..130 {
            player.tick_sleep_timer();
        }
        assert_eq!(player.sleep_timer(), 100);
        player.stop_sleep_in_bed(false);
        assert_eq!(player.sleep_timer(), 100);
        let mut seen = Vec::new();
        for _ in 0..11 {
            player.tick_sleep_timer();
            seen.push(player.sleep_timer());
        }
        assert_eq!(seen, [101, 102, 103, 104, 105, 106, 107, 108, 109, 0, 0]);
        player.sleep.pos = Some(photon_core::binary::BlockPos { x: 0, y: 64, z: 0 });
        player.tick_sleep_timer();
        player.stop_sleep_in_bed(true);
        assert_eq!(player.sleep_timer(), 0);
    }
}
