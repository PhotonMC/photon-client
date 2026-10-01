//! GUI family: the screens' drawing and input, answered with the production widgets.

#![expect(
    clippy::too_many_lines,
    reason = "the GUI family answers every screen scenario in one dispatch"
)]

use super::{Value, json};

/// The recorded drawing calls as the reference's oracle writes them.
fn gui_ops(ops: &[voxel_render::gui::DrawOp]) -> Vec<Value> {
    use voxel_render::gui::DrawOp;
    // Fills before text, then top to bottom, left to right and by what is
    // left to tell two apart: the reference layers what overlaps, an order
    // for the GPU and not for what is drawn.
    let key = |op: &&DrawOp| match op {
        DrawOp::Fill { rect, colour } => (
            0,
            rect.1,
            rect.0,
            rect.2,
            rect.3,
            String::new(),
            i64::from(*colour as i32),
        ),
        DrawOp::Sprite { rect, name } | DrawOp::Texture { rect, name, .. } => {
            (1, rect.1, rect.0, rect.2, rect.3, name.clone(), -1)
        }
        DrawOp::Text {
            x, y, colour, runs, ..
        } => (
            2,
            *y as i32,
            *x as i32,
            0,
            0,
            runs.iter().map(|run| run.0.as_str()).collect::<String>(),
            i64::from(*colour as i32),
        ),
    };
    let mut sorted: Vec<&DrawOp> = ops.iter().collect();
    sorted.sort_by_key(key);
    sorted
        .into_iter()
        .map(|op| match op {
            DrawOp::Fill { rect, colour } => json!({
                "op": "fill", "x0": f64::from(rect.0), "y0": f64::from(rect.1),
                "x1": f64::from(rect.2), "y1": f64::from(rect.3),
                "color": *colour as i32,
            }),
            DrawOp::Texture { name, rect, uv } => json!({
                "op": "texture", "name": name, "x0": f64::from(rect.0), "y0": f64::from(rect.1),
                "x1": f64::from(rect.2), "y1": f64::from(rect.3), "color": -1,
                "u0": f64::from(uv.0), "u1": f64::from(uv.1), "v0": f64::from(uv.2), "v1": f64::from(uv.3),
            }),
            DrawOp::Sprite { name, rect } => json!({
                "op": "sprite", "name": name, "x0": f64::from(rect.0), "y0": f64::from(rect.1),
                "x1": f64::from(rect.2), "y1": f64::from(rect.3), "color": -1,
            }),
            DrawOp::Text {
                x,
                y,
                scale,
                colour,
                shadow,
                runs,
            } => json!({
                "op": "text", "x": f64::from(*x), "y": f64::from(*y), "scale": f64::from(*scale), "color": *colour as i32,
                "shadow": shadow,
                "runs": runs.iter().map(|(text, rgb, bold, italic)| {
                    let mut run = json!({"text": text, "bold": bold, "italic": italic});
                    if let Some(rgb) = rgb {
                        run["rgb"] = json!(rgb);
                    }
                    run
                }).collect::<Vec<_>>(),
            }),
        })
        .collect()
}

/// A clipboard for a test with nothing to copy.
struct NoClipboard;

impl voxel_render::gui::edit_box::Clipboard for NoClipboard {
    fn get(&mut self) -> String {
        String::new()
    }

    fn set(&mut self, _text: &str) {}
}

/// The screens' drawing, recorded and compared with the reference's own.
/// A number format named in a request: `blank`, `style:<colour>` or
/// `fixed:<text>`.
fn number_format(spec: &str) -> voxel_game::scoreboard::NumberFormat {
    use voxel_game::scoreboard::NumberFormat;
    if spec == "blank" {
        NumberFormat::Blank
    } else if let Some(colour) = spec.strip_prefix("style:") {
        NumberFormat::Styled(voxel_game::text::Style::coloured(colour))
    } else {
        NumberFormat::Fixed(voxel_game::text::Component::literal(
            spec.strip_prefix("fixed:").unwrap_or_default(),
        ))
    }
}

