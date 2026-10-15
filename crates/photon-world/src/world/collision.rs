use crate::*;

impl ClientWorld {
    /// Java advances these entities only while the block they occupy and the
    /// block they would reach are both air. Each kind contributes its own
    /// one-tick step; this shared check keeps the two block lookups identical.
    pub(crate) fn projectile_path_is_open(
        &self,
        position: (f64, f64, f64),
        next_position: (f64, f64, f64),
    ) -> bool {
        let current_block = (
            photon_core::math::floor_f64(position.0),
            photon_core::math::floor_f64(position.1),
            photon_core::math::floor_f64(position.2),
        );
        let next_block = (
            photon_core::math::floor_f64(next_position.0),
            photon_core::math::floor_f64(next_position.1),
            photon_core::math::floor_f64(next_position.2),
        );
        self.block_state_at(current_block.0, current_block.1, current_block.2)
            == Some(VANILLA_AIR_BLOCK_STATE_ID)
            && self.block_state_at(next_block.0, next_block.1, next_block.2)
                == Some(VANILLA_AIR_BLOCK_STATE_ID)
    }

    pub(crate) fn indexed_entity_collision_candidates(
        &self,
        source_entity_id: Option<EntityId>,
        source_type_id: Option<EntityTypeId>,
        source_position: Option<Vec3>,
        bounding_box: Aabb,
    ) -> Vec<ClientEntityCollisionCandidate> {
        let mut entity_ids = BTreeSet::new();
        let x_min = entity_section_coord(bounding_box.min_x - 2.0);
        let y_min = entity_section_coord(bounding_box.min_y - 4.0);
        let z_min = entity_section_coord(bounding_box.min_z - 2.0);
        let x_max = entity_section_coord(bounding_box.max_x + 2.0);
        let y_max = entity_section_coord(bounding_box.max_y);
        let z_max = entity_section_coord(bounding_box.max_z + 2.0);
        for section_x in x_min..=x_max {
            for section_y in y_min..=y_max {
                for section_z in z_min..=z_max {
                    if let Some(ids) = self
                        .entities
                        .sections
                        .get(&(section_x, section_y, section_z))
                    {
                        entity_ids.extend(ids.iter().copied());
                    }
                }
            }
        }
        entity_ids
            .into_iter()
            .filter_map(|entity_id| self.entities.at(entity_id))
            .filter_map(|entity| {
                let same_vehicle = source_entity_id.is_some_and(|source_id| {
                    source_id != entity.id()
                        && self.root_vehicle_id(source_id) == self.root_vehicle_id(entity.id())
                });
                entity.collision_candidate(
                    source_entity_id,
                    source_type_id,
                    source_position,
                    same_vehicle,
                )
            })
            .collect()
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
    pub(crate) fn collect_block_collisions_with(
        &self,
        bounding_box: Aabb,
        shape_of: impl Fn(i32, [i32; 3]) -> Option<VoxelShape>,
    ) -> Vec<ClientBlockCollision> {
        let entity_shape = Shapes::create_from_aabb(bounding_box)
            .expect("ClientWorld collision queries require normalized bounds");
        let x0 = photon_core::math::floor_f64(bounding_box.min_x - Shapes::EPSILON) - 1;
        let x1 = photon_core::math::floor_f64(bounding_box.max_x + Shapes::EPSILON) + 1;
        let y0 = photon_core::math::floor_f64(bounding_box.min_y - Shapes::EPSILON) - 1;
        let y1 = photon_core::math::floor_f64(bounding_box.max_y + Shapes::EPSILON) + 1;
        let z0 = photon_core::math::floor_f64(bounding_box.min_z - Shapes::EPSILON) - 1;
        let z1 = photon_core::math::floor_f64(bounding_box.max_z + Shapes::EPSILON) + 1;
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
                                photon_core::shape::BooleanOp::And,
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
        source_entity_id: Option<EntityId>,
        bounding_box: Aabb,
    ) -> bool {
        let source_entity_id = source_entity_id.filter(|entity_id| self.entity_exists(*entity_id));
        let source_type_id = source_entity_id
            .and_then(|entity_id| self.entities.at(entity_id).map(|entity| entity.type_id));
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
        source_entity_id: Option<EntityId>,
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
}
