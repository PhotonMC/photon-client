use super::*;

#[test]
fn entity_collision_collection_follows_java_filter_and_epsilon() {
    let candidates = [
        ClientEntityCollisionCandidate {
            bounding_box: Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            is_source: true,
            spectator: false,
            can_be_collided_with: true,
            is_pushable: false,
            same_vehicle: false,
        },
        ClientEntityCollisionCandidate {
            bounding_box: Aabb::new(0.5, 0.0, 0.0, 1.5, 1.0, 1.0),
            is_source: false,
            spectator: false,
            can_be_collided_with: true,
            is_pushable: false,
            same_vehicle: false,
        },
        ClientEntityCollisionCandidate {
            bounding_box: Aabb::new(0.75, 0.0, 0.0, 1.75, 1.0, 1.0),
            is_source: false,
            spectator: true,
            can_be_collided_with: true,
            is_pushable: false,
            same_vehicle: false,
        },
        ClientEntityCollisionCandidate {
            bounding_box: Aabb::new(0.25, 0.0, 0.0, 1.25, 1.0, 1.0),
            is_source: false,
            spectator: false,
            can_be_collided_with: false,
            is_pushable: false,
            same_vehicle: false,
        },
        ClientEntityCollisionCandidate {
            bounding_box: Aabb::new(0.125, 0.0, 0.0, 1.125, 1.0, 1.0),
            is_source: false,
            spectator: false,
            can_be_collided_with: true,
            is_pushable: false,
            same_vehicle: true,
        },
        ClientEntityCollisionCandidate {
            bounding_box: Aabb::new(2.00000005, 0.0, 0.0, 2.5, 1.0, 1.0),
            is_source: false,
            spectator: false,
            can_be_collided_with: true,
            is_pushable: false,
            same_vehicle: false,
        },
    ];
    let test_area = Aabb::new(0.0, 0.0, 0.0, 2.0, 1.0, 1.0);
    let source_free = collect_entity_collision_shapes(false, test_area, &candidates);
    assert_eq!(source_free.len(), 4);
    assert_eq!(source_free[0].bounds(), Some(candidates[0].bounding_box));
    assert_eq!(source_free[1].bounds(), Some(candidates[1].bounding_box));
    assert_eq!(source_free[2].bounds(), Some(candidates[4].bounding_box));
    assert_eq!(source_free[3].bounds(), Some(candidates[5].bounding_box));

    let with_source = collect_entity_collision_shapes(true, test_area, &candidates);
    assert_eq!(with_source.len(), 2);
    assert_eq!(with_source[0].bounds(), Some(candidates[1].bounding_box));
    assert_eq!(with_source[1].bounds(), Some(candidates[5].bounding_box));

    let tiny_area = Aabb::new(0.0, 0.0, 0.0, 0.00000001, 0.00000001, 0.00000001);
    assert!(collect_entity_collision_shapes(false, tiny_area, &candidates).is_empty());
}

#[test]
fn indexed_minecart_collision_uses_vehicle_dispatch_for_all_registry_types() {
    let add_entity = |world: &mut ClientWorld, entity_id: i32, type_id: i32| {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, type_id as u64),
            type_id,
            x: 0.5,
            y: 0.0,
            z: 0.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    };

    for &minecart_type_id in VANILLA_MINECART_ENTITY_TYPE_IDS {
        let mut minecart_source_world = ClientWorld::default();
        add_entity(&mut minecart_source_world, 7, minecart_type_id);
        add_entity(&mut minecart_source_world, 8, VANILLA_PLAYER_ENTITY_TYPE_ID);
        assert!(
            !minecart_source_world
                .no_entity_collision_for_entity(Some(7), Aabb::new(0.0, 0.0, 0.0, 1.0, 2.0, 1.0),),
            "minecart source type {minecart_type_id} should accept pushable Player"
        );

        let mut boat_source_world = ClientWorld::default();
        add_entity(&mut boat_source_world, 7, 0);
        add_entity(&mut boat_source_world, 8, minecart_type_id);
        assert!(
            !boat_source_world
                .no_entity_collision_for_entity(Some(7), Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),),
            "boat source should accept minecart type {minecart_type_id}"
        );
    }
}

