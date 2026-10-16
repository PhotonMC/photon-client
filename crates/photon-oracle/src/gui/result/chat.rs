use super::*;
use photon_render::gui::FrameRuns;

use photon_render::gui::chat::{ChatLog, ChatSettings};

pub(super) fn chat_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let languages = &mut ctx.languages;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    use photon_render::gui::chat::Tag;
    let mut log = ChatLog::default();
    // The chat options a row sets, before the messages and again
    // after them (which wraps them anew).
    let settings = |options: &Value, base: ChatSettings| ChatSettings {
        scale: options["scale"].as_f64().unwrap_or(base.scale),
        opacity: options["opacity"].as_f64().unwrap_or(base.opacity),
        width: options["width"].as_f64().unwrap_or(base.width),
        height_focused: options["height_focused"]
            .as_f64()
            .unwrap_or(base.height_focused),
        height_unfocused: options["height_unfocused"]
            .as_f64()
            .unwrap_or(base.height_unfocused),
        line_spacing: options["line_spacing"]
            .as_f64()
            .unwrap_or(base.line_spacing),
        background_opacity: options["background_opacity"]
            .as_f64()
            .unwrap_or(base.background_opacity),
    };
    log.set_settings(settings(&row["options"], log.settings()), font);
    let lang = photon_game::text::Lang::load(root);
    let marker_style = photon_game::text::Style {
        colour: photon_game::text::named_colour("gray"),
        italic: Some(true),
        ..photon_game::text::Style::default()
    };
    let marker = lang.runs(
        &photon_game::text::Component::translatable("chat.deleted_marker"),
        &marker_style,
    );
    // The messages and the deletions, each at its tick, the
    // chat's own tick after each.
    let mut events: Vec<&Value> = row["messages"]
        .as_array()
        .ok_or("messages")?
        .iter()
        .collect();
    if let Some(deletions) = row["deletions"].as_array() {
        events.extend(deletions.iter());
    }
    events.sort_by_key(|event| event["tick"].as_u64().unwrap_or(0));
    let signature = |id: u64| vec![id as u8; 256];
    for event in events {
        let tick = event["tick"].as_u64().unwrap_or(0);
        while log.ticks() < tick {
            log.tick(font, &marker);
        }
        if let Some(id) = event["delete"].as_u64() {
            log.delete(&signature(id), font, &marker);
            continue;
        }
        let component = match event["key"].as_str() {
            Some(key) => photon_game::text::Component::translatable(key),
            None if event["json"].is_string() => photon_game::text::Component::from_json(
                &photon_core::json::parse(event["json"].as_str().unwrap_or_default())
                    .map_err(|error| format!("{error:?}"))?,
            )
            .ok_or("component")?,
            None => photon_game::text::Component::literal(event["text"].as_str().ok_or("text")?),
        };
        let sig = event["signature"].as_u64().map(signature);
        let tag = match event["tag"].as_str() {
            None if sig.is_none() => Some(Tag::SystemSinglePlayer),
            None => None,
            Some("system") => Some(Tag::System),
            Some("system_single_player") => Some(Tag::SystemSinglePlayer),
            Some("not_secure") => Some(Tag::NotSecure),
            Some("error") => Some(Tag::Error),
            Some(other) => Some(Tag::Modified(
                other.trim_start_matches("modified:").to_owned(),
            )),
        };
        log.add_component(&component, &lang, (sig, tag), font, false);
    }
    while log.ticks() < row["tick"].as_u64().unwrap_or(0) {
        log.tick(font, &marker);
    }
    if let Some(code) = row["lang_after"].as_str() {
        let source = match &languages {
            Some(source) => source,
            None => languages
                .insert(photon_game::language::LanguageSource::open(root).ok_or("languages")?),
        };
        log.set_lang(&source.lang(code), font);
    }
    if row["rescale"].is_object() {
        log.set_settings(settings(&row["rescale"], log.settings()), font);
    }
    let focused = row["focused"].as_bool().unwrap_or(false);
    if let Some(scroll) = row["scroll"].as_i64() {
        log.scroll_by(scroll as i32, focused);
    }
    let gui = (int("width"), int("height"));
    match row["mouse"].as_array() {
        // The mouse over the open chat: the tooltip of what is under it.
        Some(mouse) => {
            let mouse = (
                mouse[0].as_i64().unwrap_or(0) as i32,
                mouse[1].as_i64().unwrap_or(0) as i32,
            );
            let tooltip = log.draw_with(&mut frame, atlas, font, gui, focused, Some(mouse), &lang);
            if let Some(lines) = tooltip {
                frame.tooltip(atlas, font, &lines, mouse, gui);
            }
        }
        None => log.draw(&mut frame, atlas, font, gui, focused),
    }

    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

