//! `ClientboundServerLinksPacket`, which the configuration and the play phase share.

use photon_game::server_links::{LinkType, UntrustedEntry};

use crate::play::ConnectionError;

/// `ServerLinks.UNTRUSTED_LINKS_STREAM_CODEC`: a list of entries, each a type (a known one by id, or
/// a component) and an address.
pub fn parse_server_links(input: &[u8]) -> Result<Vec<UntrustedEntry>, ConnectionError> {
    let mut input = input;
    let codec = |name| ConnectionError::Codec(name);
    let count = photon_core::wire::read_varint(&mut input).map_err(|_| codec("server links"))?;
    let count = usize::try_from(count).map_err(|_| codec("server links"))?;
    let mut entries = Vec::new();
    for _ in 0..count {
        let known = match input.split_first() {
            Some((flag, rest)) => {
                input = rest;
                *flag != 0
            }
            None => return Err(codec("server link type")),
        };
        let link_type = if known {
            let id = photon_core::wire::read_varint(&mut input)
                .map_err(|_| codec("server link type"))?;
            LinkType::known(id)
        } else {
            let component = photon_core::nbt::read_any_nbt(&mut input)
                .ok()
                .flatten()
                .and_then(|tag| photon_game::text::Component::from_nbt(&tag))
                .ok_or_else(|| codec("server link name"))?;
            LinkType::Custom(component)
        };
        let units =
            photon_core::utf8::read_utf8(&mut input, 32767).map_err(|_| codec("server link"))?;
        entries.push(UntrustedEntry {
            link_type,
            link: String::from_utf16_lossy(&units),
        });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string(text: &str, out: &mut Vec<u8>) {
        let units: Vec<u16> = text.encode_utf16().collect();
        photon_core::utf8::write_utf8(out, &units, 32767).expect("string");
    }

    #[test]
    fn a_known_type_and_a_component_entry_are_read() {
        let mut bytes = Vec::new();
        photon_core::wire::write_varint(2, &mut bytes);
        bytes.push(1);
        photon_core::wire::write_varint(6, &mut bytes);
        string("https://example.com", &mut bytes);
        // A component: the flag, then a string tag (type 8) as the network NBT root, "Wiki".
        bytes.push(0);
        bytes.extend([0x08, 0x00, 0x04]);
        bytes.extend(b"Wiki");
        string("https://wiki.example", &mut bytes);
        let entries = parse_server_links(&bytes).expect("links");
        assert_eq!(entries[0].link_type, LinkType::known(6));
        assert_eq!(entries[0].link, "https://example.com");
        assert_eq!(
            entries[1].link_type,
            LinkType::Custom(photon_game::text::Component::literal("Wiki"))
        );
        assert_eq!(entries[1].link, "https://wiki.example");
    }
}
