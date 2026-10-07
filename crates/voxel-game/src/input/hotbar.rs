//! Choosing the hotbar slot: the number keys pick one outright, and the
//! mouse wheel steps through them the way the reference's wheel handler
//! does, adding up partial steps (from a touchpad) until a whole one is
//! reached and starting over when the direction changes.

use crate::input::scroll::ScrollWheelHandler;

/// Slots in the hotbar.
pub const SLOTS: i32 = 9;

/// `ScrollWheelHandler`: partial wheel steps not yet spent.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScrollWheel(ScrollWheelHandler);

impl ScrollWheel {
    /// `MouseHandler.onScroll` for a player on foot, with the default
    /// options (continuous scrolling at a sensitivity of one): the slot
    /// after `selected` the wheel moved to, if it moved a whole step.
    pub fn select(&mut self, x: f64, y: f64, selected: i32) -> Option<i32> {
        let (wheel_x, wheel_y) = self.0.on_mouse_scroll(x, y);
        if wheel_x == 0 && wheel_y == 0 {
            return None;
        }
        let wheel = if wheel_y == 0 { -wheel_x } else { wheel_y };
        Some(ScrollWheelHandler::next_scroll_wheel_selection(
            f64::from(wheel),
            selected,
            SLOTS,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wheel_steps_and_wraps() {
        let mut wheel = ScrollWheel::default();
        assert_eq!(wheel.select(0.0, 1.0, 0), Some(8));
        assert_eq!(wheel.select(0.0, -1.0, 8), Some(0));
        assert_eq!(wheel.select(0.0, -1.0, 3), Some(4));
    }

    #[test]
    fn partial_steps_add_up_and_reset_on_turning() {
        let mut wheel = ScrollWheel::default();
        assert_eq!(wheel.select(0.0, 0.4, 4), None);
        assert_eq!(wheel.select(0.0, 0.4, 4), None);
        assert_eq!(wheel.select(0.0, 0.4, 4), Some(3));
        // What is left over (0.2) is dropped when the wheel turns back.
        assert_eq!(wheel.select(0.0, -0.9, 3), None);
        assert_eq!(wheel.select(0.0, -0.2, 3), Some(4));
    }

    #[test]
    fn sideways_scrolling_steps_the_other_way() {
        let mut wheel = ScrollWheel::default();
        assert_eq!(wheel.select(1.0, 0.0, 4), Some(5));
    }
}
