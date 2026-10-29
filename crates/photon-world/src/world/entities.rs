use crate::*;

impl ClientWorld {
    /// `LightningBolt.tick`: on its first tick a bolt plays its thunder, heard from afar, and its impact.
    fn strike_sounds(&mut self, entity_id: EntityId) {
        let Some(mut entity) = self.entities.at_mut(entity_id) else {
            return;
        };
        let Some((thunder, impact)) = entity.take_lightning_pitches() else {
            return;
        };
        let at = entity.position();
        for (event, volume, pitch) in [
            ("minecraft:entity.lightning_bolt.thunder", 10000.0, thunder),
            ("minecraft:entity.lightning_bolt.impact", 2.0, impact),
        ] {
            let sound = local_player::LocalSound::at(
                event,
                local_player::LocalSoundSource::Weather,
                (volume, pitch),
                at,
            );
            self.push_sound(sound);
        }
    }

    /// `PrimedTnt.tick` and `LightningBolt.tick`: a spent fuse, or a bolt with no flashes left,
    /// discards the entity.
    pub(crate) fn discard_spent_entity(&mut self, entity_id: EntityId) -> bool {
        self.strike_sounds(entity_id);
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
        let mut snapshot = EntitySnapshot {
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
            full_block_light: entity.is_burning() || entity.renderer_full_block_light(),
            friendly_ghost: self.entities.friendly_ghosts.contains(&entity_id),
            spawner_display: self.spawner_display_pose(entity_id, partial_tick),
            upside_down: None,
            sleeping: self.sleeping_lie(entity),
            light_probe: (0.0, 0.0, 0.0),
        };
        snapshot.light_probe = self.light_probe(
            entity,
            snapshot.position,
            // A player's own size is scaled here (`LivingEntity.getDimensions`), a sleeping one's is not.
            f64::from(entity.leash_size().eye_height)
                * if entity.is_player() && entity.pose() != ClientEntityPose::Sleeping {
                    f64::from(entity.scale())
                } else {
                    1.0
                },
        );
        if let Some(height) = upside_down {
            snapshot.turn_upside_down(height);
        }
        Some(snapshot)
    }

