use super::*;

use photon_game::dialog::{
    Action, ActionButton, AfterAction, Body, BooleanInput, CommonData, Control, Dialog,
    DialogContext, DialogRef, DialogSet, Input, InputValue, ListSource, Next, NumberRangeInput,
    OptionEntry, PlainMessage, RangeInfo, SingleOptionInput, Template, TextInput, run_action,
};
use photon_game::text::{ClickEvent, Component};
use photon_gui::screen::dialog::{DialogScreen, Press};
use photon_gui::screen::waiting::WaitingScreen;
use photon_gui::widgets::edit_box::{Clipboard, EditKey};
use photon_render::gui::font::Font;

fn text(value: &Value) -> Component {
    Component::literal(value.as_str().unwrap_or_default())
}

fn message(value: &Value) -> PlainMessage {
    PlainMessage {
        contents: text(&value["text"]),
        width: value["width"].as_i64().unwrap_or(200) as i32,
    }
}

fn button(value: &Value) -> ActionButton {
    ActionButton {
        label: text(&value["label"]),
        tooltip: value.get("tooltip").map(text),
        width: value["width"].as_i64().unwrap_or(150) as i32,
        action: value.get("action").map(|action| {
            let value = action["value"].as_str().unwrap_or_default();
            match action["type"].as_str() {
                Some("run_command") => Action::Static(ClickEvent::RunCommand(value.to_owned())),
                Some("template") => Action::CommandTemplate(
                    Template::parse(value).expect("a template with a variable"),
                ),
                Some("custom_all") => Action::CustomAll {
                    id: value.to_owned(),
                    additions: None,
                },
                _ => Action::Static(ClickEvent::Custom(value.to_owned(), None)),
            }
        }),
    }
}

fn input(value: &Value) -> Input {
    let flag = |key: &str| value[key].as_bool().unwrap_or(false);
    let string = |key: &str, default: &str| value[key].as_str().unwrap_or(default).to_owned();
    Input {
        key: string("key", ""),
        control: match value["type"].as_str() {
            Some("boolean") => Control::Boolean(BooleanInput {
                label: text(&value["label"]),
                initial: flag("initial"),
                on_true: string("on_true", "true"),
                on_false: string("on_false", "false"),
            }),
            Some("single_option") => Control::SingleOption(SingleOptionInput {
                width: value["width"].as_i64().unwrap_or(200) as i32,
                entries: value["options"].as_array().map_or_else(Vec::new, |items| {
                    items
                        .iter()
                        .map(|entry| OptionEntry {
                            id: entry["id"].as_str().unwrap_or_default().to_owned(),
                            display: entry.get("display").map(text),
                            initial: entry["initial"].as_bool().unwrap_or(false),
                        })
                        .collect()
                }),
                label: text(&value["label"]),
                label_visible: value["label_visible"].as_bool().unwrap_or(true),
            }),
            Some("number_range") => Control::NumberRange(NumberRangeInput {
                width: value["width"].as_i64().unwrap_or(200) as i32,
                label: text(&value["label"]),
                label_format: string("label_format", "options.generic_value"),
                range: RangeInfo {
                    start: value["start"].as_f64().unwrap_or(0.0) as f32,
                    end: value["end"].as_f64().unwrap_or(1.0) as f32,
                    initial: value["initial"].as_f64().map(|v| v as f32),
                    step: value["step"].as_f64().map(|v| v as f32),
                },
            }),
            Some("text") => Control::Text(TextInput {
                width: value["width"].as_i64().unwrap_or(200) as i32,
                label: text(&value["label"]),
                label_visible: value["label_visible"].as_bool().unwrap_or(true),
                initial: string("initial", ""),
                max_length: value["max_length"].as_i64().unwrap_or(32) as i32,
                multiline: None,
            }),
            other => panic!("unknown input {other:?}"),
        },
    }
}

fn body(value: &Value) -> Body {
    if value.get("item").is_none() {
        return Body::Plain(message(value));
    }
    Body::Item {
        id: "minecraft:diamond".to_owned(),
        count: 1,
        description: value.get("description").map(message),
        show_decorations: true,
        show_tooltip: false,
        width: value["width"].as_i64().unwrap_or(16) as i32,
        height: value["height"].as_i64().unwrap_or(16) as i32,
    }
}

