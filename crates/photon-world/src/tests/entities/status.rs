use super::super::*;
use photon_core::ids::EntityId;

#[test]
fn entity_air_supply_follows_java_int_metadata_and_preserves_shared_flags() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: photon_data::entity_type::named("minecraft:snowball"),
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
            .map(crate::ClientEntity::air_supply),
        Some(300)
    );

    let mut air_metadata = vec![
        crate::ENTITY_AIR_SUPPLY_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    photon_core::wire::write_varint(250, &mut air_metadata);
    air_metadata.push(255);
    assert!(world.apply_entity_metadata(EntityId(37), &air_metadata));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("entity with air supply");
    assert_eq!(entity.air_supply(), 250);
    assert_eq!(entity.shared_flags(), 0);

    let mut combined_metadata = vec![
        crate::ENTITY_SHARED_FLAGS_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
        64,
        crate::ENTITY_AIR_SUPPLY_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    photon_core::wire::write_varint(-1, &mut combined_metadata);
    combined_metadata.push(255);
    assert!(world.apply_entity_metadata(EntityId(37), &combined_metadata));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("entity with combined metadata");
    assert_eq!(entity.air_supply(), -1);
    assert_eq!(entity.shared_flags(), 64);
    assert!(entity.is_glowing());

    assert!(world.apply_entity_metadata(EntityId(37), &[0, 0, 0, 255]));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::air_supply),
        Some(-1)
    );
}

#[test]
fn entity_health_follows_java_float_metadata_and_preserves_partial_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: photon_data::entity_type::named("minecraft:fox"),
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
            .and_then(crate::ClientEntity::health),
        None
    );

    let mut health_metadata = vec![
        8,
        crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
        0,
        crate::ENTITY_HEALTH_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_FLOAT_ID as u8,
    ];
    health_metadata.extend_from_slice(&7.5_f32.to_be_bytes());
    health_metadata.push(255);
    assert!(world.apply_entity_metadata(EntityId(37), &health_metadata));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("living entity health");
    assert_eq!(entity.health(), Some(7.5));
    assert_eq!(entity.shared_flags(), 0);

    let mut zero_health_metadata = vec![
        crate::ENTITY_HEALTH_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_FLOAT_ID as u8,
    ];
    zero_health_metadata.extend_from_slice(&0.0_f32.to_be_bytes());
    zero_health_metadata.push(255);
    assert!(world.apply_entity_metadata(EntityId(37), &zero_health_metadata));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .and_then(crate::ClientEntity::health),
        Some(0.0)
    );

    assert!(world.apply_entity_metadata(EntityId(37), &[5, 8, 1, 255]));
    let entity = world
        .entities()
        .at(EntityId(37))
        .expect("health after partial metadata");
    assert_eq!(entity.health(), Some(0.0));
    assert!(entity.is_no_gravity());
}

#[test]
fn entity_arrow_count_follows_java_int_metadata_and_preserves_partial_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: photon_data::entity_type::named("minecraft:fox"),
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
            .map(crate::ClientEntity::arrow_count),
        Some(0)
    );

    let mut arrow_metadata = vec![
        crate::ENTITY_ARROW_COUNT_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    photon_core::wire::write_varint(3, &mut arrow_metadata);
    arrow_metadata.push(255);
    assert!(world.apply_entity_metadata(EntityId(37), &arrow_metadata));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::arrow_count),
        Some(3)
    );

    assert!(world.apply_entity_metadata(EntityId(37), &[5, 8, 1, 255]));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::arrow_count),
        Some(3)
    );

    assert!(world.apply_entity_metadata(EntityId(37), &[12, 0, 7, 255]));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::arrow_count),
        Some(3)
    );
}

#[test]
fn entity_stinger_count_follows_java_int_metadata_and_preserves_partial_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: photon_data::entity_type::named("minecraft:fox"),
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
            .map(crate::ClientEntity::stinger_count),
        Some(0)
    );

    let mut stinger_metadata = vec![
        crate::ENTITY_STINGER_COUNT_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    photon_core::wire::write_varint(2, &mut stinger_metadata);
    stinger_metadata.push(255);
    assert!(world.apply_entity_metadata(EntityId(37), &stinger_metadata));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::stinger_count),
        Some(2)
    );

    assert!(world.apply_entity_metadata(EntityId(37), &[5, 8, 1, 255]));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::stinger_count),
        Some(2)
    );

    assert!(world.apply_entity_metadata(EntityId(37), &[13, 0, 4, 255]));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .map(crate::ClientEntity::stinger_count),
        Some(2)
    );
}

