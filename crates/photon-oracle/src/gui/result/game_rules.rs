use super::*;

use photon_gui::screen::game_rules::{GameRulesEvent, GameRulesScreen};

/// `game_rules_screen`: the in-world screen after the answers, keys, clicks and wheel turns of the
/// row, drawn once; the events are what it sent and which screens it asked for.
pub(super) fn game_rules_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let lang = photon_game::text::Lang::load(&ctx.root);
    let mut screen = GameRulesScreen::new(&ctx.font, &lang, (int("width"), int("height")));
    let mut events = vec!["send:client_command:REQUEST_GAMERULE_VALUES".to_owned()];
    if let Some(values) = row["values"].as_array() {
        screen.values_received(&pairs(values), &ctx.font, &lang);
    }
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    let blurriness = row["blurriness"].as_i64().unwrap_or(5) as i32;
    let draw = |screen: &GameRulesScreen, frame: &mut GuiFrame| {
        screen.draw(
            frame,
            (&ctx.atlas, &ctx.font),
            mouse,
            (blurriness, |_: &mut GuiFrame| {}),
        );
    };
    for step in row["steps"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        if step["do"] == "permission" {
            // `onGamemasterPermissionChanged(false)` shows the screen before this one.
            if step["has"] == false {
                events.push("PreviousScreen".to_owned());
            }
            continue;
        }
        let event = run_step(&mut screen, step, (ctx, &lang))?;
        events.extend(describe(event));
        draw(&screen, &mut GuiFrame::default());
    }
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    draw(&screen, &mut frame);
    Ok(json!({
        "label": row["label"],
        "blur": frame.blur.is_some(),
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
        "events": events,
    }))
}

/// The rules of a row's `values`, as the server's answer carries them.
fn pairs(values: &[Value]) -> Vec<(String, String)> {
    values
        .iter()
        .map(|pair| {
            let text = |index: usize| pair[index].as_str().unwrap_or_default().to_owned();
            (text(0), text(1))
        })
        .collect()
}

/// What an answer of the screen did, as the reference's recording game writes it down.
fn describe(event: Option<GameRulesEvent>) -> Vec<String> {
    match event {
        None => Vec::new(),
        Some(GameRulesEvent::Close) => vec!["exit:discard".to_owned()],
        Some(GameRulesEvent::ConfirmDiscard) => vec!["ConfirmScreen".to_owned()],
        Some(GameRulesEvent::Apply(changes)) => {
            let mut events = Vec::new();
            if !changes.is_empty() {
                let list: String = changes
                    .iter()
                    .map(|(name, value)| format!(":{name}={value}"))
                    .collect();
                events.push(format!("send:set_game_rule{list}"));
            }
            events.push("exit:apply".to_owned());
            events
        }
    }
}

/// The keys a step's GLFW codes stand for.
const KEYS: [(i64, glfw::Key); 12] = [
    (32, glfw::Key::Space),
    (256, glfw::Key::Escape),
    (257, glfw::Key::Enter),
    (258, glfw::Key::Tab),
    (259, glfw::Key::Backspace),
    (261, glfw::Key::Delete),
    (262, glfw::Key::Right),
    (263, glfw::Key::Left),
    (264, glfw::Key::Down),
    (265, glfw::Key::Up),
    (268, glfw::Key::Home),
    (269, glfw::Key::End),
];

fn run_step(
    screen: &mut GameRulesScreen,
    step: &Value,
    (ctx, lang): (&GuiContext, &photon_game::text::Lang),
) -> Result<Option<GameRulesEvent>, String> {
    let at = (
        step["x"].as_f64().unwrap_or(0.0),
        step["y"].as_f64().unwrap_or(0.0),
    );
    Ok(match step["do"].as_str().ok_or("do")? {
        "click" => screen.clicked(at, &ctx.font),
        "release" => {
            screen.released();
            None
        }
        "drag" => {
            screen.dragged(at, step["dy"].as_f64().unwrap_or(0.0));
            None
        }
        "scroll" => {
            screen.scrolled(at, step["amount"].as_f64().unwrap_or(0.0));
            None
        }
        "type" => {
            screen.type_search(step["text"].as_str().unwrap_or_default(), &ctx.font);
            None
        }
        "resize" => {
            let size = |field: &str| step[field].as_i64().unwrap_or(0) as i32;
            screen.resize((size("width"), size("height")));
            None
        }
        "values" => {
            let values = step["values"].as_array().ok_or("values")?;
            screen.values_received(&pairs(values), &ctx.font, lang);
            None
        }
        "key" => {
            let code = step["key"].as_i64();
            let key = KEYS
                .iter()
                .find(|(known, _)| Some(*known) == code)
                .map(|&(_, key)| key)
                .ok_or_else(|| format!("a key: {code:?}"))?;
            let (control, shift) = (step["ctrl"].is_boolean(), step["shift"].is_boolean());
            screen.key_pressed(
                key,
                (control, shift),
                &ctx.font,
                &mut crate::gui::NoClipboard,
            )
        }
        other => return Err(format!("a step: {other}")),
    })
}
