use super::*;

use photon_gui::hud::debug::{EntryList, Profile, Status, profile_text};
use photon_gui::screen::debug_options::{DebugOptionsEvent, DebugOptionsScreen};

/// `debug_options_screen`: the screen over an entry list the row's profile and statuses were
/// set on, after a search was typed and the list scrolled, drawn once.
pub(super) fn debug_options_screen_case(
    ctx: &mut GuiContext,
    row: &Value,
) -> Result<Value, String> {
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let lang = photon_game::text::Lang::load(&ctx.root);
    let mut entries = start_entries(row)?;
    let mut screen = DebugOptionsScreen::new(
        &ctx.font,
        &lang,
        (int("width"), int("height")),
        &entries,
        row["reduced"].as_bool().unwrap_or(false),
    );
    screen.type_search(row["search"].as_str().unwrap_or_default(), &ctx.font);
    let size = screen.size();
    if let Some(dy) = row["scroll"].as_f64() {
        screen.scrolled((f64::from(size.0) / 2.0, f64::from(size.1) / 2.0), dy);
    }
    let blurriness = row["blurriness"].as_i64().unwrap_or(5) as i32;
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    let mut asked = Asked::default();
    for step in row["steps"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        asked.record(run_step(&mut screen, step, ctx)?, &mut entries);
        screen.draw(
            &mut GuiFrame::default(),
            (&ctx.atlas, &ctx.font),
            mouse,
            (blurriness, |_: &mut GuiFrame| {}),
        );
    }
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    screen.draw(
        &mut frame,
        (&ctx.atlas, &ctx.font),
        mouse,
        (blurriness, |_: &mut GuiFrame| {}),
    );
    let mut out = json!({
        "label": row["label"],
        "blur": frame.blur.is_some(),
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
        "events": asked.events,
    });
    if asked.saved {
        let text = profile_text(&entries, photon_game::options::DATA_VERSION);
        out["file"] = serde_json::from_str(&text).map_err(|error| error.to_string())?;
    }
    Ok(out)
}

/// The entry list a row starts from: its profile, then its statuses.
fn start_entries(row: &Value) -> Result<EntryList, String> {
    let mut entries = EntryList::default();
    if let Some(name) = row["profile"].as_str() {
        let profile = Profile::from_name(&name.to_lowercase()).ok_or("profile")?;
        entries.load_profile(profile);
    }
    for setting in row["set"].as_array().map(Vec::as_slice).unwrap_or_default() {
        let status = match setting[1].as_str() {
            Some("ALWAYS_ON") => Status::AlwaysOn,
            Some("IN_OVERLAY") => Status::InOverlay,
            Some("NEVER") => Status::Never,
            other => return Err(format!("a status: {other:?}")),
        };
        entries.set_status(setting[0].as_str().ok_or("id")?, status);
    }
    Ok(entries)
}

/// What the screen asked of the game over the steps: whether the entry list saved, and the screens
/// it was told to show.
#[derive(Default)]
struct Asked {
    saved: bool,
    events: Vec<&'static str>,
}

impl Asked {
    /// One answer of the screen, done to the entry list as the game does.
    fn record(&mut self, event: Option<DebugOptionsEvent>, entries: &mut EntryList) {
        match event {
            Some(DebugOptionsEvent::Close) => self.events.push("close"),
            Some(DebugOptionsEvent::Status { id, status }) => {
                entries.set_status(id, status);
                self.saved = true;
            }
            Some(DebugOptionsEvent::Profile(profile)) => {
                entries.load_profile(profile);
                self.saved = true;
            }
            None => {}
        }
    }
}

/// One step of a row, and what the screen answered.
fn run_step(
    screen: &mut DebugOptionsScreen,
    step: &Value,
    ctx: &GuiContext,
) -> Result<Option<DebugOptionsEvent>, String> {
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
        "type" => {
            screen.type_search(step["text"].as_str().unwrap_or_default(), &ctx.font);
            None
        }
        "key" => key_step(screen, step, ctx)?,
        "resize" => {
            let size = |field: &str| step[field].as_i64().unwrap_or(0) as i32;
            screen.resize((size("width"), size("height")));
            None
        }
        other => return Err(format!("a step: {other}")),
    })
}

/// The keys a step's GLFW codes stand for.
const KEYS: [(i64, glfw::Key); 13] = [
    (32, glfw::Key::Space),
    (65, glfw::Key::A),
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

/// A `key` step: the key by its code, with Control and Shift held when the step says so.
fn key_step(
    screen: &mut DebugOptionsScreen,
    step: &Value,
    ctx: &GuiContext,
) -> Result<Option<DebugOptionsEvent>, String> {
    let code = step["key"].as_i64();
    let key = KEYS
        .iter()
        .find(|(known, _)| Some(*known) == code)
        .map(|&(_, key)| key)
        .ok_or_else(|| format!("a key: {code:?}"))?;
    let (control, shift) = (step["ctrl"].is_boolean(), step["shift"].is_boolean());
    Ok(screen.key_pressed(
        key,
        (control, shift),
        &ctx.font,
        &mut crate::gui::NoClipboard,
    ))
}