#[test]
fn indexed_player_collision_respects_living_alive_and_spectator_gates() {
    let add_entity = |world: &mut ClientWorld, entity_id: i32, type_id: i32| {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, type_id as u64),
            type_id,
            x: 0.5,
            y: 0.0,
            z: 0.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    };

    let mut world = ClientWorld::default();
    add_entity(&mut world, 7, 0);
    add_entity(&mut world, 8, VANILLA_PLAYER_ENTITY_TYPE_ID);
    let test_area = Aabb::new(0.0, 0.0, 0.0, 1.0, 2.0, 1.0);

    assert!(!world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(world.apply_entity_living_collision_state(8, false, false, false));
    assert!(world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(world.apply_entity_living_collision_state(8, true, true, false));
    assert!(world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(!world.apply_entity_living_collision_state(999, true, false, false));
}

#[test]
fn indexed_player_collision_respects_living_climbable_gate() {
    let mut world = ClientWorld::default();
    for (entity_id, type_id) in [(7, 0), (8, VANILLA_PLAYER_ENTITY_TYPE_ID)] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, type_id as u64),
            type_id,
            x: 0.5,
            y: 0.0,
            z: 0.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    let test_area = Aabb::new(0.0, 0.0, 0.0, 1.0, 2.0, 1.0);

    assert!(!world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(world.apply_entity_living_collision_state(8, true, false, true));
    assert!(world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(world.apply_entity_living_collision_state(8, true, false, false));
    assert!(!world.no_entity_collision_for_entity(Some(7), test_area));
}

#[test]
fn indexed_player_collision_respects_flying_ability_gate() {
    let mut world = ClientWorld::default();
    for (entity_id, type_id) in [(7, 0), (8, VANILLA_PLAYER_ENTITY_TYPE_ID)] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, type_id as u64),
            type_id,
            x: 0.5,
            y: 0.0,
            z: 0.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    let test_area = Aabb::new(0.0, 0.0, 0.0, 1.0, 2.0, 1.0);

    assert!(world.apply_entity_living_collision_state(8, true, false, true));
    assert!(world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(world.apply_entity_player_flying_state(8, true));
    assert!(!world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(!world.apply_entity_player_flying_state(7, true));
}

#[test]
fn indexed_player_collision_respects_trapdoor_ladder_context() {
    let mut world = ClientWorld::default();
    for (entity_id, type_id) in [(7, 0), (8, VANILLA_PLAYER_ENTITY_TYPE_ID)] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, type_id as u64),
            type_id,
            x: 0.5,
            y: 1.0,
            z: 0.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    let test_area = Aabb::new(0.0, 0.0, 0.0, 1.0, 3.0, 1.0);

    assert!(world.apply_entity_living_collision_state(8, true, false, false));
    assert!(world.apply_entity_living_climbable_trapdoor_state(8, true, "north", true, "north"));
    assert!(world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(world.apply_entity_living_climbable_trapdoor_state(8, false, "north", true, "north"));
    assert!(!world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(world.apply_entity_living_climbable_trapdoor_state(8, true, "north", true, "south"));
    assert!(!world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(world.apply_entity_living_climbable_trapdoor_state(8, true, "north", false, "north"));
    assert!(!world.no_entity_collision_for_entity(Some(7), test_area));
}

#[test]
fn indexed_player_collision_respects_climbable_block_state_tag() {
    let mut world = ClientWorld::default();
    world.apply_update_tags_packet(&climbing_tags());
    for (entity_id, type_id) in [(7, 0), (8, VANILLA_PLAYER_ENTITY_TYPE_ID)] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, type_id as u64),
            type_id,
            x: 0.5,
            y: 1.0,
            z: 0.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    let test_area = Aabb::new(0.0, 0.0, 0.0, 1.0, 3.0, 1.0);

    assert!(world.block_state_registry().is_climbable_state(5719));
    assert!(world.block_state_registry().is_climbable_state(5726));
    assert!(!world.block_state_registry().is_climbable_state(1));
    assert!(!world.block_state_registry().is_climbable_state(99999));
    assert!(world.apply_entity_living_collision_state(8, true, false, false));
    assert!(world.apply_entity_living_climbable_block_state(8, 5720));
    assert!(world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(world.apply_entity_living_climbable_block_state(8, 1));
    assert!(!world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(world.apply_entity_living_climbable_block_state(8, 0));
    assert!(!world.no_entity_collision_for_entity(Some(7), test_area));
}

#[test]
fn indexed_player_collision_respects_glide_through_tag() {
    let mut world = ClientWorld::default();
    world.apply_update_tags_packet(&climbing_tags());
    for (entity_id, type_id) in [(7, 0), (8, VANILLA_PLAYER_ENTITY_TYPE_ID)] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, type_id as u64),
            type_id,
            x: 0.5,
            y: 1.0,
            z: 0.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    let test_area = Aabb::new(0.0, 0.0, 0.0, 1.0, 3.0, 1.0);

    assert!(world.block_state_registry().is_glide_through_state(8389));
    assert!(!world.block_state_registry().is_glide_through_state(1));
    assert!(!world.block_state_registry().is_glide_through_state(99999));
    assert!(world.apply_entity_living_collision_state(8, true, false, false));
    assert!(world.apply_entity_living_climbable_glide_through_state(8, 8389, false));
    assert!(world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(
        !world
            .entity_at(8)
            .expect("player entity must exist")
            .is_fall_flying()
    );

    assert!(world.apply_entity_living_climbable_glide_through_state(8, 8389, true));
    assert!(!world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(
        world
            .entity_at(8)
            .expect("player entity must exist")
            .is_fall_flying()
    );

    assert!(world.apply_entity_living_climbable_glide_through_state(8, 1, true));
    assert!(!world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(
        world
            .entity_at(8)
            .expect("player entity must exist")
            .is_fall_flying()
    );

    assert!(world.apply_entity_living_climbable_glide_through_state(8, 8389, false));
    assert!(world.no_entity_collision_for_entity(Some(7), test_area));
    assert!(
        !world
            .entity_at(8)
            .expect("player entity must exist")
            .is_fall_flying()
    );
}

#[test]
fn entity_dimensions_and_block_fit_follow_java_epsilon() {
    let dimensions = ClientEntityDimensions::scalable(0.6, 1.8);
    assert_eq!(
        dimensions.eye_height().to_bits(),
        (1.8_f32 * 0.85).to_bits()
    );
    assert_eq!(
        dimensions.make_bounding_box(Vec3::new(0.5, 0.0, 0.5)),
        Aabb::new(
            0.19999998807907104,
            0.0,
            0.19999998807907104,
            0.800000011920929,
            1.7999999523162842,
            0.800000011920929,
        )
    );

    let scaled = dimensions.scale(0.5, 2.0).with_eye_height(2.75);
    assert_eq!(scaled.width().to_bits(), (0.6_f32 * 0.5).to_bits());
    assert_eq!(scaled.height().to_bits(), (1.8_f32 * 2.0).to_bits());
    assert_eq!(scaled.eye_height().to_bits(), 2.75_f32.to_bits());

    let fixed = ClientEntityDimensions::fixed(1.0, 2.0);
    assert_eq!(fixed.scale(0.5, 0.5), fixed);

    let mut blocks = vec![0; 4_096];
    blocks[0] = 1;
    let biomes = vec![0; 64];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    let shapes = BTreeMap::from([(1, Shapes::block())]);
    let unit = ClientEntityDimensions::scalable(1.0, 1.0);
    assert!(world.entity_fits_in_blocks(Vec3::new(1.5, 0.25, 0.5), unit, &shapes));
    assert!(!world.entity_fits_in_blocks(Vec3::new(1.4999997, 0.25, 0.5), unit, &shapes));
}

#[test]
fn living_entity_dimensions_follow_sleep_and_scale_branches() {
    let default_dimensions = ClientEntityDimensions::scalable(0.6, 1.8).with_eye_height(1.62);
    let regular =
        living_entity_dimensions(ClientEntityPose::Standing, default_dimensions, 0.5, 1.5);
    assert_eq!(regular.width().to_bits(), 0x3ee6_6667);
    assert_eq!(regular.height().to_bits(), 0x3fac_cccc);
    assert_eq!(regular.eye_height().to_bits(), 0x3f9b_851f);

    let sleeping =
        living_entity_dimensions(ClientEntityPose::Sleeping, default_dimensions, 0.5, 2.0);
    assert_eq!(
        sleeping,
        ClientEntityDimensions::fixed(0.2, 0.2).with_eye_height(0.2)
    );

    let fixed = ClientEntityDimensions::fixed(1.0, 2.0).with_eye_height(1.7);
    assert_eq!(
        living_entity_dimensions(ClientEntityPose::Crouching, fixed, 0.5, 2.0),
        fixed
    );
}

#[test]
fn refresh_entity_dimensions_reapplies_box_and_client_gate() {
    let old_dimensions = ClientEntityDimensions::scalable(0.6, 1.5).with_eye_height(1.27);
    let new_dimensions = ClientEntityDimensions::scalable(0.6, 1.8).with_eye_height(1.62);
    let position = Vec3::new(-1.25, 2.0, 3.75);
    let refreshed = refresh_entity_dimensions(
        old_dimensions,
        new_dimensions,
        position,
        true,
        false,
        false,
        false,
    );
    assert_eq!(refreshed.dimensions, new_dimensions);
    assert_eq!(
        refreshed.bounding_box,
        new_dimensions.make_bounding_box(position)
    );
    assert!(!refreshed.should_fudge_position);

    let server_resize = refresh_entity_dimensions(
        old_dimensions,
        new_dimensions,
        position,
        false,
        false,
        false,
        false,
    );
    assert!(server_resize.should_fudge_position);

    let player_resize = refresh_entity_dimensions(
        old_dimensions,
        new_dimensions,
        position,
        false,
        false,
        false,
        true,
    );
    assert!(!player_resize.should_fudge_position);
}

#[test]
fn replacing_and_unloading_a_chunk_updates_the_section_cache() {
    let (first_blocks, first_biomes) = section(7, 8);
    let (mut second_blocks, mut second_biomes) = section(17, 18);
    second_blocks[0] = 31;
    second_biomes[0] = 32;
    let mut world = ClientWorld::default();
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &first_blocks,
            biomes: &first_biomes,
        }],
    );
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 2,
            fluid_count: 0,
            block_states: &second_blocks,
            biomes: &second_biomes,
        }],
    );

    assert_eq!(world.loaded_chunk_count(), 1);
    assert_eq!(world.loaded_section_count(), 1);
    assert_eq!(world.block_state_at(0, -64, 0), Some(31));
    assert_eq!(world.biome_at(0, -64, 0), Some(32));
    assert!(world.unload_chunk(0, 0));
    assert!(!world.contains_chunk(0, 0));
    assert_eq!(world.loaded_section_count(), 0);
    assert_eq!(world.block_state_at(0, -64, 0), None);
    assert!(!world.unload_chunk(0, 0));
}

#[test]
fn spawner_state_decodes_java_update_tag_and_defaults() {
    use voxel_core::nbt::{Field, FieldValue};

    let mut tag = Vec::new();
    voxel_core::nbt::write_nbt(
        Some(&[
            Field {
                key: "Delay".to_owned(),
                value: FieldValue::Int(65_535),
            },
            Field {
                key: "MaxSpawnDelay".to_owned(),
                value: FieldValue::Short(640),
            },
            Field {
                key: "MinSpawnDelay".to_owned(),
                value: FieldValue::Short(120),
            },
            Field {
                key: "SpawnCount".to_owned(),
                value: FieldValue::Str("invalid numeric tag".to_owned()),
            },
        ]),
        &mut tag,
    );

    let (blocks, biomes) = section(7, 8);
    let mut world = ClientWorld::default();
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert_eq!(
        world.apply_block_entities(
            0,
            0,
            [BlockEntityInput {
                x: 1,
                y: -63,
                z: 2,
                type_id: 9,
                tag: Some(&tag),
            }],
        ),
        1
    );

    let state = world
        .block_entity_at(1, -63, 2)
        .and_then(crate::ClientBlockEntity::spawner_state)
        .expect("spawner tag should be retained as semantic state");
    assert_eq!(state.delay(), -1);
    assert_eq!(state.min_spawn_delay(), 120);
    assert_eq!(state.max_spawn_delay(), 640);
    assert_eq!(state.spawn_count(), 4);
    assert_eq!(state.max_nearby_entities(), 6);
    assert_eq!(state.required_player_range(), 16);
    assert_eq!(state.spawn_range(), 4);

    let non_spawner = world.block_entity_at(1, -63, 2).unwrap();
    assert_eq!(non_spawner.type_id, 9);
}

#[test]
fn entities_are_queryable_and_replaced_by_entity_id() {
    let mut world = ClientWorld::default();
    let (blocks, biomes) = section(0, 0);
    world.replace_chunk(
        0,
        -3,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 120,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.125, -0.25, 0.5),
        rotation: (12.5, -30.0, 45.0),
        data: 6,
    }));
    assert_eq!(world.loaded_entity_count(), 1);
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::position),
        Some((1.25, 80.0, -35.5))
    );
    assert_eq!(world.entity_at(37).map(|entity| entity.type_id), Some(120));
    assert_eq!(world.entity_at(37).map(|entity| entity.data), Some(6));
    assert_eq!(
        world.entity_at(37).map(|entity| entity.uuid),
        Some(Uuid::new(1, 2))
    );
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::motion),
        Some((0.125, -0.25, 0.5))
    );
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::rotation),
        Some((12.5, -30.0, 45.0))
    );
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::on_ground),
        Some(false)
    );
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::is_ticking),
        Some(true)
    );
    assert!(world.unload_chunk(0, -3));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::is_ticking),
        Some(false)
    );
    world.replace_chunk(
        0,
        -3,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::is_ticking),
        Some(true)
    );
    assert_eq!(world.entity_metadata_at(37), None);
    assert!(world.apply_entity_metadata(37, &[1, 0, 255]));
    assert_eq!(world.entity_metadata_at(37), Some(&[1, 0, 255][..]));
    assert!(!world.apply_entity_metadata(999, &[255]));
    assert!(world.apply_entity_motion(EntityMotionInput {
        entity_id: 37,
        x: -0.5,
        y: 0.25,
        z: 0.75,
    }));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::motion),
        Some((-0.5, 0.25, 0.75))
    );
    assert!(!world.apply_entity_motion(EntityMotionInput {
        entity_id: 999,
        x: 1.0,
        y: 2.0,
        z: 3.0,
    }));
    assert!(world.apply_entity_ground_state(37, true));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::on_ground),
        Some(true)
    );
    assert!(world.apply_entity_ground_state(37, false));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::on_ground),
        Some(false)
    );
    assert!(!world.apply_entity_ground_state(999, true));
    assert!(world.apply_entity_rotation(EntityRotationInput {
        entity_id: 37,
        y_rot: Some(90.0),
        x_rot: Some(-15.0),
        y_head_rot: Some(135.0),
        relative_y_rot: false,
        relative_x_rot: false,
    }));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::rotation),
        Some((90.0, -15.0, 135.0))
    );
    assert!(world.apply_entity_rotation(EntityRotationInput {
        entity_id: 37,
        y_rot: Some(45.0),
        x_rot: Some(10.0),
        y_head_rot: None,
        relative_y_rot: true,
        relative_x_rot: true,
    }));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::rotation),
        Some((135.0, -5.0, 135.0))
    );
    assert!(!world.apply_entity_rotation(EntityRotationInput {
        entity_id: 999,
        y_rot: Some(1.0),
        x_rot: Some(2.0),
        y_head_rot: Some(3.0),
        relative_y_rot: false,
        relative_x_rot: false,
    }));

    assert!(world.apply_entity_position(EntityPositionInput {
        entity_id: 37,
        x: 10.0,
        y: 20.0,
        z: 30.0,
        relative_flags: 0,
    }));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::position),
        Some((10.0, 20.0, 30.0))
    );
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::is_ticking),
        Some(false)
    );
    assert!(world.apply_entity_position(EntityPositionInput {
        entity_id: 37,
        x: 0.5,
        y: -1.0,
        z: 2.0,
        relative_flags: 0b111,
    }));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::position),
        Some((10.5, 19.0, 32.0))
    );
    assert!(!world.apply_entity_position(EntityPositionInput {
        entity_id: 999,
        x: 1.0,
        y: 2.0,
        z: 3.0,
        relative_flags: 0,
    }));
    assert!(world.apply_entity_move(EntityMoveInput {
        entity_id: 37,
        delta_x: 2_048,
        delta_y: -4_096,
        delta_z: 0,
        has_position: true,
        y_rot: None,
        x_rot: None,
    }));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::position),
        Some((11.0, 18.0, 32.0))
    );
    assert!(world.apply_entity_move(EntityMoveInput {
        entity_id: 37,
        delta_x: 7,
        delta_y: 8,
        delta_z: 9,
        has_position: false,
        y_rot: None,
        x_rot: None,
    }));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::position),
        Some((11.0, 18.0, 32.0))
    );
    world.replace_chunk(
        1,
        2,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert!(world.apply_entity_move(EntityMoveInput {
        entity_id: 37,
        delta_x: 32_767,
        delta_y: 0,
        delta_z: 0,
        has_position: true,
        y_rot: None,
        x_rot: None,
    }));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::is_ticking),
        Some(true)
    );

    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(3, 4),
        type_id: 71,
        x: -4.0,
        y: 65.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert_eq!(world.loaded_entity_count(), 1);
    assert_eq!(world.entity_at(37).map(|entity| entity.type_id), Some(71));
    assert_eq!(
        world.entity_at(37).map(|entity| entity.uuid),
        Some(Uuid::new(3, 4))
    );
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::position),
        Some((-4.0, 65.0, 2.0))
    );
    assert_eq!(world.entity_metadata_at(37), None);
    assert!(world.remove_entity(37));
    assert_eq!(world.loaded_entity_count(), 0);
    assert!(!world.remove_entity(37));
}

