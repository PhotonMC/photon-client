use super::super::*;
use photon_core::ids::EntityId;

#[test]
fn local_player_apply_input_updates_queryable_state_and_base_fallback() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(EntityId(42));
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
fn local_player_input_reads_sneaking_speed_attribute() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(EntityId(42));
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
    assert!(world.apply_local_player_attributes(EntityId(42), &[custom_attribute]));
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
    assert!(world.apply_local_player_attributes(EntityId(42), &[replacement_attribute]));
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
    assert!(!world.apply_local_player_attributes(EntityId(99), &[wrong_attribute]));
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
    world.set_local_player_id(EntityId(42));
    let modifiers = LocalPlayerInputModifiers {
        using_item: false,
        passenger: false,
        item_use_speed_multiplier: 0.2,
        moving_slowly: false,
        sneaking_movement_factor: 0.0,
    };

    assert!(world.apply_local_player_keyboard_input(
        Input::new((true, false, false, false), false, false, false),
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
    assert!(world.apply_local_player_attributes(EntityId(42), &[custom_attribute]));
    let sneaking_modifiers = LocalPlayerInputModifiers {
        moving_slowly: true,
        ..modifiers
    };
    assert!(world.apply_local_player_keyboard_input(
        Input::new((true, false, true, false), true, false, false),
        true,
        sneaking_modifiers,
    ));
    let player = world.local_player().expect("diagonal keyboard player");
    assert_eq!(player.input_impulses().x.to_bits(), 0x3f16_872c);
    assert_eq!(player.input_impulses().y.to_bits(), 0x3f16_872c);
    assert!(player.is_jumping());

    assert!(world.apply_local_player_keyboard_input(
        Input::new((true, true, true, true), true, true, true),
        true,
        sneaking_modifiers,
    ));
    let player = world.local_player().expect("opposing keyboard player");
    assert_eq!(player.input_impulses().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(player.input_impulses().y.to_bits(), 0.0_f32.to_bits());
    assert!(player.is_jumping());
}
