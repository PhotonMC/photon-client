use super::*;

/// `title_screen`: the title screen with a splash, after some keys and frames, under a mouse.
pub(super) fn title_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use voxel_render::gui::widgets::navigation::Direction;
    let (font, atlas) = (&ctx.font, &ctx.atlas);
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let lang = voxel_game::text::Lang::load(&ctx.root);
    let size = (int("width"), int("height"));
    let mut screen = voxel_render::gui::screen::title::TitleScreen::new(
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
