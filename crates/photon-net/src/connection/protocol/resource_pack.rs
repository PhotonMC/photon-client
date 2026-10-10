//! The server resource pack packets (`ClientboundResourcePackPushPacket`, `ClientboundResourcePackPopPacket`
//! and the client's `ServerboundResourcePackPacket`).

use photon_core::binary::{self, Uuid};

use super::chat::{read_chat_component, read_chat_string};
use super::reader::read_bool;
use super::*;

/// `ClientboundResourcePackPushPacket.MAX_HASH_LENGTH`.
const MAX_HASH_LENGTH: usize = 40;

/// A pack the server asks the client to load.
#[derive(Debug, Clone, PartialEq)]
pub struct PackRequest {
    pub id: Uuid,
    pub url: String,
    pub hash: String,
    pub required: bool,
    pub prompt: Option<photon_game::text::Component>,
}

impl PackRequest {
    /// `parseResourcePackUrl`: only `http` and `https` addresses are fetched (`new URL` takes
    /// anything with a known scheme, which it lowers).
    pub fn has_valid_url(&self) -> bool {
        self.url.split_once(':').is_some_and(|(scheme, _)| {
            scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
        })
    }
}

/// `ServerboundResourcePackPacket.Action`, in the order the server numbers it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackAction {
    SuccessfullyLoaded = 0,
    Declined = 1,
    FailedDownload = 2,
    Accepted = 3,
    Downloaded = 4,
    InvalidUrl = 5,
    FailedReload = 6,
    Discarded = 7,
}

/// `ClientboundResourcePackPushPacket.STREAM_CODEC`.
pub(crate) fn parse_resource_pack_push(input: &[u8]) -> Result<PackRequest, ConnectionError> {
    let mut input = input;
    let id = binary::read_uuid(&mut input).map_err(|_| ConnectionError::Codec("pack id"))?;
    let url = read_chat_string(&mut input, 32_767, "pack url")?;
    let hash = read_chat_string(&mut input, MAX_HASH_LENGTH, "pack hash")?;
    let required = read_bool(&mut input, "pack required")?;
    let prompt = if read_bool(&mut input, "pack prompt present")? {
        Some(read_chat_component(&mut input, "pack prompt")?)
    } else {
        None
    };
    Ok(PackRequest {
        id,
        url,
        hash,
        required,
        prompt,
    })
}

/// `ClientboundResourcePackPopPacket`: one pack, or all of them.
pub(crate) fn parse_resource_pack_pop(input: &[u8]) -> Result<Option<Uuid>, ConnectionError> {
    let mut input = input;
    if read_bool(&mut input, "pack pop present")? {
        Ok(Some(
            binary::read_uuid(&mut input).map_err(|_| ConnectionError::Codec("pack pop id"))?,
        ))
    } else {
        Ok(None)
    }
}

/// `ServerboundResourcePackPacket`.
pub(crate) fn encode_resource_pack(id: Uuid, action: PackAction, output: &mut Vec<u8>) {
    binary::write_uuid(id, output);
    photon_core::wire::write_varint(action as i32, output);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(out: &mut Vec<u8>, value: &str) {
        photon_core::utf8::write_utf8(out, &value.encode_utf16().collect::<Vec<_>>(), 32_767)
            .expect("string");
    }

    #[test]
    fn a_pack_push_without_a_prompt_is_read() {
        let mut bytes = Vec::new();
        binary::write_uuid(Uuid::new(1, 2), &mut bytes);
        text(&mut bytes, "https://example.com/pack.zip");
        text(&mut bytes, "0123456789abcdef0123456789abcdef01234567");
        bytes.extend_from_slice(&[1, 0]);
        let request = parse_resource_pack_push(&bytes).expect("push");
        assert_eq!(request.id, Uuid::new(1, 2));
        assert_eq!(request.url, "https://example.com/pack.zip");
        assert!(request.required);
        assert!(request.prompt.is_none());
        assert!(request.has_valid_url());
    }

    #[test]
    fn only_web_addresses_are_valid() {
        let request = |url: &str| PackRequest {
            id: Uuid::new(0, 0),
            url: url.to_owned(),
            hash: String::new(),
            required: false,
            prompt: None,
        };
        assert!(request("http://example.com/a.zip").has_valid_url());
        assert!(!request("file:///etc/passwd").has_valid_url());
        assert!(!request("ftp://example.com/a.zip").has_valid_url());
        assert!(request("HTTPS://example.com/a.zip").has_valid_url());
        assert!(!request("not a url").has_valid_url());
    }

    #[test]
    fn a_pop_names_one_pack_or_all() {
        assert_eq!(parse_resource_pack_pop(&[0]).expect("all"), None);
        let mut one = vec![1];
        binary::write_uuid(Uuid::new(5, 6), &mut one);
        assert_eq!(
            parse_resource_pack_pop(&one).expect("one"),
            Some(Uuid::new(5, 6))
        );
    }

    #[test]
    fn the_answer_is_the_pack_and_the_action_number() {
        let mut out = Vec::new();
        encode_resource_pack(Uuid::new(1, 2), PackAction::Accepted, &mut out);
        assert_eq!(out.len(), 17);
        assert_eq!(out[16], 3);
    }
}
