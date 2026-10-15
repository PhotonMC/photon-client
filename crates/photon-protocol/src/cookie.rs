//! Cookies, custom payloads and custom queries: the packets a server uses to keep a few bytes on
//! the client and to talk past the protocol, in the login, configuration and play phases.

use photon_core::identifier::{self, Identifier};
use photon_core::{binary, utf8, wire};

pub type ReadError = binary::ReadError;

/// `ClientboundStoreCookiePacket.MAX_PAYLOAD_SIZE`.
pub const MAX_COOKIE_PAYLOAD: usize = 5120;

/// `ClientboundCustomPayloadPacket.MAX_PAYLOAD_SIZE`: what is after the identifier.
pub const MAX_CUSTOM_PAYLOAD: usize = 1_048_576;

/// `ClientboundCustomReportDetailsPacket`'s limits.
const MAX_DETAIL_KEY_LENGTH: usize = 128;
const MAX_DETAIL_VALUE_LENGTH: usize = 4096;
const MAX_DETAIL_COUNT: usize = 32;

fn read_bytes(input: &mut &[u8], count: usize) -> Result<Vec<u8>, ReadError> {
    if input.len() < count {
        return Err(ReadError::BufferUnderflow);
    }
    let (bytes, rest) = input.split_at(count);
    *input = rest;
    Ok(bytes.to_vec())
}

/// `ByteBufCodecs.byteArray(max)`.
fn read_byte_array(input: &mut &[u8], max: usize) -> Result<Vec<u8>, ReadError> {
    let length = wire::read_varint(input).map_err(ReadError::from)?;
    let length = usize::try_from(length).map_err(|_| ReadError::DecoderError)?;
    if length > max {
        return Err(ReadError::DecoderError);
    }
    read_bytes(input, length)
}

fn read_string(input: &mut &[u8], max: usize) -> Result<String, ReadError> {
    let units = utf8::read_utf8(input, max).map_err(ReadError::from)?;
    String::from_utf16(&units).map_err(|_| ReadError::DecoderError)
}

/// `ClientboundCookieRequestPacket`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCookieRequestPacket {
    pub key: Identifier,
}

impl ClientboundCookieRequestPacket {
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        Ok(Self {
            key: identifier::read_identifier(input)?,
        })
    }
}

/// `ServerboundCookieResponsePacket`: the cookie's bytes, or nothing where there is none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerboundCookieResponsePacket {
    pub key: Identifier,
    pub payload: Option<Vec<u8>>,
}

impl ServerboundCookieResponsePacket {
    pub fn encode(&self, output: &mut Vec<u8>) -> Result<(), utf8::WriteError> {
        identifier::write_identifier(&self.key, output)?;
        match &self.payload {
            Some(payload) => {
                output.push(1);
                wire::write_varint(payload.len() as i32, output);
                output.extend_from_slice(payload);
            }
            None => output.push(0),
        }
        Ok(())
    }
}

/// `ClientboundStoreCookiePacket`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundStoreCookiePacket {
    pub key: Identifier,
    pub payload: Vec<u8>,
}

impl ClientboundStoreCookiePacket {
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        Ok(Self {
            key: identifier::read_identifier(input)?,
            payload: read_byte_array(input, MAX_COOKIE_PAYLOAD)?,
        })
    }
}

/// What the server asked the client to keep (`CommonListenerCookie.serverCookies`), by key.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CookieStore(std::collections::BTreeMap<String, Vec<u8>>);

impl CookieStore {
    pub fn store(&mut self, key: &Identifier, payload: Vec<u8>) {
        self.0.insert(key.as_string(), payload);
    }

    pub fn get(&self, key: &Identifier) -> Option<&Vec<u8>> {
        self.0.get(&key.as_string())
    }

    /// `handleRequestCookie`: the answer to a request.
    pub fn answer(
        &self,
        request: &ClientboundCookieRequestPacket,
    ) -> ServerboundCookieResponsePacket {
        ServerboundCookieResponsePacket {
            key: request.key.clone(),
            payload: self.get(&request.key).cloned(),
        }
    }
}

/// `ClientboundCustomPayloadPacket`: an identifier and what the server wants to say under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCustomPayloadPacket {
    pub id: Identifier,
    pub data: Vec<u8>,
}

impl ClientboundCustomPayloadPacket {
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        let id = identifier::read_identifier(input)?;
        if input.len() > MAX_CUSTOM_PAYLOAD {
            return Err(ReadError::DecoderError);
        }
        Ok(Self {
            id,
            data: input.to_vec(),
        })
    }

    /// The server's brand (`minecraft:brand`: a string), if this is that.
    pub fn brand(&self) -> Option<String> {
        if self.id.namespace() != "minecraft" || self.id.path() != "brand" {
            return None;
        }
        read_string(&mut self.data.as_slice(), 32_767).ok()
    }
}

/// `ClientboundCustomQueryPacket` (login phase): a question the client may answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundCustomQueryPacket {
    pub transaction_id: i32,
    pub id: Identifier,
}

impl ClientboundCustomQueryPacket {
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        let transaction_id = wire::read_varint(input).map_err(ReadError::from)?;
        let id = identifier::read_identifier(input)?;
        Ok(Self { transaction_id, id })
    }
}

/// `ServerboundCustomQueryAnswerPacket` with no answer: what `handleCustomQuery` sends.
pub fn encode_unanswered_query(transaction_id: i32, output: &mut Vec<u8>) {
    wire::write_varint(transaction_id, output);
    output.push(0);
}

