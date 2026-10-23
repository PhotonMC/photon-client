use super::*;

pub(super) fn pause_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    use photon_gui::widgets::navigation::Direction;
    let lang = photon_game::text::Lang::load(root);
    let size = (int("width"), int("height"));
    let blurriness = row["blurriness"].as_i64().unwrap_or(5) as i32;
    let mut screen = photon_gui::screen::pause::PauseScreen::new(font, &lang, size, None);
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    for step in row["steps"].as_array().map_or(&[][..], Vec::as_slice) {
        if step["do"] == "click" {
            let at = (
                step["x"].as_i64().unwrap_or(0) as i32,
                step["y"].as_i64().unwrap_or(0) as i32,
            );
            screen.clicked(at);
        } else if step["do"] == "frame" {
            // A frame drawn and thrown away, at a time.
            let mut thrown = GuiFrame {
                millis: step["ms"].as_u64().unwrap_or(0),
                ..GuiFrame::default()
            };
            screen.draw(&mut thrown, atlas, font, mouse, size, Some(blurriness));
        } else {
            match step["key"].as_str().unwrap_or_default() {
                "tab" => screen.tab(true),
                "shift+tab" => screen.tab(false),
                "left" => screen.arrow(Direction::Left),
                "right" => screen.arrow(Direction::Right),
                "up" => screen.arrow(Direction::Up),
                "down" => screen.arrow(Direction::Down),
                other => return Err(format!("unknown key {other}")),
            }
        }
    }
    frame.millis = row["ms"].as_u64().unwrap_or(0);
    screen.draw(&mut frame, atlas, font, mouse, size, Some(blurriness));

    Ok(json!({
        "label": row["label"],
        "blur": frame.blur.is_some(),
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

/// The screen a row describes: a link, a server pack or a plain message.
fn confirm_screen(
    font: &photon_render::gui::font::Font,
    lang: &photon_game::text::Lang,
    size: (i32, i32),
    row: &Value,
    background: photon_gui::screen::confirm::ConfirmBackground,
) -> photon_gui::screen::confirm::ConfirmScreen {
    use photon_gui::screen::confirm::ConfirmScreen;
    let text =
        |key: &str| photon_game::text::Component::literal(row[key].as_str().unwrap_or_default());
    if let Some(command) = row["command"].as_object() {
        let field = |key: &str| command[key].as_str().unwrap_or_default();
        return ConfirmScreen::command(
            font,
            lang,
            size,
            (field("key"), field("command"), field("accept")),
            command["back"].as_bool().unwrap_or(false),
        )
        .with_background(background);
    }
    match (row["link"].as_object(), row["pack"].as_object()) {
        (Some(link), _) => ConfirmScreen::link(
            font,
            lang,
            size,
            link["url"].as_str().unwrap_or_default(),
            link["trusted"].as_bool().unwrap_or(false),
        ),
        (None, Some(pack)) => ConfirmScreen::server_pack(
            font,
            lang,
            size,
            pack.get("required")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            pack.get("prompt")
                .and_then(Value::as_str)
                .map(photon_game::text::Component::literal)
                .as_ref(),
        ),
        (None, None) => ConfirmScreen::new(
            font,
            lang,
            size,
            &text("title"),
            &text("message"),
            &text("yes"),
            &text("no"),
        ),
    }
    .with_background(background)
}

pub(super) fn confirm_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    use photon_gui::screen::confirm::ConfirmBackground;
    let lang = photon_game::text::Lang::load(root);
    let size = (int("width"), int("height"));
    let background = if row["death"].as_bool().unwrap_or(false) {
        ConfirmBackground::Death
    } else {
        ConfirmBackground::Menu {
            in_world: row["in_world"].as_bool().unwrap_or(false),
        }
    };
    let mut screen = confirm_screen(font, &lang, size, row, background);
    if int("delay") > 0 {
        screen.set_delay(int("delay"));
    }
    for _ in 0..int("ticks") {
        screen.tick();
    }
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    let mut callbacks = Vec::new();
    photon_gui::ui_sound::take();
    for key in row["keys"].as_array().ok_or("keys")? {
        let action = match key.as_str().unwrap_or_default() {
            "tab" => {
                screen.tab(true);
                None
            }
            "shift+tab" => {
                screen.tab(false);
                None
            }
            "escape" => screen.escape(),
            "enter" | "space" => screen.pressed(),
            other => return Err(format!("unknown key {other}")),
        };
        callbacks.extend(action);
    }
    if row["click"].as_bool().unwrap_or(false) {
        callbacks.extend(screen.clicked(mouse));
    }
    let blurriness = row["blurriness"].as_i64().unwrap_or(5) as i32;
    screen.draw(&mut frame, (atlas, font), mouse, blurriness);
    let (callbacks, sounds) = (callback_names(&callbacks), asked_sounds());
    Ok(json!({
        "label": row["label"],
        "callbacks": callbacks,
        "sounds": sounds,
        "blur": frame.blur.is_some(),
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

/// The names the Java callbacks of the confirm screen go by.
fn callback_names(callbacks: &[photon_gui::screen::confirm::ConfirmAction]) -> Vec<&'static str> {
    callbacks
        .iter()
        .map(|action| match action {
            photon_gui::screen::confirm::ConfirmAction::Yes => "yes",
            photon_gui::screen::confirm::ConfirmAction::No => "no",
            photon_gui::screen::confirm::ConfirmAction::Copy => "copy",
        })
        .collect()
}

/// What the widgets asked of the sound manager since the queue was last emptied, as the Java
/// oracle writes it: the event, the pitch and the volume.
pub(super) fn asked_sounds() -> Vec<String> {
    photon_gui::ui_sound::take()
        .iter()
        .map(|sound| format!("{} {:?} {:?}", sound.event, sound.pitch, sound.volume))
        .collect()
}

pub(super) fn widgets_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    let lang = photon_game::text::Lang::load(root);
    let width = int("width");
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    let mut arena = photon_gui::layout::Arena::default();
    let rect = arena.add(width, 20);
    (arena.0[rect].x, arena.0[rect].y) = (100, 50);
    let mut widget = OptionWidget::new(
        &lang,
        rect,
        row["kind"].as_str().unwrap_or_default(),
        &row["value"],
    )?;
    widget.set_active(row["active"].as_bool().unwrap_or(true));
    for step in row["steps"].as_array().ok_or("steps")? {
        widget.step(&lang, &arena, step)?;
    }
    widget.draw(&mut frame, atlas, font, &arena, mouse);
    Ok(json!({
        "label": row["label"],
        "value": widget.value(),
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

pub(super) fn death_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
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
    let cause = row["cause"]
        .as_str()
        .map(photon_game::text::Component::literal);
    let mut screen = photon_gui::screen::death::DeathScreen::new(
        &lang,
        size,
        cause.as_ref(),
        row["hardcore"].as_bool().unwrap_or(false),
        int("score"),
    );
    for _ in 0..int("ticks") {
        screen.tick();
    }
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    screen.draw(&mut frame, atlas, font, mouse);

    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

pub(super) fn disconnected_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
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
    let reason = photon_game::text::Component::literal(row["reason"].as_str().unwrap_or_default());
    let screen =
        photon_gui::screen::disconnected::DisconnectedScreen::new(font, &lang, size, &reason);
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    screen.draw(&mut frame, atlas, font, mouse);

    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

pub(super) fn credits_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    let assets = photon_core::resources::FolderResources::new(&ctx.root);
    let mut screen = photon_gui::screen::credits::CreditsScreen::new(
        &ctx.font,
        &assets,
        (int("width"), int("height")),
        row["poem"].as_bool().unwrap_or(false),
        row["player"].as_str().unwrap_or_default(),
    );
    screen.set_scroll(row["scroll"].as_f64().unwrap_or(0.0) as f32);
    for step in row["keys"].as_array().ok_or("keys")? {
        let code = step["code"].as_i64().unwrap_or(0) as i32;
        if step["down"].as_bool().unwrap_or(false) {
            screen.key_pressed(code);
        } else {
            screen.key_released(code);
        }
    }
    let finished = screen.tick();
    screen.draw(
        &mut frame,
        &ctx.atlas,
        &ctx.font,
        row["partial"].as_f64().unwrap_or(0.0) as f32,
    );
    Ok(json!({
        "label": row["label"],
        "finished": finished,
        // As the float's shortest text, which is what the reference writes.
        "scroll": screen.scroll().to_string().parse::<f64>().unwrap_or_default(),
        "lines": screen.line_count(),
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}
