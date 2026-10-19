use super::*;

/// The packets of this family, by packet id.
pub(super) const DECODERS: &[(i32, Decoder)] = &[
    (PLAY_CLIENTBOUND_SYSTEM_CHAT_PACKET_ID, typed::<SystemChat>),
    (
        PLAY_CLIENTBOUND_DISGUISED_CHAT_PACKET_ID,
        typed::<DisguisedChat>,
    ),
    (PLAY_CLIENTBOUND_PLAYER_CHAT_PACKET_ID, typed::<PlayerChat>),
    (PLAY_CLIENTBOUND_DELETE_CHAT_PACKET_ID, typed::<DeleteChat>),
    (
        PLAY_CLIENTBOUND_COMMAND_SUGGESTIONS_PACKET_ID,
        typed::<Suggestions>,
    ),
    (PLAY_CLIENTBOUND_WAYPOINT_PACKET_ID, typed::<Waypoint>),
    (PLAY_CLIENTBOUND_COMMANDS_PACKET_ID, typed::<Commands>),
];

impl ClientboundPacket for SystemChat {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.social.chat_events.push(self.0);
        Ok(())
    }
}

impl ClientboundPacket for DisguisedChat {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.social.chat_events.push(self.0);
        Ok(())
    }
}

impl ClientboundPacket for PlayerChat {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.receive_player_chat(self.0);
        Ok(())
    }
}

impl ClientboundPacket for DeleteChat {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        match session.social.signature_cache.unpack(&self.0) {
            Some(signature) => {
                session.social.last_seen.ignore_pending(&signature);
                session
                    .social
                    .chat_events
                    .push(ChatEvent::Delete(signature));
            }
            // The reference drops the connection over a number it does not know.
            None => session.warn("delete chat: unknown signature".to_owned()),
        }
        Ok(())
    }
}

impl ClientboundPacket for Suggestions {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.social.suggestions = Some(self.0);
        Ok(())
    }
}

impl ClientboundPacket for Waypoint {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let (operation, waypoint) = self.0;
        session.social.waypoints.apply(operation, waypoint);
        Ok(())
    }
}

impl ClientboundPacket for Commands {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.social.commands = Some(std::sync::Arc::new(self.0));
        Ok(())
    }
}
