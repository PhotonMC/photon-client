//! Smooth mouse delta accumulation used by the client camera path.

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
pub struct SmoothDouble {
    target_value: f64,
    remaining_value: f64,
    last_amount: f64,
}

impl Default for SmoothDouble {
    fn default() -> Self {
        Self::new()
    }
}

impl SmoothDouble {
    pub const fn new() -> Self {
        Self {
            target_value: 0.0,
            remaining_value: 0.0,
            last_amount: 0.0,
        }
    }

    pub fn get_new_delta_value(&mut self, target_delta: f64, time: f64) -> f64 {
        self.target_value += target_delta;
        let mut delta = self.target_value - self.remaining_value;
        let new_last_amount = self.last_amount + 0.5 * (delta - self.last_amount);
        let delta_sign = java_signum(delta);
        if delta_sign * delta > delta_sign * self.last_amount {
            delta = new_last_amount;
        }

        self.last_amount = new_last_amount;
        self.remaining_value += delta * time;
        delta * time
    }

    pub const fn reset(&mut self) {
        self.target_value = 0.0;
        self.remaining_value = 0.0;
        self.last_amount = 0.0;
    }
}
