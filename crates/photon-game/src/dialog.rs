//! Server dialogs (`net.minecraft.server.dialog`): the data the server sends, as far as the
//! notice, confirmation, multi-action and button list screens use it. Inputs are T106.

mod input;
mod list;
mod template;

pub use input::{
    BooleanInput, Control, Input, InputValue, Multiline, NumberRangeInput, OptionEntry, RangeInfo,
    SingleOptionInput, TextInput, range_text,
};
pub use list::{DialogContext, DialogSet, ListSource};
pub use template::{Template, escape_without_quotes};

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

/// What a button does: the click event it runs, or one made from the dialog's inputs.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Static(ClickEvent),
    /// `CommandTemplate`: a command with the inputs filled in.
    CommandTemplate(Template),
    /// `CustomAll`: a custom event whose payload holds every input.
    CustomAll {
        id: String,
        additions: Option<Vec<Field>>,
    },
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
    pub inputs: Vec<Input>,
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
    /// `MultiActionDialog`: a grid of buttons and an optional exit button.
    MultiAction {
        common: Box<CommonData>,
        actions: Vec<ActionButton>,
        exit: Option<Box<ActionButton>>,
        columns: i32,
    },
    /// `ServerLinksDialog` and `DialogListDialog`: a `MultiAction` of what `source` names, once
    /// `resolved`.
    List {
        common: Box<CommonData>,
        source: ListSource,
        exit: Option<Box<ActionButton>>,
        columns: i32,
        button_width: i32,
    },
    /// A type whose screen is not built yet.
    Other(Box<CommonData>),
}

impl Dialog {
    pub fn common(&self) -> &CommonData {
        match self {
            Self::Notice { common, .. }
            | Self::Confirmation { common, .. }
            | Self::MultiAction { common, .. }
            | Self::List { common, .. }
            | Self::Other(common) => common,
        }
    }

    /// `Dialog.onCancel`: what closing the screen runs.
    pub fn on_cancel(&self) -> Option<&Action> {
        match self {
            Self::Notice { action, .. } => action.action.as_ref(),
            Self::Confirmation { no, .. } => no.action.as_ref(),
            Self::MultiAction { exit, .. } | Self::List { exit, .. } => {
                exit.as_ref().and_then(|exit| exit.action.as_ref())
            }
            Self::Other(_) => None,
        }
    }

    /// `SimpleDialog.mainActions`, or for a button list its buttons and then the exit button.
    pub fn main_actions(&self) -> Vec<&ActionButton> {
        match self {
            Self::Notice { action, .. } => vec![&**action],
            Self::Confirmation { yes, no, .. } => vec![&**yes, &**no],
            Self::MultiAction { actions, exit, .. } => {
                actions.iter().chain(exit.as_deref()).collect()
            }
            Self::List { exit, .. } => exit.as_deref().into_iter().collect(),
            Self::Other(_) => Vec::new(),
        }
    }

    /// `ButtonListDialog`: how many of `main_actions` stand in the body, and in how many columns.
    pub fn button_list(&self) -> Option<(usize, i32)> {
        match self {
            Self::MultiAction {
                actions, columns, ..
            } => Some((actions.len(), *columns)),
            _ => None,
        }
    }

    /// `MultiActionDialog` from its fields.
    fn multi_action(common: Box<CommonData>, fields: &[Field]) -> Option<Self> {
        let Some(FieldValue::List(items)) = get(fields, "actions") else {
            return None;
        };
        // `ExtraCodecs.nonEmptyList`.
        if items.is_empty() {
            return None;
        }
        Some(Self::MultiAction {
            common,
            actions: items
                .iter()
                .map(ActionButton::from_nbt)
                .collect::<Option<_>>()?,
            exit: exit_action(fields)?,
            columns: columns(fields)?,
        })
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
            "multi_action" => Self::multi_action(common, fields)?,
            "server_links" => Self::server_links(common, fields)?,
            "dialog_list" => Self::dialog_list(common, fields)?,
            _ => Self::Other(common),
        })
    }
}

/// Which screen `DialogScreen.runAction` leaves in place once a button has been used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Next {
    /// The screen the dialog was opened over (`CLOSE`).
    Previous,
    /// The dialog itself (`NONE`).
    Same,
    /// The "waiting for a response" screen, over the previous one (`WAIT_FOR_RESPONSE`).
    Waiting,
}

/// What `DialogScreen.runAction` asks for: the screen to leave, and the click event to run.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    pub next: Next,
    pub event: Option<ClickEvent>,
}

/// `DialogScreen.runAction(closeAction, afterAction)` with what the inputs hold.
pub fn run_action(action: Option<&Action>, after: AfterAction, values: &[InputValue]) -> Outcome {
    Outcome {
        next: match after {
            AfterAction::Close => Next::Previous,
            AfterAction::None => Next::Same,
            AfterAction::WaitForResponse => Next::Waiting,
        },
        event: action.map(|action| action.create(values)),
    }
}

impl Action {
    /// `Action.createAction`.
    fn create(&self, values: &[InputValue]) -> ClickEvent {
        match self {
            Self::Static(event) => event.clone(),
            Self::CommandTemplate(template) => {
                let arguments = values
                    .iter()
                    .map(|value| (value.key.as_str(), value.substitution.as_str()))
                    .collect();
                ClickEvent::RunCommand(template.instantiate(&arguments))
            }
            Self::CustomAll { id, additions } => {
                let mut payload = additions.clone().unwrap_or_default();
                for value in values {
                    payload.retain(|field| field.key != value.key);
                    payload.push(Field {
                        key: value.key.clone(),
                        value: value.tag.clone(),
                    });
                }
                ClickEvent::Custom(id.clone(), Some(FieldValue::Compound(payload)))
            }
        }
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
            inputs: inputs(fields)?,
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
            "dynamic/run_command" => Some(Self::CommandTemplate(Template::parse(text(
                fields, "template",
            )?)?)),
            "dynamic/custom" => Some(Self::CustomAll {
                id: text(fields, "id")?.to_owned(),
                additions: match get(fields, "additions") {
                    None => None,
                    Some(FieldValue::Compound(additions)) => Some(additions.clone()),
                    Some(_) => return None,
                },
            }),
            _ => Some(Self::Static(ClickEvent::from_nbt_fields(fields, "type")?)),
        }
    }
}

