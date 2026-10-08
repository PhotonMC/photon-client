//! GUI family: the screens' drawing and input, answered with the production widgets.

#![expect(
    clippy::too_many_lines,
    reason = "the GUI family answers every screen scenario in one dispatch"
)]

use super::{Value, json};

/// A clipboard for a test with nothing to copy.
pub(crate) struct NoClipboard;

impl voxel_render::gui::widgets::edit_box::Clipboard for NoClipboard {
    fn get(&mut self) -> String {
        String::new()
    }

    fn set(&mut self, _text: &str) {}
}

/// What a step does with a result it has no use for.
/// A key step of a settings screen.
enum MenuKey {
    Tab(bool),
    Key(voxel_render::gui::widgets::slider::SliderKey),
}

/// The widget an option makes, of the kinds the scenarios name.
enum OptionWidget {
    Fov(voxel_render::gui::widgets::slider::OptionSlider<voxel_game::options::values::IntRange>),
    Unit(voxel_render::gui::widgets::slider::OptionSlider<voxel_game::options::values::UnitDouble>),
    Boolean(voxel_render::gui::widgets::cycle::CycleButton<bool>),
    Visibility(voxel_render::gui::widgets::cycle::CycleButton<voxel_game::options::ChatVisibility>),
}

mod option_widget_active;

mod scoreboard;
use scoreboard::*;
mod result;
pub(crate) use result::*;
mod written;
use written::*;
