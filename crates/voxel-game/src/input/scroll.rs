//! Headless scroll accumulation and hotbar selection helpers.

fn java_signum(value: f64) -> f64 {
    if value.is_nan() {
        f64::NAN
    } else if value == 0.0 {
        value
    } else if value < 0.0 {
        -1.0
    } else {
        1.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollWheelHandler {
    accumulated_scroll_x: f64,
    accumulated_scroll_y: f64,
}

impl Default for ScrollWheelHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollWheelHandler {
    pub const fn new() -> Self {
        Self {
            accumulated_scroll_x: 0.0,
            accumulated_scroll_y: 0.0,
        }
    }

    pub fn on_mouse_scroll(
        &mut self,
        scaled_x_scroll_offset: f64,
        scaled_y_scroll_offset: f64,
    ) -> (i32, i32) {
        if self.accumulated_scroll_x != 0.0
            && java_signum(scaled_x_scroll_offset) != java_signum(self.accumulated_scroll_x)
        {
            self.accumulated_scroll_x = 0.0;
        }
        if self.accumulated_scroll_y != 0.0
            && java_signum(scaled_y_scroll_offset) != java_signum(self.accumulated_scroll_y)
        {
            self.accumulated_scroll_y = 0.0;
        }

        self.accumulated_scroll_x += scaled_x_scroll_offset;
        self.accumulated_scroll_y += scaled_y_scroll_offset;
        let wheel_x = self.accumulated_scroll_x as i32;
        let wheel_y = self.accumulated_scroll_y as i32;
        if wheel_x == 0 && wheel_y == 0 {
            return (0, 0);
        }

        self.accumulated_scroll_x -= f64::from(wheel_x);
        self.accumulated_scroll_y -= f64::from(wheel_y);
        (wheel_x, wheel_y)
    }

    pub fn next_scroll_wheel_selection(wheel: f64, mut current_selected: i32, limit: i32) -> i32 {
        let step = java_signum(wheel) as i32;
        current_selected = current_selected.wrapping_sub(step);
        current_selected = current_selected.max(-1);

        while current_selected < 0 {
            current_selected = current_selected.wrapping_add(limit);
        }
        while current_selected >= limit {
            current_selected = current_selected.wrapping_sub(limit);
        }

        current_selected
    }
}
