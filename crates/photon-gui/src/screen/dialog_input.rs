//! The input controls of a dialog (`InputControlHandlers`): what each one shows, whether the
//! mouse or the keyboard changes it, and the value a button takes from it.

use photon_core::nbt::FieldValue;
mod slider;

use slider::Slider;

use photon_game::dialog::{Control as Spec, Input, InputValue, SingleOptionInput};
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

use crate::layout::Arena;
use crate::widgets::navigation::Rectangle;
use crate::widgets::{Kind, Widget};
use crate::{FontRuns, FrameRuns};

/// `Checkbox.getBoxSize` and the gap before its text.
const BOX: i32 = 17;
const GAP: i32 = 4;

enum State {
    Slider(Box<Slider>),
    /// A `CycleButton`: the text of the button for each value, and the value ids.
    Cycle {
        labels: Vec<Vec<Run>>,
        ids: Vec<String>,
        index: usize,
    },
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
        match &input.control {
            Spec::Boolean(boolean) => {
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
            Spec::SingleOption(option) => Some(Self {
                rect: arena.add(option.width, 20),
                key: input.key.clone(),
                state: cycle(lang, option),
            }),
            Spec::NumberRange(range) => Some(Self {
                rect: arena.add(range.width, 20),
                key: input.key.clone(),
                state: State::Slider(Box::new(Slider::new(lang, range))),
            }),
            Spec::Text(_) => None,
        }
    }

    /// The mouse or the keyboard used the control (`AbstractButton.onPress`).
    /// With Shift down a cycle button goes back.
    pub(super) fn press(&mut self, shift: bool) {
        match &mut self.state {
            State::Check { selected, .. } => *selected = !*selected,
            State::Cycle { labels, index, .. } => {
                let count = labels.len();
                *index = (*index + if shift { count - 1 } else { 1 }) % count;
            }
            State::Slider(slider) => slider.select(),
        }
    }

    /// A press of the primary button on the control at `rect`.
    pub(super) fn clicked(&mut self, x: i32, rect: Rectangle, shift: bool) {
        match &mut self.state {
            State::Slider(slider) => slider.clicked(x, rect),
            _ => self.press(shift),
        }
    }

    pub(super) fn dragged(&mut self, x: i32, rect: Rectangle) {
        if let State::Slider(slider) = &mut self.state {
            slider.dragged(x, rect);
        }
    }

    pub(super) fn released(&mut self) {
        if let State::Slider(slider) = &mut self.state {
            slider.released();
        }
    }

    pub(super) fn focus(&mut self, gained: bool) {
        if let State::Slider(slider) = &mut self.state {
            slider.focus(gained);
        }
    }

    /// An arrow key, when the control takes it.
    pub(super) fn arrow(&mut self, right: bool, width: i32) -> bool {
        match &mut self.state {
            State::Slider(slider) => slider.arrow(right, width),
            _ => false,
        }
    }

    /// `Action.ValueGetter`: the text a template takes and the tag a payload takes.
    pub(super) fn value(&self) -> InputValue {
        match &self.state {
            State::Slider(slider) => InputValue {
                key: self.key.clone(),
                substitution: slider.text(),
                tag: FieldValue::Float(slider.scaled()),
            },
            State::Cycle { ids, index, .. } => InputValue {
                key: self.key.clone(),
                substitution: ids[*index].clone(),
                tag: FieldValue::Str(ids[*index].clone()),
            },
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
            (State::Cycle { index, .. }, State::Cycle { index: before, .. }) => *index = *before,
            (State::Slider(slider), State::Slider(before)) => slider.copy_from(before),
            _ => {}
        }
    }

    pub(super) fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        rect: Rectangle,
        (focused, hovered): (bool, bool),
    ) {
        match &self.state {
            State::Slider(slider) => slider.draw(frame, (atlas, font), rect, (focused, hovered)),
            State::Cycle { labels, index, .. } => {
                let mut arena = Arena::default();
                let leaf = arena.add(rect.width, rect.height);
                arena.0[leaf].x = rect.x;
                arena.0[leaf].y = rect.y;
                let button = Widget {
                    rect: leaf,
                    active: true,
                    kind: Kind::Button {
                        label: labels[*index].clone(),
                    },
                };
                button.draw(frame, atlas, font, &arena, focused || hovered);
            }
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

/// `SingleOptionHandler`: a cycle button on the first entry marked, else the first.
fn cycle(lang: &Lang, option: &SingleOptionInput) -> State {
    let labels = option
        .entries
        .iter()
        .map(|entry| {
            let value = entry
                .display
                .clone()
                .unwrap_or_else(|| Component::literal(entry.id.as_str()));
            // `CommonComponents.optionNameValue` unless the label is hidden.
            let shown = if option.label_visible {
                Component::translatable_with(
                    "options.generic_value",
                    vec![option.label.clone(), value],
                )
            } else {
                value
            };
            lang.runs(&shown, &Style::default())
        })
        .collect();
    let initial = option.initial_index();
    State::Cycle {
        labels,
        ids: option
            .entries
            .iter()
            .map(|entry| entry.id.clone())
            .collect(),
        index: initial,
    }
}
