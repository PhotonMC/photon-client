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
fn cursor_and_direct_player_inventory_parsers_match_java_optional_stack_codecs() {
    let cursor_payload = [0];
    assert_eq!(
        parse_cursor_item_packet(&cursor_payload).expect("empty cursor item packet"),
        super::super::CursorItemObservation {
            item_count: None,
            item_registry_id: None,
            positive_component_type_ids: Vec::new(),
            negative_component_type_ids: Vec::new(),
            item: None,
            applied_to_world: false,
        }
    );

    let mut inventory_payload = Vec::new();
    voxel_core::wire::write_varint(0, &mut inventory_payload);
    voxel_core::wire::write_varint(1, &mut inventory_payload);
    voxel_core::wire::write_varint(1, &mut inventory_payload);
    voxel_core::wire::write_varint(0, &mut inventory_payload);
    voxel_core::wire::write_varint(0, &mut inventory_payload);
    assert_eq!(
        parse_set_player_inventory_packet(&inventory_payload)
            .expect("direct player inventory packet"),
        super::super::SetPlayerInventoryObservation {
            slot: 0,
            item_count: Some(1),
            item_registry_id: Some(1),
            positive_component_type_ids: Vec::new(),
            negative_component_type_ids: Vec::new(),
            item: Some(super::super::ClientItemStackState::from_wire(
                1,
                1,
                Vec::new(),
                Vec::new(),
            )),
            applied_to_world: false,
        }
    );
}

#[test]
fn open_screen_and_container_close_parsers_preserve_java_fields() {
    let title = voxel_core::nbt::FieldValue::Compound(vec![voxel_core::nbt::Field {
        key: "text".to_owned(),
        value: voxel_core::nbt::FieldValue::Str("Chest".to_owned()),
    }]);
    let mut title_payload = Vec::new();
    voxel_core::nbt::write_any_nbt(Some(&title), &mut title_payload);

    let mut open_payload = Vec::new();
    voxel_core::wire::write_varint(7, &mut open_payload);
    voxel_core::wire::write_varint(2, &mut open_payload);
    open_payload.extend_from_slice(&title_payload);
    assert_eq!(
        parse_open_screen_packet(&open_payload).expect("open screen packet"),
        super::super::OpenScreenObservation {
            container_id: 7,
            menu_type_id: 2,
            title_nbt: title_payload,
            applied_to_world: false,
        }
    );

    let mut close_payload = Vec::new();
    voxel_core::wire::write_varint(7, &mut close_payload);
    assert_eq!(
        parse_container_close_packet(&close_payload).expect("container close packet"),
        super::super::ContainerCloseObservation {
            container_id: 7,
            applied_to_world: false,
        }
    );
}

#[test]
fn a_crafter_slot_state_is_the_slot_the_container_and_a_flag() {
    assert_eq!(encode_container_slot_state(4, 7, false), vec![4, 7, 0]);
    assert_eq!(
        encode_container_slot_state(300, 2, true),
        vec![0xAC, 0x02, 2, 1]
    );
}

#[test]
fn merchant_offers_read_each_offer_and_the_traders_standing() {
    use voxel_core::wire::write_varint;
    let mut payload = Vec::new();
    write_varint(5, &mut payload);
    write_varint(1, &mut payload);
    // The cost: item 4, three of it, no components; the result: two of item 7, no patch.
    for value in [4, 3, 0, 2, 7, 0, 0] {
        write_varint(value, &mut payload);
    }
    // No second cost, not out of stock, then uses, max uses, xp, special price.
    payload.extend_from_slice(&[0, 0]);
    for value in [1_i32, 12, 2, -1] {
        payload.extend_from_slice(&value.to_be_bytes());
    }
    payload.extend_from_slice(&0.05_f32.to_be_bytes());
    payload.extend_from_slice(&3_i32.to_be_bytes());
    // Level 2, 15 experience, the bar shown, no restocking.
    for value in [2, 15] {
        write_varint(value, &mut payload);
    }
    payload.extend_from_slice(&[1, 0]);
    let (container, state) = parse_merchant_offers_packet(&payload).unwrap();
    assert_eq!(container, 5);
    let offer = &state.offers[0];
    assert_eq!((offer.cost_a.item, offer.cost_a.count), (4, 3));
    assert_eq!(
        (offer.result.item_registry_id(), offer.result.item_count()),
        (7, 2)
    );
    assert_eq!((offer.uses, offer.max_uses, offer.xp), (1, 12, 2));
    assert_eq!((offer.special_price_diff, offer.demand), (-1, 3));
    assert!((offer.price_multiplier - 0.05).abs() < 1e-6);
    assert!(offer.cost_b.is_none() && !offer.is_out_of_stock());
    assert_eq!((state.trader.level, state.trader.xp), (2, 15));
    assert!(state.trader.show_progress && !state.trader.can_restock);
    assert_eq!(encode_select_trade(3), vec![3]);
}

#[test]
fn a_beacon_choice_is_two_optional_effect_ids() {
    assert_eq!(encode_set_beacon(None, None), vec![0, 0]);
    assert_eq!(encode_set_beacon(Some(10), None), vec![1, 10, 0]);
    assert_eq!(
        encode_set_beacon(Some(1), Some(300)),
        vec![1, 1, 1, 0xAC, 0x02]
    );
}
