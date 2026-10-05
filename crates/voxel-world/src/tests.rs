//! `ClientWorld` integration tests shared across the world submodules.

#![expect(
    clippy::cognitive_complexity,
    clippy::too_many_lines,
    reason = "a scenario test reads as one sequence of steps"
)]

use super::{
    BlockEntityInput, BoatInput, ClientAttributeModifierState, ClientAttributeState,
    ClientEntityCollisionCandidate, ClientEntityDimensions, ClientEntityMovementState,
    ClientEntityPose, ClientItemStackState, ClientPlayerAbilities, ClientPlayerGameMode,
    ClientPlayerPoseInput, ClientWorld, DimensionBounds, DimensionEnvironment, DimensionInput,
    ENTITY_DATA_SERIALIZER_BOOLEAN_ID, ENTITY_NO_GRAVITY_DATA_ID, ENTITY_RELATIVE_DELTA_X_FLAG,
    ENTITY_RELATIVE_DELTA_Y_FLAG, ENTITY_RELATIVE_DELTA_Z_FLAG, ENTITY_RELATIVE_ROTATE_DELTA_FLAG,
    ENTITY_RELATIVE_X_FLAG, ENTITY_RELATIVE_X_ROT_FLAG, ENTITY_RELATIVE_Y_FLAG,
    ENTITY_RELATIVE_Y_ROT_FLAG, ENTITY_RELATIVE_Z_FLAG, EntityInput, EntityMotionInput,
    EntityMoveInput, EntityPositionInput, EntityPositionUpdateMode, EntityRotationInput,
    EntityTeleportInput, LightLayerInput, LightUpdateInput,
    PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS, PLAYER_RESPAWN_KEEP_ENTITY_DATA,
    PROJECTILE_DEGREES_PER_RADIAN, SNOWBALL_GRAVITY, SectionInput, VANILLA_ARROW_ENTITY_TYPE_ID,
    VANILLA_BOAT_PASSENGER_MAX_ROTATION, VANILLA_BREEZE_ENTITY_TYPE_ID, VANILLA_COW_ENTITY_TYPE_ID,
    VANILLA_EXPERIENCE_ORB_ENTITY_TYPE_ID, VANILLA_FALLING_BLOCK_AIR_FRICTION,
    VANILLA_FALLING_BLOCK_ENTITY_TYPE_ID, VANILLA_FALLING_BLOCK_GRAVITY,
    VANILLA_FIREWORK_ROCKET_ENTITY_TYPE_ID, VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
    VANILLA_FIREWORK_ROCKET_VERTICAL_ACCELERATION, VANILLA_LIVING_ENTITY_TYPE_IDS,
    VANILLA_MINECART_ENTITY_TYPE_IDS, VANILLA_PLAYER_ENTITY_TYPE_ID,
    VANILLA_PRIMED_TNT_AIR_FRICTION, VANILLA_PRIMED_TNT_ENTITY_TYPE_ID, VANILLA_PRIMED_TNT_GRAVITY,
    VANILLA_SNOWBALL_ENTITY_TYPE_ID, apply_entity_movement_state,
    collect_candidate_step_up_heights, collect_entity_collision_shapes, collide_entity_movement,
    collide_entity_movement_state, is_minor_horizontal_collision, living_entity_dimensions,
    menu_top_slot_count, refresh_entity_dimensions, select_step_up_movement,
};
use crate::input::{Input, LocalPlayerInputModifiers, LocalPlayerSprintTriggerConditions};
use std::collections::BTreeMap;
use voxel_core::aabb::Aabb;
use voxel_core::binary::{BitSet, Uuid};
use voxel_core::identifier::Identifier;
use voxel_core::math::{
    Vec2, Vec3, mth_atan2, mth_cos_f64, mth_sin_f64, projectile_lerp_rotation_f32,
};
use voxel_core::shape::Shapes;
use voxel_core::tags::{ClientboundUpdateTagsPacket, RegistryTags, TagEntry};

fn section(block_value: i32, biome_value: i32) -> (Vec<i32>, Vec<i32>) {
    let mut blocks = vec![block_value; 4_096];
    blocks[0] = 11;
    blocks[15] = 12;
    blocks[16] = 13;
    blocks[255] = 14;
    blocks[256] = 15;
    blocks[4_095] = 16;

    let mut biomes = vec![biome_value; 64];
    biomes[0] = 21;
    biomes[3] = 22;
    biomes[4] = 23;
    biomes[63] = 24;
    (blocks, biomes)
}

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
fn local_player_minor_horizontal_collision_follows_java_angle_gate() {
    assert!(is_minor_horizontal_collision(
        0.0,
        0.0,
        1.0,
        Vec3::new(0.0, 0.0, 1.0),
    ));
    assert!(is_minor_horizontal_collision(
        0.0,
        0.0,
        1.0,
        Vec3::new(7.9_f64.to_radians().sin(), 0.0, 7.9_f64.to_radians().cos()),
    ));
    assert!(!is_minor_horizontal_collision(
        0.0,
        0.0,
        1.0,
        Vec3::new(8.0_f64.to_radians().sin(), 0.0, 8.0_f64.to_radians().cos()),
    ));
    assert!(!is_minor_horizontal_collision(
        0.0,
        0.0,
        0.001,
        Vec3::new(0.0, 0.0, 1.0),
    ));
    assert!(!is_minor_horizontal_collision(0.0, 0.0, 1.0, Vec3::ZERO,));
}

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

/// The server's climbing tags for the ladder (221) and the vine (366), as a
/// vanilla server sends them.
fn climbing_tags() -> ClientboundUpdateTagsPacket {
    ClientboundUpdateTagsPacket::new(vec![RegistryTags::new(
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
    )])
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
fn local_player_abilities_apply_packet_values_and_reset_on_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let abilities =
        ClientPlayerAbilities::new(true, true, true, true, -0.0, f32::from_bits(0x7fc0_0001));
    assert!(world.apply_local_player_abilities(abilities));
    assert_eq!(
        world.local_player().map(|player| player.abilities()),
        Some(abilities)
    );

    assert!(world.recreate_local_player(0));
    assert_eq!(
        world.local_player().map(|player| player.abilities()),
        Some(ClientPlayerAbilities::default())
    );
}

#[test]
fn local_player_game_mode_updates_java_abilities_and_build_permission() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let initial = ClientPlayerAbilities::new(false, true, false, false, 0.5, 0.25);
    assert!(world.apply_local_player_abilities(initial));

    assert_eq!(
        ClientPlayerGameMode::from_game_event_param(1.49),
        ClientPlayerGameMode::Creative
    );
    assert_eq!(
        ClientPlayerGameMode::from_game_event_param(1.5),
        ClientPlayerGameMode::Adventure
    );
    assert_eq!(
        ClientPlayerGameMode::from_id(99),
        ClientPlayerGameMode::Survival
    );

    assert!(world.apply_local_player_game_mode(ClientPlayerGameMode::Creative));
    let player = world.local_player().expect("creative local player");
    assert_eq!(player.game_mode(), ClientPlayerGameMode::Creative);
    assert!(player.abilities().invulnerable());
    assert!(player.abilities().flying());
    assert!(player.abilities().may_fly());
    assert!(player.abilities().instabuild());
    assert!(player.abilities().may_build());
    assert_eq!(
        player.abilities().flying_speed().to_bits(),
        0.5_f32.to_bits()
    );
    assert_eq!(
        player.abilities().walking_speed().to_bits(),
        0.25_f32.to_bits()
    );

    assert!(world.apply_local_player_game_mode(ClientPlayerGameMode::Spectator));
    let abilities = world
        .local_player()
        .expect("spectator local player")
        .abilities();
    assert!(abilities.invulnerable());
    assert!(abilities.flying());
    assert!(abilities.may_fly());
    assert!(!abilities.instabuild());
    assert!(!abilities.may_build());

    assert!(world.apply_local_player_game_mode(ClientPlayerGameMode::Adventure));
    let abilities = world
        .local_player()
        .expect("adventure local player")
        .abilities();
    assert!(!abilities.invulnerable());
    assert!(!abilities.flying());
    assert!(!abilities.may_fly());
    assert!(!abilities.instabuild());
    assert!(!abilities.may_build());

    assert!(world.apply_local_player_abilities(ClientPlayerAbilities::default()));
    assert!(
        !world
            .local_player()
            .expect("packet-updated player")
            .abilities()
            .may_build()
    );

    assert!(world.apply_local_player_game_mode(ClientPlayerGameMode::Survival));
    let player = world.local_player().expect("survival local player");
    assert_eq!(player.game_mode().id(), 0);
    assert!(player.abilities().may_build());
}

#[test]
fn local_player_apply_input_updates_queryable_state_and_base_fallback() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let modifiers = LocalPlayerInputModifiers {
        using_item: false,
        passenger: false,
        item_use_speed_multiplier: 0.2,
        moving_slowly: false,
        sneaking_movement_factor: 0.3,
    };

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.0, 0.0, 0.0), (15.0, 3.0),));
    assert!(world.apply_local_player_input(Vec2::new(0.0, 1.0), true, true, modifiers,));
    {
        let player = world.local_player().expect("controlled local player");
        let impulses = player.input_impulses();
        assert_eq!(impulses.x.to_bits(), 0.0_f32.to_bits());
        assert_eq!(impulses.y.to_bits(), 0x3f7a_e148);
        assert!(player.is_jumping());
        let bob = player.bob();
        assert_eq!(bob.0.to_bits(), 0x3fc0_0000);
        assert_eq!(bob.1.to_bits(), 0x40f0_0000);
        assert_eq!(player.previous_bob(), (0.0, 0.0));
    }

    assert!(world.apply_local_player_input(Vec2::ZERO, false, false, modifiers,));
    let player = world.local_player().expect("fallback local player");
    let impulses = player.input_impulses();
    assert_eq!(impulses.x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(impulses.y.to_bits(), 0x3f75_dcc7);
    assert!(player.is_jumping());
    assert_eq!(player.bob().0.to_bits(), 0x3fc0_0000);
    assert_eq!(player.bob().1.to_bits(), 0x40f0_0000);
    assert_eq!(player.previous_bob(), (0.0, 0.0));
}

#[test]
fn local_player_fluid_falling_adjustment_updates_motion_and_honors_sprint() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.25, 0.01, -0.5), (0.0, 0.0),));
    assert!(world.apply_local_player_fluid_falling_adjustment(0.16, true));
    assert_eq!(
        world
            .local_player()
            .expect("fluid-adjusted player")
            .motion()
            .1
            .to_bits(),
        0xbf68_9374_bc6a_7efa,
    );

    world
        .local_player
        .as_mut()
        .expect("sprint state player")
        .shared_flags |= 1 << super::SHARED_FLAG_SPRINTING;
    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.25, 0.01, -0.5), (0.0, 0.0),));
    assert!(world.apply_local_player_fluid_falling_adjustment(0.16, true));
    assert_eq!(
        world
            .local_player()
            .expect("sprinting player")
            .motion()
            .1
            .to_bits(),
        0x3f84_7ae1_47ae_147b,
    );
}

#[test]
fn local_player_climbable_movement_clamps_and_stops_ladder_slide() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert!(world.apply_local_player_climbable_state(true, false, false));
    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.4, -0.4, -0.4), (0.0, 0.0),));
    assert!(world.apply_local_player_climbable_movement());
    assert!(
        world
            .local_player()
            .expect("climbable player")
            .is_on_climbable()
    );
    assert_eq!(
        world.local_player().expect("clamped player").motion(),
        (
            f64::from_bits(0x3fc3_3333_4000_0000),
            f64::from_bits(0xbfc3_3333_4000_0000),
            f64::from_bits(0xbfc3_3333_4000_0000),
        ),
    );

    assert!(world.apply_local_player_climbable_state(true, false, true));
    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.1, -0.1), (0.0, 0.0),));
    assert!(world.apply_local_player_climbable_movement());
    assert_eq!(
        world
            .local_player()
            .expect("ladder player")
            .motion()
            .1
            .to_bits(),
        0,
    );

    assert!(world.apply_local_player_climbable_state(true, true, true));
    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.4, -0.4, -0.4), (0.0, 0.0),));
    assert!(world.apply_local_player_climbable_movement());
    assert_eq!(
        world
            .local_player()
            .expect("scaffolding player")
            .motion()
            .1
            .to_bits(),
        0xbfc3_3333_4000_0000,
    );
}

#[test]
fn local_player_friction_influenced_speed_follows_ground_and_flying_state() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);

    assert!(world.apply_local_player_friction_influenced_speed(true, 0.6, 0.1));
    assert_eq!(
        world
            .local_player()
            .expect("threshold player")
            .friction_influenced_speed()
            .to_bits(),
        0x3dcc_cccd,
    );
    assert!(world.apply_local_player_friction_influenced_speed(true, 0.8, 0.1));
    assert_eq!(
        world
            .local_player()
            .expect("friction player")
            .friction_influenced_speed()
            .to_bits(),
        0x3d2c_ccce,
    );

    {
        let player = world.local_player.as_mut().expect("flying player");
        player.abilities = ClientPlayerAbilities::new(false, true, true, false, 0.07, 0.1);
        player.shared_flags |= 1 << super::SHARED_FLAG_SPRINTING;
    }
    assert!(world.apply_local_player_friction_influenced_speed(false, 0.6, 0.1));
    assert_eq!(
        world
            .local_player()
            .expect("flying player")
            .friction_influenced_speed()
            .to_bits(),
        0x3e0f_5c29,
    );

    world
        .local_player
        .as_mut()
        .expect("passenger player")
        .vehicle_id = Some(7);
    assert!(world.apply_local_player_friction_influenced_speed(false, 0.6, 0.1));
    assert_eq!(
        world
            .local_player()
            .expect("passenger player")
            .friction_influenced_speed()
            .to_bits(),
        0x3cd4_fdf3,
    );
}

#[test]
fn local_player_friction_influenced_speed_reads_movement_speed_attribute() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);

    assert_eq!(
        world
            .local_player()
            .expect("default speed player")
            .movement_speed(),
        0.1_f32,
    );
    assert!(world.apply_local_player_friction_influenced_speed_from_attributes(true, 0.6));
    assert_eq!(
        world
            .local_player()
            .expect("default friction speed player")
            .friction_influenced_speed(),
        0.1_f32,
    );

    let attribute = ClientAttributeState::from_wire(
        super::VANILLA_PLAYER_MOVEMENT_SPEED_ATTRIBUTE_ID,
        0.25,
        vec![
            ClientAttributeModifierState::from_wire("test:add".to_owned(), 0.1, 0),
            ClientAttributeModifierState::from_wire("test:base".to_owned(), 0.2, 1),
            ClientAttributeModifierState::from_wire("test:total".to_owned(), 0.5, 2),
        ],
    );
    assert!(world.apply_local_player_attributes(42, &[attribute]));
    assert_eq!(
        world
            .local_player()
            .expect("attribute speed player")
            .movement_speed(),
        0.63_f32,
    );
    assert!(world.apply_local_player_friction_influenced_speed_from_attributes(true, 0.8));
    assert_eq!(
        world
            .local_player()
            .expect("attribute friction speed player")
            .friction_influenced_speed(),
        0.63_f32 * (0.21600002_f32 / (0.8_f32 * 0.8_f32 * 0.8_f32)),
    );
}

#[test]
fn local_player_travel_flying_rotates_input_and_applies_environment_drag() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(
        world.apply_local_player_travel_flying(super::LocalPlayerTravelFlyingParameters {
            input: Vec3::new(0.3, -0.4, 0.8),
            y_rot: 37.5,
            in_water: false,
            in_lava: false,
            water_speed: 0.02,
            lava_speed: 0.03,
            air_speed: 0.04,
        },)
    );
    assert_eq!(
        world.local_player().expect("air travel player").motion(),
        (
            f64::from_bits(0x3fb4_f9e6_b6a5_183a),
            f64::from_bits(0xbfc9_28e0_d551_2ec7),
            f64::from_bits(0x3fd3_6044_84ad_cf07),
        ),
    );

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(
        world.apply_local_player_travel_flying(super::LocalPlayerTravelFlyingParameters {
            input: Vec3::new(0.3, -0.4, 0.8),
            y_rot: 37.5,
            in_water: true,
            in_lava: false,
            water_speed: 0.04,
            lava_speed: 0.06,
            air_speed: 0.07,
        },)
    );
    assert_eq!(
        world.local_player().expect("water travel player").motion(),
        (
            f64::from_bits(0x3fb2_70ca_cee5_eb92),
            f64::from_bits(0xbfc6_1e4f_7b4a_233a),
            f64::from_bits(0x3fd1_08ac_bf6e_af6c),
        ),
    );
}

