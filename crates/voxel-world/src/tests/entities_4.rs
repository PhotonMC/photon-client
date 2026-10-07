use super::*;

#[test]
fn entity_arrow_count_follows_java_int_metadata_and_preserves_partial_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 54,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::arrow_count),
        Some(0)
    );

    let mut arrow_metadata = vec![
        crate::ENTITY_ARROW_COUNT_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    voxel_core::wire::write_varint(3, &mut arrow_metadata);
    arrow_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &arrow_metadata));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::arrow_count),
        Some(3)
    );

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::arrow_count),
        Some(3)
    );

    assert!(world.apply_entity_metadata(37, &[12, 0, 7, 255]));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::arrow_count),
        Some(3)
    );
}

#[test]
fn entity_stinger_count_follows_java_int_metadata_and_preserves_partial_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 54,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::stinger_count),
        Some(0)
    );

    let mut stinger_metadata = vec![
        crate::ENTITY_STINGER_COUNT_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    voxel_core::wire::write_varint(2, &mut stinger_metadata);
    stinger_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &stinger_metadata));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::stinger_count),
        Some(2)
    );

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::stinger_count),
        Some(2)
    );

    assert!(world.apply_entity_metadata(37, &[13, 0, 4, 255]));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::stinger_count),
        Some(2)
    );
}

#[test]
fn entity_effect_ambience_follows_java_boolean_metadata_with_empty_particles() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 54,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(!world.entity_at(37).expect("entity").is_effect_ambient());

    assert!(world.apply_entity_metadata(37, &[10, 17, 0, 11, 8, 1, 255]));
    assert!(
        world
            .entity_at(37)
            .expect("ambient entity")
            .is_effect_ambient()
    );

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    assert!(
        world
            .entity_at(37)
            .expect("ambient entity after partial metadata")
            .is_effect_ambient()
    );

    assert!(world.apply_entity_metadata(37, &[11, 0, 0, 255]));
    assert!(
        world
            .entity_at(37)
            .expect("ambient entity after wrong serializer")
            .is_effect_ambient()
    );

    assert!(world.apply_entity_metadata(37, &[10, 17, 1, 0, 11, 8, 0, 255]));
    assert!(
        world
            .entity_at(37)
            .expect("ambient entity after unsupported particles")
            .is_effect_ambient()
    );
}

#[test]
fn entity_effect_particle_types_follow_java_particle_list_and_preserve_partial_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 54,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(
        world
            .entity_at(37)
            .expect("entity")
            .effect_particle_type_ids()
            .is_empty()
    );

    let mut particle_metadata = vec![
        crate::ENTITY_EFFECT_PARTICLES_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_PARTICLES_ID as u8,
        1,
    ];
    voxel_core::wire::write_varint(
        crate::ENTITY_EFFECT_PARTICLE_TYPE_ID,
        &mut particle_metadata,
    );
    particle_metadata.extend_from_slice(&0x1122_3344_i32.to_be_bytes());
    particle_metadata.extend([11, 8, 0, 255]);
    assert!(world.apply_entity_metadata(37, &particle_metadata));
    assert_eq!(
        world
            .entity_at(37)
            .expect("entity with effect particle")
            .effect_particle_type_ids(),
        &[crate::ENTITY_EFFECT_PARTICLE_TYPE_ID]
    );

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    assert_eq!(
        world
            .entity_at(37)
            .expect("entity after partial particle metadata")
            .effect_particle_type_ids(),
        &[crate::ENTITY_EFFECT_PARTICLE_TYPE_ID]
    );

    assert!(world.apply_entity_metadata(37, &[10, 0, 1, 255]));
    assert_eq!(
        world
            .entity_at(37)
            .expect("entity after wrong particle serializer")
            .effect_particle_type_ids(),
        &[crate::ENTITY_EFFECT_PARTICLE_TYPE_ID]
    );

    assert!(world.apply_entity_metadata(37, &[10, 17, 1, 0, 255]));
    assert_eq!(
        world
            .entity_at(37)
            .expect("entity after unsupported particle type")
            .effect_particle_type_ids(),
        &[crate::ENTITY_EFFECT_PARTICLE_TYPE_ID]
    );
}

