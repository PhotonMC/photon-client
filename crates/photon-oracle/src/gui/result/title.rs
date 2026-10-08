use super::*;

/// `title_screen`: the title screen with a splash, after some keys and frames, under a mouse.
pub(super) fn title_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_render::gui::widgets::navigation::Direction;
    let (font, atlas) = (&ctx.font, &ctx.atlas);
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let lang = photon_game::text::Lang::load(&ctx.root);
    let size = (int("width"), int("height"));
    let mut screen = photon_render::gui::screen::title::TitleScreen::new(
        font,
        &lang,
        size,
        row["splash"]
            .as_str()
            .filter(|_| row["hide_splash"].as_bool() != Some(true)),
    );
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    for step in row["steps"].as_array().map_or(&[][..], Vec::as_slice) {
        if step["do"] == "frame" {
            let mut thrown = GuiFrame {
                millis: step["ms"].as_u64().unwrap_or(0),
                ..GuiFrame::default()
            };
            screen.draw(&mut thrown, atlas, font, mouse);
            continue;
        }
        let buttons = screen.buttons();
        match step["key"].as_str().unwrap_or_default() {
            "tab" => buttons.tab(true),
            "shift+tab" => buttons.tab(false),
            "left" => buttons.arrow(Direction::Left),
            "right" => buttons.arrow(Direction::Right),
            "up" => buttons.arrow(Direction::Up),
            "down" => buttons.arrow(Direction::Down),
            other => return Err(format!("unknown key {other}")),
        }
    }
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        millis: row["ms"].as_u64().unwrap_or(0),
        ..GuiFrame::default()
    };
    screen.draw(&mut frame, atlas, font, mouse);
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

/// `join_multiplayer_screen`: the server list over the row's saved servers, after clicks and
/// scrolls, under a mouse.
pub(super) fn join_multiplayer_screen_case(
    ctx: &mut GuiContext,
    row: &Value,
) -> Result<Value, String> {
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let lang = photon_game::text::Lang::load(&ctx.root);
    let font = &ctx.font;
    let names: Vec<String> = row["servers"]
        .as_array()
        .ok_or("servers")?
        .iter()
        .map(|name| name.as_str().unwrap_or_default().to_owned())
        .collect();
    // The saved servers, all at the address the reference's oracle saves them under.
    let mut saved: Vec<photon_game::servers::ServerData> = names
        .iter()
        .map(|name| photon_game::servers::ServerData::new(name, "127.0.0.1:1"))
        .collect();
    let mut screen = photon_render::gui::screen::servers::JoinMultiplayerScreen::new(
        font,
        &lang,
        (int("width"), int("height")),
        names,
    );
    for (index, ping) in row["pings"].as_array().into_iter().flatten().enumerate() {
        use photon_game::server_status::{Pinged, ServerStatus};
        let pinged = if ping.is_null() {
            continue;
        } else if ping["unreachable"].as_bool() == Some(true) {
            Pinged::unreachable(ping["unknown_host"].as_bool() == Some(true))
        } else {
            let status = ServerStatus::parse(ping["json"].as_str().unwrap_or_default())
                .ok_or("a status answer")?;
            Pinged::answered(&status, ping["ping"].as_i64().unwrap_or(0))
        };
        screen.set_pinged(index, &pinged, font, &lang);
        if let (Some(favicon), Some(server)) = (&pinged.favicon, saved.get_mut(index)) {
            server.apply_ping_icon(favicon);
        }
    }
    // Each entry's check of its icon, as its draw makes it.
    let mut shown = Vec::new();
    for (index, server) in saved.iter_mut().enumerate() {
        let sync = photon_render::gui::favicon::sync_server_icon(
            &mut ctx.atlas,
            &mut ctx.atlas_image,
            server,
            &mut None,
        );
        if let Some(sync) = sync {
            screen.set_icon(index, sync.texture);
            shown.push(photon_render::gui::favicon::texture_name(&server.ip));
        }
    }
    let atlas = &ctx.atlas;
    for step in row["steps"].as_array().map_or(&[][..], Vec::as_slice) {
        let x = step["x"].as_f64().unwrap_or(0.0);
        let y = step["y"].as_f64().unwrap_or(0.0);
        match step["do"].as_str() {
            Some("click") => {
                screen.clicked((x as i32, y as i32), false);
            }
            Some("scroll") => screen.scrolled((x, y), step["dy"].as_f64().unwrap_or(0.0)),
            other => return Err(format!("unknown step {other:?}")),
        }
    }
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        millis: row["ms"].as_u64().unwrap_or(0),
        ..GuiFrame::default()
    };
    screen.draw(&mut frame, atlas, font, mouse);
    let ops = gui_ops(&frame.log.take().unwrap_or_default());
    // The icons go back, for the rows that follow share the atlas.
    for name in shown {
        ctx.atlas.set_favicon(&mut ctx.atlas_image, &name, None);
    }
    Ok(json!({
        "label": row["label"],
        "ops": ops,
    }))
}

