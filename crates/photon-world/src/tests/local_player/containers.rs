use super::super::*;
use photon_core::ids::EntityId;

#[test]
fn local_player_inventory_state_applies_slot_cursor_and_respawn_reset() {
    let mut world = ClientWorld::default();
    let stone = ClientItemStackState::from_wire(8, photon_core::ids::ItemId(1), vec![], vec![]);
    let bottle =
        ClientItemStackState::from_wire(1, photon_core::ids::ItemId(1_149), vec![], vec![]);
    let mut items = vec![None; 46];
    items[36] = Some(stone.clone());

    assert!(!world.apply_local_player_container_content(0, 1, &items, Some(&stone),));
    world.set_local_player_id(EntityId(42));
    assert!(world.apply_local_player_container_content(0, 2, &items, Some(&stone),));
    let player = world.local_player().expect("local player inventory");
    assert_eq!(player.inventory_state_id(), 2);
    assert_eq!(player.inventory_slot_count(), 46);
    assert_eq!(player.inventory_slot(36), Some(&stone));
    assert_eq!(player.carried_item(), Some(&stone));

    assert!(world.apply_local_player_container_slot(0, 3, 36, Some(&bottle)));
    assert_eq!(
        world
            .local_player()
            .expect("local player after slot update")
            .inventory_slot(36),
        Some(&bottle)
    );
    assert_eq!(
        world
            .local_player()
            .expect("local player after slot update")
            .inventory_state_id(),
        3
    );

    assert!(world.apply_local_player_carried_item(None));
    assert!(
        world
            .local_player()
            .expect("local player after cursor clear")
            .carried_item()
            .is_none()
    );

    assert!(world.apply_local_player_inventory_slot(0, Some(&stone)));
    assert_eq!(
        world
            .local_player()
            .expect("local player after direct inventory update")
            .inventory_slot(36),
        Some(&stone)
    );
    assert!(!world.apply_local_player_inventory_slot(36, Some(&bottle)));

    assert!(world.recreate_local_player(0));
    let player = world
        .local_player()
        .expect("respawned local player inventory");
    assert_eq!(player.inventory_state_id(), 0);
    assert_eq!(player.inventory_slot_count(), 0);
    assert!(player.inventory_slot(36).is_none());
    assert!(player.carried_item().is_none());
}

#[test]
fn local_player_container_state_applies_open_content_slot_cursor_close_and_reset() {
    let mut world = ClientWorld::default();
    let stone = ClientItemStackState::from_wire(7, photon_core::ids::ItemId(1), vec![], vec![]);
    let bottle =
        ClientItemStackState::from_wire(1, photon_core::ids::ItemId(1_149), vec![], vec![]);
    let title = [10, 0, 0, 0];
    let mut items = vec![None; 63];
    items[0] = Some(stone.clone());
    items[27] = Some(bottle.clone());

    assert!(!world.apply_local_player_open_menu(7, 2, &title));
    world.set_local_player_id(EntityId(42));
    assert!(world.apply_local_player_open_menu(7, 2, &title));
    assert!(world.apply_local_player_container_content(7, 4, &items, None));
    let container = world
        .local_player()
        .expect("local player with open container")
        .open_container()
        .expect("open container");
    assert_eq!(container.container_id(), 7);
    assert_eq!(container.menu_type_id(), 2);
    assert_eq!(container.title_nbt(), title);
    assert_eq!(container.top_slot_count(), Some(27));
    assert_eq!(container.slot_count(), 63);
    assert_eq!(container.state_id(), 4);
    assert_eq!(container.slot(0), Some(&stone));
    assert_eq!(container.slot(27), Some(&bottle));
    assert!(container.carried_item().is_none());

    assert!(world.apply_local_player_container_slot(7, 5, 0, None));
    assert!(
        world
            .local_player()
            .expect("local player after custom slot update")
            .open_container()
            .expect("container after slot update")
            .slot(0)
            .is_none()
    );
    assert!(world.apply_local_player_carried_item(Some(&stone)));
    assert_eq!(
        world
            .local_player()
            .expect("local player after custom cursor update")
            .open_container()
            .expect("container after cursor update")
            .carried_item(),
        Some(&stone)
    );
    assert!(world.apply_local_player_container_close(99));
    assert!(
        world
            .local_player()
            .expect("local player after custom container close")
            .open_container()
            .is_none()
    );
    assert!(!world.apply_local_player_container_close(7));

    assert!(world.apply_local_player_open_menu(7, 2, &title));
    assert!(world.recreate_local_player(0));
    let player = world
        .local_player()
        .expect("respawned local player without container");
    assert!(player.open_container().is_none());
}
