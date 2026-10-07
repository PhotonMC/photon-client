use super::*;

#[test]
fn boss_event_packets_read_each_operation() {
    let mut add = 7u128.to_be_bytes().to_vec();
    add.extend([0, 8, 0, 2, b'h', b'i']);
    add.extend(0.5f32.to_be_bytes());
    add.extend([3, 2, 0b101]);
    assert_eq!(
        super::super::parse_hud_packet(9, &add).unwrap(),
        Some(super::super::HudEvent::Boss {
            id: 7,
            operation: super::super::BossOperation::Add {
                name: voxel_game::text::Component::literal("hi"),
                progress: 0.5,
                color: 3,
                overlay: 2,
                flags: 0b101,
            },
        })
    );
    let mut remove = 7u128.to_be_bytes().to_vec();
    remove.push(1);
    assert_eq!(
        super::super::parse_hud_packet(9, &remove).unwrap(),
        Some(super::super::HudEvent::Boss {
            id: 7,
            operation: super::super::BossOperation::Remove,
        })
    );
    let mut style = 7u128.to_be_bytes().to_vec();
    style.extend([4, 7, 0]);
    assert!(super::super::parse_hud_packet(9, &style).is_err());
}

#[test]
fn a_waypoint_packet_reads_its_operation_id_icon_and_place() {
    use voxel_game::waypoint::{WaypointId, WaypointKind, WaypointOperation};
    // Track, a UUID, the default style with no colour, a block position.
    let mut track = vec![0, 1];
    track.extend(5u128.to_be_bytes());
    track.push(17);
    track.extend(b"minecraft:default");
    track.extend([0, 1, 10, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0xE8, 0x07]);
    let (operation, waypoint) = super::super::parse_waypoint_packet(&track).unwrap();
    assert_eq!(operation, WaypointOperation::Track);
    assert_eq!(waypoint.id, WaypointId::Uuid(5));
    assert_eq!(waypoint.icon.style, "minecraft:default");
    assert_eq!(waypoint.icon.colour, None);
    // A varint of -1 is five bytes.
    assert_eq!(waypoint.kind, WaypointKind::Position([10, -1, 1000]));
    // Untrack, a name, a colour, an azimuth.
    let mut update = vec![2, 0, 1, b'n', 17];
    update.extend(b"minecraft:default");
    update.extend([1, 0x10, 0x20, 0x30, 3]);
    update.extend(1.5f32.to_be_bytes());
    let (operation, waypoint) = super::super::parse_waypoint_packet(&update).unwrap();
    assert_eq!(operation, WaypointOperation::Update);
    assert_eq!(waypoint.id, WaypointId::Name("n".to_owned()));
    assert_eq!(waypoint.icon.colour, Some(0x10_2030));
    assert_eq!(waypoint.kind, WaypointKind::Azimuth(1.5));
    // A type past the last is refused.
    let mut odd = vec![1, 1];
    odd.extend(5u128.to_be_bytes());
    odd.push(17);
    odd.extend(b"minecraft:default");
    odd.extend([0, 9]);
    assert!(super::super::parse_waypoint_packet(&odd).is_err());
}

#[test]
fn scoreboard_packets_read_their_fields() {
    use voxel_game::scoreboard::{NumberFormat, ScoreboardEvent};
    use voxel_game::text::Component;
    // Objective "o": add, display name "hi", hearts, a blank number format.
    let mut objective = vec![1, b'o', 0, 8, 0, 2, b'h', b'i', 1, 1, 0];
    assert_eq!(
        super::super::parse_scoreboard_packet(106, &objective).unwrap(),
        Some(ScoreboardEvent::Objective {
            name: "o".to_owned(),
            method: 0,
            display_name: Component::literal("hi"),
            hearts: true,
            number_format: Some(NumberFormat::Blank),
        })
    );
    // A removal carries nothing more.
    objective.truncate(3);
    objective[2] = 1;
    assert!(matches!(
        super::super::parse_scoreboard_packet(106, &objective).unwrap(),
        Some(ScoreboardEvent::Objective { method: 1, .. })
    ));
    // Score of "al" on "o": 5, no display, a fixed format.
    let score = [2, b'a', b'l', 1, b'o', 5, 0, 1, 2, 8, 0, 1, b'x'];
    assert_eq!(
        super::super::parse_scoreboard_packet(110, &score).unwrap(),
        Some(ScoreboardEvent::Score {
            owner: "al".to_owned(),
            objective: "o".to_owned(),
            value: 5,
            display: None,
            number_format: Some(NumberFormat::Fixed(Component::literal("x"))),
        })
    );
    // Reset of every score of "al".
    assert_eq!(
        super::super::parse_scoreboard_packet(79, &[2, b'a', b'l', 0]).unwrap(),
        Some(ScoreboardEvent::ResetScore {
            owner: "al".to_owned(),
            objective: None,
        })
    );
    // Slot 1 shows "o"; an empty name clears the slot.
    assert_eq!(
        super::super::parse_scoreboard_packet(98, &[1, 1, b'o']).unwrap(),
        Some(ScoreboardEvent::Display {
            slot: 1,
            objective: Some("o".to_owned()),
        })
    );
    assert_eq!(
        super::super::parse_scoreboard_packet(98, &[1, 0]).unwrap(),
        Some(ScoreboardEvent::Display {
            slot: 1,
            objective: None,
        })
    );
    // Team "t": add, three components, two rules, colour 12, options, one player.
    let mut team = vec![1, b't', 0];
    for text in ["t", "[", "]"] {
        team.extend([8, 0, 1, text.as_bytes()[0]]);
    }
    team.extend([0, 0, 1, 12, 3, 1, 2, b'a', b'l']);
    let Some(ScoreboardEvent::Team {
        parameters: Some(parameters),
        players,
        ..
    }) = super::super::parse_scoreboard_packet(109, &team).unwrap()
    else {
        panic!("a team with parameters");
    };
    assert_eq!(parameters.colour, Some(12));
    assert_eq!(players, ["al"]);
}

