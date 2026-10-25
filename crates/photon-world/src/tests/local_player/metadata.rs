use super::super::*;
use photon_core::ids::EntityId;

#[test]
fn local_player_head_rotation_follows_entity_packet_and_respawn_keep_data() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: EntityId(42),
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (15.0, -10.0, 35.0),
        data: 0,
    }));
    world.set_local_player_id(EntityId(42));
    assert_eq!(
        world.local_player().map(|player| player.head_rotation()),
        Some(35.0)
    );
    assert!(world.apply_entity_rotation(EntityRotationInput {
        entity_id: EntityId(42),
        y_rot: None,
        x_rot: None,
        y_head_rot: Some(135.0),
        relative_y_rot: false,
        relative_x_rot: false,
    }));
    assert_eq!(
        world.local_player().map(|player| player.head_rotation()),
        Some(35.0)
    );
    assert_eq!(
        world
            .entities()
            .at(EntityId(42))
            .map(|entity| entity.rotation().2),
        Some(35.0)
    );
    world.tick();
    assert_eq!(
        world
            .entities()
            .at(EntityId(42))
            .map(|entity| entity.rotation().2),
        Some(68.333336)
    );
    world.tick();
    world.tick();
    assert_eq!(
        world.local_player().map(|player| player.head_rotation()),
        Some(135.0)
    );
    assert_eq!(
        world
            .entities()
            .at(EntityId(42))
            .map(|entity| entity.rotation().2),
        Some(135.0)
    );
    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
    assert_eq!(
        world.local_player().map(|player| player.head_rotation()),
        Some(135.0)
    );
    assert!(world.recreate_local_player(0));
    assert_eq!(
        world.local_player().map(|player| player.head_rotation()),
        Some(0.0)
    );
}

#[test]
fn local_player_arrow_count_metadata_follows_entity_data_keep_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(EntityId(42));

    let mut arrow_metadata = vec![
        crate::ENTITY_ARROW_COUNT_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    photon_core::wire::write_varint(5, &mut arrow_metadata);
    arrow_metadata.push(255);
    assert!(world.apply_entity_metadata(EntityId(42), &arrow_metadata));
    assert_eq!(
        world.local_player().map(|player| player.arrow_count()),
        Some(5)
    );

    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
    assert_eq!(
        world.local_player().map(|player| player.arrow_count()),
        Some(5)
    );

    assert!(world.recreate_local_player(0));
    assert_eq!(
        world.local_player().map(|player| player.arrow_count()),
        Some(0)
    );
}

#[test]
fn local_player_stinger_count_metadata_follows_entity_data_keep_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(EntityId(42));

    let mut stinger_metadata = vec![
        crate::ENTITY_STINGER_COUNT_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    photon_core::wire::write_varint(4, &mut stinger_metadata);
    stinger_metadata.push(255);
    assert!(world.apply_entity_metadata(EntityId(42), &stinger_metadata));
    assert_eq!(
        world.local_player().map(|player| player.stinger_count()),
        Some(4)
    );

    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
    assert_eq!(
        world.local_player().map(|player| player.stinger_count()),
        Some(4)
    );

    assert!(world.recreate_local_player(0));
    assert_eq!(
        world.local_player().map(|player| player.stinger_count()),
        Some(0)
    );
}

#[test]
fn local_player_effect_ambience_metadata_follows_entity_data_keep_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(EntityId(42));
    assert!(world.apply_entity_metadata(EntityId(42), &[10, 17, 0, 11, 8, 1, 255]));
    assert!(
        world
            .local_player()
            .expect("ambient player")
            .is_effect_ambient()
    );

    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
    assert!(
        world
            .local_player()
            .expect("respawned ambient player")
            .is_effect_ambient()
    );

    assert!(world.recreate_local_player(0));
    assert!(
        !world
            .local_player()
            .expect("reset ambient player")
            .is_effect_ambient()
    );
}

#[test]
fn local_player_effect_particles_follow_entity_data_keep_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(EntityId(42));
    let mut particle_metadata = vec![
        crate::ENTITY_EFFECT_PARTICLES_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_PARTICLES_ID as u8,
        1,
    ];
    photon_core::wire::write_varint(
        crate::ENTITY_EFFECT_PARTICLE_TYPE_ID,
        &mut particle_metadata,
    );
    particle_metadata.extend_from_slice(&0x5566_7788_i32.to_be_bytes());
    particle_metadata.push(255);
    assert!(world.apply_entity_metadata(EntityId(42), &particle_metadata));
    assert_eq!(
        world
            .local_player()
            .expect("particle player")
            .effect_particles(),
        &[crate::EffectParticle {
            type_id: crate::ENTITY_EFFECT_PARTICLE_TYPE_ID,
            color: Some(0x5566_7788)
        }]
    );

    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
    assert_eq!(
        world
            .local_player()
            .expect("respawned particle player")
            .effect_particles(),
        &[crate::EffectParticle {
            type_id: crate::ENTITY_EFFECT_PARTICLE_TYPE_ID,
            color: Some(0x5566_7788)
        }]
    );

    assert!(world.recreate_local_player(0));
    assert!(
        world
            .local_player()
            .expect("reset particle player")
            .effect_particles()
            .is_empty()
    );
}

#[test]
fn local_player_sleeping_position_follows_entity_data_keep_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(EntityId(42));
    let bed = photon_core::binary::BlockPos::new(14, 90, -23);
    let mut sleeping_metadata = vec![
        crate::ENTITY_SLEEPING_POS_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID as u8,
        1,
    ];
    photon_core::binary::write_blockpos(bed, &mut sleeping_metadata);
    sleeping_metadata.push(255);
    assert!(world.apply_entity_metadata(EntityId(42), &sleeping_metadata));
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
    world.set_local_player_id(EntityId(42));
    assert_eq!(
        world.local_player().map(|player| player.living_flags()),
        Some(0)
    );

    assert!(world.apply_entity_metadata(
        EntityId(42),
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