/// `server_form`: the add, edit or direct form after text, keys and clicks.
pub(super) fn server_form_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_game::servers::PackStatus;
    use photon_render::gui::screen::server_form::{FormKind, ServerFormScreen};
    let (font, atlas) = (&ctx.font, &ctx.atlas);
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let text = |key: &str| row[key].as_str().unwrap_or_default();
    let lang = photon_game::text::Lang::load(&ctx.root);
    let kind = match text("kind") {
        "add" => FormKind::Add,
        "edit" => FormKind::Edit,
        _ => FormKind::Direct,
    };
    let pack = match text("pack") {
        "ENABLED" => PackStatus::Enabled,
        "DISABLED" => PackStatus::Disabled,
        _ => PackStatus::Prompt,
    };
    let mut screen = ServerFormScreen::new(
        font,
        &lang,
        kind,
        (int("width"), int("height")),
        (text("name"), text("ip"), pack),
    );
    for step in row["steps"].as_array().map_or(&[][..], Vec::as_slice) {
        match step["do"].as_str() {
            Some("type") => {
                for character in step["text"].as_str().unwrap_or_default().chars() {
                    screen.char_typed(font, character);
                }
            }
            Some("key") => {
                let name = step["key"].as_str().unwrap_or_default();
                let key = match name {
                    "tab" | "shift+tab" => glfw::Key::Tab,
                    "backspace" => glfw::Key::Backspace,
                    "enter" => glfw::Key::Enter,
                    "left" => glfw::Key::Left,
                    "home" => glfw::Key::Home,
                    "space" => glfw::Key::Space,
                    other => return Err(format!("unknown key {other}")),
                };
                screen.key_pressed(
                    (font, &lang),
                    key,
                    (false, name.starts_with("shift+")),
                    &mut crate::gui::NoClipboard,
                );
            }
            Some("click") => {
                let at = (
                    step["x"].as_f64().unwrap_or(0.0) as i32,
                    step["y"].as_f64().unwrap_or(0.0) as i32,
                );
                screen.clicked((font, &lang), at, false);
            }
            other => return Err(format!("unknown step {other:?}")),
        }
    }
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    screen.draw(&mut frame, atlas, font, mouse);
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

/// `connect_screen`: the connecting screen with a status, or the terrain loading screen.
pub(super) fn connect_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_render::gui::screen::connect::{ConnectScreen, draw_level_loading};
    let (font, atlas) = (&ctx.font, &ctx.atlas);
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let lang = photon_game::text::Lang::load(&ctx.root);
    let size = (int("width"), int("height"));
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    if row["screen"] == "loading" {
        draw_level_loading(&mut frame, atlas, font, &lang, size);
    } else {
        let mut screen = ConnectScreen::new(&lang, size);
        screen.set_status(&lang, row["status"].as_str().unwrap_or_default());
        screen.draw(&mut frame, atlas, font, mouse);
    }
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}