#[test]
fn a_commands_packet_links_into_a_tree() {
    use voxel_game::commands::NodeKind;
    // Node 0 is the root with children 1; node 1 literal "a" (executable)
    // with child 2; node 2 an integer argument "n" from 1 to 5.
    let mut packet = vec![3];
    packet.extend([0, 1, 1]);
    packet.extend([1 | 4, 1, 2, 1, b'a']);
    packet.extend([2 | 4, 0, 1, b'n', 3, 0b11]);
    packet.extend(1i32.to_be_bytes());
    packet.extend(5i32.to_be_bytes());
    packet.push(0);
    let tree = super::super::parse_commands_packet(&packet).unwrap();
    let root = tree.node(tree.root());
    assert_eq!(root.children.len(), 1);
    assert_eq!(
        tree.node(root.children[0]).kind,
        NodeKind::Literal("a".to_owned())
    );
    let parse = tree.parse("a 3", 0, true);
    assert!(!parse.can_read() && parse.exceptions.is_empty());
    let parse = tree.parse("a 9", 0, true);
    assert_eq!(parse.exceptions.len(), 1);
    // A child past the end is refused.
    assert!(super::super::parse_commands_packet(&[1, 0, 1, 7, 0]).is_err());
}

#[test]
fn combat_kill_names_who_died_and_the_message() {
    let packet = [0xE8, 0x07, 8, 0, 2, b'h', b'i'];
    let (id, message) = super::super::parse_player_combat_kill_packet(&packet).unwrap();
    assert_eq!(id, 1000);
    assert_eq!(message, voxel_game::text::Component::literal("hi"));
}

#[test]
fn a_disconnect_carries_its_reason() {
    let mut input: &[u8] = &[8, 0, 2, b'h', b'i'];
    assert_eq!(
        super::super::read_disconnect_reason(&mut input).unwrap(),
        voxel_game::text::Component::literal("hi")
    );
}

#[test]
fn an_entity_event_packet_names_the_entity_and_the_event() {
    let mut payload = 7_i32.to_be_bytes().to_vec();
    payload.push(26);
    assert_eq!(parse_entity_event_packet(&payload).unwrap(), (7, 26));
    assert!(parse_entity_event_packet(&payload[..4]).is_err());
}

#[test]
fn a_block_event_names_the_block_the_event_and_its_data() {
    let mut payload = voxel_core::binary::BlockPos::new(-3, 64, 12)
        .pack()
        .to_be_bytes()
        .to_vec();
    payload.extend_from_slice(&[1, 2, 0x05]);
    assert_eq!(
        parse_block_event_packet(&payload).unwrap(),
        ((-3, 64, 12), 1, 2)
    );
    assert!(parse_block_event_packet(&payload[..9]).is_err());
}

#[test]
fn the_border_packets_carry_doubles_and_a_varlong_lerp() {
    use voxel_core::wire::write_varint;
    let mut payload = Vec::new();
    for value in [10.5_f64, -3.25, 100.0, 40.0] {
        payload.extend_from_slice(&value.to_be_bytes());
    }
    // A lerp of 300 ticks (a var long), then the absolute maximum, warning blocks and time.
    payload.extend_from_slice(&[0xAC, 0x02]);
    for value in [29_999_984, 5, 15] {
        write_varint(value, &mut payload);
    }
    let init = parse_initialize_border_packet(&payload).unwrap();
    assert_eq!((init.center_x, init.center_z), (10.5, -3.25));
    assert_eq!(
        (init.old_size, init.new_size, init.lerp_time),
        (100.0, 40.0, 300)
    );
    assert_eq!(
        (
            init.absolute_max_size,
            init.warning_blocks,
            init.warning_time
        ),
        (29_999_984, 5, 15)
    );
    let mut lerp = Vec::new();
    for value in [60.0_f64, 20.0] {
        lerp.extend_from_slice(&value.to_be_bytes());
    }
    lerp.push(40);
    assert_eq!(
        parse_set_border_lerp_size_packet(&lerp).unwrap(),
        (60.0, 20.0, 40)
    );
    assert!(parse_set_border_lerp_size_packet(&lerp[..16]).is_err());
}

