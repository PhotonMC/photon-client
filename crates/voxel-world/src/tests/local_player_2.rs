use super::*;

#[test]
fn local_player_sleeping_position_follows_entity_data_keep_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let bed = voxel_core::binary::BlockPos::new(14, 90, -23);
    let mut sleeping_metadata = vec![
        crate::ENTITY_SLEEPING_POS_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID as u8,
        1,
    ];
    voxel_core::binary::write_blockpos(bed, &mut sleeping_metadata);
    sleeping_metadata.push(255);
    assert!(world.apply_entity_metadata(42, &sleeping_metadata));
    assert_eq!(
        world
            .local_player()
            .expect("sleeping player")
            .sleeping_pos(),
        Some(bed)
    );
    assert!(world.local_player().expect("sleeping player").is_sleeping());

    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
    assert_eq!(
        world
            .local_player()
            .expect("respawned sleeping player")
            .sleeping_pos(),
        Some(bed)
    );

    assert!(world.recreate_local_player(0));
    assert_eq!(
        world
            .local_player()
            .expect("reset sleeping player")
            .sleeping_pos(),
        None
    );
}

#[test]
fn local_player_living_flags_follow_metadata_and_respawn_keep_mask() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert_eq!(
        world.local_player().map(|player| player.living_flags()),
        Some(0)
    );

    assert!(world.apply_entity_metadata(
        42,
        &[
            crate::ENTITY_LIVING_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0b111,
            255,
        ],
    ));
    let player = world.local_player().expect("living flags player");
    assert_eq!(player.living_flags(), 0b111);
    assert!(player.is_using_item());
    assert!(player.used_item_is_off_hand());
    assert!(player.is_auto_spin_attack());

    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
    let player = world.local_player().expect("respawned living flags player");
    assert_eq!(player.living_flags(), 0b111);
    assert!(player.is_using_item());

    assert!(world.recreate_local_player(0));
    let player = world.local_player().expect("reset living flags player");
    assert_eq!(player.living_flags(), 0);
    assert!(!player.is_using_item());
    assert!(!player.used_item_is_off_hand());
    assert!(!player.is_auto_spin_attack());
}

#[test]
fn local_player_respawn_recreates_identity_and_applies_entity_data_mask() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert_eq!(world.local_player().map(|player| player.id()), Some(42));
    assert_eq!(
        world.local_player().map(|player| player.generation()),
        Some(0)
    );

    assert!(world.apply_local_player_position(
        (10.5, 64.0, -3.25),
        (0.125, -0.25, 0.5),
        (135.0, -30.0),
    ));
    assert!(world.apply_entity_metadata(42, &[0, 0, 32, 255]));
    let mut air_metadata = vec![1, 1];
    voxel_core::wire::write_varint(250, &mut air_metadata);
    air_metadata.push(255);
    assert!(world.apply_entity_metadata(42, &air_metadata));
    let no_gravity_metadata = [5, 8, 1, 255];
    assert!(world.apply_entity_metadata(42, &no_gravity_metadata));
    let silent_metadata = [4, 8, 1, 255];
    assert!(world.apply_entity_metadata(42, &silent_metadata));
    let mut custom_name_metadata = vec![2, 6, 1];
    voxel_core::nbt::write_any_nbt(
        Some(&voxel_core::nbt::FieldValue::Str("Player name".to_owned())),
        &mut custom_name_metadata,
    );
    custom_name_metadata.push(255);
    assert!(world.apply_entity_metadata(42, &custom_name_metadata));
    let custom_name_visible_metadata = [3, 8, 1, 255];
    assert!(world.apply_entity_metadata(42, &custom_name_visible_metadata));
    let pose_metadata = [6, 20, 10, 255];
    assert!(world.apply_entity_metadata(42, &pose_metadata));
    let ticks_frozen_metadata = [7, 1, 70, 255];
    assert!(world.apply_entity_metadata(42, &ticks_frozen_metadata));

    assert!(world.recreate_local_player(
        PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS | PLAYER_RESPAWN_KEEP_ENTITY_DATA
    ));
    let player = world.local_player().expect("respawned local player");
    assert_eq!(player.id(), 42);
    assert_eq!(player.generation(), 1);
    assert_eq!(player.position(), (0.0, 0.0, 0.0));
    assert_eq!(player.motion(), (0.125, -0.25, 0.5));
    assert_eq!(player.rotation(), (135.0, -30.0));
    assert_eq!(player.entity_data(), Some(ticks_frozen_metadata.as_slice()));
    assert_eq!(player.shared_flags(), 32);
    assert!(player.is_invisible());
    assert_eq!(player.air_supply(), 250);
    assert!(player.is_no_gravity());
    assert!(player.is_silent());
    assert!(player.has_custom_name());
    assert!(player.is_custom_name_visible());
    assert!(player.should_show_name());
    assert_eq!(player.pose(), crate::ClientEntityPose::Sitting);
    assert_eq!(player.ticks_frozen(), 70);
    assert!(player.is_freezing());

    assert!(world.recreate_local_player(0));
    let player = world.local_player().expect("reset local player");
    assert_eq!(player.id(), 42);
    assert_eq!(player.generation(), 2);
    assert_eq!(player.motion(), (0.0, 0.0, 0.0));
    assert_eq!(player.rotation(), (-180.0, 0.0));
    assert_eq!(player.entity_data(), None);
    assert_eq!(player.shared_flags(), 0);
    assert!(!player.is_invisible());
    assert_eq!(player.air_supply(), 300);
    assert!(!player.is_no_gravity());
    assert!(!player.is_silent());
    assert!(!player.has_custom_name());
    assert!(!player.is_custom_name_visible());
    assert!(player.should_show_name());
    assert_eq!(player.pose(), crate::ClientEntityPose::Standing);
    assert_eq!(player.ticks_frozen(), 0);
    assert!(!player.is_freezing());
}

