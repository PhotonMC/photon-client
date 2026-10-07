use super::*;

pub(crate) fn gui_result(request: &Value) -> Result<Value, String> {
    if request["operation"].as_str() == Some("name_tags") {
        return crate::name_tags_result(request);
    }
    use voxel_render::gui::chat::{ChatLog, ChatSettings};
    use voxel_render::gui::font::Font;
    use voxel_render::gui::{GuiAtlas, GuiFrame};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/assets");
    let font = Font::load(&root).ok_or("font")?;
    let alt = Font::load_alt(&root).ok_or("alt font")?;
    let (atlas, _) = GuiAtlas::load(&root, &[&font, &alt]).ok_or("gui atlas")?;
    if request["operation"] == "command_parse" {
        return crate::command_parse_result(request, &root);
    }
    if request["operation"] == "command_suggest" {
        return crate::command_suggest_result(request);
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("gui rows must be an array")?;
    let mut out = Vec::new();
    // The languages' source is read once: its index has thousands of files.
    let mut languages: Option<voxel_game::language::LanguageSource> = None;
    for row in rows {
        let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
        let mut frame = GuiFrame {
            log: Some(Vec::new()),
            ..GuiFrame::default()
        };
        match crate::string(request, "operation")? {
            "chat" => {
                use voxel_render::gui::chat::Tag;
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
                log.set_settings(settings(&row["options"], log.settings()), &font);
                let lang = voxel_game::text::Lang::load(&root);
                let marker_style = voxel_game::text::Style {
                    colour: voxel_game::text::named_colour("gray"),
                    italic: Some(true),
                    ..voxel_game::text::Style::default()
                };
                let marker = lang.runs(
                    &voxel_game::text::Component::translatable("chat.deleted_marker"),
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
                        log.tick(&font, &marker);
                    }
                    if let Some(id) = event["delete"].as_u64() {
                        log.delete(&signature(id), &font, &marker);
                        continue;
                    }
                    let component = match event["key"].as_str() {
                        Some(key) => voxel_game::text::Component::translatable(key),
                        None if event["json"].is_string() => {
                            voxel_game::text::Component::from_json(
                                &voxel_core::json::parse(
                                    event["json"].as_str().unwrap_or_default(),
                                )
                                .map_err(|error| format!("{error:?}"))?,
                            )
                            .ok_or("component")?
                        }
                        None => voxel_game::text::Component::literal(
                            event["text"].as_str().ok_or("text")?,
                        ),
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
                    log.add_component(&component, &lang, (sig, tag), &font, false);
                }
                while log.ticks() < row["tick"].as_u64().unwrap_or(0) {
                    log.tick(&font, &marker);
                }
                if let Some(code) = row["lang_after"].as_str() {
                    let source = match &languages {
                        Some(source) => source,
                        None => languages.insert(
                            voxel_game::language::LanguageSource::open(&root).ok_or("languages")?,
                        ),
                    };
                    log.set_lang(&source.lang(code), &font);
                }
                if row["rescale"].is_object() {
                    log.set_settings(settings(&row["rescale"], log.settings()), &font);
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
                        let tooltip = log.draw_with(
                            &mut frame,
                            &atlas,
                            &font,
                            gui,
                            focused,
                            Some(mouse),
                            &lang,
                        );
                        if let Some(lines) = tooltip {
                            frame.tooltip(&atlas, &font, &lines, mouse, gui);
                        }
                    }
                    None => log.draw(&mut frame, &atlas, &font, gui, focused),
                }
            }
            "suggestions" => {
                use voxel_render::gui::chat::ChatScreen;
                let width = int("width");
                let height = int("height");
                let text = row["text"].as_str().ok_or("text")?;
                let mut screen = ChatScreen::new(
                    std::sync::Arc::new(Font::load(&root).ok_or("font")?),
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
                    screen.draw(&mut drawn, &atlas, (0, 0), None);
                    steps.push(json!({
                        "key": key,
                        "value": screen.value(),
                        "cursor": screen.cursor(),
                        "ops": gui_ops(&drawn.log.take().unwrap_or_default()),
                    }));
                }
                out.push(json!({"label": row["label"], "steps": steps}));
                continue;
            }
            "chat_draft" => {
                use voxel_render::gui::chat::{ChatDrafts, ChatMethod, ChatScreen, ExitReason};
                let (width, height) = (int("width"), int("height"));
                let save = row["save"].as_bool().unwrap_or(false);
                let font = std::sync::Arc::new(Font::load(&root).ok_or("font")?);
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
                            let opened = ChatScreen::new(
                                font.clone(),
                                (width, height),
                                &initial,
                                Vec::new(),
                            )
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
                            screen.as_mut().ok_or("no box")?.key_pressed(
                                key,
                                (false, false),
                                &mut NoClipboard,
                            );
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
                            shown.draw(&mut drawn, &atlas, (0, 0), None);
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
                out.push(json!({"label": row["label"], "events": events}));
                continue;
            }
            "titles" => {
                use voxel_render::gui::titles::Titles;
                let mut titles = Titles::default();
                titles.set_times(int("fade_in"), int("stay"), int("fade_out"));
                let runs = |text: &str| voxel_render::gui::chat::plain_runs(text);
                if let Some(title) = row["title"].as_str() {
                    titles.set_title(runs(title));
                    for _ in 0..int("elapsed") {
                        titles.tick();
                    }
                }
                if let Some(subtitle) = row["subtitle"].as_str() {
                    titles.set_subtitle(runs(subtitle));
                }
                if let Some(overlay) = row["overlay"].as_str() {
                    titles.set_overlay(runs(overlay));
                    for _ in 0..int("overlay_elapsed") {
                        titles.tick();
                    }
                }
                let partial = row["partial"].as_f64().unwrap_or(0.0) as f32;
                titles.draw(
                    &mut frame,
                    &atlas,
                    &font,
                    (int("width"), int("height")),
                    partial,
                );
            }
            "sidebar" => {
                use voxel_render::gui::sidebar;
                let lang = voxel_game::text::Lang::new([]);
                let board = scoreboard_from(row)?;
                let style = voxel_game::text::Style::default();
                if let Some((name, objective)) =
                    board.sidebar_objective(row["local"].as_str().unwrap_or_default())
                {
                    let lines: Vec<sidebar::SidebarLine> = board
                        .sidebar_lines(name)
                        .iter()
                        .map(|(name, score)| (lang.runs(name, &style), lang.runs(score, &style)))
                        .collect();
                    sidebar::draw(
                        &mut frame,
                        &atlas,
                        &font,
                        (int("width"), int("height")),
                        &lang.runs(&objective.display_name, &style),
                        &lines,
                    );
                }
            }
            "pause_screen" => {
                use voxel_render::gui::navigation::Direction;
                let lang = voxel_game::text::Lang::load(&root);
                let size = (int("width"), int("height"));
                let mut screen = voxel_render::gui::pause::PauseScreen::new(&font, &lang, size);
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
                        screen.draw(&mut thrown, &atlas, &font, mouse, size, true);
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
                screen.draw(&mut frame, &atlas, &font, mouse, size, true);
            }
            "confirm_screen" => {
                use voxel_render::gui::confirm::{ConfirmAction, ConfirmBackground, ConfirmScreen};
                let lang = voxel_game::text::Lang::load(&root);
                let size = (int("width"), int("height"));
                let text = |key: &str| {
                    voxel_game::text::Component::literal(row[key].as_str().unwrap_or_default())
                };
                let background = if row["death"].as_bool().unwrap_or(false) {
                    ConfirmBackground::Death
                } else {
                    ConfirmBackground::Menu {
                        in_world: row["in_world"].as_bool().unwrap_or(false),
                    }
                };
                let mut screen = match row["link"].as_object() {
                    Some(link) => ConfirmScreen::link(
                        &font,
                        &lang,
                        size,
                        link["url"].as_str().unwrap_or_default(),
                        link["trusted"].as_bool().unwrap_or(false),
                    ),
                    None => ConfirmScreen::new(
                        &font,
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
                screen.draw(&mut frame, &atlas, &font, mouse);
                let callbacks: Vec<&str> = callbacks
                    .iter()
                    .map(|action| match action {
                        ConfirmAction::Yes => "yes",
                        ConfirmAction::No => "no",
                        ConfirmAction::Copy => "copy",
                    })
                    .collect();
                out.push(json!({
                    "label": row["label"],
                    "callbacks": callbacks,
                    "ops": gui_ops(&frame.log.take().unwrap_or_default()),
                }));
                continue;
            }
            "widgets" => {
                let lang = voxel_game::text::Lang::load(&root);
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
                widget.draw(&mut frame, &atlas, &font, &arena, mouse);
                out.push(json!({
                    "label": row["label"],
                    "value": widget.value(),
                    "ops": gui_ops(&frame.log.take().unwrap_or_default()),
                }));
                continue;
            }
            "options_screen" => {
                use voxel_render::gui::options::OptionsScreen;
                use voxel_render::gui::slider::SliderKey;
                let lang = voxel_game::text::Lang::load(&root);
                let size = (int("width"), int("height"));
                let mouse = (
                    row["mouse"][0].as_i64().unwrap_or(0) as i32,
                    row["mouse"][1].as_i64().unwrap_or(0) as i32,
                );
                let mut fov = int("fov");
                let mut screen = OptionsScreen::new(
                    &font,
                    &lang,
                    size,
                    fov,
                    row["in_world"].as_bool().unwrap_or(false),
                );
                for step in row["steps"].as_array().ok_or("steps")? {
                    let at = (
                        step["x"].as_f64().unwrap_or(0.0),
                        step["y"].as_f64().unwrap_or(0.0),
                    );
                    let input = match step["do"].as_str().unwrap_or_default() {
                        "key" => match step["key"].as_str().unwrap_or_default() {
                            "tab" => {
                                screen.tab(true);
                                Default::default()
                            }
                            "shift+tab" => {
                                screen.tab(false);
                                Default::default()
                            }
                            "left" => screen.key_pressed(&lang, SliderKey::Left),
                            "right" => screen.key_pressed(&lang, SliderKey::Right),
                            "up" => screen.key_pressed(&lang, SliderKey::Up),
                            "down" => screen.key_pressed(&lang, SliderKey::Down),
                            "enter" | "space" => screen.key_pressed(&lang, SliderKey::Select),
                            other => return Err(format!("unknown key {other}")),
                        },
                        "click" => screen.clicked(&lang, at),
                        "drag" => screen.dragged(&lang, at.0),
                        "release" => {
                            screen.released(&lang);
                            Default::default()
                        }
                        other => return Err(format!("unknown step {other}")),
                    };
                    fov = input.fov.unwrap_or(fov);
                }
                screen.draw(&mut frame, &atlas, &font, mouse);
                out.push(json!({
                    "label": row["label"],
                    "fov": fov,
                    "ops": gui_ops(&frame.log.take().unwrap_or_default()),
                }));
                continue;
            }
            "video_settings_screen" => {
                use voxel_render::gui::video::VideoSettingsScreen;
                use voxel_render::window::scale::WindowScale;
                let lang = voxel_game::text::Lang::load(&root);
                let framebuffer = (
                    row["framebuffer"][0].as_i64().unwrap_or(0) as i32,
                    row["framebuffer"][1].as_i64().unwrap_or(0) as i32,
                );
                let mouse = (
                    row["mouse"][0].as_i64().unwrap_or(0) as i32,
                    row["mouse"][1].as_i64().unwrap_or(0) as i32,
                );
                let mut window = WindowScale::new(framebuffer.0, framebuffer.1);
                let mut gui_scale = int("gui_scale");
                window.set_gui_scale(window.calculate_scale(gui_scale, false));
                let max_scale = window.calculate_scale(0, false);
                let mut screen = VideoSettingsScreen::new(
                    &font,
                    &lang,
                    (window.gui_scaled_width(), window.gui_scaled_height()),
                    row["in_world"].as_bool().unwrap_or(false),
                    gui_scale,
                    max_scale,
                    voxel_render::gui::video::VideoValues::default(),
                );
                let mut last = (0.0, 0.0);
                let mut clock = 0_u64;
                for step in row["steps"].as_array().ok_or("steps")? {
                    if step["do"] == "frame" {
                        clock = step["ms"].as_u64().unwrap_or(0);
                        let mut thrown = GuiFrame {
                            millis: clock,
                            ..GuiFrame::default()
                        };
                        screen.draw(&mut thrown, &atlas, &font, mouse);
                        continue;
                    }
                    // The reference draws a frame between one event and the next.
                    let mut between = GuiFrame {
                        millis: clock,
                        ..GuiFrame::default()
                    };
                    screen.draw(&mut between, &atlas, &font, mouse);
                    let at = (
                        step["x"].as_f64().unwrap_or(last.0),
                        step["y"].as_f64().unwrap_or(last.1),
                    );
                    // `Minecraft.resizeGui`: the window takes the scale.
                    let mut resize = |new: i32| {
                        gui_scale = new;
                        window.set_gui_scale(window.calculate_scale(new, false));
                        (window.gui_scaled_width(), window.gui_scaled_height())
                    };
                    match step["do"].as_str().unwrap_or_default() {
                        "click" => {
                            screen.clicked(
                                &lang,
                                at,
                                step["shift"].as_bool().unwrap_or(false),
                                &mut resize,
                            );
                        }
                        "key" => match menu_key(step)? {
                            MenuKey::Tab(forward) => screen.tab(forward),
                            MenuKey::Key(key) => {
                                screen.key_pressed(&lang, key, false, &mut resize);
                            }
                        },
                        "drag" => screen.dragged(&lang, at, at.1 - last.1),
                        "release" => screen.released(&lang),
                        "scroll" => screen.scrolled(
                            &lang,
                            at,
                            step["dy"].as_f64().unwrap_or(0.0),
                            step["ctrl"].as_bool().unwrap_or(false),
                            &mut resize,
                        ),
                        other => return Err(format!("unknown step {other}")),
                    }
                    last = at;
                }
                frame.millis = row["ms"].as_u64().unwrap_or(0);
                screen.draw(&mut frame, &atlas, &font, mouse);
                out.push(json!({
                    "label": row["label"],
                    "width": window.gui_scaled_width(),
                    "height": window.gui_scaled_height(),
                    "gui_scale": gui_scale,
                    "scroll": screen.scroll_amount(),
                    "ops": gui_ops(&frame.log.take().unwrap_or_default()),
                }));
                continue;
            }
            "chat_options_screen"
            | "mouse_settings_screen"
            | "controls_screen"
            | "sound_options_screen" => {
                use voxel_render::gui::chat_options::ChatOptionsScreen;
                use voxel_render::gui::mouse_options::{ControlsScreen, MouseSettingsScreen};
                use voxel_render::gui::sound_options::SoundOptionsScreen;
                // The settings screens that have the same inputs.
                enum Shell {
                    Chat(ChatOptionsScreen),
                    Mouse(MouseSettingsScreen),
                    Controls(ControlsScreen),
                    Sound(SoundOptionsScreen),
                }
                macro_rules! each {
                    ($shell:expr, $screen:ident => $body:expr) => {
                        match $shell {
                            Shell::Chat($screen) => $body,
                            Shell::Mouse($screen) => $body,
                            Shell::Controls($screen) => $body,
                            Shell::Sound($screen) => $body,
                        }
                    };
                }
                use voxel_render::window::scale::WindowScale;
                let lang = voxel_game::text::Lang::load(&root);
                let framebuffer = (
                    row["framebuffer"][0].as_i64().unwrap_or(0) as i32,
                    row["framebuffer"][1].as_i64().unwrap_or(0) as i32,
                );
                let mouse = (
                    row["mouse"][0].as_i64().unwrap_or(0) as i32,
                    row["mouse"][1].as_i64().unwrap_or(0) as i32,
                );
                let mut window = WindowScale::new(framebuffer.0, framebuffer.1);
                window.set_gui_scale(window.calculate_scale(int("gui_scale"), false));
                let mut options = voxel_game::options::Options::default();
                let set = |key: &str, field: &mut f64| {
                    if let Some(value) = row["options"][key].as_f64() {
                        *field = value;
                    }
                };
                set("scale", &mut options.chat_scale);
                set("opacity", &mut options.chat_opacity);
                set("width", &mut options.chat_width);
                set("height_focused", &mut options.chat_height_focused);
                set("height_unfocused", &mut options.chat_height_unfocused);
                set("line_spacing", &mut options.chat_line_spacing);
                set("background_opacity", &mut options.text_background_opacity);
                set("sensitivity", &mut options.sensitivity);
                set("wheel", &mut options.mouse_wheel_sensitivity);
                options.invert_mouse_x = row["options"]["invert_x"].as_bool().unwrap_or(false);
                options.invert_mouse_y = row["options"]["invert_y"].as_bool().unwrap_or(false);
                for source in voxel_game::sounds::SoundSource::ALL {
                    if let Some(volume) = row["options"]["volumes"][source.name()].as_f64() {
                        options.sound_volumes[source as usize] = volume;
                    }
                }
                options.show_subtitles = row["options"]["subtitles"].as_bool().unwrap_or(false);
                options.directional_audio =
                    row["options"]["directional"].as_bool().unwrap_or(false);
                if let Some(value) = row["options"]["music_frequency"]
                    .as_str()
                    .and_then(voxel_game::options::MusicFrequency::from_serialized)
                {
                    options.music_frequency = value;
                }
                if let Some(value) = row["options"]["music_toast"]
                    .as_str()
                    .and_then(voxel_game::options::MusicToast::from_serialized)
                {
                    options.music_toast = value;
                }
                let size = (window.gui_scaled_width(), window.gui_scaled_height());
                let in_world = row["in_world"].as_bool().unwrap_or(false);
                let mut screen = match crate::string(request, "operation")? {
                    "chat_options_screen" => Shell::Chat(ChatOptionsScreen::new(
                        &font, &lang, size, in_world, &options,
                    )),
                    "mouse_settings_screen" => Shell::Mouse(MouseSettingsScreen::new(
                        &font, &lang, size, in_world, &options,
                    )),
                    "sound_options_screen" => Shell::Sound(SoundOptionsScreen::new(
                        &font, &lang, size, in_world, &options,
                    )),
                    _ => Shell::Controls(ControlsScreen::new(&font, &lang, size, in_world)),
                };
                let mut last = (0.0, 0.0);
                let mut clock = 0_u64;
                for step in row["steps"].as_array().ok_or("steps")? {
                    if step["do"] == "frame" {
                        clock = step["ms"].as_u64().unwrap_or(0);
                        let mut thrown = GuiFrame {
                            millis: clock,
                            ..GuiFrame::default()
                        };
                        each!(&screen, s => s.draw(&mut thrown, &atlas, &font, mouse));
                        continue;
                    }
                    // The reference draws a frame between one event and the next.
                    let mut between = GuiFrame {
                        millis: clock,
                        ..GuiFrame::default()
                    };
                    each!(&screen, s => s.draw(&mut between, &atlas, &font, mouse));
                    let at = (
                        step["x"].as_f64().unwrap_or(last.0),
                        step["y"].as_f64().unwrap_or(last.1),
                    );
                    match step["do"].as_str().unwrap_or_default() {
                        "click" => {
                            let shift = step["shift"].as_bool().unwrap_or(false);
                            each!(&mut screen, s => { s.clicked(&lang, at, shift); });
                        }
                        "key" => match menu_key(step)? {
                            MenuKey::Tab(forward) => each!(&mut screen, s => s.tab(forward)),
                            MenuKey::Key(key) => {
                                each!(&mut screen, s => { s.key_pressed(&lang, key, false); });
                            }
                        },
                        "drag" => each!(&mut screen, s => s.dragged(&lang, at, at.1 - last.1)),
                        "release" => each!(&mut screen, s => s.released(&lang)),
                        "scroll" => {
                            let dy = step["dy"].as_f64().unwrap_or(0.0);
                            each!(&mut screen, s => s.scrolled(at, dy));
                        }
                        other => return Err(format!("unknown step {other}")),
                    }
                    last = at;
                }
                frame.millis = row["ms"].as_u64().unwrap_or(0);
                each!(&screen, s => s.draw(&mut frame, &atlas, &font, mouse));
                out.push(json!({
                    "label": row["label"],
                    "width": window.gui_scaled_width(),
                    "height": window.gui_scaled_height(),
                    "gui_scale": int("gui_scale"),
                    "scroll": each!(&screen, s => s.scroll_amount()),
                    "ops": gui_ops(&frame.log.take().unwrap_or_default()),
                }));
                continue;
            }
            "lang_text" => {
                let source = match &languages {
                    Some(source) => source,
                    None => languages.insert(
                        voxel_game::language::LanguageSource::open(&root).ok_or("languages")?,
                    ),
                };
                let lang = source.lang(row["language"].as_str().unwrap_or("en_us"));
                let keys = row["keys"].as_array().ok_or("keys")?;
                out.push(json!({
                    "label": row["label"],
                    "texts": keys.iter().map(|key| lang.text(key.as_str().unwrap_or_default())).collect::<Vec<_>>(),
                }));
                continue;
            }
            "language_screen" => {
                use voxel_render::gui::language_select::LanguageSelectScreen;
                use voxel_render::window::scale::WindowScale;
                struct NoClipboard;
                impl voxel_render::gui::edit_box::Clipboard for NoClipboard {
                    fn get(&mut self) -> String {
                        String::new()
                    }
                    fn set(&mut self, _text: &str) {}
                }
                let source = match &languages {
                    Some(source) => source,
                    None => languages.insert(
                        voxel_game::language::LanguageSource::open(&root).ok_or("languages")?,
                    ),
                };
                let code = row["language"].as_str().unwrap_or("en_us");
                let lang = source.lang(code);
                let framebuffer = (
                    row["framebuffer"][0].as_i64().unwrap_or(0) as i32,
                    row["framebuffer"][1].as_i64().unwrap_or(0) as i32,
                );
                let mouse = (
                    row["mouse"][0].as_i64().unwrap_or(0) as i32,
                    row["mouse"][1].as_i64().unwrap_or(0) as i32,
                );
                let mut window = WindowScale::new(framebuffer.0, framebuffer.1);
                window.set_gui_scale(window.calculate_scale(int("gui_scale"), false));
                let mut screen = LanguageSelectScreen::new(
                    &font,
                    &lang,
                    (window.gui_scaled_width(), window.gui_scaled_height()),
                    row["in_world"].as_bool().unwrap_or(false),
                    source
                        .languages()
                        .into_iter()
                        .filter(|(code, _)| {
                            row["codes"].as_array().is_none_or(|codes| {
                                codes.is_empty()
                                    || codes.iter().any(|listed| listed == code.as_str())
                            })
                        })
                        .collect(),
                    code,
                );
                let mut last = (0.0, 0.0);
                for step in row["steps"].as_array().ok_or("steps")? {
                    let at = (
                        step["x"].as_f64().unwrap_or(last.0),
                        step["y"].as_f64().unwrap_or(last.1),
                    );
                    match step["do"].as_str().unwrap_or_default() {
                        "click" => {
                            screen.clicked(&font, at, step["double"].as_bool().unwrap_or(false));
                        }
                        "char" => {
                            if let Some(character) =
                                step["char"].as_str().and_then(|text| text.chars().next())
                            {
                                screen.char_typed(&font, character);
                            }
                        }
                        "key" => {
                            let name = step["key"].as_str().unwrap_or_default();
                            let key = match name {
                                "backspace" => glfw::Key::Backspace,
                                "enter" => glfw::Key::Enter,
                                "space" => glfw::Key::Space,
                                "tab" | "shift+tab" => glfw::Key::Tab,
                                "left" => glfw::Key::Left,
                                "right" => glfw::Key::Right,
                                "up" => glfw::Key::Up,
                                "down" => glfw::Key::Down,
                                other => return Err(format!("unknown key {other}")),
                            };
                            let shift = name.starts_with("shift+");
                            screen.key_pressed(&font, key, (false, shift), &mut NoClipboard);
                        }
                        "drag" => screen.dragged(at, at.1 - last.1),
                        "release" => screen.released(),
                        "scroll" => screen.scrolled(at, step["dy"].as_f64().unwrap_or(0.0)),
                        other => return Err(format!("unknown step {other}")),
                    }
                    last = at;
                }
                screen.draw(&mut frame, &atlas, &font, mouse);
                out.push(json!({
                    "label": row["label"],
                    "width": window.gui_scaled_width(),
                    "height": window.gui_scaled_height(),
                    "gui_scale": int("gui_scale"),
                    "scroll": screen.scroll_amount(),
                    "search": screen.search_text(),
                    "ops": gui_ops(&frame.log.take().unwrap_or_default()),
                }));
                continue;
            }
            "death_screen" => {
                let lang = voxel_game::text::Lang::load(&root);
                let size = (int("width"), int("height"));
                let cause = row["cause"]
                    .as_str()
                    .map(voxel_game::text::Component::literal);
                let mut screen = voxel_render::gui::death::DeathScreen::new(
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
                screen.draw(&mut frame, &atlas, &font, mouse);
            }
            "disconnected_screen" => {
                let lang = voxel_game::text::Lang::load(&root);
                let size = (int("width"), int("height"));
                let reason = voxel_game::text::Component::literal(
                    row["reason"].as_str().unwrap_or_default(),
                );
                let screen = voxel_render::gui::disconnected::DisconnectedScreen::new(
                    &font, &lang, size, &reason,
                );
                let mouse = (
                    row["mouse"][0].as_i64().unwrap_or(0) as i32,
                    row["mouse"][1].as_i64().unwrap_or(0) as i32,
                );
                screen.draw(&mut frame, &atlas, &font, mouse);
            }
            "hud_hotbar" => {
                use voxel_render::gui::hud::{Hud, HudInput};
                let flag = |key: &str| row[key].as_bool().unwrap_or(false);
                let speed = row["attack_speed"].as_f64().unwrap_or(4.0);
                let delay = voxel_world::local_player::attack_strength_delay(speed);
                let ticks = row["attack_ticks"].as_i64().unwrap_or(0) as f32;
                let attack_indicator =
                    voxel_game::options::AttackIndicator::ALL[int("attack_indicator") as usize];
                let input = HudInput {
                    first_person: !flag("third_person"),
                    spectator: flag("spectator"),
                    // The reference draws the crosshair and the hotbar alone here.
                    survival: false,
                    selected_slot: int("selected_slot"),
                    health: 20.0,
                    max_health: 20.0,
                    absorption: 0.0,
                    invulnerable_time: 0,
                    heart_type: voxel_render::gui::HeartType::Normal,
                    regeneration: false,
                    hardcore: false,
                    armor: 0,
                    food: 20,
                    saturation: 5.0,
                    hunger: false,
                    air_supply: 300,
                    max_air_supply: 300,
                    eye_in_water: false,
                    experience_progress: 0.0,
                    experience_level: 0,
                    experience_prioritised: false,
                    locator: None,
                    attack_strength: (ticks / delay).clamp(0.0, 1.0),
                    attack_delay: delay,
                    attack_indicator,
                    target_is_living: row["target"] == "living",
                    hotbar: Default::default(),
                    offhand: None,
                };
                let mut draw_item =
                    |_: &voxel_world::ClientItemStackState,
                     _: f32,
                     _: f32,
                     _: &mut Vec<voxel_render::gui::GuiItemVertex>| {};
                Hud::default().draw(
                    &mut frame,
                    &atlas,
                    &font,
                    int("width"),
                    int("height"),
                    &input,
                    0,
                    &mut draw_item,
                );
            }
            "hud_status" => {
                use voxel_render::gui::HeartType;
                use voxel_render::gui::hud::{Hud, HudInput};
                let number = |key: &str| row[key].as_f64().unwrap_or(0.0);
                let has = |name: &str| {
                    row["effects"]
                        .as_array()
                        .is_some_and(|effects| effects.iter().any(|effect| effect == name))
                };
                let float = |key: &str| -> Result<f32, String> {
                    u32::from_str_radix(row[key].as_str().ok_or(key)?, 16)
                        .map(f32::from_bits)
                        .map_err(|error| error.to_string())
                };
                let mut hud = Hud::default();
                let mut ticks = 0;
                let mut frames = Vec::new();
                for step in row["frames"].as_array().ok_or("frames")? {
                    let tick = step["tick"].as_i64().unwrap_or(0);
                    while ticks < tick {
                        hud.tick();
                        ticks += 1;
                    }
                    let input = HudInput {
                        first_person: true,
                        spectator: false,
                        survival: true,
                        selected_slot: 0,
                        health: u32::from_str_radix(step["health"].as_str().ok_or("health")?, 16)
                            .map(f32::from_bits)
                            .map_err(|error| error.to_string())?,
                        max_health: number("max_health") as f32,
                        absorption: float("absorption")?,
                        invulnerable_time: step["invulnerable"].as_i64().unwrap_or(0) as i32,
                        heart_type: HeartType::for_player(
                            has("poison"),
                            has("wither"),
                            int("frozen_ticks") >= 140,
                        ),
                        regeneration: has("regeneration"),
                        hardcore: row["hardcore"].as_bool().unwrap_or(false),
                        armor: number("armor").floor() as i32,
                        food: int("food"),
                        saturation: float("saturation")?,
                        hunger: has("hunger"),
                        air_supply: int("air"),
                        max_air_supply: 300,
                        eye_in_water: row["eye_in_water"].as_bool().unwrap_or(false),
                        experience_progress: float("xp_progress")?,
                        experience_level: int("xp_level"),
                        experience_prioritised: false,
                        locator: None,
                        attack_strength: 1.0,
                        attack_delay: 5.0,
                        attack_indicator: voxel_game::options::AttackIndicator::Off,
                        target_is_living: false,
                        hotbar: Default::default(),
                        offhand: None,
                    };
                    let mut drawn = GuiFrame {
                        log: Some(Vec::new()),
                        ..GuiFrame::default()
                    };
                    let now = step["ms"].as_u64().unwrap_or(0) as u128;
                    let size = (int("width"), int("height"));
                    hud.player_health(&mut drawn, &atlas, size.0, size.1, &input, now);
                    hud.contextual_bar(&mut drawn, &atlas, &font, size, &input);
                    frames.push(Value::Array(gui_ops(&drawn.log.take().unwrap_or_default())));
                }
                out.push(json!({"label": row["label"], "frames": frames}));
                continue;
            }
            "boss_bar" => {
                use voxel_render::gui::boss_bar::BossBars;
                let mut bars = BossBars::default();
                for (index, bar) in row["bars"].as_array().ok_or("bars")?.iter().enumerate() {
                    bars.add(
                        index as u128 + 1,
                        voxel_render::gui::chat::plain_runs(
                            bar["name"].as_str().unwrap_or_default(),
                        ),
                        bar["progress"].as_f64().unwrap_or(0.0) as f32,
                        bar["color"].as_u64().unwrap_or(0) as u8,
                        bar["overlay"].as_u64().unwrap_or(0) as u8,
                        0,
                        0,
                    );
                }
                bars.draw(&mut frame, &atlas, &font, (int("width"), int("height")), 0);
            }
            "locator_bar" => {
                use voxel_render::gui::locator::{
                    self, LocatorCamera, LocatorInput, Viewer, WaypointStyles,
                };
                let numbers = |value: &Value| -> Vec<f64> {
                    value
                        .as_array()
                        .map(|list| list.iter().filter_map(Value::as_f64).collect())
                        .unwrap_or_default()
                };
                let mut waypoints = voxel_game::waypoint::WaypointManager::default();
                for packet in row["packets"].as_array().ok_or("packets")? {
                    let hex = packet.as_str().ok_or("packet")?;
                    let bytes: Vec<u8> = (0..hex.len() / 2)
                        .map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16))
                        .collect::<Result<_, _>>()
                        .map_err(|error| error.to_string())?;
                    let (operation, waypoint) =
                        voxel_net::connection::parse_waypoint_packet(&bytes)
                            .map_err(|error| format!("{error:?}"))?;
                    waypoints.apply(operation, waypoint);
                }
                let camera = &row["camera"];
                let at = numbers(&camera["pos"]);
                let feet = numbers(&row["player"]["pos"]);
                let uuid = row["player"]["uuid"].as_array().ok_or("uuid")?;
                let half = |index: usize| uuid[index].as_i64().unwrap_or(0) as u64;
                let input = LocatorInput {
                    waypoints,
                    camera: LocatorCamera {
                        y_rot: camera["yaw"].as_f64().unwrap_or(0.0) as f32,
                        x_rot: camera["pitch"].as_f64().unwrap_or(0.0) as f32,
                        position: [at[0], at[1], at[2]],
                        fov: camera["fov"].as_f64().unwrap_or(70.0) as f32,
                        size: (camera["aspect"].as_f64().unwrap_or(1.0) as f32, 1.0),
                        z_near: 0.05,
                        z_far: camera["far"].as_f64().unwrap_or(768.0) as f32,
                    },
                    viewer: Viewer {
                        uuid: u128::from(half(0)) << 64 | u128::from(half(1)),
                        position: [feet[0], feet[1], feet[2]],
                    },
                };
                let size = (int("width"), int("height"));
                locator::background(&mut frame, &atlas, size);
                locator::draw_dots(
                    &mut frame,
                    &atlas,
                    &input,
                    &WaypointStyles::load(&root),
                    size,
                );
            }
            "tab_list" => {
                use voxel_render::gui::tab_list::{self, HealthStates, TabPlayer, TabScores};
                let runs = |key: &str| {
                    row[key]
                        .as_str()
                        .map(voxel_render::gui::chat::plain_runs)
                        .unwrap_or_default()
                };
                let board = scoreboard_from(row)?;
                let lang = voxel_game::text::Lang::load(&root);
                let displays: Vec<Option<voxel_game::text::Component>> = row["players"]
                    .as_array()
                    .ok_or("players")?
                    .iter()
                    .map(|player| {
                        player["display"]
                            .as_str()
                            .map(voxel_game::text::Component::literal)
                    })
                    .collect();
                let players = row["players"].as_array().ok_or("players")?;
                let (entries, objective) = tab_list::entries(
                    players
                        .iter()
                        .zip(&displays)
                        .map(|(player, display)| TabPlayer {
                            name: player["name"].as_str().unwrap_or_default(),
                            display_name: display.as_ref(),
                            game_mode: player["mode"].as_i64().unwrap_or(0) as i32,
                            latency: player["latency"].as_i64().unwrap_or(0) as i32,
                            list_order: player["order"].as_i64().unwrap_or(0) as i32,
                        }),
                    &board,
                    &lang,
                );
                let mut health = HealthStates::default();
                tab_list::draw(
                    &mut frame,
                    &atlas,
                    &font,
                    int("width"),
                    &entries,
                    (&runs("header"), &runs("footer")),
                    TabScores {
                        objective,
                        health: &mut health,
                        gui_ticks: 0,
                        lang: &lang,
                    },
                );
            }
            "container_screen" => {
                use voxel_render::gui::screen::Screen;
                use voxel_render::gui::screen::generic::{EnchantText, GenericScreen, Titles};
                let name = row["menu"].as_str().ok_or("menu")?;
                let mut menu = crate::menu_named(name, row)?;
                for slot in row["slots"].as_array().ok_or("slots")? {
                    let index = slot["slot"].as_u64().ok_or("slot")? as usize;
                    let item = voxel_game::item::id_of(slot["item"].as_str().ok_or("item")?)
                        .ok_or("unknown item")?;
                    let mut stack = voxel_world::ClientItemStackState::new(
                        slot["count"].as_i64().unwrap_or(1) as i32,
                        item,
                    );
                    if let Some(pages) = slot["book_pages"].as_array() {
                        stack = stack.with_patch_bytes(written_book_patch(pages));
                    }
                    menu.slots[index] = Some(stack);
                }
                // Setting the payments made the merchant's result, as the reference's slots do.
                if menu.kind == voxel_game::container_menu::MenuKind::Merchant {
                    menu.update_sell_item();
                }
                // The stonecutter's entries for the item in its slot, each
                // showing an item (which the screen draws and this run does not
                // record).
                if let (Some(inputs), Some(Some(input))) = (
                    row["stonecutter_inputs"].as_array(),
                    menu.slots
                        .first()
                        .map(|slot| slot.as_ref().map(|s| s.item_registry_id())),
                ) {
                    menu.options = inputs
                        .iter()
                        .filter(|name| {
                            name.as_str()
                                .and_then(voxel_game::item::id_of)
                                .is_some_and(|id| id == input)
                        })
                        .map(|_| 1)
                        .collect();
                }
                let lang = std::sync::Arc::new(voxel_game::text::Lang::load(&root));
                let shared_font = std::sync::Arc::new(Font::load(&root).ok_or("font")?);
                let titles = Titles {
                    title: lang.text(row["title_key"].as_str().ok_or("title_key")?),
                    inventory: lang.text("container.inventory"),
                    font: shared_font.clone(),
                    repair_cost: lang.text("container.repair.cost"),
                    repair_expensive: lang.text("container.repair.expensive"),
                    patterns: std::sync::Arc::default(),
                    lang: lang.clone(),
                    alt_font: std::sync::Arc::new(Font::load_alt(&root).ok_or("alt font")?),
                    enchantments: std::sync::Arc::new(Vec::<EnchantText>::new()),
                    recipe: std::sync::Arc::default(),
                };
                let (width, height) = (int("width"), int("height"));
                let mut screen: Box<dyn Screen> =
                    if menu.kind == voxel_game::container_menu::MenuKind::Lectern {
                        Box::new(voxel_render::gui::screen::book::BookScreen::new(
                            width,
                            height,
                            lang.clone(),
                            shared_font.clone(),
                            !matches!(row["game_mode"].as_str(), Some("adventure" | "spectator")),
                        ))
                    } else {
                        Box::new(GenericScreen::new(menu.kind, width, height, titles))
                    };
                let mouse = (f64::from(int("mouse_x")), f64::from(int("mouse_y")));
                // The smithing table's empty slots show their icons, and a beacon's buttons their state,
                // once it has ticked.
                if matches!(
                    menu.kind,
                    voxel_game::container_menu::MenuKind::Smithing
                        | voxel_game::container_menu::MenuKind::Beacon
                ) {
                    screen.tick(&menu);
                }
                let mut steps = Vec::new();
                let mut keys = vec![String::new()];
                keys.extend(
                    row["keys"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|k| k.as_str().map(str::to_owned)),
                );
                let mut item_drawer =
                    |_: &voxel_world::ClientItemStackState,
                     _: f32,
                     _: f32,
                     _: &mut Vec<voxel_render::gui::GuiItemVertex>| {};
                let mut player = |_: (i32, i32, i32, i32), _: f32, _: f32, _: (f32, f32)| {
                    Vec::<voxel_render::gui::GuiItemVertex>::new()
                };
                // A tooltip is only its first line, the item's name, which is
                // what the anvil's name box takes.
                let tooltips = row["tooltips"].as_bool().unwrap_or(true);
                let mut tooltip = |stack: &voxel_world::ClientItemStackState, _: bool| {
                    if !tooltips {
                        return Vec::new();
                    }
                    let name = voxel_data::item::registry::ITEMS[stack.item_registry_id() as usize]
                        .name
                        .trim_start_matches("minecraft:");
                    let text = lang
                        .get(&format!("item.minecraft.{name}"))
                        .or_else(|| lang.get(&format!("block.minecraft.{name}")))
                        .unwrap_or(name)
                        .to_owned();
                    vec![voxel_render::gui::chat::plain_runs(&text)]
                };
                let book = voxel_world::RecipeBook::default();
                let tags = voxel_world::block::tags::ItemTags::default();
                for key in keys {
                    if let Some(text) = key.strip_prefix("type:") {
                        for character in text.chars() {
                            screen.char_typed(character);
                        }
                    }
                    let mut drawn = GuiFrame {
                        log: Some(Vec::new()),
                        ..GuiFrame::default()
                    };
                    let mut draw = voxel_render::gui::screen::ScreenDraw {
                        frame: &mut drawn,
                        atlas: &atlas,
                        font: &font,
                        item: &mut item_drawer,
                        player: &mut player,
                        tooltip: &mut tooltip,
                        book: &book,
                        tags: &tags,
                        control_held: false,
                        partial: 0.0,
                        widget_tooltips: tooltips,
                    };
                    screen.build(&mut draw, &menu, mouse);
                    steps.push(json!({
                        "key": key,
                        "ops": gui_ops(&drawn.log.take().unwrap_or_default()),
                    }));
                }
                out.push(json!({"label": row["label"], "steps": steps}));
                continue;
            }
            _ => return Err("unknown gui operation".into()),
        }
        out.push(json!({
            "label": row["label"],
            "ops": gui_ops(&frame.log.take().unwrap_or_default()),
        }));
    }
    Ok(json!({"rows": out}))
}