#[test]
fn local_player_travel_in_water_applies_drag_and_fluid_jump_state() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(
        world.apply_local_player_travel_in_water(super::LocalPlayerTravelInWaterParameters {
            input: Vec3::new(0.3, -0.4, 0.8),
            y_rot: 37.5,
            sprinting: false,
            water_slow_down: 0.8,
            water_efficiency: 0.0,
            on_ground: true,
            speed: 0.1,
            dolphins_grace: false,
            horizontal_collision: false,
            on_climbable: false,
            free_after_jump: false,
            base_gravity: 0.08,
            is_falling: false,
        },)
    );
    assert_eq!(
        world.local_player().expect("plain water player").motion(),
        (
            f64::from_bits(0x3fb3_75d6_0dd9_5c2f),
            f64::from_bits(0xbfc5_f06f_6e48_e8a8),
            f64::from_bits(0x3fd0_326a_dc84_2483),
        ),
    );

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(
        world.apply_local_player_travel_in_water(super::LocalPlayerTravelInWaterParameters {
            input: Vec3::new(0.3, -0.4, 0.8),
            y_rot: 37.5,
            sprinting: false,
            water_slow_down: 0.8,
            water_efficiency: 0.0,
            on_ground: true,
            speed: 0.1,
            dolphins_grace: false,
            horizontal_collision: true,
            on_climbable: false,
            free_after_jump: true,
            base_gravity: 0.08,
            is_falling: true,
        },)
    );
    assert_eq!(
        world.local_player().expect("jump water player").motion(),
        (
            f64::from_bits(0x3fb3_75d6_0dd9_5c2f),
            f64::from_bits(0x3fd3_3333_4000_0000),
            f64::from_bits(0x3fd0_326a_dc84_2483),
        ),
    );
}

#[test]
fn local_player_travel_in_water_reads_movement_speed_attribute() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let attribute = ClientAttributeState::from_wire(
        super::VANILLA_PLAYER_MOVEMENT_SPEED_ATTRIBUTE_ID,
        0.25,
        vec![
            ClientAttributeModifierState::from_wire("test:add".to_owned(), 0.1, 0),
            ClientAttributeModifierState::from_wire("test:base".to_owned(), 0.2, 1),
            ClientAttributeModifierState::from_wire("test:total".to_owned(), 0.5, 2),
        ],
    );
    assert!(world.apply_local_player_attributes(42, &[attribute]));
    assert_eq!(
        world
            .local_player()
            .expect("water attribute player")
            .movement_speed(),
        0.63_f32,
    );
    assert!(world.apply_local_player_position((0.0, 10.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(world.apply_local_player_travel_in_water_from_attributes(
        super::LocalPlayerTravelInWaterParameters {
            input: Vec3::new(0.3, -0.4, 0.8),
            y_rot: 37.5,
            sprinting: false,
            water_slow_down: 0.8,
            water_efficiency: 0.6,
            on_ground: true,
            speed: 0.0,
            dolphins_grace: false,
            horizontal_collision: false,
            on_climbable: false,
            free_after_jump: false,
            base_gravity: 0.08,
            is_falling: false,
        },
    ));
    assert_eq!(
        world.local_player().expect("water motion player").motion(),
        (
            f64::from_bits(0x3f64_b75e_6f6b_cd91),
            f64::from_bits(0xbfd2_771c_a860_5686),
            f64::from_bits(0x3fd9_8280_957d_57d7),
        ),
    );
}

#[test]
fn local_player_sprinting_updates_shared_flag_and_movement_speed_modifier() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);

    assert!(!world.apply_local_player_sprinting(99, true));
    assert!(world.apply_local_player_sprinting(42, true));
    {
        let player = world.local_player().expect("default sprint player");
        assert!(player.is_sprinting());
        assert_eq!(player.movement_speed().to_bits(), 0x3e05_1eb9);
        let attribute = player
            .attributes()
            .iter()
            .find(|attribute| {
                attribute.attribute_id() == super::VANILLA_PLAYER_MOVEMENT_SPEED_ATTRIBUTE_ID
            })
            .expect("default movement speed attribute");
        assert_eq!(
            attribute.base_value(),
            f64::from(super::VANILLA_PLAYER_DEFAULT_MOVEMENT_SPEED)
        );
        assert_eq!(attribute.modifiers().len(), 1);
        assert_eq!(
            attribute.modifiers()[0].id(),
            super::VANILLA_PLAYER_SPRINTING_MODIFIER_ID
        );
        assert_eq!(attribute.modifiers()[0].operation(), 2);
    }

    assert!(world.apply_local_player_sprinting(42, true));
    assert_eq!(
        world
            .local_player()
            .expect("repeated sprint player")
            .attributes()[0]
            .modifiers()
            .len(),
        1
    );

    let custom_attribute = ClientAttributeState::from_wire(
        super::VANILLA_PLAYER_MOVEMENT_SPEED_ATTRIBUTE_ID,
        0.25,
        vec![ClientAttributeModifierState::from_wire(
            "test:custom_speed".to_owned(),
            0.2,
            2,
        )],
    );
    assert!(world.apply_local_player_attributes(42, &[custom_attribute]));
    assert!(world.apply_local_player_sprinting(42, true));
    {
        let player = world.local_player().expect("custom sprint player");
        assert!(player.is_sprinting());
        assert_eq!(player.movement_speed().to_bits(), 0x3ec7_ae15);
        let attribute = &player.attributes()[0];
        assert_eq!(attribute.modifiers().len(), 2);
        assert!(
            attribute
                .modifiers()
                .iter()
                .any(|modifier| modifier.id() == "test:custom_speed")
        );
        assert_eq!(
            attribute
                .modifiers()
                .iter()
                .filter(|modifier| { modifier.id() == super::VANILLA_PLAYER_SPRINTING_MODIFIER_ID })
                .count(),
            1
        );
    }

    assert!(world.apply_local_player_sprinting(42, false));
    let player = world.local_player().expect("stopped sprint player");
    assert!(!player.is_sprinting());
    assert_eq!(player.movement_speed(), 0.3_f32);
    assert!(
        player.attributes()[0]
            .modifiers()
            .iter()
            .all(|modifier| modifier.id() != super::VANILLA_PLAYER_SPRINTING_MODIFIER_ID)
    );
    assert!(!world.apply_local_player_sprinting(99, true));
    assert!(
        !world
            .local_player()
            .expect("ignored sprint player")
            .is_sprinting()
    );
}

#[test]
fn local_player_sprint_trigger_tracks_previous_input_and_window() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let conditions = LocalPlayerSprintTriggerConditions {
        mobility_restricted: false,
        enough_food: true,
        may_fly: false,
        shallow_water: false,
        flying: false,
        slow_due_to_using_item: false,
        fall_flying: false,
        underwater: false,
        moving_slowly: false,
    };
    let empty = Input::EMPTY;
    let forward = Input::new(true, false, false, false, false, false, false);

    assert!(world.apply_local_player_sprint_trigger(empty, Vec2::ZERO, 7, conditions,));
    assert_eq!(
        world
            .local_player()
            .expect("initial sprint trigger player")
            .sprint_trigger_time(),
        0
    );

    assert!(world.apply_local_player_sprint_trigger(forward, Vec2::UNIT_Y, 7, conditions,));
    {
        let player = world.local_player().expect("opened sprint trigger window");
        assert!(!player.is_sprinting());
        assert_eq!(player.sprint_trigger_time(), 7);
        assert_eq!(player.input_key_presses(), forward);
        assert_eq!(player.input_move_vector().y.to_bits(), 1.0_f32.to_bits());
    }

    assert!(world.apply_local_player_sprint_trigger(empty, Vec2::ZERO, 7, conditions,));
    assert_eq!(
        world
            .local_player()
            .expect("held sprint trigger window")
            .sprint_trigger_time(),
        6
    );

    assert!(world.apply_local_player_sprint_trigger(forward, Vec2::UNIT_Y, 7, conditions,));
    let player = world
        .local_player()
        .expect("started sprint from double tap");
    assert!(player.is_sprinting());
    assert_eq!(player.sprint_trigger_time(), 5);
    assert_eq!(player.movement_speed().to_bits(), 0x3e05_1eb9);
}

#[test]
fn local_player_input_reads_sneaking_speed_attribute() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let modifiers = || LocalPlayerInputModifiers {
        using_item: false,
        passenger: false,
        item_use_speed_multiplier: 0.2,
        moving_slowly: true,
        sneaking_movement_factor: 0.0,
    };

    assert!(world.apply_local_player_input_from_attributes(
        Vec2::new(0.0, 1.0),
        false,
        true,
        modifiers(),
    ));
    let player = world.local_player().expect("default sneaking player");
    assert_eq!(
        player.sneaking_movement_factor_value().to_bits(),
        0x3fd3_3333_3333_3333
    );
    assert_eq!(player.input_impulses().y.to_bits(), 0x3e96_872c);

    let custom_attribute = ClientAttributeState::from_wire(
        super::VANILLA_PLAYER_SNEAKING_SPEED_ATTRIBUTE_ID,
        0.5,
        vec![ClientAttributeModifierState::from_wire(
            "test:sneaking_total".to_owned(),
            0.5,
            2,
        )],
    );
    assert!(world.apply_local_player_attributes(42, &[custom_attribute]));
    assert!(world.apply_local_player_input_from_attributes(
        Vec2::new(0.0, 1.0),
        false,
        true,
        modifiers(),
    ));
    let player = world.local_player().expect("custom sneaking player");
    assert_eq!(player.sneaking_movement_factor_value(), 0.75);
    assert_eq!(player.input_impulses().y.to_bits(), 0x3f3c_28f6);

    let replacement_attribute = ClientAttributeState::from_wire(
        super::VANILLA_PLAYER_SNEAKING_SPEED_ATTRIBUTE_ID,
        0.9,
        Vec::new(),
    );
    assert!(world.apply_local_player_attributes(42, &[replacement_attribute]));
    let non_slow_modifiers = LocalPlayerInputModifiers {
        moving_slowly: false,
        ..modifiers()
    };
    assert!(world.apply_local_player_input_from_attributes(
        Vec2::new(0.0, 1.0),
        false,
        true,
        non_slow_modifiers,
    ));
    let player = world.local_player().expect("non-slow sneaking player");
    assert_eq!(player.input_impulses().y.to_bits(), 0x3f7a_e148);

    let wrong_attribute = ClientAttributeState::from_wire(
        super::VANILLA_PLAYER_SNEAKING_SPEED_ATTRIBUTE_ID,
        0.8,
        Vec::new(),
    );
    assert!(!world.apply_local_player_attributes(99, &[wrong_attribute]));
    assert!(world.apply_local_player_input_from_attributes(
        Vec2::new(0.0, 1.0),
        false,
        true,
        modifiers(),
    ));
    assert_eq!(
        world
            .local_player()
            .expect("ignored sneaking attribute player")
            .sneaking_movement_factor_value(),
        0.9,
    );
}

#[test]
fn local_player_keyboard_input_binds_keys_before_world_modifiers() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let modifiers = LocalPlayerInputModifiers {
        using_item: false,
        passenger: false,
        item_use_speed_multiplier: 0.2,
        moving_slowly: false,
        sneaking_movement_factor: 0.0,
    };

    assert!(world.apply_local_player_keyboard_input(
        Input::new(true, false, false, false, false, false, false),
        true,
        modifiers,
    ));
    let player = world.local_player().expect("forward keyboard player");
    assert_eq!(player.input_impulses().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(player.input_impulses().y.to_bits(), 0x3f7a_e148);
    assert!(!player.is_jumping());

    let custom_attribute = ClientAttributeState::from_wire(
        super::VANILLA_PLAYER_SNEAKING_SPEED_ATTRIBUTE_ID,
        0.4,
        vec![ClientAttributeModifierState::from_wire(
            "test:keyboard_sneaking_total".to_owned(),
            0.5,
            2,
        )],
    );
    assert!(world.apply_local_player_attributes(42, &[custom_attribute]));
    let sneaking_modifiers = LocalPlayerInputModifiers {
        moving_slowly: true,
        ..modifiers
    };
    assert!(world.apply_local_player_keyboard_input(
        Input::new(true, false, true, false, true, false, false),
        true,
        sneaking_modifiers,
    ));
    let player = world.local_player().expect("diagonal keyboard player");
    assert_eq!(player.input_impulses().x.to_bits(), 0x3f16_872c);
    assert_eq!(player.input_impulses().y.to_bits(), 0x3f16_872c);
    assert!(player.is_jumping());

    assert!(world.apply_local_player_keyboard_input(
        Input::new(true, true, true, true, true, true, true),
        true,
        sneaking_modifiers,
    ));
    let player = world.local_player().expect("opposing keyboard player");
    assert_eq!(player.input_impulses().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(player.input_impulses().y.to_bits(), 0.0_f32.to_bits());
    assert!(player.is_jumping());
}

#[test]
fn local_player_travel_in_lava_applies_shallow_drag_and_jump_state() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(
        world.apply_local_player_travel_in_lava(super::LocalPlayerTravelInLavaParameters {
            input: Vec3::new(0.3, -0.4, 0.8),
            y_rot: 37.5,
            sprinting: false,
            shallow_lava: true,
            horizontal_collision: false,
            free_after_jump: false,
            base_gravity: 0.08,
            is_falling: false,
        },)
    );
    assert_eq!(
        world.local_player().expect("plain lava player").motion(),
        (
            f64::from_bits(0x3fa8_534b_8b3a_e058),
            f64::from_bits(0xbfc8_7fcb_973e_ab37),
            f64::from_bits(0x3fc4_3f05_8e95_6c40),
        ),
    );

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(
        world.apply_local_player_travel_in_lava(super::LocalPlayerTravelInLavaParameters {
            input: Vec3::new(0.3, -0.4, 0.8),
            y_rot: 37.5,
            sprinting: false,
            shallow_lava: true,
            horizontal_collision: true,
            free_after_jump: true,
            base_gravity: 0.08,
            is_falling: false,
        },)
    );
    assert_eq!(
        world.local_player().expect("jump lava player").motion(),
        (
            f64::from_bits(0x3fa8_534b_8b3a_e058),
            f64::from_bits(0x3fd3_3333_4000_0000),
            f64::from_bits(0x3fc4_3f05_8e95_6c40),
        ),
    );
}

#[test]
fn local_player_handle_relative_friction_applies_climbable_step_state() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(world.apply_local_player_handle_relative_friction(
        super::LocalPlayerHandleRelativeFrictionParameters {
            input: Vec3::new(0.3, -0.4, 0.8),
            y_rot: 37.5,
            on_ground: false,
            block_friction: 0.6,
            speed: 0.1,
            flying_speed: 0.05,
            flying: false,
            sprinting: false,
            passenger: false,
            on_climbable: false,
            scaffolding: false,
            suppress_sliding_down_ladder: false,
            horizontal_collision: false,
            jumping: false,
        },
    ));
    assert_eq!(
        world.local_player().expect("air relative player").motion(),
        (
            f64::from_bits(0x3fb8_534b_8b3a_e058),
            f64::from_bits(0xbfca_9fbe_7666_6667),
            f64::from_bits(0x3fd4_3f05_8e95_6c40),
        ),
    );

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(world.apply_local_player_handle_relative_friction(
        super::LocalPlayerHandleRelativeFrictionParameters {
            input: Vec3::new(0.3, -0.4, 0.8),
            y_rot: 37.5,
            on_ground: false,
            block_friction: 0.6,
            speed: 0.1,
            flying_speed: 0.05,
            flying: false,
            sprinting: false,
            passenger: false,
            on_climbable: true,
            scaffolding: false,
            suppress_sliding_down_ladder: false,
            horizontal_collision: true,
            jumping: false,
        },
    ));
    assert_eq!(
        world
            .local_player()
            .expect("climbable relative player")
            .motion(),
        (
            f64::from_bits(0x3fb8_534b_8b3a_e058),
            f64::from_bits(0x3fc9_9999_9999_999a),
            f64::from_bits(0x3fc3_3333_4000_0000),
        ),
    );
}

