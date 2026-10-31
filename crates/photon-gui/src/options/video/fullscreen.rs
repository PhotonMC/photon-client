//! The fullscreen resolution slider: `-1` is the monitor's current mode, then the modes it lists.

use photon_game::options::values::IntRange;
use photon_game::text::Component;
use photon_render::window::VideoMode;

use super::*;
use crate::widgets::slider::SliderValues;

/// The monitor's modes and the one the options prefer; no modes at all means no monitor, and the
/// slider says the setting is unavailable.
#[derive(Debug, Clone, Default)]
pub struct FullscreenModes {
    modes: Option<Vec<VideoMode>>,
    preferred: Option<VideoMode>,
}

impl FullscreenModes {
    pub fn new(modes: Option<Vec<VideoMode>>, preferred: Option<VideoMode>) -> Self {
        Self { modes, preferred }
    }

    fn range(&self) -> IntRange {
        IntRange::new(-1, self.modes.as_ref().map_or(-1, |m| m.len() as i32 - 1))
    }

    fn choice(&self, index: i32) -> ModeChoice {
        ModeChoice {
            index,
            mode: usize::try_from(index)
                .ok()
                .and_then(|at| self.modes.as_ref()?.get(at).copied()),
            available: self.modes.is_some(),
        }
    }

    /// The choice the slider starts on: where the preferred mode stands in the list, else current.
    pub(super) fn initial(&self) -> ModeChoice {
        let index = self
            .preferred
            .and_then(|wanted| self.modes.as_ref()?.iter().position(|mode| *mode == wanted))
            .map_or(-1, |at| at as i32);
        self.choice(index)
    }
}

/// One place on the slider.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModeChoice {
    index: i32,
    /// The mode at `index`; none for the current mode.
    mode: Option<VideoMode>,
    available: bool,
}

impl ModeChoice {
    /// What the setting asks the window for: `Some(None)` is the monitor's current mode, and
    /// none is no monitor, which asks for nothing.
    pub fn preferred(&self) -> Option<Option<VideoMode>> {
        self.available.then_some(self.mode)
    }
}

impl SliderValues for FullscreenModes {
    type Value = ModeChoice;

    fn position_of(&self, value: &ModeChoice) -> f64 {
        self.range().to_slider_value(value.index)
    }

    fn value_at(&self, slider: f64) -> ModeChoice {
        self.choice(self.range().from_slider_value(slider))
    }

    fn next(&self, current: &ModeChoice) -> Option<ModeChoice> {
        Some(self.choice(self.range().next(current.index)))
    }

    fn previous(&self, current: &ModeChoice) -> Option<ModeChoice> {
        Some(self.choice(self.range().previous(current.index)))
    }
}

/// The slider's text: the mode's size, rate and depth, or Current, or that there is no monitor.
pub(super) fn fullscreen_resolution(caption: &Component, value: &ModeChoice) -> Component {
    if !value.available {
        return Component::translatable("options.fullscreen.unavailable");
    }
    match value.mode {
        None => generic_value(
            caption,
            Component::translatable("options.fullscreen.current"),
        ),
        Some(mode) => generic_value(
            caption,
            Component::translatable_with(
                "options.fullscreen.entry",
                vec![
                    Component::literal(mode.width.to_string()),
                    Component::literal(mode.height.to_string()),
                    Component::literal(mode.refresh_rate.to_string()),
                    Component::literal(mode.bits().to_string()),
                ],
            ),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mode(width: i32, height: i32) -> VideoMode {
        VideoMode {
            width,
            height,
            red_bits: 8,
            green_bits: 8,
            blue_bits: 8,
            refresh_rate: 60,
        }
    }

    fn modes() -> FullscreenModes {
        FullscreenModes::new(
            Some(vec![mode(2560, 1440), mode(1920, 1080), mode(1280, 720)]),
            Some(mode(1920, 1080)),
        )
    }

    #[test]
    fn the_slider_starts_on_the_preferred_mode_and_current_when_it_is_not_listed() {
        assert_eq!(modes().initial().preferred(), Some(Some(mode(1920, 1080))));
        let unlisted = FullscreenModes::new(Some(vec![mode(1280, 720)]), Some(mode(800, 600)));
        assert_eq!(unlisted.initial().preferred(), Some(None));
    }

    #[test]
    fn the_slider_walks_current_then_each_mode() {
        let set = modes();
        // The slider keeps its handle on the bar, as `OptionSlider` does after a step.
        let step = |at: &ModeChoice| {
            let position = set
                .position_of(&set.next(at).expect("steps"))
                .clamp(0.0, 1.0);
            set.value_at(position)
        };
        let mut at = set.choice(-1);
        let mut seen = Vec::new();
        for _ in 0..5 {
            at = step(&at);
            seen.push(at.preferred());
        }
        assert_eq!(
            seen,
            [
                Some(Some(mode(2560, 1440))),
                Some(Some(mode(1920, 1080))),
                Some(Some(mode(1280, 720))),
                Some(Some(mode(1280, 720))),
                Some(Some(mode(1280, 720))),
            ]
        );
        assert_eq!(set.value_at(0.0).preferred(), Some(None));
        assert_eq!(set.value_at(1.0).preferred(), Some(Some(mode(1280, 720))));
    }

    #[test]
    fn without_a_monitor_the_setting_asks_for_nothing() {
        let none = FullscreenModes::default();
        assert_eq!(none.initial().preferred(), None);
        assert_eq!(none.value_at(1.0).preferred(), None);
    }
}
