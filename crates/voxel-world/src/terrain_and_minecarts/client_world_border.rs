use super::*;

impl ClientWorld {
    pub(in super::super) fn indexed_entity_collision_candidates(
        &self,
        source_entity_id: Option<i32>,
        source_type_id: Option<i32>,
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
                    if let Some(ids) = self.entity_sections.get(&(section_x, section_y, section_z))
                    {
                        entity_ids.extend(ids.iter().copied());
                    }
                }
            }
        }
        entity_ids
            .into_iter()
            .filter_map(|entity_id| self.entities.get(&entity_id))
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

    pub(in super::super) fn root_vehicle_id(&self, entity_id: i32) -> i32 {
        let mut current = entity_id;
        let mut visited = BTreeSet::new();
        while visited.insert(current) {
            let Some(vehicle_id) = self.entity_vehicle_id(current) else {
                break;
            };
            current = vehicle_id;
        }
        current
    }

    pub(in super::super) fn entity_position(&self, entity_id: i32) -> Option<Vec3> {
        if let Some(entity) = self.entities.get(&entity_id) {
            let (x, y, z) = entity.position();
            return Some(Vec3::new(x, y, z));
        }
        self.local_player
            .as_ref()
            .filter(|player| player.id == entity_id)
            .map(|player| {
                let (x, y, z) = player.position();
                Vec3::new(x, y, z)
            })
    }

    /// `CollisionGetter.borderCollision` / `Entity.collectColliders`: the world border's shape when
    /// the source stands close inside it (`WorldBorder.isInsideCloseToBorder`), `None` otherwise.
    pub fn border_collider(&self, source: Vec3, bounding_box: Aabb) -> Option<VoxelShape> {
        let border = &self.world_border;
        let margin = abs_max_f64(bounding_box.x_size(), bounding_box.z_size()).max(1.0);
        let min_x = border.min_x(0.0);
        let max_x = border.max_x(0.0);
        let min_z = border.min_z(0.0);
        let max_z = border.max_z(0.0);
        let distance = (source.x - min_x)
            .min(max_x - source.x)
            .min(source.z - min_z)
            .min(max_z - source.z);
        let inside_close = distance < margin * 2.0
            && source.x >= min_x - margin
            && source.x < max_x + margin
            && source.z >= min_z - margin
            && source.z < max_z + margin;
        inside_close.then(|| border.collision_shape())
    }

    /// Return whether the query has no collision with the current world border.
    pub fn no_border_collision(
        &self,
        source: Option<ClientWorldCollisionSource>,
        bounding_box: Aabb,
    ) -> bool {
        let Some(source) = source else {
            return true;
        };
        let Some(border_shape) = self.border_collider(source.position, bounding_box) else {
            return true;
        };
        let query_shape = Shapes::create_from_aabb(bounding_box)
            .expect("ClientWorld collision queries require normalized bounds");
        !Shapes::join_is_not_empty(&border_shape, &query_shape, BooleanOp::And)
    }

    /// Apply a server-verified block-state update to an already loaded section.
    pub fn update_block_state(&mut self, x: i32, y: i32, z: i32, state_id: i32) -> bool {
        let chunk_key = (x.div_euclid(16), z.div_euclid(16));
        let Some(chunk) = self.chunk_for_update(chunk_key.0, chunk_key.1) else {
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
        // A chest block has its own block entity on the client, and a lid that goes with it.
        self.block_animations.remove(&(x, y, z));
        if let Some(chunk) = self.chunks.get_mut(&chunk_key) {
            // `LevelChunk.setBlockState`: a block of another kind has no use for the old block's
            // entity.
            if block::state_or_air(i64::from(previous_state_id)).block
                != block::state_or_air(i64::from(state_id)).block
            {
                chunk.block_entities.remove(&(x, y, z));
            }
            if chest::is_drawn_state(state_id) {
                chunk.chests.insert((x, y, z));
            } else {
                chunk.chests.remove(&(x, y, z));
            }
        }
        if was_empty != is_empty {
            self.light
                .update_section_status((chunk_key.0, section_y, chunk_key.1), is_empty);
        }
        if light::has_different_light_properties(previous_state_id, state_id) {
            let chunk = self.chunks.get_mut(&chunk_key).expect("updated above");
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
        let is_stairs = block::state(state_id as u32).is_some_and(|state| {
            state
                .block()
                .class
                .is(voxel_data::block::classes::BlockClass::StairBlock)
        });
        if !is_stairs {
            return false;
        }
        let state_at = |x: i32, y: i32, z: i32| self.block_state_at(x, y, z).unwrap_or(0);
        let block_rules::Rule::Known(updated_state_id) = block_rules::update_shape(
            state_id,
            &state_at,
            (x, y, z),
            voxel_core::direction::Direction::North,
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
        let Some(chunk) = self.chunk_for_update(chunk_x, chunk_z) else {
            return 0;
        };
        let mut applied = 0;
        for entity in entities {
            if entity.x.div_euclid(16) != chunk_x || entity.z.div_euclid(16) != chunk_z {
                continue;
            }
            chunk.block_entities.insert(
                (entity.x, entity.y, entity.z),
                ClientBlockEntity::from_wire(entity.type_id, entity.tag.map(<[u8]>::to_vec)),
            );
            applied += 1;
        }
        applied
    }

    pub fn apply_block_entity_update(&mut self, entity: BlockEntityInput<'_>) -> bool {
        let chunk_key = (entity.x.div_euclid(16), entity.z.div_euclid(16));
        let Some(chunk) = self.chunk_for_update(chunk_key.0, chunk_key.1) else {
            return false;
        };
        chunk.block_entities.insert(
            (entity.x, entity.y, entity.z),
            ClientBlockEntity::from_wire(entity.type_id, entity.tag.map(<[u8]>::to_vec)),
        );
        true
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the struct literal names every field of a new entity"
    )]
    pub fn apply_entity(&mut self, mut entity: EntityInput) -> bool {
        (entity.x, entity.y, entity.z) =
            block_attached_position(entity.type_id, (entity.x, entity.y, entity.z));
        if self.entities.contains_key(&entity.entity_id) {
            self.remove_entity(entity.entity_id);
        }
        let ticking = self.contains_chunk_at((entity.x, entity.y, entity.z));
        let new_minecart =
            self.minecart_improvements && entity::is_vanilla_minecart_entity_type(entity.type_id);
        let head_rotation = JavaF32::new(entity.rotation.2);
        // `Projectile.recreateFromPacket`: the packet's data names the owner, if it is here.
        let owner = self.entity_exists(entity.data).then_some(entity.data);
        // An entity is one kind for as long as it lives; a display or a minecart of the new
        // behaviour is known from its type, the rest make their state when it first comes.
        let initial_kind = if let Some(display_kind) = entity::display_kind_of(entity.type_id) {
            entity_kind::EntityKind::Display(Box::new(display::DisplayState::new(display_kind)))
        } else if new_minecart {
            entity_kind::EntityKind::Minecart(Box::new(entity_kind::MinecartSlot {
                state: None,
                track: Some(Default::default()),
            }))
        } else {
            entity_kind::EntityKind::Plain
        };
        self.entities.insert(
            entity.entity_id,
            ClientEntity {
                id: entity.entity_id,
                uuid: entity.uuid,
                type_id: entity.type_id,
                tick_count: 0,
                first_tick: true,
                start_block: (
                    voxel_core::math::floor_f64(entity.x),
                    voxel_core::math::floor_f64(entity.y),
                    voxel_core::math::floor_f64(entity.z),
                ),
                position: DVec3::new(entity.x, entity.y, entity.z),
                old_position: DVec3::new(entity.x, entity.y, entity.z),
                motion: DVec3::new(entity.movement.0, entity.movement.1, entity.movement.2),
                motion_lerp_target: DVec3::new(
                    entity.movement.0,
                    entity.movement.1,
                    entity.movement.2,
                ),
                motion_lerp_steps: 0,
                x_rot: JavaF32::new(entity.rotation.1),
                y_rot: JavaF32::new(entity.rotation.0),
                y_head_rot: JavaF32::new(entity.rotation.2),
                y_body_rot: JavaF32::new(entity.rotation.0),
                old_x_rot: JavaF32::new(entity.rotation.1),
                old_y_rot: JavaF32::new(entity.rotation.0),
                old_y_head_rot: JavaF32::new(entity.rotation.2),
                old_y_body_rot: JavaF32::new(entity.rotation.0),
                delta_rotation: JavaF32::new(0.0_f32),
                boat_input: BoatInput::EMPTY,
                boat_ride: boat_ride::RideState::default(),
                mount_ride: mount_ride::MountState::default(),
                flight: flight::FlightState {
                    owner,
                    left_owner: false,
                },
                paddle_left: false,
                paddle_right: false,
                head_rotation_target: JavaF32::new(entity.rotation.2),
                head_lerp_steps: 0,
                interpolation: None,
                on_ground: false,
                tracking: Tracking::at(entity.x, entity.y, entity.z),
                always_ticking: entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID,
                ticking: ticking || entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID,
                walk: living::WalkAnimation::default(),
                swim_amount: JavaF32::new(0.0_f32),
                old_swim_amount: JavaF32::new(0.0_f32),
                head_stable_time: 0,
                last_stable_y_head_rot: JavaF32::new(entity.rotation.2),
                item_stack: None,
                equipment: Default::default(),
                bob_offset: bob_offset(entity.uuid),
                data: entity.data,
                metadata: None,
                shared_flags: 0,
                living_flags: 0,
                mob_flags: 0,
                air_supply: DEFAULT_AIR_SUPPLY,
                health_bits: None,
                living: EntityState::default(),
                arrow_count: 0,
                stinger_count: 0,
                effect_ambience: false,
                effect_particle_type_ids: Vec::new(),
                sleeping_pos: None,
                custom_name_present: false,
                custom_name_visible: false,
                no_gravity: false,
                silent: false,
                pose: ClientEntityPose::Standing,
                baby: false,
                alive: true,
                spectator: false,
                on_climbable: false,
                flying: false,
                happy_ghast_still_timeout: false,
                happy_ghast_is_vehicle: false,
                ticks_frozen: 0,
                vehicle_id: None,
                passenger_ids: Vec::new(),
                attributes: Vec::new(),
                kind: initial_kind,
                item_frame_rotation: 0,
                leash_holder_id: 0,
                lightning_done: false,
                shot_at_angle: false,
            },
        );
        self.add_entity_to_section(entity.entity_id, (entity.x, entity.y, entity.z));
        if let Some(shulker) = self
            .entities
            .get_mut(&entity.entity_id)
            .filter(|entity| entity.is_shulker())
        {
            // `Shulker.recreateFromPacket`: the body faces south; `snapTo`
            // holds it to the middle of its block.
            shulker.y_body_rot = JavaF32::new(0.0_f32);
            shulker.old_y_body_rot = JavaF32::new(0.0_f32);
            let (position, (y_rot, x_rot, _)) = (shulker.position(), shulker.rotation());
            shulker.snap_to(position, y_rot, x_rot);
        }
        if let Some(warden) = self
            .entities
            .get_mut(&entity.entity_id)
            .filter(|entity| entity.is_warden())
            && entity.data == 1
        {
            // `Warden.recreateFromPacket`: the packet's data says it is emerging.
            warden.set_pose(ClientEntityPose::Emerging);
        }
        self.snap_local_player_turn(entity.entity_id, head_rotation, entity.rotation.0);
        true
    }

    /// The local player's own head and body turn, set to what an entity
    /// packet for it says.
    pub(in super::super) fn snap_local_player_turn(
        &mut self,
        entity_id: i32,
        head_rotation: JavaF32,
        body_yaw: f32,
    ) {
        if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity_id)
        {
            player.y_head_rot = head_rotation;
            player.old_y_head_rot = head_rotation;
            player.head_rotation_target = head_rotation;
            player.head_lerp_steps = 0;
            player.y_body_rot = JavaF32::new(body_yaw);
            player.old_y_body_rot = JavaF32::new(body_yaw);
        }
    }

    /// Apply the collision-relevant Happy Ghast state used by the client query.
    pub fn apply_happy_ghast_collision_state(
        &mut self,
        entity_id: i32,
        baby: bool,
        alive: bool,
        still_timeout: bool,
        is_vehicle: bool,
    ) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if entity.type_id != VANILLA_HAPPY_GHAST_ENTITY_TYPE_ID {
            return false;
        }
        entity.baby = baby;
        entity.alive = alive;
        entity.happy_ghast_still_timeout = still_timeout;
        entity.happy_ghast_is_vehicle = is_vehicle;
        true
    }

    /// Apply the collision-relevant alive and spectator state of a living entity.
    pub fn apply_entity_living_collision_state(
        &mut self,
        entity_id: i32,
        alive: bool,
        spectator: bool,
        on_climbable: bool,
    ) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.alive = alive;
        entity.spectator = spectator;
        entity.on_climbable = on_climbable;
        true
    }
}
