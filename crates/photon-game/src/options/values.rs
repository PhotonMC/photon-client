//! Headless option value sets from the client settings layer.

use std::marker::PhantomData;

fn map(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    let alpha = (value - from_min) / (from_max - from_min);
    to_min + alpha * (to_max - to_min)
}

/// Ordered default and alternate values selected by a runtime condition.
pub struct AltEnum<T, Condition>
where
    Condition: Fn() -> bool,
{
    values: Vec<T>,
    alt_values: Vec<T>,
    alt_condition: Condition,
}

impl<T, Condition> AltEnum<T, Condition>
where
    T: Clone + PartialEq,
    Condition: Fn() -> bool,
{
    pub fn new(values: Vec<T>, alt_values: Vec<T>, alt_condition: Condition) -> Self {
        Self {
            values,
            alt_values,
            alt_condition,
        }
    }

    pub fn values(&self) -> Vec<T> {
        self.values.clone()
    }

    pub fn alt_values(&self) -> Vec<T> {
        self.alt_values.clone()
    }

    pub fn selected_values(&self) -> Vec<T> {
        if (self.alt_condition)() {
            self.alt_values()
        } else {
            self.values()
        }
    }

    pub fn validate_value(&self, value: &T) -> Option<T> {
        let values = if (self.alt_condition)() {
            &self.alt_values
        } else {
            &self.values
        };
        values.contains(value).then(|| value.clone())
    }
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

    pub fn xmap<R, To, FromValue>(
        self,
        to: To,
        from: FromValue,
        discrete: bool,
    ) -> MappedIntRange<R, To, FromValue>
    where
        To: Fn(i32) -> R,
        FromValue: Fn(&R) -> i32,
    {
        MappedIntRange {
            range: self,
            to,
            from,
            discrete,
        }
    }
}

/// An integer range mapped to another option value type.
pub struct MappedIntRange<R, To, FromValue>
where
    To: Fn(i32) -> R,
    FromValue: Fn(&R) -> i32,
{
    range: IntRange,
    to: To,
    from: FromValue,
    discrete: bool,
}

