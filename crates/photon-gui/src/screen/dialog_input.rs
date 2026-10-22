//! The input controls of a dialog (`InputControlHandlers`): what each one shows, whether the
//! mouse or the keyboard changes it, and the value a button takes from it.

use photon_core::nbt::FieldValue;
mod area;
mod slider;
mod text;

use area::Area;
use slider::Slider;
use text::Text;

use photon_game::dialog::{
    Control as Spec, Input, InputValue, SingleOptionInput, escape_without_quotes,
};
use photon_game::text::{Component, Lang, Run, Style};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

use crate::layout::Arena;
use crate::widgets::edit_box::{Clipboard, edit_key};
use crate::widgets::navigation::Rectangle;
use crate::widgets::{Kind, Widget};
use crate::{FontRuns, FrameRuns};

/// `Checkbox.getBoxSize` and the gap before its text.
const BOX: i32 = 17;
const GAP: i32 = 4;

enum State {
    Slider(Box<Slider>),
    Text(Box<Text>),
    Area(Box<Area>),
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
    /// The control for `input`.
    pub(super) fn new(arena: &mut Arena, (font, lang): (&Font, &Lang), input: &Input) -> Self {
        match &input.control {
            Spec::Boolean(boolean) => {
                let label = lang.runs(&boolean.label, &Style::default());
                let width = BOX + GAP + font.width_of_runs(&label);
                Self {
                    rect: arena.add(width, BOX),
                    key: input.key.clone(),
                    state: State::Check {
                        label,
                        selected: boolean.initial,
                        on_true: boolean.on_true.clone(),
                        on_false: boolean.on_false.clone(),
                    },
                }
            }
            Spec::SingleOption(option) => Self {
                rect: arena.add(option.width, 20),
                key: input.key.clone(),
                state: cycle(lang, option),
            },
            Spec::NumberRange(range) => Self {
                rect: arena.add(range.width, 20),
                key: input.key.clone(),
                state: State::Slider(Box::new(Slider::new(lang, range))),
            },
            Spec::Text(text) => {
                let (state, (width, height)) = match &text.multiline {
                    None => {
                        let (text, size) = Text::new(font, lang, text);
                        (State::Text(Box::new(text)), size)
                    }
                    Some(multiline) => {
                        let (area, size) = Area::new(font, lang, (text, multiline));
                        (State::Area(Box::new(area)), size)
                    }
                };
                Self {
                    rect: arena.add(width, height),
                    key: input.key.clone(),
                    state,
                }
            }
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
            State::Text(_) | State::Area(_) => {}
        }
    }

    /// Where the mouse and the focus find the control: a text box is the box under its label.
    pub(super) fn hit_rect(&self, rect: Rectangle) -> Rectangle {
        match &self.state {
            State::Text(text) => text.box_rect(rect),
            State::Area(area) => area.box_rect(rect),
            _ => rect,
        }
    }

    /// A key the focused text box edits with. Whether it took the key.
    pub(super) fn edit(
        &mut self,
        font: &Font,
        key: glfw::Key,
        modifiers: (bool, bool),
        clipboard: &mut dyn Clipboard,
    ) -> bool {
        match &mut self.state {
            State::Text(text) => {
                edit_key(key).is_some_and(|key| text.key(font, key, modifiers, clipboard))
            }
            State::Area(area) => area.key(font, key, modifiers, clipboard),
            _ => false,
        }
    }

    /// A character typed into the focused text box. Whether it took it.
    pub(super) fn char_typed(&mut self, font: &Font, character: char) -> bool {
        match &mut self.state {
            State::Text(text) => text.char_typed(font, character),
            State::Area(area) => area.char_typed(font, character),
            _ => false,
        }
    }

    /// A press of the primary button on the control at `rect`.
    pub(super) fn clicked(
        &mut self,
        (x, y): (i32, i32),
        rect: Rectangle,
        (shift, font): (bool, &Font),
    ) {
        match &mut self.state {
            State::Slider(slider) => slider.clicked(x, rect),
            State::Text(text) => text.clicked(x, rect, font),
            State::Area(area) => area.clicked(font, (x, y), (rect, shift)),
            _ => self.press(shift),
        }
    }

    pub(super) fn dragged(
        &mut self,
        at: (i32, i32),
        rect: Rectangle,
        (font, shift): (&Font, bool),
    ) {
        match &mut self.state {
            State::Slider(slider) => slider.dragged(at.0, rect),
            State::Area(area) => area.dragged(font, at, (rect, shift)),
            _ => {}
        }
    }

    pub(super) fn released(&mut self) {
        match &mut self.state {
            State::Slider(slider) => slider.released(),
            State::Area(area) => area.released(),
            _ => {}
        }
    }

    pub(super) fn focus(&mut self, gained: bool) {
        match &mut self.state {
            State::Slider(slider) => slider.focus(gained),
            State::Text(text) => text.focus(gained),
            State::Area(area) => area.focus(gained),
            _ => {}
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
            State::Text(text) => {
                let value = text.value();
                InputValue {
                    key: self.key.clone(),
                    substitution: escape_without_quotes(&value),
                    tag: FieldValue::Str(value),
                }
            }
            State::Area(area) => {
                let value = area.value();
                InputValue {
                    key: self.key.clone(),
                    substitution: escape_without_quotes(&value),
                    tag: FieldValue::Str(value),
                }
            }
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
            (State::Text(text), State::Text(before)) => text.copy_from(before),
            (State::Area(area), State::Area(before)) => area.copy_from(before),
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
            State::Text(text) => text.draw(frame, (atlas, font), rect, focused),
            State::Area(area) => area.draw(frame, (atlas, font), rect),
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
