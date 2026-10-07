use crate::*;

impl ClientWorld {
    /// `ClientLevel.tickEntities`: every entity the server added that is
    /// ticking moves on a tick, its passengers after it (the local player,
    /// who has a tick of its own, is not among them unless it rides one).
    pub fn tick_entities(&mut self) {
        let ticking_entity_ids: Vec<i32> = self
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
            self.sky_flash_time = 2;
        }
    }

    pub(crate) fn tick_entity_tree(&mut self, entity_id: i32, vehicle_id: Option<i32>) {
        if vehicle_id.is_some()
            && let Some(entity) = self.entities.get_mut(&entity_id)
        {
            entity.motion.x = JavaF64::new(0.0_f64);
            entity.motion.y = JavaF64::new(0.0_f64);
            entity.motion.z = JavaF64::new(0.0_f64);
        }
        self.tick_entity_once(entity_id);
        if let Some(vehicle_id) = vehicle_id {
            self.position_entity_passenger(vehicle_id, entity_id);
        }
        let passenger_ids = self
            .entity_passengers(entity_id)
            .map(<[i32]>::to_vec)
            .unwrap_or_default();
        for passenger_id in passenger_ids {
            self.tick_entity_passenger_tree(entity_id, passenger_id);
        }
    }

    pub(crate) fn tick_entity_once(&mut self, entity_id: i32) {
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
                objects::flight::flight_of(entity),
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

    /// A living entity that eased into a chunk that is not loaded stops
    /// ticking there (`ClientLevel`'s entity callbacks on a move).
    pub(crate) fn update_ticking_flag(&mut self, entity_id: i32, position: (f64, f64, f64)) {
        let (chunk_x, chunk_z) = entity_chunk_key(position);
        let in_loaded_chunk = self.contains_chunk(chunk_x, chunk_z);
        if let Some(entity) = self.entities.get_mut(&entity_id)
            && entity.living_kind().is_some()
            && !entity.always_ticking
        {
            entity.ticking = in_loaded_chunk;
        }
    }
}
