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

/// `ClientboundLowDiskSpaceWarningPacket`: no payload; the client shows a toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LowDiskSpaceWarning;

impl Decode for LowDiskSpaceWarning {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        if !payload.is_empty() {
            return Err(ConnectionError::Codec("low disk space trailing payload"));
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
            // 26.3 reads an id out of range as the first action (`OutOfBoundsStrategy.ZERO`).
            _ if photon_core::versions::behaviour::out_of_range_enum_ids_read_as_first() => {
                CompletionAction::Add
            }
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

/// `ClientboundServerDataPacket`: the server's message of the day and, if it sends one, its icon
/// (PNG bytes, not yet checked).
#[derive(Debug, Clone, PartialEq)]
pub struct ServerDataPacket {
    pub motd: photon_core::text::Component,
    pub icon: Option<Vec<u8>>,
}

impl Decode for ServerDataPacket {
    fn decode(
        mut payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let input = &mut payload;
        let motd = read_chat_component(input, "server data motd")?;
        let icon = if read_bool(input, "server data has icon")? {
            let limit = input.len();
            Some(read_length_prefixed_bytes(input, limit, "server data icon")?.to_vec())
        } else {
            None
        };
        if !input.is_empty() {
            return Err(ConnectionError::Codec("server data trailing payload"));
        }
        Ok(Self { motd, icon })
    }
}

/// `ClientboundTagQueryPacket`: the tag the server read for a query the client sent, or none.
#[derive(Debug, Clone, PartialEq)]
pub struct TagQueryPacket {
    pub transaction_id: i32,
    pub tag: Option<Vec<photon_core::nbt::Field>>,
}

impl Decode for TagQueryPacket {
    fn decode(
        mut payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let input = &mut payload;
        let transaction_id = read_varint(input, "tag query transaction")?;
        // `readNbt`: the end tag is none, anything but a compound is refused.
        let tag = match photon_core::nbt::read_any_nbt_wire_order(input)
            .map_err(|_| ConnectionError::Codec("tag query tag"))?
        {
            None => None,
            Some(photon_core::nbt::FieldValue::Compound(fields)) => Some(fields),
            Some(_) => return Err(ConnectionError::Codec("tag query not a compound")),
        };
        if !input.is_empty() {
            return Err(ConnectionError::Codec("tag query trailing payload"));
        }
        Ok(Self {
            transaction_id,
            tag,
        })
    }
}
