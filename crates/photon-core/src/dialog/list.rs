//! The button list dialogs: `ServerLinksDialog` and `DialogListDialog`, whose buttons come from
//! the session when the screen is made.

use crate::nbt::{Field, FieldValue};

use super::{
    Action, ActionButton, AfterAction, CommonData, Dialog, DialogRef, columns, exit_action, get,
    width,
};
use crate::server_links::ServerLinks;
use crate::text::{ClickEvent, Component};

/// The `dialogs` field of a `DialogListDialog` (`HolderSet<Dialog>`): a tag, or the entries
/// named or written out.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogSet {
    Tag(String),
    Entries(Vec<DialogRef>),
}

/// What a button list takes its buttons from.
#[derive(Debug, Clone, PartialEq)]
pub enum ListSource {
    ServerLinks,
    Dialogs(DialogSet),
}

/// What a button list asks of the session to make its buttons.
pub trait DialogContext {
    fn server_links(&self) -> &ServerLinks;
    /// The members of a dialog tag (`#minecraft:pause_screen_additions`), as references.
    fn tag_members(&self, tag: &str) -> Vec<DialogRef>;
    /// A dialog by reference: written out, or from the registry.
    fn dialog(&self, reference: &DialogRef) -> Option<Dialog>;
}

impl Dialog {
    /// `createListActions` of `ServerLinksDialogScreen` and `DialogListDialogScreen`: a link
    /// button opens its address, a dialog button opens the dialog under its external title. Any
    /// other dialog stays as it is.
    pub fn resolved(self, context: &dyn DialogContext) -> Self {
        let Self::List {
            common,
            source,
            exit,
            columns,
            button_width,
        } = self
        else {
            return self;
        };
        let button = |label: Component, event: ClickEvent| ActionButton {
            label,
            tooltip: None,
            width: button_width,
            action: Some(Action::Static(event)),
        };
        let actions = match source {
            ListSource::ServerLinks => context
                .server_links()
                .entries()
                .iter()
                .map(|entry| {
                    let event = ClickEvent::OpenUrl(entry.link.clone());
                    button(entry.link_type.display_name(), event)
                })
                .collect(),
            ListSource::Dialogs(set) => {
                let references = match set {
                    DialogSet::Tag(tag) => context.tag_members(&tag),
                    DialogSet::Entries(entries) => entries,
                };
                references
                    .into_iter()
                    .filter_map(|reference| {
                        let common = context.dialog(&reference)?.common().clone();
                        let label = common.external_title.unwrap_or(common.title);
                        Some(button(label, ClickEvent::ShowDialog(reference)))
                    })
                    .collect()
            }
        };
        Self::MultiAction {
            common,
            actions,
            exit,
            columns,
        }
    }

    /// `ServerLinksDialog` from its fields.
    pub(super) fn server_links(common: Box<CommonData>, fields: &[Field]) -> Option<Self> {
        Self::list(common, ListSource::ServerLinks, fields)
    }

    /// `DialogListDialog` from its fields.
    pub(super) fn dialog_list(common: Box<CommonData>, fields: &[Field]) -> Option<Self> {
        let source = ListSource::Dialogs(dialog_set(get(fields, "dialogs")?)?);
        Self::list(common, source, fields)
    }

    fn list(common: Box<CommonData>, source: ListSource, fields: &[Field]) -> Option<Self> {
        Some(Self::List {
            common,
            source,
            exit: exit_action(fields)?,
            columns: columns(fields)?,
            button_width: width(fields, "button_width", 150, 1024)?,
        })
    }