pub(super) fn suggestions_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    use photon_render::gui::chat::ChatScreen;
    let width = int("width");
    let height = int("height");
    let text = row["text"].as_str().ok_or("text")?;
    let mut screen = ChatScreen::new(
        std::sync::Arc::new(Font::load(root).ok_or("font")?),
        (width, height),
        text,
        Vec::new(),
    )
    .without_key();
    let matches = row["matches"]
        .as_array()
        .ok_or("matches")?
        .iter()
        .filter_map(|m| m.as_str().map(str::to_owned))
        .collect();
    screen.suggested(
        0,
        row["start"].as_u64().unwrap_or(0) as usize,
        row["length"].as_u64().unwrap_or(0) as usize,
        matches,
    );
    let mut keys = vec![String::new()];
    keys.extend(
        row["keys"]
            .as_array()
            .ok_or("keys")?
            .iter()
            .filter_map(|k| k.as_str().map(str::to_owned)),
    );
    let mut steps = Vec::new();
    for key in keys {
        if !key.is_empty() {
            let (glfw_key, shift) = match key.as_str() {
                "tab" => (glfw::Key::Tab, false),
                "shift+tab" => (glfw::Key::Tab, true),
                "up" => (glfw::Key::Up, false),
                "down" => (glfw::Key::Down, false),
                "escape" => (glfw::Key::Escape, false),
                _ => return Err("unknown key".into()),
            };
            screen.key_pressed(glfw_key, (false, shift), &mut NoClipboard);
        }
        let mut drawn = GuiFrame {
            log: Some(Vec::new()),
            ..GuiFrame::default()
        };
        screen.refocus();
        screen.draw(&mut drawn, atlas, (0, 0), None);
        steps.push(json!({
            "key": key,
            "value": screen.value(),
            "cursor": screen.cursor(),
            "ops": gui_ops(&drawn.log.take().unwrap_or_default()),
        }));
    }
    Ok(json!({"label": row["label"], "steps": steps}))
}

pub(super) fn chat_draft_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    use photon_render::gui::chat::{ChatDrafts, ChatMethod, ChatScreen, ExitReason};
    let (width, height) = (int("width"), int("height"));
    let save = row["save"].as_bool().unwrap_or(false);
    let font = std::sync::Arc::new(Font::load(root).ok_or("font")?);
    let mut drafts = ChatDrafts::default();
    let mut screen: Option<ChatScreen> = None;
    let mut events = Vec::new();
    for event in row["events"].as_array().ok_or("events")? {
        let mut entry = json!({"do": event["do"]});
        match event["do"].as_str().unwrap_or_default() {
            "open" => {
                let method = if event["method"] == "command" {
                    ChatMethod::Command
                } else {
                    ChatMethod::Message
                };
                let (initial, draft) = drafts.open(method);
                let opened = ChatScreen::new(font.clone(), (width, height), &initial, Vec::new())
                    .without_key();
                screen = Some(if draft { opened.as_draft() } else { opened });
            }
            "type" => {
                for character in event["text"].as_str().unwrap_or_default().chars() {
                    screen.as_mut().ok_or("no box")?.char_typed(character);
                }
            }
            "key" => {
                let key = match event["key"].as_str().unwrap_or_default() {
                    "backspace" => glfw::Key::Backspace,
                    "left" => glfw::Key::Left,
                    "right" => glfw::Key::Right,
                    "home" => glfw::Key::Home,
                    "end" => glfw::Key::End,
                    "delete" => glfw::Key::Delete,
                    "up" => glfw::Key::Up,
                    "down" => glfw::Key::Down,
                    other => return Err(format!("unknown key {other}")),
                };
                screen
                    .as_mut()
                    .ok_or("no box")?
                    .key_pressed(key, (false, false), &mut NoClipboard);
            }
            "close" => {
                let closed = screen.take().ok_or("no box")?;
                let reason = match event["reason"].as_str().unwrap_or_default() {
                    "intentional" => ExitReason::Intentional,
                    "interrupted" => ExitReason::Interrupted,
                    _ => ExitReason::Done,
                };
                drafts.removed(&closed.value(), closed.is_draft(), reason, save);
            }
            "draw" => {
                let mut drawn = GuiFrame {
                    log: Some(Vec::new()),
                    ..GuiFrame::default()
                };
                let shown = screen.as_mut().ok_or("no box")?;
                shown.refocus();
                shown.draw(&mut drawn, atlas, (0, 0), None);
                entry["ops"] = json!(gui_ops(&drawn.log.take().unwrap_or_default()));
            }
            other => return Err(format!("unknown event {other}")),
        }
        if let Some(shown) = &screen {
            entry["value"] = json!(shown.value());
            entry["draft"] = json!(shown.is_draft());
        }
        // The reference leaves a null out.
        if let Some((text, method)) = drafts.kept() {
            entry["kept"] = json!({
                "text": text,
                "method": if method == ChatMethod::Command { "COMMAND" } else { "MESSAGE" },
            });
        }
        events.push(entry);
    }
    Ok(json!({"label": row["label"], "events": events}))
}