#[test]
fn local_player_travel_in_air_applies_gravity_drag_and_client_fallbacks() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let base = super::LocalPlayerTravelInAirParameters {
        input: Vec3::new(0.3, -0.4, 0.8),
        y_rot: 37.5,
        on_ground: false,
        block_friction: 0.6,
        friction_modifier: 1.0,
        speed: 0.1,
        flying_speed: 0.05,
        flying: false,
        sprinting: false,
        passenger: false,
        on_climbable: false,
        scaffolding: false,
        suppress_sliding_down_ladder: false,
        horizontal_collision: false,
        jumping: false,
        levitation_amplifier: None,
        gravity: 0.08,
        client_side: false,
        has_chunk_at: true,
        current_y: 10.0,
        min_y: -64,
        should_discard_friction: false,
        air_drag_modifier: 0.0,
        omnidirectional_air_mover: false,
    };

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(world.apply_local_player_travel_in_air(base));
    assert_eq!(
        world.local_player().expect("air travel player").motion(),
        (
            f64::from_bits(0x3fb8_534b_8b3a_e058),
            f64::from_bits(0xbfd2_6e97_8d1e_b852),
            f64::from_bits(0x3fd4_3f05_8e95_6c40),
        ),
    );

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(
        world.apply_local_player_travel_in_air(super::LocalPlayerTravelInAirParameters {
            on_ground: true,
            block_friction: 0.98,
            speed: 0.12,
            ..base
        })
    );
    assert_eq!(
        world
            .local_player()
            .expect("ground air travel player")
            .motion(),
        (
            f64::from_bits(0x3fb7_5e33_31f6_ae51),
            f64::from_bits(0xbfd2_a000_ceb8_51ec),
            f64::from_bits(0x3fd4_3a4e_078d_2fbb),
        ),
    );

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(
        world.apply_local_player_travel_in_air(super::LocalPlayerTravelInAirParameters {
            levitation_amplifier: Some(2),
            ..base
        })
    );
    assert_eq!(
        world
            .local_player()
            .expect("levitating air travel player")
            .motion(),
        (
            f64::from_bits(0x3fb8_534b_8b3a_e058),
            f64::from_bits(0xbfc1_758e_2147_ae14),
            f64::from_bits(0x3fd4_3f05_8e95_6c40),
        ),
    );

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(
        world.apply_local_player_travel_in_air(super::LocalPlayerTravelInAirParameters {
            client_side: true,
            has_chunk_at: false,
            ..base
        })
    );
    assert_eq!(
        world
            .local_player()
            .expect("unloaded air travel player")
            .motion(),
        (
            f64::from_bits(0x3fb8_534b_8b3a_e058),
            f64::from_bits(0xbfb9_9999_9999_999a),
            f64::from_bits(0x3fd4_3f05_8e95_6c40),
        ),
    );

    assert!(world.apply_local_player_position((0.0, -64.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(
        world.apply_local_player_travel_in_air(super::LocalPlayerTravelInAirParameters {
            current_y: -64.0,
            client_side: true,
            has_chunk_at: false,
            ..base
        })
    );
    assert_eq!(
        world
            .local_player()
            .expect("min-y air travel player")
            .motion(),
        (
            f64::from_bits(0x3fb8_534b_8b3a_e058),
            0.0,
            f64::from_bits(0x3fd4_3f05_8e95_6c40),
        ),
    );
}

#[test]
fn local_player_update_fall_flying_movement_applies_lift_and_alignment() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let base = super::LocalPlayerUpdateFallFlyingMovementParameters {
        look_angle: Vec3::new(0.0, 0.0, 1.0),
        x_rot: 0.0,
        gravity: 0.08,
    };

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (0.0, 0.0),));
    assert!(world.apply_local_player_update_fall_flying_movement(base));
    assert_eq!(
        world
            .local_player()
            .expect("neutral fall-flying player")
            .motion(),
        (
            f64::from_bits(0x3fb6_cf41_f5c2_8f5c),
            f64::from_bits(0xbfc8_d64d_872b_020d),
            f64::from_bits(0x3fd4_5d87_66ee_39ec),
        ),
    );

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.2, -0.4, 0.3), (0.0, 30.0),));
    assert!(world.apply_local_player_update_fall_flying_movement(
        super::LocalPlayerUpdateFallFlyingMovementParameters {
            look_angle: Vec3::new(0.0, -0.5, 0.8660254037844386),
            x_rot: 30.0,
            ..base
        },
    ));
    assert_eq!(
        world.local_player().expect("nose-down player").motion(),
        (
            f64::from_bits(0x3fc6_cf41_f5c2_8f5c),
            f64::from_bits(0xbfd9_3ca9_72a8_945d),
            f64::from_bits(0x3fd5_4088_ac4b_16c2),
        ),
    );

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.3, 0.1, 0.2), (0.0, -30.0),));
    assert!(world.apply_local_player_update_fall_flying_movement(
        super::LocalPlayerUpdateFallFlyingMovementParameters {
            look_angle: Vec3::new(0.0, 0.5, 0.8660254037844386),
            x_rot: -30.0,
            ..base
        },
    ));
    assert_eq!(
        world.local_player().expect("nose-up player").motion(),
        (
            f64::from_bits(0x3fd1_1b71_7851_eb85),
            f64::from_bits(0x3fb6_1897_eca6_3b6e),
            f64::from_bits(0x3fca_8e62_5f66_105d),
        ),
    );

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (0.0, 15.0),));
    assert!(world.apply_local_player_update_fall_flying_movement(
        super::LocalPlayerUpdateFallFlyingMovementParameters {
            look_angle: Vec3::new(0.0, 1.0, 0.0),
            x_rot: 15.0,
            ..base
        },
    ));
    assert_eq!(
        world.local_player().expect("vertical-look player").motion(),
        (
            f64::from_bits(0x3fb9_5810_6666_6667),
            f64::from_bits(0xbfcc_19d9_6ad4_96cc),
            f64::from_bits(0x3fd3_020c_4ccc_cccd),
        ),
    );
}

#[test]
fn local_player_travel_dispatch_selects_branch_and_updates_motion() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let base = super::LocalPlayerTravelDispatchParameters {
        input: Vec3::new(0.3, -0.4, 0.8),
        y_rot: 37.5,
        x_rot: 0.0,
        look_angle: Vec3::new(0.0, 0.0, 1.0),
        in_water: true,
        in_lava: false,
        affected_by_fluids: true,
        can_stand_on_fluid: false,
        fall_flying: false,
        on_climbable: false,
        sprinting: false,
        water_slow_down: 0.8,
        water_efficiency: 0.0,
        on_ground: false,
        block_friction: 0.6,
        speed: 0.1,
        dolphins_grace: false,
        horizontal_collision: false,
        free_after_jump: false,
        shallow_lava: true,
        flying_speed: 0.05,
        flying: false,
        passenger: false,
        scaffolding: false,
        suppress_sliding_down_ladder: false,
        jumping: false,
        levitation_amplifier: None,
        gravity: 0.08,
        client_side: true,
        has_chunk_at: true,
        current_y: 10.0,
        min_y: -64,
        should_discard_friction: false,
        friction_modifier: 1.0,
        air_drag_modifier: 0.0,
        omnidirectional_air_mover: false,
    };

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    let result = world
        .apply_local_player_travel_dispatch(base)
        .expect("water travel player");
    assert_eq!(result.mode.as_str(), "water");
    assert!(!result.fall_flying_climbable_fallback);
    assert_eq!(
        world.local_player().expect("water travel state").motion(),
        (
            f64::from_bits(0x3fb3_75d6_0dd9_5c2f),
            f64::from_bits(0xbfc5_f06f_6e48_e8a8),
            f64::from_bits(0x3fd0_326a_dc84_2483),
        )
    );

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 20.0),));
    let result = world
        .apply_local_player_travel_dispatch(super::LocalPlayerTravelDispatchParameters {
            in_water: false,
            fall_flying: true,
            look_angle: Vec3::new(0.0, -0.35, 0.9367496997597597),
            x_rot: 20.0,
            ..base
        })
        .expect("fall-flying travel player");
    assert_eq!(result.mode.as_str(), "fall_flying");
    assert!(!result.fall_flying_climbable_fallback);
    assert_eq!(
        world
            .local_player()
            .expect("fall-flying travel state")
            .motion(),
        (
            f64::from_bits(0x3fb6_cf41_f5c2_8f5c),
            f64::from_bits(0xbfc9_f66e_4480_1f80),
            f64::from_bits(0x3fd4_4102_0145_8ac7),
        )
    );

    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    let result = world
        .apply_local_player_travel_dispatch(super::LocalPlayerTravelDispatchParameters {
            in_water: false,
            fall_flying: true,
            on_climbable: true,
            on_ground: true,
            ..base
        })
        .expect("climbable travel player");
    assert_eq!(result.mode.as_str(), "fall_flying");
    assert!(result.fall_flying_climbable_fallback);
    assert_eq!(
        world
            .local_player()
            .expect("climbable travel state")
            .motion(),
        (
            f64::from_bits(0x3fa7_127d_9fe6_cb82),
            f64::from_bits(0xbfcd_70a3_e3d7_0a3e),
            f64::from_bits(0x3fb7_0a3d_8f5c_2900),
        )
    );
}

#[test]
fn local_player_modified_friction_applies_formula_and_clamps_bounds() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);

    assert!(world.apply_local_player_modified_friction(0.8, 1.0));
    assert_eq!(
        world
            .local_player()
            .expect("unit modifier player")
            .modified_friction()
            .to_bits(),
        0x3f4c_cccd,
    );
    assert!(world.apply_local_player_modified_friction(0.8, 2.0));
    assert_eq!(
        world
            .local_player()
            .expect("double modifier player")
            .modified_friction()
            .to_bits(),
        0x3f19_999a,
    );
    assert!(world.apply_local_player_modified_friction(0.2, 2.0));
    assert_eq!(
        world
            .local_player()
            .expect("clamped modifier player")
            .modified_friction()
            .to_bits(),
        0,
    );
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

fn sky_update(value: u8) -> Vec<u8> {
    let mut sky = vec![0; 2_048];
    sky[0] = value;
    sky
}

fn sky_only(updates: &[Vec<u8>]) -> LightUpdateInput<'_> {
    static MASK: std::sync::OnceLock<BitSet> = std::sync::OnceLock::new();
    static NONE: std::sync::OnceLock<BitSet> = std::sync::OnceLock::new();
    LightUpdateInput {
        sky: LightLayerInput {
            mask: MASK.get_or_init(|| BitSet::from_bits(&[1])),
            empty_mask: NONE.get_or_init(BitSet::new),
            updates,
        },
        block: LightLayerInput {
            mask: NONE.get_or_init(BitSet::new),
            empty_mask: NONE.get_or_init(BitSet::new),
            updates: &[],
        },
    }
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
        Some(&super::ClientBlockEntity {
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
        .and_then(super::ClientBlockEntity::spawner_state)
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
        world.entity_at(37).map(super::ClientEntity::position),
        Some((1.25, 80.0, -35.5))
    );
    assert_eq!(world.entity_at(37).map(|entity| entity.type_id), Some(120));
    assert_eq!(world.entity_at(37).map(|entity| entity.data), Some(6));
    assert_eq!(
        world.entity_at(37).map(|entity| entity.uuid),
        Some(Uuid::new(1, 2))
    );
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::motion),
        Some((0.125, -0.25, 0.5))
    );
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::rotation),
        Some((12.5, -30.0, 45.0))
    );
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::on_ground),
        Some(false)
    );
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::is_ticking),
        Some(true)
    );
    assert!(world.unload_chunk(0, -3));
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::is_ticking),
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
        world.entity_at(37).map(super::ClientEntity::is_ticking),
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
        world.entity_at(37).map(super::ClientEntity::motion),
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
        world.entity_at(37).map(super::ClientEntity::on_ground),
        Some(true)
    );
    assert!(world.apply_entity_ground_state(37, false));
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::on_ground),
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
        world.entity_at(37).map(super::ClientEntity::rotation),
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
        world.entity_at(37).map(super::ClientEntity::rotation),
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
        world.entity_at(37).map(super::ClientEntity::position),
        Some((10.0, 20.0, 30.0))
    );
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::is_ticking),
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
        world.entity_at(37).map(super::ClientEntity::position),
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
        world.entity_at(37).map(super::ClientEntity::position),
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
        world.entity_at(37).map(super::ClientEntity::position),
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
        world.entity_at(37).map(super::ClientEntity::is_ticking),
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
        world.entity_at(37).map(super::ClientEntity::position),
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
fn passenger_tick_uses_ride_tick_motion_reset_before_projectile_tick() {
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
    for (entity_id, type_id, motion) in [
        (37, 71, (0.0, 0.0, 0.0)),
        (38, VANILLA_SNOWBALL_ENTITY_TYPE_ID, (0.125, 0.0, 0.0)),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: motion,
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(37, &[38]));

    world.tick();

    assert_eq!(
        world.entity_at(37).map(|entity| entity.tick_count()),
        Some(1)
    );
    let passenger = world.entity_at(38).expect("snowball passenger");
    assert_eq!(passenger.tick_count(), 1);
    assert_eq!(passenger.motion().0, 0.0);
    assert_eq!(passenger.motion().2, 0.0);
    assert_eq!(
        passenger.motion().1,
        -SNOWBALL_GRAVITY * f64::from(0.99_f32)
    );
    assert_eq!(passenger.position().0, 2.0);
}

#[test]
fn passenger_tick_positions_snowball_on_regular_boat_seat() {
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
    for (entity_id, type_id, position, motion) in [
        (37, 89, (2.0, 2.0, 2.0), (0.0, 0.0, 0.0)),
        (
            38,
            VANILLA_SNOWBALL_ENTITY_TYPE_ID,
            (2.0, 12.0, 2.0),
            (0.125, 0.0, 0.0),
        ),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: position.0,
            y: position.1,
            z: position.2,
            movement: motion,
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(37, &[38]));

    world.tick();

    let passenger = world.entity_at(38).expect("snowball passenger");
    assert_eq!(passenger.previous_position(), (2.0, 12.0, 2.0));
    assert_eq!(passenger.position(), (2.0, 2.1875, 2.0));
    assert_eq!(passenger.motion().0, 0.0);
    assert_eq!(
        passenger.motion().1,
        -SNOWBALL_GRAVITY * f64::from(0.99_f32)
    );
}

#[test]
fn passenger_tick_positions_multiple_snowballs_by_boat_order() {
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
    for (entity_id, type_id, position, rotation) in [
        (37, 89, (2.0, 2.0, 2.0), (90.0, 0.0, 0.0)),
        (
            38,
            VANILLA_SNOWBALL_ENTITY_TYPE_ID,
            (2.0, 12.0, 2.0),
            (0.0, 0.0, 0.0),
        ),
        (
            39,
            VANILLA_SNOWBALL_ENTITY_TYPE_ID,
            (2.0, 12.0, 2.0),
            (0.0, 0.0, 0.0),
        ),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: position.0,
            y: position.1,
            z: position.2,
            movement: (0.0, 0.0, 0.0),
            rotation,
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(37, &[38, 39]));

    world.tick();

    let first = world.entity_at(38).expect("first snowball passenger");
    let second = world.entity_at(39).expect("second snowball passenger");
    assert!((first.position().0 - 1.8).abs() < 1e-6);
    assert!((first.position().1 - 2.1875).abs() < 1e-6);
    assert!((first.position().2 - 2.0).abs() < 1e-6);
    assert!((second.position().0 - 2.6).abs() < 1e-6);
    assert!((second.position().1 - 2.1875).abs() < 1e-6);
    assert!((second.position().2 - 2.0).abs() < 1e-6);
}

