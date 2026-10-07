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
