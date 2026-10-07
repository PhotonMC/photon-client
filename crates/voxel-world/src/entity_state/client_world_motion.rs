use super::*;

impl ClientWorld {
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
}