#[test]
fn entity_effect_ambience_follows_java_boolean_metadata_with_empty_particles() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: photon_data::entity_type::named("minecraft:fox"),
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
            .is_effect_ambient()
    );

    assert!(world.apply_entity_metadata(EntityId(37), &[10, 17, 0, 11, 8, 1, 255]));
    assert!(
        world
            .entities()
            .at(EntityId(37))
            .expect("ambient entity")
            .is_effect_ambient()
    );

    assert!(world.apply_entity_metadata(EntityId(37), &[5, 8, 1, 255]));
    assert!(
        world
            .entities()
            .at(EntityId(37))
            .expect("ambient entity after partial metadata")
            .is_effect_ambient()
    );

    assert!(world.apply_entity_metadata(EntityId(37), &[11, 0, 0, 255]));
    assert!(
        world
            .entities()
            .at(EntityId(37))
            .expect("ambient entity after wrong serializer")
            .is_effect_ambient()
    );

    // A simple particle (`angry_villager`, id 0) is a list the data takes, so the ambience after
    // it is applied too.
    assert!(world.apply_entity_metadata(EntityId(37), &[10, 17, 1, 0, 11, 8, 0, 255]));
    assert!(
        !world
            .entities()
            .at(EntityId(37))
            .expect("entity after a simple particle and no ambience")
            .is_effect_ambient()
    );
}

#[test]
fn entity_effect_particle_types_follow_java_particle_list_and_preserve_partial_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: photon_data::entity_type::named("minecraft:fox"),
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(
        world
            .entities()
            .at(EntityId(37))
            .expect("entity")
            .effect_particles()
            .is_empty()
    );

    let mut particle_metadata = vec![
        crate::ENTITY_EFFECT_PARTICLES_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_PARTICLES_ID as u8,
        1,
    ];
    photon_core::wire::write_varint(
        crate::ENTITY_EFFECT_PARTICLE_TYPE_ID,
        &mut particle_metadata,
    );
    particle_metadata.extend_from_slice(&0x1122_3344_i32.to_be_bytes());
    particle_metadata.extend([11, 8, 0, 255]);
    assert!(world.apply_entity_metadata(EntityId(37), &particle_metadata));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .expect("entity with effect particle")
            .effect_particles(),
        &[crate::EffectParticle {
            type_id: crate::ENTITY_EFFECT_PARTICLE_TYPE_ID,
            color: Some(0x1122_3344)
        }]
    );

    assert!(world.apply_entity_metadata(EntityId(37), &[5, 8, 1, 255]));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .expect("entity after partial particle metadata")
            .effect_particles(),
        &[crate::EffectParticle {
            type_id: crate::ENTITY_EFFECT_PARTICLE_TYPE_ID,
            color: Some(0x1122_3344)
        }]
    );

    assert!(world.apply_entity_metadata(EntityId(37), &[10, 0, 1, 255]));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .expect("entity after wrong particle serializer")
            .effect_particles(),
        &[crate::EffectParticle {
            type_id: crate::ENTITY_EFFECT_PARTICLE_TYPE_ID,
            color: Some(0x1122_3344)
        }]
    );

    assert!(world.apply_entity_metadata(EntityId(37), &[10, 17, 1, 0, 255]));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .expect("entity after a simple particle")
            .effect_particles(),
        // Any particle can be in the list; a simple one (`angry_villager`, id 0) carries nothing.
        &[crate::EffectParticle {
            type_id: 0,
            color: None
        }]
    );
}

#[test]
fn entity_sleeping_position_follows_java_optional_block_pos_metadata() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(37),
        uuid: Uuid::new(1, 2),
        type_id: photon_data::entity_type::named("minecraft:fox"),
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
        .expect("default sleeping entity");
    assert_eq!(entity.sleeping_pos(), None);
    assert!(!entity.is_sleeping());

    let bed = photon_core::binary::BlockPos::new(-12, 90, 37);
    let mut sleeping_metadata = vec![
        crate::ENTITY_SLEEPING_POS_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID as u8,
        1,
    ];
    photon_core::binary::write_blockpos(bed, &mut sleeping_metadata);
    sleeping_metadata.extend([3, 8, 1, 255]);
    assert!(world.apply_entity_metadata(EntityId(37), &sleeping_metadata));
    let entity = world.entities().at(EntityId(37)).expect("sleeping entity");
    assert_eq!(entity.sleeping_pos(), Some(bed));
    assert!(entity.is_sleeping());
    assert!(entity.is_custom_name_visible());

    assert!(world.apply_entity_metadata(EntityId(37), &[5, 8, 1, 255]));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .expect("sleeping entity after partial metadata")
            .sleeping_pos(),
        Some(bed)
    );

    assert!(world.apply_entity_metadata(EntityId(37), &[14, 11, 0, 255]));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .expect("awake entity")
            .sleeping_pos(),
        None
    );

    let mut wrong_serializer = vec![14, 10, 1];
    photon_core::binary::write_blockpos(bed, &mut wrong_serializer);
    wrong_serializer.push(255);
    assert!(world.apply_entity_metadata(EntityId(37), &wrong_serializer));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .expect("entity after wrong serializer")
            .sleeping_pos(),
        None
    );

    assert!(world.apply_entity_metadata(EntityId(37), &[14, 11, 1, 0, 255]));
    assert_eq!(
        world
            .entities()
            .at(EntityId(37))
            .expect("entity after truncated sleeping position")
            .sleeping_pos(),
        None
    );
}
