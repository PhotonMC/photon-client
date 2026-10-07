use super::*;

impl ClientWorld {
    /// Where an entity is drawn between two ticks: its position
    /// (`getPosition(partialTick)`), its turn and its head's.
    pub fn entity_snapshot(&self, entity_id: i32, partial_tick: f32) -> Option<EntitySnapshot> {
        let entity = self.entities.get(&entity_id)?;
        let (yaw, pitch, head_yaw) = entity.interpolated_rotation(partial_tick);
        Some(EntitySnapshot {
            position: entity.interpolated_position(partial_tick),
            yaw,
            pitch,
            head_yaw,
            body_yaw: entity.interpolated_body_y_rotation(partial_tick),
            walk_position: entity.walk_animation().position_at(partial_tick),
            walk_speed: entity.walk_animation().speed_at(partial_tick),
            swim_amount: entity.swim_amount(partial_tick),
            pose: entity.pose(),
            red_overlay: entity.living.hurt_time > 0 || entity.living.death_time > 0,
            death_time: if entity.living.death_time > 0 {
                entity.living.death_time as f32 + partial_tick
            } else {
                0.0
            },
            attack_time: entity.living.swing.attack_anim(partial_tick),
            // `ArmedEntityRenderState.attackArm`: the main arm, or the other
            // while the off hand swings.
            attack_arm_left: entity.main_arm_is_left() != entity.living.swing.is_off_hand(),
            age_in_ticks: entity.tick_count as f32 + partial_tick,
            shaking: entity.is_shaking(self.dimension_environment.piglins_zombify),
        })
    }

    /// Whether the chunk a position lies in is loaded.
    pub(in super::super) fn contains_chunk_at(&self, position: (f64, f64, f64)) -> bool {
        let (chunk_x, chunk_z) = entity_chunk_key(position);
        self.contains_chunk(chunk_x, chunk_z)
    }

    /// `ClientLevel.setServerSimulationDistance`.
    pub fn set_server_simulation_distance(&mut self, chunks: i32) {
        self.server_simulation_distance = chunks;
    }

    /// Where the client's own player stands, for the checks that measure from
    /// its chunk. The session moves the player, so it says where each tick.
    pub fn set_local_player_chunk(&mut self, x: f64, z: f64) {
        let chunk = |value: f64| (value.floor() as i32) >> 4;
        self.local_chunk = Some((chunk(x), chunk(z)));
    }

    /// `ClientLevel.shouldTickDeath`: a dying entity falls over only within the
    /// server's simulation distance of the local player's chunk.
    pub(in super::super) fn should_tick_death(&self, entity: &ClientEntity) -> bool {
        let Some((player_x, player_z)) = self.local_chunk else {
            return true;
        };
        let (x, _, z) = entity.position();
        let chunk = |value: f64| (value.floor() as i32) >> 4;
        voxel_core::math::chessboard_distance(chunk(x), chunk(z), player_x, player_z)
            <= self.server_simulation_distance
    }

