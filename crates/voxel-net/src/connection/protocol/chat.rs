use super::*;

/// `ClientboundDisconnectPacket`: the reason the server gives.
pub(crate) fn read_disconnect_reason(
    input: &mut &[u8],
) -> Result<voxel_game::text::Component, ConnectionError> {
    read_chat_component(input, "disconnect reason")
}

impl Default for SignatureCache {
    fn default() -> Self {
        Self {
            entries: vec![None; 128],
        }
    }
}

/// Wire-level fields decoded from one clientbound system-chat packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemChatObservation {
    pub overlay: bool,
    pub translation_key: Option<String>,
}

/// How a chat type is named on the wire (`Holder<ChatType>`): by its
/// registry id, or written out.
#[derive(Debug, Clone, PartialEq)]
pub enum ChatHolder {
    Registry(i32),
    Direct(voxel_game::chat::ChatDecoration),
}

/// `ChatType.Bound`: the type, the sender's name and the target's.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatBound {
    pub holder: ChatHolder,
    pub name: voxel_game::text::Component,
    pub target: Option<voxel_game::text::Component>,
}

/// A `MessageSignature`: 256 bytes.
pub type ChatSignature = Vec<u8>;

/// `MessageSignature.Packed`: an id in the signature cache, or the signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackedSignature {
    Cached(i32),
    Full(ChatSignature),
}

/// `FilterMask`: what the server hid of a message.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FilterMask {
    #[default]
    PassThrough,
    FullyFiltered,
    /// The bit set of hidden characters, as the words the network writes.
    Partial(Vec<u64>),
}

/// `ClientboundPlayerChatPacket` as it arrives, its signatures not yet
/// looked up in the cache.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerChatPacket {
    pub global_index: i32,
    pub sender: (u64, u64),
    pub signature: Option<ChatSignature>,
    pub signed_text: String,
    pub timestamp_millis: i64,
    pub last_seen: Vec<PackedSignature>,
    pub unsigned: Option<voxel_game::text::Component>,
    pub filter: FilterMask,
    pub bound: ChatBound,
}

/// A chat packet as the client shows it.
#[derive(Debug, Clone, PartialEq)]
pub enum ChatEvent {
    /// `ClientboundSystemChatPacket`: the action bar when `overlay`.
    System {
        content: voxel_game::text::Component,
        overlay: bool,
    },
    /// `ClientboundDisguisedChatPacket`.
    Disguised {
        content: voxel_game::text::Component,
        bound: ChatBound,
    },
    /// `ClientboundPlayerChatPacket`, its signatures looked up; whether the
    /// sender is a player the client knows and is the local player.
    Player {
        packet: PlayerChatPacket,
        sender_known: bool,
        sender_is_local: bool,
    },
    /// `ClientboundDeleteChatPacket`: the signature of the message to delete.
    Delete(ChatSignature),
}

/// `MessageSignatureCache`: the last 128 signatures, by place, which the
/// packets refer to by number.
#[derive(Debug, Clone)]
pub struct SignatureCache {
    pub(super) entries: Vec<Option<ChatSignature>>,
}

pub(super) fn read_chat_component(
    input: &mut &[u8],
    name: &'static str,
) -> Result<voxel_game::text::Component, ConnectionError> {
    voxel_core::nbt::read_any_nbt(input)
        .ok()
        .flatten()
        .and_then(|tag| voxel_game::text::Component::from_nbt(&tag))
        .ok_or(ConnectionError::Codec(name))
}

pub(super) fn read_chat_string(
    input: &mut &[u8],
    max: usize,
    name: &'static str,
) -> Result<String, ConnectionError> {
    voxel_core::utf8::read_utf8(input, max)
        .map(|units| String::from_utf16_lossy(&units))
        .map_err(|_| ConnectionError::Codec(name))
}

