use super::DebugKeys;
use glfw::{Action, Key};

fn tap(debug: &mut DebugKeys, key: Key) {
    debug.key(Action::Press);
    debug.combination(key, false);
    debug.key(Action::Release);
}

#[test]
fn a_lone_f3_toggles_the_overlay_on_release() {
    let mut debug = DebugKeys::default();
    debug.key(Action::Press);
    assert!(!debug.entries.is_overlay_visible());
    debug.key(Action::Release);
    assert!(debug.entries.is_overlay_visible());
}

#[test]
fn a_combination_keeps_the_overlay_as_it_was() {
    let mut debug = DebugKeys::default();
    tap(&mut debug, Key::H);
    assert!(!debug.entries.is_overlay_visible());
    // The next lone press toggles again: the combination used F3 once.
    debug.key(Action::Press);
    debug.key(Action::Release);
    assert!(debug.entries.is_overlay_visible());
}

#[test]
fn a_key_with_no_debug_answer_does_not_use_f3() {
    let mut debug = DebugKeys::default();
    tap(&mut debug, Key::W);
    assert!(debug.entries.is_overlay_visible());
}
