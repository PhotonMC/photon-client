use super::*;

impl PlaySession {
    // The entity packets change the world's entities; one that does
    // not parse is left alone, as the reference ignores what it
    // cannot place.
    pub(super) fn on_add_entity(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some(packet) = parsed(parse_add_entity_packet(payload), "add_entity") {
            if self.removed_player_vehicle == Some(packet.entity_id) {
                self.removed_player_vehicle = None;
            }
            entities::add(&mut self.trace.world, &packet);
        }
        Ok(())
    }

    pub(super) fn on_set_entity_motion(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some(packet) = parsed(parse_set_entity_motion_packet(payload), "set_entity_motion") {
            entities::motion(&mut self.trace.world, &packet);
        }
        Ok(())
    }

    pub(super) fn on_map_item_data(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some(packet) = parsed(parse_map_item_data_packet(payload), "map_item_data") {
            self.trace.world.apply_map_item_data(&packet);
        }
        Ok(())
    }

    pub(super) fn on_move_minecart(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some((entity_id, steps)) =
            parsed(parse_move_minecart_packet(payload), "move_minecart")
        {
            entities::minecart_steps(&mut self.trace.world, entity_id, &steps);
        }
        Ok(())
    }

    pub(super) fn on_set_entity_link(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some((source, holder)) =
            parsed(parse_set_entity_link_packet(payload), "set_entity_link")
        {
            self.trace
                .world
                .entities_mut()
                .set_leash_holder(source, holder);
        }
        Ok(())
    }

    pub(super) fn on_rotate_head(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some(packet) = parsed(parse_rotate_head_packet(payload), "rotate_head") {
            entities::rotate_head(&mut self.trace.world, &packet);
        }
        Ok(())
    }

    pub(super) fn on_remove_entities(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some(packet) = parsed(parse_remove_entities_packet(payload), "remove_entities") {
            // `handleRemoveEntities`: a vehicle that goes with its rider aboard is
            // remembered, for a teleport the server may still address to it.
            for &id in &packet.entity_ids {
                if self.trace.world.has_entity(id)
                    && self.trace.world.entity_carries_local_player(id)
                {
                    self.removed_player_vehicle = Some(id);
                }
            }
            entities::remove(&mut self.trace.world, &packet);
        }
        Ok(())
    }

    pub(super) fn on_set_passengers(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some(packet) = parsed(parse_set_passengers_packet(payload), "set_passengers") {
            let was_aboard = self.trace.world.ridden_vehicle_of_local_player();
            entities::passengers(&mut self.trace.world, &packet);
            if self
                .player_id
                .is_some_and(|id| packet.passenger_ids.contains(&id))
            {
                self.removed_player_vehicle = None;
            }
            // `handleSetEntityPassengersPacket`: boarding tells the rider how to get off.
            if was_aboard != Some(packet.vehicle_id)
                && self.trace.world.ridden_vehicle_of_local_player() == Some(packet.vehicle_id)
            {
                self.hud_events.push(HudEvent::ActionBar(
                    voxel_game::text::Component::translatable_with(
                        "mount.onboard",
                        vec![voxel_game::text::Component::translatable(
                            "key.keyboard.left.shift",
                        )],
                    ),
                ));
            }
            // A player who has just boarded looks the way the boat or the horse faces.
            if was_aboard.is_none()
                && let Some(vehicle) = self.trace.world.ridden_vehicle_of_local_player()
                && let Some(view) = self.trace.world.vehicle_view(vehicle)
                && let Some(player) = self.player.as_mut()
            {
                match view.kind {
                    voxel_world::VehicleKind::Boat => player.face_boat(view.y_rot),
                    voxel_world::VehicleKind::Horse => {
                        player.set_rotation(view.y_rot, view.x_rot);
                    }
                    // Any other vehicle leaves its rider looking where it did.
                    voxel_world::VehicleKind::Pig | voxel_world::VehicleKind::Other => {}
                }
            }
        }
        Ok(())
    }

    pub(super) fn on_entity_position_sync(
        &mut self,
        payload: &[u8],
    ) -> Result<(), ConnectionError> {
        if let Some(packet) = parsed(
            parse_entity_position_sync_packet(payload),
            "entity_position_sync",
        ) {
            entities::position_sync(&mut self.trace.world, &packet);
        }
        Ok(())
    }

