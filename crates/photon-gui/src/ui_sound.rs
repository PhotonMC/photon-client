//! The sounds the interface asks of the sound manager (`SimpleSoundInstance.forUI`): a widget
//! that is pressed queues its click here, and the host plays what was queued once the event is
//! handled. The queue belongs to the thread that handles the window's events.

use std::cell::RefCell;

/// `SoundEvents.UI_BUTTON_CLICK`.
pub const BUTTON_CLICK: &str = "minecraft:ui.button.click";

/// `SimpleSoundInstance.forUI(sound, pitch)`'s default volume.
pub const UI_VOLUME: f32 = 0.25;

/// One interface sound asked for.
#[derive(Clone, Debug, PartialEq)]
pub struct UiSound {
    pub event: &'static str,
    pub pitch: f32,
    pub volume: f32,
}

thread_local! {
    static QUEUE: RefCell<Vec<UiSound>> = const { RefCell::new(Vec::new()) };
}

/// `AbstractWidget.playDownSound`: the click of a button that was pressed.
pub fn play_down_sound() {
    ask(UiSound {
        event: BUTTON_CLICK,
        pitch: 1.0,
        volume: UI_VOLUME,
    });
}

/// `SimpleSoundInstance.forUI(sound, 1.0F)`: a sound of the interface at the default volume.
pub fn play_ui(event: &'static str) {
    ask(UiSound {
        event,
        pitch: 1.0,
        volume: UI_VOLUME,
    });
}

/// Queue a sound of the interface.
pub fn ask(sound: UiSound) {
    QUEUE.with(|queue| queue.borrow_mut().push(sound));
}

/// What was asked since the last call, in order.
pub fn take() -> Vec<UiSound> {
    QUEUE.with(|queue| std::mem::take(&mut *queue.borrow_mut()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_click_is_queued_once_and_taken_once() {
        take();
        play_down_sound();
        let sounds = take();
        assert_eq!(sounds.len(), 1);
        assert_eq!(sounds[0].event, BUTTON_CLICK);
        assert_eq!(sounds[0].volume, 0.25);
        assert!(take().is_empty());
    }
}
