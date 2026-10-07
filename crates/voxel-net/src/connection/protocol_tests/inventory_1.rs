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
