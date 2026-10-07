use super::*;

#[test]
fn container_slot_parser_matches_empty_optional_item_stack() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(7, &mut payload);
    payload.extend_from_slice(&36_i16.to_be_bytes());
    voxel_core::wire::write_varint(0, &mut payload);

    assert_eq!(
        parse_container_set_slot_packet(&payload).expect("container slot packet"),
        super::super::ContainerSetSlotObservation {
            container_id: 0,
            state_id: 7,
            slot: 36,
            item_count: None,
            item_registry_id: None,
            positive_component_type_ids: Vec::new(),
            negative_component_type_ids: Vec::new(),
            item: None,
            applied_to_world: false,
        }
    );
}

#[test]
fn container_slot_parser_skips_custom_model_data_component() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(8, &mut payload);
    payload.extend_from_slice(&36_i16.to_be_bytes());
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(1_046, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(17, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);
    payload.extend_from_slice(&1.0_f32.to_be_bytes());
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);

    assert_eq!(
        parse_container_set_slot_packet(&payload).expect("component-bearing container slot"),
        super::super::ContainerSetSlotObservation {
            container_id: 0,
            state_id: 8,
            slot: 36,
            item_count: Some(1),
            item_registry_id: Some(1_046),
            positive_component_type_ids: vec![17],
            negative_component_type_ids: Vec::new(),
            item: Some(super::super::ClientItemStackState::from_wire(
                1,
                1_046,
                vec![17],
                Vec::new(),
            )),
            applied_to_world: false,
        }
    );
}

#[test]
fn container_slot_parser_skips_food_component() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(10, &mut payload);
    payload.extend_from_slice(&36_i16.to_be_bytes());
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(1_089, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(23, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);
    payload.extend_from_slice(&0.1_f32.to_be_bytes());
    payload.push(1);

    assert_eq!(
        parse_container_set_slot_packet(&payload).expect("food component-bearing container slot"),
        super::super::ContainerSetSlotObservation {
            container_id: 0,
            state_id: 10,
            slot: 36,
            item_count: Some(1),
            item_registry_id: Some(1_089),
            positive_component_type_ids: vec![23],
            negative_component_type_ids: Vec::new(),
            item: Some(super::super::ClientItemStackState::from_wire(
                1,
                1_089,
                vec![23],
                Vec::new(),
            )),
            applied_to_world: false,
        }
    );
}

#[test]
fn container_slot_parser_skips_damage_tool_and_weapon_components() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(9, &mut payload);
    payload.extend_from_slice(&36_i16.to_be_bytes());
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(1_000, &mut payload);
    voxel_core::wire::write_varint(3, &mut payload);
    voxel_core::wire::write_varint(0, &mut payload);

    voxel_core::wire::write_varint(3, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);

    voxel_core::wire::write_varint(28, &mut payload);
    voxel_core::wire::write_varint(2, &mut payload);
    let incorrect_tag = b"minecraft:incorrect_for_wooden_tool";
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(incorrect_tag.len() as i32, &mut payload);
    payload.extend_from_slice(incorrect_tag);
    payload.push(0);
    payload.push(1);
    payload.push(0);
    voxel_core::wire::write_varint(0, &mut payload);
    let mineable_tag = b"minecraft:mineable/shovel";
    voxel_core::wire::write_varint(mineable_tag.len() as i32, &mut payload);
    payload.extend_from_slice(mineable_tag);
    payload.push(1);
    payload.extend_from_slice(&2.0_f32.to_be_bytes());
    payload.push(1);
    payload.push(1);
    payload.extend_from_slice(&1.0_f32.to_be_bytes());
    voxel_core::wire::write_varint(1, &mut payload);
    payload.push(1);

    voxel_core::wire::write_varint(29, &mut payload);
    voxel_core::wire::write_varint(2, &mut payload);
    payload.extend_from_slice(&0.0_f32.to_be_bytes());

    assert_eq!(
        parse_container_set_slot_packet(&payload).expect("tool component-bearing slot"),
        super::super::ContainerSetSlotObservation {
            container_id: 0,
            state_id: 9,
            slot: 36,
            item_count: Some(1),
            item_registry_id: Some(1_000),
            positive_component_type_ids: vec![3, 28, 29],
            negative_component_type_ids: Vec::new(),
            item: Some(super::super::ClientItemStackState::from_wire(
                1,
                1_000,
                vec![3, 28, 29],
                Vec::new(),
            )),
            applied_to_world: false,
        }
    );
}

