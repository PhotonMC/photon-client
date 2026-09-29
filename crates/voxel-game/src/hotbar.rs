//! Choosing the hotbar slot: the number keys pick one outright, and the
//! mouse wheel steps through them the way the reference's wheel handler
//! does, adding up partial steps (from a touchpad) until a whole one is
//! reached and starting over when the direction changes.

/// Slots in the hotbar.
pub const SLOTS: i32 = 9;

/// `ScrollWheelHandler`: partial wheel steps not yet spent.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScrollWheel {
    accumulated_x: f64,
    accumulated_y: f64,
}

impl ScrollWheel {
    /// `onMouseScroll` with the default options (continuous scrolling at a
    /// sensitivity of one): whole wheel steps, sideways and up.
    pub fn scroll(&mut self, x: f64, y: f64) -> (i32, i32) {
        if self.accumulated_x != 0.0 && java_signum(x) != java_signum(self.accumulated_x) {
            self.accumulated_x = 0.0;
        }
        if self.accumulated_y != 0.0 && java_signum(y) != java_signum(self.accumulated_y) {
            self.accumulated_y = 0.0;
        }
        self.accumulated_x += x;
        self.accumulated_y += y;
        let wheel_x = self.accumulated_x as i32;
        let wheel_y = self.accumulated_y as i32;
        if wheel_x == 0 && wheel_y == 0 {
            return (0, 0);
        }
        self.accumulated_x -= f64::from(wheel_x);
        self.accumulated_y -= f64::from(wheel_y);
        (wheel_x, wheel_y)
    }

    /// `MouseHandler.onScroll` for a player on foot: the slot after
    /// `selected` the wheel moved to, if it moved a whole step.
    pub fn select(&mut self, x: f64, y: f64, selected: i32) -> Option<i32> {
        let (wheel_x, wheel_y) = self.scroll(x, y);
        if wheel_x == 0 && wheel_y == 0 {
            return None;
        }
        let wheel = if wheel_y == 0 { -wheel_x } else { wheel_y };
        Some(next_selection(f64::from(wheel), selected, SLOTS))
    }
}

/// `ScrollWheelHandler.getNextScrollWheelSelection`: up goes left.
pub fn next_selection(wheel: f64, selected: i32, limit: i32) -> i32 {
    let step = java_signum(wheel) as i32;
    let mut selected = (selected - step).max(-1);
    while selected < 0 {
        selected += limit;
    }
    while selected >= limit {
        selected -= limit;
    }
    selected
}

fn java_signum(value: f64) -> f64 {
    if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else {
        value
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
