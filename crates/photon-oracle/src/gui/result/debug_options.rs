use super::*;

use photon_gui::hud::debug::{EntryList, Profile, Status};
use photon_gui::screen::debug_options::DebugOptionsScreen;

/// `debug_options_screen`: the screen over an entry list the row's profile and statuses were
/// set on, after a search was typed and the list scrolled, drawn once.
pub(super) fn debug_options_screen_case(
    ctx: &mut GuiContext,
    row: &Value,
) -> Result<Value, String> {
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let lang = photon_game::text::Lang::load(&ctx.root);
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
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    screen.draw(&mut frame, &ctx.atlas, &ctx.font, mouse);
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}
