use super::super::*;

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
