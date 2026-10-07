use super::*;

impl Options {
    /// `Options.getSoundSourceVolume`: what the player set a sound category to.
    pub fn sound_category_volume(&self, source: SoundSource) -> f64 {
        self.sound_volumes[source.ordinal() as usize]
    }
}

/// `Options.logMouse`.
pub fn log_mouse(value: i32) -> f64 {
    10.0_f64.powf(f64::from(value) / 100.0)
}

/// `Options.unlogMouse`.
pub fn unlog_mouse(value: f64) -> i32 {
    (value.log10() * 100.0).floor() as i32
}

/// The wheel's value set, `IntRange(-200, 100).xmap(logMouse, unlogMouse)`:
/// a value takes the nearest step at or below it.
pub(super) fn mouse_wheel_step(value: f64) -> Option<f64> {
    IntRange::new(-200, 100)
        .xmap(log_mouse, |value: &f64| unlog_mouse(*value), false)
        .validate_value(&value)
}

/// `fov`'s range, in degrees.
pub(super) const FOV: IntRange = IntRange::new(30, 110);

pub(super) const FOV_DEFAULT: i32 = 70;

/// `ClampingLazyMaxIntRange`'s encodable maximum for `guiScale`, which is
/// also the largest the option takes before a window exists.
pub(super) const GUI_SCALE_MAX: i32 = 2_147_483_646;

/// `ChatComponent.getHeight(1.0) - 20`, the span the unfocused height's
/// default (70 pixels) is a share of.
pub(super) const CHAT_HEIGHT_SPAN: f64 = 160.0;
