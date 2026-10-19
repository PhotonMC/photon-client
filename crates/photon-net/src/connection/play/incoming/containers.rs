use super::*;

/// The packets of this family, by packet id.
pub(super) const DECODERS: &[(i32, Decoder)] = &[
    (PLAY_CLIENTBOUND_OPEN_SCREEN_PACKET_ID, typed::<OpenScreen>),
    (
        PLAY_CLIENTBOUND_MOUNT_SCREEN_OPEN_PACKET_ID,
        typed::<MountScreenOpen>,
    ),
    (
        PLAY_CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID,
        typed::<ContainerClose>,
    ),
    (
        PLAY_CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID,
        typed::<ContainerSetContent>,
    ),
    (
        PLAY_CLIENTBOUND_MERCHANT_OFFERS_PACKET_ID,
        typed::<MerchantOffers>,
    ),
    (
        PLAY_CLIENTBOUND_CONTAINER_SET_DATA_PACKET_ID,
        typed::<ContainerSetData>,
    ),
    (PLAY_CLIENTBOUND_COOLDOWN_PACKET_ID, typed::<Cooldown>),
    (
        PLAY_CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID,
        typed::<ContainerSetSlot>,
    ),
    (
        PLAY_CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID,
        typed::<SetCursorItem>,
    ),
    (
        PLAY_CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID,
        typed::<SetPlayerInventory>,
    ),
];

impl ClientboundPacket for OpenScreen {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.world.apply_local_player_open_menu(
            self.0.container_id,
            self.0.menu_type_id,
            &self.0.title_nbt,
        );
        Ok(())
    }
}

/// `handleMountScreenOpen`: the screen of a horse or nautilus the player rides or tends.
impl ClientboundPacket for MountScreenOpen {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.open_mount_screen(&self);
        Ok(())
    }
}

impl ClientboundPacket for ContainerClose {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session
            .world
            .apply_local_player_container_close(self.0.container_id);
        Ok(())
    }
}

impl ClientboundPacket for ContainerSetContent {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let packet = self.0;
        session.world.apply_local_player_container_content(
            packet.container_id,
            packet.state_id,
            &packet.items,
            packet.carried_item.as_ref(),
        );
        Ok(())
    }
}

impl ClientboundPacket for MerchantOffers {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.menus.merchant = Some(self.0);
        Ok(())
    }
}

impl ClientboundPacket for ContainerSetData {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.world.apply_local_player_container_data(
            self.0.container_id,
            self.0.id,
            self.0.value,
        );
        Ok(())
    }
}

/// `handleItemCooldown`: a group's cooldown starts, or ends at zero.
impl ClientboundPacket for Cooldown {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        if self.duration == 0 {
            session.local.item_cooldowns.remove(&self.group);
        } else {
            session.local.item_cooldowns.add(self.group, self.duration);
        }
        Ok(())
    }
}

impl ClientboundPacket for ContainerSetSlot {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let packet = self.0;
        session.note_received_item(packet.item.as_ref());
        session.world.apply_local_player_container_slot(
            packet.container_id,
            packet.state_id,
            packet.slot,
            packet.item.as_ref(),
        );
        Ok(())
    }
}

impl ClientboundPacket for SetCursorItem {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.note_received_item(self.0.item.as_ref());
        session
            .world
            .apply_local_player_carried_item(self.0.item.as_ref());
        Ok(())
    }
}

impl ClientboundPacket for SetPlayerInventory {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.note_received_item(self.0.item.as_ref());
        session
            .world
            .apply_local_player_inventory_slot(self.0.slot, self.0.item.as_ref());
        Ok(())
    }
}

impl PlaySession {
    /// `Tutorial.onGetItem`: an item the server put in a slot.
    fn note_received_item(&mut self, item: Option<&photon_world::ClientItemStackState>) {
        if let Some(item) = item {
            self.local.received_items.push(item.item_registry_id().0);
        }
    }

    /// `handleMountScreenOpen`: what the screen needs of the entity it is for.
    fn open_mount_screen(&mut self, packet: &MountScreenOpen) {
        use photon_game::menus::container::{MOUNT_MENU_TYPE, Mount, MountEntity, MountFamily};
        let MountScreenOpen {
            container_id,
            columns,
            entity_id,
        } = *packet;
        let world = &self.world;
        let Some(entity) = world.entities().at(entity_id) else {
            return;
        };
        let Some(name) = photon_data::entity_type::name(entity.type_id) else {
            return;
        };
        let Some(family) = MountFamily::of_entity(name) else {
            return;
        };
        let tame = match family {
            MountFamily::Nautilus => entity.is_tame(),
            _ => entity.horse().is_some_and(|horse| horse.is_tamed()),
        };
        let mount = Mount {
            family,
            columns: columns.clamp(0, 9) as u8,
        };
        let state = MountEntity {
            entity_id,
            entity_type: entity.type_id,
            can_use: MountEntity::can_use_slots(name, entity.is_baby(), tame),
            entity_tags: world.block_state_registry().entity_type_tags().clone(),
        };
        // The screen's title is the entity's name, which the packet does not carry.
        let key = format!("entity.minecraft.{}", name.trim_start_matches("minecraft:"));
        let title = photon_game::item::default_components::translatable(&key);
        self.menus.mount = Some((container_id, mount, state));
        self.world
            .apply_local_player_open_menu(container_id, MOUNT_MENU_TYPE, &title);
    }
}