/// `ClientboundTransferPacket`: the server sending the client on to another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientboundTransferPacket {
    pub host: String,
    pub port: i32,
}

impl ClientboundTransferPacket {
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        let host = read_string(input, 32_767)?;
        let port = wire::read_varint(input).map_err(ReadError::from)?;
        Ok(Self { host, port })
    }
}

/// `ClientboundCustomReportDetailsPacket`: what the server wants a crash report to carry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientboundCustomReportDetailsPacket {
    pub details: Vec<(String, String)>,
}

impl ClientboundCustomReportDetailsPacket {
    pub fn decode(input: &mut &[u8]) -> Result<Self, ReadError> {
        let count = wire::read_varint(input).map_err(ReadError::from)?;
        let count = usize::try_from(count).map_err(|_| ReadError::DecoderError)?;
        if count > MAX_DETAIL_COUNT {
            return Err(ReadError::DecoderError);
        }
        let mut details = Vec::with_capacity(count);
        for _ in 0..count {
            let key = read_string(input, MAX_DETAIL_KEY_LENGTH)?;
            let value = read_string(input, MAX_DETAIL_VALUE_LENGTH)?;
            details.push((key, value));
        }
        Ok(Self { details })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(text: &str) -> Identifier {
        Identifier::parse(text).expect("identifier")
    }

    fn identifier_bytes(text: &str) -> Vec<u8> {
        let mut out = Vec::new();
        identifier::write_identifier(&key(text), &mut out).expect("write");
        out
    }

    #[test]
    fn a_requested_cookie_is_answered_with_what_was_stored_or_nothing() {
        let mut stored = identifier_bytes("example:token");
        wire::write_varint(3, &mut stored);
        stored.extend_from_slice(b"abc");
        let packet = ClientboundStoreCookiePacket::decode(&mut stored.as_slice()).expect("store");
        let mut cookies = CookieStore::default();
        cookies.store(&packet.key, packet.payload);
        let request = ClientboundCookieRequestPacket {
            key: key("example:token"),
        };
        let mut answer = Vec::new();
        cookies
            .answer(&request)
            .encode(&mut answer)
            .expect("encode");
        let mut expected = identifier_bytes("example:token");
        expected.extend_from_slice(&[1, 3, b'a', b'b', b'c']);
        assert_eq!(answer, expected);
        let missing = ClientboundCookieRequestPacket {
            key: key("example:other"),
        };
        let mut empty = Vec::new();
        cookies.answer(&missing).encode(&mut empty).expect("encode");
        let mut expected = identifier_bytes("example:other");
        expected.push(0);
        assert_eq!(empty, expected);
    }

    #[test]
    fn a_cookie_over_the_limit_is_refused() {
        let mut bytes = identifier_bytes("example:big");
        wire::write_varint(MAX_COOKIE_PAYLOAD as i32 + 1, &mut bytes);
        bytes.resize(bytes.len() + MAX_COOKIE_PAYLOAD + 1, 0);
        assert!(ClientboundStoreCookiePacket::decode(&mut bytes.as_slice()).is_err());
    }

    #[test]
    fn the_brand_is_the_string_under_its_identifier() {
        let mut bytes = identifier_bytes("minecraft:brand");
        let mut text = Vec::new();
        utf8::write_utf8(
            &mut text,
            &"vanilla".encode_utf16().collect::<Vec<_>>(),
            32_767,
        )
        .expect("string");
        bytes.extend_from_slice(&text);
        let packet =
            ClientboundCustomPayloadPacket::decode(&mut bytes.as_slice()).expect("payload");
        assert_eq!(packet.brand().as_deref(), Some("vanilla"));
        let other = ClientboundCustomPayloadPacket {
            id: key("example:thing"),
            data: text,
        };
        assert_eq!(other.brand(), None);
    }

    #[test]
    fn a_transfer_names_a_host_and_a_port() {
        let mut bytes = Vec::new();
        utf8::write_utf8(
            &mut bytes,
            &"play.example.org".encode_utf16().collect::<Vec<_>>(),
            32_767,
        )
        .expect("host");
        wire::write_varint(25_566, &mut bytes);
        let packet = ClientboundTransferPacket::decode(&mut bytes.as_slice()).expect("transfer");
        assert_eq!(packet.host, "play.example.org");
        assert_eq!(packet.port, 25_566);
    }

    #[test]
    fn an_unanswered_query_says_nothing_under_its_number() {
        let mut out = Vec::new();
        encode_unanswered_query(300, &mut out);
        assert_eq!(out, [0xAC, 0x02, 0]);
    }

    #[test]
    fn report_details_are_a_bounded_list_of_pairs() {
        let mut bytes = Vec::new();
        wire::write_varint(1, &mut bytes);
        for text in ["server", "alpha"] {
            utf8::write_utf8(&mut bytes, &text.encode_utf16().collect::<Vec<_>>(), 4096)
                .expect("string");
        }
        let packet =
            ClientboundCustomReportDetailsPacket::decode(&mut bytes.as_slice()).expect("details");
        assert_eq!(packet.details, [("server".to_owned(), "alpha".to_owned())]);
        let mut too_many = Vec::new();
        wire::write_varint(33, &mut too_many);
        assert!(ClientboundCustomReportDetailsPacket::decode(&mut too_many.as_slice()).is_err());
    }
}