#[test]
fn container_content_parser_matches_empty_hotbar_stack() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(9, &mut payload);
    voxel_core::wire::write_varint(37, &mut payload);
    for _ in 0..37 {
        voxel_core::wire::write_varint(0, &mut payload);
    }
    voxel_core::wire::write_varint(0, &mut payload);

    assert_eq!(
        parse_container_set_content_packet(&payload).expect("container content packet"),
        super::super::ContainerSetContentObservation {
            container_id: 0,
            state_id: 9,
            item_count: 37,
            items: vec![None; 37],
            hotbar_slot_zero_item_count: None,
            hotbar_slot_zero_item_registry_id: None,
            hotbar_slot_zero_positive_component_type_ids: Vec::new(),
            hotbar_slot_zero_negative_component_type_ids: Vec::new(),
            main_inventory_slot_zero_item_count: None,
            main_inventory_slot_zero_item_registry_id: None,
            main_inventory_slot_zero_positive_component_type_ids: Vec::new(),
            main_inventory_slot_zero_negative_component_type_ids: Vec::new(),
            carried_item_count: None,
            carried_item_registry_id: None,
            carried_positive_component_type_ids: Vec::new(),
            carried_negative_component_type_ids: Vec::new(),
            carried_item: None,
            applied_to_world: false,
        }
    );
}

#[test]
fn container_content_parser_observes_main_inventory_slot_zero() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(0, &mut payload);
    voxel_core::wire::write_varint(10, &mut payload);
    voxel_core::wire::write_varint(46, &mut payload);
    for slot in 0..46 {
        if slot == 9 {
            voxel_core::wire::write_varint(1, &mut payload);
            voxel_core::wire::write_varint(1, &mut payload);
            voxel_core::wire::write_varint(0, &mut payload);
            voxel_core::wire::write_varint(0, &mut payload);
        } else {
            voxel_core::wire::write_varint(0, &mut payload);
        }
    }
    voxel_core::wire::write_varint(0, &mut payload);

    let observation =
        parse_container_set_content_packet(&payload).expect("main inventory slot packet");
    assert_eq!(observation.main_inventory_slot_zero_item_count, Some(1));
    assert_eq!(
        observation.main_inventory_slot_zero_item_registry_id,
        Some(1)
    );
    assert!(
        observation
            .main_inventory_slot_zero_positive_component_type_ids
            .is_empty()
    );
    assert!(
        observation
            .main_inventory_slot_zero_negative_component_type_ids
            .is_empty()
    );
    assert_eq!(
        observation.items[9],
        Some(super::super::ClientItemStackState::from_wire(
            1,
            1,
            vec![],
            vec![]
        ))
    );
    assert_eq!(observation.items.len(), 46);
    assert!(observation.carried_item.is_none());
    assert!(!observation.applied_to_world);
}

#[test]
fn use_item_on_encoder_matches_java_block_hit_result_field_order() {
    let mut payload = Vec::new();
    encode_use_item_on(&mut payload, (-12, 67, 9), 4).expect("use item on packet");

    let mut input = payload.as_slice();
    assert_eq!(voxel_core::wire::read_varint(&mut input).expect("hand"), 0);
    let (packed, rest) = input.split_at(8);
    input = rest;
    assert_eq!(
        i64::from_be_bytes(packed.try_into().expect("packed block position")),
        pack_block_position((-12, 67, 9))
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("direction"),
        1
    );
    assert_eq!(input.len(), 4 * 3 + 2 + 1);
    assert_eq!(&input[0..4], &0.5_f32.to_be_bytes());
    assert_eq!(&input[4..8], &1.0_f32.to_be_bytes());
    assert_eq!(&input[8..12], &0.5_f32.to_be_bytes());
    assert_eq!(input[12], 0);
    assert_eq!(input[13], 0);
    input = &input[14..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("sequence"),
        4
    );
    assert!(input.is_empty());
}

