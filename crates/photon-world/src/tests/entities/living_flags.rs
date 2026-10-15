use super::super::*;
use photon_core::ids::EntityId;

#[test]
fn entity_living_flags_follow_java_byte_metadata_masks() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: photon_core::ids::EntityTypeId(54),
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
        .expect("default living flags entity");
    assert_eq!(entity.living_flags(), 0);
    assert!(!entity.is_using_item());
    assert!(!entity.used_item_is_off_hand());
    assert!(!entity.is_auto_spin_attack());

    assert!(world.apply_entity_metadata(
        EntityId(37),
        &[
            crate::ENTITY_LIVING_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0b111,
            255,
        ],
    ));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("all living flags entity");
    assert_eq!(entity.living_flags(), 0b111);
    assert!(entity.is_using_item());
    assert!(entity.used_item_is_off_hand());
    assert!(entity.is_auto_spin_attack());

    assert!(world.apply_entity_metadata(
        EntityId(37),
        &[
            crate::ENTITY_LIVING_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0b010,
            255,
        ],
    ));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("off-hand living flags entity");
    assert_eq!(entity.living_flags(), 0b010);
    assert!(!entity.is_using_item());
    assert!(entity.used_item_is_off_hand());
    assert!(!entity.is_auto_spin_attack());

    assert!(world.apply_entity_metadata(EntityId(37), &[5, 8, 1, 255]));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("living flags after partial metadata");
    assert_eq!(entity.living_flags(), 0b010);
    assert!(entity.is_no_gravity());
}

#[test]
fn mob_flags_follow_java_byte_metadata_masks_and_preserve_partial_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: photon_core::ids::EntityTypeId(70),
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
        .expect("default mob flags entity");
    assert_eq!(entity.mob_flags(), 0);
    assert!(!entity.is_no_ai());
    assert!(!entity.is_left_handed());
    assert!(!entity.is_aggressive());

    assert!(world.apply_entity_metadata(
        EntityId(37),
        &[
            crate::ENTITY_MOB_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0b111,
            255,
        ],
    ));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("all mob flags entity");
    assert_eq!(entity.mob_flags(), 0b111);
    assert!(entity.is_no_ai());
    assert!(entity.is_left_handed());
    assert!(entity.is_aggressive());

    assert!(world.apply_entity_metadata(
        EntityId(37),
        &[
            crate::ENTITY_MOB_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0b001,
            255,
        ],
    ));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("no-ai mob flags entity");
    assert_eq!(entity.mob_flags(), 0b001);
    assert!(entity.is_no_ai());
    assert!(!entity.is_left_handed());
    assert!(!entity.is_aggressive());

    assert!(world.apply_entity_metadata(EntityId(37), &[5, 8, 1, 255]));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("mob flags after partial metadata");
    assert_eq!(entity.mob_flags(), 0b001);
    assert!(entity.is_no_ai());
    assert!(entity.is_no_gravity());

    assert!(world.apply_entity_metadata(EntityId(37), &[15, 8, 0, 255]));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::mob_flags),
        Some(0b001)
    );
}

#[test]
fn entity_pose_follows_java_pose_metadata_and_maps_unknown_ids_to_standing() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::pose),
        Some(crate::ClientEntityPose::Standing)
    );

    let expected_poses = [
        crate::ClientEntityPose::Standing,
        crate::ClientEntityPose::FallFlying,
        crate::ClientEntityPose::Sleeping,
        crate::ClientEntityPose::Swimming,
        crate::ClientEntityPose::SpinAttack,
        crate::ClientEntityPose::Crouching,
        crate::ClientEntityPose::LongJumping,
        crate::ClientEntityPose::Dying,
        crate::ClientEntityPose::Croaking,
        crate::ClientEntityPose::UsingTongue,
        crate::ClientEntityPose::Sitting,
        crate::ClientEntityPose::Roaring,
        crate::ClientEntityPose::Sniffing,
        crate::ClientEntityPose::Emerging,
        crate::ClientEntityPose::Digging,
        crate::ClientEntityPose::Sliding,
        crate::ClientEntityPose::Shooting,
        crate::ClientEntityPose::Inhaling,
    ];
    for (id, expected) in expected_poses.into_iter().enumerate() {
        let mut metadata = vec![
            crate::ENTITY_POSE_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_POSE_ID as u8,
        ];
        photon_core::wire::write_varint(id as i32, &mut metadata);
        metadata.push(255);
        assert!(world.apply_entity_metadata(EntityId(37), &metadata));
        let entity = world.entities().at(EntityId(37)).expect("entity with pose");
        assert_eq!(entity.pose(), expected);
        assert_eq!(entity.pose().id(), id as u8);
        assert_eq!(
            entity.player_dimensions(),
            Some(expected.player_dimensions())
        );
    }

    let combined_metadata = [0, 0, 64, 5, 8, 1, 6, 20, 10, 255];
    assert!(world.apply_entity_metadata(EntityId(37), &combined_metadata));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("combined pose entity");
    assert_eq!(entity.pose(), crate::ClientEntityPose::Sitting);
    assert!(entity.is_glowing());
    assert!(entity.is_no_gravity());

    assert!(world.apply_entity_metadata(EntityId(37), &[6, 20, 99, 255]));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("fallback pose entity");
    assert_eq!(entity.pose(), crate::ClientEntityPose::Standing);
    assert_eq!(
        entity.player_dimensions(),
        Some(crate::ClientEntityPose::Standing.player_dimensions())
    );
    assert!(entity.is_glowing());
    assert!(entity.is_no_gravity());
}

#[test]
fn entity_ticks_frozen_follows_java_int_metadata_and_derived_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: photon_core::ids::EntityTypeId(1),
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
        .expect("default freeze entity");
    assert_eq!(entity.ticks_frozen(), 0);
    assert_eq!(entity.percent_frozen(), 0.0);
    assert!(!entity.is_freezing());
    assert!(!entity.is_fully_frozen());

    let mut partial_metadata = vec![
        crate::ENTITY_TICKS_FROZEN_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    photon_core::wire::write_varint(70, &mut partial_metadata);
    partial_metadata.push(255);
    assert!(world.apply_entity_metadata(EntityId(37), &partial_metadata));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("partially frozen entity");
    assert_eq!(entity.ticks_frozen(), 70);
    assert_eq!(entity.percent_frozen(), 0.5);
    assert!(entity.is_freezing());
    assert!(!entity.is_fully_frozen());

    let mut combined_metadata = vec![0, 0, 64, 6, 20, 10, 7, 1];
    photon_core::wire::write_varint(140, &mut combined_metadata);
    combined_metadata.push(255);
    assert!(world.apply_entity_metadata(EntityId(37), &combined_metadata));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("fully frozen entity");
    assert_eq!(entity.ticks_frozen(), 140);
    assert_eq!(entity.percent_frozen(), 1.0);
    assert!(entity.is_freezing());
    assert!(entity.is_fully_frozen());
    assert_eq!(entity.pose(), crate::ClientEntityPose::Sitting);
    assert!(entity.is_glowing());

    let mut negative_metadata = vec![
        crate::ENTITY_TICKS_FROZEN_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    photon_core::wire::write_varint(-1, &mut negative_metadata);
    negative_metadata.push(255);
    assert!(world.apply_entity_metadata(EntityId(37), &negative_metadata));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("negative freeze entity");
    assert_eq!(entity.ticks_frozen(), -1);
    assert_eq!(entity.percent_frozen(), -1.0 / 140.0);
    assert!(!entity.is_freezing());
    assert!(!entity.is_fully_frozen());
    assert_eq!(entity.pose(), crate::ClientEntityPose::Sitting);
}
