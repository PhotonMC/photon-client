use super::*;

#[test]
fn collide_entity_movement_follows_java_axis_order() {
    let moving = Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
    let x_wall = Shapes::box_shape(2.0, 0.0, 1.5, 3.0, 1.0, 2.5).unwrap();
    let z_wall = Shapes::box_shape(1.5, 0.0, 2.0, 2.5, 1.0, 3.0).unwrap();
    let shapes = [x_wall, z_wall];

    let x_first = collide_entity_movement(Vec3::new(2.0, 0.0, 1.0), moving, &shapes);
    assert_eq!((x_first.x, x_first.y, x_first.z), (2.0, 0.0, 0.5));

    let z_first = collide_entity_movement(Vec3::new(1.0, 0.0, 2.0), moving, &shapes);
    assert_eq!((z_first.x, z_first.y, z_first.z), (0.5, 0.0, 2.0));

    let y_first_shapes = [
        Shapes::box_shape(-1.0, 0.0, -1.0, 2.0, 1.0, 2.0).unwrap(),
        Shapes::box_shape(1.0, 1.0, -1.0, 2.0, 4.0, 2.0).unwrap(),
    ];
    let y_first = collide_entity_movement(
        Vec3::new(2.0, -2.0, 0.0),
        Aabb::new(0.25, 2.0, 0.25, 0.75, 3.0, 0.75),
        &y_first_shapes,
    );
    assert_eq!((y_first.x, y_first.y, y_first.z), (0.25, -1.0, 0.0));

    let no_collision = collide_entity_movement(Vec3::new(1.25, -2.5, 3.75), moving, &[]);
    assert_eq!(
        (no_collision.x, no_collision.y, no_collision.z),
        (1.25, -2.5, 3.75)
    );
}

#[test]
fn collision_state_follows_java_movement_flags() {
    let moving = Aabb::new(0.25, 2.0, 0.25, 0.75, 3.0, 0.75);
    let floor = Shapes::box_shape(-1.0, 0.0, -1.0, 2.0, 1.0, 2.0).unwrap();
    let falling = collide_entity_movement_state(Vec3::new(0.0, -2.0, 0.0), moving, &[floor]);
    assert_eq!(
        (falling.movement.x, falling.movement.y, falling.movement.z),
        (0.0, -1.0, 0.0)
    );
    assert!(!falling.x_collision);
    assert!(falling.y_collision);
    assert!(!falling.z_collision);
    assert!(!falling.horizontal_collision);
    assert!(falling.vertical_collision);
    assert!(falling.vertical_collision_below);
    assert!(falling.on_ground);

    let ceiling = Shapes::box_shape(-1.0, 4.0, -1.0, 2.0, 5.0, 2.0).unwrap();
    let rising = collide_entity_movement_state(Vec3::new(0.0, 2.0, 0.0), moving, &[ceiling]);
    assert_eq!(
        (rising.movement.x, rising.movement.y, rising.movement.z),
        (0.0, 1.0, 0.0)
    );
    assert!(!rising.horizontal_collision);
    assert!(rising.vertical_collision);
    assert!(!rising.vertical_collision_below);
    assert!(!rising.on_ground);
}