#[test]
fn entity_passenger_state_replaces_previous_vehicle_and_cleans_relationships() {
    let mut world = ClientWorld::default();
    for entity_id in [37, 38, 39, 40] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, 0),
            type_id: 71,
            x: 0.0,
            y: 64.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    world.set_local_player_id(42);

    assert!(world.apply_entity_passengers(39, &[38]));
    assert_eq!(world.entity_passengers(39), Some(&[38][..]));
    assert_eq!(world.entity_vehicle_id(38), Some(39));

    assert!(world.apply_entity_passengers(37, &[38, 39, 999, 38, 42]));
    assert_eq!(world.entity_passengers(37), Some(&[38, 39, 42][..]));
    assert_eq!(world.entity_passengers(39), Some(&[][..]));
    assert_eq!(world.entity_vehicle_id(38), Some(37));
    assert_eq!(world.entity_vehicle_id(39), Some(37));
    assert_eq!(world.entity_vehicle_id(42), Some(37));
    assert!(world.apply_entity_passengers(39, &[37]));
    assert_eq!(world.entity_passengers(39), Some(&[][..]));
    assert_eq!(world.entity_passengers(37), Some(&[38, 39, 42][..]));

    assert!(world.apply_entity_passengers(37, &[]));
    assert_eq!(world.entity_passengers(37), Some(&[][..]));
    assert_eq!(world.entity_vehicle_id(38), None);
    assert_eq!(world.entity_vehicle_id(39), None);
    assert_eq!(world.entity_vehicle_id(42), None);
    assert!(!world.apply_entity_passengers(999, &[38]));

    assert!(world.apply_entity_passengers(37, &[38]));
    assert!(world.remove_entity(37));
    assert_eq!(world.entity_vehicle_id(38), None);
    assert_eq!(world.entity_at(37), None);
}

