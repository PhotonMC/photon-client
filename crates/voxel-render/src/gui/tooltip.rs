//! A widget's tooltip (`Tooltip`, `WidgetTooltipHolder`): shown while the
//! mouse is on the widget, or the keyboard has it, once it has waited out
//! its delay.

use std::cell::Cell;

use voxel_game::text::{Component, Lang, Run, Style};

use crate::gui::chat::split;
use crate::gui::font::Font;
use crate::gui::navigation::Rectangle;
use crate::gui::slider::InputType;
use crate::gui::{GuiAtlas, GuiFrame, Positioner};

/// `Tooltip.MAX_WIDTH`.
const MAX_WIDTH: i32 = 170;

/// `WidgetTooltipHolder`: a widget's tooltip, and when it began to be shown.
#[derive(Default)]
pub struct Holder {
    /// The words, which are split to the tooltip's width as they are shown.
    runs: Option<Vec<Run>>,
    delay_millis: u64,
    /// `wasDisplayed` and `displayStartTime`.
    state: Cell<(bool, u64)>,
}

/// A tooltip a widget asks for in a frame.
pub struct Pending {
    runs: Vec<Run>,
    positioner: Positioner,
}

/// What a tooltip says: lines of messages, a line of `None` being blank
/// (`CommonComponents.joinLines`).
pub fn joined(lang: &Lang, lines: &[Option<Component>]) -> Vec<Run> {
    let mut runs = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            runs.push(Run {
                text: "\n".to_owned(),
                style: Style::default(),
            });
        }
        if let Some(line) = line {
            runs.extend(lang.runs(line, &Style::default()));
        }
    }
    runs
}

impl Holder {
    /// A tooltip of a message, with no delay.
    pub fn of(lang: &Lang, message: &Component) -> Self {
        let mut holder = Self::default();
        holder.set(Some(lang.runs(message, &Style::default())));
        holder
    }

    /// `set`: the words change, the wait stays.
    pub fn set(&mut self, runs: Option<Vec<Run>>) {
        self.runs = runs;
    }

    /// `refreshTooltipForNextRenderPass`: whether the widget shows its
    /// tooltip now, and where it goes.
    pub fn refresh(
        &self,
        millis: u64,
        (hovered, focused): (bool, bool),
        input: InputType,
        widget: Rectangle,
    ) -> Option<Pending> {
        let Some(runs) = &self.runs else {
            self.state.set((false, 0));
            return None;
        };
        let keyboard = matches!(input, InputType::KeyboardArrow | InputType::KeyboardTab);
        let should_display = hovered || (focused && keyboard);
        let (was_displayed, mut started) = self.state.get();
        if should_display != was_displayed && should_display {
            started = millis;
        }
        self.state.set((should_display, started));
        (should_display && millis.saturating_sub(started) > self.delay_millis).then(|| Pending {
            runs: runs.clone(),
            positioner: if !hovered && focused && keyboard {
                Positioner::BelowOrAbove(widget)
            } else {
                Positioner::Menu(widget)
            },
        })
    }
}

impl Pending {
    /// `extractDeferredElements`: the tooltip over everything drawn.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        mouse: (i32, i32),
        screen: (i32, i32),
    ) {
        let lines = split(&self.runs, MAX_WIDTH, font);
        frame.tooltip_with(atlas, font, &lines, &self.positioner, mouse, screen);
    }
}