#[test]
fn zero_y_remote_movement_retains_vertical_state() {
    let previous = ClientEntityMovementState {
        movement: Vec3::ZERO,
        x_collision: false,
        y_collision: false,
        z_collision: false,
        horizontal_collision: false,
        vertical_collision: true,
        vertical_collision_below: true,
        on_ground: true,
    };
    let retained =
        apply_entity_movement_state(Vec3::new(2.0, 0.0, 0.0), Vec3::ZERO, previous, false);
    assert!(retained.horizontal_collision);
    assert!(retained.vertical_collision);
    assert!(retained.vertical_collision_below);
    assert!(retained.on_ground);

    let authoritative =
        apply_entity_movement_state(Vec3::new(2.0, 0.0, 0.0), Vec3::ZERO, previous, true);
    assert!(!authoritative.vertical_collision);
    assert!(!authoritative.vertical_collision_below);
    assert!(!authoritative.on_ground);

    let falling = apply_entity_movement_state(
        Vec3::new(0.0, -2.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        previous,
        false,
    );
    assert!(falling.vertical_collision);
    assert!(falling.vertical_collision_below);
    assert!(falling.on_ground);
}

#[test]
fn client_world_applies_block_update_tags_to_state_registry() {
    let mut world = ClientWorld::default();
    let packet = ClientboundUpdateTagsPacket::new(vec![RegistryTags::new(
        Identifier::parse("minecraft:block").unwrap(),
        vec![
            TagEntry::new(
                Identifier::parse("minecraft:climbable").unwrap(),
                vec![221, 366],
            ),
            TagEntry::new(
                Identifier::parse("minecraft:can_glide_through").unwrap(),
                vec![366],
            ),
        ],
    )]);

    world.apply_update_tags_packet(&packet);
    assert!(world.block_state_registry().is_climbable_state(5720));
    assert!(world.block_state_registry().is_climbable_state(8389));
    assert!(world.block_state_registry().is_glide_through_state(8389));
    assert!(!world.block_state_registry().is_glide_through_state(5720));

    let replacement = ClientboundUpdateTagsPacket::new(vec![RegistryTags::new(
        Identifier::parse("minecraft:block").unwrap(),
        vec![TagEntry::new(
            Identifier::parse("minecraft:climbable").unwrap(),
            vec![221],
        )],
    )]);
    world.apply_update_tags_packet(&replacement);
    assert!(world.block_state_registry().is_climbable_state(5720));
    assert!(!world.block_state_registry().is_climbable_state(8389));
    assert!(!world.block_state_registry().is_glide_through_state(8389));
}

#[test]
fn client_world_updates_stair_shape_from_loaded_neighbors() {
    let mut target_blocks = vec![0; 4_096];
    target_blocks[0] = 3918;
    let mut north_blocks = vec![0; 4_096];
    north_blocks[15 * 16] = 3958;
    let biomes = vec![0; 64];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &target_blocks,
            biomes: &biomes,
        }],
    );
    world.replace_chunk(
        0,
        -1,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &north_blocks,
            biomes: &biomes,
        }],
    );

    assert!(world.apply_stair_neighbor_shape_update(0, 0, 0));
    assert_eq!(world.block_state_at(0, 0, 0), Some(3924));
    assert_eq!(
        world.block_state_registry().stair_shape_name(3924),
        Some("outer_left")
    );
    assert_eq!(
        world
            .block_state_registry()
            .collision_shape(3924)
            .to_aabbs(),
        vec![
            Aabb::new(0.0, 0.0, 0.0, 1.0, 0.5, 1.0),
            Aabb::new(0.0, 0.5, 0.0, 0.5, 1.0, 0.5),
        ]
    );
}

#[test]
fn candidate_step_up_heights_follow_java_filter_and_sort_order() {
    let colliders = [
        Shapes::box_shape(0.0, 2.5, 0.0, 1.0, 2.75, 1.0).unwrap(),
        Shapes::box_shape(0.0, 2.0, 0.0, 1.0, 2.5, 1.0).unwrap(),
        Shapes::box_shape(0.0, 2.25, 0.0, 1.0, 2.5, 1.0).unwrap(),
    ];
    let candidates = collect_candidate_step_up_heights(
        Aabb::new(0.0, 2.0, 0.0, 1.0, 3.0, 1.0),
        &colliders,
        0.75,
        -1.0,
    );
    assert_eq!(
        candidates
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        vec![0.0_f32, 0.25, 0.5, 0.75]
            .into_iter()
            .map(f32::to_bits)
            .collect::<Vec<_>>()
    );

    let filtered = [
        Shapes::box_shape(0.0, 9.0, 0.0, 1.0, 10.25, 1.0).unwrap(),
        Shapes::box_shape(0.0, 10.125, 0.0, 1.0, 10.5, 1.0).unwrap(),
        Shapes::box_shape(0.0, 10.375, 0.0, 1.0, 11.0, 1.0).unwrap(),
    ];
    let candidates = collect_candidate_step_up_heights(
        Aabb::new(0.0, 10.0, 0.0, 1.0, 11.0, 1.0),
        &filtered,
        0.5,
        0.25,
    );
    assert_eq!(
        candidates
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        vec![0.125_f32, 0.375, 0.5]
            .into_iter()
            .map(f32::to_bits)
            .collect::<Vec<_>>()
    );
}

