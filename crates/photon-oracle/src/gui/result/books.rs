use super::*;

/// A key a step names (`ctrl+shift+left`): the key, Control, Shift.
fn step_key(name: &str) -> Result<(glfw::Key, bool, bool), String> {
    let control = name.starts_with("ctrl+");
    let name = name.trim_start_matches("ctrl+");
    let shift = name.starts_with("shift+");
    let key = match name.trim_start_matches("shift+") {
        "tab" => glfw::Key::Tab,
        "backspace" => glfw::Key::Backspace,
        "delete" => glfw::Key::Delete,
        "enter" => glfw::Key::Enter,
        "space" => glfw::Key::Space,
        "left" => glfw::Key::Left,
        "right" => glfw::Key::Right,
        "up" => glfw::Key::Up,
        "down" => glfw::Key::Down,
        "home" => glfw::Key::Home,
        "end" => glfw::Key::End,
        "pageup" => glfw::Key::PageUp,
        "pagedown" => glfw::Key::PageDown,
        "a" => glfw::Key::A,
        other => return Err(format!("unknown key {other}")),
    };
    Ok((key, control, shift))
}

/// `book_edit_screen`: the book and quill's screen after text, keys and clicks, drawn at a time
/// after it opened, its pages and the page it is on.
pub(super) fn book_edit_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_render::gui::screen::book_edit::BookEditScreen;
    let (font, atlas) = (&ctx.font, &ctx.atlas);
    let lang = photon_game::text::Lang::load(&ctx.root);
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let pages = row["pages"]
        .as_array()
        .ok_or("pages")?
        .iter()
        .map(|page| page.as_str().unwrap_or_default().to_owned())
        .collect();
    let mut screen =
        BookEditScreen::new((font, &lang), pages, (int("width"), int("height")), Some(0));
    for step in row["steps"].as_array().map_or(&[][..], Vec::as_slice) {
        match step["do"].as_str() {
            Some("type") => {
                for character in step["text"].as_str().unwrap_or_default().chars() {
                    screen.char_typed(font, character);
                }
            }
            Some("key") => {
                let (key, control, shift) = step_key(step["key"].as_str().unwrap_or_default())?;
                screen.key_pressed(
                    (font, &lang),
                    key,
                    (control, shift),
                    &mut crate::gui::NoClipboard,
                );
            }
            Some("click") => {
                let at = (
                    step["x"].as_f64().unwrap_or(0.0),
                    step["y"].as_f64().unwrap_or(0.0),
                );
                let shift = step["shift"].as_bool() == Some(true);
                screen.clicked((font, &lang), at, (shift, false), Some(0));
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
        millis: row["ms"].as_u64().unwrap_or(0),
        ..GuiFrame::default()
    };
    screen.draw(&mut frame, atlas, font, mouse);
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
        "pages": screen.pages(),
        "page": screen.current(),
    }))
}

/// `book_sign_screen`: the signing screen with a title kept from before, shown for the first time
/// or again, after text, keys and clicks, and the title as it stands.
pub(super) fn book_sign_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_render::gui::screen::book_sign::BookSignScreen;
    let (font, atlas) = (&ctx.font, &ctx.atlas);
    let lang = photon_game::text::Lang::load(&ctx.root);
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut screen = BookSignScreen::new(
        (font, &lang),
        row["title"].as_str().unwrap_or_default(),
        "Steve",
        (int("width"), int("height")),
    );
    for step in row["steps"].as_array().map_or(&[][..], Vec::as_slice) {
        match step["do"].as_str() {
            Some("type") => {
                for character in step["text"].as_str().unwrap_or_default().chars() {
                    screen.char_typed(font, character);
                }
            }
            Some("key") => {
                let (key, control, shift) = step_key(step["key"].as_str().unwrap_or_default())?;
                screen.key_pressed(font, key, (control, shift), &mut crate::gui::NoClipboard);
            }
            Some("click") => {
                let at = (
                    step["x"].as_f64().unwrap_or(0.0) as i32,
                    step["y"].as_f64().unwrap_or(0.0) as i32,
                );
                screen.clicked(font, at);
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
        "title": screen.title(),
    }))
}

/// `book_view`: a written book the server opened in the hand, after clicks and keys, and the
/// page open.
pub(super) fn book_view_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_render::gui::screen::Screen;
    use photon_render::gui::screen::book::BookScreen;
    let font = std::sync::Arc::new(Font::load(&ctx.root).ok_or("font")?);
    let lang = std::sync::Arc::new(photon_game::text::Lang::load(&ctx.root));
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut pages = Vec::new();
    for page in row["pages"].as_array().ok_or("pages")? {
        let json = photon_core::json::parse(&page.to_string()).map_err(|_| "json")?;
        pages.push(photon_game::text::Component::from_json(&json).ok_or("component")?);
    }
    let mut screen = BookScreen::held(
        (int("width"), int("height")),
        lang.clone(),
        font.clone(),
        pages,
    );
    let menu = photon_game::menus::container::ContainerMenu::default();
    for step in row["steps"].as_array().map_or(&[][..], Vec::as_slice) {
        match step["do"].as_str() {
            Some("key") => {
                let (key, ..) = step_key(step["key"].as_str().unwrap_or_default())?;
                screen.key_pressed(
                    key,
                    photon_render::gui::screen::KeyMods::default(),
                    &menu,
                    &mut crate::gui::NoClipboard,
                );
            }
            Some("click") => {
                let at = (
                    step["x"].as_f64().unwrap_or(0.0),
                    step["y"].as_f64().unwrap_or(0.0),
                );
                screen.mouse_clicked(at, 0, false, std::time::Instant::now(), &menu);
            }
            other => return Err(format!("unknown step {other:?}")),
        }
    }
    let mouse = (
        f64::from(row["mouse"][0].as_i64().unwrap_or(0) as i32),
        f64::from(row["mouse"][1].as_i64().unwrap_or(0) as i32),
    );
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    let mut item = |_: &photon_world::ClientItemStackState,
                    _: f32,
                    _: f32,
                    _: &mut Vec<photon_render::gui::GuiItemVertex>| {};
    let mut player = |_: (i32, i32, i32, i32), _: f32, _: f32, _: (f32, f32)| {
        Vec::<photon_render::gui::GuiItemVertex>::new()
    };
    let mut entity = |_: EntityId, _: (i32, i32, i32, i32), _: f32, _: f32, _: (f32, f32)| {
        Vec::<photon_render::gui::GuiItemVertex>::new()
    };
    let mut tooltip = |_: &photon_world::ClientItemStackState, _: bool| Vec::new();
    let (book, tags) = (
        photon_world::RecipeBook::default(),
        photon_world::block::tags::ItemTags::default(),
    );
    let mut draw = photon_render::gui::screen::ScreenDraw {
        frame: &mut frame,
        atlas: &ctx.atlas,
        font: &font,
        item: &mut item,
        player: &mut player,
        entity: &mut entity,
        tooltip: &mut tooltip,
        lang: &lang,
        book: &book,
        tags: &tags,
        control_held: false,
        partial: 0.0,
        widget_tooltips: true,
    };
    screen.build(&mut draw, &menu, mouse);
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
        "page": screen.held_page(),
    }))
}
