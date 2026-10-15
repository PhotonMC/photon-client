use super::*;

/// The packets of this family, by packet id.
pub(super) const DECODERS: &[(i32, Decoder)] = &[
    (
        PLAY_CLIENTBOUND_START_CONFIGURATION_PACKET_ID,
        typed::<StartConfiguration>,
    ),
    (PLAY_CLIENTBOUND_TRANSFER_PACKET_ID, typed::<Transfer>),
    (
        PLAY_CLIENTBOUND_RESOURCE_PACK_PUSH_PACKET_ID,
        typed::<ResourcePackPush>,
    ),
    (
        PLAY_CLIENTBOUND_RESOURCE_PACK_POP_PACKET_ID,
        typed::<ResourcePackPop>,
    ),
    (PLAY_CLIENTBOUND_SHOW_DIALOG_PACKET_ID, typed::<ShowDialog>),
    (
        PLAY_CLIENTBOUND_CLEAR_DIALOG_PACKET_ID,
        typed::<ClearDialog>,
    ),
    (
        PLAY_CLIENTBOUND_COOKIE_REQUEST_PACKET_ID,
        typed::<CookieRequest>,
    ),
    (
        PLAY_CLIENTBOUND_STORE_COOKIE_PACKET_ID,
        typed::<StoreCookie>,
    ),
    (
        PLAY_CLIENTBOUND_CUSTOM_PAYLOAD_PACKET_ID,
        typed::<CustomPayload>,
    ),
    (
        PLAY_CLIENTBOUND_CUSTOM_REPORT_DETAILS_PACKET_ID,
        typed::<CustomReportDetails>,
    ),
    (
        PLAY_CLIENTBOUND_OPEN_SIGN_EDITOR_PACKET_ID,
        typed::<OpenSignEditor>,
    ),
    (PLAY_CLIENTBOUND_OPEN_BOOK_PACKET_ID, typed::<OpenBook>),
];

impl ClientboundPacket for StartConfiguration {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.reconfigure()
    }
}

impl ClientboundPacket for Transfer {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.transfer(self.0)
    }
}

impl ClientboundPacket for ResourcePackPush {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_resource_pack_push(self.0)
    }
}

/// Nothing was loaded, so there is nothing to take off.
impl ClientboundPacket for ResourcePackPop {
    fn handle(self: Box<Self>, _session: &mut PlaySession) -> Result<(), ConnectionError> {
        Ok(())
    }
}

impl ClientboundPacket for ShowDialog {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_show_dialog(self.0);
        Ok(())
    }
}

impl ClientboundPacket for ClearDialog {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.dialog_events.push(DialogEvent::Clear);
        Ok(())
    }
}

impl ClientboundPacket for CookieRequest {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_cookie_request(&self.0)
    }
}

impl ClientboundPacket for StoreCookie {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.state.cookies.store(&self.0.key, self.0.payload);
        Ok(())
    }
}

/// `handleCustomPayload`: the brand is kept, other payloads are not acted on.
impl ClientboundPacket for CustomPayload {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        if let Some(brand) = self.0.brand() {
            session.state.server_brand = Some(brand);
        }
        Ok(())
    }
}

impl ClientboundPacket for CustomReportDetails {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.state.custom_report_details = self.0.details;
        Ok(())
    }
}

impl ClientboundPacket for OpenSignEditor {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.on_open_sign_editor(self.pos, self.front);
        Ok(())
    }
}

impl ClientboundPacket for OpenBook {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.opened_book = Some(u8::from(self.0 != 0));
        Ok(())
    }
}