#[test]
fn step_up_selection_uses_first_strictly_longer_candidate() {
    let low_obstacle = [Shapes::box_shape(1.0, 0.0, 0.0, 2.0, 0.5, 1.0).unwrap()];
    let selected = select_step_up_movement(
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::ZERO,
        Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        &low_obstacle,
        0.6,
        0.0,
    );
    assert_eq!((selected.x, selected.y, selected.z), (2.0, 0.5, 0.0));

    let tie = select_step_up_movement(
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
        &low_obstacle,
        0.6,
        0.0,
    );
    assert_eq!((tie.x, tie.y, tie.z), (2.0, 0.0, 0.0));

    let offset = [Shapes::box_shape(1.0, 0.5, 0.0, 2.0, 1.0, 1.0).unwrap()];
    let corrected = select_step_up_movement(
        Vec3::new(2.0, -1.0, 0.0),
        Vec3::new(0.0, -0.5, 0.0),
        Aabb::new(0.0, 1.0, 0.0, 1.0, 2.0, 1.0),
        Aabb::new(0.0, 0.5, 0.0, 1.0, 1.5, 1.0),
        &offset,
        0.6,
        -0.5,
    );
    assert_eq!((corrected.x, corrected.y, corrected.z), (2.0, 0.0, 0.0));
}

#[test]
fn player_pose_dimensions_follow_avatar_table_and_fallback() {
    let expected = [
        (
            ClientEntityPose::Standing,
            0.6_f32,
            1.8_f32,
            1.62_f32,
            false,
        ),
        (ClientEntityPose::Sleeping, 0.2, 0.2, 0.2, true),
        (ClientEntityPose::FallFlying, 0.6, 0.6, 0.4, false),
        (ClientEntityPose::Swimming, 0.6, 0.6, 0.4, false),
        (ClientEntityPose::SpinAttack, 0.6, 0.6, 0.4, false),
        (ClientEntityPose::Crouching, 0.6, 1.5, 1.27, false),
        (ClientEntityPose::Dying, 0.2, 0.2, 1.62, true),
    ];
    for (pose, width, height, eye_height, fixed) in expected {
        let dimensions = pose.player_dimensions();
        assert_eq!(dimensions.width().to_bits(), width.to_bits());
        assert_eq!(dimensions.height().to_bits(), height.to_bits());
        assert_eq!(dimensions.eye_height().to_bits(), eye_height.to_bits());
        assert_eq!(dimensions.is_fixed(), fixed);
    }

    for id in [-1, 18] {
        assert_eq!(ClientEntityPose::from_id(id), ClientEntityPose::from_id(0));
    }
    let standing = ClientEntityPose::Standing.player_dimensions();
    for pose in [
        ClientEntityPose::LongJumping,
        ClientEntityPose::Croaking,
        ClientEntityPose::UsingTongue,
        ClientEntityPose::Sitting,
        ClientEntityPose::Roaring,
        ClientEntityPose::Sniffing,
        ClientEntityPose::Emerging,
        ClientEntityPose::Digging,
        ClientEntityPose::Sliding,
        ClientEntityPose::Shooting,
        ClientEntityPose::Inhaling,
    ] {
        assert_eq!(pose.player_dimensions(), standing);
    }
}

