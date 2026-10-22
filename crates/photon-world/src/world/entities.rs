use crate::*;

impl ClientWorld {
    /// `PrimedTnt.tick` and `LightningBolt.tick`: a spent fuse, or a bolt with no flashes left,
    /// discards the entity.
    pub(crate) fn discard_spent_entity(&mut self, entity_id: EntityId) -> bool {
        let spent = self.entities.at(entity_id).is_some_and(|entity| {
            entity
                .primed_tnt()
                .is_some_and(objects::primed_tnt::PrimedTntState::is_spent)
                || entity.lightning_finished()
        });
        if spent {
            self.remove_entity(entity_id);
        }
        spent
    }

    /// Where an entity is drawn between two ticks: its position
    /// (`getPosition(partialTick)`), its turn and its head's.
    pub fn entity_snapshot(
        &self,
        entity_id: EntityId,
        partial_tick: f32,
    ) -> Option<EntitySnapshot> {
        let entity = self
            .entities
            .at(entity_id)
            .or_else(|| self.spawner_display_entity(entity_id))?;
        let (yaw, pitch, head_yaw) = entity.interpolated_rotation(partial_tick);
        let body_yaw = entity.interpolated_body_y_rotation(partial_tick);
        let upside_down = entity.is_upside_down().then(|| entity.leash_size().height);
        // `extractRenderState`: an upside-down mob looks the other way up and aside.
        let (pitch, head_yaw) = if upside_down.is_some() {
            (
                -pitch,
                body_yaw - photon_core::math::wrap_degrees_f32(head_yaw - body_yaw),
            )
        } else {
            (pitch, head_yaw)
        };
        Some(EntitySnapshot {
            position: entity.interpolated_position(partial_tick),
            yaw,
            pitch,
            head_yaw,
            body_yaw,
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
            shaking: entity.is_shaking(self.level.dimension_environment.piglins_zombify),
            burning: entity.is_burning(),
            friendly_ghost: self.entities.friendly_ghosts.contains(&entity_id),
            spawner_display: self.spawner_display_pose(entity_id, partial_tick),
            upside_down,
        })
    }

    /// The entities the server has sent.
    pub fn entities(&self) -> &Entities {
        &self.entities
    }

    /// The entities the server has sent, to change.
    pub fn entities_mut(&mut self) -> &mut Entities {
        &mut self.entities
    }

    pub fn remove_entity(&mut self, entity_id: EntityId) -> bool {
        let Some(position) = self.entities.at(entity_id).map(ClientEntity::position) else {
            return false;
        };
        self.detach_entity_from_vehicle(entity_id);
        self.eject_entity_passengers(entity_id);
        self.entities_mut().remove_from_section(entity_id, position);
        self.minecart_track_output.remove(&entity_id);
        self.entities.remove(entity_id).is_some()
    }

    /// Update one live entity position and its persistent section membership.
    pub fn update_entity_position(
        &mut self,
        entity_id: EntityId,
        position: (f64, f64, f64),
    ) -> bool {
        let Some(previous_position) = self.entities.at(entity_id).map(ClientEntity::position)
        else {
            return false;
        };
        let (chunk_x, chunk_z) = entity_chunk_key(position);
        let ticking = self.contains_chunk(chunk_x, chunk_z);
        if let Some(entity) = self.entities.at_mut(entity_id) {
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

    pub fn remove_entities(&mut self, entity_ids: &[EntityId]) -> usize {
        entity_ids
            .iter()
            .filter(|&&entity_id| self.remove_entity(entity_id))
            .count()
    }

    pub(crate) fn entity_position(&self, entity_id: EntityId) -> Option<Vec3> {
        if let Some(entity) = self.entities.at(entity_id) {
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
        let owner = Some(EntityId(entity.data)).filter(|&id| self.entity_exists(id));
        // An entity is one kind for as long as it lives; a display or a minecart of the new
        // behaviour is known from its type, the rest make their state when it first comes.
        let initial_kind = if let Some(display_kind) = entity::display_kind_of(entity.type_id) {
            entity_kind::EntityKind::Display(Box::new(objects::display::DisplayState::new(
                display_kind,
            )))
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
                    photon_core::math::floor_f64(entity.x),
                    photon_core::math::floor_f64(entity.y),
                    photon_core::math::floor_f64(entity.z),
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
                boat_ride: ride::boat::RideState::default(),
                mount_ride: ride::mount::MountState::default(),
                flight: objects::flight::FlightState {
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
                health: None,
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
                leash_holder_id: EntityId(0),
                lightning_done: false,
                shot_at_angle: false,
                attached_target: None,
            },
        );
        self.entities_mut().enter_world(entity.entity_id);
        if let Some(shulker) = self
            .entities
            .at_mut(entity.entity_id)
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
            .at_mut(entity.entity_id)
            .filter(|entity| entity.is_warden())
            && entity.data == 1
        {
            // `Warden.recreateFromPacket`: the packet's data says it is emerging.
            warden.set_pose(ClientEntityPose::Emerging);
        }
        self.snap_local_player_turn(entity.entity_id, head_rotation, entity.rotation.0);
        true
    }

    pub(crate) fn update_entity_section(
        &mut self,
        entity_id: EntityId,
        previous_position: (f64, f64, f64),
        position: (f64, f64, f64),
    ) {
        let previous_key = entity_section_key(previous_position);
        let key = entity_section_key(position);
        if previous_key != key {
            self.entities_mut()
                .remove_from_section(entity_id, previous_position);
            self.entities
                .sections
                .entry(key)
                .or_default()
                .insert(entity_id);
        }
    }

    pub(crate) fn resolve_entity_position(
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
}

mod events;
mod motion;
mod passengers;
mod rockets;
mod seeds;
mod state;
mod store;
pub use rockets::RocketHand;
pub use store::Entities;
mod targets;
mod ticking;

impl Entities {
    pub fn count(&self) -> usize {
        self.len()
    }

    /// Return the number of non-empty entity sections in the client index.
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    pub fn at(&self, entity_id: EntityId) -> Option<&ClientEntity> {
        self.get(&entity_id)
    }

    pub fn metadata(&self, entity_id: EntityId) -> Option<&[u8]> {
        self.get(&entity_id).and_then(|entity| entity.metadata())
    }

    /// A new entity takes its place in its section; an item frame or a painting first moves to the
    /// centre of its box (`HangingEntity.recalculateBoundingBox`).
    pub(crate) fn enter_world(&mut self, entity_id: EntityId) {
        let Some(entity) = self.at_mut(entity_id) else {
            return;
        };
        entity.recentre_hanging();
        let position = entity.position();
        self.add_to_section(entity_id, position);
    }

    pub(crate) fn add_to_section(&mut self, entity_id: EntityId, position: (f64, f64, f64)) {
        self.sections
            .entry(entity_section_key(position))
            .or_default()
            .insert(entity_id);
    }

    pub(crate) fn remove_from_section(&mut self, entity_id: EntityId, position: (f64, f64, f64)) {
        let key = entity_section_key(position);
        let Some(entity_ids) = self.sections.get_mut(&key) else {
            return;
        };
        entity_ids.remove(&entity_id);
        if entity_ids.is_empty() {
            self.sections.remove(&key);
        }
    }
}

impl ClientWorld {
    pub(crate) fn entity_exists(&self, entity_id: EntityId) -> bool {
        self.entities.contains_key(&entity_id)
            || self
                .local_player
                .as_ref()
                .is_some_and(|player| player.id == entity_id)
    }
}
