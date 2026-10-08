use super::super::*;

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
        .shared_flags |= 1 << crate::SHARED_FLAG_SPRINTING;
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
fn local_player_travel_in_water_applies_drag_and_fluid_jump_state() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(
        world.apply_local_player_travel_in_water(crate::LocalPlayerTravelInWaterParameters {
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
        world.apply_local_player_travel_in_water(crate::LocalPlayerTravelInWaterParameters {
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
            .expect("water attribute player")
            .movement_speed(),
        0.63_f32,
    );
    assert!(world.apply_local_player_position((0.0, 10.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(world.apply_local_player_travel_in_water_from_attributes(
        crate::LocalPlayerTravelInWaterParameters {
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
fn local_player_travel_in_lava_applies_shallow_drag_and_jump_state() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    assert!(world.apply_local_player_position((0.0, 0.0, 0.0), (0.1, -0.2, 0.3), (37.5, 0.0),));
    assert!(
        world.apply_local_player_travel_in_lava(crate::LocalPlayerTravelInLavaParameters {
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
        world.apply_local_player_travel_in_lava(crate::LocalPlayerTravelInLavaParameters {
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
