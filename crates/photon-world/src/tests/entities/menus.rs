use super::super::*;
use photon_core::ids::EntityId;

#[test]
fn an_open_containers_last_slots_are_the_players_inventory() {
    let mut world = ClientWorld::default();
    world.set_local_player_id(EntityId(42));
    let stone = ClientItemStackState::from_wire(7, photon_core::ids::ItemId(1), vec![], vec![]);
    let dirt = ClientItemStackState::from_wire(3, photon_core::ids::ItemId(10), vec![], vec![]);
    let mut items = vec![None; 63];
    // A main inventory slot (menu 9) and the first hotbar slot (menu 36),
    // which a three row chest lists at 27 and 54.
    items[27] = Some(stone.clone());
    items[54] = Some(dirt.clone());
    assert!(world.apply_local_player_open_menu(7, 2, &[10, 0, 0, 0]));
    assert!(world.apply_local_player_container_content(7, 1, &items, None));
    let player = world.local_player().expect("player");
    assert_eq!(player.inventory_slot(9), Some(&stone));
    assert_eq!(player.inventory_slot(36), Some(&dirt));

    // A click in the container's menu writes through both ways.
    let crate::OpenContainerMenu {
        mut slots, carried, ..
    } = world.local_player_container_menu().expect("menu");
    slots[54] = None;
    world.set_local_player_container_menu(&slots, carried);
    assert!(
        world
            .local_player()
            .expect("player")
            .inventory_slot(36)
            .is_none()
    );

    // And the server changing the inventory shows in the container's menu.
    assert!(world.apply_local_player_inventory_slot(1, Some(&dirt)));
    let crate::OpenContainerMenu { slots, .. } = world.local_player_container_menu().expect("menu");
    assert_eq!(slots[55].as_ref(), Some(&dirt));
}

#[test]
fn vanilla_menu_registry_slot_counts_cover_all_builtin_menu_types() {
    let expected = [
        (0, 9),
        (1, 18),
        (2, 27),
        (3, 36),
        (4, 45),
        (5, 54),
        (6, 9),
        (7, 10),
        (8, 3),
        (9, 1),
        (10, 3),
        (11, 5),
        (12, 10),
        (13, 2),
        (14, 3),
        (15, 3),
        (16, 5),
        (17, 1),
        (18, 4),
        (19, 3),
        (20, 27),
        (21, 4),
        (22, 3),
        (23, 3),
        (24, 2),
    ];

    for (menu_type_id, slot_count) in expected {
        assert_eq!(menu_top_slot_count(menu_type_id), Some(slot_count));
    }
    assert_eq!(menu_top_slot_count(-1), None);
    assert_eq!(menu_top_slot_count(25), None);
}
