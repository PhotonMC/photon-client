use super::*;
use photon_core::ids::EntityId;
use photon_world::objects::minecart_track::MinecartStep;
use photon_world::particles::pickup_target;

/// The packets of this family, by packet id.
pub(super) const DECODERS: &[(i32, Decoder)] = &[
    (PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID, typed::<AddEntity>),
    (
        PLAY_CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID,
        typed::<SetEntityMotion>,
    ),
    (
        PLAY_CLIENTBOUND_MAP_ITEM_DATA_PACKET_ID,
        typed::<MapItemData>,
    ),
    (
        PLAY_CLIENTBOUND_MOVE_MINECART_PACKET_ID,
        typed::<MoveMinecart>,
    ),
    (
        PLAY_CLIENTBOUND_SET_ENTITY_LINK_PACKET_ID,
        typed::<SetEntityLink>,
    ),
    (PLAY_CLIENTBOUND_ROTATE_HEAD_PACKET_ID, typed::<RotateHead>),
    (
        PLAY_CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
        typed::<RemoveEntities>,
    ),
    (
        PLAY_CLIENTBOUND_TAKE_ITEM_ENTITY_PACKET_ID,
        typed::<TakeItemEntity>,
    ),
    (
        PLAY_CLIENTBOUND_SET_PASSENGERS_PACKET_ID,
        typed::<SetPassengers>,
    ),
    (
        PLAY_CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID,
        typed::<EntityPositionSync>,
    ),
    (
        PLAY_CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID,
        typed::<TeleportEntity>,
    ),
    (
        PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID,
        typed::<MoveEntityPos>,
    ),
    (
        PLAY_CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID,
        typed::<MoveEntityPosRot>,
    ),
    (
        PLAY_CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID,
        typed::<MoveEntityRot>,
    ),
    (PLAY_CLIENTBOUND_ANIMATE_PACKET_ID, typed::<Animate>),
    (
        PLAY_CLIENTBOUND_ENTITY_EVENT_PACKET_ID,
        typed::<EntityEvent>,
    ),
    (
        PLAY_CLIENTBOUND_DAMAGE_EVENT_PACKET_ID,
        typed::<DamageEvent>,
    ),
    (
        PLAY_CLIENTBOUND_HURT_ANIMATION_PACKET_ID,
        typed::<HurtAnimation>,
    ),
    (
        PLAY_CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID,
        typed::<SetEntityData>,
    ),
    (
        PLAY_CLIENTBOUND_SET_EQUIPMENT_PACKET_ID,
        typed::<SetEquipment>,
    ),
    (
        PLAY_CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID,
        typed::<UpdateAttributes>,
    ),
    (
        PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID,
        typed::<UpdateMobEffect>,
    ),
    (
        PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID,
        typed::<RemoveMobEffect>,
    ),
];

impl ClientboundPacket for AddEntity {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_add_entity(self.0)
    }
}

impl ClientboundPacket for SetEntityMotion {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_set_entity_motion(self.0)
    }
}

impl ClientboundPacket for MapItemData {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_map_item_data(self.0)
    }
}

impl ClientboundPacket for MoveMinecart {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_move_minecart(self.0)
    }
}

impl ClientboundPacket for SetEntityLink {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_set_entity_link(self.0)
    }
}

impl ClientboundPacket for RotateHead {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_rotate_head(self.0)
    }
}

impl ClientboundPacket for RemoveEntities {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_remove_entities(self.0)
    }
}

impl ClientboundPacket for TakeItemEntity {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_take_item_entity(self.0)
    }
}

impl ClientboundPacket for SetPassengers {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_set_passengers(self.0)
    }
}

impl ClientboundPacket for EntityPositionSync {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_entity_position_sync(self.0)
    }
}

impl ClientboundPacket for TeleportEntity {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_teleport_entity(self.0)
    }
}

impl ClientboundPacket for MoveEntityPos {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_move_entity_pos(self.0)
    }
}

impl ClientboundPacket for MoveEntityPosRot {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_move_entity_pos_rot(self.0)
    }
}

impl ClientboundPacket for MoveEntityRot {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_move_entity_rot(self.0)
    }
}

impl ClientboundPacket for Animate {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_animate(self.0)
    }
}

impl ClientboundPacket for EntityEvent {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_entity_event(self.0)
    }
}

impl ClientboundPacket for DamageEvent {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_damage_event(self.0)
    }
}

impl ClientboundPacket for HurtAnimation {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_hurt_animation(self.0)
    }
}

impl ClientboundPacket for SetEntityData {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_set_entity_data(self.0)
    }
}

impl ClientboundPacket for SetEquipment {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_set_equipment(self.0)
    }
}

impl ClientboundPacket for UpdateAttributes {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_update_attributes(self.0)
    }
}

impl ClientboundPacket for UpdateMobEffect {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_update_mob_effect(self.0)
    }
}

impl ClientboundPacket for RemoveMobEffect {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_remove_mob_effect(self.0)
    }
}

