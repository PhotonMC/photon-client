use super::super::*;
use photon_core::ids::{EntityId, EntityTypeId};

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
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, type_id as u64),
            type_id: EntityTypeId(type_id),
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
        add_entity(
            &mut minecart_source_world,
            8,
            VANILLA_PLAYER_ENTITY_TYPE_ID.0,
        );
        assert!(
            !minecart_source_world.no_entity_collision_for_entity(
                Some(EntityId(7)),
                Aabb::new(0.0, 0.0, 0.0, 1.0, 2.0, 1.0),
            ),
            "minecart source type {minecart_type_id} should accept pushable Player"
        );

        let mut boat_source_world = ClientWorld::default();
        add_entity(&mut boat_source_world, 7, 0);
        add_entity(&mut boat_source_world, 8, minecart_type_id);
        assert!(
            !boat_source_world.no_entity_collision_for_entity(
                Some(EntityId(7)),
                Aabb::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0),
            ),
            "boat source should accept minecart type {minecart_type_id}"
        );
    }
}

#[test]
fn indexed_player_collision_respects_living_alive_and_spectator_gates() {
    let add_entity = |world: &mut ClientWorld, entity_id: i32, type_id: i32| {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, type_id as u64),
            type_id: EntityTypeId(type_id),
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
    add_entity(&mut world, 8, VANILLA_PLAYER_ENTITY_TYPE_ID.0);
    let test_area = Aabb::new(0.0, 0.0, 0.0, 1.0, 2.0, 1.0);

    assert!(!world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(
        world
            .entities_mut()
            .apply_living_collision_state(EntityId(8), false, false, false)
    );
    assert!(world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(
        world
            .entities_mut()
            .apply_living_collision_state(EntityId(8), true, true, false)
    );
    assert!(world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(
        !world
            .entities_mut()
            .apply_living_collision_state(EntityId(999), true, false, false)
    );
}

#[test]
fn indexed_player_collision_respects_living_climbable_gate() {
    let mut world = ClientWorld::default();
    for (entity_id, type_id) in [(7, 0), (8, VANILLA_PLAYER_ENTITY_TYPE_ID.0)] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, type_id as u64),
            type_id: EntityTypeId(type_id),
            x: 0.5,
            y: 0.0,
            z: 0.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    let test_area = Aabb::new(0.0, 0.0, 0.0, 1.0, 2.0, 1.0);

    assert!(!world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(
        world
            .entities_mut()
            .apply_living_collision_state(EntityId(8), true, false, true)
    );
    assert!(world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(
        world
            .entities_mut()
            .apply_living_collision_state(EntityId(8), true, false, false)
    );
    assert!(!world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
}

#[test]
fn indexed_player_collision_respects_flying_ability_gate() {
    let mut world = ClientWorld::default();
    for (entity_id, type_id) in [(7, 0), (8, VANILLA_PLAYER_ENTITY_TYPE_ID.0)] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, type_id as u64),
            type_id: EntityTypeId(type_id),
            x: 0.5,
            y: 0.0,
            z: 0.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    let test_area = Aabb::new(0.0, 0.0, 0.0, 1.0, 2.0, 1.0);

    assert!(
        world
            .entities_mut()
            .apply_living_collision_state(EntityId(8), true, false, true)
    );
    assert!(world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(
        world
            .entities_mut()
            .apply_player_flying_state(EntityId(8), true)
    );
    assert!(!world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(
        !world
            .entities_mut()
            .apply_player_flying_state(EntityId(7), true)
    );
}

#[test]
fn indexed_player_collision_respects_trapdoor_ladder_context() {
    let mut world = ClientWorld::default();
    for (entity_id, type_id) in [(7, 0), (8, VANILLA_PLAYER_ENTITY_TYPE_ID.0)] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, type_id as u64),
            type_id: EntityTypeId(type_id),
            x: 0.5,
            y: 1.0,
            z: 0.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    let test_area = Aabb::new(0.0, 0.0, 0.0, 1.0, 3.0, 1.0);

    assert!(
        world
            .entities_mut()
            .apply_living_collision_state(EntityId(8), true, false, false)
    );
    assert!(world.entities_mut().apply_climbable_trapdoor_state(
        EntityId(8),
        true,
        "north",
        true,
        "north"
    ));
    assert!(world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(world.entities_mut().apply_climbable_trapdoor_state(
        EntityId(8),
        false,
        "north",
        true,
        "north"
    ));
    assert!(!world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(world.entities_mut().apply_climbable_trapdoor_state(
        EntityId(8),
        true,
        "north",
        true,
        "south"
    ));
    assert!(!world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(world.entities_mut().apply_climbable_trapdoor_state(
        EntityId(8),
        true,
        "north",
        false,
        "north"
    ));
    assert!(!world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
}

#[test]
fn indexed_player_collision_respects_climbable_block_state_tag() {
    let mut world = ClientWorld::default();
    world.apply_update_tags_packet(&climbing_tags());
    for (entity_id, type_id) in [(7, 0), (8, VANILLA_PLAYER_ENTITY_TYPE_ID.0)] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, type_id as u64),
            type_id: EntityTypeId(type_id),
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
    assert!(
        world
            .entities_mut()
            .apply_living_collision_state(EntityId(8), true, false, false)
    );
    assert!(world.apply_entity_living_climbable_block_state(EntityId(8), 5720));
    assert!(world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(world.apply_entity_living_climbable_block_state(EntityId(8), 1));
    assert!(!world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(world.apply_entity_living_climbable_block_state(EntityId(8), 0));
    assert!(!world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
}

#[test]
fn indexed_player_collision_respects_glide_through_tag() {
    let mut world = ClientWorld::default();
    world.apply_update_tags_packet(&climbing_tags());
    for (entity_id, type_id) in [(7, 0), (8, VANILLA_PLAYER_ENTITY_TYPE_ID.0)] {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(entity_id),
            uuid: Uuid::new(entity_id as u64, type_id as u64),
            type_id: EntityTypeId(type_id),
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
    assert!(
        world
            .entities_mut()
            .apply_living_collision_state(EntityId(8), true, false, false)
    );
    assert!(world.apply_entity_living_climbable_glide_through_state(EntityId(8), 8389, false));
    assert!(world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(
        !world
            .entities()
            .at(EntityId(8))
            .expect("player entity must exist")
            .is_fall_flying()
    );

    assert!(world.apply_entity_living_climbable_glide_through_state(EntityId(8), 8389, true));
    assert!(!world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(
        world
            .entities()
            .at(EntityId(8))
            .expect("player entity must exist")
            .is_fall_flying()
    );

    assert!(world.apply_entity_living_climbable_glide_through_state(EntityId(8), 1, true));
    assert!(!world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(
        world
            .entities()
            .at(EntityId(8))
            .expect("player entity must exist")
            .is_fall_flying()
    );

    assert!(world.apply_entity_living_climbable_glide_through_state(EntityId(8), 8389, false));
    assert!(world.no_entity_collision_for_entity(Some(EntityId(7)), test_area));
    assert!(
        !world
            .entities()
            .at(EntityId(8))
            .expect("player entity must exist")
            .is_fall_flying()
    );
}
