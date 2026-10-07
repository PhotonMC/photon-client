use super::*;

#[test]
fn chunk_packet_parser_consumes_empty_java_payload() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&(-17_i32).to_be_bytes());
    payload.extend_from_slice(&23_i32.to_be_bytes());
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);
    for _ in 0..6 {
        voxel_core::wire::write_varint(0, &mut payload);
    }

    let observation = parse_chunk_packet(&payload).expect("empty chunk payload");
    assert_eq!(observation.x, -17);
    assert_eq!(observation.z, 23);
    assert_eq!(observation.heightmap_count, 0);
    assert!(observation.heightmap_observations.is_empty());
    assert_eq!(observation.section_data_length, 0);
    assert_eq!(observation.block_entity_count, 0);
    assert_eq!(observation.sky_mask_word_count, 0);
    assert_eq!(observation.block_mask_word_count, 0);
    assert_eq!(observation.empty_sky_mask_word_count, 0);
    assert_eq!(observation.empty_block_mask_word_count, 0);
    assert_eq!(observation.sky_update_count, 0);
    assert!(observation.sky_update_lengths.is_empty());
    assert_eq!(observation.block_update_count, 0);
    assert!(observation.block_update_lengths.is_empty());
}

#[test]
fn chunk_packet_parser_consumes_nullable_block_entity_nbt() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&0_i32.to_be_bytes());
    payload.extend_from_slice(&0_i32.to_be_bytes());
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);
    payload.push(0x21);
    payload.extend_from_slice(&(-64_i16).to_be_bytes());
    voxel_core::wire::write_varint(0, &mut payload);
    payload.push(0);
    for _ in 0..6 {
        voxel_core::wire::write_varint(0, &mut payload);
    }

    let observation = parse_chunk_packet(&payload).expect("nullable block entity tag");
    assert_eq!(observation.block_entity_count, 1);
    assert_eq!(
        observation.block_entities,
        [super::super::BlockEntityObservation {
            x: 2,
            y: -64,
            z: 1,
            type_id: 0,
            tag_length: 0,
            applied_to_world: false,
            tag: None,
        }]
    );
}

#[test]
fn chunk_packet_parser_records_heightmap_and_light_lengths() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&4_i32.to_be_bytes());
    payload.extend_from_slice(&(-9_i32).to_be_bytes());

    voxel_core::wire::write_varint(2, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(2, &mut payload);
    payload.extend_from_slice(&[0; 16]);
    voxel_core::wire::write_varint(4, &mut payload);
    voxel_core::wire::write_varint(3, &mut payload);
    payload.extend_from_slice(&[0; 24]);

    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);
    for _ in 0..4 {
        voxel_core::wire::write_varint(0, &mut payload);
    }

    voxel_core::wire::write_varint(2, &mut payload);
    voxel_core::wire::write_varint(2_048, &mut payload);
    payload.extend_from_slice(&[0xa5; 2_048]);
    voxel_core::wire::write_varint(3, &mut payload);
    payload.extend_from_slice(&[0x5a; 3]);

    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(2_048, &mut payload);
    payload.extend_from_slice(&[0x3c; 2_048]);

    let observation = parse_chunk_packet(&payload).expect("heightmap and light lengths");
    assert_eq!(observation.heightmap_count, 2);
    assert_eq!(
        observation.heightmap_observations,
        [
            super::super::HeightmapObservation {
                type_id: 1,
                word_count: 2,
            },
            super::super::HeightmapObservation {
                type_id: 4,
                word_count: 3,
            },
        ]
    );
    assert_eq!(observation.sky_update_count, 2);
    assert_eq!(observation.sky_update_lengths, [2_048, 3]);
    assert_eq!(observation.block_update_count, 1);
    assert_eq!(observation.block_update_lengths, [2_048]);
}

#[test]
fn forget_level_chunk_parser_matches_java_chunk_pos_packing() {
    let packed = ((2_u64 << 32) | u64::from(u32::MAX)) as i64;

    assert_eq!(
        parse_forget_level_chunk_packet(&packed.to_be_bytes()).expect("chunk position"),
        (-1, 2)
    );
    assert!(parse_forget_level_chunk_packet(&packed.to_be_bytes()[..7]).is_err());

    let mut trailing = packed.to_be_bytes().to_vec();
    trailing.push(0);
    assert!(parse_forget_level_chunk_packet(&trailing).is_err());
}

#[test]
fn chunk_cache_center_and_radius_parsers_match_varint_payloads() {
    let mut center = Vec::new();
    voxel_core::wire::write_varint(-12, &mut center);
    voxel_core::wire::write_varint(37, &mut center);
    assert_eq!(
        parse_set_chunk_cache_center_packet(&center).expect("chunk cache center"),
        (-12, 37)
    );

    let mut radius = Vec::new();
    voxel_core::wire::write_varint(6, &mut radius);
    assert_eq!(
        parse_set_chunk_cache_radius_packet(&radius).expect("chunk cache radius"),
        6
    );

    radius.push(0);
    assert!(parse_set_chunk_cache_radius_packet(&radius).is_err());
}

