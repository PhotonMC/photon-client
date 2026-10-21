use super::*;

use photon_game::chat_abilities::RestrictionAction;
use photon_gui::screen::restrictions::{Press, RestrictionsScreen};

/// What a press opens, as the reference's recording game names the screen.
fn opened(press: Press) -> &'static str {
    match press {
        Press::Action(RestrictionAction::GoToChatSettings) => "ChatOptionsScreen",
        Press::Action(RestrictionAction::GoToAccount) => "link",
        Press::Done => "PreviousScreen",
    }
}

/// One step of a row on the screen: the press it made, if any.
fn step(screen: &mut RestrictionsScreen, step: &Value) -> Result<Option<Press>, String> {
    let (x, y) = (
        step["x"].as_f64().unwrap_or(0.0),
        step["y"].as_f64().unwrap_or(0.0),
    );
    Ok(match step["do"].as_str().unwrap_or_default() {
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
            "escape" => Some(screen.escape()),
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
    })
}

pub(super) fn restrictions_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let lang = photon_game::text::Lang::load(&ctx.root);
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    let in_world = row["in_world"].as_bool().unwrap_or(false);
    let mut screen = RestrictionsScreen::new(
        &ctx.font,
        &lang,
        (int("width"), int("height")),
        &abilities(&row["restrictions"])?,
        in_world,
    );
    let mut events = Vec::new();
    for entry in row["steps"].as_array().ok_or("steps")? {
        if let Some(press) = step(&mut screen, entry)? {
            events.push(opened(press));
        }
    }
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    screen.draw(&mut frame, (&ctx.atlas, &ctx.font), (mouse, 5));
    Ok(json!({
        "label": row["label"],
        "events": events,
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

/// A dialog's screen or the restrictions screen.
pub(super) fn dialog_case(
    ctx: &mut GuiContext,
    row: &Value,
    operation: &str,
) -> Result<Value, String> {
    if operation == "dialog_screen" {
        dialog_screen_case(ctx, row)
    } else {
        restrictions_screen_case(ctx, row)
    }
}
