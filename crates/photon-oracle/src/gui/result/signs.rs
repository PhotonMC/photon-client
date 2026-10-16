use super::*;

/// `sign_texts`: what a sign's text submits, each line's pose, place, runs, light and colours.
pub(super) fn sign_texts_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_render::block_entity::sign::{SignFace, SignKind, sign_text_submits};
    let lang = photon_game::text::Lang::load(&ctx.root);
    let kind = if row["kind"] == "hanging" {
        SignKind::Hanging
    } else {
        SignKind::Standing
    };
    let wall = row["wall"]
        .as_str()
        .map(|name| photon_core::direction::Direction::from_name(name).ok_or("wall"))
        .transpose()?;
    let filtering = row["filtering"].as_bool() == Some(true);
    // Each face's lines (written or filtered, as the row filters) in words.
    let face = |face: &Value| -> Result<([Vec<photon_game::text::Run>; 4], usize, bool), String> {
        let key = if filtering && !face["filtered"].is_null() {
            "filtered"
        } else {
            "messages"
        };
        let mut lines: [Vec<photon_game::text::Run>; 4] = Default::default();
        for (line, value) in lines.iter_mut().zip(face[key].as_array().ok_or("lines")?) {
            let json = photon_core::json::parse(&value.to_string()).map_err(|_| "json")?;
            let component = photon_game::text::Component::from_json(&json).ok_or("component")?;
            *line = lang.runs(&component, &photon_game::text::Style::default());
        }
        let color = photon_game::sign::DYES
            .iter()
            .position(|dye| face["color"] == *dye)
            .ok_or("dye")?;
        Ok((lines, color, face["glowing"].as_bool() == Some(true)))
    };
    let (front, back) = (face(&row["front"])?, face(&row["back"])?);
    let faces = [&front, &back].map(|(lines, color, glowing)| SignFace {
        lines,
        color: *color,
        glowing: *glowing,
    });
    let transformations = kind.transformations(wall, row["rotation"].as_i64().unwrap_or(0) as i32);
    let submits = sign_text_submits(
        &photon_render::joml::Pose::default(),
        kind,
        (&transformations, faces),
        (
            row["draw_outline"].as_bool() == Some(true),
            row["light"].as_i64().unwrap_or(0) as i32,
        ),
        &ctx.font,
    );
    let submits: Vec<Value> = submits
        .iter()
        .map(|submit| {
            // The line as runs of one colour, as the oracle merges them.
            let mut runs: Vec<(String, i64)> = Vec::new();
            for run in &submit.line {
                let colour = run.colour.map_or(-1, i64::from);
                match runs.last_mut() {
                    Some((text, last)) if *last == colour => text.push_str(&run.text),
                    _ if run.text.is_empty() => {}
                    _ => runs.push((run.text.clone(), colour)),
                }
            }
            json!({
                "pose": crate::strider::matrix_bits(&submit.pose),
                "x": format!("{:08x}", submit.x.to_bits()),
                "y": format!("{:08x}", submit.y.to_bits()),
                "runs": runs.iter().map(|(text, colour)| json!([text, colour])).collect::<Vec<_>>(),
                "shadow": false,
                "mode": "POLYGON_OFFSET",
                "light": submit.light,
                "color": format!("{:x}", submit.color),
                "background": 0,
                "outline": format!("{:x}", submit.outline),
            })
        })
        .collect();
    Ok(json!({"label": row["label"], "submits": submits}))
}

/// `sign_edit_screen`: the sign editing screen of a face after text and keys, drawn at a time
/// after it opened, and the lines it would send.
pub(super) fn sign_edit_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_render::gui::screen::sign_edit::{SignBoard, SignEditScreen};
    let (font, atlas) = (&ctx.font, &ctx.atlas);
    let lang = photon_game::text::Lang::load(&ctx.root);
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let board = match row["kind"].as_str() {
        Some("standing") => SignBoard::Standing,
        Some("wall") => SignBoard::Wall,
        _ => SignBoard::Hanging,
    };
    let face = &row["face"];
    let mut messages: [String; 4] = Default::default();
    for (message, value) in messages
        .iter_mut()
        .zip(face["messages"].as_array().ok_or("lines")?)
    {
        let json = photon_core::json::parse(&value.to_string()).map_err(|_| "json")?;
        let component = photon_game::text::Component::from_json(&json).ok_or("component")?;
        *message = lang.plain(&component);
    }
    let color = photon_game::sign::DYES
        .iter()
        .position(|dye| face["color"] == *dye)
        .ok_or("dye")?;
    let mut screen = SignEditScreen::new(
        &lang,
        (board, row["wood"].as_str().unwrap_or("oak")),
        messages,
        (color, face["glowing"].as_bool() == Some(true)),
        (int("width"), int("height")),
        Some(0),
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
                let control = name.starts_with("ctrl+");
                let name = name.trim_start_matches("ctrl+");
                let shift = name.starts_with("shift+");
                let key = match name.trim_start_matches("shift+") {
                    "tab" => glfw::Key::Tab,
                    "backspace" => glfw::Key::Backspace,
                    "delete" => glfw::Key::Delete,
                    "enter" => glfw::Key::Enter,
                    "left" => glfw::Key::Left,
                    "right" => glfw::Key::Right,
                    "up" => glfw::Key::Up,
                    "down" => glfw::Key::Down,
                    "home" => glfw::Key::Home,
                    "end" => glfw::Key::End,
                    "a" => glfw::Key::A,
                    other => return Err(format!("unknown key {other}")),
                };
                screen.key_pressed(font, key, (control, shift), &mut crate::gui::NoClipboard);
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
        "messages": screen.messages(),
    }))
}
