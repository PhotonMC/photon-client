use super::*;

impl ClientWorld {
    /// Apply a pinned block-state's BlockTags.CLIMBABLE membership to a living entity.
    pub fn apply_entity_living_climbable_block_state(
        &mut self,
        entity_id: i32,
        state_id: i32,
    ) -> bool {
        let on_climbable = self.block_state_registry.is_climbable_state(state_id);
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.on_climbable = on_climbable;
        true
    }

    /// Apply the LivingEntity.onClimbable tag and fall-flying context.
    pub fn apply_entity_living_climbable_glide_through_state(
        &mut self,
        entity_id: i32,
        state_id: i32,
        fall_flying: bool,
    ) -> bool {
        let is_climbable = self.block_state_registry.is_climbable_state(state_id);
        let is_glide_through = self.block_state_registry.is_glide_through_state(state_id);
        let on_climbable = if fall_flying && is_glide_through {
            false
        } else {
            is_climbable
        };
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        entity.on_climbable = on_climbable;
        if fall_flying {
            entity.shared_flags |= 1 << SHARED_FLAG_FALL_FLYING;
        } else {
            entity.shared_flags &= !(1 << SHARED_FLAG_FALL_FLYING);
        }
        true
    }

    /// Apply the Player ability state used by Java's `Player.onClimbable` gate.
    pub fn apply_entity_player_flying_state(&mut self, entity_id: i32, flying: bool) -> bool {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return false;
        };
        if entity.type_id != VANILLA_PLAYER_ENTITY_TYPE_ID {
            return false;
        }
        entity.flying = flying;
        true
    }

    /// Apply a clientbound vehicle and ordered passenger relationship.
    pub fn apply_entity_passengers(&mut self, vehicle_id: i32, passenger_ids: &[i32]) -> bool {
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
            if let Some(vehicle) = self.entities.get_mut(&vehicle_id) {
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

    pub(in super::super) fn entity_exists(&self, entity_id: i32) -> bool {
        self.entities.contains_key(&entity_id)
            || self
                .local_player
                .as_ref()
                .is_some_and(|player| player.id == entity_id)
    }

    pub fn entity_vehicle_id(&self, entity_id: i32) -> Option<i32> {
        if let Some(entity) = self.entities.get(&entity_id) {
            return entity.vehicle_id;
        }
        self.local_player
            .as_ref()
            .filter(|player| player.id == entity_id)
            .and_then(|player| player.vehicle_id)
    }

    pub fn entity_passengers(&self, entity_id: i32) -> Option<&[i32]> {
        if let Some(entity) = self.entities.get(&entity_id) {
            return Some(&entity.passenger_ids);
        }
        self.local_player
            .as_ref()
            .filter(|player| player.id == entity_id)
            .map(|player| player.passenger_ids.as_slice())
    }

    pub(in super::super) fn set_entity_vehicle_id(
        &mut self,
        entity_id: i32,
        vehicle_id: Option<i32>,
    ) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
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

    pub(in super::super) fn detach_entity_from_vehicle(&mut self, entity_id: i32) {
        let Some(vehicle_id) = self.entity_vehicle_id(entity_id) else {
            return;
        };
        if let Some(vehicle) = self.entities.get_mut(&vehicle_id) {
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

    pub(in super::super) fn eject_entity_passengers(&mut self, vehicle_id: i32) {
        let passengers = if let Some(vehicle) = self.entities.get_mut(&vehicle_id) {
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

    pub(in super::super) fn would_create_passenger_cycle(
        &self,
        passenger_id: i32,
        vehicle_id: i32,
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