#[test]
fn entity_sleeping_position_follows_java_optional_block_pos_metadata() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 54,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let entity = world.entity_at(37).expect("default sleeping entity");
    assert_eq!(entity.sleeping_pos(), None);
    assert!(!entity.is_sleeping());

    let bed = voxel_core::binary::BlockPos::new(-12, 90, 37);
    let mut sleeping_metadata = vec![
        crate::ENTITY_SLEEPING_POS_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID as u8,
        1,
    ];
    voxel_core::binary::write_blockpos(bed, &mut sleeping_metadata);
    sleeping_metadata.extend([3, 8, 1, 255]);
    assert!(world.apply_entity_metadata(37, &sleeping_metadata));
    let entity = world.entity_at(37).expect("sleeping entity");
    assert_eq!(entity.sleeping_pos(), Some(bed));
    assert!(entity.is_sleeping());
    assert!(entity.is_custom_name_visible());

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    assert_eq!(
        world
            .entity_at(37)
            .expect("sleeping entity after partial metadata")
            .sleeping_pos(),
        Some(bed)
    );

    assert!(world.apply_entity_metadata(37, &[14, 11, 0, 255]));
    assert_eq!(
        world.entity_at(37).expect("awake entity").sleeping_pos(),
        None
    );

    let mut wrong_serializer = vec![14, 10, 1];
    voxel_core::binary::write_blockpos(bed, &mut wrong_serializer);
    wrong_serializer.push(255);
    assert!(world.apply_entity_metadata(37, &wrong_serializer));
    assert_eq!(
        world
            .entity_at(37)
            .expect("entity after wrong serializer")
            .sleeping_pos(),
        None
    );

    assert!(world.apply_entity_metadata(37, &[14, 11, 1, 0, 255]));
    assert_eq!(
        world
            .entity_at(37)
            .expect("entity after truncated sleeping position")
            .sleeping_pos(),
        None
    );
}

#[test]
fn entity_living_flags_follow_java_byte_metadata_masks() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 54,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let entity = world.entity_at(37).expect("default living flags entity");
    assert_eq!(entity.living_flags(), 0);
    assert!(!entity.is_using_item());
    assert!(!entity.used_item_is_off_hand());
    assert!(!entity.is_auto_spin_attack());

    assert!(world.apply_entity_metadata(
        37,
        &[
            crate::ENTITY_LIVING_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0b111,
            255,
        ],
    ));
    let entity = world.entity_at(37).expect("all living flags entity");
    assert_eq!(entity.living_flags(), 0b111);
    assert!(entity.is_using_item());
    assert!(entity.used_item_is_off_hand());
    assert!(entity.is_auto_spin_attack());

    assert!(world.apply_entity_metadata(
        37,
        &[
            crate::ENTITY_LIVING_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0b010,
            255,
        ],
    ));
    let entity = world.entity_at(37).expect("off-hand living flags entity");
    assert_eq!(entity.living_flags(), 0b010);
    assert!(!entity.is_using_item());
    assert!(entity.used_item_is_off_hand());
    assert!(!entity.is_auto_spin_attack());

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    let entity = world
        .entity_at(37)
        .expect("living flags after partial metadata");
    assert_eq!(entity.living_flags(), 0b010);
    assert!(entity.is_no_gravity());
}

#[test]
fn mob_flags_follow_java_byte_metadata_masks_and_preserve_partial_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 70,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let entity = world.entity_at(37).expect("default mob flags entity");
    assert_eq!(entity.mob_flags(), 0);
    assert!(!entity.is_no_ai());
    assert!(!entity.is_left_handed());
    assert!(!entity.is_aggressive());

    assert!(world.apply_entity_metadata(
        37,
        &[
            crate::ENTITY_MOB_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0b111,
            255,
        ],
    ));
    let entity = world.entity_at(37).expect("all mob flags entity");
    assert_eq!(entity.mob_flags(), 0b111);
    assert!(entity.is_no_ai());
    assert!(entity.is_left_handed());
    assert!(entity.is_aggressive());

    assert!(world.apply_entity_metadata(
        37,
        &[
            crate::ENTITY_MOB_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0b001,
            255,
        ],
    ));
    let entity = world.entity_at(37).expect("no-ai mob flags entity");
    assert_eq!(entity.mob_flags(), 0b001);
    assert!(entity.is_no_ai());
    assert!(!entity.is_left_handed());
    assert!(!entity.is_aggressive());

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    let entity = world
        .entity_at(37)
        .expect("mob flags after partial metadata");
    assert_eq!(entity.mob_flags(), 0b001);
    assert!(entity.is_no_ai());
    assert!(entity.is_no_gravity());

    assert!(world.apply_entity_metadata(37, &[15, 8, 0, 255]));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::mob_flags),
        Some(0b001)
    );
}