fn dialog(value: &Value) -> Dialog {
    let after_action = match value["after"].as_str() {
        Some("NONE") => AfterAction::None,
        Some("WAIT_FOR_RESPONSE") => AfterAction::WaitForResponse,
        _ => AfterAction::Close,
    };
    let common = Box::new(CommonData {
        title: text(&value["title"]),
        external_title: None,
        can_close_with_escape: value["can_close"].as_bool().unwrap_or(true),
        pause: false,
        after_action,
        body: value["body"]
            .as_array()
            .map_or_else(Vec::new, |items| items.iter().map(body).collect()),
        inputs: value["inputs"]
            .as_array()
            .map_or_else(Vec::new, |items| items.iter().map(input).collect()),
    });
    let buttons: Vec<ActionButton> = value["buttons"]
        .as_array()
        .map_or_else(Vec::new, |items| items.iter().map(button).collect());
    if value["kind"] == "server_links" || value["kind"] == "dialog_list" {
        return listed(common, value);
    }
    if value["kind"] == "multi_action" {
        return Dialog::MultiAction {
            common,
            actions: buttons,
            exit: value.get("exit").map(|exit| Box::new(button(exit))),
            columns: value["columns"].as_i64().unwrap_or(2) as i32,
        };
    }
    let mut buttons = buttons.into_iter().map(Box::new);
    match (buttons.next(), buttons.next()) {
        (Some(yes), Some(no)) => Dialog::Confirmation { common, yes, no },
        (Some(action), None) => Dialog::Notice { common, action },
        _ => Dialog::Other(common),
    }
}

/// What the row's list dialogs take their buttons from: the links it lists. It has no registry.
struct RowContext(photon_game::server_links::ServerLinks);

impl DialogContext for RowContext {
    fn server_links(&self) -> &photon_game::server_links::ServerLinks {
        &self.0
    }

    fn tag_members(&self, _tag: &str) -> Vec<DialogRef> {
        Vec::new()
    }

    fn dialog(&self, reference: &DialogRef) -> Option<Dialog> {
        match reference {
            DialogRef::Inline(dialog) => Some((**dialog).clone()),
            DialogRef::Registry(_) => None,
        }
    }
}

/// The links a row lists.
fn row_links(value: &Value) -> photon_game::server_links::ServerLinks {
    use photon_game::server_links::{LinkType, ServerLinks, UntrustedEntry};
    let entries = value["links"].as_array().map_or_else(Vec::new, |links| {
        links
            .iter()
            .map(|link| UntrustedEntry {
                link_type: match link["known"].as_i64() {
                    Some(id) => LinkType::known(id as i32),
                    None => LinkType::Custom(text(&link["text"])),
                },
                link: link["link"].as_str().unwrap_or_default().to_owned(),
            })
            .collect()
    });
    ServerLinks::trusted(entries)
}

/// A `ServerLinksDialog` or a `DialogListDialog` of the sub-dialogs the row lists, as the screen
/// builds its buttons from them.
fn listed(common: Box<CommonData>, value: &Value) -> Dialog {
    let source = if value["kind"] == "server_links" {
        ListSource::ServerLinks
    } else {
        let subs = value["subs"].as_array().map_or_else(Vec::new, |subs| {
            subs.iter()
                .map(|sub| DialogRef::Inline(Box::new(sub_dialog(sub))))
                .collect()
        });
        ListSource::Dialogs(DialogSet::Entries(subs))
    };
    Dialog::List {
        common,
        source,
        exit: value.get("exit").map(|exit| Box::new(button(exit))),
        columns: value["columns"].as_i64().unwrap_or(2) as i32,
        button_width: value["button_width"].as_i64().unwrap_or(150) as i32,
    }
    .resolved(&RowContext(row_links(value)))
}

/// A dialog a list opens: a notice with a title and maybe a name for the button.
fn sub_dialog(value: &Value) -> Dialog {
    let common = Box::new(CommonData {
        title: text(&value["title"]),
        external_title: value.get("external").map(text),
        can_close_with_escape: true,
        pause: false,
        after_action: AfterAction::Close,
        body: Vec::new(),
        inputs: Vec::new(),
    });
    Dialog::Other(common)
}

/// The class the reference names the screen a press left, by the kind of dialog the row made.
fn screen_name(dialog: &Dialog, kind: &Value, next: Next) -> &'static str {
    match next {
        Next::Previous => "PreviousScreen",
        Next::Waiting => "WaitingForResponseScreen",
        Next::Same if kind == "dialog_list" => "DialogListDialogScreen",
        Next::Same if kind == "server_links" => "ServerLinksDialogScreen",
        Next::Same if dialog.button_list().is_some() => "MultiButtonDialogScreen",
        Next::Same => "SimpleDialogScreen",
    }
}

