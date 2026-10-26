use crate::*;
use photon_core::ids::BlockStateId;

impl ClientWorld {
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

    /// The block entity at a place, to change.
    pub(crate) fn block_entity_mut_at(
        &mut self,
        (x, y, z): (i32, i32, i32),
    ) -> Option<&mut ClientBlockEntity> {
        self.chunk_for_update(x.div_euclid(16), z.div_euclid(16))?
            .block_entities
            .get_mut(&(x, y, z))
    }

    /// `Level.removeBlockEntity`.
    pub(crate) fn remove_block_entity(&mut self, (x, y, z): (i32, i32, i32)) {
        if let Some(chunk) = self.chunk_for_update(x.div_euclid(16), z.div_euclid(16)) {
            chunk.block_entities.remove(&(x, y, z));
        }
        self.block_tickers.unregister((x, y, z));
        self.spawner_displays.retain(|shown| shown.pos != (x, y, z));
        self.forget_spawner_extractions(|pos| pos == (x, y, z));
    }

    /// The block entity at a place, made as the block's own type when the chunk holds none
    /// (`LevelChunk.getBlockEntity` with `IMMEDIATE`).
    pub(crate) fn block_entity_or_new(
        &mut self,
        (x, y, z): (i32, i32, i32),
        type_name: &str,
    ) -> Option<&mut ClientBlockEntity> {
        let type_id = photon_data::block_entity_type::id(type_name)?;
        let time = self.game_time() as i64;
        let min_y = self.dimension().min_y;
        let ticks = self.has_ticker(type_id, (x, y, z));
        if ticks && self.block_entity_at(x, y, z).is_none() {
            self.block_tickers.register((x, y, z));
        }
        let chunk = self.chunk_for_update(x.div_euclid(16), z.div_euclid(16))?;
        Some(
            chunk.block_entities.entry((x, y, z)).or_insert_with(|| {
                ClientBlockEntity::from_wire(type_id, None).made_at(time, min_y)
            }),
        )
    }

    /// Apply a server-verified block-state update to an already loaded section.
    pub fn update_block_state(&mut self, x: i32, y: i32, z: i32, state_id: BlockStateId) -> bool {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        let Some(chunk) = self.chunks.get_mut(chunk_key.0, chunk_key.1) else {
            return false;
        };
        let section_y = y.div_euclid(16);
        let Some(section) = chunk.sections.get_mut(&section_y) else {
            return false;
        };
        let local_x = x.rem_euclid(16) as usize;
        let local_y = y.rem_euclid(16) as usize;
        let local_z = z.rem_euclid(16) as usize;
        let index = local_y * 256 + local_z * 16 + local_x;
        let Some(previous_state_id) = section.block_states.get(index) else {
            return false;
        };
        if (section.has_only_air() && state_id == VANILLA_AIR_BLOCK_STATE_ID)
            || previous_state_id == state_id
        {
            return false;
        }
        let was_empty = section.has_only_air();
        if section.set_block_state(index, state_id).is_none() {
            return false;
        }
        // `LevelChunk.setBlockState`: the light hears of a section turning
        // empty or not, and of a change in how light passes the block.
        let is_empty = section.has_only_air();
        chunk.update_heightmaps(local_z * 16 + local_x, y, state_id);
        // A chest block has its own block entity on the client, and a lid that goes with it.
        self.block_animations.remove(&(x, y, z));
        // `LevelChunk.setBlockState`: a block of another kind has no use for the old block's
        // entity.
        if block::state_or_air(previous_state_id).block != block::state_or_air(state_id).block {
            chunk.block_entities.remove(&(x, y, z));
            self.block_tickers.unregister((x, y, z));
            self.spawner_displays.retain(|shown| shown.pos != (x, y, z));
            super::spawner::forget_extracted(&mut self.spawners_extracted, (x, y, z));
        }
        chunk.mark_chest((x, y, z), state_id);
        if was_empty != is_empty {
            self.light
                .update_section_status((chunk_key.0, section_y, chunk_key.1), is_empty);
        }
        if light::has_different_light_properties(previous_state_id, state_id) {
            let sections = &chunk.sections;
            let read = |lx: i32, y: i32, lz: i32| {
                sections
                    .get(&y.div_euclid(16))
                    .and_then(|section| {
                        section
                            .block_states
                            .get((y.rem_euclid(16) * 256 + lz * 16 + lx) as usize)
                    })
                    .unwrap_or(VANILLA_AIR_BLOCK_STATE_ID)
            };
            if let Some(sources) = chunk.sky_sources.as_mut() {
                sources.update(read, local_x as i32, y, local_z as i32);
            }
            self.light.check_block((x, y, z));
        }
        self.rebind_ticker((x, y, z), state_id);
        true
    }