#[test]
fn local_player_respawn_preserves_attribute_bases_and_applies_modifier_mask() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let modifier =
        ClientAttributeModifierState::from_wire("minecraft:test_modifier".to_owned(), 0.25, 2);
    let attribute = ClientAttributeState::from_wire(17, 2.5, vec![modifier.clone()]);

    assert!(world.apply_local_player_attributes(42, &[attribute]));
    let player = world.local_player().expect("initial local player");
    assert_eq!(player.attributes().len(), 1);
    assert_eq!(player.attributes()[0].base_value(), 2.5);
    assert_eq!(
        player.attributes()[0].modifiers(),
        std::slice::from_ref(&modifier)
    );

    assert!(world.recreate_local_player(0));
    let player = world.local_player().expect("base-only respawned player");
    assert_eq!(player.attributes().len(), 1);
    assert_eq!(player.attributes()[0].base_value(), 2.5);
    assert!(player.attributes()[0].modifiers().is_empty());

    assert!(world.apply_local_player_attributes(
        42,
        &[ClientAttributeState::from_wire(
            17,
            3.5,
            vec![modifier.clone()],
        )],
    ));
    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS));
    let player = world.local_player().expect("modifier-preserving player");
    assert_eq!(player.attributes()[0].base_value(), 3.5);
    assert_eq!(player.attributes()[0].modifiers(), &[modifier]);
}

#[test]
fn local_player_mob_effects_replace_remove_and_clear_on_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);

    assert!(world.apply_local_player_mob_effect(42, 18, 0, 1_200, 0b1000));
    let player = world.local_player().expect("initial local player");
    assert_eq!(player.effects().len(), 1);
    assert_eq!(player.effects()[0].effect_id(), 18);
    assert_eq!(player.effects()[0].amplifier(), 0);
    assert_eq!(player.effects()[0].duration_ticks(), 1_200);
    assert_eq!(player.effects()[0].flags(), 0b1000);

    assert!(world.apply_local_player_mob_effect(42, 18, 1, 60, 0b1111));
    let player = world.local_player().expect("updated local player");
    assert_eq!(player.effects().len(), 1);
    assert_eq!(player.effects()[0].amplifier(), 1);
    assert_eq!(player.effects()[0].duration_ticks(), 60);
    assert_eq!(player.effects()[0].flags(), 0b1111);

    assert!(!world.apply_local_player_mob_effect(7, 19, 0, 20, 0));
    assert!(world.remove_local_player_mob_effect(42, 18));
    assert!(
        world
            .local_player()
            .expect("local player after removal")
            .effects()
            .is_empty()
    );
    assert!(world.apply_local_player_mob_effect(42, 18, 0, 1_200, 0b1000));
    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS));
    assert!(
        world
            .local_player()
            .expect("respawned local player")
            .effects()
            .is_empty()
    );
}