/// What the game's side of a press did, as the reference's recording gui and connection write it:
/// `Next` is the screen it was left on.
fn press_events(
    dialog: &Dialog,
    (kind, press, values): (&Value, Press, &[InputValue]),
    events: &mut Vec<String>,
) {
    let common = dialog.common();
    let (action, after) = match press {
        Press::Warning => {
            events.push("WarningScreen".to_owned());
            return;
        }
        Press::Close => (dialog.on_cancel(), AfterAction::Close),
        Press::Button(index) => (
            dialog.main_actions()[index].action.as_ref(),
            common.after_action,
        ),
    };
    let outcome = run_action(action, after, values);
    let screen = screen_name(dialog, kind, outcome.next);
    match outcome.event {
        Some(ClickEvent::RunCommand(command)) => {
            let command = command.strip_prefix('/').unwrap_or(&command);
            events.push(format!("run_command:{command}:{screen}"));
        }
        Some(ClickEvent::Custom(id, payload)) => events.push(custom_event(&id, payload.as_ref())),
        // These two leave the screen to what they open: the question about an address, or the
        // dialog.
        Some(ClickEvent::ShowDialog(_)) => {
            events.push(format!("open_dialog:{screen}"));
            return;
        }
        Some(ClickEvent::OpenUrl(_)) => {
            events.push("link".to_owned());
            return;
        }
        _ => {}
    }
    events.push(screen.to_owned());
}

/// `StringTag.quoteAndEscape`: the quote is the one the string does not have first.
fn quote_and_escape(input: &str) -> String {
    let mut body = String::new();
    let mut quote = None;
    for c in input.chars() {
        match c {
            '\\' => body.push_str("\\\\"),
            '"' | '\'' => {
                let mark = *quote.get_or_insert(if c == '"' { '\'' } else { '"' });
                if mark == c {
                    body.push('\\');
                }
                body.push(c);
            }
            c => {
                let escaped = photon_game::dialog::escape_without_quotes(&c.to_string());
                body.push_str(&escaped);
            }
        }
    }
    let quote = quote.unwrap_or('"');
    format!("{quote}{body}{quote}")
}

/// `custom:<id>`, and for a payload its entries in key order, each as the reference writes a tag.
fn custom_event(id: &str, payload: Option<&photon_core::nbt::FieldValue>) -> String {
    use photon_core::nbt::FieldValue;
    let Some(FieldValue::Compound(fields)) = payload else {
        return format!("custom:{id}");
    };
    let entries: std::collections::BTreeMap<&str, String> = fields
        .iter()
        .map(|field| {
            let tag = match &field.value {
                FieldValue::Byte(value) => format!("{value}b"),
                FieldValue::Str(value) => quote_and_escape(value),
                FieldValue::Float(value) => {
                    format!("{}f", photon_game::commands::reader::java_float(*value))
                }
                other => panic!("tag {other:?} is not written yet"),
            };
            (field.key.as_str(), tag)
        })
        .collect();
    let entries: Vec<String> = entries
        .iter()
        .map(|(key, tag)| format!("{key}={tag}"))
        .collect();
    format!("custom:{id}:{{{}}}", entries.join(", "))
}

/// The steps of a row for the waiting screen; leaving it is an event.
fn wait_steps(
    screen: &mut WaitingScreen,
    row: &Value,
    lang: &photon_game::text::Lang,
    events: &mut Vec<String>,
) -> Result<(), String> {
    for step in row["steps"].as_array().ok_or("steps")? {
        let closed = match step["do"].as_str().unwrap_or_default() {
            "tick" => {
                (0..step["count"].as_i64().unwrap_or(0)).for_each(|_| screen.tick(lang));
                false
            }
            "click" => screen.clicked((
                step["x"].as_f64().unwrap_or(0.0) as i32,
                step["y"].as_f64().unwrap_or(0.0) as i32,
            )),
            "key" => step["key"] == "escape" && screen.escape(),
            other => return Err(format!("unknown step {other}")),
        };
        if closed {
            events.push("PreviousScreen".to_owned());
        }
    }
    Ok(())
}

/// A click, with Shift down if the step says so.
fn click_step(
    screen: &mut DialogScreen,
    font: &Font,
    at: (i32, i32),
    shift: &Value,
) -> Option<Press> {
    screen.set_shift(shift.as_bool().unwrap_or(false));
    let press = screen.clicked(at, font);
    screen.set_shift(false);
    press
}

/// What a box does with the clipboard: nothing the reference can be asked about.
struct NoClip;

impl Clipboard for NoClip {
    fn get(&mut self) -> String {
        String::new()
    }

    fn set(&mut self, _: &str) {}
}

