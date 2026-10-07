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

    /// Copy local player movement flags into a client-authoritative boat.
    pub fn set_boat_input(&mut self, vehicle_id: i32, input: BoatInput) -> bool {
        let Some(boat) = self.entities.get_mut(&vehicle_id) else {
            return false;
        };
        if vanilla_boat_max_passengers(boat.type_id).is_none() {
            return false;
        }
        boat.boat_input = input;
        true
    }

    pub(super) fn tick_local_player(&mut self, vehicle_id: Option<i32>) {
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

    pub(super) fn tick_entity_tree(&mut self, entity_id: i32, vehicle_id: Option<i32>) {
        if vehicle_id.is_some()
            && let Some(entity) = self.entities.get_mut(&entity_id)
        {
            entity.motion.x = JavaF64::new(0.0_f64);
            entity.motion.y = JavaF64::new(0.0_f64);
            entity.motion.z = JavaF64::new(0.0_f64);
        }
        self.tick_entity_once(entity_id);
        if let Some(vehicle_id) = vehicle_id {
            self.position_entity_passenger(vehicle_id, entity_id);
        }
        let passenger_ids = self
            .entity_passengers(entity_id)
            .map(<[i32]>::to_vec)
            .unwrap_or_default();
        for passenger_id in passenger_ids {
            self.tick_entity_passenger_tree(entity_id, passenger_id);
        }
    }

    pub(super) fn tick_entity_passenger_tree(&mut self, vehicle_id: i32, passenger_id: i32) {
        if self.entity_vehicle_id(passenger_id) != Some(vehicle_id) {
            return;
        }
        if self
            .local_player
            .as_ref()
            .is_some_and(|player| player.id == passenger_id)
        {
            self.tick_local_player(Some(vehicle_id));
        } else if self
            .entities
            .get(&passenger_id)
            .is_some_and(|entity| entity.ticking || entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID)
        {
            self.tick_entity_tree(passenger_id, Some(vehicle_id));
        }
    }

    /// `getPassengerRidingPosition` of a boat for a passenger: where the boat carries it, from
    /// where the boat stands and faces and how many ride with it.
    pub(crate) fn boat_rider_position(
        &self,
        vehicle_id: i32,
        passenger_id: i32,
    ) -> Option<voxel_core::math::Vec3> {
        let passenger_type_id = self
            .entities
            .get(&passenger_id)
            .map(|entity| entity.type_id)
            .or_else(|| {
                self.local_player
                    .as_ref()
                    .filter(|player| player.id == passenger_id)
                    .map(|_| VANILLA_PLAYER_ENTITY_TYPE_ID)
            })?;
        let (vehicle_type_id, vehicle_position, vehicle_y_rot, passenger_index, passenger_count) =
            self.entities.get(&vehicle_id).and_then(|vehicle| {
                let passenger_index = vehicle
                    .passenger_ids
                    .iter()
                    .position(|&entity_id| entity_id == passenger_id)?;
                Some((
                    vehicle.type_id,
                    vehicle.position(),
                    vehicle.y_rot.get(),
                    passenger_index,
                    vehicle.passenger_ids.len(),
                ))
            })?;
        let (ride_height, single_passenger_z_offset) = VANILLA_BOAT_PASSENGER_GEOMETRY
            .iter()
            .find(|&&(type_id, _, _)| type_id == vehicle_type_id)
            .map(|&(_, ride_height, single_passenger_z_offset)| {
                (f64::from(ride_height), f64::from(single_passenger_z_offset))
            })?;
        let passenger_is_baby = self
            .entities
            .get(&passenger_id)
            .is_some_and(|passenger| passenger.baby);
        let passenger_vehicle_attachment_y =
            vanilla_passenger_vehicle_attachment_y(passenger_type_id, passenger_is_baby);

        let passenger_z_offset = if passenger_count > 1 {
            let base_offset = if passenger_index == 0 {
                0.2_f32
            } else {
                -0.6_f32
            };
            let animal_offset = if vanilla_passenger_is_animal(passenger_type_id) {
                0.2_f32
            } else {
                0.0_f32
            };
            f64::from(base_offset + animal_offset)
        } else {
            single_passenger_z_offset
        };
        let radians = f64::from(-vehicle_y_rot * (std::f32::consts::PI / 180.0_f32));
        let rotated_x = passenger_z_offset * f64::from(mth_sin_f64(radians));
        let rotated_z = passenger_z_offset * f64::from(mth_cos_f64(radians));
        let target_position = (
            vehicle_position.0 + rotated_x,
            vehicle_position.1 + ride_height - passenger_vehicle_attachment_y,
            vehicle_position.2 + rotated_z,
        );
        Some(voxel_core::math::Vec3::new(
            target_position.0,
            target_position.1,
            target_position.2,
        ))
    }

    pub(super) fn position_entity_passenger(&mut self, vehicle_id: i32, passenger_id: i32) {
        let Some(target) = self.boat_rider_position(vehicle_id, passenger_id) else {
            return;
        };
        let target_position = (target.x, target.y, target.z);
        let previous_entity_position = self.entities.get(&passenger_id).map(ClientEntity::position);
        if let Some(passenger) = self.entities.get_mut(&passenger_id) {
            passenger.set_position(target_position);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == passenger_id)
        {
            player.set_position(target_position);
        }
        if let Some(previous_position) = previous_entity_position {
            self.update_entity_section(passenger_id, previous_position, target_position);
        }
        self.clamp_entity_passenger_rotation(vehicle_id, passenger_id);
    }

    pub(super) fn clamp_entity_passenger_rotation(&mut self, vehicle_id: i32, passenger_id: i32) {
        let Some((vehicle_type_id, vehicle_y_rot, delta_rotation, passenger_count)) =
            self.entities.get(&vehicle_id).map(|vehicle| {
                (
                    vehicle.type_id,
                    vehicle.y_rot.get(),
                    vehicle.delta_rotation.get(),
                    vehicle.passenger_ids.len(),
                )
            })
        else {
            return;
        };
        let Some(passenger_type_id) = self
            .entities
            .get(&passenger_id)
            .map(|passenger| passenger.type_id)
            .or_else(|| {
                self.local_player
                    .as_ref()
                    .filter(|player| player.id == passenger_id)
                    .map(|_| VANILLA_PLAYER_ENTITY_TYPE_ID)
            })
        else {
            return;
        };
        if vanilla_passenger_can_turn_in_boat(passenger_type_id) {
            return;
        }
        let Some(max_passenger_count) = vanilla_boat_max_passengers(vehicle_type_id) else {
            return;
        };
        let animal_rotation_offset = (vanilla_passenger_is_animal(passenger_type_id)
            && passenger_count == max_passenger_count)
            .then_some({
                if passenger_id % 2 == 0 {
                    90.0_f32
                } else {
                    270.0_f32
                }
            });
        if let Some(passenger) = self.entities.get_mut(&passenger_id) {
            passenger.add_vanilla_boat_passenger_rotation(delta_rotation);
            passenger.clamp_vanilla_boat_passenger_rotation(vehicle_y_rot, animal_rotation_offset);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == passenger_id)
        {
            player.add_vanilla_boat_passenger_rotation(delta_rotation);
            player.clamp_vanilla_boat_passenger_rotation(vehicle_y_rot);
        }
    }

    pub(super) fn clamp_entity_passenger_rotation_after_turn(
        &mut self,
        vehicle_id: i32,
        passenger_id: i32,
    ) {
        let Some(vehicle_y_rot) = self.entities.get(&vehicle_id).and_then(|vehicle| {
            VANILLA_BOAT_PASSENGER_GEOMETRY
                .iter()
                .any(|&(type_id, _, _)| type_id == vehicle.type_id)
                .then(|| vehicle.y_rot.get())
        }) else {
            return;
        };
        if let Some(passenger) = self.entities.get_mut(&passenger_id) {
            passenger.clamp_vanilla_boat_passenger_rotation(vehicle_y_rot, None);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == passenger_id)
        {
            player.clamp_vanilla_boat_passenger_rotation(vehicle_y_rot);
        }
    }

    /// `Cat.handleLieDown`'s search: a player who sleeps within two blocks of
    /// the cat's block (the local player, or one the server added).
    pub(super) fn sleeping_player_near(&self, cat: &ClientEntity) -> bool {
        let (x, y, z) = cat.position();
        let (x, y, z) = (x.floor(), y.floor(), z.floor());
        let area =
            voxel_core::aabb::Aabb::new(x, y, z, x + 1.0, y + 1.0, z + 1.0).inflate(2.0, 2.0, 2.0);
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

    /// What the species that look about them read of the world: whether this
    /// is a cat that lies down with a sleeping player near, and whether it
    /// thunders.
    /// `Entity.updateFluidInteraction`'s answer for the mobs that read it: whether
    /// the water reaches their box (none for the others, whose state is not
    /// tracked). A squid is not pushed by fluids, so no current is gathered.
    /// `AbstractBoat.isUnderwater`'s answer (the status is under (flowing) water):
    /// whether the water reaches a thousandth above the top of the boat's box.
    /// Whether a boat (and only a boat) is under water, as its tick reads it.
    pub(super) fn boat_under_water_of(&self, entity_id: i32) -> Option<bool> {
        let entity = self
            .entities
            .get(&entity_id)
            .filter(|entity| entity.is_boat())?;
        Some(self.boat_is_under_water(entity))
    }

    pub(super) fn boat_is_under_water(&self, entity: &ClientEntity) -> bool {
        let Some(pick) = voxel_data::entity_type::pick(entity.type_id) else {
            return false;
        };
        let (width, height, _) = pick.size;
        let (x, y, z) = entity.position();
        let area = ClientEntityDimensions::scalable(width, height)
            .scale(entity.scale(), entity.scale())
            .make_bounding_box(voxel_core::math::Vec3::new(x, y, z));
        let top = area.max_y + 0.001;
        let level = local_player::Level::new(self);
        for bx in voxel_core::math::floor_f64(area.min_x)..voxel_core::math::ceil_f64(area.max_x) {
            for by in voxel_core::math::floor_f64(area.max_y)..voxel_core::math::ceil_f64(top) {
                for bz in
                    voxel_core::math::floor_f64(area.min_z)..voxel_core::math::ceil_f64(area.max_z)
                {
                    let pos = voxel_core::binary::BlockPos::new(bx, by, bz);
                    let info = level.info(pos);
                    if info.is_water()
                        && top < f64::from(by) + f64::from(level.fluid_height(pos, &info))
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub(super) fn mob_in_water(&self, entity_id: i32) -> Option<bool> {
        let entity = self.entities.get(&entity_id)?;
        if !entity.tracks_water() {
            return None;
        }
        let pick = voxel_data::entity_type::pick(entity.type_id)?;
        let (width, height, eye) = match pick.baby {
            Some(baby) if entity.is_baby() => baby,
            _ => pick.size,
        };
        let dimensions = ClientEntityDimensions::scalable(width, height)
            .with_eye_height(eye)
            .scale(entity.fish_dimension_scale(), entity.fish_dimension_scale())
            .scale(entity.scale(), entity.scale());
        let (x, y, z) = entity.position();
        let position = voxel_core::math::Vec3::new(x, y, z);
        let found = fluid::scan(
            &local_player::Level::new(self),
            (
                dimensions.make_bounding_box(position),
                position,
                dimensions.eye_height(),
            ),
            true,
        )
        .ok()?;
        Some(found.water.height > 0.0)
    }

    /// `Guardian.getActiveAttackTarget` on the client: the living entity the
    /// beam's id names, the local player among them.
    pub(super) fn guardian_target_of(&self, entity_id: i32) -> Option<guardian::GuardianTarget> {
        let id = self.entities.get(&entity_id)?.guardian_attack_target()?;
        self.living_target(id)
    }

    /// What a tick of the entity looks up in the world.
    pub(super) fn targets_of(&self, entity_id: i32) -> entity::EntityTargets {
        entity::EntityTargets {
            guardian: self.guardian_target_of(entity_id),
            wither: self.wither_targets_of(entity_id),
            dragon: self.dragon_crystals_of(entity_id),
        }
    }

    /// `EnderDragon.checkCrystals`' look at the world: the nearest end crystal whose box
    /// meets the dragon's box made 32 blocks wider (by the distance between the two
    /// entities' places), and whether the crystal the dragon holds is still there.
    pub(super) fn dragon_crystals_of(&self, entity_id: i32) -> ender_dragon::DragonCrystals {
        let Some(dragon) = self
            .entities
            .get(&entity_id)
            .filter(|e| e.is_ender_dragon())
        else {
            return ender_dragon::DragonCrystals::default();
        };
        let current = dragon.ender_dragon().and_then(|own| own.nearest_crystal());
        let current_exists = current.is_some_and(|id| {
            self.entities
                .get(&id)
                .is_some_and(ClientEntity::is_end_crystal)
        });
        let (x, y, z) = dragon.position();
        let size = dragon.dimensions();
        let half = f64::from(size.width()) / 2.0 + 32.0;
        let (low, high) = (
            [x - half, y - 32.0, z - half],
            [x + half, y + f64::from(size.height()) + 32.0, z + half],
        );
        let mut nearest = None;
        let mut distance = f64::MAX;
        for crystal in self.entities.values().filter(|e| e.is_end_crystal()) {
            let (cx, cy, cz) = crystal.position();
            let crystal_size = crystal.dimensions();
            let reach = f64::from(crystal_size.width()) / 2.0;
            let height = f64::from(crystal_size.height());
            let meets = low[0] < cx + reach
                && high[0] > cx - reach
                && low[1] < cy + height
                && high[1] > cy
                && low[2] < cz + reach
                && high[2] > cz - reach;
            if !meets {
                continue;
            }
            let squared = (cx - x).powi(2) + (cy - y).powi(2) + (cz - z).powi(2);
            if squared < distance {
                distance = squared;
                nearest = Some(crystal.id());
            }
        }
        ender_dragon::DragonCrystals {
            nearest,
            current_exists,
        }
    }

    /// The two entities a wither's side heads look at, by the ids it holds.
    pub(super) fn wither_targets_of(
        &self,
        entity_id: i32,
    ) -> [Option<guardian::GuardianTarget>; 2] {
        let ids = self
            .entities
            .get(&entity_id)
            .map_or([0; 2], ClientEntity::wither_targets);
        ids.map(|id| (id > 0).then(|| self.living_target(id)).flatten())
    }

    /// Where the living entity of an id is and how tall its eyes are, if the world holds one.
    pub(super) fn living_target(&self, id: i32) -> Option<guardian::GuardianTarget> {
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

    pub(super) fn species_surroundings(&self, entity_id: i32) -> (bool, bool) {
        let sleeper_near_cat = self
            .entities
            .get(&entity_id)
            .is_some_and(|entity| entity.is_lying_cat() && self.sleeping_player_near(entity));
        (sleeper_near_cat, self.is_thundering())
    }
}
