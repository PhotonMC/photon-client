//! Server dialogs (`net.minecraft.server.dialog`): the data the server sends, as far as the
//! notice and confirmation screens use it. Buttons, lists, links and inputs are T106.

use photon_core::nbt::{Field, FieldValue};

use crate::text::{ClickEvent, Component};

/// `DialogAction`: what happens to the screen once a button has been used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AfterAction {
    Close,
    None,
    WaitForResponse,
}

impl AfterAction {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "close" => Self::Close,
            "none" => Self::None,
            "wait_for_response" => Self::WaitForResponse,
            _ => return Option::None,
        })
    }

    /// `DialogAction.willUnpause`.
    pub fn will_unpause(self) -> bool {
        self != Self::None
    }
}

/// `PlainMessage`.
#[derive(Debug, Clone, PartialEq)]
pub struct PlainMessage {
    pub contents: Component,
    pub width: i32,
}

/// `DialogBody`.
#[derive(Debug, Clone, PartialEq)]
pub enum Body {
    Plain(PlainMessage),
    Item {
        id: String,
        count: i32,
        description: Option<PlainMessage>,
        show_decorations: bool,
        show_tooltip: bool,
        width: i32,
        height: i32,
    },
}

/// What a button does: the click event it runs. `dynamic_*` actions fill a command template from
/// the dialog's inputs and belong to T106.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Static(ClickEvent),
    Dynamic,
}

/// `ActionButton`.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionButton {
    pub label: Component,
    pub tooltip: Option<Component>,
    pub width: i32,
    pub action: Option<Action>,
}

/// `CommonDialogData`.
#[derive(Debug, Clone, PartialEq)]
pub struct CommonData {
    pub title: Component,
    pub external_title: Option<Component>,
    pub can_close_with_escape: bool,
    pub pause: bool,
    pub after_action: AfterAction,
    pub body: Vec<Body>,
    /// Whether the dialog asks for inputs; they are not read yet.
    pub has_inputs: bool,
}

/// A dialog, as far as it is read.
#[derive(Debug, Clone, PartialEq)]
pub enum Dialog {
    Notice {
        common: Box<CommonData>,
        action: Box<ActionButton>,
    },
    Confirmation {
        common: Box<CommonData>,
        yes: Box<ActionButton>,
        no: Box<ActionButton>,
    },
    /// A type whose screen is not built yet.
    Other(Box<CommonData>),
}

impl Dialog {
    pub fn common(&self) -> &CommonData {
        match self {
            Self::Notice { common, .. }
            | Self::Confirmation { common, .. }
            | Self::Other(common) => common,
        }
    }

    /// `Dialog.onCancel`: what closing the screen runs.
    pub fn on_cancel(&self) -> Option<&Action> {
        match self {
            Self::Notice { action, .. } => action.action.as_ref(),
            Self::Confirmation { no, .. } => no.action.as_ref(),
            Self::Other(_) => None,
        }
    }

    /// `SimpleDialog.mainActions`.
    pub fn main_actions(&self) -> Vec<&ActionButton> {
        match self {
            Self::Notice { action, .. } => vec![&**action],
            Self::Confirmation { yes, no, .. } => vec![&**yes, &**no],
            Self::Other(_) => Vec::new(),
        }
    }

    /// `Dialog.DIRECT_CODEC` from the tag.
    pub fn from_nbt(value: &FieldValue) -> Option<Self> {
        let FieldValue::Compound(fields) = value else {
            return None;
        };
        let common = Box::new(CommonData::from_fields(fields)?);
        Some(match strip_namespace(text(fields, "type")?) {
            "notice" => Self::Notice {
                common,
                action: Box::new(match get(fields, "action") {
                    Some(value) => ActionButton::from_nbt(value)?,
                    None => ActionButton {
                        label: Component::translatable("gui.ok"),
                        tooltip: None,
                        width: 150,
                        action: None,
                    },
                }),
            },
            "confirmation" => Self::Confirmation {
                common,
                yes: Box::new(ActionButton::from_nbt(get(fields, "yes")?)?),
                no: Box::new(ActionButton::from_nbt(get(fields, "no")?)?),
            },
            _ => Self::Other(common),
        })
    }
}

/// A dialog a click event shows: one inline, or one the server's registry names.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogRef {
    Inline(Box<Dialog>),
    Registry(String),
}

impl DialogRef {
    /// `Dialog.CODEC` (a registry file codec) from the tag.
    pub fn from_nbt(value: &FieldValue) -> Option<Self> {
        match value {
            FieldValue::Str(id) => Some(Self::Registry(id.clone())),
            other => Some(Self::Inline(Box::new(Dialog::from_nbt(other)?))),
        }
    }
}

impl CommonData {
    fn from_fields(fields: &[Field]) -> Option<Self> {
        let flag = |key: &str, default: bool| match get(fields, key) {
            Some(FieldValue::Byte(value)) => *value != 0,
            _ => default,
        };
        let after_action = match get(fields, "after_action") {
            Some(FieldValue::Str(name)) => AfterAction::from_name(name)?,
            _ => AfterAction::Close,
        };
        let pause = flag("pause", true);
        // The codec refuses a dialog that pauses the game and keeps it paused.
        if pause && !after_action.will_unpause() {
            return None;
        }
        Some(Self {
            title: Component::from_nbt(get(fields, "title")?)?,
            external_title: get(fields, "external_title").and_then(Component::from_nbt),
            can_close_with_escape: flag("can_close_with_escape", true),
            pause,
            after_action,
            body: match get(fields, "body") {
                Some(FieldValue::List(items)) => {
                    items.iter().map(Body::from_nbt).collect::<Option<_>>()?
                }
                Some(single) => vec![Body::from_nbt(single)?],
                None => Vec::new(),
            },
            has_inputs: matches!(get(fields, "inputs"), Some(FieldValue::List(items)) if !items.is_empty()),
        })
    }
}

