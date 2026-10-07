use super::super::*;

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
        player.shared_flags |= 1 << crate::SHARED_FLAG_SPRINTING;
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
        crate::VANILLA_PLAYER_MOVEMENT_SPEED_ATTRIBUTE_ID,
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
fn local_player_handle_relative_friction_applies_climbable_step_state() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(world.apply_local_player_handle_relative_friction(
        crate::LocalPlayerHandleRelativeFrictionParameters {
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
        crate::LocalPlayerHandleRelativeFrictionParameters {
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
