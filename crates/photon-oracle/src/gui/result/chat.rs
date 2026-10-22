use super::*;
use photon_gui::FrameRuns;

use photon_game::chat_abilities::{ChatAbilities, ChatRestriction, MessageSource};
use photon_gui::chat::{ChatLog, ChatSettings};

/// The abilities a row's restriction names leave.
pub(super) fn abilities(names: &Value) -> Result<ChatAbilities, String> {
    names
        .as_array()
        .into_iter()
        .flatten()
        .map(|name| match name.as_str() {
            Some("CHAT_AND_COMMANDS_DISABLED_BY_OPTIONS") => {
                Ok(ChatRestriction::ChatAndCommandsDisabledByOptions)
            }
            Some("CHAT_DISABLED_BY_OPTIONS") => Ok(ChatRestriction::ChatDisabledByOptions),
            Some("DISABLED_BY_LAUNCHER") => Ok(ChatRestriction::DisabledByLauncher),
            Some("DISABLED_BY_PROFILE") => Ok(ChatRestriction::DisabledByProfile),
            other => Err(format!("restriction {other:?}")),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(ChatAbilities::of)
}

/// The command tree of the reference's player: nothing in it, or the one command `say`, which
/// takes a message.
fn commands_of(say: bool) -> photon_game::commands::CommandTree {
    use photon_game::commands::{ArgumentKind, CommandTree, NodeKind, NodeStub};
    let stub = |kind, children: Vec<usize>, executable| NodeStub {
        kind,
        children,
        redirect: None,
        executable,
        custom_suggestions: false,
        restricted: false,
        type_id: 0,
    };
    let mut nodes = vec![stub(NodeKind::Root, Vec::new(), false)];
    if say {
        nodes[0].children = vec![1];
        nodes.push(stub(NodeKind::Literal("say".into()), vec![2], false));
        let message = NodeKind::Argument {
            name: "message".into(),
            kind: ArgumentKind::Greedy,
        };
        nodes.push(stub(message, Vec::new(), true));
    }
    CommandTree::build(nodes, 0).expect("a tree of nodes")
}

/// The chat options a row sets, before the messages and again after them (which wraps them anew).
fn chat_settings(options: &Value, base: ChatSettings) -> ChatSettings {
    let option = |key: &str, default: f64| options[key].as_f64().unwrap_or(default);
    ChatSettings {
        scale: option("scale", base.scale),
        opacity: option("opacity", base.opacity),
        width: option("width", base.width),
        height_focused: option("height_focused", base.height_focused),
        height_unfocused: option("height_unfocused", base.height_unfocused),
        line_spacing: option("line_spacing", base.line_spacing),
        background_opacity: option("background_opacity", base.background_opacity),
    }
}

/// The chat options a row sets under `key`: `chatColors`, which a later set keeps when it says
/// nothing of it, and the shape of the chat.
fn apply_options(log: &mut ChatLog, row: &Value, key: &str, font: &Font) {
    if let Some(colors) = row[key]["colors"].as_bool() {
        log.set_colors(colors);
    }
    log.set_settings(chat_settings(&row[key], log.settings()), font);
}

/// `rescaleChat` after the options a row names under `rescale`.
fn apply_rescale(log: &mut ChatLog, row: &Value, font: &Font) {
    if row["rescale"].is_object() {
        apply_options(log, row, "rescale", font);
        log.rescale(font);
    }
}

/// The text a message event carries: a translation key, a JSON component or plain text.
fn event_component(event: &Value) -> Result<photon_game::text::Component, String> {
    use photon_game::text::Component;
    if let Some(key) = event["key"].as_str() {
        return Ok(Component::translatable(key));
    }
    if let Some(json) = event["json"].as_str() {
        let parsed = photon_core::json::parse(json).map_err(|error| format!("{error:?}"))?;
        return Component::from_json(&parsed).ok_or_else(|| "component".to_owned());
    }
    Ok(Component::literal(event["text"].as_str().ok_or("text")?))
}

/// The tag a message event names and where it came from: a message with neither a tag nor a
/// signature is the server's system message.
fn event_origin(event: &Value, signed: bool) -> (MessageSource, Option<photon_gui::chat::Tag>) {
    use photon_gui::chat::Tag;
    let tag = match event["tag"].as_str() {
        None if !signed => Some(Tag::SystemSinglePlayer),
        None => None,
        Some("system") => Some(Tag::System),
        Some("system_single_player") => Some(Tag::SystemSinglePlayer),
        Some("not_secure") => Some(Tag::NotSecure),
        Some("error") => Some(Tag::Error),
        Some(other) => Some(Tag::Modified(
            other.trim_start_matches("modified:").to_owned(),
        )),
    };
    let origin = if event["tag"].is_null() && !signed {
        MessageSource::SystemServer
    } else {
        MessageSource::Player
    };
    (origin, tag)
}

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
    let mut log = ChatLog::default();
    apply_options(&mut log, row, "options", font);
    if row["restrictions"].is_array() {
        log.set_abilities(abilities(&row["restrictions"])?, font);
    }
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
        let component = event_component(event)?;
        let sig = event["signature"].as_u64().map(signature);
        let (origin, tag) = event_origin(event, sig.is_some());
        log.add_component(&component, &lang, (origin, sig, tag), font, false);
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
    apply_rescale(&mut log, row, font);
    if row["restrictions_after"].is_array() {
        log.set_abilities(abilities(&row["restrictions_after"])?, font);
    }
    let focused = row["focused"].as_bool().unwrap_or(false);
    log.set_queue_size(row["queued"].as_u64().unwrap_or(0));
    if let Some(scroll) = row["scroll"].as_i64() {
        log.scroll_by(scroll as i32, focused);
    }
    let gui = (int("width"), int("height"));
    // The mouse over the open chat: the tooltip of what is under it.
    let mouse = row["mouse"].as_array().map(|mouse| {
        (
            mouse[0].as_i64().unwrap_or(0) as i32,
            mouse[1].as_i64().unwrap_or(0) as i32,
        )
    });
    let tooltip = log.draw_with(&mut frame, atlas, font, gui, focused, mouse, &lang);
    if let (Some(lines), Some(mouse)) = (tooltip, mouse) {
        frame.tooltip(atlas, font, &lines, mouse, gui);
    }

    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

/// The key a suggestion row presses, and whether Shift is down.
fn suggestion_key(name: &str) -> Result<(glfw::Key, bool), String> {
    match name {
        "tab" => Ok((glfw::Key::Tab, false)),
        "shift+tab" => Ok((glfw::Key::Tab, true)),
        "up" => Ok((glfw::Key::Up, false)),
        "down" => Ok((glfw::Key::Down, false)),
        "escape" => Ok((glfw::Key::Escape, false)),
        _ => Err("unknown key".into()),
    }
}

/// The key a draft row presses.
fn draft_key(name: &str) -> Result<glfw::Key, String> {
    Ok(match name {
        "backspace" => glfw::Key::Backspace,
        "left" => glfw::Key::Left,
        "right" => glfw::Key::Right,
        "home" => glfw::Key::Home,
        "end" => glfw::Key::End,
        "delete" => glfw::Key::Delete,
        "up" => glfw::Key::Up,
        "down" => glfw::Key::Down,
        "enter" => glfw::Key::Enter,
        other => return Err(format!("unknown key {other}")),
    })
}

fn close_reason(name: &str) -> photon_gui::chat::ExitReason {
    use photon_gui::chat::ExitReason;
    match name {
        "intentional" => ExitReason::Intentional,
        "interrupted" => ExitReason::Interrupted,
        _ => ExitReason::Done,
    }
}

/// The ops a chat box draws, focused, with the command tree.
fn draw_box(
    screen: &mut photon_gui::chat::ChatScreen,
    atlas: &GuiAtlas,
    tree: &photon_game::commands::CommandTree,
    lang: &photon_game::text::Lang,
) -> Vec<Value> {
    let mut drawn = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    screen.refocus();
    screen.draw(&mut drawn, atlas, (0, 0), (Some(tree), lang));
    gui_ops(&drawn.log.take().unwrap_or_default())
}

pub(super) fn suggestions_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    use photon_gui::chat::ChatScreen;
    let lang = photon_game::text::Lang::load(root);
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
            let (glfw_key, shift) = suggestion_key(&key)?;
            screen.key_pressed(glfw_key, (false, shift), &mut NoClipboard);
        }
        let mut drawn = GuiFrame {
            log: Some(Vec::new()),
            ..GuiFrame::default()
        };
        screen.refocus();
        screen.draw(&mut drawn, atlas, (0, 0), (None, &lang));
        steps.push(json!({
            "key": key,
            "value": screen.value(),
            "cursor": screen.cursor(),
            "ops": gui_ops(&drawn.log.take().unwrap_or_default()),
        }));
    }
    Ok(json!({"label": row["label"], "steps": steps}))
}

