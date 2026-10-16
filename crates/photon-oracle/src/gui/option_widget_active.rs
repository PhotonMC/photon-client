use super::*;

impl OptionWidget {
    pub(super) fn new(
        lang: &photon_game::text::Lang,
        rect: usize,
        kind: &str,
        value: &Value,
    ) -> Result<Self, String> {
        use photon_game::options::ChatVisibility;
        use photon_game::options::values::{IntRange, UnitDouble};
        use photon_game::text::Component;
        use photon_gui::options::text;
        use photon_gui::widgets::cycle::CycleButton;
        use photon_gui::widgets::slider::OptionSlider;
        let caption = |key: &str| Component::translatable(key);
        let double = || value.as_f64().ok_or("value must be a number");
        Ok(match kind {
            "fov" => Self::Fov(OptionSlider::new(
                lang,
                rect,
                IntRange::new(30, 110),
                caption("options.fov"),
                text::fov,
                value.as_i64().ok_or("value must be an integer")? as i32,
            )),
            "sensitivity" => Self::Unit(OptionSlider::new(
                lang,
                rect,
                UnitDouble,
                caption("options.sensitivity"),
                text::sensitivity,
                double()?,
            )),
            "chat_opacity" => Self::Unit(OptionSlider::new(
                lang,
                rect,
                UnitDouble,
                caption("options.chat.opacity"),
                text::chat_opacity,
                double()?,
            )),
            "invert_y" => Self::Boolean(CycleButton::new(
                lang,
                rect,
                caption("options.invertMouseY"),
                vec![true, false],
                |value| text::on_off(*value),
                &value.as_bool().ok_or("value must be a boolean")?,
            )),
            "chat_visibility" => {
                let id = value.as_i64().ok_or("value must be an id")? as i32;
                Self::Visibility(CycleButton::new(
                    lang,
                    rect,
                    caption("options.chat.visibility"),
                    vec![
                        ChatVisibility::Full,
                        ChatVisibility::System,
                        ChatVisibility::Hidden,
                    ],
                    text::chat_visibility,
                    &ChatVisibility::from_id(id).ok_or("unknown chat visibility")?,
                ))
            }
            other => return Err(format!("unknown option widget {other}")),
        })
    }

    pub(super) fn set_active(&mut self, active: bool) {
        match self {
            Self::Fov(slider) => slider.set_active(active),
            Self::Unit(slider) => slider.set_active(active),
            Self::Boolean(button) => button.set_active(active),
            Self::Visibility(button) => button.set_active(active),
        }
    }

    /// One step of a scenario: a focus, a mouse event, a key or the wheel.
    pub(super) fn step(
        &mut self,
        lang: &photon_game::text::Lang,
        arena: &photon_gui::layout::Arena,
        step: &Value,
    ) -> Result<(), String> {
        use photon_gui::widgets::slider::{InputType, SliderKey};
        let number = |key: &str| step[key].as_f64().unwrap_or(0.0);
        let shift = step["shift"].as_bool().unwrap_or(false);
        let key = match step["key"].as_str() {
            Some("left") => Some(SliderKey::Left),
            Some("right") => Some(SliderKey::Right),
            Some("enter" | "space") => Some(SliderKey::Select),
            _ => None,
        };
        match step["do"].as_str().unwrap_or_default() {
            "focus" => {
                let input = match step["input"].as_str().unwrap_or_default() {
                    "MOUSE" => InputType::Mouse,
                    "KEYBOARD_TAB" => InputType::KeyboardTab,
                    "KEYBOARD_ARROW" => InputType::KeyboardArrow,
                    _ => InputType::None,
                };
                match self {
                    Self::Fov(slider) => slider.set_focused(true, input),
                    Self::Unit(slider) => slider.set_focused(true, input),
                    Self::Boolean(button) => button.set_focused(true),
                    Self::Visibility(button) => button.set_focused(true),
                }
            }
            "unfocus" => match self {
                Self::Fov(slider) => slider.set_focused(false, InputType::None),
                Self::Unit(slider) => slider.set_focused(false, InputType::None),
                Self::Boolean(button) => button.set_focused(false),
                Self::Visibility(button) => button.set_focused(false),
            },
            "click" => {
                let (x, y) = (number("x"), number("y"));
                let at = (x as i32, y as i32);
                match self {
                    Self::Fov(slider) => ignore(slider.clicked(lang, arena, (x, y))),
                    Self::Unit(slider) => ignore(slider.clicked(lang, arena, (x, y))),
                    Self::Boolean(button) => ignore(button.clicked(lang, arena, at, shift)),
                    Self::Visibility(button) => ignore(button.clicked(lang, arena, at, shift)),
                }
            }
            "drag" => match self {
                Self::Fov(slider) => ignore(slider.dragged(lang, arena, number("x"))),
                Self::Unit(slider) => ignore(slider.dragged(lang, arena, number("x"))),
                _ => {}
            },
            "release" => match self {
                Self::Fov(slider) => slider.released(lang),
                Self::Unit(slider) => slider.released(lang),
                _ => {}
            },
            "key" => match (self, key) {
                (Self::Fov(slider), Some(key)) => ignore(slider.key_pressed(lang, arena, key)),
                (Self::Unit(slider), Some(key)) => ignore(slider.key_pressed(lang, arena, key)),
                (Self::Boolean(button), Some(SliderKey::Select)) => {
                    ignore(button.selected(lang, shift));
                }
                (Self::Visibility(button), Some(SliderKey::Select)) => {
                    ignore(button.selected(lang, shift));
                }
                _ => {}
            },
            "scroll" => match self {
                Self::Boolean(button) => ignore(button.scrolled(lang, number("dy"))),
                Self::Visibility(button) => ignore(button.scrolled(lang, number("dy"))),
                _ => {}
            },
            other => return Err(format!("unknown step {other}")),
        }
        Ok(())
    }

    pub(super) fn draw(
        &self,
        frame: &mut photon_render::gui::GuiFrame,
        atlas: &photon_render::gui::GuiAtlas,
        font: &photon_render::gui::font::Font,
        arena: &photon_gui::layout::Arena,
        mouse: (i32, i32),
    ) {
        match self {
            Self::Fov(slider) => slider.draw(frame, atlas, font, arena, mouse),
            Self::Unit(slider) => slider.draw(frame, atlas, font, arena, mouse),
            Self::Boolean(button) => button.draw(frame, atlas, font, arena, mouse),
            Self::Visibility(button) => button.draw(frame, atlas, font, arena, mouse),
        }
    }

    /// The option's value as the oracle writes it: an integer, a boolean,
    /// an id, or the bits of a double.
    pub(super) fn value(&self) -> String {
        match self {
            Self::Fov(slider) => slider.value().to_string(),
            Self::Unit(slider) => format!("{:016x}", slider.value().to_bits()),
            Self::Boolean(button) => button.value().to_string(),
            Self::Visibility(button) => button.value().id().to_string(),
        }
    }
}