#[test]
fn light_update_parser_matches_java_masks_and_data_layer_lengths() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(-3, &mut payload);
    voxel_core::wire::write_varint(7, &mut payload);
    voxel_core::binary::write_bitset(
        &voxel_core::binary::BitSet::from_bits(&[0, 2]),
        &mut payload,
    );
    voxel_core::binary::write_bitset(&voxel_core::binary::BitSet::from_bits(&[1]), &mut payload);
    voxel_core::binary::write_bitset(&voxel_core::binary::BitSet::from_bits(&[3]), &mut payload);
    voxel_core::binary::write_bitset(&voxel_core::binary::BitSet::from_bits(&[4]), &mut payload);
    voxel_core::wire::write_varint(2, &mut payload);
    voxel_core::wire::write_varint(2_048, &mut payload);
    payload.extend_from_slice(&[0x12; 2_048]);
    voxel_core::wire::write_varint(2_048, &mut payload);
    payload.extend_from_slice(&[0x34; 2_048]);
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(2_048, &mut payload);
    payload.extend_from_slice(&[0x56; 2_048]);

    let observation = parse_light_update_packet(&payload).expect("light update");
    assert_eq!((observation.x, observation.z), (-3, 7));
    assert_eq!(observation.sky_mask_word_count, 1);
    assert_eq!(observation.block_mask_word_count, 1);
    assert_eq!(observation.empty_sky_mask_word_count, 1);
    assert_eq!(observation.empty_block_mask_word_count, 1);
    assert_eq!(observation.sky_update_lengths, [2_048, 2_048]);
    assert_eq!(observation.block_update_lengths, [2_048]);
    assert_eq!(observation.applied_sky_sections, 0);
    assert_eq!(observation.applied_block_sections, 0);

    let mut trailing = payload;
    trailing.push(0);
    assert!(parse_light_update_packet(&trailing).is_err());
}

#[test]
fn chunk_packet_parser_rejects_trailing_payload() {
    let mut payload = Vec::new();
    payload.extend_from_slice(&0_i32.to_be_bytes());
    payload.extend_from_slice(&0_i32.to_be_bytes());
    for _ in 0..9 {
        voxel_core::wire::write_varint(0, &mut payload);
    }
    payload.push(0xff);

    match parse_chunk_packet(&payload) {
        Err(super::super::ConnectionError::Codec(name)) => {
            assert_eq!(name, "chunk trailing payload")
        }
        result => panic!("unexpected chunk parser result: {result:?}"),
    }
}

#[test]
fn section_parser_matches_java_palette_storage_layout() {
    let mut section = Vec::new();
    section.extend_from_slice(&7_i16.to_be_bytes());
    section.extend_from_slice(&2_i16.to_be_bytes());

    section.push(4);
    voxel_core::wire::write_varint(2, &mut section);
    voxel_core::wire::write_varint(0, &mut section);
    voxel_core::wire::write_varint(3, &mut section);
    section.extend_from_slice(&0x101_u64.to_be_bytes());
    section.resize(section.len() + 255 * 8, 0);

    section.push(3);
    voxel_core::wire::write_varint(2, &mut section);
    voxel_core::wire::write_varint(0, &mut section);
    voxel_core::wire::write_varint(1, &mut section);
    section.extend_from_slice(&0x249_u64.to_be_bytes());
    section.resize(section.len() + 3 * 8, 0);

    let observations = parse_section_data(&section).expect("serialized section");
    assert_eq!(observations.len(), 1);
    let observation = &observations[0];
    assert_eq!(observation.non_empty_block_count, 7);
    assert_eq!(observation.fluid_count, 2);
    assert_eq!(observation.block_state_bits_per_entry, 4);
    assert_eq!(observation.block_state_palette_entry_count, 2);
    assert_eq!(observation.block_state_palette_ids, [0, 3]);
    assert_eq!(observation.block_state_storage_word_count, 256);
    assert_eq!(observation.block_state_first_values, [3, 0, 3, 0]);
    assert_eq!(observation.block_state_values.len(), 4_096);
    assert_eq!(&observation.block_state_values[..4], [3, 0, 3, 0]);
    assert_eq!(observation.block_state_distinct_value_count, 2);
    assert_eq!(observation.biome_bits_per_entry, 3);
    assert_eq!(observation.biome_palette_entry_count, 2);
    assert_eq!(observation.biome_palette_ids, [0, 1]);
    assert_eq!(observation.biome_storage_word_count, 4);
    assert_eq!(observation.biome_first_values, [1, 1, 1, 1]);
    assert_eq!(observation.biome_values.len(), 64);
    assert_eq!(&observation.biome_values[..4], [1, 1, 1, 1]);
    assert_eq!(observation.biome_distinct_value_count, 2);
}

#[test]
fn section_parser_reads_zero_bit_palettes_without_storage() {
    let mut section = Vec::new();
    section.extend_from_slice(&4096_i16.to_be_bytes());
    section.extend_from_slice(&0_i16.to_be_bytes());
    section.push(0);
    voxel_core::wire::write_varint(0, &mut section);
    section.push(0);
    voxel_core::wire::write_varint(0, &mut section);

    let observations = parse_section_data(&section).expect("single-valued section");
    assert_eq!(observations.len(), 1);
    assert_eq!(observations[0].block_state_bits_per_entry, 0);
    assert_eq!(observations[0].block_state_palette_entry_count, 1);
    assert_eq!(observations[0].block_state_palette_ids, [0]);
    assert_eq!(observations[0].block_state_storage_word_count, 0);
    assert_eq!(observations[0].block_state_first_values, [0, 0, 0, 0]);
    assert_eq!(observations[0].block_state_values, vec![0; 4_096]);
    assert_eq!(observations[0].block_state_distinct_value_count, 1);
    assert_eq!(observations[0].biome_bits_per_entry, 0);
    assert_eq!(observations[0].biome_palette_entry_count, 1);
    assert_eq!(observations[0].biome_palette_ids, [0]);
    assert_eq!(observations[0].biome_storage_word_count, 0);
    assert_eq!(observations[0].biome_first_values, [0, 0, 0, 0]);
    assert_eq!(observations[0].biome_values, vec![0; 64]);
    assert_eq!(observations[0].biome_distinct_value_count, 1);
}
