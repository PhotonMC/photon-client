use super::*;

impl ClientWorld {
    /// Apply a pinned block-state's BlockTags.CLIMBABLE membership to a living entity.
    pub fn apply_entity_living_climbable_block_state(
        &mut self,
        entity_id: i32,
        state_id: i32,
    ) -> bool {
        let on_climbable = self.block_state_registry.is_climbable_state(state_id);
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.on_climbable = on_climbable;
        true
    }

    /// Apply the LivingEntity.onClimbable tag and fall-flying context.
    pub fn apply_entity_living_climbable_glide_through_state(
        &mut self,
        entity_id: i32,
        state_id: i32,
        fall_flying: bool,
    ) -> bool {
        let is_climbable = self.block_state_registry.is_climbable_state(state_id);
        let is_glide_through = self.block_state_registry.is_glide_through_state(state_id);
        let on_climbable = if fall_flying && is_glide_through {
            false
        } else {
            is_climbable
        };
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.on_climbable = on_climbable;
        if fall_flying {
            entity.shared_flags |= 1 << SHARED_FLAG_FALL_FLYING;
        } else {
            entity.shared_flags &= !(1 << SHARED_FLAG_FALL_FLYING);
        }
        true
    }

    /// Apply the Player ability state used by Java's `Player.onClimbable` gate.
    pub fn apply_entity_player_flying_state(&mut self, entity_id: i32, flying: bool) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if entity.type_id != VANILLA_PLAYER_ENTITY_TYPE_ID {
            return false;
        }
        entity.flying = flying;
        true
    }

    /// Apply a clientbound vehicle and ordered passenger relationship.
    pub fn apply_entity_passengers(&mut self, vehicle_id: i32, passenger_ids: &[i32]) -> bool {
        if !self.entity_exists(vehicle_id) {
            return false;
        }

        self.eject_entity_passengers(vehicle_id);
        for &passenger_id in passenger_ids {
            if !self.entity_exists(passenger_id)
                || self.would_create_passenger_cycle(passenger_id, vehicle_id)
                || self.entity_vehicle_id(passenger_id) == Some(vehicle_id)
            {
                continue;
            }
            self.detach_entity_from_vehicle(passenger_id);
            self.set_entity_vehicle_id(passenger_id, Some(vehicle_id));
            if let Some(vehicle) = self.entities.get_mut(&vehicle_id) {
                vehicle.passenger_ids.push(passenger_id);
            } else if let Some(player) = self
                .local_player
                .as_mut()
                .filter(|player| player.id == vehicle_id)
            {
                player.passenger_ids.push(passenger_id);
            }
        }
        true
    }

    pub(super) fn entity_exists(&self, entity_id: i32) -> bool {
        self.entities.contains_key(&entity_id)
            || self
                .local_player
                .as_ref()
                .is_some_and(|player| player.id == entity_id)
    }

    pub fn entity_vehicle_id(&self, entity_id: i32) -> Option<i32> {
        if let Some(entity) = self.entities.get(&entity_id) {
            return entity.vehicle_id;
        }
        self.local_player
            .as_ref()
            .filter(|player| player.id == entity_id)
            .and_then(|player| player.vehicle_id)
    }

    pub fn entity_passengers(&self, entity_id: i32) -> Option<&[i32]> {
        if let Some(entity) = self.entities.get(&entity_id) {
            return Some(&entity.passenger_ids);
        }
        self.local_player
            .as_ref()
            .filter(|player| player.id == entity_id)
            .map(|player| player.passenger_ids.as_slice())
    }

    pub(super) fn set_entity_vehicle_id(&mut self, entity_id: i32, vehicle_id: Option<i32>) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            if entity.is_shulker() && entity.vehicle_id != vehicle_id {
                if vehicle_id.is_some() {
                    entity.shulker_starts_riding();
                } else {
                    entity.shulker_stops_riding();
                }
            }
            entity.vehicle_id = vehicle_id;
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity_id)
        {
            player.vehicle_id = vehicle_id;
        }
    }

    pub(super) fn detach_entity_from_vehicle(&mut self, entity_id: i32) {
        let Some(vehicle_id) = self.entity_vehicle_id(entity_id) else {
            return;
        };
        if let Some(vehicle) = self.entities.get_mut(&vehicle_id) {
            vehicle
                .passenger_ids
                .retain(|&passenger_id| passenger_id != entity_id);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == vehicle_id)
        {
            player
                .passenger_ids
                .retain(|&passenger_id| passenger_id != entity_id);
        }
        self.set_entity_vehicle_id(entity_id, None);
    }

    pub(super) fn eject_entity_passengers(&mut self, vehicle_id: i32) {
        let passengers = if let Some(vehicle) = self.entities.get_mut(&vehicle_id) {
            Some(std::mem::take(&mut vehicle.passenger_ids))
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == vehicle_id)
        {
            Some(std::mem::take(&mut player.passenger_ids))
        } else {
            None
        };
        let Some(passengers) = passengers else {
            return;
        };
        for passenger_id in passengers {
            if self.entity_vehicle_id(passenger_id) == Some(vehicle_id) {
                self.set_entity_vehicle_id(passenger_id, None);
            }
        }
    }

    pub(super) fn would_create_passenger_cycle(&self, passenger_id: i32, vehicle_id: i32) -> bool {
        if passenger_id == vehicle_id {
            return true;
        }
        let mut visited = BTreeSet::new();
        let mut current = Some(vehicle_id);
        while let Some(entity_id) = current {
            if !visited.insert(entity_id) {
                return true;
            }
            if entity_id == passenger_id {
                return true;
            }
            current = self.entity_vehicle_id(entity_id);
        }
        false
    }

    #[expect(
        clippy::cognitive_complexity,
        clippy::too_many_lines,
        reason = "one step per metadata field the client applies"
    )]
    pub fn apply_entity_metadata(&mut self, entity_id: i32, metadata: &[u8]) -> bool {
        let shared_flags = shared_flags_from_metadata(metadata);
        let living_flags = living_flags_from_metadata(metadata);
        let mob_flags = mob_flags_from_metadata(metadata);
        let air_supply = air_supply_from_metadata(metadata);
        let health = health_from_metadata(metadata);
        let arrow_count = arrow_count_from_metadata(metadata);
        let stinger_count = stinger_count_from_metadata(metadata);
        let effect_ambience = effect_ambience_from_metadata(metadata);
        let effect_particle_type_ids = effect_particle_type_ids_from_metadata(metadata);
        let zombie_baby = zombie_baby_from_metadata(metadata);
        let sleeping_pos = sleeping_pos_from_metadata(metadata);
        let custom_name_present = custom_name_present_from_metadata(metadata);
        let custom_name_visible = custom_name_visible_from_metadata(metadata);
        let no_gravity = no_gravity_from_metadata(metadata);
        let silent = silent_from_metadata(metadata);
        let pose = metadata_pose_value(metadata);
        let ticks_frozen = ticks_frozen_from_metadata(metadata);
        if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity_id)
        {
            player.entity_data = Some(metadata.to_vec());
            if let Some(shared_flags) = shared_flags {
                player.shared_flags = shared_flags;
            }
            if let Some(living_flags) = living_flags {
                player.living_flags = living_flags;
            }
            if let Some(air_supply) = air_supply {
                player.air_supply = air_supply;
            }
            if let Some(health) = health {
                player.health_bits = health.to_bits();
            }
            if let Some(arrow_count) = arrow_count {
                player.arrow_count = arrow_count;
            }
            if let Some(stinger_count) = stinger_count {
                player.stinger_count = stinger_count;
            }
            if let Some(effect_ambience) = effect_ambience {
                player.effect_ambience = effect_ambience;
            }
            if let Some(effect_particle_type_ids) = effect_particle_type_ids.as_ref() {
                player.effect_particle_type_ids = effect_particle_type_ids.clone();
            }
            if let Some(sleeping_pos) = sleeping_pos {
                player.sleeping_pos = sleeping_pos;
            }
            if let Some(custom_name_present) = custom_name_present {
                player.custom_name_present = custom_name_present;
            }
            if let Some(custom_name_visible) = custom_name_visible {
                player.custom_name_visible = custom_name_visible;
            }
            if let Some(no_gravity) = no_gravity {
                player.no_gravity = no_gravity;
            }
            if let Some(silent) = silent {
                player.silent = silent;
            }
            if let Some(pose) = pose {
                player.pose = pose;
            }
            if let Some(ticks_frozen) = ticks_frozen {
                player.ticks_frozen = ticks_frozen;
            }
            if let Some(absorption) = player_absorption_from_metadata(metadata) {
                player.absorption_bits = absorption.to_bits();
            }
            if let Some(customisation) = player_model_customisation_from_metadata(metadata) {
                player.model_customisation = customisation;
            }
            if let Some(score) = player_score_from_metadata(metadata) {
                player.score = score;
            }
            return true;
        }
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.metadata = Some(metadata.to_vec());
        if let Some(variant) = variant_from_metadata(entity.type_id, metadata) {
            entity.living.variant = Some(variant);
        }
        if let Some(value) = orb_value_from_metadata(entity.type_id, metadata) {
            entity.living.orb_value = Some(value);
        }
        let (piglin_baby, piglin_dancing, piglin_charging) =
            piglin_flags_from_metadata(entity.type_id, metadata);
        if let Some(charging) = piglin_charging {
            entity.living.charging_crossbow = charging;
        }
        if let Some(baby) = piglin_baby {
            entity.baby = baby;
        }
        if let Some(dancing) = piglin_dancing {
            entity.living.dancing = dancing;
        }
        if let Some(left) = avatar_main_arm_is_left(metadata) {
            entity.living.main_arm_left = left;
        }
        if let Some(data) = villager_data_from_metadata(entity.type_id, metadata) {
            entity.living.villager_data = Some(data);
        }
        if let Some(counter) = unhappy_counter_from_metadata(entity.type_id, metadata) {
            entity.living.unhappy_counter = counter;
        }
        let (carrying, creepy) = enderman_flags_from_metadata(entity.type_id, metadata);
        if let Some(carried) = carrying {
            entity.living.carried_state = carried;
        }
        let (left_horn, right_horn) = goat_horns_from_metadata(entity.type_id, metadata);
        if let Some(left) = left_horn {
            entity.living.lost_left_horn = !left;
        }
        if let Some(right) = right_horn {
            entity.living.lost_right_horn = !right;
        }
        if let Some(powered) = creeper_powered_from_metadata(entity.type_id, metadata) {
            entity.living.powered = powered;
        }
        let (camel_dash, camel_last_pose) = camel_data_from_metadata(entity.type_id, metadata);
        if camel_dash.is_some() || camel_last_pose.is_some() {
            let horse = entity
                .kind
                .horse_or_insert_with(|| Box::new(horse::HorseState::new()));
            if let Some(dash) = camel_dash {
                horse.camel.set_dashing(dash);
            }
            if let Some(tick) = camel_last_pose {
                horse.camel.set_last_pose_change_tick(tick);
            }
        }
        let (horse_flags, horse_variant, horse_chest) =
            horse_data_from_metadata(entity.type_id, metadata);
        if let Some(flags) = horse_flags {
            entity
                .kind
                .horse_or_insert_with(|| Box::new(horse::HorseState::new()))
                .set_flags(flags);
        }
        if let Some(variant) = horse_variant {
            entity.living.horse_variant = variant;
        }
        if let Some(chest) = horse_chest {
            entity.living.horse_chest = chest;
        }
        let (bee_flags, bee_anger) = bee_data_from_metadata(entity.type_id, metadata);
        if let Some(flags) = bee_flags {
            entity.living.bee_flags = flags;
        }
        if let Some(end) = bee_anger {
            entity.living.anger_end = end;
        }
        entity
            .living
            .apply_wolf(wolf_data_from_metadata(entity.type_id, metadata));
        entity
            .living
            .apply_cat(cat_data_from_metadata(entity.type_id, metadata));
        let panda_data = panda_data_from_metadata(entity.type_id, metadata);
        if panda_data != panda::PandaData::default() {
            entity
                .kind
                .panda_or_insert_with(Default::default)
                .apply(panda_data);
        }
        let (fox_flags, fox_variant) = fox_data_from_metadata(entity.type_id, metadata);
        entity.living.fox.set_data(fox_flags, fox_variant);
        if let Some(suffocating) = strider_suffocating_from_metadata(entity.type_id, metadata) {
            entity.living.strider_suffocating = suffocating;
        }
        if let Some((state, weather)) = copper_golem_data_from_metadata(entity.type_id, metadata) {
            if let Some(state) = state {
                entity.living.copper_golem_action = copper_golem::GolemAction::from_id(state);
            }
            if let Some(weather) = weather {
                entity.living.copper_golem_weather = weather.clamp(0, 3) as u8;
            }
        }
        if let Some(ticks) = glow_squid_dark_ticks_from_metadata(entity.type_id, metadata) {
            entity.living.glow_dark_ticks = ticks;
        }
        if let Some(variant) = fish_variant_from_metadata(entity.type_id, metadata) {
            entity.living.fish_variant = Some(variant);
        }
        apply_species_data(entity, metadata);
        let (has_egg, laying_egg) = turtle_eggs_from_metadata(entity.type_id, metadata);
        if let Some(has_egg) = has_egg {
            entity.living.turtle_has_egg = has_egg;
        }
        if let Some(laying_egg) = laying_egg {
            entity.living.turtle_laying_egg = laying_egg;
        }
        if let Some(immune) = hoglin_immune_from_metadata(entity.type_id, metadata) {
            entity.living.hoglin_immune = immune;
        }
        if let Some(standing) = polar_bear_standing_from_metadata(entity.type_id, metadata) {
            entity.living.polar_bear.set_standing(standing);
        }
        if let Some(variant) = rabbit_type_from_metadata(entity.type_id, metadata) {
            entity.living.rabbit_type = variant;
        }
        if let Some(variant) = mooshroom_type_from_metadata(entity.type_id, metadata) {
            entity.living.mooshroom_type = variant;
        }
        if let Some(pumpkin) = snow_golem_pumpkin_from_metadata(entity.type_id, metadata) {
            entity.living.bare_head = !pumpkin;
        }
        if let Some(creepy) = creepy {
            entity.living.creepy = creepy;
        }
        let (celebrating, casting) = illager_flags_from_metadata(entity.type_id, metadata);
        if let Some(celebrating) = celebrating {
            entity.living.celebrating = celebrating;
        }
        if let Some(casting) = casting {
            entity.living.casting_spell = casting;
        }
        if let Some(wool) = sheep_wool_from_metadata(entity.type_id, metadata) {
            entity.living.animal.wool = wool;
        }
        if voxel_data::entity_type::name(entity.type_id) == Some("minecraft:armor_stand") {
            entity.living.armor_stand.apply(metadata);
        }
        let (swell_dir, ignited) = creeper_fuse_from_metadata(entity.type_id, metadata);
        if let Some(direction) = swell_dir {
            entity.living.animal.set_swell_dir(direction);
        }
        if let Some(ignited) = ignited {
            entity.living.animal.set_ignited(ignited);
        }
        if let Some(shared_flags) = shared_flags {
            entity.shared_flags = shared_flags;
        }
        if let Some(living_flags) = living_flags {
            entity.living_flags = living_flags;
        }
        if let Some(mob_flags) = mob_flags {
            entity.mob_flags = mob_flags;
        }
        if let Some(air_supply) = air_supply {
            entity.air_supply = air_supply;
        }
        if let Some(health) = health {
            entity.health_bits = Some(health.to_bits());
            // `isAlive` of a living entity: not removed, health left.
            entity.alive = health > 0.0;
        }
        if let Some(arrow_count) = arrow_count {
            entity.arrow_count = arrow_count;
        }
        if let Some(stinger_count) = stinger_count {
            entity.stinger_count = stinger_count;
        }
        if let Some(effect_ambience) = effect_ambience {
            entity.effect_ambience = effect_ambience;
        }
        if let Some(effect_particle_type_ids) = effect_particle_type_ids {
            entity.effect_particle_type_ids = effect_particle_type_ids;
        }
        if let Some(sleeping_pos) = sleeping_pos {
            entity.sleeping_pos = sleeping_pos;
        }
        if let Some(custom_name_present) = custom_name_present {
            entity.custom_name_present = custom_name_present;
        }
        if let Some(custom_name_visible) = custom_name_visible {
            entity.custom_name_visible = custom_name_visible;
        }
        if let Some(no_gravity) = no_gravity {
            entity.no_gravity = no_gravity;
        }
        if let Some(silent) = silent {
            entity.silent = silent;
        }
        if let Some(pose) = pose {
            entity.set_pose(pose);
        }
        // `DATA_BABY_ID` is the same field for a zombie and for every
        // animal and villager (`AgeableMob`).
        if (entity.is_zombie_type()
            || vanilla_passenger_is_animal(entity.type_id)
            || matches!(
                voxel_data::entity_type::name(entity.type_id),
                Some("minecraft:villager" | "minecraft:dolphin")
            ))
            && let Some(zombie_baby) = zombie_baby
        {
            entity.baby = zombie_baby;
        }
        if let Some(ticks_frozen) = ticks_frozen {
            entity.ticks_frozen = ticks_frozen;
        }
        true
    }

    pub fn apply_entity_motion(&mut self, input: EntityMotionInput) -> bool {
        let Some(entity) = self.entities.get_mut(&input.entity_id) else {
            return false;
        };
        if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
            entity.motion_lerp_target.x = JavaF64::new(input.x);
            entity.motion_lerp_target.y = JavaF64::new(input.y);
            entity.motion_lerp_target.z = JavaF64::new(input.z);
            entity.motion_lerp_steps = REMOTE_PLAYER_MOTION_LERP_STEPS;
        } else {
            entity.motion.x = JavaF64::new(input.x);
            entity.motion.y = JavaF64::new(input.y);
            entity.motion.z = JavaF64::new(input.z);
        }
        true
    }

    /// `handleEntityLinkPacket`: the entity is tied to the entity with this id, or let go when
    /// it is zero. Only what can be leashed takes it; the holder is looked up when the leash is
    /// drawn, as it may arrive later than the lead.
    pub fn set_leash_holder(&mut self, entity_id: i32, holder_id: i32) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if voxel_data::leash::leash_of(entity.type_id).is_none() {
            return false;
        }
        entity.leash_holder_id = holder_id;
        true
    }

    pub fn apply_entity_ground_state(&mut self, entity_id: i32, on_ground: bool) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.on_ground = on_ground;
        self.check_item_supporting_block(entity_id);
        true
    }

    pub fn apply_entity_rotation(&mut self, input: EntityRotationInput) -> bool {
        {
            let Some(entity) = self.entities.get_mut(&input.entity_id) else {
                return false;
            };
            if (entity.has_interpolation_handler())
                && (input.y_rot.is_some() || input.x_rot.is_some())
            {
                let (current_y_rot, current_x_rot) = entity.interpolation_rotation();
                let y_rot = input.y_rot.map_or(current_y_rot, |y_rot| {
                    if input.relative_y_rot {
                        current_y_rot + y_rot
                    } else {
                        y_rot
                    }
                });
                let x_rot = input.x_rot.map_or(current_x_rot, |x_rot| {
                    if input.relative_x_rot {
                        current_x_rot + x_rot
                    } else {
                        x_rot
                    }
                });
                entity.move_or_interpolate_to(entity.interpolation_position(), y_rot, x_rot);
            } else {
                if let Some(y_rot) = input.y_rot {
                    let current = entity.y_rot.get();
                    let value = if input.relative_y_rot {
                        current + y_rot
                    } else {
                        y_rot
                    };
                    entity.y_rot = JavaF32::new(value % 360.0);
                }
                if let Some(x_rot) = input.x_rot {
                    let current = entity.x_rot.get();
                    let value = if input.relative_x_rot {
                        current + x_rot
                    } else {
                        x_rot
                    };
                    entity.x_rot = JavaF32::new((value % 360.0).clamp(-90.0, 90.0));
                }
            }
        }
        if let Some(y_head_rot) = input.y_head_rot
            && !self.apply_entity_head_rotation(input.entity_id, y_head_rot)
        {
            return false;
        }
        true
    }

    #[cfg(test)]
    pub(crate) fn apply_entity_position(&mut self, input: EntityPositionInput) -> bool {
        let Some(resolved) = self.resolve_entity_position(&input) else {
            return false;
        };
        self.apply_entity_position_target(
            input.entity_id,
            resolved.current,
            resolved.target,
            resolved.ticking,
            None,
            EntityPositionUpdateMode::PositionSync,
        )
    }

    pub fn apply_entity_position_and_rotation(
        &mut self,
        input: EntityPositionInput,
        rotation: EntityRotationInput,
        mode: EntityPositionUpdateMode,
    ) -> bool {
        let Some(resolved) = self.resolve_entity_position(&input) else {
            return false;
        };
        let Some(entity) = self.entities.get(&input.entity_id) else {
            return false;
        };
        let (current_y_rot, current_x_rot) = entity.interpolation_rotation();
        let target_y_rot = rotation.y_rot.map_or(current_y_rot, |y_rot| {
            if rotation.relative_y_rot {
                current_y_rot + y_rot
            } else {
                y_rot
            }
        });
        let target_x_rot = rotation.x_rot.map_or(current_x_rot, |x_rot| {
            if rotation.relative_x_rot {
                current_x_rot + x_rot
            } else {
                x_rot
            }
        });
        let target_x_rot = if mode == EntityPositionUpdateMode::Teleport {
            target_x_rot.clamp(-90.0, 90.0)
        } else {
            target_x_rot
        };
        self.apply_entity_position_target(
            input.entity_id,
            resolved.current,
            resolved.target,
            resolved.ticking,
            Some((target_y_rot, target_x_rot)),
            mode,
        )
    }

    pub fn apply_entity_teleport(&mut self, input: EntityTeleportInput) -> bool {
        let Some((current_motion, current_y_rot, current_x_rot)) =
            self.entities.get(&input.entity_id).map(|entity| {
                (
                    entity.motion(),
                    entity.interpolation_rotation().0,
                    entity.interpolation_rotation().1,
                )
            })
        else {
            return false;
        };
        let target_y_rot = if input.relative_flags & ENTITY_RELATIVE_Y_ROT_FLAG != 0 {
            current_y_rot + input.y_rot
        } else {
            input.y_rot
        };
        let target_x_rot = if input.relative_flags & ENTITY_RELATIVE_X_ROT_FLAG != 0 {
            current_x_rot + input.x_rot
        } else {
            input.x_rot
        }
        .clamp(-90.0, 90.0);
        let target_motion = resolve_entity_teleport_motion(
            current_motion,
            (current_y_rot, current_x_rot),
            (target_y_rot, target_x_rot),
            (input.delta_x, input.delta_y, input.delta_z),
            input.relative_flags,
        );
        let applied = self.apply_entity_position_and_rotation(
            EntityPositionInput {
                entity_id: input.entity_id,
                x: input.x,
                y: input.y,
                z: input.z,
                relative_flags: input.relative_flags,
            },
            EntityRotationInput {
                entity_id: input.entity_id,
                y_rot: Some(input.y_rot),
                x_rot: Some(input.x_rot),
                y_head_rot: None,
                relative_y_rot: input.relative_flags & ENTITY_RELATIVE_Y_ROT_FLAG != 0,
                relative_x_rot: input.relative_flags & ENTITY_RELATIVE_X_ROT_FLAG != 0,
            },
            EntityPositionUpdateMode::Teleport,
        );
        if applied && let Some(entity) = self.entities.get_mut(&input.entity_id) {
            entity.set_motion_if_finite(target_motion);
        }
        applied
    }

    pub fn apply_entity_move(&mut self, input: EntityMoveInput) -> bool {
        let Some((previous_position, current)) = self
            .entities
            .get(&input.entity_id)
            .map(|entity| (entity.position(), entity.packet_base()))
        else {
            return false;
        };
        let new_position = input.has_position.then(|| {
            (
                f64::from_bits(decode_entity_delta(current.0, input.delta_x)),
                f64::from_bits(decode_entity_delta(current.1, input.delta_y)),
                f64::from_bits(decode_entity_delta(current.2, input.delta_z)),
            )
        });
        let ticking = new_position.map(|position| {
            let (chunk_x, chunk_z) = entity_chunk_key(position);
            self.contains_chunk(chunk_x, chunk_z)
        });
        let position;
        {
            let entity = self
                .entities
                .get_mut(&input.entity_id)
                .expect("entity checked above");
            if let Some(position) = new_position {
                entity.set_packet_base(position);
            }
            if entity.has_interpolation_handler() {
                let target_position =
                    new_position.unwrap_or_else(|| entity.interpolation_position());
                let (current_y_rot, current_x_rot) = entity.interpolation_rotation();
                let target_y_rot = input.y_rot.unwrap_or(current_y_rot);
                let target_x_rot = input.x_rot.unwrap_or(current_x_rot);
                entity.move_or_interpolate_to(target_position, target_y_rot, target_x_rot);
            } else {
                if input.has_position {
                    let (x, y, z) = new_position.expect("position was requested");
                    entity.position.x = JavaF64::new(x);
                    entity.position.y = JavaF64::new(y);
                    entity.position.z = JavaF64::new(z);
                    entity.ticking =
                        entity.always_ticking || ticking.expect("position ticking state");
                }
                if let Some(y_rot) = input.y_rot {
                    entity.y_rot = JavaF32::new(y_rot % 360.0);
                }
                if let Some(x_rot) = input.x_rot {
                    entity.x_rot = JavaF32::new((x_rot % 360.0).clamp(-90.0, 90.0));
                }
            }
            if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
                entity.ticking = true;
            }
            position = entity.position();
        }
        self.update_entity_section(input.entity_id, previous_position, position);
        true
    }

    /// Apply Java's absolute entity position-sync path for a remote entity.
    pub fn apply_entity_position_sync(
        &mut self,
        entity_id: i32,
        position: (f64, f64, f64),
        rotation: (f32, f32),
    ) -> bool {
        self.apply_entity_position_and_rotation(
            EntityPositionInput {
                entity_id,
                x: position.0,
                y: position.1,
                z: position.2,
                relative_flags: 0,
            },
            EntityRotationInput {
                entity_id,
                y_rot: Some(rotation.0),
                x_rot: Some(rotation.1),
                y_head_rot: None,
                relative_y_rot: false,
                relative_x_rot: false,
            },
            EntityPositionUpdateMode::PositionSync,
        )
    }

    pub(super) fn resolve_entity_position(
        &self,
        input: &EntityPositionInput,
    ) -> Option<ResolvedEntityPosition> {
        let current = self
            .entities
            .get(&input.entity_id)
            .map(|entity| entity.interpolation_position())?;
        let target = (
            if input.relative_flags & ENTITY_RELATIVE_X_FLAG != 0 {
                current.0 + input.x
            } else {
                input.x
            },
            if input.relative_flags & ENTITY_RELATIVE_Y_FLAG != 0 {
                current.1 + input.y
            } else {
                input.y
            },
            if input.relative_flags & ENTITY_RELATIVE_Z_FLAG != 0 {
                current.2 + input.z
            } else {
                input.z
            },
        );
        let (chunk_x, chunk_z) = entity_chunk_key(target);
        Some(ResolvedEntityPosition {
            current,
            target,
            ticking: self.contains_chunk(chunk_x, chunk_z),
        })
    }

    pub(super) fn apply_entity_position_target(
        &mut self,
        entity_id: i32,
        current_position: (f64, f64, f64),
        target_position: (f64, f64, f64),
        ticking: bool,
        target_rotation: Option<(f32, f32)>,
        mode: EntityPositionUpdateMode,
    ) -> bool {
        let Some(previous_position) = self.entities.get(&entity_id).map(ClientEntity::position)
        else {
            return false;
        };
        let position;
        {
            let entity = self
                .entities
                .get_mut(&entity_id)
                .expect("entity position was checked above");
            if mode == EntityPositionUpdateMode::PositionSync {
                entity.set_packet_base(target_position);
            }
            if entity.has_interpolation_handler() {
                let rotation = target_rotation.unwrap_or_else(|| entity.interpolation_rotation());
                let distance_position = match mode {
                    EntityPositionUpdateMode::PositionSync => entity.position(),
                    EntityPositionUpdateMode::Teleport => current_position,
                };
                let delta_x = target_position.0 - distance_position.0;
                let delta_y = target_position.1 - distance_position.1;
                let delta_z = target_position.2 - distance_position.2;
                let can_interpolate =
                    match mode {
                        EntityPositionUpdateMode::PositionSync => entity.ticking,
                        EntityPositionUpdateMode::Teleport => true,
                    } && delta_x * delta_x + delta_y * delta_y + delta_z * delta_z
                        <= ENTITY_INTERPOLATION_DISTANCE_SQR;
                if can_interpolate {
                    entity.move_or_interpolate_to(target_position, rotation.0, rotation.1);
                } else {
                    entity.snap_to(target_position, rotation.0, rotation.1);
                }
            } else {
                entity.position.x = JavaF64::new(target_position.0);
                entity.position.y = JavaF64::new(target_position.1);
                entity.position.z = JavaF64::new(target_position.2);
                if let Some((y_rot, x_rot)) = target_rotation {
                    entity.y_rot = JavaF32::new(y_rot % 360.0);
                    entity.x_rot = JavaF32::new((x_rot % 360.0).clamp(-90.0, 90.0));
                }
            }
            entity.ticking = entity.always_ticking || ticking;
            position = entity.position();
        }
        self.update_entity_section(entity_id, previous_position, position);
        true
    }

    pub fn section_counts_at(
        &self,
        chunk_x: i32,
        section_y: i32,
        chunk_z: i32,
    ) -> Option<(i16, i16)> {
        let chunk = self.chunk_for_query(chunk_x, chunk_z)?;
        let section = chunk.sections.get(&section_y)?;
        Some((section.non_empty_block_count, section.fluid_count))
    }

    /// `getNoiseBiome`: the biome at a quart position, its row clamped into
    /// the dimension.
    pub fn noise_biome(&self, x: i32, y: i32, z: i32) -> Option<i32> {
        self.biome_at(x * 4, self.dimension.clamp_quart_y(y) * 4, z * 4)
    }
}
