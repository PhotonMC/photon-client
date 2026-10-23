//! The events the title, the failed join and the question take.

use super::*;

/// An event for the title screen's buttons: what the press chose.
pub(super) fn title_event(
    screen: &mut TitleScreen,
    event: WindowEvent,
    mouse: (i32, i32),
) -> Option<TitleAction> {
    let buttons = screen.buttons();
    match event {
        WindowEvent::CursorPos(..) => {
            buttons.mouse_input();
            None
        }
        WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => buttons.clicked(mouse),
        WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
            buttons.tab(!modifiers.contains(Modifiers::Shift));
            None
        }
        WindowEvent::Key(key, _, Action::Press | Action::Repeat, _) => {
            match key {
                Key::Left => buttons.arrow(Direction::Left),
                Key::Right => buttons.arrow(Direction::Right),
                Key::Up => buttons.arrow(Direction::Up),
                Key::Down => buttons.arrow(Direction::Down),
                Key::Enter | Key::KpEnter | Key::Space => return buttons.pressed(),
                _ => {}
            }
            None
        }
        _ => None,
    }
}

/// An event for the failed join's screen: whether its button was pressed.
pub(super) fn failed_event(
    screen: &mut DisconnectedScreen,
    event: WindowEvent,
    mouse: (i32, i32),
) -> bool {
    match event {
        WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => screen.clicked(mouse),
        WindowEvent::Key(Key::Tab, _, Action::Press, _) => {
            screen.tab();
            false
        }
        WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
            screen.pressed()
        }
        _ => false,
    }
}

/// An event for the question: its answer, if one was given.
pub(super) fn confirm_event(
    screen: &mut ConfirmScreen,
    event: WindowEvent,
    mouse: (i32, i32),
) -> Option<ConfirmAction> {
    match event {
        WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => screen.clicked(mouse),
        WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
            screen.tab(!modifiers.contains(Modifiers::Shift));
            None
        }
        WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
            screen.pressed()
        }
        WindowEvent::Key(Key::Escape, _, Action::Press, _) => screen.escape(),
        _ => None,
    }
}
