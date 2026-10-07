use super::*;

#[test]
fn compression_uses_java_boundary_and_round_trips() {
    let below = vec![0x5a; 255];
    let at = vec![0xa5; 256];

    let below_frame = encode_packet_frame(&below, Some(256)).expect("frame below threshold");
    let mut below_header = below_frame.as_slice();
    voxel_core::wire::read_varint(&mut below_header).expect("below frame header");
    assert_eq!(below_header[0], 0x00);
    assert_eq!(decode_packet_frame(below_header, Some(256)).unwrap(), below);

    let at_frame = encode_packet_frame(&at, Some(256)).expect("frame at threshold");
    let mut at_header = at_frame.as_slice();
    voxel_core::wire::read_varint(&mut at_header).expect("at frame header");
    assert_ne!(at_header[0], 0x00);
    assert_eq!(decode_packet_frame(at_header, Some(256)).unwrap(), at);
}

#[test]
fn play_packet_ids_match_java_26_2_dispatch_order() {
    assert_eq!(PLAY_CLIENTBOUND_ADD_ENTITY_PACKET_ID, 1);
    assert_eq!(PLAY_CLIENTBOUND_ENTITY_POSITION_SYNC_PACKET_ID, 35);
    assert_eq!(PLAY_CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID, 53);
    assert_eq!(PLAY_CLIENTBOUND_MOVE_ENTITY_POS_ROT_PACKET_ID, 54);
    assert_eq!(PLAY_CLIENTBOUND_MOVE_ENTITY_ROT_PACKET_ID, 56);
    assert_eq!(PLAY_CLIENTBOUND_ROTATE_HEAD_PACKET_ID, 83);
    assert_eq!(PLAY_CLIENTBOUND_SET_ENTITY_LINK_PACKET_ID, 100);
    assert_eq!(PLAY_CLIENTBOUND_SET_ENTITY_MOTION_PACKET_ID, 101);
    assert_eq!(PLAY_CLIENTBOUND_MOVE_MINECART_PACKET_ID, 55);
    assert_eq!(PLAY_CLIENTBOUND_SET_PASSENGERS_PACKET_ID, 107);
    assert_eq!(PLAY_CLIENTBOUND_KEEP_ALIVE_PACKET_ID, 44);
    assert_eq!(PLAY_CLIENTBOUND_CHUNK_PACKET_ID, 45);
    assert_eq!(PLAY_CLIENTBOUND_LOGIN_PACKET_ID, 49);
    assert_eq!(PLAY_CLIENTBOUND_PLAYER_POSITION_PACKET_ID, 72);
    assert_eq!(PLAY_CLIENTBOUND_RESPAWN_PACKET_ID, 82);
    assert_eq!(PLAY_CLIENTBOUND_SET_HEALTH_PACKET_ID, 104);
    assert_eq!(PLAY_CLIENTBOUND_UPDATE_ATTRIBUTES_PACKET_ID, 131);
    assert_eq!(PLAY_CLIENTBOUND_BLOCK_UPDATE_PACKET_ID, 8);
    assert_eq!(PLAY_CLIENTBOUND_BLOCK_ENTITY_DATA_PACKET_ID, 6);
    assert_eq!(PLAY_CLIENTBOUND_FORGET_LEVEL_CHUNK_PACKET_ID, 37);
    assert_eq!(PLAY_CLIENTBOUND_GAME_EVENT_PACKET_ID, 38);
    assert_eq!(PLAY_CLIENTBOUND_LIGHT_UPDATE_PACKET_ID, 48);
    assert_eq!(PLAY_CLIENTBOUND_SET_CHUNK_CACHE_CENTER_PACKET_ID, 94);
    assert_eq!(PLAY_CLIENTBOUND_SET_CHUNK_CACHE_RADIUS_PACKET_ID, 95);
    assert_eq!(PLAY_CLIENTBOUND_BLOCK_CHANGED_ACK_PACKET_ID, 4);
    assert_eq!(PLAY_CLIENTBOUND_BLOCK_DESTRUCTION_PACKET_ID, 5);
    assert_eq!(PLAY_CLIENTBOUND_CONTAINER_SET_CONTENT_PACKET_ID, 18);
    assert_eq!(PLAY_CLIENTBOUND_CONTAINER_SET_SLOT_PACKET_ID, 20);
    assert_eq!(PLAY_CLIENTBOUND_CONTAINER_CLOSE_PACKET_ID, 17);
    assert_eq!(PLAY_CLIENTBOUND_OPEN_SCREEN_PACKET_ID, 59);
    assert_eq!(PLAY_CLIENTBOUND_PLAYER_ABILITIES_PACKET_ID, 64);
    assert_eq!(PLAY_CLIENTBOUND_SET_CURSOR_ITEM_PACKET_ID, 96);
    assert_eq!(PLAY_CLIENTBOUND_SET_PLAYER_INVENTORY_PACKET_ID, 108);
    assert_eq!(PLAY_CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID, 77);
    assert_eq!(PLAY_CLIENTBOUND_REMOVE_MOB_EFFECT_PACKET_ID, 78);
    assert_eq!(PLAY_CLIENTBOUND_SET_ENTITY_DATA_PACKET_ID, 99);
    assert_eq!(PLAY_CLIENTBOUND_TELEPORT_ENTITY_PACKET_ID, 125);
    assert_eq!(PLAY_CLIENTBOUND_SYSTEM_CHAT_PACKET_ID, 121);
    assert_eq!(PLAY_CLIENTBOUND_UPDATE_MOB_EFFECT_PACKET_ID, 132);
    assert_eq!(PLAY_SERVERBOUND_ACCEPT_TELEPORTATION_PACKET_ID, 0);
    assert_eq!(PLAY_SERVERBOUND_PLAYER_ACTION_PACKET_ID, 41);
    assert_eq!(PLAY_SERVERBOUND_CHUNK_BATCH_RECEIVED_PACKET_ID, 11);
    assert_eq!(PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID, 12);
    assert_eq!(PLAY_SERVERBOUND_CLIENT_TICK_END_PACKET_ID, 13);
    assert_eq!(PLAY_SERVERBOUND_KEEP_ALIVE_PACKET_ID, 28);
    assert_eq!(PLAY_SERVERBOUND_MOVE_PLAYER_POS_ROT_PACKET_ID, 31);
    assert_eq!(PLAY_SERVERBOUND_PLAYER_LOADED_PACKET_ID, 44);
    assert_eq!(PLAY_SERVERBOUND_CONTAINER_CLICK_PACKET_ID, 18);
    assert_eq!(PLAY_SERVERBOUND_SET_CARRIED_ITEM_PACKET_ID, 53);
    assert_eq!(PLAY_SERVERBOUND_SET_CREATIVE_MODE_SLOT_PACKET_ID, 56);
    assert_eq!(PLAY_SERVERBOUND_USE_ITEM_ON_PACKET_ID, 66);
    assert_eq!(PLAY_SERVERBOUND_USE_ITEM_PACKET_ID, 67);
    assert_eq!(GAME_EVENT_CHANGE_GAME_MODE_ID, 3);
}

