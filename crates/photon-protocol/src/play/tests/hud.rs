use super::*;

#[test]
fn title_packets_read_their_fields() {
    let animation = [0, 0, 0, 10, 0, 0, 0, 70, 0, 0, 0, 20];
    assert_eq!(
        super::super::parse_hud_packet(115, &animation).unwrap(),
        Some(super::super::HudEvent::Times {
            fade_in: 10,
            stay: 70,
            fade_out: 20
        })
    );
    assert_eq!(
        super::super::parse_hud_packet(14, &[1]).unwrap(),
        Some(super::super::HudEvent::Clear { reset_times: true })
    );
    let title = [8, 0, 2, b'h', b'i'];
    assert_eq!(
        super::super::parse_hud_packet(114, &title).unwrap(),
        Some(super::super::HudEvent::Title(
            photon_game::text::Component::literal("hi")
        ))
    );
}

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
                name: photon_game::text::Component::literal("hi"),
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
    use photon_game::waypoint::{WaypointId, WaypointKind, WaypointOperation};
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
    use photon_game::scoreboard::{NumberFormat, ScoreboardEvent};
    use photon_game::text::Component;
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
fn the_border_packets_carry_doubles_and_a_varlong_lerp() {
    use photon_core::wire::write_varint;
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
fn player_info_update_reads_its_actions_in_order() {
    // Add, update listed, update latency for one player.
    let mut payload = vec![1 | 8 | 16, 1];
    payload.extend([0; 15]);
    payload.push(9); // uuid
    payload.extend([2, b'A', b'l', 0]); // name, no properties
    payload.extend([1, 50]); // listed, latency
    let entries = super::super::parse_player_info_update(&payload).unwrap();
    assert_eq!(entries[0].0, (0, 9));
    assert_eq!(
        entries[0].1,
        vec![
            super::super::InfoAction::Add {
                name: "Al".into(),
                properties: Vec::new(),
            },
            super::super::InfoAction::Listed(true),
            super::super::InfoAction::Latency(50),
        ]
    );
}

#[test]
fn player_info_add_keeps_the_profile_properties() {
    let mut payload = vec![1, 1];
    payload.extend([0; 15]);
    payload.push(9); // uuid
    payload.extend([2, b'A', b'l', 2]); // name, two properties
    payload.extend([8]);
    payload.extend(b"textures");
    payload.extend([3]);
    payload.extend(b"abc");
    payload.extend([1, 2]); // signed
    payload.extend(b"sg");
    payload.extend([1, b'x', 1, b'y', 0]); // unsigned
    let entries = super::super::parse_player_info_update(&payload).unwrap();
    let super::super::InfoAction::Add { properties, .. } = &entries[0].1[0] else {
        panic!("an add action");
    };
    assert_eq!(properties.len(), 2);
    assert_eq!(properties[0].name, "textures");
    assert_eq!(properties[0].value, "abc");
    assert_eq!(properties[0].signature.as_deref(), Some("sg"));
    assert_eq!(properties[1].signature, None);
}
