use super::*;

#[test]
fn inventory_component_hash_prediction_encoder_keeps_registry_hash_fields() {
    let mut payload = Vec::new();
    encode_inventory_pickup_with_component_hash_prediction(&mut payload, 2, 1, 1_046)
        .expect("component-hash inventory prediction packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        2
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        1
    );
    assert_eq!(
        i16::from_be_bytes(input[..2].try_into().expect("changed slot")),
        36
    );
    input = &input[2..];
    assert_eq!(input[0], 0, "changed slot should be empty");
    input = &input[1..];
    assert_eq!(input[0], 1, "carried should be non-empty");
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item registry id"),
        1_046
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item count"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("component map count"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("component type id"),
        17
    );
    assert_eq!(
        i32::from_be_bytes(input[..4].try_into().expect("component hash")),
        0x0102_0304
    );
    input = &input[4..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("removed component count"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("removed component type id"),
        17
    );
    assert!(input.is_empty());
}

#[test]
fn inventory_item_component_conformance_encoder_uses_table_rows() {
    for spec in VANILLA_ITEM_COMPONENT_CONFORMANCE {
        let carried_ids: Vec<_> = spec
            .carried_components
            .iter()
            .map(|(component_type_id, _)| *component_type_id)
            .collect();
        assert_eq!(
            carried_ids.as_slice(),
            spec.expected_positive_component_type_ids,
            "table component ids for {}",
            spec.component_path
        );
        assert_eq!(
            spec.carried_components
                .iter()
                .find(|(component_type_id, _)| *component_type_id == spec.component_type_id)
                .map(|(_, hash)| *hash),
            Some(spec.component_hash),
            "table component hash for {}",
            spec.component_path
        );

        let mut payload = Vec::new();
        encode_inventory_pickup_with_component_spec_prediction(&mut payload, 2, spec)
            .expect("table component prediction packet");
        let mut input = payload.as_slice();
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("container id"),
            0
        );
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("state id"),
            2
        );
        assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
        input = &input[2..];
        assert_eq!(input[0], 0, "button");
        input = &input[1..];
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("container input"),
            0
        );
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
            1
        );
        assert_eq!(
            i16::from_be_bytes(input[..2].try_into().expect("changed slot")),
            36
        );
        input = &input[2..];
        assert_eq!(input[0], 0, "changed slot should be empty");
        input = &input[1..];
        assert_eq!(input[0], 1, "carried should be non-empty");
        input = &input[1..];
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("item registry id"),
            spec.item_registry_id
        );
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("item count"),
            1
        );
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("component map count"),
            spec.carried_components.len() as i32
        );
        for (expected_type_id, expected_hash) in spec.carried_components {
            assert_eq!(
                voxel_core::wire::read_varint(&mut input).expect("component type id"),
                *expected_type_id,
                "component type for {}",
                spec.component_path
            );
            assert_eq!(
                i32::from_be_bytes(input[..4].try_into().expect("component hash")),
                *expected_hash,
                "component hash for {}",
                spec.component_path
            );
            input = &input[4..];
        }
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("removed component count"),
            0
        );
        assert!(
            input.is_empty(),
            "trailing bytes for {}",
            spec.component_path
        );
    }
}

#[test]
fn inventory_item_component_conformance_parser_consumes_table_codecs() {
    for spec in VANILLA_ITEM_COMPONENT_CONFORMANCE {
        let mut payload = Vec::new();
        voxel_core::wire::write_varint(1, &mut payload);
        voxel_core::wire::write_varint(spec.item_registry_id, &mut payload);
        voxel_core::wire::write_varint(1, &mut payload);
        voxel_core::wire::write_varint(0, &mut payload);
        voxel_core::wire::write_varint(spec.component_type_id, &mut payload);
        write_component_fixture(spec.codec, &mut payload);

        let mut input = payload.as_slice();
        let observation = parse_optional_item_stack(&mut input, "table component item stack")
            .unwrap_or_else(|error| {
                panic!(
                    "table component item stack {}: {error:?}",
                    spec.component_path
                )
            });
        assert_eq!(
            observation.item_count,
            Some(1),
            "item count for {}",
            spec.component_path
        );
        assert_eq!(
            observation.item_registry_id,
            Some(spec.item_registry_id),
            "item registry id for {}",
            spec.component_path
        );
        assert_eq!(
            observation.positive_component_type_ids,
            vec![spec.component_type_id],
            "component type for {}",
            spec.component_path
        );
        assert!(observation.negative_component_type_ids.is_empty());
        assert!(
            input.is_empty(),
            "trailing bytes for {}",
            spec.component_path
        );
    }
}

#[test]
fn inventory_partial_changed_slot_prediction_encoder_omits_slot_hash() {
    let mut payload = Vec::new();
    encode_inventory_pickup_with_partial_changed_slot_prediction(&mut payload, 2, 8, 1)
        .expect("partial changed-slot prediction packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        2
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input[0], 1, "carried should be non-empty");
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item id"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item count"),
        8
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("component map count"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("removed component count"),
        0
    );
    assert!(input.is_empty());
}