#[test]
fn status_packet_ids_match_java_26_2_dispatch_order() {
    assert_eq!(STATUS_SERVERBOUND_REQUEST_PACKET_ID, 0);
    assert_eq!(STATUS_SERVERBOUND_PING_REQUEST_PACKET_ID, 1);
    assert_eq!(STATUS_CLIENTBOUND_RESPONSE_PACKET_ID, 0);
    assert_eq!(STATUS_CLIENTBOUND_PONG_RESPONSE_PACKET_ID, 1);
}

#[test]
fn login_and_respawn_parsers_match_java_common_spawn_info_order() {
    fn append_identifier(output: &mut Vec<u8>, value: &str) {
        let identifier = voxel_core::identifier::Identifier::parse(value).expect("identifier");
        voxel_core::identifier::write_identifier(&identifier, output).expect("identifier bytes");
    }

    fn append_spawn_info(output: &mut Vec<u8>) {
        voxel_core::wire::write_varint(1, output);
        append_identifier(output, "minecraft:the_nether");
        output.extend_from_slice(&0x0102_0304_0506_0708_i64.to_be_bytes());
        output.push(1);
        output.push(0);
        output.push(1);
        output.push(0);
        output.push(1);
        append_identifier(output, "minecraft:overworld");
        output.extend_from_slice(&pack_block_position((-3, 64, 7)).to_be_bytes());
        voxel_core::wire::write_varint(42, output);
        voxel_core::wire::write_varint(32, output);
    }

    let mut login_payload = Vec::new();
    login_payload.extend_from_slice(&17_i32.to_be_bytes());
    login_payload.push(1);
    voxel_core::wire::write_varint(2, &mut login_payload);
    append_identifier(&mut login_payload, "minecraft:overworld");
    append_identifier(&mut login_payload, "minecraft:the_nether");
    voxel_core::wire::write_varint(20, &mut login_payload);
    voxel_core::wire::write_varint(5, &mut login_payload);
    voxel_core::wire::write_varint(6, &mut login_payload);
    login_payload.push(0);
    login_payload.push(1);
    login_payload.push(0);
    append_spawn_info(&mut login_payload);
    login_payload.push(0);
    login_payload.push(1);

    let login = parse_login_packet(&login_payload).expect("login packet");
    assert_eq!(login.player_id, 17);
    assert!(login.hardcore);
    assert_eq!(login.level_count, 2);
    assert_eq!(login.max_players, 20);
    assert_eq!(login.chunk_radius, 5);
    assert_eq!(login.simulation_distance, 6);
    assert!(login.show_death_screen);
    assert_eq!(login.spawn_info.dimension_type_id, 1);
    assert_eq!(login.spawn_info.dimension, "minecraft:the_nether");
    assert_eq!(login.spawn_info.seed, 0x0102_0304_0506_0708);
    assert_eq!(login.spawn_info.game_type, 1);
    assert_eq!(login.spawn_info.previous_game_type, Some(0));
    assert!(login.spawn_info.is_debug);
    assert!(!login.spawn_info.is_flat);
    assert_eq!(login.spawn_info.portal_cooldown, 42);
    assert_eq!(login.spawn_info.sea_level, 32);
    assert!(!login.online_mode);
    assert!(login.enforces_secure_chat);
    assert_eq!(
        login.spawn_info.last_death_location,
        Some(super::super::GlobalPositionObservation {
            dimension: "minecraft:overworld".to_owned(),
            position: pack_block_position((-3, 64, 7)),
        })
    );

    let mut respawn_payload = Vec::new();
    append_spawn_info(&mut respawn_payload);
    respawn_payload.push(3);
    let (spawn_info, data_to_keep) = parse_respawn_packet(&respawn_payload).expect("respawn");
    assert_eq!(spawn_info, login.spawn_info);
    assert_eq!(data_to_keep, 3);

    respawn_payload.push(0);
    assert!(parse_respawn_packet(&respawn_payload).is_err());
}

