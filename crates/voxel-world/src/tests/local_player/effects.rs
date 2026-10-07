use super::super::*;

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
