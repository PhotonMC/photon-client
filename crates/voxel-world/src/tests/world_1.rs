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