#[test]
fn a_hurt_animation_names_the_entity_and_the_yaw_of_the_blow() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(7, &mut payload);
    payload.extend_from_slice(&(-135.5_f32).to_be_bytes());
    assert_eq!(parse_hurt_animation_packet(&payload).unwrap(), (7, -135.5));
    assert!(parse_hurt_animation_packet(&payload[..3]).is_err());
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

#[test]
fn a_damage_event_names_the_entity_and_the_type_by_registry_id_plus_one() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(300, &mut payload);
    voxel_core::wire::write_varint(5, &mut payload);
    assert_eq!(parse_damage_event_packet(&payload).unwrap(), (300, Some(4)));
    // A type written out in the packet is not one the registry names.
    let mut direct = Vec::new();
    voxel_core::wire::write_varint(7, &mut direct);
    voxel_core::wire::write_varint(0, &mut direct);
    assert_eq!(parse_damage_event_packet(&direct).unwrap(), (7, None));
    assert!(parse_damage_event_packet(&payload[..2]).is_err());
}

#[test]
fn animate_reads_the_entity_and_the_action() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(300, &mut payload);
    payload.push(3);
    assert_eq!(
        super::super::parse_animate_packet(&payload).unwrap(),
        (300, 3)
    );
    assert!(super::super::parse_animate_packet(&payload[..2]).is_err());
}

#[test]
fn an_entity_link_names_the_leashed_entity_and_its_holder() {
    let mut packet = Vec::new();
    packet.extend_from_slice(&7_i32.to_be_bytes());
    packet.extend_from_slice(&12_i32.to_be_bytes());
    assert_eq!(
        super::super::parse_set_entity_link_packet(&packet).unwrap(),
        (7, 12)
    );
    // Nobody holds it any more.
    packet[4..].copy_from_slice(&0_i32.to_be_bytes());
    assert_eq!(
        super::super::parse_set_entity_link_packet(&packet).unwrap(),
        (7, 0)
    );
    assert!(super::super::parse_set_entity_link_packet(&packet[..6]).is_err());
}

#[test]
fn minecart_steps_parser_reads_doubles_packed_turns_and_a_float_weight() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(9, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);
    for value in [1.5_f64, 64.1, -2.0, 0.25, 0.0, -0.5] {
        payload.extend(value.to_be_bytes());
    }
    payload.extend([64, 0xe0]);
    payload.extend(0.75_f32.to_be_bytes());

    let (entity_id, steps) = parse_move_minecart_packet(&payload).expect("minecart steps");
    assert_eq!(entity_id, 9);
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].position, (1.5, 64.1, -2.0));
    assert_eq!(steps[0].movement, (0.25, 0.0, -0.5));
    assert_eq!((steps[0].y_rot, steps[0].x_rot), (90.0, -45.0));
    assert_eq!(steps[0].weight, 0.75);

    payload.push(0);
    assert!(parse_move_minecart_packet(&payload).is_err());
}

#[test]
fn map_item_data_parser_reads_decorations_and_a_patch_of_colours() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(7, &mut payload);
    payload.extend([2, 1]);
    // Decorations: one, of type 3, at (-4, 5), turned 19 (kept to 3), with no name.
    payload.push(1);
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(3, &mut payload);
    payload.extend([(-4_i8) as u8, 5, 19, 0]);
    // A patch of 2 by 2 colours at (10, 20).
    payload.extend([2, 2, 10, 20]);
    voxel_core::wire::write_varint(4, &mut payload);
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

    let mut world = voxel_world::ClientWorld::new(voxel_world::DimensionBounds::default());
    world.apply_map_item_data(&packet);
    let map = world.map_data(7).expect("the map");
    assert_eq!(map.colors()[10 + 20 * 128], 1);
    assert_eq!(map.colors()[11 + 21 * 128], 4);

    // A width of zero is no patch, and no decorations is none sent.
    let bare = [vec![7_u8, 0, 0, 0, 0]].concat();
    let none = parse_map_item_data_packet(&bare).expect("bare");
    assert!(none.decorations.is_none() && none.patch.is_none());
    payload.push(0);
    assert!(parse_map_item_data_packet(&payload).is_err());
}