pub(crate) fn get<'a>(fields: &'a [Field], key: &str) -> Option<&'a FieldValue> {
    fields
        .iter()
        .find(|field| field.key == key)
        .map(|field| &field.value)
}

pub(crate) fn text<'a>(fields: &'a [Field], key: &str) -> Option<&'a str> {
    match get(fields, key) {
        Some(FieldValue::Str(text)) => Some(text),
        _ => None,
    }
}

/// `inputs`: a list of inputs with different keys (`Input.CODEC.listOf()`).
fn inputs(fields: &[Field]) -> Option<Vec<Input>> {
    let items = match get(fields, "inputs") {
        None => return Some(Vec::new()),
        Some(FieldValue::List(items)) => items,
        Some(_) => return None,
    };
    items.iter().map(Input::from_nbt).collect()
}

pub(crate) fn strip_namespace(name: &str) -> &str {
    name.strip_prefix("minecraft:").unwrap_or(name)
}

/// `ExtraCodecs.POSITIVE_INT.optionalFieldOf("columns", 2)`.
/// The optional `exit_action` button: `Some(None)` for none, `None` for one that does not read.
pub(crate) fn exit_action(fields: &[Field]) -> Option<Option<Box<ActionButton>>> {
    match get(fields, "exit_action") {
        Some(value) => ActionButton::from_nbt(value).map(|button| Some(Box::new(button))),
        None => Some(None),
    }
}

pub(crate) fn columns(fields: &[Field]) -> Option<i32> {
    match get(fields, "columns") {
        None => Some(2),
        Some(FieldValue::Int(value)) if *value >= 1 => Some(*value),
        _ => None,
    }
}

/// An `intRange(1, max)` field with a default.
pub(crate) fn width(fields: &[Field], key: &str, default: i32, max: i32) -> Option<i32> {
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
    fn a_multi_action_dialog_lists_its_buttons_then_the_exit_button() {
        let button = |label: &str| FieldValue::Compound(vec![field("label", string(label))]);
        let dialog = |actions: Vec<FieldValue>| {
            FieldValue::Compound(vec![
                field("type", string("multi_action")),
                field("title", string("Pick")),
                field("actions", FieldValue::List(actions)),
                field("exit_action", button("Back")),
                field("columns", FieldValue::Int(3)),
            ])
        };
        let dialog_value =
            Dialog::from_nbt(&dialog(vec![button("A"), button("B")])).expect("multi");
        assert_eq!(dialog_value.button_list(), Some((2, 3)));
        assert_eq!(dialog_value.main_actions().len(), 3);
        assert_eq!(Dialog::from_nbt(&dialog(Vec::new())), None);
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

    #[test]
    fn a_boolean_input_and_the_dynamic_actions_read_and_fill_from_the_inputs() {
        let compound = FieldValue::Compound;
        let input = compound(vec![
            field("type", string("minecraft:boolean")),
            field("key", string("agree")),
            field("label", string("Agree")),
            field("initial", FieldValue::Byte(1)),
        ]);
        let notice = |action: FieldValue| {
            compound(vec![
                field("type", string("notice")),
                field("title", string("T")),
                field("inputs", FieldValue::List(vec![input.clone()])),
                field(
                    "action",
                    compound(vec![field("label", string("Go")), field("action", action)]),
                ),
            ])
        };
        let template = compound(vec![
            field("type", string("dynamic/run_command")),
            field("template", string("say $(agree)")),
        ]);
        let Some(Dialog::Notice { common, action }) = Dialog::from_nbt(&notice(template)) else {
            panic!("notice with a template");
        };
        let Control::Boolean(boolean) = &common.inputs[0].control else {
            panic!("boolean");
        };
        assert!(boolean.initial);
        assert_eq!(
            (boolean.on_true.as_str(), boolean.on_false.as_str()),
            ("true", "false")
        );
        let value = InputValue {
            key: "agree".to_owned(),
            substitution: "true".to_owned(),
            tag: FieldValue::Byte(1),
        };
        let outcome = run_action(
            action.action.as_ref(),
            AfterAction::Close,
            std::slice::from_ref(&value),
        );
        assert_eq!(
            outcome.event,
            Some(ClickEvent::RunCommand("say true".to_owned()))
        );

        let custom = compound(vec![
            field("type", string("dynamic/custom")),
            field("id", string("minecraft:answer")),
            field(
                "additions",
                compound(vec![field("extra", FieldValue::Int(7))]),
            ),
        ]);
        let Some(Dialog::Notice { action, .. }) = Dialog::from_nbt(&notice(custom)) else {
            panic!("notice with a custom action");
        };
        let outcome = run_action(action.action.as_ref(), AfterAction::Close, &[value]);
        let Some(ClickEvent::Custom(id, Some(FieldValue::Compound(payload)))) = outcome.event
        else {
            panic!("custom event");
        };
        assert_eq!(id, "minecraft:answer");
        assert_eq!(payload.len(), 2);
        assert_eq!(get(&payload, "extra"), Some(&FieldValue::Int(7)));
        assert_eq!(get(&payload, "agree"), Some(&FieldValue::Byte(1)));
    }
}