#[test]
fn air_boat_float_applies_java_drag_gravity_and_position_step() {
    for (name, no_gravity, expected_motion_y, expected_position_y) in [
        ("falling", false, 0.46_f64, 8.46_f64),
        ("no_gravity", true, 0.5_f64, 8.5_f64),
    ] {
        let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
        let (blocks, biomes) = section(0, 0);
        world.replace_chunk(
            0,
            -1,
            [SectionInput {
                non_empty_block_count: 0,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        );
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 37),
            type_id: 89,
            x: 12.5,
            y: 8.0,
            z: -4.25,
            movement: (0.25, 0.5, -0.75),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        world.set_local_player_id(38);
        assert!(world.apply_entity_passengers(37, &[38]));
        if no_gravity {
            assert!(world.apply_entity_metadata(37, &[ENTITY_NO_GRAVITY_DATA_ID, 8, 1, 255]));
        }

        world.tick();

        let boat = world.entity_at(37).expect("air boat");
        assert_eq!(boat.motion().0.to_bits(), 0x3fcc_cccc_c000_0000, "{name}");
        assert_eq!(
            boat.motion().1.to_bits(),
            expected_motion_y.to_bits(),
            "{name}"
        );
        assert_eq!(boat.motion().2.to_bits(), 0xbfe5_9999_9000_0000, "{name}");
        assert_eq!(boat.position().0.to_bits(), 0x4029_7333_3300_0000, "{name}");
        assert_eq!(
            boat.position().1.to_bits(),
            expected_position_y.to_bits(),
            "{name}"
        );
        assert_eq!(boat.position().2.to_bits(), 0xc013_b333_3200_0000, "{name}");
    }
}

#[test]
fn remote_boat_tick_does_not_apply_client_control_or_motion() {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let (blocks, biomes) = section(0, 0);
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 37),
        type_id: 89,
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.25, 0.5, -0.75),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity(EntityInput {
        entity_id: 38,
        uuid: Uuid::new(1, 38),
        type_id: 151,
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity_passengers(37, &[38]));
    assert!(world.set_boat_input(37, BoatInput::new(false, true, false, false)));

    world.tick();

    let boat = world.entity_at(37).expect("remote boat");
    assert_eq!(boat.position(), (2.0, 12.0, 2.0));
    assert_eq!(boat.motion(), (0.0, 0.0, 0.0));
    assert_eq!(boat.rotation(), (0.0, 0.0, 0.0));
    assert_eq!(boat.boat_delta_rotation(), 0.0);
    assert_eq!(boat.paddle_state(), (false, false));
}