    /// `handleDamageEvent`: the entity flinches, and its limbs swing at once.
    pub fn apply_entity_damage_event(&mut self, entity_id: i32) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.walk.set_speed(1.5);
        entity.living.hurt_time = 10;
        true
    }

    /// `handleHurtAnimation` (`LivingEntity.animateHurt`).
    pub fn apply_entity_hurt_animation(&mut self, entity_id: i32) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.living.hurt_time = 10;
        true
    }

    /// `LivingEntity.handleEntityEvent(3)` for an entity that is not a
    /// player: it dies (health spent), the sound aside.
    /// `handleAnimate`: an arm swings (actions 0 and 3). The others (waking
    /// from bed, the critical-hit particles) are not carried out.
    pub fn apply_entity_animate(&mut self, entity_id: i32, action: u8) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if entity.living_kind().is_none() || !matches!(action, 0 | 3) {
            return false;
        }
        let durations = entity.swing_durations();
        entity.living.swing.swing(action == 3, durations);
        true
    }

    /// `handleEntityEvent` for the events that are not a death: a sheep is
    /// told to eat.
    pub fn apply_entity_event(&mut self, entity_id: i32, event: i8) -> bool {
        let game_time = self.game_time;
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if event == 2 && entity.living_kind().is_some() {
            entity.on_kinetic_hit(game_time);
            return true;
        }
        if event == 19 && entity.is_squid() {
            entity
                .kind
                .squid_or_insert_with(|| Box::new(squid::SquidState::new()))
                .restart_beat();
            return true;
        }
        if event == animal::SHEEP_EAT_EVENT
            && voxel_data::entity_type::name(entity.type_id) == Some("minecraft:sheep")
        {
            entity.living.animal.eat_tick = animal::SHEEP_EAT_TICKS;
            return true;
        }
        match voxel_data::entity_type::name(entity.type_id) {
            Some("minecraft:iron_golem") => return entity.living.animal.golem_event(event),
            Some("minecraft:goat") => return entity.living.animal.goat_event(event),
            Some("minecraft:ravager") => {
                return entity
                    .kind
                    .ravager_or_insert_with(Default::default)
                    .event(event);
            }
            Some("minecraft:hoglin" | "minecraft:zoglin") => {
                return entity.living.animal.hoglin_event(event);
            }
            Some("minecraft:tnt_minecart") if event == minecart::PRIME_FUSE_EVENT => {
                entity.minecart_mut().prime_fuse();
                return true;
            }
            Some("minecraft:evoker_fangs") if event == evoker_fangs::ATTACK_EVENT => {
                entity.evoker_fangs_mut().start_attack();
                return true;
            }
            Some("minecraft:armadillo") if event == armadillo::PEEK_EVENT => {
                return entity.armadillo_mut().event(event);
            }
            Some("minecraft:creaking")
                if matches!(event, creaking::ATTACK_EVENT | creaking::INVULNERABLE_EVENT) =>
            {
                return entity.creaking_mut().event(event);
            }
            Some("minecraft:warden")
                if matches!(
                    event,
                    warden::ATTACK_EVENT | warden::TENDRIL_EVENT | warden::SONIC_BOOM_EVENT
                ) =>
            {
                let tick_count = entity.tick_count;
                return entity.warden_mut().event(event, tick_count);
            }
            Some("minecraft:wolf") if event == wolf::SHAKE_EVENT => {
                entity.living.wolf.start_shake();
                return true;
            }
            Some("minecraft:wolf") if event == wolf::CANCEL_SHAKE_EVENT => {
                entity.living.wolf.cancel_shake();
                return true;
            }
            Some("minecraft:rabbit") if event == 1 => {
                entity
                    .kind
                    .rabbit_or_insert_with(|| Box::new(rabbit::RabbitState::new()))
                    .start_hop();
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn apply_entity_death_event(&mut self, entity_id: i32) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if entity.is_player() {
            return true;
        }
        entity.health_bits = Some(0.0_f32.to_bits());
        entity.alive = false;
        // `EnderDragon.handleKillingBlow`: unless it sits, it is not dead yet but at
        // one health, in the dying phase, till the server says otherwise.
        if entity.is_ender_dragon() && !entity.ender_dragon_mut().phase().is_sitting() {
            entity.health_bits = Some(1.0_f32.to_bits());
            entity.alive = true;
            entity
                .ender_dragon_mut()
                .set_phase(ender_dragon::DragonPhase::Dying);
        }
        // `Wolf.die` stops a shake.
        entity.living.wolf.cancel_shake();
        true
    }

    pub fn entity_at(&self, entity_id: i32) -> Option<&ClientEntity> {
        self.entities.get(&entity_id)
    }

    pub fn entities(&self) -> impl Iterator<Item = &ClientEntity> {
        self.entities.values()
    }

    pub fn entity_metadata_at(&self, entity_id: i32) -> Option<&[u8]> {
        self.entities
            .get(&entity_id)
            .and_then(|entity| entity.metadata())
    }

    pub fn remove_entity(&mut self, entity_id: i32) -> bool {
        let Some(position) = self.entities.get(&entity_id).map(ClientEntity::position) else {
            return false;
        };
        self.detach_entity_from_vehicle(entity_id);
        self.eject_entity_passengers(entity_id);
        self.remove_entity_from_section(entity_id, position);
        self.entities.remove(&entity_id).is_some()
    }

    /// Update one live entity position and its persistent section membership.
    pub fn update_entity_position(&mut self, entity_id: i32, position: (f64, f64, f64)) -> bool {
        let Some(previous_position) = self.entities.get(&entity_id).map(ClientEntity::position)
        else {
            return false;
        };
        let (chunk_x, chunk_z) = entity_chunk_key(position);
        let ticking = self.contains_chunk(chunk_x, chunk_z);
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            entity.set_position(position);
            entity.ticking = entity.always_ticking || ticking;
        }
        // A shulker is held to its block, so the section is the one it is in.
        let position = self
            .entities
            .get(&entity_id)
            .map_or(position, ClientEntity::position);
        self.update_entity_section(entity_id, previous_position, position);
        true
    }

    pub fn remove_entities(&mut self, entity_ids: &[i32]) -> usize {
        entity_ids
            .iter()
            .filter(|&&entity_id| self.remove_entity(entity_id))
            .count()
    }

    /// A light update packet: `handleLightUpdatePacket` queues its layers
    /// for the next frame, and their sections are redrawn when it runs.
    pub fn apply_light_updates(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        updates: LightUpdateInput<'_>,
    ) -> LightApplication {
        self.queue_light_data(chunk_x, chunk_z, updates, true, false)
    }

    /// The light that came with a chunk: `handleLevelChunkWithLight` queues
    /// it, then switches the chunk's sections on for light.
    pub fn apply_chunk_light(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        updates: LightUpdateInput<'_>,
    ) -> LightApplication {
        self.queue_light_data(chunk_x, chunk_z, updates, false, true)
    }

    pub(in super::super) fn queue_light_data(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        updates: LightUpdateInput<'_>,
        schedule_rebuild: bool,
        enable_chunk: bool,
    ) -> LightApplication {
        let light_section_count = self.dimension.section_count().saturating_add(2);
        let invalid_sky_updates =
            count_invalid_light_updates(light_section_count, updates.sky.mask, updates.sky.updates);
        let invalid_block_updates = count_invalid_light_updates(
            light_section_count,
            updates.block.mask,
            updates.block.updates,
        );
        if invalid_sky_updates != 0 || invalid_block_updates != 0 {
            return LightApplication {
                invalid_sky_updates,
                invalid_block_updates,
                ..LightApplication::default()
            };
        }
        let application = LightApplication {
            applied_sky_sections: count_light_sections(
                light_section_count,
                updates.sky.mask,
                updates.sky.empty_mask,
            ),
            applied_block_sections: count_light_sections(
                light_section_count,
                updates.block.mask,
                updates.block.empty_mask,
            ),
            queued: !self.contains_chunk(chunk_x, chunk_z),
            ..LightApplication::default()
        };
        self.light_tasks.push_back(LightTask::Data {
            update: PendingLightUpdate::new(chunk_x, chunk_z, updates),
            schedule_rebuild,
            enable_chunk,
        });
        application
    }

    /// `ClientLevel.unload`: the chunk's column stops lighting at once.
    pub(in super::super) fn unload_chunk_light(&mut self, chunk_x: i32, chunk_z: i32) {
        let view = LightView {
            chunks: &self.chunks,
            cache: self.chunk_cache.as_ref(),
        };
        self.light
            .set_light_enabled(&view, (chunk_x, chunk_z), false);
    }

    /// `queueLightRemoval`: whether or not the chunk was loaded, its light
    /// is dropped on a later frame.
    pub(in super::super) fn queue_light_removal(&mut self, chunk_x: i32, chunk_z: i32) {
        self.light_tasks
            .push_back(LightTask::Removal { chunk_x, chunk_z });
    }

    /// `enableChunkLight`: each of the loaded chunk's sections tells the
    /// engine whether it is empty, and the chunk and its neighbours redraw.
    pub fn enable_chunk_light(&mut self, chunk_x: i32, chunk_z: i32) {
        let Some(chunk) = self.chunk_for_query(chunk_x, chunk_z) else {
            return;
        };
        let statuses: Vec<(i32, bool)> = chunk
            .sections
            .iter()
            .map(|(y, section)| (*y, section.has_only_air()))
            .collect();
        for (section_y, empty) in statuses {
            self.light
                .update_section_status((chunk_x, section_y, chunk_z), empty);
        }
        let min_section = self.dimension.min_section_y();
        let max_section = min_section + self.dimension.section_count() as i32 - 1;
        for x in chunk_x - 1..=chunk_x + 1 {
            for z in chunk_z - 1..=chunk_z + 1 {
                for y in min_section..=max_section {
                    self.light_dirty.push((x, y, z));
                }
            }
        }
    }

    /// `ClientLevel.update`'s light part, once a frame: some queued light
    /// tasks, then the engine. Returns the sections to redraw.
    pub fn update_light(&mut self) -> Vec<light::SectionKey> {
        // `pollLightUpdates`: a tenth of the queue, at least ten, or all of
        // it once it reaches a thousand.
        let size = self.light_tasks.len();
        let budget = if size < 1000 {
            (size / 10).max(10)
        } else {
            size
        };
        for _ in 0..budget {
            let Some(task) = self.light_tasks.pop_front() else {
                break;
            };
            self.run_light_task(task);
        }
        let mut dirty = std::mem::take(&mut self.light_dirty);
        dirty.extend(self.run_light_updates());
        dirty
    }

    /// Every queued light task and the engine, as enough frames would.
    pub fn flush_light(&mut self) -> Vec<light::SectionKey> {
        let mut dirty = Vec::new();
        while !self.light_tasks.is_empty() || self.light.has_light_work() {
            dirty.extend(self.update_light());
        }
        dirty
    }

    /// `LevelLightEngine.runLightUpdates`: the sections whose light changed.
    pub fn run_light_updates(&mut self) -> Vec<light::SectionKey> {
        let view = LightView {
            chunks: &self.chunks,
            cache: self.chunk_cache.as_ref(),
        };
        self.light.run_light_updates(&view)
    }

    /// Light every loaded chunk from its own sources, as the server first
    /// lights a chunk: the sections holding blocks are marked, every column
    /// is switched on and the sources spread. Not what the client does with
    /// a server's light; it measures the engine on its own.
    pub fn light_from_sources(&mut self) {
        let chunks: Vec<(i32, i32)> = self.chunks.keys().copied().collect();
        for &(chunk_x, chunk_z) in &chunks {
            let filled: Vec<i32> = self.chunks[&(chunk_x, chunk_z)]
                .sections
                .iter()
                .filter(|(_, section)| !section.has_only_air())
                .map(|(y, _)| *y)
                .collect();
            for section_y in filled {
                self.light
                    .update_section_status((chunk_x, section_y, chunk_z), false);
            }
        }
        let view = LightView {
            chunks: &self.chunks,
            cache: self.chunk_cache.as_ref(),
        };
        for &chunk in &chunks {
            self.light.set_light_enabled(&view, chunk, true);
        }
        for &chunk in &chunks {
            self.light.propagate_light_sources(&view, chunk);
        }
    }
}
