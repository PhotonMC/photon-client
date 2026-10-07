use super::*;

impl ClientWorld {
    /// Copy local player movement flags into a client-authoritative boat.
    pub fn set_boat_input(&mut self, vehicle_id: i32, input: BoatInput) -> bool {
        let Some(boat) = self.entities.get_mut(&vehicle_id) else {
            return false;
        };
        if vanilla_boat_max_passengers(boat.type_id).is_none() {
            return false;
        }
        boat.boat_input = input;
        true
    }

    pub(in super::super) fn tick_local_player(&mut self, vehicle_id: Option<i32>) {
        let passenger_ids = {
            let Some(player) = self.local_player.as_mut() else {
                return;
            };
            if vehicle_id.is_some() {
                player.motion.x = JavaF64::new(0.0_f64);
                player.motion.y = JavaF64::new(0.0_f64);
                player.motion.z = JavaF64::new(0.0_f64);
            }
            player.capture_old_transform();
            player.tick_head_rotation();
            player.tick_count = player.tick_count.wrapping_add(1);
            for effect in &mut player.effects {
                effect.tick_client();
            }
            player.passenger_ids.clone()
        };
        let Some(player_id) = self.local_player.as_ref().map(|player| player.id) else {
            return;
        };
        if let Some(vehicle_id) = vehicle_id {
            self.position_entity_passenger(vehicle_id, player_id);
        }
        for passenger_id in passenger_ids {
            self.tick_entity_passenger_tree(player_id, passenger_id);
        }
    }

