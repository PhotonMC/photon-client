use super::*;

#[test]
fn map_item_data_parser_reads_decorations_and_a_patch_of_colours() {
    let mut payload = Vec::new();
    photon_core::wire::write_varint(7, &mut payload);
    payload.extend([2, 1]);
    // Decorations: one, of type 3, at (-4, 5), turned 19 (kept to 3), with no name.
    payload.push(1);
    photon_core::wire::write_varint(1, &mut payload);
    photon_core::wire::write_varint(3, &mut payload);
    payload.extend([(-4_i8) as u8, 5, 19, 0]);
    // A patch of 2 by 2 colours at (10, 20).
    payload.extend([2, 2, 10, 20]);
    photon_core::wire::write_varint(4, &mut payload);
    payload.extend([1, 2, 3, 4]);

    let packet = parse_map_item_data_packet(&payload).expect("map data");
    assert_eq!((packet.map_id, packet.scale, packet.locked), (7, 2, true));
    let decoration = &packet.decorations.as_ref().expect("decorations")[0];
    assert_eq!(
        (
            decoration.type_id,
            decoration.x,
            decoration.y,
            decoration.rot
        ),
        (3, -4, 5, 3)
    );
    let patch = packet.patch.as_ref().expect("a patch");
    assert_eq!(
        (patch.start_x, patch.start_y, patch.width, patch.height),
        (10, 20, 2, 2)
    );

    // A width of zero is no patch, and no decorations is none sent.
    let bare = [vec![7_u8, 0, 0, 0, 0]].concat();
    let none = parse_map_item_data_packet(&bare).expect("bare");
    assert!(none.decorations.is_none() && none.patch.is_none());
    payload.push(0);
    assert!(parse_map_item_data_packet(&payload).is_err());
}