    /// Apply the horizontal `StairBlock.updateShape` state transition against
    /// the currently loaded six-neighbor block states.
    /// `StairBlock.updateShape` after a side neighbour changed: the stair
    /// takes the shape its neighbours give it. `false` when the block there
    /// is not a stair.
    pub fn apply_stair_neighbor_shape_update(&mut self, x: i32, y: i32, z: i32) -> bool {
        let Some(state_id) = self.block_state_at(x, y, z) else {
            return false;
        };
        let is_stairs = block::state(state_id).is_some_and(|state| {
            state
                .block()
                .class
                .is(photon_data::block::classes::BlockClass::StairBlock)
        });
        if !is_stairs {
            return false;
        }
        let state_at = |x: i32, y: i32, z: i32| self.block_state_or_air(x, y, z);
        let block::rules::Rule::Known(updated_state_id) = block::rules::update_shape(
            state_id,
            &state_at,
            self.block_state_registry().tags(),
            (x, y, z),
            photon_core::direction::Direction::North,
        ) else {
            return false;
        };
        if updated_state_id == state_id {
            return true;
        }
        self.update_block_state(x, y, z, updated_state_id)
    }

    pub fn apply_block_entities<'a, I>(&mut self, chunk_x: i32, chunk_z: i32, entities: I) -> usize
    where
        I: IntoIterator<Item = BlockEntityInput<'a>>,
    {
        let time = self.game_time() as i64;
        let min_y = self.dimension().min_y;
        let Some(chunk) = self.chunk_for_update(chunk_x, chunk_z) else {
            return 0;
        };
        let mut applied = 0;
        let mut ticking = Vec::new();
        for entity in entities {
            if entity.x.div_euclid(16) != chunk_x || entity.z.div_euclid(16) != chunk_z {
                continue;
            }
            let key = (entity.x, entity.y, entity.z);
            chunk.block_entities.insert(
                key,
                ClientBlockEntity::from_wire(entity.type_id, entity.tag.map(<[u8]>::to_vec))
                    .made_at(time, min_y),
            );
            ticking.push((key, entity.type_id));
            applied += 1;
        }
        // `BaseSpawner.load`: a spawner that is loaded again has to be drawn again.
        for (key, _) in &ticking {
            super::spawner::forget_extracted(&mut self.spawners_extracted, *key);
        }
        // `LevelChunk.addAndRegisterBlockEntity`: a ticker for each, in the order they came.
        for (key, type_id) in ticking {
            if self.has_ticker(type_id, key) {
                self.block_tickers.register(key);
            }
        }
        applied
    }

    pub fn apply_block_entity_update(&mut self, entity: BlockEntityInput<'_>) -> bool {
        let chunk_key = (entity.x.div_euclid(16), entity.z.div_euclid(16));
        let time = self.game_time() as i64;
        let min_y = self.dimension().min_y;
        let Some(chunk) = self.chunk_for_update(chunk_key.0, chunk_key.1) else {
            return false;
        };
        let key = (entity.x, entity.y, entity.z);
        let mut updated =
            ClientBlockEntity::from_wire(entity.type_id, entity.tag.map(<[u8]>::to_vec))
                .made_at(time, min_y);
        // `handleBlockEntityData` loads into the entity that is there: what events set stays.
        let existed = chunk.block_entities.contains_key(&key);
        if let Some(old) = chunk
            .block_entities
            .get(&key)
            .filter(|old| old.type_id == entity.type_id)
        {
            // What `loadAdditional` reads comes from the new tag, the rest stays.
            let (age, target) = (updated.events.age, updated.events.conduit.target);
            updated.events = old.events.clone();
            updated.events.age = age;
            updated.events.conduit.target = target;
        }
        chunk.block_entities.insert(key, updated);
        super::spawner::forget_extracted(&mut self.spawners_extracted, key);
        if !existed && self.has_ticker(entity.type_id, key) {
            self.block_tickers.register(key);
        }
        true
    }

