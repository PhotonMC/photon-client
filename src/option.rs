//! Headless option value sets from the client settings layer.

fn map(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    let alpha = (value - from_min) / (from_max - from_min);
    to_min + alpha * (to_max - to_min)
}

/// Inclusive integer range used by slider-backed options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntRange {
    pub min_inclusive: i32,
    pub max_inclusive: i32,
    pub apply_value_immediately: bool,
}

impl IntRange {
    pub const fn new(min_inclusive: i32, max_inclusive: i32) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
            apply_value_immediately: true,
        }
    }

    pub const fn with_apply_value_immediately(
        min_inclusive: i32,
        max_inclusive: i32,
        apply_value_immediately: bool,
    ) -> Self {
        Self {
            min_inclusive,
            max_inclusive,
            apply_value_immediately,
        }
    }

    pub fn validate_value(self, value: i32) -> Option<i32> {
        (self.min_inclusive..=self.max_inclusive)
            .contains(&value)
            .then_some(value)
    }

    pub fn next(self, current: i32) -> i32 {
        current.wrapping_add(1)
    }

    pub fn previous(self, current: i32) -> i32 {
        current.wrapping_sub(1)
    }

    pub fn to_slider_value(self, value: i32) -> f64 {
        if value == self.min_inclusive {
            0.0
        } else if value == self.max_inclusive {
            1.0
        } else {
            map(
                f64::from(value) + 0.5,
                f64::from(self.min_inclusive),
                f64::from(self.max_inclusive) + 1.0,
                0.0,
                1.0,
            )
        }
    }

    pub fn from_slider_value(self, mut slider: f64) -> i32 {
        if slider >= 1.0 {
            slider = f64::from(0.99999_f32);
        }
        map(
            slider,
            0.0,
            1.0,
            f64::from(self.min_inclusive),
            f64::from(self.max_inclusive) + 1.0,
        )
        .floor() as i32
    }
}

/// Unit interval value set used by normalized double options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitDouble;

impl UnitDouble {
    pub fn validate_value(value: f64) -> Option<f64> {
        (0.0..=1.0).contains(&value).then_some(value)
    }

    pub const fn to_slider_value(value: f64) -> f64 {
        value
    }

    pub const fn from_slider_value(slider: f64) -> f64 {
        slider
    }
}
