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
                attribute.attribute_id() == crate::VANILLA_PLAYER_MOVEMENT_SPEED_ATTRIBUTE_ID
            })
            .expect("default movement speed attribute");
        assert_eq!(
            attribute.base_value(),
            f64::from(crate::VANILLA_PLAYER_DEFAULT_MOVEMENT_SPEED)
        );
        assert_eq!(attribute.modifiers().len(), 1);
        assert_eq!(
            attribute.modifiers()[0].id(),
            crate::VANILLA_PLAYER_SPRINTING_MODIFIER_ID
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
        crate::VANILLA_PLAYER_MOVEMENT_SPEED_ATTRIBUTE_ID,
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
                .filter(|modifier| { modifier.id() == crate::VANILLA_PLAYER_SPRINTING_MODIFIER_ID })
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
            .all(|modifier| modifier.id() != crate::VANILLA_PLAYER_SPRINTING_MODIFIER_ID)
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
        crate::VANILLA_PLAYER_SNEAKING_SPEED_ATTRIBUTE_ID,
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
        crate::VANILLA_PLAYER_SNEAKING_SPEED_ATTRIBUTE_ID,
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
        crate::VANILLA_PLAYER_SNEAKING_SPEED_ATTRIBUTE_ID,
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
        crate::VANILLA_PLAYER_SNEAKING_SPEED_ATTRIBUTE_ID,
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
fn local_player_travel_in_air_applies_gravity_drag_and_client_fallbacks() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
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
    world.set_local_player_id(42);
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
    world.set_local_player_id(42);
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
fn local_player_health_metadata_updates_and_respawn_resets_health() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);

    let mut health_metadata = vec![
        8,
        crate::ENTITY_DATA_SERIALIZER_BYTE_ID as u8,
        0,
        crate::ENTITY_HEALTH_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_FLOAT_ID as u8,
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
fn local_player_arrow_count_metadata_follows_entity_data_keep_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);

    let mut arrow_metadata = vec![
        crate::ENTITY_ARROW_COUNT_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
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
fn local_player_stinger_count_metadata_follows_entity_data_keep_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);

    let mut stinger_metadata = vec![
        crate::ENTITY_STINGER_COUNT_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_INT_ID as u8,
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
fn local_player_effect_particle_types_follow_entity_data_keep_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(42);
    let mut particle_metadata = vec![
        crate::ENTITY_EFFECT_PARTICLES_DATA_ID,
        crate::ENTITY_DATA_SERIALIZER_PARTICLES_ID as u8,
        1,
    ];
    voxel_core::wire::write_varint(
        crate::ENTITY_EFFECT_PARTICLE_TYPE_ID,
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
        &[crate::ENTITY_EFFECT_PARTICLE_TYPE_ID]
    );

    assert!(world.recreate_local_player(PLAYER_RESPAWN_KEEP_ENTITY_DATA));
    assert_eq!(
        world
            .local_player()
            .expect("respawned particle player")
            .effect_particle_type_ids(),
        &[crate::ENTITY_EFFECT_PARTICLE_TYPE_ID]
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