impl PlaySession {
    fn on_add_entity(&mut self, packet: AddEntityObservation) -> Result<(), ConnectionError> {
        if self.local.removed_player_vehicle == Some(packet.entity_id) {
            self.local.removed_player_vehicle = None;
        }
        entities::add(&mut self.world, &packet);
        // `handleAddEntity`: a player in the tab list is remembered as seen.
        let id = (packet.uuid.most, packet.uuid.least);
        if photon_data::entity_type::id("minecraft:player") == Some(packet.entity_type_id)
            && let Some(info) = self.social.player_infos.get(&id)
        {
            self.social.seen_players.insert(id, info.name.clone());
        }
        Ok(())
    }

    fn on_set_entity_motion(
        &mut self,
        packet: EntityMotionObservation,
    ) -> Result<(), ConnectionError> {
        entities::motion(&mut self.world, &packet);
        Ok(())
    }

    fn on_map_item_data(
        &mut self,
        packet: photon_world::map::MapItemDataInput,
    ) -> Result<(), ConnectionError> {
        self.world.apply_map_item_data(&packet);
        Ok(())
    }

    fn on_move_minecart(
        &mut self,
        (entity_id, steps): (EntityId, Vec<MinecartStep>),
    ) -> Result<(), ConnectionError> {
        entities::minecart_steps(&mut self.world, entity_id, &steps);
        Ok(())
    }

    fn on_set_entity_link(
        &mut self,
        (source, holder): (EntityId, EntityId),
    ) -> Result<(), ConnectionError> {
        self.world.entities_mut().set_leash_holder(source, holder);
        Ok(())
    }

    fn on_rotate_head(
        &mut self,
        packet: EntityHeadRotationObservation,
    ) -> Result<(), ConnectionError> {
        entities::rotate_head(&mut self.world, &packet);
        Ok(())
    }

    fn on_remove_entities(
        &mut self,
        packet: RemoveEntitiesObservation,
    ) -> Result<(), ConnectionError> {
        // `handleRemoveEntities`: a vehicle that goes with its rider aboard is
        // remembered, for a teleport the server may still address to it.
        for &id in &packet.entity_ids {
            if self.world.has_entity(id) && self.world.entity_carries_local_player(id) {
                self.local.removed_player_vehicle = Some(id);
            }
        }
        entities::remove(&mut self.world, &packet);
        Ok(())
    }

    fn on_take_item_entity(
        &mut self,
        (item_id, player_id, amount): (EntityId, EntityId, i32),
    ) -> Result<(), ConnectionError> {
        let world = &self.world;
        let taker = world
            .entities()
            .at(player_id)
            .filter(|entity| Some(player_id) != self.local.player_id && entity.is_living_entity())
            .map(|entity| (player_id, entity.position(), entity.eye_height()));
        let target_at = match taker {
            Some((_, position, eye_height)) => Some(pickup_target(position, eye_height)),
            None => self.local.player.as_ref().map(|player| {
                let at = player.position();
                pickup_target((at.x, at.y, at.z), player.eye_height())
            }),
        };
        if let Some(target_at) = target_at
            && let Some(taken) =
                self.world
                    .take_item_entity(item_id, amount, &mut self.effects.listener_random)
        {
            self.effects
                .item_pickups
                .push(photon_world::particles::ItemPickup::new(
                    taken,
                    taker.map(|(id, ..)| id),
                    target_at,
                ));
        }
        Ok(())
    }

    fn on_set_passengers(
        &mut self,
        packet: EntityPassengersObservation,
    ) -> Result<(), ConnectionError> {
        let was_aboard = self.world.ridden_vehicle_of_local_player();
        entities::passengers(&mut self.world, &packet);
        if self
            .local
            .player_id
            .is_some_and(|id| packet.passenger_ids.contains(&id))
        {
            self.local.removed_player_vehicle = None;
        }
        // `handleSetEntityPassengersPacket`: boarding tells the rider how to get off.
        if was_aboard != Some(packet.vehicle_id)
            && self.world.ridden_vehicle_of_local_player() == Some(packet.vehicle_id)
        {
            self.social.hud_events.push(HudEvent::ActionBar(
                photon_game::text::Component::translatable_with(
                    "mount.onboard",
                    vec![photon_game::text::Component::translatable(
                        "key.keyboard.left.shift",
                    )],
                ),
            ));
        }
        // A player who has just boarded looks the way the boat or the horse faces.
        if was_aboard.is_none()
            && let Some(vehicle) = self.world.ridden_vehicle_of_local_player()
            && let Some(view) = self.world.vehicle_view(vehicle)
            && let Some(player) = self.local.player.as_mut()
        {
            match view.kind {
                photon_world::VehicleKind::Boat => player.face_boat(view.y_rot),
                photon_world::VehicleKind::Horse => {
                    player.set_rotation(view.y_rot, view.x_rot);
                }
                // Any other vehicle leaves its rider looking where it did.
                photon_world::VehicleKind::Pig | photon_world::VehicleKind::Other => {}
            }
        }
        Ok(())
    }