    /// `Level.getHeight(MOTION_BLOCKING, x, z)`: one above the highest
    /// state in the column that blocks motion or holds a fluid, or the
    /// bottom of the world where there is none or no chunk.
    pub fn motion_blocking_height(&self, x: i32, z: i32) -> i32 {
        self.level_height(HeightmapType::MotionBlocking, x, z)
    }

    /// `Heightmap.Types.MOTION_BLOCKING_NO_LEAVES` at a column: like
    /// [`Self::motion_blocking_height`], but leaves do not count.
    pub fn motion_blocking_no_leaves_height(&self, x: i32, z: i32) -> i32 {
        self.level_height(HeightmapType::MotionBlockingNoLeaves, x, z)
    }

    /// How far the dragon egg's place on the End's podium is from a point
    /// (`BlockPos.distToCenterSqr`), as `EnderDragonRenderer` measures it: the
    /// podium stands at the origin, on the highest block there that is not a leaf.
    pub fn end_podium_distance_squared(&self, at: (f64, f64, f64)) -> f64 {
        let y = f64::from(self.motion_blocking_no_leaves_height(0, 0));
        let (dx, dy, dz) = (0.5 - at.0, y + 0.5 - at.1, 0.5 - at.2);
        dx * dx + dy * dy + dz * dz
    }

    /// How many bytes the loaded chunks' block states take, and how many
    /// they would take stored as a number for every block.
    pub fn block_state_bytes(&self) -> (usize, usize) {
        self.chunks
            .values()
            .flat_map(|chunk| chunk.sections.values())
            .fold((0, 0), |(stored, plain), section| {
                (
                    stored + section.block_states.bytes(),
                    plain + paletted::SECTION_VOLUME * std::mem::size_of::<i32>(),
                )
            })
    }

    /// The block state, or air where no chunk was loaded.
    pub fn block_state_or_air(&self, x: i32, y: i32, z: i32) -> BlockStateId {
        self.block_state_at(x, y, z)
            .unwrap_or(crate::constants::VANILLA_AIR_BLOCK_STATE_ID)
    }

    pub fn block_state_at(&self, x: i32, y: i32, z: i32) -> Option<BlockStateId> {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        let chunk = self.chunk_for_query(chunk_key.0, chunk_key.1)?;
        let section_y = y.div_euclid(16);
        let section = chunk.sections.get(&section_y)?;
        let local_x = x.rem_euclid(16) as usize;
        let local_y = y.rem_euclid(16) as usize;
        let local_z = z.rem_euclid(16) as usize;
        let index = local_y * 256 + local_z * 16 + local_x;
        section.block_states.get(index)
    }
}

impl ClientWorld {
    /// Whether the client ticks the block entity of a type at a place: the end gateway's beam
    /// animation, the enchanting table's book, the beacon's scan, the conduit's shell, and the
    /// potent sulfur block that is not dry, and the campfire that is lit.
    fn has_ticker(&self, type_id: i32, (x, y, z): (i32, i32, i32)) -> bool {
        match photon_data::block_entity_type::name(type_id) {
            Some(
                "minecraft:end_gateway"
                | "minecraft:enchanting_table"
                | "minecraft:beacon"
                | "minecraft:conduit"
                | "minecraft:vault"
                | "minecraft:mob_spawner"
                | "minecraft:trial_spawner",
            ) => true,
            Some("minecraft:potent_sulfur") => {
                super::has_sulfur_ticker(self.block_state_or_air(x, y, z))
            }
            // `CampfireBlock.getTicker`: only a lit one ticks on a client.
            Some("minecraft:campfire") => {
                crate::block::rules::property(self.block_state_or_air(x, y, z), "lit")
                    == Some("true")
            }
            _ => false,
        }
    }
}
