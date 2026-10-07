use super::*;

impl ClientWorld {
    pub(in super::super) fn tick_entity_once(&mut self, entity_id: i32) {
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
    pub(in super::super) fn discard_spent_entity(&mut self, entity_id: i32) -> bool {
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
    pub(in super::super) fn update_ticking_flag(
        &mut self,
        entity_id: i32,
        position: (f64, f64, f64),
    ) {
        let (chunk_x, chunk_z) = entity_chunk_key(position);
        let in_loaded_chunk = self.contains_chunk(chunk_x, chunk_z);
        if let Some(entity) = self.entities.get_mut(&entity_id)
            && entity.living_kind().is_some()
            && !entity.always_ticking
        {
            entity.ticking = in_loaded_chunk;
        }
    }

    pub(in super::super) fn boat_is_local_instance_authoritative(
        &self,
        entity: &ClientEntity,
    ) -> bool {
        self.local_player
            .as_ref()
            .is_some_and(|player| entity.passenger_ids.first() == Some(&player.id))
    }

    /// Java advances these entities only while the block they occupy and the
    /// block they would reach are both air. Each kind contributes its own
    /// one-tick step; this shared check keeps the two block lookups identical.
    pub(in super::super) fn projectile_path_is_open(
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

    pub(in super::super) fn cached_chunk_is_current(&self, chunk_x: i32, chunk_z: i32) -> bool {
        let Some(cache) = self.chunk_cache.as_ref() else {
            return true;
        };
        cache.in_range(chunk_x, chunk_z)
            && cache.slots.get(&cache.index(chunk_x, chunk_z)) == Some(&(chunk_x, chunk_z))
    }

    pub(in super::super) fn chunk_for_query(
        &self,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Option<&ClientChunk> {
        self.cached_chunk_is_current(chunk_x, chunk_z)
            .then(|| self.chunks.get(&(chunk_x, chunk_z)))
            .flatten()
    }

    pub(in super::super) fn chunk_for_update(
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
    pub(in super::super) fn block_sound(
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
}