impl PlainMessage {
    /// `PlainMessage.CODEC`: the record, or a bare component.
    fn from_nbt(value: &FieldValue) -> Option<Self> {
        if let FieldValue::Compound(fields) = value
            && let Some(contents) = get(fields, "contents")
        {
            return Some(Self {
                contents: Component::from_nbt(contents)?,
                width: width(fields, "width", 200, 1024)?,
            });
        }
        Some(Self {
            contents: Component::from_nbt(value)?,
            width: 200,
        })
    }
}

impl Body {
    fn from_nbt(value: &FieldValue) -> Option<Self> {
        let FieldValue::Compound(fields) = value else {
            return Some(Self::Plain(PlainMessage::from_nbt(value)?));
        };
        match strip_namespace(text(fields, "type")?) {
            "plain_message" => Some(Self::Plain(PlainMessage::from_nbt(value)?)),
            "item" => {
                let FieldValue::Compound(item) = get(fields, "item")? else {
                    return None;
                };
                let flag = |key: &str| !matches!(get(fields, key), Some(FieldValue::Byte(0)));
                Some(Self::Item {
                    id: text(item, "id")?.to_owned(),
                    count: match get(item, "count") {
                        Some(FieldValue::Int(count)) => *count,
                        _ => 1,
                    },
                    description: match get(fields, "description") {
                        Some(description) => Some(PlainMessage::from_nbt(description)?),
                        None => None,
                    },
                    show_decorations: flag("show_decorations"),
                    show_tooltip: flag("show_tooltip"),
                    width: width(fields, "width", 16, 256)?,
                    height: width(fields, "height", 16, 256)?,
                })
            }
            _ => None,
        }
    }
}

impl ActionButton {
    fn from_nbt(value: &FieldValue) -> Option<Self> {
        let FieldValue::Compound(fields) = value else {
            return None;
        };
        Some(Self {
            label: Component::from_nbt(get(fields, "label")?)?,
            tooltip: get(fields, "tooltip").and_then(Component::from_nbt),
            width: width(fields, "width", 150, 1024)?,
            action: match get(fields, "action") {
                Some(FieldValue::Compound(action)) => Some(Action::from_fields(action)?),
                _ => None,
            },
        })
    }
}

impl Action {
    /// `Action.CODEC`: the click events a server may send, and the templated kinds.
    fn from_fields(fields: &[Field]) -> Option<Self> {
        match strip_namespace(text(fields, "type")?) {
            "dynamic/run_command" | "dynamic/custom" => Some(Self::Dynamic),
            _ => Some(Self::Static(ClickEvent::from_nbt_fields(fields, "type")?)),
        }
    }
}

fn get<'a>(fields: &'a [Field], key: &str) -> Option<&'a FieldValue> {
    fields
        .iter()
        .find(|field| field.key == key)
        .map(|field| &field.value)
}

fn text<'a>(fields: &'a [Field], key: &str) -> Option<&'a str> {
    match get(fields, key) {
        Some(FieldValue::Str(text)) => Some(text),
        _ => None,
    }
}

fn strip_namespace(name: &str) -> &str {
    name.strip_prefix("minecraft:").unwrap_or(name)
}

/// An `intRange(1, max)` field with a default.
fn width(fields: &[Field], key: &str, default: i32, max: i32) -> Option<i32> {
    match get(fields, key) {
        None => Some(default),
        Some(FieldValue::Int(value)) if (1..=max).contains(value) => Some(*value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(key: &str, value: FieldValue) -> Field {
        Field {
            key: key.to_owned(),
            value,
        }
    }

    fn string(text: &str) -> FieldValue {
        FieldValue::Str(text.to_owned())
    }

    #[test]
    fn a_notice_reads_its_defaults_and_a_confirmation_cancels_with_no() {
        let notice = FieldValue::Compound(vec![
            field("type", string("minecraft:notice")),
            field("title", string("Hi")),
            field("body", string("Read this")),
        ]);
        let Some(Dialog::Notice { common, action }) = Dialog::from_nbt(&notice) else {
            panic!("notice");
        };
        assert!(common.pause && common.can_close_with_escape);
        assert_eq!(common.after_action, AfterAction::Close);
        assert_eq!(common.body.len(), 1);
        assert_eq!(action.width, 150);

        let button = |label: &str, command: Option<&str>| {
            let mut fields = vec![field("label", string(label))];
            if let Some(command) = command {
                fields.push(field(
                    "action",
                    FieldValue::Compound(vec![
                        field("type", string("run_command")),
                        field("command", string(command)),
                    ]),
                ));
            }
            FieldValue::Compound(fields)
        };
        let confirmation = FieldValue::Compound(vec![
            field("type", string("confirmation")),
            field("title", string("Sure?")),
            field("yes", button("Yes", None)),
            field("no", button("No", Some("say no"))),
        ]);
        let dialog = Dialog::from_nbt(&confirmation).expect("confirmation");
        assert_eq!(
            dialog.on_cancel(),
            Some(&Action::Static(ClickEvent::RunCommand("say no".to_owned())))
        );
        assert_eq!(dialog.main_actions().len(), 2);
    }

    #[test]
    fn a_dialog_that_pauses_and_stays_paused_is_refused() {
        let dialog = FieldValue::Compound(vec![
            field("type", string("notice")),
            field("title", string("Hi")),
            field("after_action", string("none")),
        ]);
        assert_eq!(Dialog::from_nbt(&dialog), None);
    }
}
