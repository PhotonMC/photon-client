//! GUI family: the screens' drawing and input, answered with the production widgets.

use super::{Value, json};

/// A clipboard for a test with nothing to copy.
pub(crate) struct NoClipboard;

impl photon_gui::widgets::edit_box::Clipboard for NoClipboard {
    fn get(&mut self) -> String {
        String::new()
    }

    fn set(&mut self, _text: &str) {}
}

/// What a step does with a result it has no use for.
/// A key step of a settings screen.
enum MenuKey {
    Tab(bool),
    Key(photon_gui::widgets::slider::SliderKey),
}

/// The widget an option makes, of the kinds the scenarios name.
enum OptionWidget {
    Fov(photon_gui::widgets::slider::OptionSlider<photon_game::options::values::IntRange>),
    Unit(photon_gui::widgets::slider::OptionSlider<photon_game::options::values::UnitDouble>),
    Boolean(photon_gui::widgets::cycle::CycleButton<bool>),
    Visibility(photon_gui::widgets::cycle::CycleButton<photon_game::options::ChatVisibility>),
}

mod option_widget_active;

mod scoreboard;
use scoreboard::*;
mod result;
pub(crate) use result::*;
mod written;
use written::*;