#[test]
fn entity_shared_flags_follow_java_byte_metadata_and_preserve_unspecified_values() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
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
        37,
        &[
            crate::ENTITY_SHARED_FLAGS_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            all_defined_flags,
            255,
        ],
    ));
    let entity = world.entity_at(37).expect("entity with shared flags");
    assert_eq!(entity.shared_flags(), all_defined_flags);
    assert!(entity.is_on_fire());
    assert!(entity.is_shift_key_down());
    assert!(entity.is_sprinting());
    assert!(entity.is_swimming());
    assert!(entity.is_invisible());
    assert!(entity.is_glowing());
    assert!(entity.is_fall_flying());

    assert!(world.apply_entity_metadata(37, &[1, 0, 7, 255]));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::shared_flags),
        Some(all_defined_flags)
    );

    assert!(world.apply_entity_metadata(37, &[0, 0, 0, 255]));
    let entity = world.entity_at(37).expect("cleared shared flags");
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
fn entity_air_supply_follows_java_int_metadata_and_preserves_shared_flags() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 120,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::air_supply),
        Some(300)
    );

    let mut air_metadata = vec![
        crate::ENTITY_AIR_SUPPLY_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    voxel_core::wire::write_varint(250, &mut air_metadata);
    air_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &air_metadata));
    let entity = world.entity_at(37).expect("entity with air supply");
    assert_eq!(entity.air_supply(), 250);
    assert_eq!(entity.shared_flags(), 0);

    let mut combined_metadata = vec![
        crate::ENTITY_SHARED_FLAGS_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
        64,
        crate::ENTITY_AIR_SUPPLY_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    voxel_core::wire::write_varint(-1, &mut combined_metadata);
    combined_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &combined_metadata));
    let entity = world.entity_at(37).expect("entity with combined metadata");
    assert_eq!(entity.air_supply(), -1);
    assert_eq!(entity.shared_flags(), 64);
    assert!(entity.is_glowing());

    assert!(world.apply_entity_metadata(37, &[0, 0, 0, 255]));
    assert_eq!(
        world.entity_at(37).map(crate::ClientEntity::air_supply),
        Some(-1)
    );
}

