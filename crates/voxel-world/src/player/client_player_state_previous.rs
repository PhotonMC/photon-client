use super::*;

impl ClientPlayerState {
    pub fn passenger_ids(&self) -> &[i32] {
        &self.passenger_ids
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn tick_count(&self) -> i32 {
        self.tick_count
    }

    pub fn position(&self) -> (f64, f64, f64) {
        (
            self.position.x.get(),
            self.position.y.get(),
            self.position.z.get(),
        )
    }

    pub fn previous_position(&self) -> (f64, f64, f64) {
        (
            self.old_position.x.get(),
            self.old_position.y.get(),
            self.old_position.z.get(),
        )
    }

    pub fn interpolated_position(&self, partial_tick: f32) -> (f64, f64, f64) {
        let partial_tick = f64::from(partial_tick);
        let current = self.position();
        let previous = self.previous_position();
        (
            mth_lerp_f64(partial_tick, previous.0, current.0),
            mth_lerp_f64(partial_tick, previous.1, current.1),
            mth_lerp_f64(partial_tick, previous.2, current.2),
        )
    }

    pub fn motion(&self) -> (f64, f64, f64) {
        (
            self.motion.x.get(),
            self.motion.y.get(),
            self.motion.z.get(),
        )
    }

    pub fn friction_influenced_speed(&self) -> f32 {
        f32::from_bits(self.friction_influenced_speed_bits)
    }

    /// Return Java Player.getSpeed from the local movement-speed attribute.
    pub fn movement_speed(&self) -> f32 {
        self.attributes
            .iter()
            .find(|attribute| attribute.attribute_id == VANILLA_PLAYER_MOVEMENT_SPEED_ATTRIBUTE_ID)
            .map(|attribute| attribute.effective_value(0.0, 1024.0) as f32)
            .unwrap_or(VANILLA_PLAYER_DEFAULT_MOVEMENT_SPEED)
    }

    /// Return Java's effective LocalPlayer sneaking-speed attribute value.
    pub fn sneaking_movement_factor_value(&self) -> f64 {
        self.attributes
            .iter()
            .find(|attribute| attribute.attribute_id == VANILLA_PLAYER_SNEAKING_SPEED_ATTRIBUTE_ID)
            .map(|attribute| attribute.effective_value(0.0, 1.0))
            .unwrap_or(VANILLA_PLAYER_DEFAULT_SNEAKING_SPEED)
    }

    /// Return the f32 value used by LocalPlayer.modifyInput.
    pub fn sneaking_movement_factor(&self) -> f32 {
        self.sneaking_movement_factor_value() as f32
    }

    pub fn modified_friction(&self) -> f32 {
        f32::from_bits(self.modified_friction_bits)
    }

    pub fn input_impulses(&self) -> voxel_core::math::Vec2 {
        voxel_core::math::Vec2::new(f32::from_bits(self.xxa_bits), f32::from_bits(self.zza_bits))
    }

    pub fn input_key_presses(&self) -> Input {
        self.input_key_presses
    }

    pub fn input_move_vector(&self) -> Vec2 {
        Vec2::new(
            f32::from_bits(self.input_move_vector_x_bits),
            f32::from_bits(self.input_move_vector_y_bits),
        )
    }

    pub fn sprint_trigger_time(&self) -> i32 {
        self.sprint_trigger_time
    }

    pub fn is_jumping(&self) -> bool {
        self.jumping
    }

    pub fn is_on_climbable(&self) -> bool {
        self.on_climbable
    }

    pub fn bob(&self) -> (f32, f32) {
        (
            f32::from_bits(self.x_bob_bits),
            f32::from_bits(self.y_bob_bits),
        )
    }

    pub fn previous_bob(&self) -> (f32, f32) {
        (
            f32::from_bits(self.x_bob_o_bits),
            f32::from_bits(self.y_bob_o_bits),
        )
    }

    pub fn rotation(&self) -> (f32, f32) {
        (self.y_rot.get(), self.x_rot.get())
    }

    pub fn previous_rotation(&self) -> (f32, f32) {
        (self.old_y_rot.get(), self.old_x_rot.get())
    }

    pub fn head_rotation(&self) -> f32 {
        self.y_head_rot.get()
    }

    pub fn body_y_rotation(&self) -> f32 {
        self.y_body_rot.get()
    }

    pub fn previous_body_y_rotation(&self) -> f32 {
        self.old_y_body_rot.get()
    }

    pub(crate) fn clamp_vanilla_boat_passenger_rotation(&mut self, vehicle_y_rot: f32) {
        let (old_y_rot, y_rot) = vanilla_boat_passenger_rotation_clamped_values(
            self.y_rot.get(),
            self.old_y_rot.get(),
            vehicle_y_rot,
        );
        self.old_y_rot = JavaF32::new(old_y_rot);
        self.y_rot = JavaF32::new(y_rot);
        self.y_head_rot = self.y_rot;
        self.y_body_rot = JavaF32::new(vehicle_y_rot);
    }

    pub fn previous_head_rotation(&self) -> f32 {
        self.old_y_head_rot.get()
    }

    pub fn interpolated_head_rotation(&self, partial_tick: f32) -> f32 {
        if partial_tick == 1.0 {
            return self.head_rotation();
        }
        mth_rot_lerp_f32(
            partial_tick,
            self.previous_head_rotation(),
            self.head_rotation(),
        )
    }

    pub fn interpolated_rotation(&self, partial_tick: f32) -> (f32, f32) {
        let current = self.rotation();
        if partial_tick == 1.0 {
            return current;
        }
        let previous = self.previous_rotation();
        (
            mth_rot_lerp_f32(partial_tick, previous.0, current.0),
            mth_lerp_f32(partial_tick, previous.1, current.1),
        )
    }

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

    pub fn pose(&self) -> ClientEntityPose {
        self.pose
    }

    /// Return player/Avatar dimensions derived from the current pose.
    pub fn player_dimensions(&self) -> ClientEntityDimensions {
        self.pose.player_dimensions()
    }

    pub fn ticks_frozen(&self) -> i32 {
        self.ticks_frozen
    }

    pub fn absorption(&self) -> f32 {
        f32::from_bits(self.absorption_bits)
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

    pub fn attributes(&self) -> &[ClientAttributeState] {
        &self.attributes
    }

    pub fn effects(&self) -> &[ClientMobEffectState] {
        &self.effects
    }

    pub fn health(&self) -> f32 {
        f32::from_bits(self.health_bits)
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

    pub fn effect_particle_type_ids(&self) -> &[i32] {
        &self.effect_particle_type_ids
    }

    pub fn sleeping_pos(&self) -> Option<voxel_core::binary::BlockPos> {
        self.sleeping_pos
    }

    pub fn is_sleeping(&self) -> bool {
        self.sleeping_pos.is_some()
    }

    pub fn food(&self) -> i32 {
        self.food
    }

    pub fn saturation(&self) -> f32 {
        f32::from_bits(self.saturation_bits)
    }

    pub fn inventory_state_id(&self) -> i32 {
        self.inventory_state_id
    }

    pub fn inventory_slot(&self, slot: i16) -> Option<&ClientItemStackState> {
        self.inventory_slots.get(&slot).and_then(Option::as_ref)
    }

    pub fn inventory_slot_count(&self) -> usize {
        self.inventory_slots.len()
    }

    pub fn carried_item(&self) -> Option<&ClientItemStackState> {
        self.carried_item.as_ref()
    }

    /// A container menu's last 36 slots are the player's own inventory (the
    /// 27 of the main inventory, then the hotbar), so what the container
    /// says of them is what the inventory holds.
    pub(crate) fn mirror_container_into_inventory(&mut self) {
        let Some(container) = self.open_container.as_ref() else {
            return;
        };
        let Some(top) = container.top_slot_count else {
            return;
        };
        for index in 0..36 {
            let from = (top + index) as i16;
            let Some(stack) = container.slots.get(&from) else {
                continue;
            };
            // Main inventory slots 9 to 35 keep their menu numbers, and the
            // hotbar 0 to 8 is the player menu's 36 to 44.
            let to = if index < 27 {
                9 + index
            } else {
                36 + index - 27
            };
            self.inventory_slots.insert(to as i16, stack.clone());
        }
    }

    pub fn open_container(&self) -> Option<&ClientContainerState> {
        self.open_container.as_ref()
    }
}