/// The box an `open` event makes: its method's draft restored, and the leave-bed button for a
/// `bed` event.
fn open_draft_screen(
    event: &Value,
    drafts: &mut photon_gui::chat::ChatDrafts,
    font: &std::sync::Arc<Font>,
    size: (i32, i32),
    root: &std::path::Path,
) -> photon_gui::chat::ChatScreen {
    use photon_gui::chat::{ChatMethod, ChatScreen};
    let method = if event["method"] == "command" {
        ChatMethod::Command
    } else {
        ChatMethod::Message
    };
    let (initial, draft) = drafts.open(method);
    let opened = ChatScreen::new(font.clone(), size, &initial, Vec::new()).without_key();
    let opened = if event["bed"].as_bool().unwrap_or(false) {
        opened.in_bed(&photon_game::text::Lang::load(root))
    } else {
        opened
    };
    if draft { opened.as_draft() } else { opened }
}

pub(super) fn chat_draft_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    use photon_gui::chat::{ChatDrafts, ChatMethod, ChatScreen};
    let (width, height) = (int("width"), int("height"));
    let save = row["save"].as_bool().unwrap_or(false);
    let font = std::sync::Arc::new(Font::load(root).ok_or("font")?);
    let lang = photon_game::text::Lang::load(root);
    let allowed = abilities(&row["restrictions"])?;
    let say = row["say"].as_bool().unwrap_or(false);
    let mut drafts = ChatDrafts::default();
    let mut screen: Option<ChatScreen> = None;
    let mut events = Vec::new();
    for event in row["events"].as_array().ok_or("events")? {
        let mut entry = json!({"do": event["do"]});
        match event["do"].as_str().unwrap_or_default() {
            "open" => {
                let mut opened =
                    open_draft_screen(event, &mut drafts, &font, (width, height), root);
                opened.set_restrictions(allowed.can_send_messages(), allowed.can_send_commands());
                // The reference's player has a command dispatcher with no commands in it.
                opened.set_commands(Some(std::sync::Arc::new(commands_of(say))));
                screen = Some(opened);
            }
            "type" => {
                for character in event["text"].as_str().unwrap_or_default().chars() {
                    screen.as_mut().ok_or("no box")?.char_typed(character);
                }
            }
            "key" => {
                let key = draft_key(event["key"].as_str().unwrap_or_default())?;
                screen
                    .as_mut()
                    .ok_or("no box")?
                    .key_pressed(key, (false, false), &mut NoClipboard);
            }
            "close" => {
                let closed = screen.take().ok_or("no box")?;
                let reason = close_reason(event["reason"].as_str().unwrap_or_default());
                drafts.removed(&closed.value(), closed.is_draft(), reason, save);
            }
            "draw" => {
                let shown = screen.as_mut().ok_or("no box")?;
                entry["ops"] = json!(draw_box(shown, atlas, &commands_of(say), &lang));
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