#[test]
fn creative_stone_slot_encoder_matches_java_optional_item_stack() {
    let mut payload = Vec::new();
    encode_creative_stone_slot(&mut payload, 36, 1).expect("creative slot packet");

    let mut input = payload.as_slice();
    let (slot, rest) = input.split_at(2);
    input = rest;
    assert_eq!(i16::from_be_bytes(slot.try_into().expect("slot")), 36);
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item count"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item registry id"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("positive components"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("negative components"),
        0
    );
    assert!(input.is_empty());
}

#[test]
fn creative_slot_encoder_writes_the_patch_the_stack_carries() {
    use voxel_world::ClientItemStackState as Stack;
    // A stack made here has no patch: both counts are zero.
    let mut payload = Vec::new();
    encode_creative_slot(&mut payload, 38, Some(&Stack::new(3, 1_044)));
    let mut input = &payload[2..];
    assert_eq!(
        i16::from_be_bytes(payload[..2].try_into().expect("slot")),
        38
    );
    assert_eq!(voxel_core::wire::read_varint(&mut input).expect("count"), 3);
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item"),
        1_044
    );
    assert_eq!(input, &[0, 0]);
    // A stack with a patch writes each value behind its length.
    let stack = Stack::from_wire(1, 5, vec![103], Vec::new()).with_patch_bytes(vec![1, 0, 103, 1]);
    let mut payload = Vec::new();
    encode_creative_slot(&mut payload, -1, Some(&stack));
    assert_eq!(
        i16::from_be_bytes(payload[..2].try_into().expect("slot")),
        -1
    );
    assert_eq!(&payload[2..], &[1, 5, 1, 0, 103, 1, 1]);
    // No stack is a count of zero.
    let mut payload = Vec::new();
    encode_creative_slot(&mut payload, 36, None);
    assert_eq!(&payload[2..], &[0]);
}

#[test]
fn creative_item_slot_encoder_keeps_registry_id_and_empty_components() {
    let mut payload = Vec::new();
    encode_creative_item_slot(&mut payload, 36, 1, 1_044).expect("creative item slot");

    let mut input = payload.as_slice();
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item count"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item registry id"),
        1_044
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("positive components"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("negative components"),
        0
    );
    assert!(input.is_empty());
}

#[test]
fn carried_item_encoder_matches_java_short_slot() {
    let mut payload = Vec::new();
    encode_set_carried_item(&mut payload, 0).expect("carried item packet");
    assert_eq!(payload, [0, 0]);
}

