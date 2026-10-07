use super::*;

impl ClientWorld {
    pub(super) fn tick_entity_once(&mut self, entity_id: i32) {
        // One read of the entity decides every branch below; the reference
        // evaluates these conditions against the pre-tick state, so they are
        // resolved before the mutable tick borrow is taken.
        let Some((
            boat_is_non_authoritative,
            projectile_kind,
            projectile_path_is_open,
            (ridden, ridden_mount),
            flying,
        )) = self.entities.get(&entity_id).map(|entity| {
            let is_boat = vanilla_boat_max_passengers(entity.type_id).is_some();
            let authoritative = is_boat && self.boat_is_local_instance_authoritative(entity);
            let kind = vanilla_projectile_kind(entity.type_id);
            let path_open = kind.is_some_and(|kind| {
                self.projectile_path_is_open(
                    entity.position(),
                    kind.next_position(entity.position(), entity.motion(), entity.no_gravity),
                )
            });
            // A saddled horse is driven by the player in its first seat.
            let mount = self.mount_kind_of(entity).is_some();
            (
                is_boat && !authoritative,
                kind,
                path_open,
                (authoritative, mount),
                flight::flight_of(entity),
            )
        })
        else {
            return;
        };

        let Some(previous_position) = self.entities.get(&entity_id).map(ClientEntity::position)
        else {
            return;
        };
        let tick_death = self
            .entities
            .get(&entity_id)
            .is_some_and(|entity| self.should_tick_death(entity));
        let surroundings = self.species_surroundings(entity_id);
        let in_water = self.mob_in_water(entity_id);
        let targets = self.targets_of(entity_id);
        let boat_under_water = self.boat_under_water_of(entity_id);
        {
            let entity = self
                .entities
                .get_mut(&entity_id)
                .expect("entity position was checked above");
            entity.capture_old_transform();
            entity.tick_count = entity.tick_count.wrapping_add(1);
        }
        // The level ticks a mount's physics (`super.tick`) before its own `tick` counts its
        // rearing and eases its animations.
        if ridden_mount {
            self.tick_ridden_mount(entity_id);
        }
        if let Some(flight) = flying {
            self.tick_flying_projectile(entity_id, flight);
            if !self.entities.contains_key(&entity_id) {
                return;
            }
        }
        {
            let entity = self
                .entities
                .get_mut(&entity_id)
                .expect("entity position was checked above");
            if ridden {
                entity.tick_boat_start();
            } else {
                entity.tick_boat(boat_under_water);
            }
            let living = entity.living_kind();
            // A mount the player steers is moved by the player's client, not eased to the
            // server's packets, nor slowed as a mob the client does not move is.
            entity.tick_ai_step(living.filter(|_| !ridden_mount), in_water, targets);
            if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
                entity.tick_motion_interpolation();
            }
            if let Some(kind) = living {
                entity.tick_living(kind, tick_death, self.game_time);
            }
            entity.tick_species(surroundings);
            if boat_is_non_authoritative {
                entity.motion.x = JavaF64::new(0.0_f64);
                entity.motion.y = JavaF64::new(0.0_f64);
                entity.motion.z = JavaF64::new(0.0_f64);
            }
            if let Some(kind) = projectile_kind {
                tick_projectile(entity, kind, projectile_path_is_open);
            }
        }
        if ridden {
            self.tick_ridden_boat(entity_id);
        }
        if self.discard_spent_entity(entity_id) {
            return;
        }
        self.tick_item_entity(entity_id);
        let position = self
            .entities
            .get(&entity_id)
            .expect("entity remains after tick")
            .position();
        self.update_entity_section(entity_id, previous_position, position);
        self.update_ticking_flag(entity_id, position);
    }

    /// `PrimedTnt.tick` and `LightningBolt.tick`: a spent fuse, or a bolt with no flashes left,
    /// discards the entity.
    pub(super) fn discard_spent_entity(&mut self, entity_id: i32) -> bool {
        let spent = self.entities.get(&entity_id).is_some_and(|entity| {
            entity
                .primed_tnt()
                .is_some_and(primed_tnt::PrimedTntState::is_spent)
                || entity.lightning_finished()
        });
        if spent {
            self.remove_entity(entity_id);
        }
        spent
    }

    /// A living entity that eased into a chunk that is not loaded stops
    /// ticking there (`ClientLevel`'s entity callbacks on a move).
    pub(super) fn update_ticking_flag(&mut self, entity_id: i32, position: (f64, f64, f64)) {
        let (chunk_x, chunk_z) = entity_chunk_key(position);
        let in_loaded_chunk = self.contains_chunk(chunk_x, chunk_z);
        if let Some(entity) = self.entities.get_mut(&entity_id)
            && entity.living_kind().is_some()
            && !entity.always_ticking
        {
            entity.ticking = in_loaded_chunk;
        }
    }

    pub(super) fn boat_is_local_instance_authoritative(&self, entity: &ClientEntity) -> bool {
        self.local_player
            .as_ref()
            .is_some_and(|player| entity.passenger_ids.first() == Some(&player.id))
    }

    /// Java advances these entities only while the block they occupy and the
    /// block they would reach are both air. Each kind contributes its own
    /// one-tick step; this shared check keeps the two block lookups identical.
    pub(super) fn projectile_path_is_open(
        &self,
        position: (f64, f64, f64),
        next_position: (f64, f64, f64),
    ) -> bool {
        let current_block = (
            voxel_core::math::floor_f64(position.0),
            voxel_core::math::floor_f64(position.1),
            voxel_core::math::floor_f64(position.2),
        );
        let next_block = (
            voxel_core::math::floor_f64(next_position.0),
            voxel_core::math::floor_f64(next_position.1),
            voxel_core::math::floor_f64(next_position.2),
        );
        self.block_state_at(current_block.0, current_block.1, current_block.2)
            == Some(VANILLA_AIR_BLOCK_STATE_ID)
            && self.block_state_at(next_block.0, next_block.1, next_block.2)
                == Some(VANILLA_AIR_BLOCK_STATE_ID)
    }

    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn loaded_section_count(&self) -> usize {
        self.chunks.values().map(|chunk| chunk.sections.len()).sum()
    }

    pub(super) fn cached_chunk_is_current(&self, chunk_x: i32, chunk_z: i32) -> bool {
        let Some(cache) = self.chunk_cache.as_ref() else {
            return true;
        };
        cache.in_range(chunk_x, chunk_z)
            && cache.slots.get(&cache.index(chunk_x, chunk_z)) == Some(&(chunk_x, chunk_z))
    }

    pub(super) fn chunk_for_query(&self, chunk_x: i32, chunk_z: i32) -> Option<&ClientChunk> {
        self.cached_chunk_is_current(chunk_x, chunk_z)
            .then(|| self.chunks.get(&(chunk_x, chunk_z)))
            .flatten()
    }

    pub(super) fn chunk_for_update(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Option<&mut ClientChunk> {
        if !self.cached_chunk_is_current(chunk_x, chunk_z) {
            return None;
        }
        self.chunks.get_mut(&(chunk_x, chunk_z))
    }

    pub fn contains_chunk(&self, chunk_x: i32, chunk_z: i32) -> bool {
        self.chunk_for_query(chunk_x, chunk_z).is_some()
    }

    /// `handleForgetLevelChunk`: the chunk is dropped if the cache holds it,
    /// and its light is queued for removal either way.
    pub fn unload_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> bool {
        self.queue_light_removal(chunk_x, chunk_z);
        if let Some(cache) = self.chunk_cache.as_ref() {
            if !cache.in_range(chunk_x, chunk_z) {
                return false;
            }
            let index = cache.index(chunk_x, chunk_z);
            if cache.slots.get(&index) != Some(&(chunk_x, chunk_z)) {
                return false;
            }
            let removed = self.chunks.remove(&(chunk_x, chunk_z));
            if let Some(chunk) = removed.as_ref() {
                let section_ys = chunk.sections.keys().copied().collect::<Vec<_>>();
                let cache = self
                    .chunk_cache
                    .as_mut()
                    .expect("chunk cache exists during unload");
                cache.slots.remove(&index);
                cache.on_chunk_removed(chunk_x, chunk_z, section_ys);
            }
            if removed.is_some() {
                self.unload_chunk_light(chunk_x, chunk_z);
                for entity in self.entities.values_mut() {
                    if !entity.always_ticking
                        && entity_chunk_key(entity.position()) == (chunk_x, chunk_z)
                    {
                        entity.ticking = false;
                    }
                }
            }
            return removed.is_some();
        }

        let removed = self.chunks.remove(&(chunk_x, chunk_z)).is_some();
        if removed {
            self.unload_chunk_light(chunk_x, chunk_z);
            for entity in self.entities.values_mut() {
                if !entity.always_ticking
                    && entity_chunk_key(entity.position()) == (chunk_x, chunk_z)
                {
                    entity.ticking = false;
                }
            }
        }
        removed
    }

    /// Sections of loaded chunks with published light data in either layer.
    pub fn loaded_light_section_count(&self) -> usize {
        let mut sections: BTreeSet<light::SectionKey> = BTreeSet::new();
        for layer in [light::LightLayer::Sky, light::LightLayer::Block] {
            sections.extend(
                self.light
                    .visible_sections(layer)
                    .into_iter()
                    .filter(|key| self.contains_chunk(key.0, key.2)),
            );
        }
        sections.len()
    }

    /// Light tasks waiting for a frame.
    pub fn pending_light_update_count(&self) -> usize {
        self.light_tasks.len()
    }

    /// The layer's value where the engine holds or has queued a layer for the
    /// section, as `getDataLayerData` finds it.
    pub fn sky_light_at(&self, x: i32, y: i32, z: i32) -> Option<u8> {
        self.light_at(LightLayer::Sky, x, y, z)
    }

    pub fn block_light_at(&self, x: i32, y: i32, z: i32) -> Option<u8> {
        self.light_at(LightLayer::Block, x, y, z)
    }

    /// `getBrightness(LightLayer.SKY, pos)`: above the highest section with
    /// data the sky is full, and a section with no data takes the bottom row
    /// of the nearest one above.
    pub fn sky_brightness(&self, x: i32, y: i32, z: i32) -> u8 {
        self.light.light_value(light::LightLayer::Sky, (x, y, z))
    }

    /// `getBrightness(LightLayer.BLOCK, pos)`: nothing where no data arrived.
    pub fn block_brightness(&self, x: i32, y: i32, z: i32) -> u8 {
        self.light.light_value(light::LightLayer::Block, (x, y, z))
    }

    /// A sound the world makes at a place (`Level.playLocalSound`), for the sound engine to play.
    pub fn push_sound(&mut self, sound: local_player::LocalSound) {
        self.sounds.0.push(sound);
    }

    /// The sounds made since the last call, in order.
    pub fn take_sounds(&mut self) -> Vec<local_player::LocalSound> {
        std::mem::take(&mut self.sounds.0)
    }

    /// The sound a block makes (`SoundType`) at its middle: its volume and pitch scaled as the event
    /// asks, whichever of the five sounds `pick` takes. Nothing for a state with no sound type.
    pub(super) fn block_sound(
        &mut self,
        pos: (i32, i32, i32),
        state: i32,
        pick: fn(&voxel_data::block_sound::SoundTypeRow) -> &'static str,
        (volume, pitch): (fn(f32) -> f32, f32),
    ) {
        let id = i64::from(state);
        let info = block::state_or_air(id);
        let Some(sound_type) = u32::try_from(id)
            .ok()
            .and_then(|id| voxel_data::block_sound::of_state(info.block().name, id))
        else {
            return;
        };
        self.sounds.0.push(local_player::LocalSound {
            event: pick(sound_type),
            source: local_player::LocalSoundSource::Blocks,
            volume: volume(sound_type.volume),
            pitch: sound_type.pitch * pitch,
            position: (
                f64::from(pos.0) + 0.5,
                f64::from(pos.1) + 0.5,
                f64::from(pos.2) + 0.5,
            ),
        });
    }

    /// `BlockItem.place`'s sound: the block placed, a little softer than loud and lower.
    pub fn play_place_sound(&mut self, pos: (i32, i32, i32), state: i32) {
        self.block_sound(
            pos,
            state,
            voxel_data::block_sound::SoundTypeRow::place_sound,
            (|volume| (volume + 1.0) / 2.0, 0.8),
        );
    }

    /// `MultiPlayerGameMode.continueDestroyBlock`'s hit sound: a block struck while it is broken.
    pub fn play_hit_sound(&mut self, pos: (i32, i32, i32), state: i32) {
        self.block_sound(
            pos,
            state,
            voxel_data::block_sound::SoundTypeRow::hit_sound,
            (|volume| (volume + 1.0) / 8.0, 0.5),
        );
    }

    /// `LevelEventHandler.levelEvent`, for the events that make a sound the client knows: `2001`,
    /// a block broken (its state's number is the data).
    pub fn apply_level_event(&mut self, event: i32, pos: (i32, i32, i32), data: i32) {
        if event == 2001 && block::state_or_air(i64::from(data)).block().name != "minecraft:air" {
            self.block_sound(
                pos,
                data,
                voxel_data::block_sound::SoundTypeRow::break_sound,
                (|volume| (volume + 1.0) / 2.0, 0.8),
            );
        }
    }

    pub fn light_engine(&self) -> &light::LevelLightEngine {
        &self.light
    }

    pub fn loaded_block_entity_count(&self) -> usize {
        self.chunks
            .values()
            .map(|chunk| chunk.block_entities.len())
            .sum()
    }

    pub fn block_entity_at(&self, x: i32, y: i32, z: i32) -> Option<&ClientBlockEntity> {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        self.chunk_for_query(chunk_key.0, chunk_key.1)?
            .block_entities
            .get(&(x, y, z))
    }

    pub fn loaded_entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Return the number of non-empty entity sections in the client index.
    pub fn indexed_entity_section_count(&self) -> usize {
        self.entity_sections.len()
    }

    /// `ItemEntity.setItem` from the entity data: what a dropped item holds.
    pub fn apply_entity_item_stack(
        &mut self,
        entity_id: i32,
        stack: Option<ClientItemStackState>,
    ) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.item_stack = stack;
        if let Some(display) = entity.kind.display_mut() {
            display.item_changed();
        }
        true
    }

    /// `handleSetEquipment`: what the entity holds or wears in a slot.
    pub fn apply_entity_equipment(
        &mut self,
        entity_id: i32,
        slot: EquipmentSlot,
        stack: Option<ClientItemStackState>,
    ) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.equipment[slot as usize] = stack;
        true
    }

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
    pub(super) fn contains_chunk_at(&self, position: (f64, f64, f64)) -> bool {
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
    pub(super) fn should_tick_death(&self, entity: &ClientEntity) -> bool {
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

    pub(super) fn queue_light_data(
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
    pub(super) fn unload_chunk_light(&mut self, chunk_x: i32, chunk_z: i32) {
        let view = LightView {
            chunks: &self.chunks,
            cache: self.chunk_cache.as_ref(),
        };
        self.light
            .set_light_enabled(&view, (chunk_x, chunk_z), false);
    }

    /// `queueLightRemoval`: whether or not the chunk was loaded, its light
    /// is dropped on a later frame.
    pub(super) fn queue_light_removal(&mut self, chunk_x: i32, chunk_z: i32) {
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