#[test]
fn game_event_parser_matches_java_id_and_float_order() {
    let payload = [GAME_EVENT_CHANGE_GAME_MODE_ID, 0x3f, 0x80, 0x00, 0x00];
    assert_eq!(
        parse_game_event_packet(&payload).expect("game event packet"),
        super::super::GameEventPacket {
            event_id: GAME_EVENT_CHANGE_GAME_MODE_ID,
            param: 1.0,
        }
    );

    let mut trailing = payload.to_vec();
    trailing.push(0);
    assert!(parse_game_event_packet(&trailing).is_err());
}

#[test]
fn update_attributes_parser_matches_java_snapshot_and_modifier_order() {
    let mut payload = Vec::new();
    voxel_core::wire::write_varint(42, &mut payload);
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::wire::write_varint(17, &mut payload);
    payload.extend_from_slice(&2.5_f64.to_be_bytes());
    voxel_core::wire::write_varint(1, &mut payload);
    voxel_core::identifier::write_identifier(
        &voxel_core::identifier::Identifier::parse("minecraft:test_modifier")
            .expect("modifier identifier"),
        &mut payload,
    )
    .expect("modifier identifier bytes");
    payload.extend_from_slice(&0.25_f64.to_be_bytes());
    payload.push(2);

    let observation = parse_update_attributes_packet(&payload).expect("update attributes packet");
    assert_eq!(observation.entity_id, 42);
    assert!(!observation.applied_to_world);
    assert_eq!(observation.attributes.len(), 1);
    let attribute = &observation.attributes[0];
    assert_eq!(attribute.attribute_id(), 17);
    assert_eq!(attribute.base_value(), 2.5);
    assert_eq!(attribute.modifiers().len(), 1);
    assert_eq!(attribute.modifiers()[0].id(), "minecraft:test_modifier");
    assert_eq!(attribute.modifiers()[0].amount(), 0.25);
    assert_eq!(attribute.modifiers()[0].operation(), 2);

    payload.push(0);
    assert!(parse_update_attributes_packet(&payload).is_err());
}