/// `ChatTypeDecoration.STREAM_CODEC`.
pub(super) fn read_chat_decoration(
    input: &mut &[u8],
) -> Result<voxel_game::chat::ChatDecoration, ConnectionError> {
    let translation_key = read_chat_string(input, 32767, "chat decoration key")?;
    let count = read_varint(input, "chat decoration parameters")?;
    let mut parameters = Vec::new();
    for _ in 0..count {
        let id = read_varint(input, "chat decoration parameter")?;
        parameters.push(
            voxel_game::chat::ChatParameter::from_id(id)
                .ok_or(ConnectionError::Codec("chat decoration parameter"))?,
        );
    }
    let style = voxel_core::nbt::read_any_nbt(input)
        .ok()
        .flatten()
        .and_then(|tag| voxel_game::text::Component::from_nbt(&tag))
        .map(|component| component.style)
        .ok_or(ConnectionError::Codec("chat decoration style"))?;
    Ok(voxel_game::chat::ChatDecoration {
        translation_key,
        parameters,
        style,
    })
}

/// `ChatType.Bound.STREAM_CODEC`: a holder whose id 0 is a chat type written
/// out (its chat decoration, then its narration), then the names.
pub(super) fn read_chat_bound(input: &mut &[u8]) -> Result<ChatBound, ConnectionError> {
    let holder = match read_varint(input, "chat type")? {
        0 => {
            let chat = read_chat_decoration(input)?;
            read_chat_decoration(input)?;
            ChatHolder::Direct(chat)
        }
        id => ChatHolder::Registry(id - 1),
    };
    let name = read_chat_component(input, "chat sender name")?;
    let target = if read_bool(input, "chat target flag")? {
        Some(read_chat_component(input, "chat target name")?)
    } else {
        None
    };
    Ok(ChatBound {
        holder,
        name,
        target,
    })
}

/// `ClientboundSystemChatPacket`.
pub(crate) fn parse_system_chat_event(input: &[u8]) -> Result<ChatEvent, ConnectionError> {
    let mut input = input;
    let content = read_chat_component(&mut input, "system chat component")?;
    let overlay = read_bool(&mut input, "system chat overlay")?;
    Ok(ChatEvent::System { content, overlay })
}

/// `ClientboundDisguisedChatPacket`.
pub(crate) fn parse_disguised_chat_event(input: &[u8]) -> Result<ChatEvent, ConnectionError> {
    let mut input = input;
    let content = read_chat_component(&mut input, "disguised chat message")?;
    let bound = read_chat_bound(&mut input)?;
    Ok(ChatEvent::Disguised { content, bound })
}

pub(super) fn read_packed_signature(input: &mut &[u8]) -> Result<PackedSignature, ConnectionError> {
    match read_varint(input, "packed signature")? {
        0 => Ok(PackedSignature::Full(read_signature(input)?)),
        id => Ok(PackedSignature::Cached(id - 1)),
    }
}

pub(super) fn read_signature(input: &mut &[u8]) -> Result<ChatSignature, ConnectionError> {
    if input.len() < 256 {
        return Err(ConnectionError::Codec("signature"));
    }
    let (signature, rest) = input.split_at(256);
    *input = rest;
    Ok(signature.to_vec())
}

/// `ClientboundPlayerChatPacket`.
pub(crate) fn parse_player_chat_packet(input: &[u8]) -> Result<PlayerChatPacket, ConnectionError> {
    let mut input = input;
    let global_index = read_varint(&mut input, "chat global index")?;
    let sender = voxel_core::binary::read_uuid(&mut input)
        .map_err(|_| ConnectionError::Codec("chat sender"))?;
    read_varint(&mut input, "chat index")?;
    let signature = if read_bool(&mut input, "chat signature flag")? {
        Some(read_signature(&mut input)?)
    } else {
        None
    };
    let signed_text = read_chat_string(&mut input, 256, "chat content")?;
    let timestamp_millis = read_i64(&mut input, "chat timestamp")?;
    skip_bytes(&mut input, 8, "chat salt")?;
    let count = read_varint(&mut input, "chat last seen")?;
    let mut last_seen = Vec::new();
    for _ in 0..count {
        last_seen.push(read_packed_signature(&mut input)?);
    }
    let unsigned = if read_bool(&mut input, "chat unsigned flag")? {
        Some(read_chat_component(&mut input, "chat unsigned content")?)
    } else {
        None
    };
    // `FilterMask`: pass through, fully filtered, or partly (with a bit set).
    let filter = match read_varint(&mut input, "chat filter mask")? {
        1 => FilterMask::FullyFiltered,
        2 => {
            let words = read_varint(&mut input, "chat filter bits")?.max(0) as usize;
            let mut bits = Vec::with_capacity(words);
            for _ in 0..words {
                bits.push(read_i64(&mut input, "chat filter word")? as u64);
            }
            FilterMask::Partial(bits)
        }
        _ => FilterMask::PassThrough,
    };
    let bound = read_chat_bound(&mut input)?;
    Ok(PlayerChatPacket {
        global_index,
        sender: (sender.most, sender.least),
        signature,
        signed_text,
        timestamp_millis,
        last_seen,
        unsigned,
        filter,
        bound,
    })
}

