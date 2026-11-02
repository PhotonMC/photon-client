//! Small clientbound packets: combat notices and the words the server adds to chat completion.

use super::*;

/// `ClientboundPlayerCombatEnterPacket`: no payload, and the client does nothing with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerCombatEnter;

impl Decode for PlayerCombatEnter {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        if !payload.is_empty() {
            return Err(ConnectionError::Codec("combat enter trailing payload"));
        }
        Ok(Self)
    }
}

/// `ClientboundPlayerCombatEndPacket`: how long the fight lasted, in ticks; the client does
/// nothing with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerCombatEnd {
    pub duration: i32,
}

impl Decode for PlayerCombatEnd {
    fn decode(
        mut payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let duration = read_varint(&mut payload, "combat end duration")?;
        if !payload.is_empty() {
            return Err(ConnectionError::Codec("combat end trailing payload"));
        }
        Ok(Self { duration })
    }
}

/// `ClientboundCustomChatCompletionsPacket.Action`, in the order of its ordinals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionAction {
    Add,
    Remove,
    Set,
}

/// `ClientboundCustomChatCompletionsPacket`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomChatCompletions {
    pub action: CompletionAction,
    pub entries: Vec<String>,
}

impl Decode for CustomChatCompletions {
    fn decode(
        mut payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let input = &mut payload;
        let action = match read_varint(input, "completion action")? {
            0 => CompletionAction::Add,
            1 => CompletionAction::Remove,
            2 => CompletionAction::Set,
            _ => return Err(ConnectionError::Codec("completion action")),
        };
        let count = read_count(input, "completion count")?;
        let mut entries = Vec::with_capacity(count.min(1024));
        for _ in 0..count {
            entries.push(read_chat_string(input, 32_767, "completion entry")?);
        }
        if !input.is_empty() {
            return Err(ConnectionError::Codec("completions trailing payload"));
        }
        Ok(Self { action, entries })
    }
}