#[test]
fn inventory_click_encoder_matches_java_empty_prediction_fields() {
    let mut payload = Vec::new();
    encode_inventory_pickup_state_correction(&mut payload, 0).expect("inventory click packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
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
    assert_eq!(input, [0]);
}

#[test]
fn inventory_secondary_pickup_encoder_keeps_secondary_button() {
    let mut payload = Vec::new();
    encode_inventory_secondary_pickup_state_correction(&mut payload, 0)
        .expect("inventory secondary pickup packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 1);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_throw_encoder_keeps_throw_input_id() {
    let mut payload = Vec::new();
    encode_inventory_throw_one_state_correction(&mut payload, 0).expect("inventory throw packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        4
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_throw_all_encoder_keeps_secondary_throw_button() {
    let mut payload = Vec::new();
    encode_inventory_throw_all_state_correction(&mut payload, 0)
        .expect("inventory throw-all packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 1);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        4
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_drag_encoder_keeps_empty_slot_and_quick_craft_input_id() {
    let mut payload = Vec::new();
    encode_inventory_drag_empty_cancel_state_correction(&mut payload, 0)
        .expect("inventory drag packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(
        i16::from_be_bytes(input[..2].try_into().expect("slot")),
        -999
    );
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        5
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_drag_distribution_encoder_keeps_slot_button_and_input() {
    let mut payload = Vec::new();
    encode_inventory_click_with_input(&mut payload, 0, -999, 2, 5)
        .expect("inventory drag distribution packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(
        i16::from_be_bytes(input[..2].try_into().expect("slot")),
        -999
    );
    input = &input[2..];
    assert_eq!(input[0], 2);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        5
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_double_click_encoder_keeps_empty_target_and_pickup_all_input() {
    let mut payload = Vec::new();
    encode_inventory_click_with_input(&mut payload, 0, 10, 0, 6)
        .expect("inventory double-click packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 10);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        6
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_click_encoder_keeps_ahead_state_id() {
    let mut payload = Vec::new();
    encode_inventory_click_with_input(&mut payload, 99, 36, 0, 0)
        .expect("ahead-state inventory click packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        99
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
    assert_eq!(input, [0]);
}

#[test]
fn inventory_pickup_prediction_encoder_keeps_item_hash_fields() {
    let mut payload = Vec::new();
    encode_inventory_pickup_with_prediction(&mut payload, 2, 8, 1)
        .expect("inventory prediction packet");

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
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(input[0], 1);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item registry id"),
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

#[test]
fn inventory_wrong_slot_prediction_encoder_keeps_pre_click_slot_and_carried_hashes() {
    let mut payload = Vec::new();
    encode_inventory_pickup_with_wrong_slot_prediction(&mut payload, 2, 8, 1)
        .expect("wrong-slot inventory prediction packet");

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

    for hash in 0..2 {
        assert_eq!(input[0], 1, "hash {hash} should be non-empty");
        input = &input[1..];
        assert_eq!(
            voxel_core::wire::read_varint(&mut input).expect("item registry id"),
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
    }
    assert!(input.is_empty());
}

#[test]
fn inventory_wrong_carried_prediction_encoder_keeps_empty_carried_hash() {
    let mut payload = Vec::new();
    encode_inventory_pickup_with_wrong_carried_prediction(&mut payload, 2)
        .expect("wrong-carried inventory prediction packet");

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
    assert_eq!(input[0], 0, "carried prediction should be empty");
    input = &input[1..];
    assert!(input.is_empty());
}

#[test]
fn inventory_wrong_both_prediction_encoder_keeps_slot_item_and_empty_carried_hash() {
    let mut payload = Vec::new();
    encode_inventory_pickup_with_wrong_both_prediction(&mut payload, 2, 8, 1)
        .expect("wrong-both inventory prediction packet");

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
    assert_eq!(input[0], 1, "changed slot should be non-empty");
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("item registry id"),
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
    assert_eq!(input[0], 0, "carried prediction should be empty");
    input = &input[1..];
    assert!(input.is_empty());
}

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

#[test]
fn inventory_quick_move_encoder_keeps_quick_move_input_id() {
    let mut payload = Vec::new();
    encode_inventory_quick_move_state_correction(&mut payload, 0)
        .expect("inventory quick move packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[3..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        1
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn inventory_swap_encoder_keeps_target_slot_and_swap_input_id() {
    let mut payload = Vec::new();
    encode_inventory_swap_state_correction(&mut payload, 0).expect("inventory swap packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 9);
    input = &input[3..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("container input"),
        2
    );
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}

#[test]
fn use_item_encoder_matches_java_field_order() {
    let mut payload = Vec::new();
    encode_use_item(&mut payload, 10, 30.0, -15.0).expect("use item packet");

    let mut input = payload.as_slice();
    assert_eq!(voxel_core::wire::read_varint(&mut input).expect("hand"), 0);
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("sequence"),
        10
    );
    assert_eq!(&input[0..4], &30.0_f32.to_be_bytes());
    assert_eq!(&input[4..8], &(-15.0_f32).to_be_bytes());
    assert!(input[8..].is_empty());
}
