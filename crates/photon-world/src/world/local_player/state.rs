use crate::*;
use photon_core::java_float::{JavaF32, JavaF64};

impl ClientWorld {
    pub fn local_player(&self) -> Option<&ClientPlayerState> {
        self.local_player.as_ref()
    }

    /// Create or replace the local player state for a clientbound player ID.
    pub fn set_local_player_id(&mut self, player_id: EntityId) {
        let mut player = ClientPlayerState::new(player_id);
        if let Some(entity) = self.entities.at(player_id) {
            player.y_head_rot = entity.y_head_rot;
            player.old_y_head_rot = entity.old_y_head_rot;
            player.head_rotation_target = entity.y_head_rot;
            player.y_body_rot = entity.y_body_rot;
            player.old_y_body_rot = entity.old_y_body_rot;
        }
        self.local_player = Some(player);
    }

    pub fn apply_local_player_position(
        &mut self,
        position: (f64, f64, f64),
        motion: (f64, f64, f64),
        rotation: (f32, f32),
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        let initialize_transform_history = player.tick_count == 0;
        player.position.x = JavaF64::new(position.0);
        player.position.y = JavaF64::new(position.1);
        player.position.z = JavaF64::new(position.2);
        player.motion.x = JavaF64::new(motion.0);
        player.motion.y = JavaF64::new(motion.1);
        player.motion.z = JavaF64::new(motion.2);
        player.y_rot = JavaF32::new(rotation.0);
        player.x_rot = JavaF32::new(rotation.1);
        if initialize_transform_history {
            player.old_position.x = player.position.x;
            player.old_position.y = player.position.y;
            player.old_position.z = player.position.z;
            player.old_y_rot = player.y_rot;
            player.old_x_rot = player.x_rot;
        }
        true
    }

