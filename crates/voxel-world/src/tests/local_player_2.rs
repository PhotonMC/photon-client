use super::*;

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