#[test]
fn passenger_tick_positions_all_vanilla_boat_geometry_rows() {
    let expected_chest_offset = f64::from(0.15_f32);
    let expected_raft_height = f64::from(0.5625_f32 * 0.8888889_f32);
    let expected_geometry = [
        (0, 0.1875, 0.0),
        (1, 0.1875, expected_chest_offset),
        (8, expected_raft_height, expected_chest_offset),
        (9, expected_raft_height, 0.0),
        (12, 0.1875, 0.0),
        (13, 0.1875, expected_chest_offset),
        (23, 0.1875, 0.0),
        (24, 0.1875, expected_chest_offset),
        (33, 0.1875, 0.0),
        (34, 0.1875, expected_chest_offset),
        (74, 0.1875, 0.0),
        (75, 0.1875, expected_chest_offset),
        (81, 0.1875, 0.0),
        (82, 0.1875, expected_chest_offset),
        (89, 0.1875, 0.0),
        (90, 0.1875, expected_chest_offset),
        (94, 0.1875, 0.0),
        (95, 0.1875, expected_chest_offset),
        (125, 0.1875, 0.0),
        (126, 0.1875, expected_chest_offset),
    ];
    for (index, &(vehicle_type_id, ride_height, passenger_z_offset)) in
        expected_geometry.iter().enumerate()
    {
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
        let vehicle_id = 1000 + index as i32;
        let passenger_id = 2000 + index as i32;
        assert!(world.apply_entity(EntityInput {
            entity_id: vehicle_id,
            uuid: Uuid::new(vehicle_id as u64, 0),
            type_id: vehicle_type_id,
            x: 2.0,
            y: 10.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert!(world.apply_entity(EntityInput {
            entity_id: passenger_id,
            uuid: Uuid::new(passenger_id as u64, 0),
            type_id: VANILLA_SNOWBALL_ENTITY_TYPE_ID,
            x: 2.0,
            y: 12.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert!(world.apply_entity_passengers(vehicle_id, &[passenger_id]));

        world.tick();

        let passenger = world
            .entity_at(passenger_id)
            .expect("boat geometry passenger");
        assert_eq!(
            passenger.position(),
            (2.0, 10.0 + ride_height, 2.0 + passenger_z_offset)
        );
    }
}

#[test]
fn passenger_tick_positions_remote_player_from_vehicle_attachment() {
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
    for (entity_id, type_id, position) in [
        (37, 89, (2.0, 2.0, 2.0)),
        (38, VANILLA_PLAYER_ENTITY_TYPE_ID, (2.0, 12.0, 2.0)),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: position.0,
            y: position.1,
            z: position.2,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(37, &[38]));

    world.tick();

    let passenger = world.entity_at(38).expect("remote player passenger");
    assert_eq!(passenger.previous_position(), (2.0, 12.0, 2.0));
    assert_eq!(passenger.position(), (2.0, 1.5875, 2.0));
}

#[test]
fn passenger_tick_positions_generic_entity_vehicle_attachment() {
    for (passenger_id, passenger_type_id, expected_y) in [
        (38, 71, 2.1875_f64),
        (39, 151, 2.1875_f64),
        (40, 124, 1.875_f64),
    ] {
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
            uuid: Uuid::new(37, 0),
            type_id: 89,
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert!(world.apply_entity(EntityInput {
            entity_id: passenger_id,
            uuid: Uuid::new(passenger_id as u64, 0),
            type_id: passenger_type_id,
            x: 2.0,
            y: 12.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert!(world.apply_entity_passengers(37, &[passenger_id]));

        world.tick();

        let passenger = world
            .entity_at(passenger_id)
            .expect("generic vehicle passenger");
        assert_eq!(passenger.previous_position(), (2.0, 12.0, 2.0));
        assert_eq!(passenger.position(), (2.0, expected_y, 2.0));
    }
}

#[test]
fn passenger_tick_uses_zombie_baby_vehicle_attachment_metadata() {
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
        uuid: Uuid::new(37, 0),
        type_id: 89,
        x: 2.0,
        y: 2.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity(EntityInput {
        entity_id: 38,
        uuid: Uuid::new(38, 0),
        type_id: 151,
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity_passengers(37, &[38]));

    world.tick();
    assert_eq!(
        world.entity_at(38).expect("adult zombie").position(),
        (2.0, 2.1875, 2.0)
    );

    assert!(world.apply_entity_metadata(38, &[16, 8, 1, 255]));
    world.tick();
    let baby = world.entity_at(38).expect("baby zombie");
    assert!(baby.is_baby());
    assert_eq!(baby.position(), (2.0, 2.0, 2.0));

    assert!(world.apply_entity_metadata(38, &[16, 8, 0, 255]));
    world.tick();
    let adult = world.entity_at(38).expect("adult zombie after reset");
    assert!(!adult.is_baby());
    assert_eq!(adult.position(), (2.0, 2.1875, 2.0));
}

#[test]
fn passenger_tick_clamps_remote_player_rotation_to_boat_heading() {
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
        uuid: Uuid::new(37, 0),
        type_id: 89,
        x: 2.0,
        y: 2.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity(EntityInput {
        entity_id: 38,
        uuid: Uuid::new(38, 0),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (140.0, 0.0, 140.0),
        data: 0,
    }));
    assert!(world.apply_entity_passengers(37, &[38]));

    world.tick();

    let passenger = world.entity_at(38).expect("remote player passenger");
    assert_eq!(passenger.rotation(), (105.0, 0.0, 105.0));
    assert_eq!(passenger.previous_rotation(), (105.0, 0.0, 140.0));
    assert_eq!(passenger.body_y_rotation(), 0.0);
    assert_eq!(passenger.previous_body_y_rotation(), 140.0);
}

#[test]
fn passenger_tick_clamps_boat_rotation_for_generic_item_passenger() {
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
    for (entity_id, type_id, rotation) in [(37, 89, (0.0, 0.0, 0.0)), (38, 71, (170.0, 0.0, 170.0))]
    {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation,
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(37, &[38]));

    world.tick();

    let passenger = world.entity_at(38).expect("generic boat passenger");
    assert_eq!(passenger.rotation(), (105.0, 0.0, 170.0));
    assert_eq!(passenger.previous_rotation(), (105.0, 0.0, 170.0));
    assert_eq!(passenger.body_y_rotation(), 170.0);
}

#[test]
fn passenger_tick_applies_animal_rotation_offset_by_entity_parity_at_capacity() {
    for (animal_id, expected_offset) in [(38, 90.0_f32), (39, 270.0_f32)] {
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
        for (entity_id, type_id, rotation) in [
            (37, 89, (0.0, 0.0, 0.0)),
            (animal_id, VANILLA_COW_ENTITY_TYPE_ID, (170.0, 0.0, 170.0)),
            (41, VANILLA_SNOWBALL_ENTITY_TYPE_ID, (0.0, 0.0, 0.0)),
        ] {
            assert!(world.apply_entity(EntityInput {
                entity_id,
                uuid: Uuid::new(entity_id as u64, 0),
                type_id,
                x: 2.0,
                y: 2.0,
                z: 2.0,
                movement: (0.0, 0.0, 0.0),
                rotation,
                data: 0,
            }));
        }
        assert!(world.apply_entity_passengers(37, &[animal_id, 41]));

        world.tick();

        let passenger = world.entity_at(animal_id).expect("Cow passenger");
        assert_eq!(passenger.rotation(), (105.0, 0.0, 105.0 + expected_offset));
        assert_eq!(passenger.previous_rotation(), (105.0, 0.0, 170.0));
        assert_eq!(passenger.body_y_rotation(), expected_offset);
        assert_eq!(passenger.previous_body_y_rotation(), 170.0);
    }
}

#[test]
fn passenger_tick_positions_animal_on_full_boat_seat_offset() {
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
    for (entity_id, type_id, position) in [
        (37, 89, (2.0, 2.0, 2.0)),
        (38, VANILLA_COW_ENTITY_TYPE_ID, (2.0, 12.0, 2.0)),
        (41, VANILLA_SNOWBALL_ENTITY_TYPE_ID, (2.0, 12.0, 2.0)),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: position.0,
            y: position.1,
            z: position.2,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(37, &[38, 41]));

    world.tick();

    let passenger = world.entity_at(38).expect("Cow passenger");
    assert_eq!(
        passenger.position(),
        (2.0, 2.1875, 2.0 + f64::from(0.2_f32 + 0.2_f32))
    );
}

#[test]
fn passenger_tick_applies_animal_living_rotation_before_capacity_offset() {
    for (vehicle_type_id, expected_rotation, expected_body) in
        [(89, 105.0_f32, 0.0_f32), (90, 195.0_f32, 90.0_f32)]
    {
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
        for (entity_id, type_id, rotation) in [
            (37, vehicle_type_id, (0.0, 0.0, 0.0)),
            (38, VANILLA_COW_ENTITY_TYPE_ID, (170.0, 0.0, 170.0)),
        ] {
            assert!(world.apply_entity(EntityInput {
                entity_id,
                uuid: Uuid::new(entity_id as u64, 0),
                type_id,
                x: 2.0,
                y: 2.0,
                z: 2.0,
                movement: (0.0, 0.0, 0.0),
                rotation,
                data: 0,
            }));
        }
        assert!(world.apply_entity_passengers(37, &[38]));

        world.tick();

        let passenger = world.entity_at(38).expect("Cow passenger");
        assert_eq!(passenger.rotation(), (105.0, 0.0, expected_rotation));
        assert_eq!(passenger.previous_rotation(), (105.0, 0.0, 170.0));
        assert_eq!(passenger.body_y_rotation(), expected_body);
        assert_eq!(passenger.previous_body_y_rotation(), 170.0);
    }
}

#[test]
fn passenger_turn_calls_boat_on_passenger_turned() {
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
    for (entity_id, type_id, rotation) in [
        (37, 89, (0.0, 0.0, 0.0)),
        (38, VANILLA_PLAYER_ENTITY_TYPE_ID, (168.75, 0.0, 168.75)),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation,
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(37, &[38]));

    assert!(world.turn_entity(38, -100.0, 0.0));

    let passenger = world.entity_at(38).expect("Player passenger");
    assert_eq!(passenger.rotation(), (105.0, 0.0, 105.0));
    assert_eq!(passenger.previous_rotation(), (105.0, 0.0, 168.75));
    assert_eq!(passenger.body_y_rotation(), 0.0);
    assert_eq!(passenger.previous_body_y_rotation(), 168.75);
    assert_eq!(passenger.position(), (2.0, 2.0, 2.0));
}

#[test]
fn passenger_turn_clamps_breeze_despite_boat_turn_exemption() {
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
    for (entity_id, type_id, rotation) in [
        (37, 89, (0.0, 0.0, 0.0)),
        (38, VANILLA_BREEZE_ENTITY_TYPE_ID, (168.75, 0.0, 168.75)),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation,
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(37, &[38]));

    assert!(world.turn_entity(38, -100.0, 0.0));

    let passenger = world.entity_at(38).expect("Breeze passenger");
    assert_eq!(passenger.rotation(), (105.0, 0.0, 105.0));
    assert_eq!(passenger.previous_rotation(), (105.0, 0.0, 168.75));
    assert_eq!(passenger.body_y_rotation(), 0.0);
    assert_eq!(passenger.previous_body_y_rotation(), 168.75);
}

#[test]
fn boat_control_applies_java_input_delta_and_paddles() {
    let rows = [
        (
            "left",
            BoatInput::new(true, false, false, false),
            0xbf80_0000,
            0x4110_0000,
            0x3fbf_ccc0_8100_0000,
            0xbfcf_5e2d_1900_0000,
            (false, true),
        ),
        (
            "right",
            BoatInput::new(false, true, false, false),
            0x3f80_0000,
            0x4130_0000,
            0x3fbf_c17d_a440_0000,
            0xbfcf_5f2b_3300_0000,
            (true, false),
        ),
        (
            "forward",
            BoatInput::new(false, false, true, false),
            0x0000_0000,
            0x4120_0000,
            0x3fbe_38e6_d800_0000,
            0xbfca_f52e_e000_0000,
            (true, true),
        ),
        (
            "backward",
            BoatInput::new(false, false, false, true),
            0x0000_0000,
            0x4120_0000,
            0x3fc0_1c71_9280_0000,
            0xbfd0_50ad_1200_0000,
            (false, false),
        ),
        (
            "both_turn",
            BoatInput::new(true, true, false, false),
            0x0000_0000,
            0x4120_0000,
            0x3fc0_0000_0000_0000,
            0xbfd0_0000_0000_0000,
            (false, false),
        ),
        (
            "left_forward",
            BoatInput::new(true, false, true, false),
            0xbf80_0000,
            0x4110_0000,
            0x3fbe_6604_0800_0000,
            0xbfca_f168_c800_0000,
            (true, true),
        ),
    ];

    for (
        name,
        input,
        expected_delta,
        expected_yaw,
        expected_motion_x,
        expected_motion_z,
        expected_paddles,
    ) in rows
    {
        let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
        let (mut blocks, biomes) = section(0, 0);
        blocks[12 * 256 + 2 * 16 + 2] = 1;
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
        assert!(world.apply_entity(EntityInput {
            entity_id: 37,
            uuid: Uuid::new(1, 37),
            type_id: 89,
            x: 2.0,
            y: 12.0,
            z: 2.0,
            movement: (0.125, 0.0, -0.25),
            rotation: (10.0, 0.0, 0.0),
            data: 0,
        }));
        assert!(world.apply_entity(EntityInput {
            entity_id: 38,
            uuid: Uuid::new(1, 38),
            type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
            x: 2.0,
            y: 12.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        world.set_local_player_id(38);
        assert!(world.apply_entity_passengers(37, &[38]));
        assert!(
            world.set_boat_input(37, input),
            "{name}: boat input rejected"
        );

        world.tick();

        let boat = world.entity_at(37).expect("controlled boat");
        assert_eq!(
            boat.boat_delta_rotation().to_bits(),
            expected_delta,
            "{name}"
        );
        assert_eq!(boat.rotation().0.to_bits(), expected_yaw, "{name}");
        assert_eq!(boat.motion().0.to_bits(), expected_motion_x, "{name}");
        assert_eq!(boat.motion().2.to_bits(), expected_motion_z, "{name}");
        assert_eq!(boat.paddle_state(), expected_paddles, "{name}");
    }
}

#[test]
fn boat_control_delta_rotation_reaches_passenger_tick() {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let (mut blocks, biomes) = section(0, 0);
    blocks[12 * 256 + 2 * 16 + 2] = 1;
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
    for (entity_id, type_id) in [(37, 89), (38, 151), (39, VANILLA_PLAYER_ENTITY_TYPE_ID)] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(1, entity_id as u64),
            type_id,
            x: 2.0,
            y: 12.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    world.set_local_player_id(39);
    assert!(world.apply_entity_passengers(37, &[39, 38]));
    assert!(world.set_boat_input(37, BoatInput::new(false, true, false, false)));

    world.tick();

    let passenger = world.entity_at(38).expect("living boat passenger");
    assert_eq!(passenger.rotation(), (1.0, 0.0, 1.0));
    assert_eq!(passenger.previous_rotation(), (0.0, 0.0, 0.0));
    assert_eq!(passenger.body_y_rotation(), 1.0);
    let radians = f64::from(-(std::f32::consts::PI / 180.0_f32));
    let passenger_z_offset = f64::from(-0.6_f32);
    let expected_position = (
        2.0 + passenger_z_offset * f64::from(mth_sin_f64(radians)),
        12.1875,
        2.0 + passenger_z_offset * f64::from(mth_cos_f64(radians)),
    );
    assert_eq!(passenger.position(), expected_position);
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
fn boat_authority_uses_first_passenger_only() {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let (mut blocks, biomes) = section(0, 0);
    blocks[12 * 256 + 2 * 16 + 2] = 1;
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
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 37),
        type_id: 89,
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
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
    assert!(world.apply_entity(EntityInput {
        entity_id: 39,
        uuid: Uuid::new(1, 39),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 2.0,
        y: 12.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    world.set_local_player_id(39);
    assert!(world.apply_entity_passengers(37, &[38, 39]));
    assert!(world.set_boat_input(37, BoatInput::new(false, true, false, false)));

    world.tick();

    let boat = world.entity_at(37).expect("boat");
    assert_eq!(boat.position(), (2.0, 12.0, 2.0));
    assert_eq!(boat.motion(), (0.0, 0.0, 0.0));
    assert_eq!(boat.rotation(), (0.0, 0.0, 0.0));
    assert_eq!(boat.boat_delta_rotation(), 0.0);
    assert_eq!(boat.paddle_state(), (false, false));
}

#[test]
fn passenger_turn_clamps_every_vanilla_living_entity_body_and_head() {
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
        uuid: Uuid::new(37, 0),
        type_id: 89,
        x: 2.0,
        y: 2.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    for (index, &type_id) in VANILLA_LIVING_ENTITY_TYPE_IDS.iter().enumerate() {
        let entity_id = 100 + index as i32;
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (168.75, 0.0, 168.75),
            data: 0,
        }));
    }
    for index in 0..VANILLA_LIVING_ENTITY_TYPE_IDS.len() {
        let passenger_id = 100 + index as i32;
        assert!(world.apply_entity_passengers(37, &[passenger_id]));
        assert!(world.turn_entity(passenger_id, -100.0, 0.0));
        let passenger = world.entity_at(passenger_id).expect("living passenger");
        assert_eq!(
            passenger.rotation(),
            (
                VANILLA_BOAT_PASSENGER_MAX_ROTATION,
                0.0,
                VANILLA_BOAT_PASSENGER_MAX_ROTATION,
            )
        );
        assert_eq!(
            passenger.previous_rotation(),
            (VANILLA_BOAT_PASSENGER_MAX_ROTATION, 0.0, 168.75)
        );
        assert_eq!(passenger.body_y_rotation(), 0.0);
    }
}

#[test]
fn passenger_tick_leaves_breeze_rotation_unclamped() {
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
    for (entity_id, type_id, rotation) in [
        (37, 89, (0.0, 0.0, 0.0)),
        (38, VANILLA_BREEZE_ENTITY_TYPE_ID, (170.0, 0.0, 170.0)),
    ] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(entity_id as u64, 0),
            type_id,
            x: 2.0,
            y: 2.0,
            z: 2.0,
            movement: (0.0, 0.0, 0.0),
            rotation,
            data: 0,
        }));
    }
    assert!(world.apply_entity_passengers(37, &[38]));

    world.tick();

    let passenger = world.entity_at(38).expect("Breeze passenger");
    assert_eq!(passenger.rotation(), (170.0, 0.0, 170.0));
    assert_eq!(passenger.previous_rotation(), (170.0, 0.0, 170.0));
}

#[test]
fn passenger_tick_positions_local_player_from_vehicle_attachment() {
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
        uuid: Uuid::new(37, 0),
        type_id: 89,
        x: 2.0,
        y: 2.0,
        z: 2.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    world.set_local_player_id(38);
    assert!(world.apply_entity_metadata(37, &[ENTITY_NO_GRAVITY_DATA_ID, 8, 1, 255]));
    assert!(world.apply_local_player_position((2.0, 12.0, 2.0), (0.0, 0.0, 0.0), (0.0, 0.0),));
    assert!(world.apply_entity_passengers(37, &[38]));

    world.tick();

    let player = world.local_player().expect("local player passenger");
    assert_eq!(player.previous_position(), (2.0, 12.0, 2.0));
    assert_eq!(player.position(), (2.0, 1.5875, 2.0));
}

#[test]
fn local_player_head_rotation_follows_entity_packet_and_respawn_keep_data() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 42,
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (15.0, -10.0, 35.0),
        data: 0,
    }));
    world.set_local_player_id(42);
    assert_eq!(
        world.local_player().map(|player| player.head_rotation()),
        Some(35.0)
    );
    assert!(world.apply_entity_rotation(EntityRotationInput {
        entity_id: 42,
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
        world.entity_at(42).map(|entity| entity.rotation().2),
        Some(35.0)
    );
    world.tick();
    assert_eq!(
        world.entity_at(42).map(|entity| entity.rotation().2),
        Some(68.333336)
    );
    world.tick();
    world.tick();
    assert_eq!(
        world.local_player().map(|player| player.head_rotation()),
        Some(135.0)
    );
    assert_eq!(
        world.entity_at(42).map(|entity| entity.rotation().2),
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
            super::ENTITY_SHARED_FLAGS_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
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
        world.entity_at(37).map(super::ClientEntity::shared_flags),
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
        world.entity_at(37).map(super::ClientEntity::air_supply),
        Some(300)
    );

    let mut air_metadata = vec![
        super::ENTITY_AIR_SUPPLY_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    voxel_core::wire::write_varint(250, &mut air_metadata);
    air_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &air_metadata));
    let entity = world.entity_at(37).expect("entity with air supply");
    assert_eq!(entity.air_supply(), 250);
    assert_eq!(entity.shared_flags(), 0);

    let mut combined_metadata = vec![
        super::ENTITY_SHARED_FLAGS_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
        64,
        super::ENTITY_AIR_SUPPLY_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
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
        world.entity_at(37).map(super::ClientEntity::air_supply),
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
        super::ENTITY_NO_GRAVITY_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
        1,
        255,
    ];
    assert!(world.apply_entity_metadata(37, &no_gravity_metadata));
    let entity = world.entity_at(37).expect("no-gravity entity");
    assert!(entity.is_no_gravity());
    assert_eq!(entity.air_supply(), 300);

    no_gravity_metadata = vec![
        super::ENTITY_SHARED_FLAGS_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
        64,
        super::ENTITY_NO_GRAVITY_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
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
        super::ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID as u8,
        1,
    ];
    voxel_core::nbt::write_any_nbt(
        Some(&voxel_core::nbt::FieldValue::Str("Display name".to_owned())),
        &mut visible_metadata,
    );
    visible_metadata.extend([
        super::ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
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
            super::ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
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
        super::ENTITY_CUSTOM_NAME_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID as u8,
        1,
    ];
    voxel_core::nbt::write_any_nbt(
        Some(&voxel_core::nbt::FieldValue::Str("Display name".to_owned())),
        &mut custom_name_metadata,
    );
    custom_name_metadata.extend([
        super::ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
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
            super::ENTITY_CUSTOM_NAME_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID as u8,
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
        world.entity_at(37).and_then(super::ClientEntity::health),
        None
    );

    let mut health_metadata = vec![
        8,
        super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
        0,
        super::ENTITY_HEALTH_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_FLOAT_ID as u8,
    ];
    health_metadata.extend_from_slice(&7.5_f32.to_be_bytes());
    health_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &health_metadata));
    let entity = world.entity_at(37).expect("living entity health");
    assert_eq!(entity.health(), Some(7.5));
    assert_eq!(entity.shared_flags(), 0);

    let mut zero_health_metadata = vec![
        super::ENTITY_HEALTH_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_FLOAT_ID as u8,
    ];
    zero_health_metadata.extend_from_slice(&0.0_f32.to_be_bytes());
    zero_health_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &zero_health_metadata));
    assert_eq!(
        world.entity_at(37).and_then(super::ClientEntity::health),
        Some(0.0)
    );

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    let entity = world.entity_at(37).expect("health after partial metadata");
    assert_eq!(entity.health(), Some(0.0));
    assert!(entity.is_no_gravity());
}

#[test]
fn local_player_health_metadata_updates_and_respawn_resets_health() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);

    let mut health_metadata = vec![
        8,
        super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
        0,
        super::ENTITY_HEALTH_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_FLOAT_ID as u8,
    ];
    health_metadata.extend_from_slice(&7.5_f32.to_be_bytes());
    health_metadata.push(255);
    assert!(world.apply_entity_metadata(42, &health_metadata));
    assert_eq!(
        world.local_player().map(|player| player.health()),
        Some(7.5)
    );

    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
    assert_eq!(
        world.local_player().map(|player| player.health()),
        Some(20.0)
    );
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
        world.entity_at(37).map(super::ClientEntity::arrow_count),
        Some(0)
    );

    let mut arrow_metadata = vec![
        super::ENTITY_ARROW_COUNT_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    voxel_core::wire::write_varint(3, &mut arrow_metadata);
    arrow_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &arrow_metadata));
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::arrow_count),
        Some(3)
    );

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::arrow_count),
        Some(3)
    );

    assert!(world.apply_entity_metadata(37, &[12, 0, 7, 255]));
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::arrow_count),
        Some(3)
    );
}

