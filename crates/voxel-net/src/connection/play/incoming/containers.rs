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