#[test]
fn player_pose_selection_follows_java_priority_and_fit_fallbacks() {
    let cases = [
        (
            ClientPlayerPoseInput {
                sleeping: true,
                swimming: true,
                fall_flying: true,
                auto_spin_attack: true,
                shift_key_down: true,
                swimming_fits: true,
                desired_fits: true,
                crouching_fits: true,
                ..ClientPlayerPoseInput::default()
            },
            ClientEntityPose::Sleeping,
            Some(ClientEntityPose::Sleeping),
        ),
        (
            ClientPlayerPoseInput {
                swimming: true,
                fall_flying: true,
                auto_spin_attack: true,
                shift_key_down: true,
                swimming_fits: true,
                desired_fits: true,
                crouching_fits: true,
                ..ClientPlayerPoseInput::default()
            },
            ClientEntityPose::Swimming,
            Some(ClientEntityPose::Swimming),
        ),
        (
            ClientPlayerPoseInput {
                fall_flying: true,
                auto_spin_attack: true,
                swimming_fits: true,
                desired_fits: true,
                crouching_fits: true,
                ..ClientPlayerPoseInput::default()
            },
            ClientEntityPose::FallFlying,
            Some(ClientEntityPose::FallFlying),
        ),
        (
            ClientPlayerPoseInput {
                auto_spin_attack: true,
                swimming_fits: true,
                desired_fits: true,
                crouching_fits: true,
                ..ClientPlayerPoseInput::default()
            },
            ClientEntityPose::SpinAttack,
            Some(ClientEntityPose::SpinAttack),
        ),
        (
            ClientPlayerPoseInput {
                shift_key_down: true,
                swimming_fits: true,
                desired_fits: true,
                crouching_fits: true,
                ..ClientPlayerPoseInput::default()
            },
            ClientEntityPose::Crouching,
            Some(ClientEntityPose::Crouching),
        ),
        (
            ClientPlayerPoseInput {
                shift_key_down: true,
                flying: true,
                swimming_fits: true,
                desired_fits: true,
                crouching_fits: true,
                ..ClientPlayerPoseInput::default()
            },
            ClientEntityPose::Standing,
            Some(ClientEntityPose::Standing),
        ),
        (
            ClientPlayerPoseInput {
                swimming_fits: false,
                ..ClientPlayerPoseInput::default()
            },
            ClientEntityPose::Standing,
            None,
        ),
        (
            ClientPlayerPoseInput {
                swimming_fits: true,
                desired_fits: false,
                crouching_fits: true,
                ..ClientPlayerPoseInput::default()
            },
            ClientEntityPose::Standing,
            Some(ClientEntityPose::Crouching),
        ),
        (
            ClientPlayerPoseInput {
                swimming_fits: true,
                desired_fits: false,
                crouching_fits: false,
                ..ClientPlayerPoseInput::default()
            },
            ClientEntityPose::Standing,
            Some(ClientEntityPose::Swimming),
        ),
        (
            ClientPlayerPoseInput {
                spectator: true,
                swimming_fits: true,
                desired_fits: false,
                ..ClientPlayerPoseInput::default()
            },
            ClientEntityPose::Standing,
            Some(ClientEntityPose::Standing),
        ),
        (
            ClientPlayerPoseInput {
                passenger: true,
                swimming_fits: true,
                desired_fits: false,
                ..ClientPlayerPoseInput::default()
            },
            ClientEntityPose::Standing,
            Some(ClientEntityPose::Standing),
        ),
    ];
    for (input, desired, actual) in cases {
        assert_eq!(input.desired_pose(), desired);
        assert_eq!(input.updated_pose(), actual);
    }
}

#[test]
fn block_collision_collection_follows_cursor_order_and_overlap() {
    let mut blocks = vec![0; 4_096];
    let set_block = |blocks: &mut [i32], x: usize, y: usize, z: usize, state_id: i32| {
        blocks[y * 256 + z * 16 + x] = state_id;
    };
    set_block(&mut blocks, 0, 0, 0, 1);
    set_block(&mut blocks, 1, 0, 0, 2);
    set_block(&mut blocks, 0, 1, 0, 3);
    set_block(&mut blocks, 0, 0, 1, 99);
    set_block(&mut blocks, 1, 0, 1, 4);

    let biomes = vec![0; 64];
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 5,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    let shapes = BTreeMap::from([
        (1, Shapes::block()),
        (2, Shapes::box_shape(0.0, 0.0, 0.0, 1.0, 0.5, 1.0).unwrap()),
        (3, Shapes::box_shape(0.0, 0.5, 0.0, 1.0, 1.0, 1.0).unwrap()),
        (4, Shapes::empty()),
    ]);

    let collisions = world
        .collect_block_collision_shapes(Aabb::new(0.25, 0.25, 0.25, 1.75, 1.75, 1.75), &shapes);
    assert_eq!(
        collisions
            .iter()
            .map(|collision| collision.position)
            .collect::<Vec<_>>(),
        vec![
            voxel_core::binary::BlockPos::new(0, 0, 0),
            voxel_core::binary::BlockPos::new(1, 0, 0),
            voxel_core::binary::BlockPos::new(0, 1, 0),
        ]
    );
    assert_eq!(
        collisions[1].shape.to_aabbs(),
        vec![Aabb::new(1.0, 0.0, 0.0, 2.0, 0.5, 1.0)]
    );
    assert!(
        world
            .collect_block_collision_shapes(Aabb::new(2.0, 0.25, 0.25, 3.0, 0.75, 0.75), &shapes,)
            .is_empty()
    );
}

