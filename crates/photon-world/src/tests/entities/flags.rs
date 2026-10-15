use super::super::*;
use photon_core::ids::EntityId;

#[test]
fn entity_shared_flags_follow_java_byte_metadata_and_preserve_unspecified_values() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: 120,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let all_defined_flags = 0b1111_1011;
    assert!(world.apply_entity_metadata(
        EntityId(37),
        &[
            crate::ENTITY_SHARED_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            all_defined_flags,
            255,
        ],
    ));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("entity with shared flags");
    assert_eq!(entity.shared_flags(), all_defined_flags);
    assert!(entity.is_on_fire());
    assert!(entity.is_shift_key_down());
    assert!(entity.is_sprinting());
    assert!(entity.is_swimming());
    assert!(entity.is_invisible());
    assert!(entity.is_glowing());
    assert!(entity.is_fall_flying());

    assert!(world.apply_entity_metadata(EntityId(37), &[1, 0, 7, 255]));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::shared_flags),
        Some(all_defined_flags)
    );

    assert!(world.apply_entity_metadata(EntityId(37), &[0, 0, 0, 255]));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("cleared shared flags");
    assert_eq!(entity.shared_flags(), 0);
    assert!(!entity.is_on_fire());
    assert!(!entity.is_shift_key_down());
    assert!(!entity.is_sprinting());
    assert!(!entity.is_swimming());
    assert!(!entity.is_invisible());
    assert!(!entity.is_glowing());
    assert!(!entity.is_fall_flying());
}

#[test]
fn entity_no_gravity_follows_java_boolean_metadata_and_preserves_air_supply() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: 78,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let mut no_gravity_metadata = vec![
        crate::ENTITY_NO_GRAVITY_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
        1,
        255,
    ];
    assert!(world.apply_entity_metadata(EntityId(37), &no_gravity_metadata));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("no-gravity entity");
    assert!(entity.is_no_gravity());
    assert_eq!(entity.air_supply(), 300);

    no_gravity_metadata = vec![
        crate::ENTITY_SHARED_FLAGS_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
        64,
        crate::ENTITY_NO_GRAVITY_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
        0,
        255,
    ];
    assert!(world.apply_entity_metadata(EntityId(37), &no_gravity_metadata));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("cleared no-gravity entity");
    assert!(!entity.is_no_gravity());
    assert_eq!(entity.shared_flags(), 64);
    assert!(entity.is_glowing());

    let mut air_metadata = vec![1, 1];
    photon_core::wire::write_varint(250, &mut air_metadata);
    air_metadata.push(255);
    assert!(world.apply_entity_metadata(EntityId(37), &air_metadata));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("air supply entity");
    assert_eq!(entity.air_supply(), 250);
    assert!(!entity.is_no_gravity());
}

#[test]
fn entity_silent_follows_java_boolean_metadata_and_preserves_other_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: 5,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(
        !world
            .entities()
            .at(EntityId(37))
            .expect("entity")
            .is_silent()
    );

    let metadata = [4, 8, 1, 255];
    assert!(world.apply_entity_metadata(EntityId(37), &metadata));
    let entity = world.entities().at(EntityId(37)).expect("silent entity");
    assert!(entity.is_silent());
    assert!(!entity.is_no_gravity());
    assert_eq!(entity.air_supply(), 300);

    assert!(world.apply_entity_metadata(EntityId(37), &[5, 8, 1, 255]));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("silent no-gravity entity");
    assert!(entity.is_silent());
    assert!(entity.is_no_gravity());

    assert!(world.apply_entity_metadata(EntityId(37), &[4, 8, 0, 255]));
    let entity = world.entities().at(EntityId(37)).expect("audible entity");
    assert!(!entity.is_silent());
    assert!(entity.is_no_gravity());
}

#[test]
fn entity_custom_name_visibility_follows_java_boolean_metadata() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: 5,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("default name visibility entity");
    assert!(!entity.is_custom_name_visible());
    assert!(!entity.should_show_name());

    let mut visible_metadata = vec![
        2,
        crate::ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID as u8,
        1,
    ];
    photon_core::nbt::write_any_nbt(
        Some(&photon_core::nbt::FieldValue::Str(
            "Display name".to_owned(),
        )),
        &mut visible_metadata,
    );
    visible_metadata.extend([
        crate::ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
        1,
        255,
    ]);
    assert!(world.apply_entity_metadata(EntityId(37), &visible_metadata));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("visible name entity");
    assert!(entity.is_custom_name_visible());
    assert!(entity.should_show_name());

    assert!(world.apply_entity_metadata(
        EntityId(37),
        &[
            crate::ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
            0,
            255,
        ],
    ));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("hidden name entity");
    assert!(!entity.is_custom_name_visible());
    assert!(!entity.should_show_name());
}

#[test]
fn entity_custom_name_presence_follows_java_optional_component_metadata() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: 5,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("default custom-name entity");
    assert!(!entity.has_custom_name());
    assert!(!entity.is_custom_name_visible());

    let mut custom_name_metadata = vec![
        crate::ENTITY_CUSTOM_NAME_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID as u8,
        1,
    ];
    photon_core::nbt::write_any_nbt(
        Some(&photon_core::nbt::FieldValue::Str(
            "Display name".to_owned(),
        )),
        &mut custom_name_metadata,
    );
    custom_name_metadata.extend([
        crate::ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
        1,
        255,
    ]);
    assert!(world.apply_entity_metadata(EntityId(37), &custom_name_metadata));
    let entity = world.entities().at(EntityId(37)).expect("named entity");
    assert!(entity.has_custom_name());
    assert!(entity.is_custom_name_visible());
    assert!(entity.should_show_name());

    assert!(world.apply_entity_metadata(
        EntityId(37),
        &[
            crate::ENTITY_CUSTOM_NAME_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID as u8,
            0,
            255,
        ],
    ));
    let entity = world.entities().at(EntityId(37)).expect("unnamed entity");
    assert!(!entity.has_custom_name());
    assert!(entity.is_custom_name_visible());
    assert!(entity.should_show_name());
}
