use super::super::*;
use photon_core::ids::EntityId;

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
fn local_player_travel_flying_rotates_input_and_applies_environment_drag() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(EntityId(42));
    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(
        world.apply_local_player_travel_flying(crate::LocalPlayerTravelFlyingParameters {
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
        world.apply_local_player_travel_flying(crate::LocalPlayerTravelFlyingParameters {
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
fn local_player_travel_in_air_applies_gravity_drag_and_client_fallbacks() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(EntityId(42));
    let base = crate::LocalPlayerTravelInAirParameters {
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
        world.apply_local_player_travel_in_air(crate::LocalPlayerTravelInAirParameters {
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
        world.apply_local_player_travel_in_air(crate::LocalPlayerTravelInAirParameters {
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
        world.apply_local_player_travel_in_air(crate::LocalPlayerTravelInAirParameters {
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
        world.apply_local_player_travel_in_air(crate::LocalPlayerTravelInAirParameters {
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
    world.set_local_player_id(EntityId(42));
    let base = crate::LocalPlayerUpdateFallFlyingMovementParameters {
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
        crate::LocalPlayerUpdateFallFlyingMovementParameters {
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
        crate::LocalPlayerUpdateFallFlyingMovementParameters {
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
        crate::LocalPlayerUpdateFallFlyingMovementParameters {
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
    world.set_local_player_id(EntityId(42));
    let base = crate::LocalPlayerTravelDispatchParameters {
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
        .apply_local_player_travel_dispatch(crate::LocalPlayerTravelDispatchParameters {
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
        .apply_local_player_travel_dispatch(crate::LocalPlayerTravelDispatchParameters {
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