#[test]
fn local_player_mob_effect_durations_tick_without_removing_expired_entries() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert!(world.apply_local_player_mob_effect(42, 18, 0, 2, 0));
    assert!(world.apply_local_player_mob_effect(42, 19, 0, -1, 0));
    assert!(world.apply_local_player_mob_effect(42, 20, 0, 0, 0));
    assert!(world.apply_local_player_mob_effect(42, 21, 0, -2, 0));

    let player = world.local_player().expect("local player effects");
    assert!(
        player
            .effects()
            .iter()
            .find(|effect| effect.effect_id() == 18)
            .is_some_and(|effect| effect.has_remaining_duration())
    );
    assert!(
        player
            .effects()
            .iter()
            .find(|effect| effect.effect_id() == 19)
            .is_some_and(|effect| effect.has_remaining_duration())
    );
    assert!(
        player
            .effects()
            .iter()
            .find(|effect| effect.effect_id() == 20)
            .is_some_and(|effect| !effect.has_remaining_duration())
    );

    world.tick();
    let player = world
        .local_player()
        .expect("local player after first effect tick");
    assert_eq!(
        player
            .effects()
            .iter()
            .find(|effect| effect.effect_id() == 18)
            .map(|effect| effect.duration_ticks()),
        Some(1)
    );
    assert_eq!(
        player
            .effects()
            .iter()
            .find(|effect| effect.effect_id() == 19)
            .map(|effect| effect.duration_ticks()),
        Some(-1)
    );

    world.tick();
    let player = world
        .local_player()
        .expect("local player after effect expiry");
    assert_eq!(player.effects().len(), 4);
    assert_eq!(
        player
            .effects()
            .iter()
            .find(|effect| effect.effect_id() == 18)
            .map(|effect| effect.duration_ticks()),
        Some(0)
    );
    assert!(
        player
            .effects()
            .iter()
            .all(|effect| effect.effect_id() != 18 || !effect.has_remaining_duration())
    );

    world.tick();
    let player = world
        .local_player()
        .expect("local player after expired effect tick");
    assert_eq!(
        player
            .effects()
            .iter()
            .find(|effect| effect.effect_id() == 18)
            .map(|effect| effect.duration_ticks()),
        Some(0)
    );
}

#[test]
fn local_player_health_state_applies_and_resets_with_new_player() {
    let mut world = ClientWorld::default();
    assert!(!world.apply_local_player_health(0.0, 0, 0.0));
    world.set_local_player_id(42);
    let player = world.local_player().expect("initial local player");
    assert_eq!(player.health(), 20.0);
    assert_eq!(player.food(), 20);
    assert_eq!(player.saturation(), 5.0);

    assert!(world.apply_local_player_health(0.0, 7, 1.5));
    let player = world.local_player().expect("updated local player");
    assert_eq!(player.health(), 0.0);
    assert_eq!(player.food(), 7);
    assert_eq!(player.saturation(), 1.5);

    assert!(world.recreate_local_player(0));
    let player = world.local_player().expect("respawned local player");
    assert_eq!(player.health(), 20.0);
    assert_eq!(player.food(), 20);
    assert_eq!(player.saturation(), 5.0);
}

