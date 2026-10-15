use super::super::*;
use photon_core::ids::EntityId;

#[test]
fn local_player_abilities_apply_packet_values_and_reset_on_respawn() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(EntityId(42));
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
    world.set_local_player_id(EntityId(42));
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
