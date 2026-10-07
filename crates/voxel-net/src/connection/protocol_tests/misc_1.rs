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
