use crate::*;

impl ClientWorld {
    /// `ClientLevel.tickEntities`: every entity the server added that is
    /// ticking moves on a tick, its passengers after it (the local player,
    /// who has a tick of its own, is not among them unless it rides one).
    pub fn tick_entities(&mut self) {
        let ticking_entity_ids: Vec<EntityId> = self
            .entities
            .iter()
            .filter(|(_, entity)| entity.ticking && entity.vehicle_id.is_none())
            .map(|(&entity_id, _)| entity_id)
            .collect();
        for entity_id in ticking_entity_ids {
            self.tick_entity_tree(entity_id, None);
        }
        // `LightningBolt.tick`: a bolt that is still lit flashes the sky.
        if self
            .entities
            .values()
            .any(|entity| entity.ticking && entity.lightning().is_some_and(|bolt| bolt.is_lit()))
        {
            self.weather.sky_flash_time = 2;
        }
    }

    pub(crate) fn tick_entity_tree(&mut self, entity_id: EntityId, vehicle_id: Option<EntityId>) {
        if vehicle_id.is_some()
            && let Some(entity) = self.entities.at_mut(entity_id)
        {
            entity.motion.x = JavaF64::new(0.0_f64);
            entity.motion.y = JavaF64::new(0.0_f64);
            entity.motion.z = JavaF64::new(0.0_f64);
        }
        self.tick_entity_once(entity_id);
        self.throw_effect_particles(entity_id);
        self.throw_eating_particles(entity_id);
        if let Some(vehicle_id) = vehicle_id {
            self.position_entity_passenger(vehicle_id, entity_id);
        }
        let passenger_ids = self
            .entity_passengers(entity_id)
            .map(<[EntityId]>::to_vec)
            .unwrap_or_default();
        for passenger_id in passenger_ids {
            self.tick_entity_passenger_tree(entity_id, passenger_id);
        }
    }

    /// What decides the branches of an entity's tick, read before the tick: whether a boat is
    /// not the local player's to move, a projectile's kind and whether its path is open, whether
    /// the player rides or steers it, and its flight.
    #[allow(clippy::type_complexity)]
    fn pre_tick_facts(
        &self,
        entity_id: EntityId,
    ) -> Option<(
        bool,
        Option<VanillaProjectileKind>,
        bool,
        (bool, bool),
        Option<objects::flight::Flight>,
    )> {
        self.entities.at(entity_id).map(|entity| {
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
                objects::flight::flight_of(entity),
            )
        })
    }

    pub(crate) fn tick_entity_once(&mut self, entity_id: EntityId) {
        // One read of the entity decides every branch below; the reference
        // evaluates these conditions against the pre-tick state, so they are
        // resolved before the mutable tick borrow is taken.
        let Some((
            boat_is_non_authoritative,
            projectile_kind,
            projectile_path_is_open,
            (ridden, ridden_mount),
            flying,
        )) = self.pre_tick_facts(entity_id)
        else {
            return;
        };

        let Some(previous_position) = self.entities.at(entity_id).map(ClientEntity::position)
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
                .at_mut(entity_id)
                .expect("entity position was checked above");
            entity.capture_old_transform();
            if entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID {
                entity.tick_cloak();
            }
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
                .at_mut(entity_id)
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
                entity.tick_bob();
            }
            if let Some(kind) = living {
                entity.tick_living(kind, tick_death, self.level.game_time);
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
        self.tick_after_species(entity_id, ridden);
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

    /// What follows an entity's own tick: a parrot draws from the level's random source
    /// (`Parrot.aiStep`, of which the particle engine holds the source), and the boat it rides moves.
    fn tick_after_species(&mut self, entity_id: EntityId, ridden: bool) {
        if self
            .entities
            .get(&entity_id)
            .is_some_and(ClientEntity::is_parrot)
        {
            self.level_events
                .0
                .push(LevelEvent::ParrotImitation { id: entity_id });
        }
        if ridden {
            self.tick_ridden_boat(entity_id);
        }
    }

    /// A living entity that eased into a chunk that is not loaded stops
    /// ticking there (`ClientLevel`'s entity callbacks on a move).
    pub(crate) fn update_ticking_flag(&mut self, entity_id: EntityId, position: (f64, f64, f64)) {
        let (chunk_x, chunk_z) = entity_chunk_key(position);
        let in_loaded_chunk = self.contains_chunk(chunk_x, chunk_z);
        if let Some(entity) = self.entities.at_mut(entity_id)
            && entity.living_kind().is_some()
            && !entity.always_ticking
        {
            entity.ticking = in_loaded_chunk;
        }
    }
}