impl<R, To, FromValue> MappedIntRange<R, To, FromValue>
where
    To: Fn(i32) -> R,
    FromValue: Fn(&R) -> i32,
{
    pub fn validate_value(&self, value: &R) -> Option<R> {
        self.range
            .validate_value((self.from)(value))
            .map(|value| (self.to)(value))
    }

    pub fn to_slider_value(&self, value: &R) -> f64 {
        self.range.to_slider_value((self.from)(value))
    }

    pub fn next(&self, current: &R) -> Option<R> {
        if !self.discrete {
            return None;
        }
        let current_index = (self.from)(current);
        let next_index = self
            .range
            .validate_value(current_index.wrapping_add(1))
            .unwrap_or(current_index);
        Some((self.to)(next_index))
    }

    pub fn previous(&self, current: &R) -> Option<R> {
        if !self.discrete {
            return None;
        }
        let current_index = (self.from)(current);
        let previous_index = self
            .range
            .validate_value(current_index.wrapping_sub(1))
            .unwrap_or(current_index);
        Some((self.to)(previous_index))
    }

    pub fn from_slider_value(&self, slider: f64) -> R {
        (self.to)(self.range.from_slider_value(slider))
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

    pub fn xmap<R, To, FromValue>(to: To, from: FromValue) -> MappedUnitDouble<R, To, FromValue>
    where
        To: Fn(f64) -> R,
        FromValue: Fn(&R) -> f64,
    {
        MappedUnitDouble { to, from }
    }
}

/// A unit double value set mapped to another option value type.
pub struct MappedUnitDouble<R, To, FromValue>
where
    To: Fn(f64) -> R,
    FromValue: Fn(&R) -> f64,
{
    to: To,
    from: FromValue,
}

impl<R, To, FromValue> MappedUnitDouble<R, To, FromValue>
where
    To: Fn(f64) -> R,
    FromValue: Fn(&R) -> f64,
{
    pub fn validate_value(&self, value: &R) -> Option<R> {
        UnitDouble::validate_value((self.from)(value)).map(|value| (self.to)(value))
    }

    pub fn to_slider_value(&self, value: &R) -> f64 {
        UnitDouble::to_slider_value((self.from)(value))
    }

    pub fn from_slider_value(&self, slider: f64) -> R {
        (self.to)(UnitDouble::from_slider_value(slider))
    }
}

/// Ordered non-empty enum values used by a discrete slider option.
#[derive(Debug, Clone, PartialEq)]
pub struct SliderableEnum<T> {
    values: Vec<T>,
}

impl<T> SliderableEnum<T>
where
    T: Clone + PartialEq,
{
    /// Create a slider value set. Java's slider methods require at least one value.
    pub fn new(values: Vec<T>) -> Self {
        assert!(
            !values.is_empty(),
            "SliderableEnum requires a non-empty value list"
        );
        Self { values }
    }

    pub fn values(&self) -> &[T] {
        &self.values
    }

    pub fn to_slider_value(&self, value: &T) -> f64 {
        if value == &self.values[0] {
            return 0.0;
        }
        if value == self.values.last().expect("non-empty SliderableEnum") {
            return 1.0;
        }

        let index = self
            .values
            .iter()
            .position(|candidate| candidate == value)
            .map_or(-1_i32, |index| index as i32);
        map(index as f64, 0.0, self.values.len() as f64 - 1.0, 0.0, 1.0)
    }

    pub fn next(&self, current: &T) -> T {
        let current_index = self
            .values
            .iter()
            .position(|candidate| candidate == current)
            .map_or(-1_i32, |index| index as i32);
        let next_index = current_index
            .wrapping_add(1)
            .clamp(0, self.values.len() as i32 - 1);
        self.values[next_index as usize].clone()
    }

    pub fn previous(&self, current: &T) -> T {
        let current_index = self
            .values
            .iter()
            .position(|candidate| candidate == current)
            .map_or(-1_i32, |index| index as i32);
        let previous_index = current_index
            .wrapping_sub(1)
            .clamp(0, self.values.len() as i32 - 1);
        self.values[previous_index as usize].clone()
    }

    pub fn from_slider_value(&self, mut slider: f64) -> T {
        if slider >= 1.0 {
            slider = f64::from(0.99999_f32);
        }
        let index = map(slider, 0.0, 1.0, 0.0, self.values.len() as f64).floor() as i32;
        let index = index.clamp(0, self.values.len() as i32 - 1);
        self.values[index as usize].clone()
    }

    pub fn validate_value(&self, value: &T) -> Option<T> {
        self.values
            .iter()
            .any(|candidate| candidate == value)
            .then(|| value.clone())
    }
}

/// Ordered values exposed by a cycle button option.
#[derive(Debug, Clone, PartialEq)]
pub struct CycleableEnum<T> {
    values: Vec<T>,
}

impl<T> CycleableEnum<T>
where
    T: Clone + PartialEq,
{
    pub fn new(values: Vec<T>) -> Self {
        Self { values }
    }

    pub fn values(&self) -> &[T] {
        &self.values
    }

    pub fn validate_value(&self, value: &T) -> Option<T> {
        self.values
            .iter()
            .any(|candidate| candidate == value)
            .then(|| value.clone())
    }
}

/// Runtime-supplied values and validation for a cycle button option.
pub struct LazyEnum<T, Values, Validator>
where
    Values: Fn() -> Vec<T>,
    Validator: Fn(&T) -> Option<T>,
{
    values: Values,
    validate_value: Validator,
    marker: PhantomData<fn() -> T>,
}

impl<T, Values, Validator> LazyEnum<T, Values, Validator>
where
    Values: Fn() -> Vec<T>,
    Validator: Fn(&T) -> Option<T>,
{
    pub fn new(values: Values, validate_value: Validator) -> Self {
        Self {
            values,
            validate_value,
            marker: PhantomData,
        }
    }

    pub fn values(&self) -> Vec<T> {
        (self.values)()
    }

    pub fn validate_value(&self, value: &T) -> Option<T> {
        (self.validate_value)(value)
    }
}

/// Integer range whose upper bound is supplied when it is read.
pub struct ClampingLazyMaxIntRange<Max>
where
    Max: Fn() -> i32,
{
    min_inclusive: i32,
    max_supplier: Max,
    encodable_max_inclusive: i32,
}

impl<Max> ClampingLazyMaxIntRange<Max>
where
    Max: Fn() -> i32,
{
    pub fn new(min_inclusive: i32, max_supplier: Max, encodable_max_inclusive: i32) -> Self {
        Self {
            min_inclusive,
            max_supplier,
            encodable_max_inclusive,
        }
    }

    pub const fn min_inclusive(&self) -> i32 {
        self.min_inclusive
    }

    pub fn max_inclusive(&self) -> i32 {
        (self.max_supplier)()
    }

    pub const fn encodable_max_inclusive(&self) -> i32 {
        self.encodable_max_inclusive
    }

    pub fn validate_value(&self, value: i32) -> i32 {
        value.max(self.min_inclusive).min(self.max_inclusive())
    }

    pub const fn next(&self, current: i32) -> i32 {
        current.wrapping_add(1)
    }

    pub const fn previous(&self, current: i32) -> i32 {
        current.wrapping_sub(1)
    }

    pub fn to_slider_value(&self, value: i32) -> f64 {
        let maximum = self.max_inclusive();
        if value == self.min_inclusive {
            0.0
        } else if value == maximum {
            1.0
        } else {
            map(
                f64::from(value) + 0.5,
                f64::from(self.min_inclusive),
                f64::from(maximum) + 1.0,
                0.0,
                1.0,
            )
        }
    }

    pub fn from_slider_value(&self, mut slider: f64) -> i32 {
        let maximum = self.max_inclusive();
        if slider >= 1.0 {
            slider = f64::from(0.99999_f32);
        }
        map(
            slider,
            0.0,
            1.0,
            f64::from(self.min_inclusive),
            f64::from(maximum) + 1.0,
        )
        .floor() as i32
    }

    pub const fn create_cycle_button(&self) -> bool {
        true
    }

    pub fn values(&self) -> Vec<i32> {
        let end = self.max_inclusive().wrapping_add(1);
        if self.min_inclusive >= end {
            Vec::new()
        } else {
            (self.min_inclusive..end).collect()
        }
    }
}
