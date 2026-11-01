//! The in-world game rule screen's traffic.

use super::*;

impl PlaySession {
    /// `InWorldGameRulesScreen.initContent`: ask the server for its game rule values, and drop
    /// any older answer (the reference ignores an answer when no rule screen is open).
    pub fn request_game_rules(&mut self) -> Result<(), ConnectionError> {
        self.local.game_rule_values = None;
        self.send(
            PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID,
            encode_request_game_rule_values,
        )
    }

    /// The game rule values that arrived since the rule screen last looked.
    pub fn take_game_rule_values(&mut self) -> Option<Vec<(String, String)>> {
        self.local.game_rule_values.take()
    }

    /// `InWorldGameRulesScreen.onDone`: the changed rules, each with its new value as text.
    pub fn set_game_rules(&mut self, entries: &[(String, String)]) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_SET_GAME_RULE_PACKET_ID, |output| {
            encode_set_game_rules(output, entries)
        })
    }
}