    pub fn apply_local_player_health(&mut self, health: f32, food: i32, saturation: f32) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.health = JavaF32::new(health);
        player.food = food;
        player.saturation = JavaF32::new(saturation);
        true
    }

    pub fn apply_local_player_abilities(&mut self, abilities: ClientPlayerAbilities) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.abilities = ClientPlayerAbilities {
            may_build: player.abilities.may_build,
            ..abilities
        };
        true
    }

    pub fn apply_local_player_game_mode(&mut self, game_mode: ClientPlayerGameMode) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.game_mode = game_mode;
        player.abilities = game_mode.update_player_abilities(player.abilities);
        true
    }

    /// Apply one LocalPlayer input snapshot to its queryable movement state.
    pub fn apply_local_player_input(
        &mut self,
        move_vector: photon_core::math::Vec2,
        jump_pressed: bool,
        controlled_camera: bool,
        modifiers: LocalPlayerInputModifiers,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_input(move_vector, jump_pressed, controlled_camera, modifiers);
        true
    }

    /// Apply LocalPlayer input while resolving the sneaking factor from its
    /// current movement attribute state.
    pub fn apply_local_player_input_from_attributes(
        &mut self,
        move_vector: photon_core::math::Vec2,
        jump_pressed: bool,
        controlled_camera: bool,
        modifiers: LocalPlayerInputModifiers,
    ) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_input_from_attributes(move_vector, jump_pressed, controlled_camera, modifiers);
        true
    }

    /// Sample keyboard input and apply the resulting LocalPlayer input state.
    pub fn apply_local_player_keyboard_input(
        &mut self,
        key_states: Input,
        controlled_camera: bool,
        modifiers: LocalPlayerInputModifiers,
    ) -> bool {
        let mut keyboard = KeyboardInput::new(key_states);
        keyboard.tick();
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.apply_input_from_attributes(
            keyboard.move_vector(),
            keyboard.key_presses.jump,
            controlled_camera,
            modifiers,
        );
        true
    }

    pub fn recreate_local_player(&mut self, data_to_keep: u8) -> bool {
        let Some(previous) = self.local_player.take() else {
            return false;
        };
        let keep_entity_data = data_to_keep & PLAYER_RESPAWN_KEEP_ENTITY_DATA != 0;
        let keep_attribute_modifiers = data_to_keep & PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS != 0;
        let mut player = ClientPlayerState::new(previous.id);
        player.generation = previous.generation.saturating_add(1);
        player.attributes = previous
            .attributes
            .iter()
            .map(|attribute| {
                let mut attribute = attribute.clone();
                if !keep_attribute_modifiers {
                    attribute.modifiers.clear();
                }
                attribute
            })
            .collect();
        if keep_entity_data {
            player.motion.x = previous.motion.x;
            player.motion.y = previous.motion.y;
            player.motion.z = previous.motion.z;
            player.y_rot = previous.y_rot;
            player.x_rot = previous.x_rot;
            player.old_y_rot = previous.old_y_rot;
            player.old_x_rot = previous.old_x_rot;
            player.y_body_rot = previous.y_body_rot;
            player.old_y_body_rot = previous.old_y_body_rot;
            player.y_head_rot = previous.y_head_rot;
            player.old_y_head_rot = previous.old_y_head_rot;
            player.head_rotation_target = previous.head_rotation_target;
            player.head_lerp_steps = previous.head_lerp_steps;
            player.shared_flags = previous.shared_flags;
            player.living_flags = previous.living_flags;
            player.arrow_count = previous.arrow_count;
            player.stinger_count = previous.stinger_count;
            player.effect_ambience = previous.effect_ambience;
            player.effect_particle_type_ids = previous.effect_particle_type_ids;
            player.sleeping_pos = previous.sleeping_pos;
            player.air_supply = previous.air_supply;
            player.custom_name_present = previous.custom_name_present;
            player.custom_name_visible = previous.custom_name_visible;
            player.no_gravity = previous.no_gravity;
            player.silent = previous.silent;
            player.pose = previous.pose;
            player.ticks_frozen = previous.ticks_frozen;
            player.entity_data = previous.entity_data;
        } else {
            player.y_rot = JavaF32::new(-180.0_f32);
        }
        self.local_player = Some(player);
        true
    }

    /// Apply a clientbound attribute snapshot to the matching local player.
    pub fn apply_local_player_attributes(
        &mut self,
        player_id: EntityId,
        attributes: &[ClientAttributeState],
    ) -> bool {
        let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == player_id)
        else {
            return false;
        };
        for attribute in attributes {
            if let Some(existing) = player
                .attributes
                .iter_mut()
                .find(|existing| existing.attribute_id == attribute.attribute_id)
            {
                *existing = attribute.clone();
            } else {
                player.attributes.push(attribute.clone());
            }
        }
        true
    }

    pub fn apply_local_player_mob_effect(
        &mut self,
        player_id: EntityId,
        effect_id: i32,
        amplifier: i32,
        duration_ticks: i32,
        flags: u8,
    ) -> bool {
        let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == player_id)
        else {
            return false;
        };
        let effect = ClientMobEffectState {
            effect_id,
            amplifier,
            duration_ticks,
            flags,
        };
        if let Some(existing) = player
            .effects
            .iter_mut()
            .find(|existing| existing.effect_id == effect_id)
        {
            *existing = effect;
        } else {
            player.effects.push(effect);
        }
        true
    }

    /// Apply the three-tick head rotation interpolation target used by the
    /// clientbound rotate-head path for the local player.
    pub fn apply_local_player_head_rotation(&mut self, y_head_rot: f32) -> bool {
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.head_rotation_target = JavaF32::new(y_head_rot);
        player.head_lerp_steps = LIVING_ENTITY_HEAD_LERP_STEPS;
        true
    }

    pub fn remove_local_player_mob_effect(&mut self, player_id: EntityId, effect_id: i32) -> bool {
        let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == player_id)
        else {
            return false;
        };
        player
            .effects
            .retain(|effect| effect.effect_id != effect_id);
        true
    }

    /// Copy local player movement flags into a client-authoritative boat.
    pub fn set_boat_input(&mut self, vehicle_id: EntityId, input: BoatInput) -> bool {
        let Some(boat) = self.entities.at_mut(vehicle_id) else {
            return false;
        };
        if vanilla_boat_max_passengers(boat.type_id).is_none() {
            return false;
        }
        boat.boat_input = input;
        true
    }

    pub(crate) fn tick_local_player(&mut self, vehicle_id: Option<EntityId>) {
        let passenger_ids = {
            let Some(player) = self.local_player.as_mut() else {
                return;
            };
            if vehicle_id.is_some() {
                player.motion.x = JavaF64::new(0.0_f64);
                player.motion.y = JavaF64::new(0.0_f64);
                player.motion.z = JavaF64::new(0.0_f64);
            }
            player.capture_old_transform();
            player.tick_head_rotation();
            player.tick_count = player.tick_count.wrapping_add(1);
            for effect in &mut player.effects {
                effect.tick_client();
            }
            player.passenger_ids.clone()
        };
        let Some(player_id) = self.local_player.as_ref().map(|player| player.id) else {
            return;
        };
        if let Some(vehicle_id) = vehicle_id {
            self.position_entity_passenger(vehicle_id, player_id);
        }
        for passenger_id in passenger_ids {
            self.tick_entity_passenger_tree(player_id, passenger_id);
        }
    }

    /// `Cat.handleLieDown`'s search: a player who sleeps within two blocks of
    /// the cat's block (the local player, or one the server added).
    pub(crate) fn sleeping_player_near(&self, cat: &ClientEntity) -> bool {
        let (x, y, z) = cat.position();
        let (x, y, z) = (x.floor(), y.floor(), z.floor());
        let area =
            photon_core::aabb::Aabb::new(x, y, z, x + 1.0, y + 1.0, z + 1.0).inflate(2.0, 2.0, 2.0);
        let local = self.local_player.as_ref().is_some_and(|player| {
            let (x, y, z) = player.position();
            player.is_sleeping()
                && player
                    .pose()
                    .player_dimensions()
                    .make_bounding_box(Vec3::new(x, y, z))
                    .intersects(area)
        });
        local
            || self.entities.values().any(|other| {
                other.is_player() && other.is_sleeping() && other.bounding_box().intersects(area)
            })
    }

    /// The local player's own head and body turn, set to what an entity
    /// packet for it says.
    pub(crate) fn snap_local_player_turn(
        &mut self,
        entity_id: EntityId,
        head_rotation: JavaF32,
        body_yaw: f32,
    ) {
        if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity_id)
        {
            player.y_head_rot = head_rotation;
            player.old_y_head_rot = head_rotation;
            player.head_rotation_target = head_rotation;
            player.head_lerp_steps = 0;
            player.y_body_rot = JavaF32::new(body_yaw);
            player.old_y_body_rot = JavaF32::new(body_yaw);
        }
    }
}
