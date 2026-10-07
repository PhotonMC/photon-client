use super::*;

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
fn carried_item_encoder_matches_java_short_slot() {
    let mut payload = Vec::new();
    encode_set_carried_item(&mut payload, 0).expect("carried item packet");
    assert_eq!(payload, [0, 0]);
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

#[test]
fn side_use_item_on_encoder_keeps_click_position_and_face() {
    let mut payload = Vec::new();
    encode_use_item_on_hit(&mut payload, (2, 70, -4), 2, (0.5, 0.5, 0.0), 5)
        .expect("side use item on packet");

    let mut input = payload.as_slice();
    assert_eq!(voxel_core::wire::read_varint(&mut input).expect("hand"), 0);
    input = &input[8..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("north direction"),
        2
    );
    assert_eq!(&input[0..4], &0.5_f32.to_be_bytes());
    assert_eq!(&input[4..8], &0.5_f32.to_be_bytes());
    assert_eq!(&input[8..12], &0.0_f32.to_be_bytes());
    assert_eq!(input[12], 0);
    assert_eq!(input[13], 0);
    input = &input[14..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("sequence"),
        5
    );
    assert!(input.is_empty());
}

#[test]
fn player_action_encoder_matches_java_abort_field_order() {
    let mut payload = Vec::new();
    encode_abort_destroy_block(&mut payload, (-12, 67, 9), 1).expect("action packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("action"),
        1
    );
    let (packed, rest) = input.split_at(8);
    input = rest;
    assert_eq!(
        i64::from_be_bytes(packed.try_into().expect("packed block position")),
        pack_block_position((-12, 67, 9))
    );
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("sequence"),
        1
    );
    assert!(input.is_empty());
}

#[test]
fn player_action_encoder_matches_java_stop_action() {
    let mut payload = Vec::new();
    encode_stop_destroy_block(&mut payload, (-12, 67, 9), 9).expect("stop action packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("action"),
        2
    );
    input = &input[8..];
    assert_eq!(input[0], 1);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("sequence"),
        9
    );
    assert!(input.is_empty());
}

#[test]
fn player_action_encoder_matches_java_release_use_item() {
    let mut payload = Vec::new();
    encode_release_use_item(&mut payload).expect("release use item packet");

    let mut input = payload.as_slice();
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("action"),
        5
    );
    assert_eq!(input[..8], [0; 8]);
    input = &input[8..];
    assert_eq!(input[0], 0);
    input = &input[1..];
    assert_eq!(
        voxel_core::wire::read_varint(&mut input).expect("sequence"),
        0
    );
    assert!(input.is_empty());
}
