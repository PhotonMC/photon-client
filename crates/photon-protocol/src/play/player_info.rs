use super::*;

/// `ClientboundPlayerInfoUpdatePacket`: for each player (by its id), the
/// actions the packet's set names, in order. Hat flags are read and dropped.
pub fn parse_player_info_update(input: &[u8]) -> Result<Vec<PlayerActions>, ConnectionError> {
    let mut input = input;
    let actions = read_byte(&mut input, "player info actions")?;
    let count = read_varint(&mut input, "player info count")?;
    let mut entries = Vec::new();
    for _ in 0..count {
        let uuid = photon_core::binary::read_uuid(&mut input)
            .map_err(|_| ConnectionError::Codec("player info uuid"))?;
        let mut list = Vec::new();
        for bit in 0..8 {
            if actions & (1 << bit) == 0 {
                continue;
            }
            match bit {
                0 => {
                    let name = read_chat_string(&mut input, 16, "player info name")?;
                    let properties = read_properties(&mut input)?;
                    list.push(InfoAction::Add { name, properties });
                }
                1 => list.push(InfoAction::ChatSession(read_chat_session(&mut input)?)),
                2 => list.push(InfoAction::GameMode(read_varint(&mut input, "game mode")?)),
                3 => list.push(InfoAction::Listed(read_bool(&mut input, "listed")?)),
                4 => list.push(InfoAction::Latency(read_varint(&mut input, "latency")?)),
                5 => {
                    let name = if read_bool(&mut input, "display name flag")? {
                        Some(read_chat_component(&mut input, "display name")?)
                    } else {
                        None
                    };
                    list.push(InfoAction::DisplayName(name));
                }
                6 => list.push(InfoAction::ListOrder(read_varint(
                    &mut input,
                    "list order",
                )?)),
                _ => {
                    read_bool(&mut input, "hat")?;
                }
            }
        }
        entries.push(((uuid.most, uuid.least), list));
    }
    Ok(entries)
}

/// `RemoteChatSession.Data` after its presence flag.
fn read_chat_session(input: &mut &[u8]) -> Result<Option<RemoteChatSession>, ConnectionError> {
    if !read_bool(input, "player info chat flag")? {
        return Ok(None);
    }
    let session_id = photon_core::binary::read_uuid(input)
        .map_err(|_| ConnectionError::Codec("chat session id"))?;
    let expires_at_ms = read_i64(input, "chat key expiry")?;
    let mut arrays = Vec::with_capacity(2);
    for limit in [usize::MAX, 4096] {
        let length = read_varint(input, "chat key length")?.max(0) as usize;
        if length > limit || input.len() < length {
            return Err(ConnectionError::Codec("chat key"));
        }
        let (bytes, rest) = input.split_at(length);
        *input = rest;
        arrays.push(bytes.to_vec());
    }
    let key_signature = arrays.pop().unwrap_or_default();
    let public_key = arrays.pop().unwrap_or_default();
    Ok(Some(RemoteChatSession {
        session_id,
        expires_at_ms,
        public_key,
        key_signature,
    }))
}

/// A profile's properties, each with an optional signature.
fn read_properties(input: &mut &[u8]) -> Result<Vec<ProfileProperty>, ConnectionError> {
    let count = read_varint(input, "player info properties")?;
    let mut properties = Vec::new();
    for _ in 0..count {
        let name = read_chat_string(input, 64, "property name")?;
        let value = read_chat_string(input, 32767, "property value")?;
        let signature = if read_bool(input, "property signature flag")? {
            Some(read_chat_string(input, 1024, "property signature")?)
        } else {
            None
        };
        properties.push(ProfileProperty {
            name,
            value,
            signature,
        });
    }
    Ok(properties)
}

/// `ClientboundPlayerInfoRemovePacket`.
pub fn parse_player_info_remove(input: &[u8]) -> Result<Vec<(u64, u64)>, ConnectionError> {
    let mut input = input;
    let count = read_varint(&mut input, "player info remove count")?;
    let mut ids = Vec::new();
    for _ in 0..count {
        let uuid = photon_core::binary::read_uuid(&mut input)
            .map_err(|_| ConnectionError::Codec("player info remove uuid"))?;
        ids.push((uuid.most, uuid.least));
    }
    Ok(ids)
}

/// A `PlayerInfo` of the tab list: what the client keeps of a player.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerInfo {
    pub name: String,
    /// 0 survival, 1 creative, 2 adventure, 3 spectator.
    pub game_mode: i32,
    pub latency: i32,
    pub listed: bool,
    pub display_name: Option<photon_core::text::Component>,
    pub list_order: i32,
    /// The profile's properties (`GameProfile.properties`), among them the packed textures.
    pub properties: Vec<ProfileProperty>,
}

/// A `Property` of a game profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileProperty {
    pub name: String,
    pub value: String,
    /// The base64 signature, where the property is signed.
    pub signature: Option<String>,
}

/// One `ClientboundPlayerInfoUpdatePacket` action of a player.
#[derive(Debug, Clone, PartialEq)]
pub enum InfoAction {
    Add {
        name: String,
        properties: Vec<ProfileProperty>,
    },
    GameMode(i32),
    Listed(bool),
    Latency(i32),
    DisplayName(Option<photon_core::text::Component>),
    ListOrder(i32),
    /// The player's chat session, or none where the player has no profile key.
    ChatSession(Option<RemoteChatSession>),
}

/// `RemoteChatSession.Data`: the session a player signs its chat in and the public half of the
/// key pair (`ProfilePublicKey.Data`) with the services' signature over it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteChatSession {
    pub session_id: photon_core::binary::Uuid,
    pub expires_at_ms: i64,
    /// X.509 DER.
    pub public_key: Vec<u8>,
    pub key_signature: Vec<u8>,
}

/// A player's id and what a player info packet does to it.
pub type PlayerActions = ((u64, u64), Vec<InfoAction>);

wrapped_packet!(
    /// `ClientboundPlayerInfoUpdatePacket`.
    PlayerInfoUpdate(Vec<PlayerActions>) = parse_player_info_update,
    OnError::Warn("player info")
);
wrapped_packet!(
    /// `ClientboundPlayerInfoRemovePacket`: the profile ids that leave the tab list.
    PlayerInfoRemove(Vec<(u64, u64)>) = parse_player_info_remove,
    OnError::Warn("player info remove")
);
