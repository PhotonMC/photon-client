use super::*;

#[test]
fn system_chat_parser_matches_translatable_overlay_component() {
    let mut payload = Vec::new();
    photon_core::nbt::write_nbt(
        Some(&[photon_core::nbt::Field {
            key: "translate".to_owned(),
            value: photon_core::nbt::FieldValue::Str("build.spawn_protection".to_owned()),
        }]),
        &mut payload,
    );
    payload.push(1);

    assert_eq!(
        parse_system_chat_packet(&payload).expect("system chat packet"),
        super::super::SystemChatObservation {
            overlay: true,
            translation_key: Some("build.spawn_protection".to_owned()),
        }
    );
}

#[test]
fn the_signature_cache_pushes_new_entries_to_the_front() {
    let sig = |byte: u8| vec![byte; 256];
    let mut cache = super::super::SignatureCache::default();
    cache.push([sig(1), sig(2)].into());
    // The newest is at place 0, the one before it at 1.
    assert_eq!(
        cache.unpack(&super::super::PackedSignature::Cached(0)),
        Some(sig(2))
    );
    assert_eq!(
        cache.unpack(&super::super::PackedSignature::Cached(1)),
        Some(sig(1))
    );
    // One more shifts the old ones back; one it repeats is not kept twice.
    cache.push([sig(2), sig(3)].into());
    assert_eq!(
        cache.unpack(&super::super::PackedSignature::Cached(0)),
        Some(sig(3))
    );
    assert_eq!(
        cache.unpack(&super::super::PackedSignature::Cached(1)),
        Some(sig(2))
    );
    assert_eq!(
        cache.unpack(&super::super::PackedSignature::Cached(2)),
        Some(sig(1))
    );
    assert_eq!(
        cache.unpack(&super::super::PackedSignature::Cached(3)),
        None
    );
}

#[test]
fn a_partial_filter_mask_hides_its_set_bits() {
    let mut payload = vec![0];
    payload.extend([0; 16]);
    payload.extend([0, 0, 2, b'h', b'i']);
    payload.extend([0; 16]);
    payload.extend([0, 0, 2, 1]);
    payload.extend([0, 0, 0, 0, 0, 0, 0, 0b10]); // the second character
    payload.push(1);
    payload.extend([8, 0, 3, b'B', b'o', b'b', 0]);
    let packet = super::super::parse_player_chat_packet(&payload).unwrap();
    assert!(!packet.filter.hides(0) && packet.filter.hides(1));
}

#[test]
fn an_unsigned_chat_packet_ends_in_an_empty_acknowledgement() {
    let mut output = Vec::new();
    super::super::encode_chat("hi", (5, 7), (0, 0, 1), &mut output);
    assert_eq!(output[..3], [2, b'h', b'i']);
    // offset 0, three bytes of bits, checksum 1, after the signature flag.
    assert_eq!(output[3 + 16..], [0, 0, 0, 0, 0, 1]);
}

#[test]
fn a_chat_packet_carries_the_offset_the_bits_and_the_checksum() {
    let mut output = Vec::new();
    super::super::encode_chat("hi", (5, 7), (70, 0x0A_0003, 0xE5), &mut output);
    // The signature flag, the offset as a varint, the 20 bits little endian, the checksum.
    assert_eq!(output[3 + 16..], [0, 70, 0x03, 0x00, 0x0A, 0xE5]);
}

#[test]
fn player_chat_reads_the_text_the_type_and_the_sender() {
    let mut payload = vec![0]; // global index
    payload.extend([0; 16]); // sender
    payload.push(0); // index
    payload.push(0); // no signature
    payload.extend([2, b'h', b'i']); // content
    payload.extend([0; 16]); // timestamp and salt
    payload.push(0); // last seen
    payload.push(0); // no unsigned content
    payload.push(0); // filter mask: pass through
    payload.push(1); // chat type: registry id 0
    payload.extend([8, 0, 3, b'B', b'o', b'b']); // the sender's name
    payload.push(0); // no target
    let packet = super::super::parse_player_chat_packet(&payload).unwrap();
    assert_eq!(packet.global_index, 0);
    assert_eq!(packet.signed_text, "hi");
    assert_eq!(packet.signature, None);
    assert_eq!(packet.filter, super::super::FilterMask::PassThrough);
    assert_eq!(packet.bound.holder, super::super::ChatHolder::Registry(0));
    assert_eq!(
        packet.bound.name,
        photon_game::text::Component::literal("Bob")
    );
}
