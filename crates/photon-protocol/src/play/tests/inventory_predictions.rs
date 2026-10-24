use super::*;

#[test]
fn inventory_pickup_prediction_encoder_keeps_item_hash_fields() {
    let mut payload = Vec::new();
    encode_inventory_pickup_with_prediction(&mut payload, 2, 8, ItemId(1))
        .expect("inventory prediction packet");

    let mut input = payload.as_slice();
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("state id"),
        2
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
        photon_core::wire::read_varint(&mut input).expect("item registry id"),
        1
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("item count"),
        8
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("component map count"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("removed component count"),
        0
    );
    assert!(input.is_empty());
}

#[test]
fn inventory_wrong_slot_prediction_encoder_keeps_pre_click_slot_and_carried_hashes() {
    let mut payload = Vec::new();
    encode_inventory_pickup_with_wrong_slot_prediction(&mut payload, 2, 8, ItemId(1))
        .expect("wrong-slot inventory prediction packet");

    let mut input = payload.as_slice();
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("state id"),
        2
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
            photon_core::wire::read_varint(&mut input).expect("item registry id"),
            1
        );
        assert_eq!(
            photon_core::wire::read_varint(&mut input).expect("item count"),
            8
        );
        assert_eq!(
            photon_core::wire::read_varint(&mut input).expect("component map count"),
            0
        );
        assert_eq!(
            photon_core::wire::read_varint(&mut input).expect("removed component count"),
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
        photon_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("state id"),
        2
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
    encode_inventory_pickup_with_wrong_both_prediction(&mut payload, 2, 8, ItemId(1))
        .expect("wrong-both inventory prediction packet");

    let mut input = payload.as_slice();
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("state id"),
        2
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
        photon_core::wire::read_varint(&mut input).expect("item registry id"),
        1
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("item count"),
        8
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("component map count"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("removed component count"),
        0
    );
    assert_eq!(input[0], 0, "carried prediction should be empty");
    input = &input[1..];
    assert!(input.is_empty());
}

#[test]
fn inventory_quick_move_encoder_keeps_quick_move_input_id() {
    let mut payload = Vec::new();
    encode_inventory_quick_move_state_correction(&mut payload, 0)
        .expect("inventory quick move packet");

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
    input = &input[3..];
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("container input"),
        1
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("changed slot count"),
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
        photon_core::wire::read_varint(&mut input).expect("container id"),
        0
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("state id"),
        0
    );
    assert_eq!(i16::from_be_bytes(input[..2].try_into().expect("slot")), 9);
    input = &input[3..];
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("container input"),
        2
    );
    assert_eq!(
        photon_core::wire::read_varint(&mut input).expect("changed slot count"),
        0
    );
    assert_eq!(input, [0]);
}