/// The edit key a key name stands for.
fn edit_of(base: &str) -> Option<EditKey> {
    Some(match base {
        "backspace" => EditKey::Backspace,
        "delete" => EditKey::Delete,
        "left" => EditKey::Left,
        "right" => EditKey::Right,
        "home" => EditKey::Home,
        "end" => EditKey::End,
        "a" => EditKey::A,
        _ => return None,
    })
}

/// A key the reference's screen was given, with the modifiers its name carries
/// (`shift+`, `ctrl+`): the focused box takes it first.
fn key_press(screen: &mut DialogScreen, font: &Font, key: &str) -> Result<Option<Press>, String> {
    let base = key.rsplit('+').next().unwrap_or(key);
    let modifiers = (key.contains("ctrl+"), key.contains("shift+"));
    if let Some(edit) = edit_of(base) {
        if screen.edit_key(font, edit, modifiers, &mut NoClip) {
            return Ok(None);
        }
        // A key the box would have edited with, and none has the focus: nothing happens.
        if base != "left" && base != "right" {
            return Ok(None);
        }
    }
    plain_key(screen, key)
}

/// A key the screen itself takes.
fn plain_key(screen: &mut DialogScreen, key: &str) -> Result<Option<Press>, String> {
    Ok(match key {
        "tab" | "shift+tab" => {
            screen.tab(key == "tab");
            None
        }
        "enter" | "space" => screen.pressed(),
        "shift+enter" => {
            screen.set_shift(true);
            let press = screen.pressed();
            screen.set_shift(false);
            press
        }
        "escape" => screen.escape(),
        "left" | "right" => {
            screen.arrow(key == "right");
            None
        }
        other => return Err(format!("unknown key {other}")),
    })
}

/// A step that is the keyboard or the mouse; what it pressed.
fn input_step(
    screen: &mut DialogScreen,
    font: &Font,
    step: &Value,
) -> Result<Option<Press>, String> {
    let (x, y) = (
        step["x"].as_f64().unwrap_or(0.0),
        step["y"].as_f64().unwrap_or(0.0),
    );
    Ok(match step["do"].as_str().unwrap_or_default() {
        "key" => key_press(screen, font, step["key"].as_str().unwrap_or_default())?,
        "type" => {
            for character in step["text"].as_str().unwrap_or_default().chars() {
                screen.char_typed(font, character);
            }
            None
        }
        "click" => click_step(screen, font, (x as i32, y as i32), &step["shift"]),
        "drag" => {
            screen.dragged(x, y, step["dy"].as_f64().unwrap_or(0.0));
            None
        }
        "release" => {
            screen.released();
            None
        }
        "scroll" => {
            screen.scrolled((x as i32, y as i32), step["amount"].as_f64().unwrap_or(0.0));
            None
        }
        other => return Err(format!("unknown step {other}")),
    })
}

pub(super) fn dialog_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    let lang = photon_game::text::Lang::load(root);
    let size = (int("width"), int("height"));
    let in_world = row["in_world"].as_bool().unwrap_or(false);
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    let blurriness = row["blurriness"].as_i64().unwrap_or(5) as i32;
    let mut events: Vec<String> = Vec::new();
    // The reference records no items: the screen's draw asks for none.
    let mut no_items = no_items;
    let shown = &row["dialog"];
    if shown["kind"] == "waiting" {
        let mut screen = WaitingScreen::new(font, &lang, size, in_world);
        wait_steps(&mut screen, row, &lang, &mut events)?;
        screen.draw(&mut frame, (atlas, font), mouse, blurriness);
    } else {
        let dialog = dialog(shown);
        let mut screen =
            DialogScreen::new(font, &lang, size, &dialog, in_world).ok_or("dialog screen")?;
        for step in row["steps"].as_array().ok_or("steps")? {
            let press = if step["do"] == "frame" {
                // A frame drawn and thrown away, at a time.
                let mut thrown = GuiFrame {
                    millis: step["ms"].as_u64().unwrap_or(0),
                    ..GuiFrame::default()
                };
                screen.draw(
                    &mut thrown,
                    (atlas, font),
                    &mut no_items,
                    (mouse, blurriness),
                );
                None
            } else {
                input_step(&mut screen, font, step)?
            };
            if let Some(press) = press {
                press_events(
                    &dialog,
                    (&shown["kind"], press, &screen.values()),
                    &mut events,
                );
            }
        }
        frame.millis = row["ms"].as_u64().unwrap_or(0);
        screen.draw(
            &mut frame,
            (atlas, font),
            &mut no_items,
            (mouse, blurriness),
        );
    }
    Ok(json!({
        "label": row["label"],
        "events": events,
        "blur": frame.blur.is_some(),
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}