#[test]
fn local_player_arrow_count_metadata_follows_entity_data_keep_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);

    let mut arrow_metadata = vec![
        super::ENTITY_ARROW_COUNT_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    voxel_core::wire::write_varint(5, &mut arrow_metadata);
    arrow_metadata.push(255);
    assert!(world.apply_entity_metadata(42, &arrow_metadata));
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
        world.entity_at(37).map(super::ClientEntity::stinger_count),
        Some(0)
    );

    let mut stinger_metadata = vec![
        super::ENTITY_STINGER_COUNT_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    voxel_core::wire::write_varint(2, &mut stinger_metadata);
    stinger_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &stinger_metadata));
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::stinger_count),
        Some(2)
    );

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::stinger_count),
        Some(2)
    );

    assert!(world.apply_entity_metadata(37, &[13, 0, 4, 255]));
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::stinger_count),
        Some(2)
    );
}

#[test]
fn local_player_stinger_count_metadata_follows_entity_data_keep_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);

    let mut stinger_metadata = vec![
        super::ENTITY_STINGER_COUNT_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    voxel_core::wire::write_varint(4, &mut stinger_metadata);
    stinger_metadata.push(255);
    assert!(world.apply_entity_metadata(42, &stinger_metadata));
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
fn local_player_effect_ambience_metadata_follows_entity_data_keep_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert!(world.apply_entity_metadata(42, &[10, 17, 0, 11, 8, 1, 255]));
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
        super::ENTITY_EFFECT_PARTICLES_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_PARTICLES_ID as u8,
        1,
    ];
    voxel_core::wire::write_varint(
        super::ENTITY_EFFECT_PARTICLE_TYPE_ID,
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
        &[super::ENTITY_EFFECT_PARTICLE_TYPE_ID]
    );

    assert!(world.apply_entity_metadata(37, &[5, 8, 1, 255]));
    assert_eq!(
        world
            .entity_at(37)
            .expect("entity after partial particle metadata")
            .effect_particle_type_ids(),
        &[super::ENTITY_EFFECT_PARTICLE_TYPE_ID]
    );

    assert!(world.apply_entity_metadata(37, &[10, 0, 1, 255]));
    assert_eq!(
        world
            .entity_at(37)
            .expect("entity after wrong particle serializer")
            .effect_particle_type_ids(),
        &[super::ENTITY_EFFECT_PARTICLE_TYPE_ID]
    );

    assert!(world.apply_entity_metadata(37, &[10, 17, 1, 0, 255]));
    assert_eq!(
        world
            .entity_at(37)
            .expect("entity after unsupported particle type")
            .effect_particle_type_ids(),
        &[super::ENTITY_EFFECT_PARTICLE_TYPE_ID]
    );
}

#[test]
fn local_player_effect_particle_types_follow_entity_data_keep_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let mut particle_metadata = vec![
        super::ENTITY_EFFECT_PARTICLES_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_PARTICLES_ID as u8,
        1,
    ];
    voxel_core::wire::write_varint(
        super::ENTITY_EFFECT_PARTICLE_TYPE_ID,
        &mut particle_metadata,
    );
    particle_metadata.extend_from_slice(&0x5566_7788_i32.to_be_bytes());
    particle_metadata.push(255);
    assert!(world.apply_entity_metadata(42, &particle_metadata));
    assert_eq!(
        world
            .local_player()
            .expect("particle player")
            .effect_particle_type_ids(),
        &[super::ENTITY_EFFECT_PARTICLE_TYPE_ID]
    );

    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
    assert_eq!(
        world
            .local_player()
            .expect("respawned particle player")
            .effect_particle_type_ids(),
        &[super::ENTITY_EFFECT_PARTICLE_TYPE_ID]
    );

    assert!(world.recreate_local_player(0));
    assert!(
        world
            .local_player()
            .expect("reset particle player")
            .effect_particle_type_ids()
            .is_empty()
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
        super::ENTITY_SLEEPING_POS_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID as u8,
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
fn local_player_sleeping_position_follows_entity_data_keep_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let bed = voxel_core::binary::BlockPos::new(14, 90, -23);
    let mut sleeping_metadata = vec![
        super::ENTITY_SLEEPING_POS_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID as u8,
        1,
    ];
    voxel_core::binary::write_blockpos(bed, &mut sleeping_metadata);
    sleeping_metadata.push(255);
    assert!(world.apply_entity_metadata(42, &sleeping_metadata));
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
            super::ENTITY_LIVING_FLAGS_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
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
            super::ENTITY_LIVING_FLAGS_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
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
fn local_player_living_flags_follow_metadata_and_respawn_keep_mask() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert_eq!(
        world.local_player().map(|player| player.living_flags()),
        Some(0)
    );

    assert!(world.apply_entity_metadata(
        42,
        &[
            super::ENTITY_LIVING_FLAGS_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
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
            super::ENTITY_MOB_FLAGS_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
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
            super::ENTITY_MOB_FLAGS_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
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
        world.entity_at(37).map(super::ClientEntity::mob_flags),
        Some(0b001)
    );
}

#[test]
fn entity_pose_follows_java_pose_metadata_and_maps_unknown_ids_to_standing() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
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
        world.entity_at(37).map(super::ClientEntity::pose),
        Some(super::ClientEntityPose::Standing)
    );

    let expected_poses = [
        super::ClientEntityPose::Standing,
        super::ClientEntityPose::FallFlying,
        super::ClientEntityPose::Sleeping,
        super::ClientEntityPose::Swimming,
        super::ClientEntityPose::SpinAttack,
        super::ClientEntityPose::Crouching,
        super::ClientEntityPose::LongJumping,
        super::ClientEntityPose::Dying,
        super::ClientEntityPose::Croaking,
        super::ClientEntityPose::UsingTongue,
        super::ClientEntityPose::Sitting,
        super::ClientEntityPose::Roaring,
        super::ClientEntityPose::Sniffing,
        super::ClientEntityPose::Emerging,
        super::ClientEntityPose::Digging,
        super::ClientEntityPose::Sliding,
        super::ClientEntityPose::Shooting,
        super::ClientEntityPose::Inhaling,
    ];
    for (id, expected) in expected_poses.into_iter().enumerate() {
        let mut metadata = vec![
            super::ENTITY_POSE_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_POSE_ID as u8,
        ];
        voxel_core::wire::write_varint(id as i32, &mut metadata);
        metadata.push(255);
        assert!(world.apply_entity_metadata(37, &metadata));
        let entity = world.entity_at(37).expect("entity with pose");
        assert_eq!(entity.pose(), expected);
        assert_eq!(entity.pose().id(), id as u8);
        assert_eq!(
            entity.player_dimensions(),
            Some(expected.player_dimensions())
        );
    }

    let combined_metadata = [0, 0, 64, 5, 8, 1, 6, 20, 10, 255];
    assert!(world.apply_entity_metadata(37, &combined_metadata));
    let entity = world.entity_at(37).expect("combined pose entity");
    assert_eq!(entity.pose(), super::ClientEntityPose::Sitting);
    assert!(entity.is_glowing());
    assert!(entity.is_no_gravity());

    assert!(world.apply_entity_metadata(37, &[6, 20, 99, 255]));
    let entity = world.entity_at(37).expect("fallback pose entity");
    assert_eq!(entity.pose(), super::ClientEntityPose::Standing);
    assert_eq!(
        entity.player_dimensions(),
        Some(super::ClientEntityPose::Standing.player_dimensions())
    );
    assert!(entity.is_glowing());
    assert!(entity.is_no_gravity());
}

#[test]
fn entity_ticks_frozen_follows_java_int_metadata_and_derived_state() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 1,
        x: 1.25,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let entity = world.entity_at(37).expect("default freeze entity");
    assert_eq!(entity.ticks_frozen(), 0);
    assert_eq!(entity.percent_frozen(), 0.0);
    assert!(!entity.is_freezing());
    assert!(!entity.is_fully_frozen());

    let mut partial_metadata = vec![
        super::ENTITY_TICKS_FROZEN_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    voxel_core::wire::write_varint(70, &mut partial_metadata);
    partial_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &partial_metadata));
    let entity = world.entity_at(37).expect("partially frozen entity");
    assert_eq!(entity.ticks_frozen(), 70);
    assert_eq!(entity.percent_frozen(), 0.5);
    assert!(entity.is_freezing());
    assert!(!entity.is_fully_frozen());

    let mut combined_metadata = vec![0, 0, 64, 6, 20, 10, 7, 1];
    voxel_core::wire::write_varint(140, &mut combined_metadata);
    combined_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &combined_metadata));
    let entity = world.entity_at(37).expect("fully frozen entity");
    assert_eq!(entity.ticks_frozen(), 140);
    assert_eq!(entity.percent_frozen(), 1.0);
    assert!(entity.is_freezing());
    assert!(entity.is_fully_frozen());
    assert_eq!(entity.pose(), super::ClientEntityPose::Sitting);
    assert!(entity.is_glowing());

    let mut negative_metadata = vec![
        super::ENTITY_TICKS_FROZEN_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_INT_ID as u8,
    ];
    voxel_core::wire::write_varint(-1, &mut negative_metadata);
    negative_metadata.push(255);
    assert!(world.apply_entity_metadata(37, &negative_metadata));
    let entity = world.entity_at(37).expect("negative freeze entity");
    assert_eq!(entity.ticks_frozen(), -1);
    assert_eq!(entity.percent_frozen(), -1.0 / 140.0);
    assert!(!entity.is_freezing());
    assert!(!entity.is_fully_frozen());
    assert_eq!(entity.pose(), super::ClientEntityPose::Sitting);
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
        type_id: super::VANILLA_PLAYER_ENTITY_TYPE_ID,
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
        world.entity_at(42).map(super::ClientEntity::tick_count),
        Some(1)
    );
}

#[test]
fn tick_counter_is_independent_of_chunk_storage() {
    let mut world = ClientWorld::default();
    assert_eq!(world.game_time(), 0);
    world.tick();
    world.tick();
    assert_eq!(world.game_time(), 2);
    assert_eq!(world.loaded_chunk_count(), 0);
}

#[test]
fn ticking_snowballs_applies_java_free_flight_gravity_drag_and_history() {
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
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_SNOWBALL_ENTITY_TYPE_ID,
        x: 2.0,
        y: 1.0,
        z: 2.0,
        movement: (0.25, 0.5, 0.25),
        rotation: (10.0, -20.0, 30.0),
        data: 0,
    }));
    assert!(world.apply_entity(EntityInput {
        entity_id: 38,
        uuid: Uuid::new(3, 4),
        type_id: VANILLA_SNOWBALL_ENTITY_TYPE_ID,
        x: 4.0,
        y: 1.0,
        z: 2.0,
        movement: (0.0, 0.0, 1.0),
        rotation: (45.0, 30.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity_metadata(
        38,
        &[
            ENTITY_NO_GRAVITY_DATA_ID,
            ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
            1,
            255,
        ],
    ));

    world.tick();

    let air_drag = f64::from(0.99_f32);
    let expected_motion = (
        0.25 * air_drag,
        (0.5 - SNOWBALL_GRAVITY) * air_drag,
        0.25 * air_drag,
    );
    let entity = world.entity_at(37).expect("gravity snowball");
    assert_eq!(entity.previous_position(), (2.0, 1.0, 2.0));
    assert_eq!(entity.motion(), expected_motion);
    assert_eq!(
        entity.position(),
        (
            2.0 + expected_motion.0,
            1.0 + expected_motion.1,
            2.0 + expected_motion.2
        )
    );
    assert_eq!(entity.tick_count(), 1);

    let entity = world.entity_at(38).expect("no-gravity snowball");
    assert_eq!(entity.motion(), (0.0, 0.0, air_drag));
    assert_eq!(entity.position(), (4.0, 1.0, 2.0 + air_drag));
    assert_eq!(entity.previous_position(), (4.0, 1.0, 2.0));
    assert_eq!(entity.rotation(), (36.0, 24.0, 0.0));
}

#[test]
fn ticking_arrows_applies_java_free_flight_inertia_gravity_and_history() {
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
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_ARROW_ENTITY_TYPE_ID,
        x: 2.0,
        y: 1.0,
        z: 2.0,
        movement: (0.25, 0.5, 0.25),
        rotation: (10.0, -20.0, 30.0),
        data: 0,
    }));
    assert!(world.apply_entity(EntityInput {
        entity_id: 38,
        uuid: Uuid::new(3, 4),
        type_id: VANILLA_ARROW_ENTITY_TYPE_ID,
        x: 4.0,
        y: 1.0,
        z: 2.0,
        movement: (0.0, 0.0, 1.0),
        rotation: (45.0, 30.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity_metadata(
        38,
        &[
            ENTITY_NO_GRAVITY_DATA_ID,
            ENTITY_DATA_SERIALIZER_BOOLEAN_ID as u8,
            1,
            255,
        ],
    ));

    world.tick();

    let air_drag = f64::from(0.99_f32);
    let expected_motion = (0.25 * air_drag, 0.5 * air_drag - 0.05, 0.25 * air_drag);
    let entity = world.entity_at(37).expect("gravity arrow");
    assert_eq!(entity.previous_position(), (2.0, 1.0, 2.0));
    assert_eq!(entity.motion(), expected_motion);
    assert_eq!(entity.position(), (2.0 + 0.25, 1.0 + 0.5, 2.0 + 0.25));
    assert_eq!(entity.tick_count(), 1);

    let entity = world.entity_at(38).expect("no-gravity arrow");
    assert_eq!(entity.motion(), (0.0, 0.0, air_drag));
    assert_eq!(entity.position(), (4.0, 1.0, 3.0));
    assert_eq!(entity.previous_position(), (4.0, 1.0, 2.0));
    assert_eq!(entity.rotation(), (36.0, 24.0, 0.0));
}

#[test]
fn experience_orb_first_client_tick_then_applies_gravity_drag_and_motion() {
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
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_EXPERIENCE_ORB_ENTITY_TYPE_ID,
        x: 2.0,
        y: 10.0,
        z: 2.0,
        movement: (0.125, 0.25, -0.125),
        rotation: (10.0, 20.0, 30.0),
        data: 1,
    }));

    world.tick();

    let entity = world
        .entity_at(37)
        .expect("experience orb after first tick");
    assert_eq!(entity.position(), (2.0, 10.0, 2.0));
    assert_eq!(entity.motion(), (0.125, 0.25, -0.125));
    assert_eq!(entity.previous_position(), (2.0, 10.0, 2.0));
    assert_eq!(entity.tick_count(), 1);

    world.tick();

    let gravity_motion = (0.125, 0.25 - 0.03, -0.125);
    let air_drag = f64::from(0.98_f32);
    let entity = world
        .entity_at(37)
        .expect("experience orb after second tick");
    assert_eq!(entity.previous_position(), (2.0, 10.0, 2.0));
    assert_eq!(
        entity.position(),
        (
            2.0 + gravity_motion.0,
            10.0 + gravity_motion.1,
            2.0 + gravity_motion.2,
        )
    );
    assert_eq!(
        entity.motion(),
        (
            gravity_motion.0 * air_drag,
            gravity_motion.1 * air_drag,
            gravity_motion.2 * air_drag,
        )
    );
    assert_eq!(entity.tick_count(), 2);
}

