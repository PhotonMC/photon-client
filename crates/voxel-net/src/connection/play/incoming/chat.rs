use super::*;

impl PlaySession {
    pub(super) fn on_system_chat(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        match parse_system_chat_event(payload) {
            Ok(event) => self.chat_events.push(event),
            Err(error) => self.warn(format!("system chat: {error:?}")),
        };
        Ok(())
    }

    pub(super) fn on_disguised_chat(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        match parse_disguised_chat_event(payload) {
            Ok(event) => self.chat_events.push(event),
            Err(error) => self.warn(format!("disguised chat: {error:?}")),
        }
        Ok(())
    }

    pub(super) fn on_player_chat(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        match parse_player_chat_packet(payload) {
            Ok(packet) => self.receive_player_chat(packet),
            Err(error) => self.warn(format!("player chat: {error:?}")),
        };
        Ok(())
    }

    pub(super) fn on_delete_chat(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        match parse_delete_chat_packet(payload) {
            Ok(packed) => match self.signature_cache.unpack(&packed) {
                Some(signature) => self.chat_events.push(ChatEvent::Delete(signature)),
                // The reference drops the connection over a number it
                // does not know.
                None => self.warn("delete chat: unknown signature".to_owned()),
            },
            Err(error) => self.warn(format!("delete chat: {error:?}")),
        };
        Ok(())
    }

    pub(super) fn on_command_suggestions(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        match parse_command_suggestions(payload) {
            Ok(suggestions) => self.suggestions = Some(suggestions),
            Err(error) => self.warn(format!("suggestions: {error:?}")),
        }
        Ok(())
    }

    pub(super) fn on_waypoint(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        match parse_waypoint_packet(payload) {
            Ok((operation, waypoint)) => self.waypoints.apply(operation, waypoint),
            Err(error) => self.warn(format!("waypoint: {error:?}")),
        };
        Ok(())
    }

    pub(super) fn on_commands(&mut self, payload: &[u8]) -> Result<(), ConnectionError> {
        match parse_commands_packet(payload) {
            Ok(tree) => self.commands = Some(std::sync::Arc::new(tree)),
            Err(error) => self.warn(format!("commands: {error:?}")),
        };
        Ok(())
    }
}
