use super::*;

#[test]
fn creative_stone_slot_encoder_matches_java_optional_item_stack() {
    let mut payload = Vec::new();
    encode_creative_stone_slot(&mut payload, 36, ItemId(1)).expect("creative slot packet");

    let mut input = payload.as_slice();
    let (slot, rest) = input.split_at(2);
    input = rest;
    assert_eq!(i16::from_be_bytes(slot.try_into().expect("slot")), 36);
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("item count"),
        1
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("item registry id"),
        1
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("positive components"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("negative components"),
        0
    );
    assert!(input.is_empty());
}

#[test]
fn creative_slot_encoder_writes_the_patch_the_stack_carries() {
    use photon_data::item_stack::ClientItemStackState as Stack;
    // A stack made here has no patch: both counts are zero.
    let mut payload = Vec::new();
    encode_creative_slot(&mut payload, 38, Some(&Stack::new(3, ItemId(1_044))));
    let mut input = &payload[2..];
    assert_eq!(
        i16::from_be_bytes(payload[..2].try_into().expect("slot")),
        38
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("count"),
        3
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("item"),
        1_044
    );
    assert_eq!(input, &[0, 0]);
    // A stack with a patch writes each value behind its length.
    let s = Stack::from_wire(1, ItemId(5), vec![103], vec![]).with_patch_bytes(vec![1, 0, 103, 1]);
    let mut payload = Vec::new();
    encode_creative_slot(&mut payload, -1, Some(&s));
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
    encode_creative_item_slot(&mut payload, 36, 1, ItemId(1_044)).expect("creative item slot");

    let mut input = payload.as_slice();
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("item count"),
        1
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("item registry id"),
        1_044
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("positive components"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("negative components"),
        0
    );
    assert!(input.is_empty());
}

#[test]
fn inventory_click_encoder_matches_java_empty_prediction_fields() {
    let mut payload = Vec::new();
    encode_inventory_pickup_state_correction(&mut payload, 0).expect("inventory click packet");

    let mut input = payload.as_slice();
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("container input"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("changed slot count"),
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
        photon_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 1);
    input = &input[1..];
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("container input"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("changed slot count"),
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
        photon_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("container input"),
        4
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("changed slot count"),
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
        photon_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 1);
    input = &input[1..];
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("container input"),
        4
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("changed slot count"),
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
        photon_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("state id"),
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
        photon_core::wire::read_varint(&mut input).expect("container input"),
        5
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("changed slot count"),
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
        photon_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("state id"),
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
        photon_core::wire::read_varint(&mut input).expect("container input"),
        5
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("changed slot count"),
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
        photon_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 10);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("container input"),
        6
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("changed slot count"),
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
        photon_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("state id"),
        99
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 36);
    input = &input[2..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("container input"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}
