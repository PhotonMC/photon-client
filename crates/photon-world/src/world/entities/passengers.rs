use crate::*;

impl ClientWorld {
    pub(crate) fn tick_entity_passenger_tree(
        &mut self,
        vehicle_id: EntityId,
        passenger_id: EntityId,
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
        vehicle_id: EntityId,
        passenger_id: EntityId,
    ) -> Option<photon_core::math::Vec3> {
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
            self.entities.at(vehicle_id).and_then(|vehicle| {
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
            .find(|&&(type_id, _, _)| type_id == vehicle_type_id.0)
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
        Some(photon_core::math::Vec3::new(
            target_position.0,
            target_position.1,
            target_position.2,
        ))
    }

    /// `Entity.positionRider` for a vehicle with the default seat: the passenger sits at the top
    /// of the vehicle's box, less its own vehicle attachment.
    fn default_rider_position(
        &self,
        vehicle_id: EntityId,
        passenger_id: EntityId,
    ) -> Option<photon_core::math::Vec3> {
        let vehicle = self.entities.at(vehicle_id)?;
        // Only the vehicles whose seat is Java's default point (the top of the box) are ported:
        // the stand, the displays, and the small entities that hologram setups ride; the other types have seats of
        // their own, which stay where they were.
        static DEFAULT_SEATS: std::sync::LazyLock<Vec<Option<EntityTypeId>>> =
            std::sync::LazyLock::new(|| {
                [
                    "armor_stand",
                    "item_display",
                    "block_display",
                    "text_display",
                    "marker",
                    "area_effect_cloud",
                    "falling_block",
                    "item",
                ]
                .map(|name| photon_data::entity_type::id(&format!("minecraft:{name}")))
                .to_vec()
            });
        if !DEFAULT_SEATS.contains(&Some(vehicle.type_id)) {
            return None;
        }
        let (x, y, z) = vehicle.position();
        let passenger = self.entities.at(passenger_id)?;
        let attachment = vanilla_passenger_vehicle_attachment_y(passenger.type_id, passenger.baby);
        // `ArmorStand.getDimensions`: a marker has none and a small stand is half the size.
        let stand = (DEFAULT_SEATS[0] == Some(vehicle.type_id)).then(|| vehicle.armor_stand());
        let height = match stand {
            Some(stand) if stand.is_marker() => 0.0,
            Some(stand) if stand.is_small() => vehicle.bb_height() * 0.5,
            _ => vehicle.bb_height(),
        };
        Some(photon_core::math::Vec3::new(
            x,
            y + f64::from(height) - attachment,
            z,
        ))
    }

    pub(crate) fn position_entity_passenger(
        &mut self,
        vehicle_id: EntityId,
        passenger_id: EntityId,
    ) {
        let Some(target) = self
            .boat_rider_position(vehicle_id, passenger_id)
            .or_else(|| self.default_rider_position(vehicle_id, passenger_id))
        else {
            return;
        };
        let target_position = (target.x, target.y, target.z);
        let previous_entity_position = self.entities.at(passenger_id).map(ClientEntity::position);
        if let Some(passenger) = self.entities.at_mut(passenger_id) {
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

    pub(crate) fn clamp_entity_passenger_rotation(
        &mut self,
        vehicle_id: EntityId,
        passenger_id: EntityId,
    ) {
        let Some((vehicle_type_id, vehicle_y_rot, delta_rotation, passenger_count)) =
            self.entities.at(vehicle_id).map(|vehicle| {
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
                if passenger_id.0 % 2 == 0 {
                    90.0_f32
                } else {
                    270.0_f32
                }
            });
        if let Some(passenger) = self.entities.at_mut(passenger_id) {
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

    /// What the species that look about them read of the world: whether this
    /// is a cat that lies down with a sleeping player near, and whether it
    /// thunders.
    /// `Entity.updateFluidInteraction`'s answer for the mobs that read it: whether
    /// the water reaches their box (none for the others, whose state is not
    /// tracked). A squid is not pushed by fluids, so no current is gathered.
    /// `AbstractBoat.isUnderwater`'s answer (the status is under (flowing) water):
    /// whether the water reaches a thousandth above the top of the boat's box.
    /// Whether a boat (and only a boat) is under water, as its tick reads it.
    pub(crate) fn boat_under_water_of(&self, entity_id: EntityId) -> Option<bool> {
        let entity = self
            .entities
            .get(&entity_id)
            .filter(|entity| entity.is_boat())?;
        Some(self.boat_is_under_water(entity))
    }

    pub(crate) fn boat_is_under_water(&self, entity: &ClientEntity) -> bool {
        let Some(pick) = photon_data::entity_type::pick(entity.type_id) else {
            return false;
        };
        let (width, height, _) = pick.size;
        let (x, y, z) = entity.position();
        let area = ClientEntityDimensions::scalable(width, height)
            .scale(entity.scale(), entity.scale())
            .make_bounding_box(photon_core::math::Vec3::new(x, y, z));
        let top = area.max_y + 0.001;
        let level = local_player::Level::new(self);
        for bx in photon_core::math::floor_f64(area.min_x)..photon_core::math::ceil_f64(area.max_x)
        {
            for by in photon_core::math::floor_f64(area.max_y)..photon_core::math::ceil_f64(top) {
                for bz in photon_core::math::floor_f64(area.min_z)
                    ..photon_core::math::ceil_f64(area.max_z)
                {
                    let pos = photon_core::binary::BlockPos::new(bx, by, bz);
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

    pub(crate) fn boat_is_local_instance_authoritative(&self, entity: &ClientEntity) -> bool {
        self.local_player
            .as_ref()
            .is_some_and(|player| entity.passenger_ids.first() == Some(&player.id))
    }

    pub(crate) fn root_vehicle_id(&self, entity_id: EntityId) -> EntityId {
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

    /// Apply a clientbound vehicle and ordered passenger relationship.
    pub fn apply_entity_passengers(
        &mut self,
        vehicle_id: EntityId,
        passenger_ids: &[EntityId],
    ) -> bool {
        if !self.entity_exists(vehicle_id) {
            return false;
        }

        self.eject_entity_passengers(vehicle_id);
        for &passenger_id in passenger_ids {
            if !self.entity_exists(passenger_id)
                || self.would_create_passenger_cycle(passenger_id, vehicle_id)
                || self.entity_vehicle_id(passenger_id) == Some(vehicle_id)
            {
                continue;
            }
            self.detach_entity_from_vehicle(passenger_id);
            self.set_entity_vehicle_id(passenger_id, Some(vehicle_id));
            if let Some(vehicle) = self.entities.at_mut(vehicle_id) {
                vehicle.passenger_ids.push(passenger_id);
            } else if let Some(player) = self
                .local_player
                .as_mut()
                .filter(|player| player.id == vehicle_id)
            {
                player.passenger_ids.push(passenger_id);
            }
        }
        true
    }

    pub fn entity_vehicle_id(&self, entity_id: EntityId) -> Option<EntityId> {
        if let Some(entity) = self.entities.at(entity_id) {
            return entity.vehicle_id;
        }
        self.local_player
            .as_ref()
            .filter(|player| player.id == entity_id)
            .and_then(|player| player.vehicle_id)
    }

    pub fn entity_passengers(&self, entity_id: EntityId) -> Option<&[EntityId]> {
        if let Some(entity) = self.entities.at(entity_id) {
            return Some(&entity.passenger_ids);
        }
        self.local_player
            .as_ref()
            .filter(|player| player.id == entity_id)
            .map(|player| player.passenger_ids.as_slice())
    }

    pub(crate) fn detach_entity_from_vehicle(&mut self, entity_id: EntityId) {
        let Some(vehicle_id) = self.entity_vehicle_id(entity_id) else {
            return;
        };
        if let Some(vehicle) = self.entities.at_mut(vehicle_id) {
            vehicle
                .passenger_ids
                .retain(|&passenger_id| passenger_id != entity_id);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == vehicle_id)
        {
            player
                .passenger_ids
                .retain(|&passenger_id| passenger_id != entity_id);
        }
        self.set_entity_vehicle_id(entity_id, None);
    }

    pub(crate) fn eject_entity_passengers(&mut self, vehicle_id: EntityId) {
        let passengers = if let Some(vehicle) = self.entities.at_mut(vehicle_id) {
            Some(std::mem::take(&mut vehicle.passenger_ids))
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == vehicle_id)
        {
            Some(std::mem::take(&mut player.passenger_ids))
        } else {
            None
        };
        let Some(passengers) = passengers else {
            return;
        };
        for passenger_id in passengers {
            if self.entity_vehicle_id(passenger_id) == Some(vehicle_id) {
                self.set_entity_vehicle_id(passenger_id, None);
            }
        }
    }

    pub(crate) fn would_create_passenger_cycle(
        &self,
        passenger_id: EntityId,
        vehicle_id: EntityId,
    ) -> bool {
        if passenger_id == vehicle_id {
            return true;
        }
        let mut visited = BTreeSet::new();
        let mut current = Some(vehicle_id);
        while let Some(entity_id) = current {
            if !visited.insert(entity_id) {
                return true;
            }
            if entity_id == passenger_id {
                return true;
            }
            current = self.entity_vehicle_id(entity_id);
        }
        false
    }
}

impl ClientWorld {
    pub(crate) fn clamp_entity_passenger_rotation_after_turn(
        &mut self,
        vehicle_id: EntityId,
        passenger_id: EntityId,
    ) {
        let Some(vehicle_y_rot) = self.entities.get(&vehicle_id).and_then(|vehicle| {
            VANILLA_BOAT_PASSENGER_GEOMETRY
                .iter()
                .any(|&(type_id, _, _)| type_id == vehicle.type_id.0)
                .then(|| vehicle.y_rot.get())
        }) else {
            return;
        };
        if let Some(passenger) = self.entities.at_mut(passenger_id) {
            passenger.clamp_vanilla_boat_passenger_rotation(vehicle_y_rot, None);
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == passenger_id)
        {
            player.clamp_vanilla_boat_passenger_rotation(vehicle_y_rot);
        }
    }

    pub(crate) fn mob_in_water(&self, entity_id: EntityId) -> Option<bool> {
        let entity = self.entities.get(&entity_id)?;
        if !entity.tracks_water() {
            return None;
        }
        let pick = photon_data::entity_type::pick(entity.type_id)?;
        let (width, height, eye) = match pick.baby {
            Some(baby) if entity.is_baby() => baby,
            _ => pick.size,
        };
        let dimensions = ClientEntityDimensions::scalable(width, height)
            .with_eye_height(eye)
            .scale(entity.fish_dimension_scale(), entity.fish_dimension_scale())
            .scale(entity.scale(), entity.scale());
        let (x, y, z) = entity.position();
        let position = photon_core::math::Vec3::new(x, y, z);
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

    pub(crate) fn set_entity_vehicle_id(
        &mut self,
        entity_id: EntityId,
        vehicle_id: Option<EntityId>,
    ) {
        if let Some(entity) = self.entities.at_mut(entity_id) {
            if entity.is_shulker() && entity.vehicle_id != vehicle_id {
                if vehicle_id.is_some() {
                    entity.shulker_starts_riding();
                } else {
                    entity.shulker_stops_riding();
                }
            }
            entity.vehicle_id = vehicle_id;
        } else if let Some(player) = self
            .local_player
            .as_mut()
            .filter(|player| player.id == entity_id)
        {
            player.vehicle_id = vehicle_id;
        }
    }
}
