use super::*;

use photon_game::dialog::{
    Action, ActionButton, AfterAction, Body, CommonData, Dialog, Next, PlainMessage, run_action,
};
use photon_game::text::{ClickEvent, Component};
use photon_gui::screen::dialog::{DialogScreen, Press};
use photon_gui::screen::waiting::WaitingScreen;

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
            Action::Static(match action["type"].as_str() {
                Some("run_command") => ClickEvent::RunCommand(value.to_owned()),
                _ => ClickEvent::Custom(value.to_owned(), None),
            })
        }),
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
        has_inputs: false,
    });
    let buttons: Vec<ActionButton> = value["buttons"]
        .as_array()
        .map_or_else(Vec::new, |items| items.iter().map(button).collect());
    if value["kind"] == "server_links" {
        return linked(common, value);
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

/// A `ServerLinksDialog` with the links the row lists, as the screen builds its buttons from them.
fn linked(common: Box<CommonData>, value: &Value) -> Dialog {
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
    Dialog::ServerLinks {
        common,
        exit: value.get("exit").map(|exit| Box::new(button(exit))),
        columns: value["columns"].as_i64().unwrap_or(2) as i32,
        button_width: value["button_width"].as_i64().unwrap_or(150) as i32,
    }
    .resolved(&ServerLinks::trusted(entries))
}

/// What the game's side of a press did, as the reference's recording gui and connection write it:
/// `Next` is the screen it was left on.
fn press_events(dialog: &Dialog, press: Press, events: &mut Vec<String>) {
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
    let outcome = run_action(action, after);
    let screen = match outcome.next {
        Next::Previous => "PreviousScreen",
        Next::Same if dialog.button_list().is_some() => "MultiButtonDialogScreen",
        Next::Same => "SimpleDialogScreen",
        Next::Waiting => "WaitingForResponseScreen",
    };
    match outcome.event {
        Some(ClickEvent::RunCommand(command)) => {
            let command = command.strip_prefix('/').unwrap_or(&command);
            events.push(format!("run_command:{command}:{screen}"));
        }
        Some(ClickEvent::Custom(id, _)) => events.push(format!("custom:{id}")),
        // `clickUrlAction` asks first and leaves the screen to the question.
        Some(ClickEvent::OpenUrl(_)) => {
            events.push("link".to_owned());
            return;
        }
        _ => {}
    }
    events.push(screen.to_owned());
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
            let (x, y) = (
                step["x"].as_f64().unwrap_or(0.0),
                step["y"].as_f64().unwrap_or(0.0),
            );
            let press = match step["do"].as_str().unwrap_or_default() {
                "frame" => {
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
                }
                "key" => match step["key"].as_str().unwrap_or_default() {
                    "tab" => {
                        screen.tab(true);
                        None
                    }
                    "shift+tab" => {
                        screen.tab(false);
                        None
                    }
                    "enter" | "space" => screen.pressed(),
                    "escape" => screen.escape(),
                    other => return Err(format!("unknown key {other}")),
                },
                "click" => screen.clicked((x as i32, y as i32)),
                "drag" => {
                    screen.dragged(y, step["dy"].as_f64().unwrap_or(0.0));
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
            };
            if let Some(press) = press {
                press_events(&dialog, press, &mut events);
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