#[test]
fn entity_no_gravity_follows_java_boolean_metadata_and_preserves_air_supply() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
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
    assert!(world.apply_entity_metadata(37, &no_gravity_metadata));
    let entity = world.entity_at(37).expect("no-gravity entity");
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
    assert!(world.apply_entity_metadata(37, &no_gravity_metadata));
    let entity = world.entity_at(37).expect("cleared no-gravity entity");
    assert!(!entity.is_no_gravity());
    assert_eq!(entity.shared_flags(), 64);
    assert!(entity.is_glowing());

    let mut air_metadata = vec![1, 1];
    voxel_core::wire::write_varint(250, &mut air_metadata);
    air_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &air_metadata));
    let entity = world.entity_at(37).expect("air supply entity");
    assert_eq!(entity.air_supply(), 250);
    assert!(!entity.is_no_gravity());
}

#[test]
fn entity_silent_follows_java_boolean_metadata_and_preserves_other_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 5,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(!world.entity_at(37).expect("entity").is_silent());

    let metadata = [4, 8, 1, 255];
    assert!(world.apply_entity_metadata(37, &metadata));
    let entity = world.entity_at(37).expect("silent entity");
    assert!(entity.is_silent());
    assert!(!entity.is_no_gravity());
    assert_eq!(entity.air_supply(), 300);

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    let entity = world.entity_at(37).expect("silent no-gravity entity");
    assert!(entity.is_silent());
    assert!(entity.is_no_gravity());

    assert!(world.apply_entity_metadata(37, &[4, 8, 0, 255]));
    let entity = world.entity_at(37).expect("audible entity");
    assert!(!entity.is_silent());
    assert!(entity.is_no_gravity());
}

