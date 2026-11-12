//! `PHOTON_MENU_SCRIPT` (dev only): mouse moves and clicks made on the menus, so a run can pick
//! a game version and join a server the way a player does, with no person at the mouse.
//!
//! The script is one segment per visit to the menus, separated by `|`; a segment is steps
//! separated by `;`: `wait/N` (N frames), `move/X/Y` (the mouse to a place, in GUI units) and
//! `click` (the primary button pressed and let go). The events go through the same handler as
//! the window's own.

use glfw::{Action, Modifiers, MouseButton, WindowEvent};
use std::collections::VecDeque;
use std::sync::Mutex;

/// One step of a segment.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Step {
    Wait(u32),
    Move(f64, f64),
    Click,
}

#[derive(Default)]
struct State {
    segments: Vec<VecDeque<Step>>,
    /// The visit being played, counted from one; nothing runs before the first.
    visit: usize,
    waiting: u32,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

/// Whether a menu script is set.
pub(super) fn is_set() -> bool {
    std::env::var_os("PHOTON_MENU_SCRIPT").is_some()
}

fn parse(text: &str) -> Result<Vec<VecDeque<Step>>, String> {
    let number = |part: &str| {
        part.parse::<f64>()
            .map_err(|_| format!("bad number {part}"))
    };
    text.split('|')
        .map(|segment| {
            segment
                .split(';')
                .map(str::trim)
                .filter(|step| !step.is_empty())
                .map(|step| {
                    let parts: Vec<&str> = step.split('/').collect();
                    match parts.as_slice() {
                        ["wait", frames] => Ok(Step::Wait(number(frames)? as u32)),
                        ["move", x, y] => Ok(Step::Move(number(x)?, number(y)?)),
                        ["click"] => Ok(Step::Click),
                        _ => Err(format!("unknown step {step} in PHOTON_MENU_SCRIPT")),
                    }
                })
                .collect()
        })
        .collect()
}

/// A visit to the menus begins: the next segment is played.
pub(super) fn begin() {
    let mut guard = STATE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if guard.is_none() {
        let Ok(text) = std::env::var("PHOTON_MENU_SCRIPT") else {
            return;
        };
        match parse(&text) {
            Ok(segments) => {
                *guard = Some(State {
                    segments,
                    ..State::default()
                });
            }
            Err(error) => {
                eprintln!("{error}");
                return;
            }
        }
    }
    if let Some(state) = guard.as_mut() {
        state.visit += 1;
        state.waiting = 0;
    }
}

/// The window events of this frame for `surface`.
pub(super) fn frame(surface: &photon_render::window::WindowSurface) -> Vec<WindowEvent> {
    let scale = surface.scale();
    events(
        surface.window_size(),
        (scale.gui_scaled_width(), scale.gui_scaled_height()),
    )
}

/// The events of this frame, for a window `window` pixels big whose GUI is `gui` units.
fn events(window: (i32, i32), gui: (i32, i32)) -> Vec<WindowEvent> {
    let mut guard = STATE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(state) = guard.as_mut() else {
        return Vec::new();
    };
    if state.waiting > 0 {
        state.waiting -= 1;
        return Vec::new();
    }
    let Some(segment) = state.segments.get_mut(state.visit.wrapping_sub(1)) else {
        return Vec::new();
    };
    match segment.pop_front() {
        Some(Step::Wait(frames)) => {
            state.waiting = frames.saturating_sub(1);
            Vec::new()
        }
        Some(Step::Move(x, y)) => vec![WindowEvent::CursorPos(
            (x + 0.5) * f64::from(window.0) / f64::from(gui.0.max(1)),
            (y + 0.5) * f64::from(window.1) / f64::from(gui.1.max(1)),
        )],
        Some(Step::Click) => ["press", "release"]
            .iter()
            .map(|kind| {
                let action = if *kind == "press" {
                    Action::Press
                } else {
                    Action::Release
                };
                WindowEvent::MouseButton(MouseButton::Button1, action, Modifiers::empty())
            })
            .collect(),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_script_has_one_segment_per_visit() {
        let segments = parse("wait/3;move/10/20;click|click").unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(
            segments[0],
            VecDeque::from([Step::Wait(3), Step::Move(10.0, 20.0), Step::Click])
        );
        assert_eq!(segments[1], VecDeque::from([Step::Click]));
        assert!(parse("jump").is_err());
    }
}
