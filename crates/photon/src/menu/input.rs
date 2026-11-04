//! The events the title, the failed join and the question take.

use super::*;
use photon_gui::screen::notice::Answer;

/// An event for the home screen: what the press chose.
pub(super) fn title_event(
    screen: &mut HomeScreen,
    event: WindowEvent,
    mouse: (i32, i32),
) -> Option<HomeAction> {
    match event {
        WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => screen.clicked(mouse),
        WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, modifiers) => {
            screen.step_focus(!modifiers.contains(Modifiers::Shift));
            None
        }
        WindowEvent::Key(key, _, Action::Press | Action::Repeat, _) => match key {
            Key::Left | Key::Up => {
                screen.step_focus(false);
                None
            }
            Key::Right | Key::Down => {
                screen.step_focus(true);
                None
            }
            Key::Enter | Key::KpEnter | Key::Space => screen.pressed(),
            Key::Escape => {
                screen.escape();
                None
            }
            _ => None,
        },
        _ => None,
    }
}

/// An event for the failed join's screen: whether its button was pressed.
pub(super) fn failed_event(
    screen: &mut NoticeScreen,
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
        WindowEvent::Key(Key::Escape, _, Action::Press, _) => screen.escape(),
        _ => false,
    }
}

/// An event for a website sign-in's question: the answer, if one was given. Escape and a stray
/// Enter say no.
pub(super) fn ask_event(
    screen: &mut NoticeScreen,
    event: WindowEvent,
    mouse: (i32, i32),
) -> Option<Answer> {
    match event {
        WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
            screen.answer_clicked(mouse)
        }
        WindowEvent::Key(Key::Tab, _, Action::Press | Action::Repeat, _) => {
            screen.cycle();
            None
        }
        WindowEvent::Key(Key::Enter | Key::KpEnter | Key::Space, _, Action::Press, _) => {
            screen.answer_pressed()
        }
        WindowEvent::Key(Key::Escape, _, Action::Press, _) => Some(Answer::Back),
        _ => None,
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
