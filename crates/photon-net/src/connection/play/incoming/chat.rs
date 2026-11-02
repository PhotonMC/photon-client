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
    (
        PLAY_CLIENTBOUND_SERVER_DATA_PACKET_ID,
        typed::<ServerDataPacket>,
    ),
    (
        PLAY_CLIENTBOUND_CUSTOM_CHAT_COMPLETIONS_PACKET_ID,
        typed::<CustomChatCompletions>,
    ),
];

/// `handleServerData`: what it changes is the saved server entry the player joined from, which
/// the app holds.
impl ClientboundPacket for ServerDataPacket {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        session.social.server_updates.push(*self);
        Ok(())
    }
}

/// `handleCustomChatCompletions`: `modifyCustomCompletions`.
impl ClientboundPacket for CustomChatCompletions {
    fn handle(self: Box<Self>, session: &mut PlaySession) -> Result<(), ConnectionError> {
        let words = &mut session.social.custom_completions;
        match self.action {
            CompletionAction::Add => words.extend(self.entries),
            CompletionAction::Remove => {
                for entry in &self.entries {
                    words.remove(entry);
                }
            }
            CompletionAction::Set => {
                words.clear();
                words.extend(self.entries);
            }
        }
        Ok(())
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::play::profile_tests::session;

    fn completions(action: CompletionAction, entries: &[&str]) -> Box<CustomChatCompletions> {
        Box::new(CustomChatCompletions {
            action,
            entries: entries.iter().map(|entry| (*entry).to_owned()).collect(),
        })
    }

    #[test]
    fn server_data_waits_for_the_app_to_apply_it() {
        let (mut session, _server) = session();
        let packet = ServerDataPacket {
            motd: photon_core::text::Component::literal("hi"),
            icon: Some(vec![1, 2]),
        };
        Box::new(packet.clone())
            .handle(&mut session)
            .expect("handled");
        assert_eq!(session.take_server_updates(), vec![packet]);
        assert!(session.take_server_updates().is_empty());
    }

    #[test]
    fn custom_completions_are_added_removed_and_set() {
        let (mut session, _server) = session();
        completions(CompletionAction::Add, &["b", "a", "a"])
            .handle(&mut session)
            .expect("handled");
        assert_eq!(session.custom_completions(), ["a", "b"]);
        completions(CompletionAction::Remove, &["a", "missing"])
            .handle(&mut session)
            .expect("handled");
        assert_eq!(session.custom_completions(), ["b"]);
        completions(CompletionAction::Set, &["z", "y"])
            .handle(&mut session)
            .expect("handled");
        assert_eq!(session.custom_completions(), ["y", "z"]);
    }
}