#[test]
fn system_chat_parser_matches_translatable_overlay_component() {
    let mut payload = Vec::new();
    voxel_core::nbt::write_nbt(
        Some(&[voxel_core::nbt::Field {
            key: "translate".to_owned(),
            value: voxel_core::nbt::FieldValue::Str("build.spawn_protection".to_owned()),
        }]),
        &mut payload,
    );
    payload.push(1);

    assert_eq!(
        parse_system_chat_packet(&payload).expect("system chat packet"),
        super::super::SystemChatObservation {
            overlay: true,
            translation_key: Some("build.spawn_protection".to_owned()),
        }
    );
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
fn recipe_book_add_reads_a_shaped_entry_and_its_flags() {
    // One entry: id 7, a 2x1 shaped display (an item, an empty slot), the
    // result item 20, an empty station, no group, category 3, one
    // requirement of item 10, the highlight flag, replacing the book.
    let payload = [
        1, 7, 1, 2, 1, 2, 4, 10, 0, 4, 20, 0, 0, 3, 1, 1, 2, 10, 2, 1,
    ];
    let (entries, replace) = super::super::parse_recipe_book_add_packet(&payload).unwrap();
    assert!(replace);
    let (entry, flags) = &entries[0];
    assert_eq!(
        (entry.id, entry.group, entry.category, *flags),
        (7, None, 3, 2)
    );
    assert_eq!(
        entry.display,
        voxel_world::RecipeDisplay::Shaped {
            width: 2,
            height: 1,
            ingredients: vec![
                voxel_world::SlotDisplay::Item(10),
                voxel_world::SlotDisplay::Empty
            ],
            result: voxel_world::SlotDisplay::Item(20),
        }
    );
    assert_eq!(
        entry.requirements,
        Some(vec![voxel_world::Ingredient::Items(vec![10])])
    );
    let mut book = voxel_world::RecipeBook::default();
    book.add(entries, replace);
    assert!(book.highlight.contains(&7));
}

#[test]
fn the_signature_cache_pushes_new_entries_to_the_front() {
    let sig = |byte: u8| vec![byte; 256];
    let mut cache = super::super::SignatureCache::default();
    cache.push([sig(1), sig(2)].into());
    // The newest is at place 0, the one before it at 1.
    assert_eq!(
        cache.unpack(&super::super::PackedSignature::Cached(0)),
        Some(sig(2))
    );
    assert_eq!(
        cache.unpack(&super::super::PackedSignature::Cached(1)),
        Some(sig(1))
    );
    // One more shifts the old ones back; one it repeats is not kept twice.
    cache.push([sig(2), sig(3)].into());
    assert_eq!(
        cache.unpack(&super::super::PackedSignature::Cached(0)),
        Some(sig(3))
    );
    assert_eq!(
        cache.unpack(&super::super::PackedSignature::Cached(1)),
        Some(sig(2))
    );
    assert_eq!(
        cache.unpack(&super::super::PackedSignature::Cached(2)),
        Some(sig(1))
    );
    assert_eq!(
        cache.unpack(&super::super::PackedSignature::Cached(3)),
        None
    );
}

#[test]
fn a_partial_filter_mask_hides_its_set_bits() {
    let mut payload = vec![0];
    payload.extend([0; 16]);
    payload.extend([0, 0, 2, b'h', b'i']);
    payload.extend([0; 16]);
    payload.extend([0, 0, 2, 1]);
    payload.extend([0, 0, 0, 0, 0, 0, 0, 0b10]); // the second character
    payload.push(1);
    payload.extend([8, 0, 3, b'B', b'o', b'b', 0]);
    let packet = super::super::parse_player_chat_packet(&payload).unwrap();
    assert!(!packet.filter.hides(0) && packet.filter.hides(1));
}

#[test]
fn an_unsigned_chat_packet_ends_in_an_empty_acknowledgement() {
    let mut output = Vec::new();
    super::super::encode_chat("hi", 5, 7, &mut output);
    assert_eq!(output[..3], [2, b'h', b'i']);
    // offset 0, three bytes of bits, checksum 1, after the signature flag.
    assert_eq!(output[3 + 16..], [0, 0, 0, 0, 0, 1]);
}

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
            voxel_game::text::Component::literal("hi")
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