/// `ClientboundDeleteChatPacket`.
pub(crate) fn parse_delete_chat_packet(input: &[u8]) -> Result<PackedSignature, ConnectionError> {
    let mut input = input;
    read_packed_signature(&mut input)
}

/// `ServerboundChatPacket` for a session without a chat key: the message, the
/// time, a salt, no signature and an empty acknowledgement (offset 0, no
/// messages, checksum 1).
pub(crate) fn encode_chat(message: &str, millis: i64, salt: i64, output: &mut Vec<u8>) {
    let units: Vec<u16> = message.encode_utf16().collect();
    let _ = voxel_core::utf8::write_utf8(output, &units, 256);
    output.extend_from_slice(&millis.to_be_bytes());
    output.extend_from_slice(&salt.to_be_bytes());
    output.push(0);
    voxel_core::wire::write_varint(0, output);
    output.extend_from_slice(&[0, 0, 0]);
    output.push(1);
}

pub(crate) fn parse_system_chat_packet(
    input: &[u8],
) -> Result<SystemChatObservation, ConnectionError> {
    let mut input = input;
    let component = voxel_core::nbt::read_nbt(&mut input)
        .map_err(|_| ConnectionError::Codec("system chat component"))?;
    let translation_key = component.and_then(|fields| {
        fields.into_iter().find_map(|field| {
            if field.key == "translate" {
                match field.value {
                    voxel_core::nbt::FieldValue::Str(value) => Some(value),
                    _ => None,
                }
            } else {
                None
            }
        })
    });
    let overlay = read_byte(&mut input, "system chat overlay")? != 0;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("system chat trailing payload"));
    }
    Ok(SystemChatObservation {
        overlay,
        translation_key,
    })
}

impl SignatureCache {
    /// `MessageSignature.Packed.unpack`: `None` for a number nothing holds.
    pub fn unpack(&self, packed: &PackedSignature) -> Option<ChatSignature> {
        match packed {
            PackedSignature::Full(signature) => Some(signature.clone()),
            PackedSignature::Cached(id) => usize::try_from(*id)
                .ok()
                .and_then(|id| self.entries.get(id))
                .and_then(Clone::clone),
        }
    }

    /// `MessageSignatureCache.push`: the new signatures go in at the front,
    /// the old ones they do not repeat shifted back.
    pub fn push(&mut self, mut queue: std::collections::VecDeque<ChatSignature>) {
        let new: std::collections::HashSet<ChatSignature> = queue.iter().cloned().collect();
        for index in 0..self.entries.len() {
            if queue.is_empty() {
                break;
            }
            let old = self.entries[index].take();
            self.entries[index] = queue.pop_back();
            if let Some(old) = old
                && !new.contains(&old)
            {
                queue.push_front(old);
            }
        }
    }
}

impl FilterMask {
    /// Whether character `index` is hidden.
    pub fn hides(&self, index: usize) -> bool {
        match self {
            Self::Partial(words) => words
                .get(index / 64)
                .is_some_and(|word| word >> (index % 64) & 1 == 1),
            _ => false,
        }
    }
}
