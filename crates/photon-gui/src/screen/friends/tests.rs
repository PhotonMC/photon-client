use super::*;

fn view(ready: bool) -> FriendsView {
    FriendsView {
        friends: Listing::Empty,
        pending: Listing::Empty,
        requests: 2,
        profile_name: "Steve".to_owned(),
        ready,
        sending: false,
    }
}

fn overlay(ready: bool) -> FriendsOverlay {
    FriendsOverlay::new(&Lang::new([]), (320, 240), view(ready))
}

fn centre((x, y, width, height): (i32, i32, i32, i32)) -> (f64, f64) {
    (f64::from(x + width / 2), f64::from(y + height / 2))
}

#[test]
fn the_panel_is_centred_and_eighty_shorter_than_the_window() {
    assert_eq!(overlay(true).content(), (50, 40, 220, 160));
}

#[test]
fn a_click_on_the_pending_tab_selects_it() {
    let mut screen = overlay(true);
    let at = centre(screen.tab_rect(Tab::Pending));
    assert_eq!(screen.clicked(at, &Font::default()), None);
    assert_eq!(screen.tab(), Tab::Pending);
}

#[test]
fn a_click_outside_the_panel_closes_it() {
    let mut screen = overlay(true);
    assert_eq!(
        screen.clicked((2.0, 2.0), &Font::default()),
        Some(FriendsEvent::Close)
    );
}

#[test]
fn a_name_is_sent_trimmed_and_only_when_the_list_is_ready() {
    let font = Font::default();
    for (ready, sent) in [(false, None), (true, Some("Alex".to_owned()))] {
        let mut screen = overlay(ready);
        let rect = screen.box_rect();
        screen.clicked(centre(rect), &font);
        for character in "  Alex ".chars() {
            screen.char_typed(character, &font);
        }
        let asked = screen.clicked(centre(screen.button_rect()), &font);
        assert_eq!(asked, sent.map(FriendsEvent::Send));
    }
}

#[test]
fn the_key_that_opens_the_overlay_closes_it_unless_the_box_is_typed_in() {
    let font = Font::default();
    let mut clipboard = Memory(String::new());
    let mut screen = overlay(true);
    let asked = screen.key_pressed(glfw::Key::F, (false, false), true, &font, &mut clipboard);
    assert_eq!(asked, Some(FriendsEvent::Close));
    screen.clicked(centre(screen.box_rect()), &font);
    let asked = screen.key_pressed(glfw::Key::F, (false, false), true, &font, &mut clipboard);
    assert_eq!(asked, None);
    let asked = screen.key_pressed(
        glfw::Key::Escape,
        (false, false),
        false,
        &font,
        &mut clipboard,
    );
    assert_eq!(asked, Some(FriendsEvent::Close));
}

#[test]
fn a_click_on_the_profile_name_copies_it() {
    let Some(root) = std::env::var_os("PHOTON_ASSETS") else {
        return;
    };
    let Some(font) = Font::load(std::path::Path::new(&root)) else {
        return;
    };
    let mut screen = overlay(true);
    let (x, y, ..) = screen.name_rect(&font);
    let asked = screen.clicked((f64::from(x + 1), f64::from(y + 1)), &font);
    assert_eq!(asked, Some(FriendsEvent::CopyName));
}

fn entry(kind: EntryKind, n: u64) -> Entry {
    Entry {
        id: Uuid::new(0, n),
        name: format!("P{n}"),
        kind,
        status: Vec::new(),
    }
}

#[test]
fn the_buttons_of_the_entries_ask_for_their_actions() {
    let font = Font::default();
    let mut screen = overlay(true);
    let mut shown = view(true);
    shown.friends = Listing::Entries(vec![entry(EntryKind::Friend, 1)]);
    shown.pending = Listing::Entries(vec![
        entry(EntryKind::Incoming, 2),
        entry(EntryKind::Outgoing, 3),
    ]);
    screen.set_view(&Lang::new([]), shown);
    let (left, top, ..) = screen.list_area();
    // The first friend: its remove button is at the right of the 204-wide row.
    let at = (f64::from(left + 8 + 204 - 10), f64::from(top + 14));
    assert_eq!(
        screen.clicked(at, &font),
        Some(FriendsEvent::Remove(Uuid::new(0, 1)))
    );
    // The pending tab: a header (9 + 4), then the request's decline button.
    screen.clicked(centre(screen.tab_rect(Tab::Pending)), &font);
    let at = (f64::from(left + 8 + 204 - 10), f64::from(top + 13 + 14));
    assert_eq!(
        screen.clicked(at, &font),
        Some(FriendsEvent::Decline(Uuid::new(0, 2)))
    );
}

struct Memory(String);

impl crate::widgets::edit_box::Clipboard for Memory {
    fn get(&mut self) -> String {
        self.0.clone()
    }

    fn set(&mut self, text: &str) {
        self.0 = text.to_owned();
    }
}
