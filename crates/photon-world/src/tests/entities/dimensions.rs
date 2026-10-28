use super::super::*;
use crate::ResizeContext;
use photon_core::ids::BlockStateId;
use photon_core::ids::EntityId;

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
    let shapes = BTreeMap::from([(BlockStateId(1), Shapes::block())]);
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
        ResizeContext {
            level_is_client_side: true,
            first_tick: false,
            no_physics: false,
            is_player: false,
        },
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
        ResizeContext {
            level_is_client_side: false,
            first_tick: false,
            no_physics: false,
            is_player: false,
        },
    );
    assert!(server_resize.should_fudge_position);

    let player_resize = refresh_entity_dimensions(
        old_dimensions,
        new_dimensions,
        position,
        ResizeContext {
            level_is_client_side: false,
            first_tick: false,
            no_physics: false,
            is_player: true,
        },
    );
    assert!(!player_resize.should_fudge_position);
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

/// The renderers whose `getBlockLightLevel` answers 15 light their entity at full block light;
/// a pig's does not.
#[test]
fn some_renderers_light_their_entity_at_full_block_light() {
    let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
    let names = [
        ("minecraft:allay", true),
        ("minecraft:blaze", true),
        ("minecraft:magma_cube", true),
        ("minecraft:vex", true),
        ("minecraft:wither", true),
        ("minecraft:dragon_fireball", true),
        ("minecraft:shulker_bullet", true),
        ("minecraft:wither_skull", true),
        ("minecraft:pig", false),
        ("minecraft:slime", false),
        ("minecraft:ghast", false),
    ];
    for (id, (name, _)) in (1..).zip(names) {
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(id),
            uuid: Uuid::new(1, u64::from(id as u32)),
            type_id: photon_data::entity_type::id(name).expect("the type"),
            x: 0.0,
            y: 9.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
    }
    for (id, (name, full)) in (1..).zip(names) {
        let entity = world.entities().at(EntityId(id)).expect("the entity");
        assert_eq!(entity.renderer_full_block_light(), full, "{name}");
    }
}

/// What the camera entity's eyes sit at (`LivingEntity.getDimensions(pose)`): the pose's eye
/// height grown by the scale attribute, a player's too, and the sleeping size left alone. The
/// numbers are the reference's: a pig's default eye is 0.85 of its 0.9 height, a player's is
/// 1.62 standing and 1.27 crouching, a sleeper's is 0.2.
#[test]
fn camera_eye_height_follows_pose_and_scale() {
    let eye = |name: &str, pose: Option<crate::ClientEntityPose>, scale: f64| {
        let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(5),
            uuid: Uuid::new(1, 5),
            type_id: photon_data::entity_type::id(name).expect("the type"),
            x: 0.0,
            y: 9.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let scale_id = photon_data::item::effect::attribute_named("minecraft:scale");
        assert!(world.entities_mut().apply_attributes(
            EntityId(5),
            &[ClientAttributeState::new(scale_id, scale, Vec::new())],
        ));
        if let Some(pose) = pose {
            let mut metadata = vec![
                crate::ENTITY_POSE_DATA_ID,
                crate::ENTITY_DATA_SERIALIZER_POSE_ID as u8,
            ];
            photon_core::wire::write_varint(i32::from(pose.id()), &mut metadata);
            metadata.push(255);
            assert!(world.apply_entity_metadata(EntityId(5), &metadata));
        }
        let entity = world.entities().at(EntityId(5)).expect("the entity");
        entity.scaled_dimensions().eye_height()
    };
    use crate::ClientEntityPose::{Crouching, Sleeping};
    let near = |actual: f32, expected: f32| {
        assert!((actual - expected).abs() < 1e-5, "{actual} != {expected}");
    };
    near(eye("minecraft:pig", None, 1.0), 0.9 * 0.85);
    near(eye("minecraft:pig", None, 2.0), 0.9 * 0.85 * 2.0);
    near(eye("minecraft:pig", Some(Sleeping), 2.0), 0.2);
    near(eye("minecraft:player", None, 1.0), 1.62);
    near(eye("minecraft:player", None, 2.0), 3.24);
    near(eye("minecraft:player", Some(Crouching), 2.0), 2.54);
    near(eye("minecraft:player", Some(Sleeping), 2.0), 0.2);
}

#[test]
fn hurt_animation_keeps_the_yaw_for_players_only() {
    let hurt = |name: &str| {
        let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(5),
            uuid: Uuid::new(1, 5),
            type_id: photon_data::entity_type::id(name).expect("the type"),
            x: 0.0,
            y: 9.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        assert!(world.entities_mut().apply_hurt_animation(EntityId(5), 90.0));
        let entity = world.entities().at(EntityId(5)).expect("the entity");
        (entity.hurt_time(), entity.hurt_dir())
    };
    assert_eq!(hurt("minecraft:player"), (10, 90.0));
    assert_eq!(hurt("minecraft:pig"), (10, 0.0));
}

#[test]
fn camera_bob_belongs_to_players() {
    let bob = |name: &str| {
        let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
        assert!(world.apply_entity(EntityInput {
            entity_id: EntityId(5),
            uuid: Uuid::new(1, 5),
            type_id: photon_data::entity_type::id(name).expect("the type"),
            x: 0.0,
            y: 9.0,
            z: 0.0,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        let entity = world
            .entities_mut()
            .at_mut(EntityId(5))
            .expect("the entity");
        entity.living.cloak.update_bob(0.1);
        entity.camera_bob(1.0)
    };
    assert_eq!(
        bob("minecraft:player").to_bits(),
        (0.1_f32 * 0.4_f32).to_bits()
    );
    assert_eq!(bob("minecraft:pig"), 0.0);
}
