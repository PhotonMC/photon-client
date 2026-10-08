use super::*;
use photon_core::java_float::{JavaF32, JavaF64};

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
        self.friction_influenced_speed.get()
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
        self.modified_friction.get()
    }

    pub fn input_impulses(&self) -> photon_core::math::Vec2 {
        photon_core::math::Vec2::new(self.xxa.get(), self.zza.get())
    }

    pub fn input_key_presses(&self) -> Input {
        self.input_key_presses
    }

    pub fn input_move_vector(&self) -> Vec2 {
        Vec2::new(
            self.input_move_vector_x.get(),
            self.input_move_vector_y.get(),
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
        (self.x_bob.get(), self.y_bob.get())
    }

    pub fn previous_bob(&self) -> (f32, f32) {
        (self.x_bob_o.get(), self.y_bob_o.get())
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

    pub(crate) fn capture_old_transform(&mut self) {
        self.old_position.x = self.position.x;
        self.old_position.y = self.position.y;
        self.old_position.z = self.position.z;
        self.old_y_rot = self.y_rot;
        self.old_x_rot = self.x_rot;
        self.old_y_head_rot = self.y_head_rot;
        self.old_y_body_rot = self.y_body_rot;
    }

    pub(crate) fn turn(&mut self, mouse_x: f64, mouse_y: f64) {
        let x_delta = mouse_y as f32 * 0.15_f32;
        let y_delta = mouse_x as f32 * 0.15_f32;
        self.x_rot = JavaF32::new(self.x_rot.get() + x_delta);
        self.y_rot = JavaF32::new(self.y_rot.get() + y_delta);
        self.x_rot = JavaF32::new(clamp_f32(self.x_rot.get(), -90.0, 90.0));
        self.old_x_rot = JavaF32::new(self.old_x_rot.get() + x_delta);
        self.old_y_rot = JavaF32::new(self.old_y_rot.get() + y_delta);
        self.old_x_rot = JavaF32::new(clamp_f32(self.old_x_rot.get(), -90.0, 90.0));
    }

    pub(crate) fn set_position(&mut self, position: (f64, f64, f64)) {
        self.position.x = JavaF64::new(position.0);
        self.position.y = JavaF64::new(position.1);
        self.position.z = JavaF64::new(position.2);
    }

    pub(crate) fn tick_head_rotation(&mut self) {
        advance_head_rotation(
            &mut self.y_head_rot,
            self.head_rotation_target,
            &mut self.head_lerp_steps,
        );
    }

    pub(crate) fn add_vanilla_boat_passenger_rotation(&mut self, delta_rotation: f32) {
        self.y_rot = JavaF32::new(self.y_rot.get() + delta_rotation);
        self.y_head_rot = JavaF32::new(self.y_head_rot.get() + delta_rotation);
    }

    pub(crate) fn new(id: i32) -> Self {
        Self {
            id,
            generation: 0,
            tick_count: 0,
            position: DVec3::ZERO,
            old_position: DVec3::ZERO,
            motion: DVec3::ZERO,
            friction_influenced_speed: JavaF32::new(0.0_f32),
            modified_friction: JavaF32::new(0.0_f32),
            xxa: JavaF32::new(0.0_f32),
            zza: JavaF32::new(0.0_f32),
            input_key_presses: Input::EMPTY,
            input_move_vector_x: JavaF32::new(0.0_f32),
            input_move_vector_y: JavaF32::new(0.0_f32),
            sprint_trigger_time: 0,
            jumping: false,
            on_climbable: false,
            in_scaffolding: false,
            suppress_sliding_down_ladder: false,
            x_bob: JavaF32::new(0.0_f32),
            y_bob: JavaF32::new(0.0_f32),
            x_bob_o: JavaF32::new(0.0_f32),
            y_bob_o: JavaF32::new(0.0_f32),
            y_rot: JavaF32::new(0.0_f32),
            x_rot: JavaF32::new(0.0_f32),
            old_y_rot: JavaF32::new(0.0_f32),
            old_x_rot: JavaF32::new(0.0_f32),
            y_head_rot: JavaF32::new(0.0_f32),
            old_y_head_rot: JavaF32::new(0.0_f32),
            y_body_rot: JavaF32::new(0.0_f32),
            old_y_body_rot: JavaF32::new(0.0_f32),
            head_rotation_target: JavaF32::new(0.0_f32),
            head_lerp_steps: 0,
            entity_data: None,
            game_mode: ClientPlayerGameMode::default(),
            abilities: ClientPlayerAbilities::default(),
            shared_flags: 0,
            living_flags: 0,
            arrow_count: 0,
            stinger_count: 0,
            effect_ambience: false,
            effect_particle_type_ids: Vec::new(),
            sleeping_pos: None,
            air_supply: DEFAULT_AIR_SUPPLY,
            custom_name_present: false,
            custom_name_visible: false,
            no_gravity: false,
            silent: false,
            pose: ClientEntityPose::Standing,
            ticks_frozen: 0,
            absorption: JavaF32::new(0.0),
            model_customisation: 0,
            score: 0,
            attributes: Vec::new(),
            effects: Vec::new(),
            health: JavaF32::new(20.0_f32),
            food: 20,
            saturation: JavaF32::new(5.0_f32),
            inventory_state_id: 0,
            inventory_slots: BTreeMap::new(),
            carried_item: None,
            open_container: None,
            vehicle_id: None,
            passenger_ids: Vec::new(),
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn vehicle_id(&self) -> Option<i32> {
        self.vehicle_id
    }
}
