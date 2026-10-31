use super::*;

fn player(id: u128, name: &str, game_mode: i32) -> Candidate {
    Candidate {
        id,
        name: name.to_owned(),
        game_mode,
    }
}

fn many(count: u128) -> Sources {
    Sources {
        players: (1..=count)
            .map(|n| player(n, &format!("p{n}"), 0))
            .collect(),
        teams: Vec::new(),
    }
}

#[test]
fn the_first_press_of_a_slot_selects_and_the_second_runs_it() {
    let mut menu = SpectatorMenu::new(&many(2));
    assert_eq!(menu.select_slot(0), None);
    assert_eq!(menu.selected_slot(), 0);
    assert_eq!(menu.select_slot(0), None);
    // The players' page: the category opened, nothing selected.
    assert_eq!(menu.selected_slot(), -1);
    assert_eq!(menu.prompt(), "spectatorMenu.teleport.prompt");
    menu.select_slot(1);
    assert_eq!(menu.select_slot(1), Some(Action::Teleport(2)));
}

#[test]
fn an_empty_slot_is_not_selected() {
    let mut menu = SpectatorMenu::new(&many(1));
    menu.select_slot(5);
    assert_eq!(menu.selected_slot(), -1);
}

#[test]
fn pages_hold_six_entries_between_the_arrows() {
    let mut menu = SpectatorMenu::new(&many(9));
    menu.select_slot(0);
    menu.select_slot(0);
    // Page 0: entries 0 to 6 in slots 0 to 6 (the slot of the arrow is an entry), the right arrow
    // is enabled, the close slot is last.
    assert_eq!(menu.item(6), Item::Player(player(7, "p7", 0)));
    assert_eq!(
        menu.item(7),
        Item::Scroll {
            direction: 1,
            enabled: true
        }
    );
    assert_eq!(menu.item(8), Item::Close);
    menu.select_slot(7);
    menu.select_slot(7);
    // Page 1 starts at entry 6, behind a left arrow.
    assert_eq!(
        menu.item(0),
        Item::Scroll {
            direction: -1,
            enabled: true
        }
    );
    assert_eq!(menu.item(1), Item::Player(player(8, "p8", 0)));
    // Entry 13 does not exist, so the right arrow is disabled there.
    assert_eq!(
        menu.item(7),
        Item::Scroll {
            direction: 1,
            enabled: false
        }
    );
}

#[test]
fn players_are_ordered_by_the_signed_halves_of_their_id() {
    let high = 1_u128 << 127;
    let sources = Sources {
        players: vec![player(high, "high", 0), player(5, "five", 0)],
        teams: Vec::new(),
    };
    let menu = SpectatorMenu::new(&sources);
    let Item::TeleportToPlayer(items) = menu.item(0) else {
        panic!("the first entry lists the players");
    };
    // A set top bit is negative, so it comes first.
    assert_eq!(items[0], Item::Player(player(high, "high", 0)));
}

#[test]
fn spectators_are_not_listed_and_an_empty_category_is_disabled() {
    let sources = Sources {
        players: vec![player(1, "watcher", 3)],
        teams: Vec::new(),
    };
    let menu = SpectatorMenu::new(&sources);
    assert!(!menu.item(0).is_enabled());
    assert!(!menu.item(1).is_enabled());
}

#[test]
fn the_wheel_skips_empty_and_disabled_slots() {
    let mut gui = SpectatorGui::default();
    gui.on_hotbar_selected(0, 100, &many(1));
    // Nothing is selected yet, so the wheel starts from -1: the player category first.
    gui.on_mouse_scrolled(1, 200);
    assert_eq!(gui.menu().map(SpectatorMenu::selected_slot), Some(0));
    // Then the team category (disabled), the empty slots and the disabled arrow are skipped.
    gui.on_mouse_scrolled(1, 300);
    assert_eq!(gui.menu().map(SpectatorMenu::selected_slot), Some(8));
}

#[test]
fn the_menu_fades_after_five_seconds_and_closes() {
    let mut gui = SpectatorGui::default();
    gui.on_hotbar_selected(0, 1000, &many(1));
    assert_eq!(gui.alpha(1000), 1.0);
    // Opaque for the first three seconds, then two seconds of fade.
    assert_eq!(gui.alpha(4000), 1.0);
    assert_eq!(gui.alpha(5000), 0.5);
    gui.close_faded(5500);
    assert!(gui.is_menu_active());
    gui.close_faded(6000);
    assert!(!gui.is_menu_active());
}
