use super::*;

impl ClientWorld {
    pub(in super::super) fn run_light_task(&mut self, task: LightTask) {
        match task {
            LightTask::Data {
                update,
                schedule_rebuild,
                enable_chunk,
            } => {
                // `applyLightData`.
                let (chunk_x, chunk_z) = (update.chunk_x, update.chunk_z);
                let min = self.light.min_light_section();
                let count = self.light.light_section_count();
                for (layer, input) in [
                    (light::LightLayer::Sky, &update.sky),
                    (light::LightLayer::Block, &update.block),
                ] {
                    let mut updates = input.updates.iter();
                    for index in 0..count {
                        let bit = index as u64;
                        let have_data = input.mask.contains(bit);
                        if !have_data && !input.empty_mask.contains(bit) {
                            continue;
                        }
                        let data = if have_data {
                            light::DataLayer::from_bytes(updates.next().expect("counted"))
                        } else {
                            light::DataLayer::new(0)
                        };
                        let section_y = min + index;
                        self.light.queue_section_data(
                            layer,
                            (chunk_x, section_y, chunk_z),
                            Some(data),
                        );
                        if schedule_rebuild {
                            // `setSectionDirtyWithNeighbors`.
                            for dx in -1..=1 {
                                for dy in -1..=1 {
                                    for dz in -1..=1 {
                                        self.light_dirty.push((
                                            chunk_x + dx,
                                            section_y + dy,
                                            chunk_z + dz,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
                let view = LightView {
                    chunks: &self.chunks,
                    cache: self.chunk_cache.as_ref(),
                };
                self.light
                    .set_light_enabled(&view, (chunk_x, chunk_z), true);
                if enable_chunk {
                    self.enable_chunk_light(chunk_x, chunk_z);
                }
            }
            LightTask::Removal { chunk_x, chunk_z } => {
                let view = LightView {
                    chunks: &self.chunks,
                    cache: self.chunk_cache.as_ref(),
                };
                self.light
                    .set_light_enabled(&view, (chunk_x, chunk_z), false);
                let min = self.light.min_light_section();
                for section_y in min..min + self.light.light_section_count() {
                    for layer in [light::LightLayer::Block, light::LightLayer::Sky] {
                        self.light
                            .queue_section_data(layer, (chunk_x, section_y, chunk_z), None);
                    }
                }
                let min_section = self.dimension.min_section_y();
                for section_y in min_section..min_section + self.dimension.section_count() as i32 {
                    self.light
                        .update_section_status((chunk_x, section_y, chunk_z), true);
                }
            }
        }
    }

    /// Return the resolved global block-state ID at a world coordinate.
    /// `Level.getHeight(MOTION_BLOCKING, x, z)`: one above the highest
    /// state in the column that blocks motion or holds a fluid, or the
    /// bottom of the world where there is none or no chunk.
    pub fn motion_blocking_height(&self, x: i32, z: i32) -> i32 {
        let bottom = self.dimension().min_y;
        let Some(chunk) = self.chunk_for_query(x.div_euclid(16), z.div_euclid(16)) else {
            return bottom;
        };
        let column = z.rem_euclid(16) as usize * 16 + x.rem_euclid(16) as usize;
        for (section_y, section) in chunk.sections.iter().rev() {
            for local_y in (0..16).rev() {
                let Some(state) = section.block_states.get(local_y * 256 + column) else {
                    continue;
                };
                if motion_blocking(state) {
                    return section_y * 16 + local_y as i32 + 1;
                }
            }
        }
        bottom
    }

    /// `Heightmap.Types.MOTION_BLOCKING_NO_LEAVES` at a column: like
    /// [`Self::motion_blocking_height`], but leaves do not count.
    pub fn motion_blocking_no_leaves_height(&self, x: i32, z: i32) -> i32 {
        let bottom = self.dimension().min_y;
        let Some(chunk) = self.chunk_for_query(x.div_euclid(16), z.div_euclid(16)) else {
            return bottom;
        };
        let column = z.rem_euclid(16) as usize * 16 + x.rem_euclid(16) as usize;
        for (section_y, section) in chunk.sections.iter().rev() {
            for local_y in (0..16).rev() {
                let Some(state) = section.block_states.get(local_y * 256 + column) else {
                    continue;
                };
                if motion_blocking(state)
                    && !block::state_or_air(i64::from(state))
                        .block()
                        .name
                        .ends_with("_leaves")
                {
                    return section_y * 16 + local_y as i32 + 1;
                }
            }
        }
        bottom
    }

    /// `ClientboundUpdateEnabledFeaturesPacket`: whether `minecart_improvements` is on, which the
    /// minecarts made after this read to choose their behaviour.
    pub fn set_minecart_improvements(&mut self, enabled: bool) {
        self.minecart_improvements = enabled;
    }

    /// `handleMinecartAlongTrack`: the steps a minecart of the new behaviour is to play.
    pub fn apply_minecart_steps(
        &mut self,
        entity_id: i32,
        steps: &[minecart_track::MinecartStep],
    ) -> bool {
        let Some(track) = self
            .entities
            .get_mut(&entity_id)
            .and_then(|entity| entity.kind.minecart_slot_mut())
            .and_then(|slot| slot.track.as_mut())
        else {
            return false;
        };
        track.push_steps(steps);
        true
    }

    /// `OldMinecartBehavior.getPos`: where the rail puts a minecart that is at this place.
    pub fn minecart_rail_pos(&self, at: (f64, f64, f64)) -> Option<(f64, f64, f64)> {
        minecart::rail_pos(&|x, y, z| self.block_state_at(x, y, z), at)
    }

    /// `OldMinecartBehavior.getPosOffs`: the same this far along the rail from here.
    pub fn minecart_rail_pos_offset(
        &self,
        at: (f64, f64, f64),
        offset: f64,
    ) -> Option<(f64, f64, f64)> {
        minecart::rail_pos_offset(&|x, y, z| self.block_state_at(x, y, z), at, offset)
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

    pub fn block_state_at(&self, x: i32, y: i32, z: i32) -> Option<i32> {
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

    /// Collect loaded block collision shapes using Java's `BlockCollisions` traversal.
    ///
    /// The caller supplies the local collision shape for each resolved block-state ID.
    /// Missing chunks, missing shape entries, and empty shapes produce no collision.
    /// Block-state context rules, entity colliders, and world-border shapes are handled
    /// by later client-world slices.
    pub fn collect_block_collision_shapes(
        &self,
        bounding_box: Aabb,
        shapes_by_state: &BTreeMap<i32, VoxelShape>,
    ) -> Vec<ClientBlockCollision> {
        self.collect_block_collisions_with(bounding_box, |state_id, _| {
            shapes_by_state.get(&state_id).cloned()
        })
    }

    /// The collision walk, asking `shape_of` for each state it meets.
    pub(in super::super) fn collect_block_collisions_with(
        &self,
        bounding_box: Aabb,
        shape_of: impl Fn(i32, [i32; 3]) -> Option<VoxelShape>,
    ) -> Vec<ClientBlockCollision> {
        let entity_shape = Shapes::create_from_aabb(bounding_box)
            .expect("ClientWorld collision queries require normalized bounds");
        let x0 = voxel_core::math::floor_f64(bounding_box.min_x - Shapes::EPSILON) - 1;
        let x1 = voxel_core::math::floor_f64(bounding_box.max_x + Shapes::EPSILON) + 1;
        let y0 = voxel_core::math::floor_f64(bounding_box.min_y - Shapes::EPSILON) - 1;
        let y1 = voxel_core::math::floor_f64(bounding_box.max_y + Shapes::EPSILON) + 1;
        let z0 = voxel_core::math::floor_f64(bounding_box.min_z - Shapes::EPSILON) - 1;
        let z1 = voxel_core::math::floor_f64(bounding_box.max_z + Shapes::EPSILON) + 1;
        let mut collisions = Vec::new();

        // Cursor3D advances x fastest, then y, then z. Its corner cells are skipped.
        for z in z0..=z1 {
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let cursor_face_type = (x == x0 || x == x1) as u8
                        + (y == y0 || y == y1) as u8
                        + (z == z0 || z == z1) as u8;
                    if cursor_face_type == 3 {
                        continue;
                    }

                    let Some(state_id) = self.block_state_at(x, y, z) else {
                        continue;
                    };
                    let Some(local_shape) = shape_of(state_id, [x, y, z]) else {
                        continue;
                    };
                    let position = BlockPos::new(x, y, z);
                    let shape = if local_shape.equals_java(&Shapes::block()) {
                        if !bounding_box.intersects_coords(
                            f64::from(x),
                            f64::from(y),
                            f64::from(z),
                            f64::from(x.wrapping_add(1)),
                            f64::from(y.wrapping_add(1)),
                            f64::from(z.wrapping_add(1)),
                        ) {
                            continue;
                        }
                        local_shape.move_by_block_pos(position)
                    } else {
                        let shape = local_shape.move_by_block_pos(position);
                        if shape.is_empty()
                            || !Shapes::join_is_not_empty(
                                &shape,
                                &entity_shape,
                                voxel_core::shape::BooleanOp::And,
                            )
                        {
                            continue;
                        }
                        shape
                    };
                    collisions.push(ClientBlockCollision { position, shape });
                }
            }
        }

        collisions
    }

    /// Collect loaded block collisions through the ClientWorld state registry.
    pub fn collect_registered_block_collision_shapes(
        &self,
        bounding_box: Aabb,
    ) -> Vec<ClientBlockCollision> {
        let registry = &self.block_state_registry;
        self.collect_block_collisions_with(bounding_box, |state_id, at| {
            Some(registry.collision_shape_at(state_id, at))
        })
    }

    /// Check whether an entity dimensions box fits in loaded block collisions.
    ///
    /// Java's player pose check deflates the candidate box by one shape epsilon
    /// before asking the level for block collisions. Entity and world-border
    /// collision sources remain caller-owned until their world slices land.
    pub fn entity_fits_in_blocks(
        &self,
        position: Vec3,
        dimensions: ClientEntityDimensions,
        shapes_by_state: &BTreeMap<i32, VoxelShape>,
    ) -> bool {
        let bounding_box = dimensions
            .make_bounding_box(position)
            .deflate(Shapes::EPSILON);
        self.collect_block_collision_shapes(bounding_box, shapes_by_state)
            .is_empty()
    }

    /// Return whether the query has no block, entity, or world-border collision.
    ///
    /// The order is the same short-circuit order as Java's `CollisionGetter`:
    /// blocks, then entities, then the border. The source position is optional
    /// because Java skips the border branch for a null source entity.
    pub fn no_collision(
        &self,
        source: Option<ClientWorldCollisionSource>,
        bounding_box: Aabb,
        shapes_by_state: &BTreeMap<i32, VoxelShape>,
        candidates: &[ClientEntityCollisionCandidate],
    ) -> bool {
        self.no_block_collision(bounding_box, shapes_by_state)
            && self.no_entity_collision(source.is_some(), bounding_box, candidates)
            && self.no_border_collision(source, bounding_box)
    }

    /// Return whether the query has no loaded block collision.
    pub fn no_block_collision(
        &self,
        bounding_box: Aabb,
        shapes_by_state: &BTreeMap<i32, VoxelShape>,
    ) -> bool {
        self.collect_block_collision_shapes(bounding_box, shapes_by_state)
            .is_empty()
    }

    /// Return whether the query has no entity collision.
    pub fn no_entity_collision(
        &self,
        source_present: bool,
        bounding_box: Aabb,
        candidates: &[ClientEntityCollisionCandidate],
    ) -> bool {
        collect_entity_collision_shapes(source_present, bounding_box, candidates).is_empty()
    }

    /// Return whether a query has no collision with the loaded entity state.
    ///
    /// The source ID is resolved against the same entity map that stores live
    /// positions and passenger links. The current type-backed collider table
    /// covers vanilla boat and minecart families and the Happy Ghast's
    /// source-aware override; entity types with Java's base non-collidable
    /// behavior produce no candidate until their dedicated type branch lands.
    pub fn no_entity_collision_for_entity(
        &self,
        source_entity_id: Option<i32>,
        bounding_box: Aabb,
    ) -> bool {
        let source_entity_id = source_entity_id.filter(|entity_id| self.entity_exists(*entity_id));
        let source_type_id = source_entity_id
            .and_then(|entity_id| self.entities.get(&entity_id).map(|entity| entity.type_id));
        let source_position =
            source_entity_id.and_then(|entity_id| self.entity_position(entity_id));
        let candidates = self.indexed_entity_collision_candidates(
            source_entity_id,
            source_type_id,
            source_position,
            bounding_box,
        );
        collect_entity_collision_shapes_for_source(
            source_type_id,
            source_entity_id.is_some(),
            bounding_box,
            &candidates,
        )
        .is_empty()
    }

    /// Return whether a query has no collision in the loaded world state.
    pub fn no_collision_for_entity(
        &self,
        source_entity_id: Option<i32>,
        bounding_box: Aabb,
        shapes_by_state: &BTreeMap<i32, VoxelShape>,
    ) -> bool {
        let source_entity_id = source_entity_id.filter(|entity_id| self.entity_exists(*entity_id));
        let source = source_entity_id
            .and_then(|entity_id| self.entity_position(entity_id))
            .map(|position| ClientWorldCollisionSource { position });
        self.no_block_collision(bounding_box, shapes_by_state)
            && self.no_entity_collision_for_entity(source_entity_id, bounding_box)
            && self.no_border_collision(source, bounding_box)
    }

    pub(in super::super) fn add_entity_to_section(
        &mut self,
        entity_id: i32,
        position: (f64, f64, f64),
    ) {
        self.entity_sections
            .entry(entity_section_key(position))
            .or_default()
            .insert(entity_id);
    }

    pub(in super::super) fn remove_entity_from_section(
        &mut self,
        entity_id: i32,
        position: (f64, f64, f64),
    ) {
        let key = entity_section_key(position);
        let Some(entity_ids) = self.entity_sections.get_mut(&key) else {
            return;
        };
        entity_ids.remove(&entity_id);
        if entity_ids.is_empty() {
            self.entity_sections.remove(&key);
        }
    }

    pub(in super::super) fn update_entity_section(
        &mut self,
        entity_id: i32,
        previous_position: (f64, f64, f64),
        position: (f64, f64, f64),
    ) {
        let previous_key = entity_section_key(previous_position);
        let key = entity_section_key(position);
        if previous_key != key {
            self.remove_entity_from_section(entity_id, previous_position);
            self.entity_sections
                .entry(key)
                .or_default()
                .insert(entity_id);
        }
    }
}