#[test]
fn ticking_primed_tnt_applies_gravity_move_and_air_drag_in_order() {
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
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_PRIMED_TNT_ENTITY_TYPE_ID,
        x: 2.0,
        y: 10.0,
        z: 2.0,
        movement: (0.25, 0.5, -0.125),
        rotation: (10.0, 20.0, 30.0),
        data: 0,
    }));

    world.tick();

    let gravity_motion = (0.25, 0.5 - VANILLA_PRIMED_TNT_GRAVITY, -0.125);
    let air_drag = f64::from(VANILLA_PRIMED_TNT_AIR_FRICTION);
    let entity = world.entity_at(37).expect("primed TNT after tick");
    assert_eq!(entity.previous_position(), (2.0, 10.0, 2.0));
    assert_eq!(
        entity.position(),
        (
            2.0 + gravity_motion.0,
            10.0 + gravity_motion.1,
            2.0 + gravity_motion.2,
        )
    );
    assert_eq!(
        entity.motion(),
        (
            gravity_motion.0 * air_drag,
            gravity_motion.1 * air_drag,
            gravity_motion.2 * air_drag,
        )
    );
    assert_eq!(entity.rotation(), (10.0, 20.0, 30.0));
    assert_eq!(entity.tick_count(), 1);
}

#[test]
fn ticking_falling_blocks_applies_gravity_and_air_drag_after_move() {
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
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_FALLING_BLOCK_ENTITY_TYPE_ID,
        x: 2.0,
        y: 10.0,
        z: 2.0,
        movement: (0.25, 0.5, -0.125),
        rotation: (10.0, 20.0, 30.0),
        data: 0,
    }));

    world.tick();

    let gravity_motion = (0.25, 0.5 - VANILLA_FALLING_BLOCK_GRAVITY, -0.125);
    let entity = world.entity_at(37).expect("falling block after tick");
    assert_eq!(entity.previous_position(), (2.0, 10.0, 2.0));
    assert_eq!(
        entity.position(),
        (
            2.0 + gravity_motion.0,
            10.0 + gravity_motion.1,
            2.0 + gravity_motion.2,
        )
    );
    let air_drag = f64::from(VANILLA_FALLING_BLOCK_AIR_FRICTION);
    assert_eq!(
        entity.motion(),
        (
            gravity_motion.0 * air_drag,
            gravity_motion.1 * air_drag,
            gravity_motion.2 * air_drag,
        )
    );
    assert_eq!(entity.rotation(), (10.0, 20.0, 30.0));
    assert_eq!(entity.tick_count(), 1);
}

#[test]
fn ticking_firework_rockets_accelerates_and_rotates_from_free_flight_motion() {
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
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_FIREWORK_ROCKET_ENTITY_TYPE_ID,
        x: 2.0,
        y: 10.0,
        z: 2.0,
        movement: (0.25, 0.5, -0.125),
        rotation: (10.0, 20.0, 30.0),
        data: 0,
    }));

    world.tick();

    let movement = (
        0.25 * VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
        0.5 + VANILLA_FIREWORK_ROCKET_VERTICAL_ACCELERATION,
        -0.125 * VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION,
    );
    let horizontal_distance = (movement.0 * movement.0 + movement.2 * movement.2).sqrt();
    let target_x_rotation =
        (mth_atan2(movement.1, horizontal_distance) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
    let target_y_rotation =
        (mth_atan2(movement.0, movement.2) * PROJECTILE_DEGREES_PER_RADIAN) as f32;
    let expected_rotation = (
        projectile_lerp_rotation_f32(10.0, target_y_rotation),
        projectile_lerp_rotation_f32(20.0, target_x_rotation),
        30.0,
    );
    let entity = world.entity_at(37).expect("firework rocket after tick");
    assert_eq!(entity.previous_position(), (2.0, 10.0, 2.0));
    assert_eq!(
        entity.position(),
        (2.0 + movement.0, 10.0 + movement.1, 2.0 + movement.2,)
    );
    assert_eq!(entity.motion(), movement);
    assert_eq!(entity.rotation(), expected_rotation);
    assert_eq!(entity.tick_count(), 1);
}

#[test]
fn tick_counters_follow_local_player_and_entity_ticking_eligibility() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
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
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert_eq!(
        world.local_player().map(|player| player.tick_count()),
        Some(0)
    );
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::tick_count),
        Some(0)
    );

    world.tick();
    assert_eq!(
        world.local_player().map(|player| player.tick_count()),
        Some(1)
    );
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::tick_count),
        Some(1)
    );

    assert!(world.unload_chunk(0, -3));
    world.tick();
    assert_eq!(
        world.local_player().map(|player| player.tick_count()),
        Some(2)
    );
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::tick_count),
        Some(1)
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
    world.tick();
    assert_eq!(
        world.local_player().map(|player| player.tick_count()),
        Some(3)
    );
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::tick_count),
        Some(2)
    );

    assert!(world.recreate_local_player(0));
    assert_eq!(
        world.local_player().map(|player| player.tick_count()),
        Some(0)
    );
    assert_eq!(
        world.local_player().map(|player| player.generation()),
        Some(1)
    );
}

#[test]
fn tick_captures_previous_transform_for_position_and_rotation_interpolation() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert!(
        world.apply_local_player_position((10.0, 64.0, -2.0), (0.0, 0.0, 0.0), (350.0, -20.0),)
    );
    assert_eq!(
        world
            .local_player()
            .map(super::ClientPlayerState::previous_position),
        Some((10.0, 64.0, -2.0))
    );
    assert_eq!(
        world
            .local_player()
            .map(super::ClientPlayerState::previous_rotation),
        Some((350.0, -20.0))
    );

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
        movement: (0.0, 0.0, 0.0),
        rotation: (350.0, -20.0, 350.0),
        data: 0,
    }));

    world.tick();
    assert_eq!(
        world
            .local_player()
            .map(super::ClientPlayerState::previous_position),
        Some((10.0, 64.0, -2.0))
    );
    assert_eq!(
        world
            .entity_at(37)
            .map(super::ClientEntity::previous_position),
        Some((1.25, 80.0, -35.5))
    );

    assert!(world.apply_local_player_position((14.0, 68.0, 6.0), (0.0, 0.0, 0.0), (10.0, 20.0),));
    let player = world.local_player().expect("local player transform");
    assert_eq!(player.previous_position(), (10.0, 64.0, -2.0));
    assert_eq!(player.interpolated_position(0.5), (12.0, 66.0, 2.0));
    assert_eq!(player.previous_rotation(), (350.0, -20.0));
    assert_eq!(player.interpolated_rotation(0.5), (360.0, 0.0));
    assert_eq!(player.interpolated_rotation(1.0), (10.0, 20.0));

    assert!(world.apply_entity_position(EntityPositionInput {
        entity_id: 37,
        x: 3.0,
        y: 82.0,
        z: -34.5,
        relative_flags: 0,
    }));
    assert!(world.apply_entity_rotation(EntityRotationInput {
        entity_id: 37,
        y_rot: Some(10.0),
        x_rot: Some(20.0),
        y_head_rot: Some(10.0),
        relative_y_rot: false,
        relative_x_rot: false,
    }));
    let entity = world.entity_at(37).expect("remote entity transform");
    assert_eq!(entity.previous_position(), (1.25, 80.0, -35.5));
    assert_eq!(entity.interpolated_position(0.5), (2.125, 81.0, -35.0));
    assert_eq!(entity.previous_rotation(), (350.0, -20.0, 350.0));
    assert_eq!(entity.interpolated_rotation(0.5), (360.0, 0.0, 360.0));

    assert!(world.unload_chunk(0, -3));
    assert!(world.apply_entity_position(EntityPositionInput {
        entity_id: 37,
        x: 4.0,
        y: 82.0,
        z: -34.5,
        relative_flags: 0,
    }));
    world.tick();
    assert_eq!(
        world
            .entity_at(37)
            .map(super::ClientEntity::previous_position),
        Some((1.25, 80.0, -35.5))
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
    world.tick();
    assert_eq!(
        world
            .entity_at(37)
            .map(super::ClientEntity::previous_position),
        Some((4.0, 82.0, -34.5))
    );
}

#[test]
fn remote_player_position_interpolation_follows_java_three_step_handler() {
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
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 1.0,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (350.0, -20.0, 350.0),
        data: 0,
    }));

    assert!(world.apply_entity_position(EntityPositionInput {
        entity_id: 37,
        x: 4.0,
        y: 82.0,
        z: -34.5,
        relative_flags: 0,
    }));
    assert_eq!(
        world.entity_at(37).map(super::ClientEntity::position),
        Some((1.0, 80.0, -35.5))
    );
    assert_eq!(
        world
            .entity_at(37)
            .and_then(super::ClientEntity::interpolation_target_position),
        Some((4.0, 82.0, -34.5))
    );
    assert!(world.apply_entity_rotation(EntityRotationInput {
        entity_id: 37,
        y_rot: Some(10.0),
        x_rot: Some(20.0),
        y_head_rot: None,
        relative_y_rot: false,
        relative_x_rot: false,
    }));
    assert_eq!(
        world
            .entity_at(37)
            .and_then(super::ClientEntity::interpolation_target_rotation),
        Some((10.0, 20.0))
    );

    world.tick();
    let first = world.entity_at(37).expect("first interpolation step");
    assert_eq!(first.interpolation_steps(), 2);
    assert_eq!(
        first.position(),
        (2.0, 80.66666666666667, -35.166666666666664)
    );
    assert_eq!(first.rotation(), (356.666_66_f32, -6.6666665_f32, 350.0));

    world.tick();
    let second = world.entity_at(37).expect("second interpolation step");
    assert_eq!(second.interpolation_steps(), 1);
    assert_eq!(
        second.position(),
        (3.0, 81.33333333333334, -34.83333333333333)
    );
    assert_eq!(second.rotation(), (3.333313_f32, 6.666667_f32, 350.0));

    world.tick();
    let third = world.entity_at(37).expect("final interpolation step");
    assert_eq!(third.interpolation_steps(), 0);
    assert_eq!(third.position(), (4.0, 82.0, -34.5));
    assert_eq!(third.rotation(), (10.0, 20.0, 350.0));
}

#[test]
fn remote_player_motion_packet_interpolates_for_three_ticks() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 1.0,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));

    let target = (0.3, -0.6, 0.9);
    assert!(world.apply_entity_motion(EntityMotionInput {
        entity_id: 37,
        x: target.0,
        y: target.1,
        z: target.2,
    }));
    let entity = world.entity_at(37).expect("remote player motion target");
    assert_eq!(entity.motion(), (0.0, 0.0, 0.0));
    assert_eq!(entity.motion_interpolation_target(), Some(target));
    assert_eq!(entity.motion_interpolation_steps(), 3);

    let first = (target.0 / 3.0, target.1 / 3.0, target.2 / 3.0);
    world.tick();
    let entity = world
        .entity_at(37)
        .expect("first remote player motion step");
    assert_eq!(entity.motion(), first);
    assert_eq!(entity.motion_interpolation_steps(), 2);

    let second = (
        first.0 + (target.0 - first.0) / 2.0,
        first.1 + (target.1 - first.1) / 2.0,
        first.2 + (target.2 - first.2) / 2.0,
    );
    world.tick();
    let entity = world
        .entity_at(37)
        .expect("second remote player motion step");
    assert_eq!(entity.motion(), second);
    assert_eq!(entity.motion_interpolation_steps(), 1);

    let third = (
        second.0 + (target.0 - second.0),
        second.1 + (target.1 - second.1),
        second.2 + (target.2 - second.2),
    );
    world.tick();
    let entity = world
        .entity_at(37)
        .expect("final remote player motion step");
    assert_eq!(entity.motion(), third);
    assert_eq!(entity.motion(), target);
    assert_eq!(entity.motion_interpolation_target(), None);
    assert_eq!(entity.motion_interpolation_steps(), 0);
}

