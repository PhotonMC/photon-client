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
    use voxel_render::gui::widgets::navigation::Direction;
    let lang = voxel_game::text::Lang::load(root);
    let size = (int("width"), int("height"));
    let mut screen = voxel_render::gui::screen::pause::PauseScreen::new(font, &lang, size);
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    for step in row["steps"].as_array().map_or(&[][..], Vec::as_slice) {
        if step["do"] == "frame" {
            // A frame drawn and thrown away, at a time.
            let mut thrown = GuiFrame {
                millis: step["ms"].as_u64().unwrap_or(0),
                ..GuiFrame::default()
            };
            screen.draw(&mut thrown, atlas, font, mouse, size, true);
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
    screen.draw(&mut frame, atlas, font, mouse, size, true);

    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
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
    use voxel_render::gui::screen::confirm::{ConfirmAction, ConfirmBackground, ConfirmScreen};
    let lang = voxel_game::text::Lang::load(root);
    let size = (int("width"), int("height"));
    let text =
        |key: &str| voxel_game::text::Component::literal(row[key].as_str().unwrap_or_default());
    let background = if row["death"].as_bool().unwrap_or(false) {
        ConfirmBackground::Death
    } else {
        ConfirmBackground::Menu {
            in_world: row["in_world"].as_bool().unwrap_or(false),
        }
    };
    let mut screen = match row["link"].as_object() {
        Some(link) => ConfirmScreen::link(
            font,
            &lang,
            size,
            link["url"].as_str().unwrap_or_default(),
            link["trusted"].as_bool().unwrap_or(false),
        ),
        None => ConfirmScreen::new(
            font,
            &lang,
            size,
            &text("title"),
            &text("message"),
            &text("yes"),
            &text("no"),
        ),
    }
    .with_background(background);
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
    screen.draw(&mut frame, atlas, font, mouse);
    let callbacks: Vec<&str> = callbacks
        .iter()
        .map(|action| match action {
            ConfirmAction::Yes => "yes",
            ConfirmAction::No => "no",
            ConfirmAction::Copy => "copy",
        })
        .collect();
    Ok(json!({
        "label": row["label"],
        "callbacks": callbacks,
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
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
    let lang = voxel_game::text::Lang::load(root);
    let width = int("width");
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    let mut arena = voxel_render::gui::layout::Arena::default();
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
    let lang = voxel_game::text::Lang::load(root);
    let size = (int("width"), int("height"));
    let cause = row["cause"]
        .as_str()
        .map(voxel_game::text::Component::literal);
    let mut screen = voxel_render::gui::screen::death::DeathScreen::new(
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
    let lang = voxel_game::text::Lang::load(root);
    let size = (int("width"), int("height"));
    let reason = voxel_game::text::Component::literal(row["reason"].as_str().unwrap_or_default());
    let screen = voxel_render::gui::screen::disconnected::DisconnectedScreen::new(
        font, &lang, size, &reason,
    );
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
