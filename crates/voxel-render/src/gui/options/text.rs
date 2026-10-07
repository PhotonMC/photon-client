//! The words on an option's widget (`Options`' value labels and
//! `CommonComponents.optionNameValue`): the caption and the value in one
//! line.

use voxel_game::options::ChatVisibility;
use voxel_game::text::Component;

/// `CommonComponents.OPTION_ON` or `OPTION_OFF`.
pub fn on_off(value: bool) -> Component {
    Component::translatable(if value { "options.on" } else { "options.off" })
}

/// `Options.genericValueLabel`: "caption: value".
pub fn generic_value(caption: &Component, value: Component) -> Component {
    Component::translatable_with("options.generic_value", vec![caption.clone(), value])
}

/// `Options.genericValueLabel` of a number.
pub fn generic_int(caption: &Component, value: i32) -> Component {
    generic_value(caption, Component::literal(value.to_string()))
}

/// `Options.percentValueLabel`: the share cut to a whole percent.
pub fn percent_value(caption: &Component, value: f64) -> Component {
    Component::translatable_with(
        "options.percent_value",
        vec![
            caption.clone(),
            Component::literal(((value * 100.0) as i32).to_string()),
        ],
    )
}

/// `percentValueOrOffLabel`: a sound slider's label, "Off" at zero.
pub fn percent_value_or_off(caption: &Component, value: &f64) -> Component {
    if *value == 0.0 {
        generic_value(caption, on_off(false))
    } else {
        percent_value(caption, *value)
    }
}

/// `fov`'s label: its ends are named.
pub fn fov(caption: &Component, value: &i32) -> Component {
    match *value {
        70 => generic_value(caption, Component::translatable("options.fov.min")),
        110 => generic_value(caption, Component::translatable("options.fov.max")),
        degrees => generic_int(caption, degrees),
    }
}

/// `mouseSensitivity`'s label: its ends are named, and it reads out twice
/// its value.
pub fn sensitivity(caption: &Component, value: &f64) -> Component {
    if *value == 0.0 {
        generic_value(caption, Component::translatable("options.sensitivity.min"))
    } else if *value == 1.0 {
        generic_value(caption, Component::translatable("options.sensitivity.max"))
    } else {
        percent_value(caption, 2.0 * value)
    }
}

/// `chatOpacity`'s label: the share the text is drawn at.
pub fn chat_opacity(caption: &Component, value: &f64) -> Component {
    percent_value(caption, value * 0.9 + 0.1)
}

/// `chatVisibility`'s value: the name of the choice.
pub fn chat_visibility(value: &ChatVisibility) -> Component {
    Component::translatable(match value {
        ChatVisibility::Full => "options.chat.visibility.full",
        ChatVisibility::System => "options.chat.visibility.system",
        ChatVisibility::Hidden => "options.chat.visibility.hidden",
    })
}
