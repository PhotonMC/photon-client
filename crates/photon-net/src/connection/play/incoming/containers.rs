use super::*;

impl PlaySession {
    pub(super) fn on_open_screen(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let packet = parse_open_screen_packet(payload)?;
        self.trace.world.apply_local_player_open_menu(
            packet.container_id,
            packet.menu_type_id,
            &packet.title_nbt,
        );
        Ok(())
    }

    /// `handleMountScreenOpen`: the screen of a horse or nautilus the player rides or tends.
    pub(super) fn on_mount_screen_open(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        use photon_game::menus::container::{MOUNT_MENU_TYPE, Mount, MountEntity, MountFamily};
        let mut input = payload;
        let container_id = read_varint(&mut input, "mount screen container id")?;
        let columns = read_varint(&mut input, "mount screen columns")?;
        let Ok(entity_id) = <[u8; 4]>::try_from(input).map(i32::from_be_bytes) else {
            return Err(ConnectionError::Codec("mount screen entity id"));
        };
        let world = &self.trace.world;
        let Some(entity) = world.entities().at(entity_id) else {
            return Ok(());
        };
        let Some(name) = photon_data::entity_type::name(entity.type_id) else {
            return Ok(());
        };
        let Some(family) = MountFamily::of_entity(name) else {
            return Ok(());
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
            entity_type: entity.type_id,
            can_use: MountEntity::can_use_slots(name, entity.is_baby(), tame),
            entity_tags: world.block_state_registry().entity_type_tags().clone(),
        };
        // The screen's title is the entity's name, which the packet does not carry.
        let key = format!("entity.minecraft.{}", name.trim_start_matches("minecraft:"));
        let title = photon_game::item::default_components::translatable(&key);
        self.mount = Some((container_id, mount, state));
        self.trace
            .world
            .apply_local_player_open_menu(container_id, MOUNT_MENU_TYPE, &title);
        Ok(())
    }

    pub(super) fn on_container_close(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let packet = parse_container_close_packet(payload)?;
        self.trace
            .world
            .apply_local_player_container_close(packet.container_id);
        Ok(())
    }

    pub(super) fn on_container_set_content(
        &mut self,
        payload: &[u8],
    ) -> Result<(), ConnectionError> {
        let packet = match parse_container_set_content_packet(payload) {
            Ok(packet) => packet,
            // An item with a component the parser does not know
            // leaves that stack unknown rather than the session.
            Err(error) => {
                self.warn(format!("inventory: {error:?}"));
                return self.maybe_finish_loading();
            }
        };
        self.trace.world.apply_local_player_container_content(
            packet.container_id,
            packet.state_id,
            &packet.items,
            packet.carried_item.as_ref(),
        );
        Ok(())
    }

    pub(super) fn on_merchant_offers(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let (container_id, state) = parse_merchant_offers_packet(payload)?;
        self.merchant = Some((container_id, state));
        Ok(())
    }

    pub(super) fn on_container_set_data(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let packet = parse_container_set_data_packet(payload)?;
        self.trace.world.apply_local_player_container_data(
            packet.container_id,
            packet.id,
            packet.value,
        );
        Ok(())
    }

    /// `handleItemCooldown`: a group's cooldown starts, or ends at zero.
    pub(super) fn on_cooldown(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let mut input = payload;
        let group = photon_core::identifier::read_identifier(&mut input)
            .map_err(|_| ConnectionError::Codec("cooldown group"))?
            .as_string();
        let duration = photon_core::wire::read_varint(&mut input)
            .map_err(|_| ConnectionError::Codec("cooldown duration"))?;
        if duration == 0 {
            self.item_cooldowns.remove(&group);
        } else {
            self.item_cooldowns.add(group, duration);
        }
        Ok(())
    }

    pub(super) fn on_container_set_slot(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let packet = match parse_container_set_slot_packet(payload) {
            Ok(packet) => packet,
            // An item with a component the parser does not know
            // leaves that stack unknown rather than the session.
            Err(error) => {
                self.warn(format!("inventory: {error:?}"));
                return self.maybe_finish_loading();
            }
        };
        self.trace.world.apply_local_player_container_slot(
            packet.container_id,
            packet.state_id,
            packet.slot,
            packet.item.as_ref(),
        );
        Ok(())
    }

    pub(super) fn on_set_cursor_item(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        let packet = match parse_cursor_item_packet(payload) {
            Ok(packet) => packet,
            // An item with a component the parser does not know
            // leaves that stack unknown rather than the session.
            Err(error) => {
                self.warn(format!("inventory: {error:?}"));
                return self.maybe_finish_loading();
            }
        };
        self.trace
            .world
            .apply_local_player_carried_item(packet.item.as_ref());
        Ok(())
    }

    pub(super) fn on_set_player_inventory(
        &mut self,
        payload: &[u8],
    ) -> Result<(), ConnectionError> {
        let packet = match parse_set_player_inventory_packet(payload) {
            Ok(packet) => packet,
            // An item with a component the parser does not know
            // leaves that stack unknown rather than the session.
            Err(error) => {
                self.warn(format!("inventory: {error:?}"));
                return self.maybe_finish_loading();
            }
        };
        self.trace
            .world
            .apply_local_player_inventory_slot(packet.slot, packet.item.as_ref());
        Ok(())
    }
}