impl ClientWorld {
    /// `LivingEntity.tickEffects` on a client: the swirls of the effects a living entity's data
    /// lists. The reference draws from each entity's own unseeded source; one unseeded source
    /// for them all is the same to see.
    fn throw_effect_particles(&mut self, entity_id: EntityId) {
        let Some(entity) = self
            .entities
            .at(entity_id)
            .filter(|entity| entity.living_kind().is_some())
        else {
            return;
        };
        let flags = (entity.is_effect_ambient(), entity.is_invisible());
        let (x, y, z) = entity.position();
        let body = (
            Vec3::new(x, y, z),
            (entity.dimensions().width(), entity.dimensions().height()),
        );
        let (queue, random) = &mut self.effect_particles;
        queue.extend(particles::effect_particle(
            entity.effect_particles(),
            flags,
            body,
            random.source(),
        ));
    }

    /// `Consumable.onUseTick` on a client for a living entity the server says is eating or
    /// drinking: five crumbs of the item in the used hand on the ticks the local player's own
    /// would. (Its sound is not made yet.)
    fn throw_eating_particles(&mut self, entity_id: EntityId) {
        let Some(entity) = self
            .entities
            .at(entity_id)
            .filter(|entity| entity.living_kind().is_some() && entity.is_using_item())
        else {
            return;
        };
        let hand = if entity.used_item_is_off_hand() {
            EquipmentSlot::OffHand
        } else {
            EquipmentSlot::MainHand
        };
        let Some(stack) = entity.equipment(hand) else {
            return;
        };
        // The tick that began the use counted 1; the reference's first `onUseTick` is the 0th.
        let used = entity.living.using_item_ticks - 1;
        let crumbs = super::super::local_player::consume_cue(Some(stack))
            .is_some_and(|cue| cue.particles && cue.emits(used));
        if !crumbs {
            return;
        }
        let item = stack.item_registry_id();
        let (x, y, z) = entity.position();
        let eye = Vec3::new(x, y + f64::from(entity.dimensions().eye_height()), z);
        let (y_rot, x_rot, _) = entity.rotation();
        let (queue, random) = &mut self.effect_particles;
        for _ in 0..5 {
            let crumb =
                crate::local_player::crumb_particle(random.source(), item, eye, (y_rot, x_rot));
            queue.extend(crumb);
        }
    }

    /// `LivingEntity.handleEntityEvent(46)`: the portal particles of a living entity that a
    /// chorus fruit or its own teleport moved, along the way from last tick's place to this.
    pub fn throw_teleport_particles(&mut self, entity_id: EntityId) {
        let Some(entity) = self
            .entities
            .at(entity_id)
            .filter(|entity| entity.living_kind().is_some())
        else {
            return;
        };
        let (from, to) = (entity.old_position, entity.position());
        let size = (entity.dimensions().width(), entity.dimensions().height());
        let from = Vec3::new(from.x.get(), from.y.get(), from.z.get());
        let (queue, random) = &mut self.effect_particles;
        queue.extend(particles::teleport_particles(
            random.source(),
            (from, Vec3::new(to.0, to.1, to.2)),
            size,
        ));
    }
}
