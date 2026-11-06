use super::*;

/// The packets of this family, by packet id.
pub(super) const DECODERS: &[(i32, Decoder)] = &[
    (
        PLAY_CLIENTBOUND_TAG_QUERY_PACKET_ID,
        typed::<TagQueryPacket>,
    ),
    (
        PLAY_CLIENTBOUND_START_CONFIGURATION_PACKET_ID,
        typed::<StartConfiguration>,
    ),
    (PLAY_CLIENTBOUND_TRANSFER_PACKET_ID, typed::<Transfer>),
    (
        PLAY_CLIENTBOUND_TEST_INSTANCE_BLOCK_STATUS_PACKET_ID,
        typed::<TestInstanceBlockStatus>,
    ),
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
        PLAY_CLIENTBOUND_SERVER_LINKS_PACKET_ID,
        typed::<ServerLinksPacket>,
    ),
    (
        PLAY_CLIENTBOUND_OPEN_SIGN_EDITOR_PACKET_ID,
        typed::<OpenSignEditor>,
    ),
    (PLAY_CLIENTBOUND_OPEN_BOOK_PACKET_ID, typed::<OpenBook>),
    (
        PLAY_CLIENTBOUND_POST_EFFECTS_PACKET_ID,
        typed::<PostEffects>,
    ),
    (
        PLAY_CLIENTBOUND_ADD_TRANSIENT_BLOCK_PACKET_ID,
        typed::<AddTransientBlock>,
    ),
];

impl ClientboundPacket for PostEffects {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.local.post_effects = self.0;
        Ok(())
    }
}

/// The block is only drawn for a moment, while a section mesh is made (the reference queues it
/// for the renderer); nothing here keeps it.
impl ClientboundPacket for AddTransientBlock {
    fn handle(self: Box<Self>, _session: &mut PlaySession) -> Result<(), ConnectionError> {
        Ok(())
    }
}

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
        session.state.packs.push(self.0);
        Ok(())
    }
}

impl ClientboundPacket for ResourcePackPop {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.state.packs.pop(self.0);
        Ok(())
    }
}

/// `handleTestInstanceBlockStatus`: the open screen shows it.
impl ClientboundPacket for TestInstanceBlockStatus {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.menus.test_instance_statuses.push(*self);
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
        session.social.dialog_events.push(DialogEvent::Clear);
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

/// How many `photon:` payloads may wait for the app; more are dropped.
const MAX_WAITING_PHOTON_PAYLOADS: usize = 64;

/// `handleCustomPayload`: the brand is kept, `photon:` payloads wait for the app, and other
/// payloads are not acted on.
impl ClientboundPacket for CustomPayload {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        if let Some(brand) = self.0.brand() {
            session.state.server_brand = Some(brand);
        }
        if self.0.id.namespace() == "photon" {
            let queue = &mut session.state.photon_payloads;
            if queue.len() < MAX_WAITING_PHOTON_PAYLOADS {
                queue.push((self.0.id.as_string(), self.0.data));
            }
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

/// `handleServerLinks`: the entries that pass the address check replace the links.
impl ClientboundPacket for ServerLinksPacket {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.state.server_links = photon_game::server_links::ServerLinks::trusted(self.0);
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
        session.menus.opened_book = Some(u8::from(self.0 != 0));
        Ok(())
    }
}

/// `handleTagQueryPacket`: an answer to no query that waits is dropped.
impl ClientboundPacket for TagQueryPacket {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session
            .social
            .debug_queries
            .answer(self.transaction_id, self.tag);
        Ok(())
    }
}