#[test]
fn entity_teleport_applies_relative_delta_motion_and_rotation() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 37,
        uuid: Uuid::new(1, 2),
        type_id: 1,
        x: 10.0,
        y: 64.0,
        z: 20.0,
        movement: (1.0, 2.0, 3.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let relative_flags = ENTITY_RELATIVE_X_FLAG
        | ENTITY_RELATIVE_Y_FLAG
        | ENTITY_RELATIVE_Z_FLAG
        | ENTITY_RELATIVE_Y_ROT_FLAG
        | ENTITY_RELATIVE_X_ROT_FLAG
        | ENTITY_RELATIVE_DELTA_X_FLAG
        | ENTITY_RELATIVE_DELTA_Y_FLAG
        | ENTITY_RELATIVE_DELTA_Z_FLAG;
    assert!(world.apply_entity_teleport(EntityTeleportInput {
        entity_id: 37,
        x: 2.0,
        y: 3.0,
        z: 4.0,
        delta_x: 0.25,
        delta_y: -0.5,
        delta_z: 0.75,
        y_rot: 15.0,
        x_rot: 10.0,
        relative_flags,
    }));
    let entity = world.entity_at(37).expect("relative teleport entity");
    assert_eq!(entity.position(), (12.0, 67.0, 24.0));
    assert_eq!(entity.rotation(), (15.0, 10.0, 0.0));
    assert_eq!(entity.motion(), (1.25, 1.5, 3.75));

    assert!(world.apply_entity(EntityInput {
        entity_id: 38,
        uuid: Uuid::new(3, 4),
        type_id: 1,
        x: 10.0,
        y: 64.0,
        z: 20.0,
        movement: (1.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    let rotate_flags = ENTITY_RELATIVE_Y_ROT_FLAG
        | ENTITY_RELATIVE_DELTA_X_FLAG
        | ENTITY_RELATIVE_DELTA_Z_FLAG
        | ENTITY_RELATIVE_ROTATE_DELTA_FLAG;
    assert!(world.apply_entity_teleport(EntityTeleportInput {
        entity_id: 38,
        x: 10.0,
        y: 64.0,
        z: 20.0,
        delta_x: 0.0,
        delta_y: 0.0,
        delta_z: 0.0,
        y_rot: 90.0,
        x_rot: 0.0,
        relative_flags: rotate_flags,
    }));
    let entity = world.entity_at(38).expect("rotated teleport entity");
    let radians = (f64::from(-90.0_f32) * (std::f64::consts::PI / 180.0)) as f32;
    let expected_x = f64::from(mth_cos_f64(f64::from(radians)));
    let expected_z = -f64::from(mth_sin_f64(f64::from(radians)));
    assert_eq!(entity.rotation(), (90.0, 0.0, 0.0));
    assert_eq!(entity.motion(), (expected_x, 0.0, expected_z));

    assert!(world.apply_entity(EntityInput {
        entity_id: 39,
        uuid: Uuid::new(5, 6),
        type_id: 1,
        x: 10.0,
        y: 64.0,
        z: 20.0,
        movement: (0.5, 0.25, -0.75),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    assert!(world.apply_entity_teleport(EntityTeleportInput {
        entity_id: 39,
        x: 10.0,
        y: 64.0,
        z: 20.0,
        delta_x: f64::NAN,
        delta_y: 0.0,
        delta_z: 0.0,
        y_rot: 0.0,
        x_rot: 0.0,
        relative_flags: ENTITY_RELATIVE_DELTA_X_FLAG,
    }));
    assert_eq!(
        world
            .entity_at(39)
            .expect("nonfinite teleport entity")
            .motion(),
        (0.5, 0.25, -0.75)
    );
}

#[test]
fn remote_player_position_sync_snaps_when_distance_exceeds_java_threshold() {
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
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 1.0,
        y: 80.0,
        z: -35.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (350.0, -20.0, 350.0),
        data: 0,
    }));

    assert!(world.apply_entity_position_and_rotation(
        EntityPositionInput {
            entity_id: 37,
            x: 66.0,
            y: 80.0,
            z: -35.5,
            relative_flags: 0,
        },
        EntityRotationInput {
            entity_id: 37,
            y_rot: Some(10.0),
            x_rot: Some(20.0),
            y_head_rot: None,
            relative_y_rot: false,
            relative_x_rot: false,
        },
        EntityPositionUpdateMode::PositionSync,
    ));
    let entity = world.entity_at(37).expect("snapped remote player");
    assert_eq!(entity.position(), (66.0, 80.0, -35.5));
    assert_eq!(entity.previous_position(), (66.0, 80.0, -35.5));
    assert_eq!(entity.rotation(), (10.0, 20.0, 350.0));
    assert_eq!(entity.previous_rotation(), (10.0, 20.0, 350.0));
    assert_eq!(entity.interpolation_steps(), 0);
    assert_eq!(entity.interpolation_target_position(), None);
    assert_eq!(entity.interpolation_target_rotation(), None);

    world.tick();
    let entity = world.entity_at(37).expect("snapped remote player tick");
    assert_eq!(entity.position(), (66.0, 80.0, -35.5));
    assert_eq!(entity.previous_position(), (66.0, 80.0, -35.5));
    assert_eq!(entity.rotation(), (10.0, 20.0, 350.0));

    assert!(world.apply_entity_position_and_rotation(
        EntityPositionInput {
            entity_id: 37,
            x: 130.0,
            y: 80.0,
            z: -35.5,
            relative_flags: 0,
        },
        EntityRotationInput {
            entity_id: 37,
            y_rot: Some(30.0),
            x_rot: Some(10.0),
            y_head_rot: None,
            relative_y_rot: false,
            relative_x_rot: false,
        },
        EntityPositionUpdateMode::PositionSync,
    ));
    let entity = world.entity_at(37).expect("threshold remote player");
    assert_eq!(entity.position(), (66.0, 80.0, -35.5));
    assert_eq!(
        entity.interpolation_target_position(),
        Some((130.0, 80.0, -35.5))
    );
    assert_eq!(entity.interpolation_target_rotation(), Some((30.0, 10.0)));
    assert_eq!(entity.interpolation_steps(), 3);

    assert!(world.apply_entity_position_and_rotation(
        EntityPositionInput {
            entity_id: 37,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            relative_flags: 0b111,
        },
        EntityRotationInput {
            entity_id: 37,
            y_rot: Some(20.0),
            x_rot: Some(100.0),
            y_head_rot: None,
            relative_y_rot: true,
            relative_x_rot: true,
        },
        EntityPositionUpdateMode::Teleport,
    ));
    let entity = world.entity_at(37).expect("relative teleport target");
    assert_eq!(
        entity.interpolation_target_position(),
        Some((130.0, 80.0, -35.5))
    );
    assert_eq!(entity.interpolation_target_rotation(), Some((50.0, 90.0)));
}

#[test]
fn local_player_respawn_recreates_identity_and_applies_entity_data_mask() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert_eq!(world.local_player().map(|player| player.id()), Some(42));
    assert_eq!(
        world.local_player().map(|player| player.generation()),
        Some(0)
    );

    assert!(world.apply_local_player_position(
        (10.5, 64.0, -3.25),
        (0.125, -0.25, 0.5),
        (135.0, -30.0),
    ));
    assert!(world.apply_entity_metadata(42, &[0, 0, 32, 255]));
    let mut air_metadata = vec![1, 1];
    voxel_core::wire::write_varint(250, &mut air_metadata);
    air_metadata.push(255);
    assert!(world.apply_entity_metadata(42, &air_metadata));
    let no_gravity_metadata = [5, 8, 1, 255];
    assert!(world.apply_entity_metadata(42, &no_gravity_metadata));
    let silent_metadata = [4, 8, 1, 255];
    assert!(world.apply_entity_metadata(42, &silent_metadata));
    let mut custom_name_metadata = vec![2, 6, 1];
    voxel_core::nbt::write_any_nbt(
        Some(&voxel_core::nbt::FieldValue::Str("Player name".to_owned())),
        &mut custom_name_metadata,
    );
    custom_name_metadata.push(255);
    assert!(world.apply_entity_metadata(42, &custom_name_metadata));
    let custom_name_visible_metadata = [3, 8, 1, 255];
    assert!(world.apply_entity_metadata(42, &custom_name_visible_metadata));
    let pose_metadata = [6, 20, 10, 255];
    assert!(world.apply_entity_metadata(42, &pose_metadata));
    let ticks_frozen_metadata = [7, 1, 70, 255];
    assert!(world.apply_entity_metadata(42, &ticks_frozen_metadata));

    assert!(world.recreate_local_player(
        PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS | PLAYER_RESPAWN_KEEP_ENTITY_DATA
    ));
    let player = world.local_player().expect("respawned local player");
    assert_eq!(player.id(), 42);
    assert_eq!(player.generation(), 1);
    assert_eq!(player.position(), (0.0, 0.0, 0.0));
    assert_eq!(player.motion(), (0.125, -0.25, 0.5));
    assert_eq!(player.rotation(), (135.0, -30.0));
    assert_eq!(player.entity_data(), Some(ticks_frozen_metadata.as_slice()));
    assert_eq!(player.shared_flags(), 32);
    assert!(player.is_invisible());
    assert_eq!(player.air_supply(), 250);
    assert!(player.is_no_gravity());
    assert!(player.is_silent());
    assert!(player.has_custom_name());
    assert!(player.is_custom_name_visible());
    assert!(player.should_show_name());
    assert_eq!(player.pose(), super::ClientEntityPose::Sitting);
    assert_eq!(player.ticks_frozen(), 70);
    assert!(player.is_freezing());

    assert!(world.recreate_local_player(0));
    let player = world.local_player().expect("reset local player");
    assert_eq!(player.id(), 42);
    assert_eq!(player.generation(), 2);
    assert_eq!(player.motion(), (0.0, 0.0, 0.0));
    assert_eq!(player.rotation(), (-180.0, 0.0));
    assert_eq!(player.entity_data(), None);
    assert_eq!(player.shared_flags(), 0);
    assert!(!player.is_invisible());
    assert_eq!(player.air_supply(), 300);
    assert!(!player.is_no_gravity());
    assert!(!player.is_silent());
    assert!(!player.has_custom_name());
    assert!(!player.is_custom_name_visible());
    assert!(player.should_show_name());
    assert_eq!(player.pose(), super::ClientEntityPose::Standing);
    assert_eq!(player.ticks_frozen(), 0);
    assert!(!player.is_freezing());
}

#[test]
fn local_player_respawn_preserves_attribute_bases_and_applies_modifier_mask() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let modifier =
        ClientAttributeModifierState::from_wire("minecraft:test_modifier".to_owned(), 0.25, 2);
    let attribute = ClientAttributeState::from_wire(17, 2.5, vec![modifier.clone()]);

    assert!(world.apply_local_player_attributes(42, &[attribute]));
    let player = world.local_player().expect("initial local player");
    assert_eq!(player.attributes().len(), 1);
    assert_eq!(player.attributes()[0].base_value(), 2.5);
    assert_eq!(
        player.attributes()[0].modifiers(),
        std::slice::from_ref(&modifier)
    );

    assert!(world.recreate_local_player(0));
    let player = world.local_player().expect("base-only respawned player");
    assert_eq!(player.attributes().len(), 1);
    assert_eq!(player.attributes()[0].base_value(), 2.5);
    assert!(player.attributes()[0].modifiers().is_empty());

    assert!(world.apply_local_player_attributes(
        42,
        &[ClientAttributeState::from_wire(
            17,
            3.5,
            vec![modifier.clone()],
        )],
    ));
    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS));
    let player = world.local_player().expect("modifier-preserving player");
    assert_eq!(player.attributes()[0].base_value(), 3.5);
    assert_eq!(player.attributes()[0].modifiers(), &[modifier]);
}

#[test]
fn attribute_effective_value_matches_java_operation_order_and_sanitization() {
    let attribute = ClientAttributeState::from_wire(
        26,
        0.25,
        vec![
            ClientAttributeModifierState::from_wire("minecraft:oracle_add".to_owned(), 0.1, 0),
            ClientAttributeModifierState::from_wire("minecraft:oracle_base".to_owned(), 0.2, 1),
            ClientAttributeModifierState::from_wire("minecraft:oracle_total".to_owned(), 0.5, 2),
        ],
    );
    assert_eq!(
        attribute.effective_value(0.0, 1024.0).to_bits(),
        0x3fe4_28f5_c28f_5c29
    );

    let unknown_operation = ClientAttributeState::from_wire(
        26,
        1.0,
        vec![ClientAttributeModifierState::from_wire(
            "minecraft:oracle_unknown".to_owned(),
            0.5,
            255,
        )],
    );
    assert_eq!(unknown_operation.effective_value(0.0, 1024.0), 1.5);

    let clamped = ClientAttributeState::from_wire(26, 2048.0, Vec::new());
    assert_eq!(clamped.effective_value(0.0, 1024.0), 1024.0);

    let nan = ClientAttributeState::from_wire(26, f64::NAN, Vec::new());
    assert_eq!(nan.effective_value(0.0, 1024.0), 0.0);
}

#[test]
fn local_player_mob_effects_replace_remove_and_clear_on_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);

    assert!(world.apply_local_player_mob_effect(42, 18, 0, 1_200, 0b1000));
    let player = world.local_player().expect("initial local player");
    assert_eq!(player.effects().len(), 1);
    assert_eq!(player.effects()[0].effect_id(), 18);
    assert_eq!(player.effects()[0].amplifier(), 0);
    assert_eq!(player.effects()[0].duration_ticks(), 1_200);
    assert_eq!(player.effects()[0].flags(), 0b1000);

    assert!(world.apply_local_player_mob_effect(42, 18, 1, 60, 0b1111));
    let player = world.local_player().expect("updated local player");
    assert_eq!(player.effects().len(), 1);
    assert_eq!(player.effects()[0].amplifier(), 1);
    assert_eq!(player.effects()[0].duration_ticks(), 60);
    assert_eq!(player.effects()[0].flags(), 0b1111);

    assert!(!world.apply_local_player_mob_effect(7, 19, 0, 20, 0));
    assert!(world.remove_local_player_mob_effect(42, 18));
    assert!(
        world
            .local_player()
            .expect("local player after removal")
            .effects()
            .is_empty()
    );
    assert!(world.apply_local_player_mob_effect(42, 18, 0, 1_200, 0b1000));
    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS));
    assert!(
        world
            .local_player()
            .expect("respawned local player")
            .effects()
            .is_empty()
    );
}

#[test]
fn local_player_mob_effect_durations_tick_without_removing_expired_entries() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert!(world.apply_local_player_mob_effect(42, 18, 0, 2, 0));
    assert!(world.apply_local_player_mob_effect(42, 19, 0, -1, 0));
    assert!(world.apply_local_player_mob_effect(42, 20, 0, 0, 0));
    assert!(world.apply_local_player_mob_effect(42, 21, 0, -2, 0));

    let player = world.local_player().expect("local player effects");
    assert!(
        player
            .effects()
            .iter()
            .find(|effect| effect.effect_id() == 18)
            .is_some_and(|effect| effect.has_remaining_duration())
    );
    assert!(
        player
            .effects()
            .iter()
            .find(|effect| effect.effect_id() == 19)
            .is_some_and(|effect| effect.has_remaining_duration())
    );
    assert!(
        player
            .effects()
            .iter()
            .find(|effect| effect.effect_id() == 20)
            .is_some_and(|effect| !effect.has_remaining_duration())
    );

    world.tick();
    let player = world
        .local_player()
        .expect("local player after first effect tick");
    assert_eq!(
        player
            .effects()
            .iter()
            .find(|effect| effect.effect_id() == 18)
            .map(|effect| effect.duration_ticks()),
        Some(1)
    );
    assert_eq!(
        player
            .effects()
            .iter()
            .find(|effect| effect.effect_id() == 19)
            .map(|effect| effect.duration_ticks()),
        Some(-1)
    );

    world.tick();
    let player = world
        .local_player()
        .expect("local player after effect expiry");
    assert_eq!(player.effects().len(), 4);
    assert_eq!(
        player
            .effects()
            .iter()
            .find(|effect| effect.effect_id() == 18)
            .map(|effect| effect.duration_ticks()),
        Some(0)
    );
    assert!(
        player
            .effects()
            .iter()
            .all(|effect| effect.effect_id() != 18 || !effect.has_remaining_duration())
    );

    world.tick();
    let player = world
        .local_player()
        .expect("local player after expired effect tick");
    assert_eq!(
        player
            .effects()
            .iter()
            .find(|effect| effect.effect_id() == 18)
            .map(|effect| effect.duration_ticks()),
        Some(0)
    );
}

#[test]
fn local_player_health_state_applies_and_resets_with_new_player() {
    let mut world = ClientWorld::default();
    assert!(!world.apply_local_player_health(0.0, 0, 0.0));
    world.set_local_player_id(42);
    let player = world.local_player().expect("initial local player");
    assert_eq!(player.health(), 20.0);
    assert_eq!(player.food(), 20);
    assert_eq!(player.saturation(), 5.0);

    assert!(world.apply_local_player_health(0.0, 7, 1.5));
    let player = world.local_player().expect("updated local player");
    assert_eq!(player.health(), 0.0);
    assert_eq!(player.food(), 7);
    assert_eq!(player.saturation(), 1.5);

    assert!(world.recreate_local_player(0));
    let player = world.local_player().expect("respawned local player");
    assert_eq!(player.health(), 20.0);
    assert_eq!(player.food(), 20);
    assert_eq!(player.saturation(), 5.0);
}

#[test]
fn local_player_inventory_state_applies_slot_cursor_and_respawn_reset() {
    let mut world = ClientWorld::default();
    let stone = ClientItemStackState::from_wire(8, 1, vec![], vec![]);
    let bottle = ClientItemStackState::from_wire(1, 1_149, vec![], vec![]);
    let mut items = vec![None; 46];
    items[36] = Some(stone.clone());

    assert!(!world.apply_local_player_container_content(0, 1, &items, Some(&stone),));
    world.set_local_player_id(42);
    assert!(world.apply_local_player_container_content(0, 2, &items, Some(&stone),));
    let player = world.local_player().expect("local player inventory");
    assert_eq!(player.inventory_state_id(), 2);
    assert_eq!(player.inventory_slot_count(), 46);
    assert_eq!(player.inventory_slot(36), Some(&stone));
    assert_eq!(player.carried_item(), Some(&stone));

    assert!(world.apply_local_player_container_slot(0, 3, 36, Some(&bottle)));
    assert_eq!(
        world
            .local_player()
            .expect("local player after slot update")
            .inventory_slot(36),
        Some(&bottle)
    );
    assert_eq!(
        world
            .local_player()
            .expect("local player after slot update")
            .inventory_state_id(),
        3
    );

    assert!(world.apply_local_player_carried_item(None));
    assert!(
        world
            .local_player()
            .expect("local player after cursor clear")
            .carried_item()
            .is_none()
    );

    assert!(world.apply_local_player_inventory_slot(0, Some(&stone)));
    assert_eq!(
        world
            .local_player()
            .expect("local player after direct inventory update")
            .inventory_slot(36),
        Some(&stone)
    );
    assert!(!world.apply_local_player_inventory_slot(36, Some(&bottle)));

    assert!(world.recreate_local_player(0));
    let player = world
        .local_player()
        .expect("respawned local player inventory");
    assert_eq!(player.inventory_state_id(), 0);
    assert_eq!(player.inventory_slot_count(), 0);
    assert!(player.inventory_slot(36).is_none());
    assert!(player.carried_item().is_none());
}