    /// How a living entity in the sleeping pose lies (`LivingEntity.getBedOrientation`).
    fn sleeping_lie(&self, entity: EntityView<'_>) -> Option<(Option<(i32, i32)>, f32)> {
        if entity.pose() != ClientEntityPose::Sleeping {
            return None;
        }
        let step = entity
            .sleeping_pos()
            .and_then(|pos| crate::block::bed::bed_orientation(self, (pos.x, pos.y, pos.z)))
            .map(|facing| {
                let (x, _, z) = facing.step();
                (x, z)
            });
        Some((step, entity.leash_size().eye_height))
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
        let Some(position) = self.entities.at(entity_id).map(|entity| entity.position()) else {
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
        let Some(previous_position) = self.entities.at(entity_id).map(|entity| entity.position())
        else {
            return false;
        };
        let (chunk_x, chunk_z) = entity_chunk_key(position);
        let ticking = self.contains_chunk(chunk_x, chunk_z);
        if let Some(mut entity) = self.entities.at_mut(entity_id) {
            entity.set_position(position);
            entity.ticking = entity.always_ticking || ticking;
        }
        // A shulker is held to its block, so the section is the one it is in.
        let position = self
            .entities
            .get(&entity_id)
            .map_or(position, |entity| entity.position());
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
        // An entity is one kind for as long as it lives, and makes its state as it spawns.
        let mut initial_kind = entity::initial_kind(entity.type_id, entity.entity_id);
        if let (true, entity_kind::EntityKind::Minecart(slot)) = (new_minecart, &mut initial_kind) {
            slot.track = Some(Default::default());
        }
        self.entities.insert(
            entity.entity_id,
            EntityParts {
                rotation: Rotation::spawned(
                    entity.rotation.0,
                    entity.rotation.1,
                    entity.rotation.2,
                ),
                lerp: Interpolation::spawned(entity.rotation.2),
                kind: initial_kind,
                worn: Equipment::default(),
                living: EntityState::default(),
                ride: Riding::default(),
                links: Links::default(),
                item: DroppedItem::new(bob_offset(entity.uuid)),
                pos: Position::spawned(entity.x, entity.y, entity.z),
                vel: Motion::spawned(entity.movement.0, entity.movement.1, entity.movement.2),
                entity: ClientEntity {
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
                    delta_rotation: JavaF32::new(0.0_f32),
                    flight: objects::flight::FlightState {
                        owner,
                        left_owner: false,
                    },
                    on_ground: false,
                    tracking: Tracking::at(entity.x, entity.y, entity.z),
                    always_ticking: entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID,
                    ticking: ticking || entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID,
                    walk: living::WalkAnimation::default(),
                    swim_amount: JavaF32::new(0.0_f32),
                    old_swim_amount: JavaF32::new(0.0_f32),
                    head_stable_time: 0,
                    last_stable_y_head_rot: JavaF32::new(entity.rotation.2),
                    data: entity.data,
                    metadata: None,
                    shared_flags: 0,
                    living_flags: 0,
                    mob_flags: 0,
                    air_supply: DEFAULT_AIR_SUPPLY,
                    health: None,
                    arrow_count: 0,
                    stinger_count: 0,
                    effect_ambience: false,
                    effect_particles: Vec::new(),
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
                    attributes: Vec::new(),
                    item_frame_rotation: 0,
                    lightning_done: false,
                    shot_at_angle: false,
                    attached_target: None,
                },
            },
        );
        self.entities_mut().enter_world(entity.entity_id);
        if let Some(mut shulker) = self
            .entities
            .at_mut(entity.entity_id)
            .filter(|entity| entity.is_shulker())
        {
            // `Shulker.recreateFromPacket`: the body faces south; `snapTo`
            // holds it to the middle of its block.
            shulker.rotation.y_body_rot = JavaF32::new(0.0_f32);
            shulker.rotation.old_y_body_rot = JavaF32::new(0.0_f32);
            let (position, (y_rot, x_rot, _)) = (shulker.position(), shulker.as_view().rotation());
            shulker.snap_to(position, y_rot, x_rot);
        }
        if let Some(mut warden) = self
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

    pub fn at(&self, entity_id: EntityId) -> Option<EntityView<'_>> {
        self.get(&entity_id)
    }

    pub fn metadata(&self, entity_id: EntityId) -> Option<&[u8]> {
        self.get(&entity_id)
            .and_then(|entity| entity.entity.metadata())
    }

    /// A new entity takes its place in its section; an item frame or a painting first moves to the
    /// centre of its box (`HangingEntity.recalculateBoundingBox`).
    pub(crate) fn enter_world(&mut self, entity_id: EntityId) {
        let Some(mut entity) = self.at_mut(entity_id) else {
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

/// See [`ClientWorld::entity_click_view`].
#[derive(Debug, Clone, Copy)]
pub struct EntityClickView {
    pub marker: bool,
    /// Whether an item frame holds an item (`!ItemFrame.getItem().isEmpty()`).
    pub frame_has_item: bool,
}

impl ClientWorld {
    /// The registry name of an entity's type, or `None` when no such entity is there.
    pub fn entity_type_name(&self, entity_id: EntityId) -> Option<&'static str> {
        photon_data::entity_type::name(self.entities.at(entity_id)?.type_id)
    }

    /// What a click on an entity asks of it on the client: an armor stand's marker flag and
    /// whether an item frame is filled.
    pub fn entity_click_view(&self, entity_id: EntityId) -> Option<EntityClickView> {
        let entity = self.entities.at(entity_id)?;
        Some(EntityClickView {
            marker: entity.armor_stand().is_marker()
                && photon_data::entity_type::name(entity.type_id) == Some("minecraft:armor_stand"),
            frame_has_item: entity.is_item_frame() && entity.item_stack().is_some(),
        })
    }

    /// Whether the level holds the entity (the local player included).
    pub fn knows_entity(&self, entity_id: EntityId) -> bool {
        self.entity_exists(entity_id)
    }

    pub(crate) fn entity_exists(&self, entity_id: EntityId) -> bool {
        self.entities.contains_key(&entity_id)
            || self
                .local_player
                .as_ref()
                .is_some_and(|player| player.id == entity_id)
    }
}
