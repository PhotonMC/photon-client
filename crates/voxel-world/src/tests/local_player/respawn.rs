use super::super::*;

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
fn local_player_health_metadata_updates_and_respawn_resets_health() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);

    let mut health_metadata = vec![
        8,
        crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
        0,
        crate::ENTITY_HEALTH_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_FLOAT_ID as u8,
    ];
    health_metadata.extend_from_slice(&7.5_f32.to_be_bytes());
    health_metadata.push(255);
    assert!(world.apply_entity_metadata(42, &health_metadata));
    assert_eq!(
        world.local_player().map(|player| player.health()),
        Some(7.5)
    );

    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
    assert_eq!(
        world.local_player().map(|player| player.health()),
        Some(20.0)
    );
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