#[test]
fn local_player_container_state_applies_open_content_slot_cursor_close_and_reset() {
    let mut world = ClientWorld::default();
    let stone = ClientItemStackState::from_wire(7, 1, vec![], vec![]);
    let bottle = ClientItemStackState::from_wire(1, 1_149, vec![], vec![]);
    let title = [10, 0, 0, 0];
    let mut items = vec![None; 63];
    items[0] = Some(stone.clone());
    items[27] = Some(bottle.clone());

    assert!(!world.apply_local_player_open_menu(7, 2, &title));
    world.set_local_player_id(42);
    assert!(world.apply_local_player_open_menu(7, 2, &title));
    assert!(world.apply_local_player_container_content(7, 4, &items, None));
    let container = world
        .local_player()
        .expect("local player with open container")
        .open_container()
        .expect("open container");
    assert_eq!(container.container_id(), 7);
    assert_eq!(container.menu_type_id(), 2);
    assert_eq!(container.title_nbt(), title);
    assert_eq!(container.top_slot_count(), Some(27));
    assert_eq!(container.slot_count(), 63);
    assert_eq!(container.state_id(), 4);
    assert_eq!(container.slot(0), Some(&stone));
    assert_eq!(container.slot(27), Some(&bottle));
    assert!(container.carried_item().is_none());

    assert!(world.apply_local_player_container_slot(7, 5, 0, None));
    assert!(
        world
            .local_player()
            .expect("local player after custom slot update")
            .open_container()
            .expect("container after slot update")
            .slot(0)
            .is_none()
    );
    assert!(world.apply_local_player_carried_item(Some(&stone)));
    assert_eq!(
        world
            .local_player()
            .expect("local player after custom cursor update")
            .open_container()
            .expect("container after cursor update")
            .carried_item(),
        Some(&stone)
    );
    assert!(world.apply_local_player_container_close(99));
    assert!(
        world
            .local_player()
            .expect("local player after custom container close")
            .open_container()
            .is_none()
    );
    assert!(!world.apply_local_player_container_close(7));

    assert!(world.apply_local_player_open_menu(7, 2, &title));
    assert!(world.recreate_local_player(0));
    let player = world
        .local_player()
        .expect("respawned local player without container");
    assert!(player.open_container().is_none());
}

#[test]
fn an_open_containers_last_slots_are_the_players_inventory() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let stone = ClientItemStackState::from_wire(7, 1, vec![], vec![]);
    let dirt = ClientItemStackState::from_wire(3, 10, vec![], vec![]);
    let mut items = vec![None; 63];
    // A main inventory slot (menu 9) and the first hotbar slot (menu 36),
    // which a three row chest lists at 27 and 54.
    items[27] = Some(stone.clone());
    items[54] = Some(dirt.clone());
    assert!(world.apply_local_player_open_menu(7, 2, &[10, 0, 0, 0]));
    assert!(world.apply_local_player_container_content(7, 1, &items, None));
    let player = world.local_player().expect("player");
    assert_eq!(player.inventory_slot(9), Some(&stone));
    assert_eq!(player.inventory_slot(36), Some(&dirt));

    // A click in the container's menu writes through both ways.
    let (_, _, mut slots, carried, _) = world.local_player_container_menu().expect("menu");
    slots[54] = None;
    world.set_local_player_container_menu(&slots, carried);
    assert!(
        world
            .local_player()
            .expect("player")
            .inventory_slot(36)
            .is_none()
    );

    // And the server changing the inventory shows in the container's menu.
    assert!(world.apply_local_player_inventory_slot(1, Some(&dirt)));
    let (_, _, slots, _, _) = world.local_player_container_menu().expect("menu");
    assert_eq!(slots[55].as_ref(), Some(&dirt));
}

#[test]
fn vanilla_menu_registry_slot_counts_cover_all_builtin_menu_types() {
    let expected = [
        (0, 9),
        (1, 18),
        (2, 27),
        (3, 36),
        (4, 45),
        (5, 54),
        (6, 9),
        (7, 10),
        (8, 3),
        (9, 1),
        (10, 3),
        (11, 5),
        (12, 10),
        (13, 2),
        (14, 3),
        (15, 3),
        (16, 5),
        (17, 1),
        (18, 4),
        (19, 3),
        (20, 27),
        (21, 4),
        (22, 3),
        (23, 3),
        (24, 2),
    ];

    for (menu_type_id, slot_count) in expected {
        assert_eq!(menu_top_slot_count(menu_type_id), Some(slot_count));
    }
    assert_eq!(menu_top_slot_count(-1), None);
    assert_eq!(menu_top_slot_count(25), None);
}

#[test]
fn changing_dimension_keeps_local_player_identity_until_respawn_replacement() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(17);
    let input = DimensionInput {
        key: "minecraft:the_nether",
        dimension_type_id: 3,
        seed: 776,
        is_debug: false,
        is_flat: false,
        sea_level: 32,
    };

    assert!(world.replace_dimension_with_environment(
        input,
        DimensionBounds::new(0, 256),
        DimensionEnvironment::for_builtin_dimension(input.key),
    ));
    assert_eq!(world.local_player().map(|player| player.id()), Some(17));
    assert_eq!(
        world.local_player().map(|player| player.generation()),
        Some(0)
    );
    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
    assert_eq!(world.local_player().map(|player| player.id()), Some(17));
    assert_eq!(
        world.local_player().map(|player| player.generation()),
        Some(1)
    );
}

#[test]
fn changing_dimension_replaces_world_storage_and_builtin_bounds() {
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
    assert!(world.apply_entity(EntityInput {
        entity_id: 9,
        uuid: Uuid::new(5, 6),
        type_id: 71,
        x: 0.0,
        y: 64.0,
        z: 0.0,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    world.tick();

    let input = DimensionInput {
        key: "minecraft:the_nether",
        dimension_type_id: 1,
        seed: 776,
        is_debug: false,
        is_flat: false,
        sea_level: 32,
    };
    assert!(world.replace_dimension_with_environment(
        input,
        DimensionBounds::for_builtin_dimension(input.key),
        DimensionEnvironment::for_builtin_dimension(input.key),
    ));
    assert_eq!(world.dimension_key(), "minecraft:the_nether");
    assert_eq!(world.dimension(), DimensionBounds::new(0, 256));
    assert_eq!(world.dimension_type_id(), 1);
    assert_eq!(world.seed(), 776);
    assert_eq!(world.sea_level(), 32);
    assert_eq!(world.loaded_chunk_count(), 0);
    assert_eq!(world.loaded_entity_count(), 0);
    assert_eq!(world.game_time(), 0);

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
    assert!(!world.replace_dimension_with_environment(
        input,
        DimensionBounds::for_builtin_dimension(input.key),
        DimensionEnvironment::for_builtin_dimension(input.key),
    ));
    assert_eq!(world.loaded_chunk_count(), 1);
    assert_eq!(world.loaded_entity_count(), 0);
}

#[test]
fn a_walking_player_swings_its_limbs_and_turns_its_body_after_the_way_it_walks() {
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 9,
        uuid: Uuid::new(1, 2),
        type_id: VANILLA_PLAYER_ENTITY_TYPE_ID,
        x: 0.5,
        y: 0.0,
        z: 0.5,
        movement: (0.0, 0.0, 0.0),
        // Facing west, the body and the head.
        rotation: (90.0, 0.0, 90.0),
        data: 0,
    }));
    let snapshot = |world: &ClientWorld| world.entity_snapshot(9, 1.0).unwrap();
    assert_eq!(snapshot(&world).walk_speed, 0.0);
    assert_eq!(snapshot(&world).body_yaw, 90.0);
    // Standing still moves nothing.
    world.tick_entities();
    assert_eq!(snapshot(&world).walk_speed, 0.0);
    // Walking south (towards +z), 0.2158 a tick, as the server steps it.
    for _ in 0..12 {
        assert!(world.apply_entity_move(EntityMoveInput {
            entity_id: 9,
            delta_x: 0,
            delta_y: 0,
            delta_z: 884,
            has_position: true,
            y_rot: None,
            x_rot: None,
        }));
        world.tick_entities();
    }
    let walking = snapshot(&world);
    assert!(walking.walk_speed > 0.5, "{walking:?}");
    assert!(walking.walk_position > 3.0, "{walking:?}");
    // The body went round towards the way it walks (yaw 0), but the head
    // (looking west) held it within fifty degrees.
    assert!(
        walking.body_yaw < 90.0 && walking.body_yaw >= 40.0,
        "{walking:?}"
    );
    // Between two ticks the limbs are read part of the way.
    let mid = world.entity_snapshot(9, 0.5).unwrap();
    assert!(mid.walk_position < walking.walk_position);
    assert!(mid.walk_position > walking.walk_position - walking.walk_speed);
    // It stops, and the swing dies away.
    for _ in 0..30 {
        world.tick_entities();
    }
    assert!(snapshot(&world).walk_speed < 0.001);
}

#[test]
fn a_walking_mob_eases_to_each_step_and_faces_where_it_goes() {
    let mut world = ClientWorld::default();
    let (blocks, biomes) = ([0_i32; 4096], [0_i32; 64]);
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
    // A cow, looking west, its body and head.
    assert!(world.apply_entity(EntityInput {
        entity_id: 9,
        uuid: Uuid::new(3, 4),
        type_id: VANILLA_COW_ENTITY_TYPE_ID,
        x: 4.5,
        y: 0.0,
        z: 4.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (90.0, 0.0, 90.0),
        data: 0,
    }));
    assert!(world.apply_entity_move(EntityMoveInput {
        entity_id: 9,
        delta_x: 0,
        delta_y: 0,
        delta_z: 4096,
        has_position: true,
        y_rot: None,
        x_rot: None,
    }));
    // The step is not taken at once: three ticks bring the cow to it.
    let z_after = |world: &ClientWorld| world.entity_at(9).unwrap().position().2;
    assert_eq!(z_after(&world), 4.5);
    let mut seen = Vec::new();
    for _ in 0..3 {
        world.tick_entities();
        seen.push(z_after(&world));
    }
    assert!(
        seen[0] > 4.5 && seen[0] < seen[1] && seen[1] < seen[2],
        "{seen:?}"
    );
    assert!((seen[2] - 5.5).abs() < 1e-9, "{seen:?}");
    // It moved, so its legs swing and its body turned to where it looks (west).
    let moving = world.entity_snapshot(9, 1.0).unwrap();
    assert!(moving.walk_speed > 0.0, "{moving:?}");
    assert_eq!(moving.body_yaw, 90.0);
}

#[test]
fn held_items_follow_the_equipment_slots_and_the_main_arm() {
    use crate::entity::{ClientEntity, EquipmentSlot};
    let mut world = ClientWorld::default();
    let player = voxel_data::entity_type::id("minecraft:player").expect("the player type");
    let mob = voxel_data::entity_type::id("minecraft:zombie").expect("the zombie type");
    for (entity_id, type_id) in [(5, player), (6, mob)] {
        assert!(world.apply_entity(EntityInput {
            entity_id,
            uuid: Uuid::new(1, entity_id as u64),
            type_id,
            x: 0.0,
            y: 80.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let main = ClientItemStackState::new(1, 10);
        let off = ClientItemStackState::new(2, 11);
        assert!(world.apply_entity_equipment(entity_id, EquipmentSlot::MainHand, Some(main)));
        assert!(world.apply_entity_equipment(entity_id, EquipmentSlot::OffHand, Some(off)));
    }
    let id_of = |entity: &ClientEntity, left: bool| {
        entity
            .item_held_by_arm(left)
            .map(|stack| stack.item_registry_id())
    };
    // Right-handed: the main hand is the right arm's.
    let entity = world.entity_at(5).expect("the player");
    assert_eq!(
        (id_of(entity, false), id_of(entity, true)),
        (Some(10), Some(11))
    );
    // The avatar's data names the left arm (`HumanoidArm` id zero).
    assert!(world.apply_entity_metadata(
        5,
        &[
            super::AVATAR_MAIN_HAND_DATA_ID,
            super::HUMANOID_ARM_SERIALIZER_ID as u8,
            0,
            255,
        ],
    ));
    let entity = world.entity_at(5).expect("the player");
    assert!(entity.main_arm_is_left());
    assert_eq!(
        (id_of(entity, false), id_of(entity, true)),
        (Some(11), Some(10))
    );
    // A mob is left-handed by its flag.
    assert!(world.apply_entity_metadata(
        6,
        &[
            super::ENTITY_MOB_FLAGS_DATA_ID,
            super::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
            0b010,
            255,
        ],
    ));
    let entity = world.entity_at(6).expect("the mob");
    assert_eq!(
        (id_of(entity, false), id_of(entity, true)),
        (Some(11), Some(10))
    );
    // An empty slot clears it.
    assert!(world.apply_entity_equipment(6, EquipmentSlot::OffHand, None));
    assert_eq!(id_of(world.entity_at(6).expect("the mob"), false), None);
    assert!(!world.apply_entity_equipment(99, EquipmentSlot::Head, None));
}

#[test]
fn a_cat_lies_on_top_of_a_player_who_sleeps_within_two_blocks_of_it() {
    let cat = voxel_data::entity_type::id("minecraft:cat").expect("a cat");
    let mut world = ClientWorld::default();
    let input = |entity_id, type_id, x| EntityInput {
        entity_id,
        uuid: Uuid::new(1, entity_id as u64),
        type_id,
        x,
        y: 80.0,
        z: 0.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    };
    assert!(world.apply_entity(input(1, cat, 0.5)));
    assert!(world.apply_entity(input(2, VANILLA_PLAYER_ENTITY_TYPE_ID, 2.5)));
    // The cat lies down (`IS_LYING`); the player stands near it, then sleeps.
    assert!(world.apply_entity_metadata(1, &[super::CAT_LYING_DATA_ID, 8, 1, 255]));
    let lies_on_player = |world: &mut ClientWorld| {
        // No chunk is loaded here, which would leave the cat idle.
        world.entities.get_mut(&1).unwrap().ticking = true;
        world.tick_entities();
        world
            .entity_at(1)
            .unwrap()
            .cat()
            .is_lying_on_top_of_sleeping_player()
    };
    assert!(!lies_on_player(&mut world));
    let mut asleep = vec![
        super::ENTITY_SLEEPING_POS_DATA_ID,
        super::ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID as u8,
        1,
    ];
    voxel_core::binary::write_blockpos(voxel_core::binary::BlockPos::new(2, 80, 0), &mut asleep);
    // The sleeping pose (`Pose.SLEEPING`) makes the player's box small.
    asleep.extend([6, 20, 2, 255]);
    assert!(world.apply_entity_metadata(2, &asleep));
    assert!(lies_on_player(&mut world));
    // One that sleeps beyond the box the cat looks in does not count.
    assert!(world.apply_entity(input(3, VANILLA_PLAYER_ENTITY_TYPE_ID, 9.5)));
    assert!(world.apply_entity_metadata(2, &[14, 11, 0, 255]));
    assert!(world.apply_entity_metadata(3, &asleep));
    assert!(!lies_on_player(&mut world));
    // And a cat that does not lie never lies on anyone.
    assert!(world.apply_entity_metadata(2, &asleep));
    assert!(world.apply_entity_metadata(1, &[super::CAT_LYING_DATA_ID, 8, 0, 255]));
    assert!(!lies_on_player(&mut world));
}

#[test]
fn a_guardian_turns_its_head_to_the_local_player_it_aims_at() {
    let guardian = voxel_data::entity_type::id("minecraft:guardian").expect("a guardian");
    let mut world = ClientWorld::default();
    assert!(world.apply_entity(EntityInput {
        entity_id: 1,
        uuid: Uuid::new(1, 1),
        type_id: guardian,
        x: 0.5,
        y: 80.0,
        z: 0.5,
        movement: (0.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0),
        data: 0,
    }));
    // The local player stands six blocks east: the guardian looks along -x.
    world.set_local_player_id(2);
    assert!(world.apply_local_player_position((6.5, 80.0, 0.5), (0.0, 0.0, 0.0), (0.0, 0.0)));
    let mut data = vec![super::GUARDIAN_ATTACK_TARGET_DATA_ID, 1, 2, 255];
    assert!(world.apply_entity_metadata(1, &data));
    // No chunk is loaded here, which would leave the guardian idle.
    world.entities.get_mut(&1).unwrap().ticking = true;
    world.tick_entities();
    let (y_rot, x_rot, head) = world.entity_at(1).unwrap().rotation();
    assert_eq!((head, y_rot), (-90.0, -90.0));
    // The player's eyes are 1.195 blocks above the guardian's, six away.
    assert!(
        (x_rot + (1.195_f32 / 6.0).atan().to_degrees()).abs() < 0.01,
        "{x_rot}"
    );
    // Aiming at nothing the world holds does not look: head and pitch stay.
    let looked = world.entity_at(1).unwrap().rotation();
    data[2] = 9;
    assert!(world.apply_entity_metadata(1, &data));
    world.tick_entities();
    assert_eq!(world.entity_at(1).unwrap().rotation(), looked);
}

#[test]
fn a_leash_knot_stands_above_the_middle_of_the_block_the_packet_names() {
    let knot = voxel_data::entity_type::id("minecraft:leash_knot").expect("leash knot");
    assert_eq!(
        crate::block_attached_position(knot, (10.0, 64.0, -3.0)),
        (10.5, 64.375, -2.5)
    );
    // What does not hang on a block keeps the position it was sent.
    let pig = voxel_data::entity_type::id("minecraft:pig").expect("pig");
    assert_eq!(
        crate::block_attached_position(pig, (10.25, 64.0, -3.75)),
        (10.25, 64.0, -3.75)
    );
}