/// The scoreboard a gui request describes, built the way the packets build
/// the game's.
fn scoreboard_from(row: &Value) -> Result<voxel_game::scoreboard::Scoreboard, String> {
    use voxel_game::scoreboard::{Scoreboard, ScoreboardEvent, TeamParameters};
    use voxel_game::text::Component;
    let literal = |value: &Value| Component::literal(value.as_str().unwrap_or_default());
    let mut board = Scoreboard::default();
    for objective in row["objectives"].as_array().into_iter().flatten() {
        board.apply(ScoreboardEvent::Objective {
            name: objective["name"]
                .as_str()
                .ok_or("objective name")?
                .to_owned(),
            method: 0,
            display_name: literal(&objective["title"]),
            hearts: objective["hearts"].as_bool().unwrap_or(false),
            number_format: objective["format"].as_str().map(number_format),
        });
    }
    for score in row["scores"].as_array().into_iter().flatten() {
        board.apply(ScoreboardEvent::Score {
            owner: score["owner"].as_str().ok_or("score owner")?.to_owned(),
            objective: score["objective"]
                .as_str()
                .ok_or("score objective")?
                .to_owned(),
            value: score["value"].as_i64().unwrap_or(0) as i32,
            display: score["display"].as_str().map(Component::literal),
            number_format: score["format"].as_str().map(number_format),
        });
    }
    if let Some(slots) = row["slots"].as_object() {
        for (slot, objective) in slots {
            board.apply(ScoreboardEvent::Display {
                slot: slot.parse().map_err(|_| "slot")?,
                objective: objective.as_str().map(str::to_owned),
            });
        }
    }
    for team in row["teams"].as_array().into_iter().flatten() {
        board.apply(ScoreboardEvent::Team {
            name: team["name"].as_str().ok_or("team name")?.to_owned(),
            method: 0,
            parameters: Some(TeamParameters {
                display_name: literal(&team["name"]),
                prefix: literal(&team["prefix"]),
                suffix: literal(&team["suffix"]),
                colour: team["color"].as_u64().map(|colour| colour as u8),
            }),
            players: team["players"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|player| player.as_str().map(str::to_owned))
                .collect(),
        });
    }
    Ok(board)
}

pub(crate) fn gui_result(request: &Value) -> Result<Value, String> {
    use voxel_render::gui::chat::ChatLog;
    use voxel_render::gui::font::Font;
    use voxel_render::gui::{GuiAtlas, GuiFrame};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/assets");
    let font = Font::load(&root).ok_or("font")?;
    let alt = Font::load_alt(&root).ok_or("alt font")?;
    let (atlas, _) = GuiAtlas::load(&root, &[&font, &alt]).ok_or("gui atlas")?;
    if request["operation"] == "command_parse" {
        return super::command_parse_result(request, &root);
    }
    if request["operation"] == "command_suggest" {
        return super::command_suggest_result(request);
    }
    let rows = request["rows"]
        .as_array()
        .ok_or("gui rows must be an array")?;
    let mut out = Vec::new();
    for row in rows {
        let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
        let mut frame = GuiFrame {
            log: Some(Vec::new()),
            ..GuiFrame::default()
        };
        match super::string(request, "operation")? {
            "chat" => {
                use voxel_render::gui::chat::Tag;
                let mut log = ChatLog::default();
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
                    let text = event["text"].as_str().ok_or("text")?;
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
                    log.add_message(
                        &voxel_render::gui::chat::plain_runs(text),
                        sig,
                        tag,
                        &font,
                        false,
                    );
                }
                while log.ticks() < row["tick"].as_u64().unwrap_or(0) {
                    log.tick(&font, &marker);
                }
                if let Some(scroll) = row["scroll"].as_i64() {
                    log.scroll_by(scroll as i32);
                }
                let focused = row["focused"].as_bool().unwrap_or(false);
                log.draw(&mut frame, &atlas, &font, int("height"), focused);
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
                let lang = voxel_game::text::Lang::load(&root);
                let size = (int("width"), int("height"));
                let screen = voxel_render::gui::pause::PauseScreen::new(&font, &lang, size);
                let mouse = (
                    row["mouse"][0].as_i64().unwrap_or(0) as i32,
                    row["mouse"][1].as_i64().unwrap_or(0) as i32,
                );
                screen.draw(&mut frame, &atlas, &font, mouse, size, true);
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
                let mut menu = super::menu_named(name, row)?;
                for slot in row["slots"].as_array().ok_or("slots")? {
                    let index = slot["slot"].as_u64().ok_or("slot")? as usize;
                    let item = voxel_game::item::id_of(slot["item"].as_str().ok_or("item")?)
                        .ok_or("unknown item")?;
                    menu.slots[index] = Some(voxel_world::ClientItemStackState::new(
                        slot["count"].as_i64().unwrap_or(1) as i32,
                        item,
                    ));
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
                let mut screen = GenericScreen::new(menu.kind, width, height, titles);
                let mouse = (f64::from(int("mouse_x")), f64::from(int("mouse_y")));
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