#[test]
fn local_player_inventory_state_applies_slot_cursor_and_respawn_reset() {
    let mut world = ClientWorld::default();
    let stone = ClientItemStackState::from_wire(8, 1, vec![], vec![]);
    let bottle = ClientItemStackState::from_wire(1, 1_149, vec![], vec![]);
    let mut items = vec![None; 46];
    items[36] = Some(stone.clone());

    assert!(!world.apply_local_player_container_content(0, 1, &items, Some(&stone),));
    world.set_local_player_id(42);
    assert!(world.apply_local_player_container_content(0, 2, &items, Some(&stone),));
    let player = world.local_player().expect("local player inventory");
    assert_eq!(player.inventory_state_id(), 2);
    assert_eq!(player.inventory_slot_count(), 46);
    assert_eq!(player.inventory_slot(36), Some(&stone));
    assert_eq!(player.carried_item(), Some(&stone));

    assert!(world.apply_local_player_container_slot(0, 3, 36, Some(&bottle)));
    assert_eq!(
        world
            .local_player()
            .expect("local player after slot update")
            .inventory_slot(36),
        Some(&bottle)
    );
    assert_eq!(
        world
            .local_player()
            .expect("local player after slot update")
            .inventory_state_id(),
        3
    );

    assert!(world.apply_local_player_carried_item(None));
    assert!(
        world
            .local_player()
            .expect("local player after cursor clear")
            .carried_item()
            .is_none()
    );

    assert!(world.apply_local_player_inventory_slot(0, Some(&stone)));
    assert_eq!(
        world
            .local_player()
            .expect("local player after direct inventory update")
            .inventory_slot(36),
        Some(&stone)
    );
    assert!(!world.apply_local_player_inventory_slot(36, Some(&bottle)));

    assert!(world.recreate_local_player(0));
    let player = world
        .local_player()
        .expect("respawned local player inventory");
    assert_eq!(player.inventory_state_id(), 0);
    assert_eq!(player.inventory_slot_count(), 0);
    assert!(player.inventory_slot(36).is_none());
    assert!(player.carried_item().is_none());
}

#[test]
fn local_player_container_state_applies_open_content_slot_cursor_close_and_reset() {
    let mut world = ClientWorld::default();
    let stone = ClientItemStackState::from_wire(7, 1, vec![], vec![]);
    let bottle = ClientItemStackState::from_wire(1, 1_149, vec![], vec![]);
    let title = [10, 0, 0, 0];
    let mut items = vec![None; 63];
    items[0] = Some(stone.clone());
    items[27] = Some(bottle.clone());

    assert!(!world.apply_local_player_open_menu(7, 2, &title));
    world.set_local_player_id(42);
    assert!(world.apply_local_player_open_menu(7, 2, &title));
    assert!(world.apply_local_player_container_content(7, 4, &items, None));
    let container = world
        .local_player()
        .expect("local player with open container")
        .open_container()
        .expect("open container");
    assert_eq!(container.container_id(), 7);
    assert_eq!(container.menu_type_id(), 2);
    assert_eq!(container.title_nbt(), title);
    assert_eq!(container.top_slot_count(), Some(27));
    assert_eq!(container.slot_count(), 63);
    assert_eq!(container.state_id(), 4);
    assert_eq!(container.slot(0), Some(&stone));
    assert_eq!(container.slot(27), Some(&bottle));
    assert!(container.carried_item().is_none());

    assert!(world.apply_local_player_container_slot(7, 5, 0, None));
    assert!(
        world
            .local_player()
            .expect("local player after custom slot update")
            .open_container()
            .expect("container after slot update")
            .slot(0)
            .is_none()
    );
    assert!(world.apply_local_player_carried_item(Some(&stone)));
    assert_eq!(
        world
            .local_player()
            .expect("local player after custom cursor update")
            .open_container()
            .expect("container after cursor update")
            .carried_item(),
        Some(&stone)
    );
    assert!(world.apply_local_player_container_close(99));
    assert!(
        world
            .local_player()
            .expect("local player after custom container close")
            .open_container()
            .is_none()
    );
    assert!(!world.apply_local_player_container_close(7));

    assert!(world.apply_local_player_open_menu(7, 2, &title));
    assert!(world.recreate_local_player(0));
    let player = world
        .local_player()
        .expect("respawned local player without container");
    assert!(player.open_container().is_none());
}