#[test]
fn chunk_sections_are_queryable_with_java_local_order() {
    let (blocks, biomes) = section(7, 8);
    let mut world = ClientWorld::new(DimensionBounds::new(-64, 384));
    world.replace_chunk(
        -1,
        2,
        [SectionInput {
            non_empty_block_count: 6,
            fluid_count: 1,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );

    assert_eq!(world.loaded_chunk_count(), 1);
    assert_eq!(world.loaded_section_count(), 1);
    assert_eq!(world.section_counts_at(-1, -4, 2), Some((6, 1)));
    assert_eq!(world.block_state_at(-16, -64, 32), Some(11));
    assert_eq!(world.block_state_at(-1, -64, 32), Some(12));
    assert_eq!(world.block_state_at(-16, -64, 33), Some(13));
    assert_eq!(world.block_state_at(-1, -64, 47), Some(14));
    assert_eq!(world.block_state_at(-16, -63, 32), Some(15));
    assert_eq!(world.block_state_at(-1, -49, 47), Some(16));
    assert_eq!(world.block_state_at(-17, -64, 32), None);
    assert_eq!(world.biome_at(-16, -64, 32), Some(21));
    assert_eq!(world.biome_at(-4, -64, 32), Some(22));
    assert_eq!(world.biome_at(-16, -64, 36), Some(23));
    assert_eq!(world.biome_at(-1, -49, 47), Some(24));
}

#[test]
fn chunk_cache_hides_stale_slots_from_world_queries() {
    let (blocks, biomes) = section(7, 8);
    let mut world = ClientWorld::new(DimensionBounds::new(-64, 384));
    world.configure_chunk_cache(0);
    assert!(world.replace_chunk(
        0,
        0,
        [SectionInput {
            non_empty_block_count: 6,
            fluid_count: 1,
            block_states: &blocks,
            biomes: &biomes,
        }],
    ));

    assert!(world.contains_chunk(0, 0));
    assert_eq!(world.block_state_at(0, -64, 0), Some(11));
    assert_eq!(world.section_counts_at(0, -4, 0), Some((6, 1)));
    assert_eq!(world.biome_at(0, -64, 0), Some(21));
    assert!(world.update_block_state(0, -64, 0, 86));

    world.update_chunk_cache_center(8, 0);
    assert_eq!(world.loaded_chunk_count(), 1);
    assert!(!world.contains_chunk(0, 0));
    assert_eq!(world.block_state_at(0, -64, 0), None);
    assert_eq!(world.section_counts_at(0, -4, 0), None);
    assert_eq!(world.biome_at(0, -64, 0), None);
    assert!(!world.update_block_state(0, -64, 0, 87));

    world.update_chunk_cache_center(0, 0);
    assert!(world.contains_chunk(0, 0));
    assert_eq!(world.block_state_at(0, -64, 0), Some(86));
}

#[test]
fn server_block_update_replaces_loaded_state() {
    let (blocks, biomes) = section(7, 8);
    let mut world = ClientWorld::default();
    world.replace_chunk(
        -1,
        2,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );

    assert_eq!(world.block_state_at(-1, -64, 32), Some(12));
    assert!(world.update_block_state(-1, -64, 32, 86));
    assert_eq!(world.block_state_at(-1, -64, 32), Some(86));
    assert!(!world.update_block_state(-17, -64, 32, 86));
    assert!(!world.update_block_state(-1, -48, 32, 86));
}

#[test]
fn chunk_light_stores_java_nibble_order_and_empty_sections() {
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

    let mut sky = vec![0; 2_048];
    sky[0] = 0xc3;
    sky[8] = 0x05;
    sky[128] = 0x09;
    let mut block = vec![0; 2_048];
    block[0] = 0xa6;
    let application = world.apply_chunk_light(
        0,
        0,
        LightUpdateInput {
            sky: LightLayerInput {
                mask: &BitSet::from_bits(&[1]),
                empty_mask: &BitSet::from_bits(&[2]),
                updates: &[sky],
            },
            block: LightLayerInput {
                mask: &BitSet::from_bits(&[1]),
                empty_mask: &BitSet::from_bits(&[2]),
                updates: &[block],
            },
        },
    );

    assert_eq!(application.applied_sky_sections, 2);
    assert_eq!(application.applied_block_sections, 2);
    assert_eq!(application.invalid_sky_updates, 0);
    assert_eq!(application.invalid_block_updates, 0);
    // Nothing reaches the engine until a frame runs the queued task.
    assert_eq!(world.sky_light_at(0, -64, 0), None);
    world.flush_light();
    // The filled section and its neighbours above and below hold light.
    assert_eq!(world.loaded_light_section_count(), 3);
    assert_eq!(world.sky_light_at(0, -64, 0), Some(3));
    assert_eq!(world.sky_light_at(1, -64, 0), Some(12));
    assert_eq!(world.sky_light_at(0, -64, 1), Some(5));
    assert_eq!(world.sky_light_at(0, -63, 0), Some(9));
    assert_eq!(world.block_light_at(0, -64, 0), Some(6));
    assert_eq!(world.block_light_at(1, -64, 0), Some(10));
    assert_eq!(world.sky_light_at(0, -48, 0), Some(0));
    assert_eq!(world.block_light_at(0, -48, 0), Some(0));

    assert!(world.unload_chunk(0, 0));
    world.flush_light();
    assert_eq!(world.sky_light_at(0, -64, 0), None);
    assert_eq!(world.loaded_light_section_count(), 0);
}

#[test]
fn light_updates_for_unloaded_chunks_stay_queued_in_the_engine() {
    let mut world = ClientWorld::default();
    let (first, second) = (vec![sky_update(0x05)], vec![sky_update(0x0a)]);
    assert!(world.apply_light_updates(3, -2, sky_only(&first)).queued);
    assert!(world.apply_light_updates(3, -2, sky_only(&second)).queued);
    assert_eq!(world.pending_light_update_count(), 2);
    assert_eq!(world.sky_light_at(48, -64, -32), None);

    // The later layer replaces the earlier one in the engine's queue, which
    // the getters read before any section stores it.
    world.flush_light();
    assert_eq!(world.pending_light_update_count(), 0);
    assert_eq!(world.sky_light_at(48, -64, -32), Some(10));

    let (blocks, biomes) = section(0, 0);
    world.replace_chunk(
        3,
        -2,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    assert_eq!(world.sky_light_at(48, -64, -32), Some(10));
    assert_eq!(world.loaded_light_section_count(), 0);
}

#[test]
fn unloading_a_chunk_removes_its_queued_light() {
    let mut world = ClientWorld::default();
    let (blocks, biomes) = section(0, 0);
    world.replace_chunk(
        3,
        -2,
        [SectionInput {
            non_empty_block_count: 0,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    let sky = vec![sky_update(0x0f)];
    world.apply_light_updates(3, -2, sky_only(&sky));
    assert!(world.unload_chunk(3, -2));
    assert_eq!(world.pending_light_update_count(), 2);
    world.flush_light();
    assert_eq!(world.sky_light_at(48, -64, -32), None);
}

#[test]
fn block_entities_are_queryable_and_removed_with_their_chunk() {
    let (blocks, biomes) = section(7, 8);
    let mut world = ClientWorld::default();
    world.replace_chunk(
        -1,
        2,
        [SectionInput {
            non_empty_block_count: 1,
            fluid_count: 0,
            block_states: &blocks,
            biomes: &biomes,
        }],
    );
    let tag = [10, 0];

    assert_eq!(
        world.apply_block_entities(
            -1,
            2,
            [BlockEntityInput {
                x: -1,
                y: -63,
                z: 32,
                type_id: 7,
                tag: Some(&tag),
            }],
        ),
        1
    );
    assert_eq!(world.loaded_block_entity_count(), 1);
    assert_eq!(
        world.block_entity_at(-1, -63, 32),
        Some(&crate::ClientBlockEntity {
            type_id: 7,
            tag: Some(tag.to_vec()),
            spawner_state: None,
        })
    );

    let replacement = [10, 0, 0];
    assert!(world.apply_block_entity_update(BlockEntityInput {
        x: -1,
        y: -63,
        z: 32,
        type_id: 8,
        tag: Some(&replacement),
    }));
    assert_eq!(world.block_entity_at(-1, -63, 32).unwrap().type_id, 8);
    assert_eq!(
        world.block_entity_at(-1, -63, 32).unwrap().tag,
        Some(replacement.to_vec())
    );
    assert!(world.unload_chunk(-1, 2));
    assert_eq!(world.loaded_block_entity_count(), 0);
    assert_eq!(world.block_entity_at(-1, -63, 32), None);
}

#[test]
fn player_entities_keep_ticking_after_their_chunk_unloads() {
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
        entity_id: 42,
        uuid: Uuid::new(5, 6),
        type_id: crate::VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(
        world
            .entity_at(42)
            .is_some_and(|entity| { entity.is_always_ticking() && entity.is_ticking() })
    );

    assert!(world.unload_chunk(0, -3));
    assert!(
        world
            .entity_at(42)
            .is_some_and(|entity| { entity.is_always_ticking() && entity.is_ticking() })
    );
    world.tick();
    assert_eq!(
        world.entity_at(42).map(crate::ClientEntity::tick_count),
        Some(1)
    );
}
