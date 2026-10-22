//! The input controls of a dialog (`InputControlHandlers`): what each one shows, whether the
//! mouse or the keyboard changes it, and the value a button takes from it.

use photon_core::nbt::FieldValue;
use photon_game::dialog::{Control as Spec, Input, InputValue};
use photon_game::text::{Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

use crate::layout::Arena;
use crate::widgets::navigation::Rectangle;
use crate::{FontRuns, FrameRuns};

/// `Checkbox.getBoxSize` and the gap before its text.
const BOX: i32 = 17;
const GAP: i32 = 4;

enum State {
    Check {
        label: Vec<Run>,
        selected: bool,
        on_true: String,
        on_false: String,
    },
}

/// One control of the screen: the place in the layout and what it holds.
pub(super) struct Control {
    pub(super) rect: usize,
    key: String,
    state: State,
}

impl Control {
    /// The control for `input`, or none for a kind that is not built yet.
    pub(super) fn new(
        arena: &mut Arena,
        (font, lang): (&Font, &Lang),
        input: &Input,
    ) -> Option<Self> {
        let Spec::Boolean(boolean) = &input.control else {
            return None;
        };
        let label = lang.runs(&boolean.label, &Style::default());
        let width = BOX + GAP + font.width_of_runs(&label);
        Some(Self {
            rect: arena.add(width, BOX),
            key: input.key.clone(),
            state: State::Check {
                label,
                selected: boolean.initial,
                on_true: boolean.on_true.clone(),
                on_false: boolean.on_false.clone(),
            },
        })
    }

    /// The mouse or the keyboard used the control (`AbstractButton.onPress`).
    pub(super) fn press(&mut self) {
        match &mut self.state {
            State::Check { selected, .. } => *selected = !*selected,
        }
    }

    /// `Action.ValueGetter`: the text a template takes and the tag a payload takes.
    pub(super) fn value(&self) -> InputValue {
        match &self.state {
            State::Check {
                selected,
                on_true,
                on_false,
                ..
            } => InputValue {
                key: self.key.clone(),
                substitution: if *selected { on_true } else { on_false }.clone(),
                tag: FieldValue::Byte(i8::from(*selected)),
            },
        }
    }

    /// The state of the control a screen of another size had (`repositionElements` keeps it).
    pub(super) fn keep(&mut self, old: &Self) {
        match (&mut self.state, &old.state) {
            (
                State::Check { selected, .. },
                State::Check {
                    selected: before, ..
                },
            ) => {
                *selected = *before;
            }
        }
    }

    pub(super) fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        rect: Rectangle,
        focused: bool,
    ) {
        match &self.state {
            State::Check {
                label, selected, ..
            } => {
                let sprite = match (*selected, focused) {
                    (true, true) => "widget/checkbox_selected_highlighted",
                    (true, false) => "widget/checkbox_selected",
                    (false, true) => "widget/checkbox_highlighted",
                    (false, false) => "widget/checkbox",
                };
                frame.sprite(atlas, sprite, rect.x, rect.y, BOX, BOX);
                let (x, y) = (rect.x + BOX + GAP, rect.y + BOX / 2 - 9 / 2);
                frame.styled_text(atlas, font, label, x, y, 0xFFFF_FFFF, true);
            }
        }
    }
}