    fn on_entity_position_sync(
        &mut self,
        packet: EntityPositionSyncObservation,
    ) -> Result<(), ConnectionError> {
        entities::position_sync(&mut self.world, &packet);
        Ok(())
    }

    fn on_teleport_entity(
        &mut self,
        packet: EntityTeleportObservation,
    ) -> Result<(), ConnectionError> {
        if !self.world.has_entity(packet.entity_id)
            && self.local.removed_player_vehicle == Some(packet.entity_id)
        {
            // The vehicle is gone: the teleport meant for it moves the rider.
            self.move_former_rider(&packet)?;
        }
        entities::teleport(&mut self.world, &packet);
        Ok(())
    }

    fn on_move_entity_pos(&mut self, packet: EntityMoveObservation) -> Result<(), ConnectionError> {
        entities::step(&mut self.world, &packet);
        Ok(())
    }

    fn on_move_entity_pos_rot(
        &mut self,
        packet: EntityMoveObservation,
    ) -> Result<(), ConnectionError> {
        entities::step(&mut self.world, &packet);
        Ok(())
    }

    fn on_move_entity_rot(&mut self, packet: EntityMoveObservation) -> Result<(), ConnectionError> {
        entities::step(&mut self.world, &packet);
        Ok(())
    }

    fn on_animate(&mut self, (id, action): (EntityId, u8)) -> Result<(), ConnectionError> {
        // `ClientPacketListener.handleAnimate`: 4 and 5 are the hits that crit.
        let hit = match action {
            4 => Some("crit"),
            5 => Some("enchanted_hit"),
            _ => None,
        };
        if let Some(particle) = hit {
            self.queue_emitter(id, particle, 3, None);
        }
        self.world.entities_mut().apply_animate(id, action);
        Ok(())
    }

    fn on_entity_event(&mut self, (id, event): (EntityId, i8)) -> Result<(), ConnectionError> {
        // `LocalPlayer.handleEntityEvent`: 24 to 28 set the level.
        if self.local.player_id == Some(id) && (24..=28).contains(&event) {
            self.local.permission_level = i32::from(event) - 24;
        }
        // `ClientPacketListener.handleEntityEvent`: 35 is a totem saving its holder.
        if event == 35 {
            self.queue_emitter(id, "totem_of_undying", 30, Some("minecraft:item.totem.use"));
        }
        // `LivingEntity.handleEntityEvent`: 3 is a death.
        if event == 3 {
            if self.local.player_id == Some(id)
                && let Some(player) = self.local.player.as_mut()
            {
                player.handle_death_event();
            }
            self.world.entities_mut().apply_death_event(id);
        } else {
            self.world.apply_entity_event(id, event);
        }
        Ok(())
    }

    fn on_damage_event(
        &mut self,
        (id, damage_type): (EntityId, Option<usize>),
    ) -> Result<(), ConnectionError> {
        self.world.entities_mut().apply_damage_event(id);
        if self.local.player_id == Some(id) {
            let name = damage_type
                .and_then(|index| self.registry_names("minecraft:damage_type").get(index))
                .cloned()
                .unwrap_or_default();
            if let Some(player) = self.local.player.as_mut() {
                player.handle_damage_event(&name);
            }
        }
        Ok(())
    }

    fn on_hurt_animation(&mut self, (id, yaw): (EntityId, f32)) -> Result<(), ConnectionError> {
        self.world.entities_mut().apply_hurt_animation(id);
        if self.local.player_id == Some(id)
            && let Some(player) = self.local.player.as_mut()
        {
            player.animate_hurt(yaw);
        }
        Ok(())
    }

    fn on_set_entity_data(
        &mut self,
        packet: EntityMetadataObservation,
    ) -> Result<(), ConnectionError> {
        entities::metadata(&mut self.world, &packet);
        Ok(())
    }

    fn on_set_equipment(
        &mut self,
        packet: EntityEquipmentObservation,
    ) -> Result<(), ConnectionError> {
        entities::equipment(&mut self.world, &packet);
        Ok(())
    }

    fn on_update_attributes(
        &mut self,
        packet: AttributeUpdateObservation,
    ) -> Result<(), ConnectionError> {
        let world = &mut self.world;
        if !world.apply_local_player_attributes(packet.entity_id, &packet.attributes) {
            world
                .entities_mut()
                .apply_attributes(packet.entity_id, &packet.attributes);
        }
        Ok(())
    }

    fn on_update_mob_effect(
        &mut self,
        packet: MobEffectUpdateObservation,
    ) -> Result<(), ConnectionError> {
        self.world.apply_local_player_mob_effect(
            packet.entity_id,
            packet.effect_registry_id,
            packet.amplifier,
            packet.duration_ticks,
            packet.flags,
        );
        Ok(())
    }

    fn on_remove_mob_effect(
        &mut self,
        packet: MobEffectRemoveObservation,
    ) -> Result<(), ConnectionError> {
        self.world
            .remove_local_player_mob_effect(packet.entity_id, packet.effect_registry_id);
        Ok(())
    }
}