    pub(super) fn on_teleport_entity(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some(packet) = parsed(parse_entity_teleport_packet(payload), "entity_teleport") {
            if !self.trace.world.has_entity(packet.entity_id)
                && self.removed_player_vehicle == Some(packet.entity_id)
            {
                // The vehicle is gone: the teleport meant for it moves the rider.
                self.move_former_rider(&packet)?;
            }
            entities::teleport(&mut self.trace.world, &packet);
        }
        Ok(())
    }

    pub(super) fn on_move_entity_pos(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some(packet) = parsed(parse_move_entity_pos_packet(payload), "move_entity_pos") {
            entities::step(&mut self.trace.world, &packet);
        }
        Ok(())
    }

    pub(super) fn on_move_entity_pos_rot(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some(packet) = parsed(
            parse_move_entity_pos_rot_packet(payload),
            "move_entity_pos_rot",
        ) {
            entities::step(&mut self.trace.world, &packet);
        }
        Ok(())
    }

    pub(super) fn on_move_entity_rot(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some(packet) = parsed(parse_move_entity_rot_packet(payload), "move_entity_rot") {
            entities::step(&mut self.trace.world, &packet);
        }
        Ok(())
    }

    pub(super) fn on_animate(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some((id, action)) = parsed(parse_animate_packet(payload), "animate") {
            self.trace.world.entities_mut().apply_animate(id, action);
        }
        Ok(())
    }

    pub(super) fn on_entity_event(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let (id, event) = parse_entity_event_packet(payload)?;
        // `LocalPlayer.handleEntityEvent`: 24 to 28 set the level.
        if self.player_id == Some(id) && (24..=28).contains(&event) {
            self.permission_level = i32::from(event) - 24;
        }
        // `LivingEntity.handleEntityEvent`: 3 is a death.
        if event == 3 {
            if self.player_id == Some(id)
                && let Some(player) = self.player.as_mut()
            {
                player.handle_death_event();
            }
            self.trace.world.entities_mut().apply_death_event(id);
        } else {
            self.trace.world.apply_entity_event(id, event);
        }
        Ok(())
    }

    pub(super) fn on_damage_event(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some((id, damage_type)) = parsed(parse_damage_event_packet(payload), "damage_event")
        {
            self.trace.world.entities_mut().apply_damage_event(id);
            if self.player_id == Some(id) {
                let name = damage_type
                    .and_then(|index| self.registry_names("minecraft:damage_type").get(index))
                    .cloned()
                    .unwrap_or_default();
                if let Some(player) = self.player.as_mut() {
                    player.handle_damage_event(&name);
                }
            }
        }
        Ok(())
    }

    pub(super) fn on_hurt_animation(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some((id, yaw)) = parsed(parse_hurt_animation_packet(payload), "hurt_animation") {
            self.trace.world.entities_mut().apply_hurt_animation(id);
            if self.player_id == Some(id)
                && let Some(player) = self.player.as_mut()
            {
                player.animate_hurt(yaw);
            }
        }
        Ok(())
    }

    pub(super) fn on_set_entity_data(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        // Data of a kind this client cannot read leaves it as it was.
        if let Some(packet) = parsed(parse_entity_metadata_packet(payload), "entity_metadata") {
            entities::metadata(&mut self.trace.world, &packet);
        }
        Ok(())
    }

    pub(super) fn on_set_equipment(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        if let Some(packet) = parsed(parse_entity_equipment_packet(payload), "entity_equipment") {
            entities::equipment(&mut self.trace.world, &packet);
        }
        Ok(())
    }

    pub(super) fn on_update_attributes(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let packet = parse_update_attributes_packet(payload)?;
        let world = &mut self.trace.world;
        if !world.apply_local_player_attributes(packet.entity_id, &packet.attributes) {
            world
                .entities_mut()
                .apply_attributes(packet.entity_id, &packet.attributes);
        }
        Ok(())
    }

    pub(super) fn on_update_mob_effect(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let packet = parse_update_mob_effect_packet(payload)?;
        self.trace.world.apply_local_player_mob_effect(
            packet.entity_id,
            packet.effect_registry_id,
            packet.amplifier,
            packet.duration_ticks,
            packet.flags,
        );
        Ok(())
    }

    pub(super) fn on_remove_mob_effect(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let packet = parse_remove_mob_effect_packet(payload)?;
        self.trace
            .world
            .remove_local_player_mob_effect(packet.entity_id, packet.effect_registry_id);
        Ok(())
    }
}