    /// The dialog a registry entry of the game's own data pack stands for, when the server left
    /// its data out because the client knows the pack (`Dialogs.bootstrap`): the server links
    /// dialog, the list of the pause menu's additions and the quick actions list.
    pub fn built_in(name: &str) -> Option<Self> {
        let (title, source) = match name {
            "minecraft:server_links" => ("server_links", ListSource::ServerLinks),
            "minecraft:custom_options" => (
                "custom_options",
                ListSource::Dialogs(DialogSet::Tag(
                    "minecraft:pause_screen_additions".to_owned(),
                )),
            ),
            "minecraft:quick_actions" => (
                "quick_actions",
                ListSource::Dialogs(DialogSet::Tag("minecraft:quick_actions".to_owned())),
            ),
            _ => return None,
        };
        Some(Self::List {
            common: Box::new(CommonData {
                title: Component::translatable(format!("menu.{title}.title")),
                external_title: Some(Component::translatable(format!("menu.{title}"))),
                can_close_with_escape: true,
                pause: true,
                after_action: AfterAction::Close,
                body: Vec::new(),
                inputs: Vec::new(),
            }),
            source,
            exit: Some(Box::new(ActionButton {
                label: Component::translatable("gui.back"),
                tooltip: None,
                width: 200,
                action: None,
            })),
            columns: 1,
            button_width: 310,
        })
    }
}

/// `Dialog.LIST_CODEC`: `#tag`, one entry, or a list of them (a name or a dialog written out).
fn dialog_set(value: &FieldValue) -> Option<DialogSet> {
    match value {
        FieldValue::Str(tag) if tag.starts_with('#') => Some(DialogSet::Tag(tag[1..].to_owned())),
        FieldValue::List(items) => items
            .iter()
            .map(DialogRef::from_nbt)
            .collect::<Option<_>>()
            .map(DialogSet::Entries),
        single => DialogRef::from_nbt(single).map(|entry| DialogSet::Entries(vec![entry])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Registry;

    impl DialogContext for Registry {
        fn server_links(&self) -> &ServerLinks {
            static EMPTY: std::sync::OnceLock<ServerLinks> = std::sync::OnceLock::new();
            EMPTY.get_or_init(ServerLinks::default)
        }

        fn tag_members(&self, tag: &str) -> Vec<DialogRef> {
            assert_eq!(tag, "minecraft:extras");
            vec![
                DialogRef::Registry("a:one".to_owned()),
                DialogRef::Registry("a:gone".to_owned()),
            ]
        }

        fn dialog(&self, reference: &DialogRef) -> Option<Dialog> {
            let DialogRef::Registry(name) = reference else {
                return None;
            };
            (name == "a:one").then(|| Dialog::built_in("minecraft:server_links").expect("built in"))
        }
    }

    #[test]
    fn quick_actions_is_a_built_in_list_of_its_tag() {
        let Some(Dialog::List { source, .. }) = Dialog::built_in("minecraft:quick_actions") else {
            panic!("a list");
        };
        assert!(matches!(
            source,
            ListSource::Dialogs(DialogSet::Tag(tag)) if tag == "minecraft:quick_actions"
        ));
    }

    #[test]
    fn a_tag_becomes_one_button_per_dialog_that_resolves() {
        let list = Dialog::built_in("minecraft:custom_options").expect("built in");
        let Dialog::List {
            common,
            exit,
            columns,
            button_width,
            ..
        } = list
        else {
            panic!("a list");
        };
        let tagged = Dialog::List {
            common,
            source: ListSource::Dialogs(DialogSet::Tag("minecraft:extras".to_owned())),
            exit,
            columns,
            button_width,
        };
        let Dialog::MultiAction {
            actions, columns, ..
        } = tagged.resolved(&Registry)
        else {
            panic!("a multi action");
        };
        assert_eq!(columns, 1);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].width, 310);
        assert_eq!(
            actions[0].label,
            Component::translatable("menu.server_links")
        );
        assert_eq!(
            actions[0].action,
            Some(Action::Static(ClickEvent::ShowDialog(DialogRef::Registry(
                "a:one".to_owned()
            ))))
        );
    }

    #[test]
    fn the_dialogs_field_reads_a_tag_an_entry_or_a_list() {
        let name = |text: &str| FieldValue::Str(text.to_owned());
        assert_eq!(
            dialog_set(&name("#a:b")),
            Some(DialogSet::Tag("a:b".to_owned()))
        );
        let one = Some(DialogSet::Entries(vec![DialogRef::Registry(
            "a:b".to_owned(),
        )]));
        assert_eq!(dialog_set(&name("a:b")), one);
        assert_eq!(dialog_set(&FieldValue::List(vec![name("a:b")])), one);
    }
}