#[test]
fn entity_custom_name_visibility_follows_java_boolean_metadata() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 5,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let entity = world.entity_at(37).expect("default name visibility entity");
    assert!(!entity.is_custom_name_visible());
    assert!(!entity.should_show_name());

    let mut visible_metadata = vec![
        2,
        crate::ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID as u8,
        1,
    ];
    voxel_core::nbt::write_any_nbt(
        Some(&voxel_core::nbt::FieldValue::Str("Display name".to_owned())),
        &mut visible_metadata,
    );
    visible_metadata.extend([
        crate::ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
        1,
        255,
    ]);
    assert!(world.apply_entity_metadata(37, &visible_metadata));
    let entity = world.entity_at(37).expect("visible name entity");
    assert!(entity.is_custom_name_visible());
    assert!(entity.should_show_name());

    assert!(world.apply_entity_metadata(
        37,
        &[
            crate::ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
            0,
            255,
        ],
    ));
    let entity = world.entity_at(37).expect("hidden name entity");
    assert!(!entity.is_custom_name_visible());
    assert!(!entity.should_show_name());
}

#[test]
fn entity_custom_name_presence_follows_java_optional_component_metadata() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 5,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let entity = world.entity_at(37).expect("default custom-name entity");
    assert!(!entity.has_custom_name());
    assert!(!entity.is_custom_name_visible());

    let mut custom_name_metadata = vec![
        crate::ENTITY_CUSTOM_NAME_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID as u8,
        1,
    ];
    voxel_core::nbt::write_any_nbt(
        Some(&voxel_core::nbt::FieldValue::Str("Display name".to_owned())),
        &mut custom_name_metadata,
    );
    custom_name_metadata.extend([
        crate::ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
        1,
        255,
    ]);
    assert!(world.apply_entity_metadata(37, &custom_name_metadata));
    let entity = world.entity_at(37).expect("named entity");
    assert!(entity.has_custom_name());
    assert!(entity.is_custom_name_visible());
    assert!(entity.should_show_name());

    assert!(world.apply_entity_metadata(
        37,
        &[
            crate::ENTITY_CUSTOM_NAME_DATA_ID,
            crate::ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID as u8,
            0,
            255,
        ],
    ));
    let entity = world.entity_at(37).expect("unnamed entity");
    assert!(!entity.has_custom_name());
    assert!(entity.is_custom_name_visible());
    assert!(entity.should_show_name());
}

#[test]
fn entity_health_follows_java_float_metadata_and_preserves_partial_state() {
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
        world.entity_at(37).and_then(crate::ClientEntity::health),
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
    assert!(world.apply_entity_metadata(37, &health_metadata));
    let entity = world.entity_at(37).expect("living entity health");
    assert_eq!(entity.health(), Some(7.5));
    assert_eq!(entity.shared_flags(), 0);

    let mut zero_health_metadata = vec![
        crate::ENTITY_HEALTH_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_FLOAT_ID as u8,
    ];
    zero_health_metadata.extend_from_slice(&0.0_f32.to_be_bytes());
    zero_health_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &zero_health_metadata));
    assert_eq!(
        world.entity_at(37).and_then(crate::ClientEntity::health),
        Some(0.0)
    );

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    let entity = world.entity_at(37).expect("health after partial metadata");
    assert_eq!(entity.health(), Some(0.0));
    assert!(entity.is_no_gravity());
}

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
