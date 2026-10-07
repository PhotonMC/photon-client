use super::*;

impl ClientWorld {
    pub fn apply_local_player_inventory_slot(
        &mut self,
        slot: i32,
        item: Option<&ClientItemStackState>,
    ) -> bool {
        let Some(menu_slot) = player_container_menu_slot(slot) else {
            return false;
        };
        let Some(player) = self.local_player.as_mut() else {
            return false;
        };
        player.inventory_slots.insert(menu_slot, item.cloned());
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
        player_id: i32,
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

    /// `ClientPacketListener.handleUpdateAttributes` for an entity that is not
    /// the local player: an attribute the type has takes the server's base
    /// value and modifiers; one it does not have is ignored.
    pub fn apply_entity_attributes(
        &mut self,
        entity_id: i32,
        attributes: &[ClientAttributeState],
    ) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        let Some(type_name) = voxel_data::entity_type::name(entity.type_id) else {
            return false;
        };
        for attribute in attributes {
            let known = voxel_data::entity_type::attribute_by_id(attribute.attribute_id)
                .is_some_and(|range| {
                    voxel_data::entity_type::default_attribute(type_name, range.name).is_some()
                });
            if !known {
                continue;
            }
            if let Some(existing) = entity
                .attributes
                .iter_mut()
                .find(|existing| existing.attribute_id == attribute.attribute_id)
            {
                *existing = attribute.clone();
            } else {
                entity.attributes.push(attribute.clone());
            }
        }
        true
    }

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
            .guardian_or_insert_with(|| Box::new(guardian::GuardianState::new()))
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
            .squid_or_insert_with(|| Box::new(squid::SquidState::new()))
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
            .copper_golem_or_insert_with(|| Box::new(copper_golem::CopperGolemAnimations::new()))
            .fix_seed(seed);
        true
    }

    /// Apply Java LivingEntity.setSprinting to the local player state.
    pub fn apply_local_player_sprinting(&mut self, player_id: i32, sprinting: bool) -> bool {
        let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == player_id)
        else {
            return false;
        };
        player.apply_sprinting(sprinting);
        true
    }

    pub fn apply_local_player_mob_effect(
        &mut self,
        player_id: i32,
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

    /// Apply the three-tick head rotation interpolation target from a
    /// clientbound RotateHead packet to an entity in the client world.
    pub fn apply_entity_head_rotation(&mut self, entity_id: i32, y_head_rot: f32) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if entity.living_kind().is_some() {
            entity.head_rotation_target = JavaF32::new(y_head_rot);
            entity.head_lerp_steps = LIVING_ENTITY_HEAD_LERP_STEPS;
        } else {
            entity.y_head_rot = JavaF32::new(y_head_rot);
        }
        if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity_id)
        {
            player.head_rotation_target = JavaF32::new(y_head_rot);
            player.head_lerp_steps = LIVING_ENTITY_HEAD_LERP_STEPS;
        }
        true
    }

    pub fn remove_local_player_mob_effect(&mut self, player_id: i32, effect_id: i32) -> bool {
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

    pub fn replace_dimension_with_environment(
        &mut self,
        input: DimensionInput<'_>,
        bounds: DimensionBounds,
        environment: DimensionEnvironment,
    ) -> bool {
        let changed = self.dimension_key != input.key;
        // A new level has its own light engine.
        if changed
            || self.dimension.min_y != bounds.min_y
            || self.dimension.height != bounds.height
            || self.dimension_environment.has_skylight != environment.has_skylight
        {
            self.light = new_light_engine(bounds, environment.has_skylight);
            self.light_tasks.clear();
            self.light_dirty.clear();
        }
        self.dimension = bounds;
        self.dimension_key = input.key.to_owned();
        self.dimension_type_id = input.dimension_type_id;
        self.seed = input.seed;
        self.is_debug = input.is_debug;
        self.is_flat = input.is_flat;
        self.sea_level = input.sea_level;
        self.dimension_environment = environment;
        if changed {
            let server_view_radius = self
                .chunk_cache
                .as_ref()
                .map(|cache| cache.server_view_radius);
            self.chunks.clear();
            self.chunk_cache = server_view_radius.map(ClientChunkCacheState::new);
            self.entities.clear();
            self.maps.clear();
            if let Some(player) = self.local_player.as_mut() {
                player.vehicle_id = None;
                player.passenger_ids.clear();
            }
            self.world_border = ClientWorldBorder::default();
            self.game_time = 0;
            // A new level starts with no predictions and no cracks.
            self.predictions = prediction::BlockPredictions::new();
            self.destroying_blocks.clear();
        }
        changed
    }

    pub fn game_time(&self) -> u64 {
        self.game_time
    }

    /// `Level.setRainLevel` and `setThunderLevel`: the weather the server sent
    /// (the level starts dry).
    pub fn set_weather(&mut self, rain_level: f32, thunder_level: f32) {
        self.rain_level_bits = rain_level.to_bits();
        self.thunder_level_bits = thunder_level.to_bits();
    }

    /// `Level.isThundering`: the level can have weather and the thunder,
    /// scaled by the rain, is above nine tenths.
    pub fn is_thundering(&self) -> bool {
        let environment = &self.dimension_environment;
        let can_have_weather = environment.has_skylight
            && !environment.has_ceiling
            && self.dimension_key != "minecraft:the_end";
        can_have_weather
            && f32::from_bits(self.thunder_level_bits) * f32::from_bits(self.rain_level_bits) > 0.9
    }

    /// `ClientLevel.setTimeFromServer`: the game time the server sent.
    pub fn set_game_time(&mut self, game_time: u64) {
        self.game_time = game_time;
    }

    pub fn tick(&mut self) {
        self.world_border.tick();
        self.block_animations.tick();
        self.tick_skulls();
        self.game_time = self.game_time.wrapping_add(1);
        let local_player_is_passenger = self.local_player.as_ref().is_some_and(|player| {
            self.entity_vehicle_id(player.id).is_some_and(|vehicle_id| {
                self.entity_passengers(vehicle_id)
                    .is_some_and(|ids| ids.contains(&player.id))
            })
        });
        if !local_player_is_passenger {
            self.tick_local_player(None);
        }
        self.tick_entities();
        self.tick_sky_flash();
    }

    /// `ClientLevel.tick`'s share: the sky flash counts down.
    pub fn tick_sky_flash(&mut self) {
        if self.sky_flash_time > 0 {
            self.sky_flash_time -= 1;
        }
    }

    /// `ClientLevel.skyFlashTime`, without the option that hides the flash.
    pub fn sky_flash_time(&self) -> i32 {
        self.sky_flash_time
    }

    /// `ClientLevel.tickEntities`: every entity the server added that is
    /// ticking moves on a tick, its passengers after it (the local player,
    /// who has a tick of its own, is not among them unless it rides one).
    pub fn tick_entities(&mut self) {
        let ticking_entity_ids: Vec<i32> = self
            .entities
            .iter()
            .filter(|(_, entity)| entity.ticking && entity.vehicle_id.is_none())
            .map(|(&entity_id, _)| entity_id)
            .collect();
        for entity_id in ticking_entity_ids {
            self.tick_entity_tree(entity_id, None);
        }
        // `LightningBolt.tick`: a bolt that is still lit flashes the sky.
        if self
            .entities
            .values()
            .any(|entity| entity.ticking && entity.lightning().is_some_and(|bolt| bolt.is_lit()))
        {
            self.sky_flash_time = 2;
        }
    }

    /// Apply Java's `Entity.turn` deltas and notify a boat passenger's vehicle.
    pub fn turn_entity(&mut self, entity_id: i32, mouse_x: f64, mouse_y: f64) -> bool {
        let vehicle_id = self.entity_vehicle_id(entity_id);
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.turn(mouse_x, mouse_y);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity_id)
        {
            player.turn(mouse_x, mouse_y);
        } else {
            return false;
        }
        if let Some(vehicle_id) = vehicle_id {
            self.clamp_entity_passenger_rotation_after_turn(vehicle_id, entity_id);
        }
        true
    }
}
