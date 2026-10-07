use super::*;

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
fn local_player_travel_flying_rotates_input_and_applies_environment_drag() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
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