    pub(in super::super) fn tick_entity_tree(&mut self, entity_id: i32, vehicle_id: Option<i32>) {
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

    pub(in super::super) fn tick_entity_passenger_tree(
        &mut self,
        vehicle_id: i32,
        passenger_id: i32,
    ) {
        if self.entity_vehicle_id(passenger_id) != Some(vehicle_id) {
            return;
        }
        if self
            .local_player
            .as_ref()
            .is_some_and(|player| player.id == passenger_id)
        {
            self.tick_local_player(Some(vehicle_id));
        } else if self
            .entities
            .get(&passenger_id)
            .is_some_and(|entity| entity.ticking || entity.type_id == VANILLA_PLAYER_ENTITY_TYPE_ID)
        {
            self.tick_entity_tree(passenger_id, Some(vehicle_id));
        }
    }

    /// `getPassengerRidingPosition` of a boat for a passenger: where the boat carries it, from
    /// where the boat stands and faces and how many ride with it.
    pub(crate) fn boat_rider_position(
        &self,
        vehicle_id: i32,
        passenger_id: i32,
    ) -> Option<voxel_core::math::Vec3> {
        let passenger_type_id = self
            .entities
            .get(&passenger_id)
            .map(|entity| entity.type_id)
            .or_else(|| {
                self.local_player
                    .as_ref()
                    .filter(|player| player.id == passenger_id)
                    .map(|_| VANILLA_PLAYER_ENTITY_TYPE_ID)
            })?;
        let (vehicle_type_id, vehicle_position, vehicle_y_rot, passenger_index, passenger_count) =
            self.entities.get(&vehicle_id).and_then(|vehicle| {
                let passenger_index = vehicle
                    .passenger_ids
                    .iter()
                    .position(|&entity_id| entity_id == passenger_id)?;
                Some((
                    vehicle.type_id,
                    vehicle.position(),
                    vehicle.y_rot.get(),
                    passenger_index,
                    vehicle.passenger_ids.len(),
                ))
            })?;
        let (ride_height, single_passenger_z_offset) = VANILLA_BOAT_PASSENGER_GEOMETRY
            .iter()
            .find(|&&(type_id, _, _)| type_id == vehicle_type_id)
            .map(|&(_, ride_height, single_passenger_z_offset)| {
                (f64::from(ride_height), f64::from(single_passenger_z_offset))
            })?;
        let passenger_is_baby = self
            .entities
            .get(&passenger_id)
            .is_some_and(|passenger| passenger.baby);
        let passenger_vehicle_attachment_y =
            vanilla_passenger_vehicle_attachment_y(passenger_type_id, passenger_is_baby);

        let passenger_z_offset = if passenger_count > 1 {
            let base_offset = if passenger_index == 0 {
                0.2_f32
            } else {
                -0.6_f32
            };
            let animal_offset = if vanilla_passenger_is_animal(passenger_type_id) {
                0.2_f32
            } else {
                0.0_f32
            };
            f64::from(base_offset + animal_offset)
        } else {
            single_passenger_z_offset
        };
        let radians = f64::from(-vehicle_y_rot * (std::f32::consts::PI / 180.0_f32));
        let rotated_x = passenger_z_offset * f64::from(mth_sin_f64(radians));
        let rotated_z = passenger_z_offset * f64::from(mth_cos_f64(radians));
        let target_position = (
            vehicle_position.0 + rotated_x,
            vehicle_position.1 + ride_height - passenger_vehicle_attachment_y,
            vehicle_position.2 + rotated_z,
        );
        Some(voxel_core::math::Vec3::new(
            target_position.0,
            target_position.1,
            target_position.2,
        ))
    }

    pub(in super::super) fn position_entity_passenger(
        &mut self,
        vehicle_id: i32,
        passenger_id: i32,
    ) {
        let Some(target) = self.boat_rider_position(vehicle_id, passenger_id) else {
            return;
        };
        let target_position = (target.x, target.y, target.z);
        let previous_entity_position = self.entities.get(&passenger_id).map(ClientEntity::position);
        if let Some(passenger) = self.entities.get_mut(&passenger_id) {
            passenger.set_position(target_position);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == passenger_id)
        {
            player.set_position(target_position);
        }
        if let Some(previous_position) = previous_entity_position {
            self.update_entity_section(passenger_id, previous_position, target_position);
        }
        self.clamp_entity_passenger_rotation(vehicle_id, passenger_id);
    }

    pub(in super::super) fn clamp_entity_passenger_rotation(
        &mut self,
        vehicle_id: i32,
        passenger_id: i32,
    ) {
        let Some((vehicle_type_id, vehicle_y_rot, delta_rotation, passenger_count)) =
            self.entities.get(&vehicle_id).map(|vehicle| {
                (
                    vehicle.type_id,
                    vehicle.y_rot.get(),
                    vehicle.delta_rotation.get(),
                    vehicle.passenger_ids.len(),
                )
            })
        else {
            return;
        };
        let Some(passenger_type_id) = self
            .entities
            .get(&passenger_id)
            .map(|passenger| passenger.type_id)
            .or_else(|| {
                self.local_player
                    .as_ref()
                    .filter(|player| player.id == passenger_id)
                    .map(|_| VANILLA_PLAYER_ENTITY_TYPE_ID)
            })
        else {
            return;
        };
        if vanilla_passenger_can_turn_in_boat(passenger_type_id) {
            return;
        }
        let Some(max_passenger_count) = vanilla_boat_max_passengers(vehicle_type_id) else {
            return;
        };
        let animal_rotation_offset = (vanilla_passenger_is_animal(passenger_type_id)
            && passenger_count == max_passenger_count)
            .then_some({
                if passenger_id % 2 == 0 {
                    90.0_f32
                } else {
                    270.0_f32
                }
            });
        if let Some(passenger) = self.entities.get_mut(&passenger_id) {
            passenger.add_vanilla_boat_passenger_rotation(delta_rotation);
            passenger.clamp_vanilla_boat_passenger_rotation(vehicle_y_rot, animal_rotation_offset);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == passenger_id)
        {
            player.add_vanilla_boat_passenger_rotation(delta_rotation);
            player.clamp_vanilla_boat_passenger_rotation(vehicle_y_rot);
        }
    }

    pub(in super::super) fn clamp_entity_passenger_rotation_after_turn(
        &mut self,
        vehicle_id: i32,
        passenger_id: i32,
    ) {
        let Some(vehicle_y_rot) = self.entities.get(&vehicle_id).and_then(|vehicle| {
            VANILLA_BOAT_PASSENGER_GEOMETRY
                .iter()
                .any(|&(type_id, _, _)| type_id == vehicle.type_id)
                .then(|| vehicle.y_rot.get())
        }) else {
            return;
        };
        if let Some(passenger) = self.entities.get_mut(&passenger_id) {
            passenger.clamp_vanilla_boat_passenger_rotation(vehicle_y_rot, None);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == passenger_id)
        {
            player.clamp_vanilla_boat_passenger_rotation(vehicle_y_rot);
        }
    }

    /// `Cat.handleLieDown`'s search: a player who sleeps within two blocks of
    /// the cat's block (the local player, or one the server added).
    pub(in super::super) fn sleeping_player_near(&self, cat: &ClientEntity) -> bool {
        let (x, y, z) = cat.position();
        let (x, y, z) = (x.floor(), y.floor(), z.floor());
        let area =
            voxel_core::aabb::Aabb::new(x, y, z, x + 1.0, y + 1.0, z + 1.0).inflate(2.0, 2.0, 2.0);
        let local = self.local_player.as_ref().is_some_and(|player| {
            let (x, y, z) = player.position();
            player.is_sleeping()
                && player
                    .pose()
                    .player_dimensions()
                    .make_bounding_box(Vec3::new(x, y, z))
                    .intersects(area)
        });
        local
            || self.entities.values().any(|other| {
                other.is_player() && other.is_sleeping() && other.bounding_box().intersects(area)
            })
    }

    /// What the species that look about them read of the world: whether this
    /// is a cat that lies down with a sleeping player near, and whether it
    /// thunders.
    /// `Entity.updateFluidInteraction`'s answer for the mobs that read it: whether
    /// the water reaches their box (none for the others, whose state is not
    /// tracked). A squid is not pushed by fluids, so no current is gathered.
    /// `AbstractBoat.isUnderwater`'s answer (the status is under (flowing) water):
    /// whether the water reaches a thousandth above the top of the boat's box.
    /// Whether a boat (and only a boat) is under water, as its tick reads it.
    pub(in super::super) fn boat_under_water_of(&self, entity_id: i32) -> Option<bool> {
        let entity = self
            .entities
            .get(&entity_id)
            .filter(|entity| entity.is_boat())?;
        Some(self.boat_is_under_water(entity))
    }

    pub(in super::super) fn boat_is_under_water(&self, entity: &ClientEntity) -> bool {
        let Some(pick) = voxel_data::entity_type::pick(entity.type_id) else {
            return false;
        };
        let (width, height, _) = pick.size;
        let (x, y, z) = entity.position();
        let area = ClientEntityDimensions::scalable(width, height)
            .scale(entity.scale(), entity.scale())
            .make_bounding_box(voxel_core::math::Vec3::new(x, y, z));
        let top = area.max_y + 0.001;
        let level = local_player::Level::new(self);
        for bx in voxel_core::math::floor_f64(area.min_x)..voxel_core::math::ceil_f64(area.max_x) {
            for by in voxel_core::math::floor_f64(area.max_y)..voxel_core::math::ceil_f64(top) {
                for bz in
                    voxel_core::math::floor_f64(area.min_z)..voxel_core::math::ceil_f64(area.max_z)
                {
                    let pos = voxel_core::binary::BlockPos::new(bx, by, bz);
                    let info = level.info(pos);
                    if info.is_water()
                        && top < f64::from(by) + f64::from(level.fluid_height(pos, &info))
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub(in super::super) fn mob_in_water(&self, entity_id: i32) -> Option<bool> {
        let entity = self.entities.get(&entity_id)?;
        if !entity.tracks_water() {
            return None;
        }
        let pick = voxel_data::entity_type::pick(entity.type_id)?;
        let (width, height, eye) = match pick.baby {
            Some(baby) if entity.is_baby() => baby,
            _ => pick.size,
        };
        let dimensions = ClientEntityDimensions::scalable(width, height)
            .with_eye_height(eye)
            .scale(entity.fish_dimension_scale(), entity.fish_dimension_scale())
            .scale(entity.scale(), entity.scale());
        let (x, y, z) = entity.position();
        let position = voxel_core::math::Vec3::new(x, y, z);
        let found = fluid::scan(
            &local_player::Level::new(self),
            (
                dimensions.make_bounding_box(position),
                position,
                dimensions.eye_height(),
            ),
            true,
        )
        .ok()?;
        Some(found.water.height > 0.0)
    }

    /// `Guardian.getActiveAttackTarget` on the client: the living entity the
    /// beam's id names, the local player among them.
    pub(in super::super) fn guardian_target_of(
        &self,
        entity_id: i32,
    ) -> Option<guardian::GuardianTarget> {
        let id = self.entities.get(&entity_id)?.guardian_attack_target()?;
        self.living_target(id)
    }

    /// What a tick of the entity looks up in the world.
    pub(in super::super) fn targets_of(&self, entity_id: i32) -> entity::EntityTargets {
        entity::EntityTargets {
            guardian: self.guardian_target_of(entity_id),
            wither: self.wither_targets_of(entity_id),
            dragon: self.dragon_crystals_of(entity_id),
        }
    }

    /// `EnderDragon.checkCrystals`' look at the world: the nearest end crystal whose box
    /// meets the dragon's box made 32 blocks wider (by the distance between the two
    /// entities' places), and whether the crystal the dragon holds is still there.
    pub(in super::super) fn dragon_crystals_of(
        &self,
        entity_id: i32,
    ) -> ender_dragon::DragonCrystals {
        let Some(dragon) = self
            .entities
            .get(&entity_id)
            .filter(|e| e.is_ender_dragon())
        else {
            return ender_dragon::DragonCrystals::default();
        };
        let current = dragon.ender_dragon().and_then(|own| own.nearest_crystal());
        let current_exists = current.is_some_and(|id| {
            self.entities
                .get(&id)
                .is_some_and(ClientEntity::is_end_crystal)
        });
        let (x, y, z) = dragon.position();
        let size = dragon.dimensions();
        let half = f64::from(size.width()) / 2.0 + 32.0;
        let (low, high) = (
            [x - half, y - 32.0, z - half],
            [x + half, y + f64::from(size.height()) + 32.0, z + half],
        );
        let mut nearest = None;
        let mut distance = f64::MAX;
        for crystal in self.entities.values().filter(|e| e.is_end_crystal()) {
            let (cx, cy, cz) = crystal.position();
            let crystal_size = crystal.dimensions();
            let reach = f64::from(crystal_size.width()) / 2.0;
            let height = f64::from(crystal_size.height());
            let meets = low[0] < cx + reach
                && high[0] > cx - reach
                && low[1] < cy + height
                && high[1] > cy
                && low[2] < cz + reach
                && high[2] > cz - reach;
            if !meets {
                continue;
            }
            let squared = (cx - x).powi(2) + (cy - y).powi(2) + (cz - z).powi(2);
            if squared < distance {
                distance = squared;
                nearest = Some(crystal.id());
            }
        }
        ender_dragon::DragonCrystals {
            nearest,
            current_exists,
        }
    }

    /// The two entities a wither's side heads look at, by the ids it holds.
    pub(in super::super) fn wither_targets_of(
        &self,
        entity_id: i32,
    ) -> [Option<guardian::GuardianTarget>; 2] {
        let ids = self
            .entities
            .get(&entity_id)
            .map_or([0; 2], ClientEntity::wither_targets);
        ids.map(|id| (id > 0).then(|| self.living_target(id)).flatten())
    }
}
