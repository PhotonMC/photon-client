use super::*;

use photon_data::stat::{self, StatKey};
use std::collections::HashMap;

/// The id of a stat's value in the registry its type counts, by name.
fn value_id(stat_type: &str, name: &str) -> Result<i32, String> {
    let id = match stat::value_registry(
        stat::type_names()
            .iter()
            .position(|candidate| *candidate == stat_type)
            .ok_or("stat type")? as i32,
    ) {
        Some(stat::ValueRegistry::Custom) => stat::custom_names().iter().position(|c| *c == name),
        Some(stat::ValueRegistry::Block) => photon_data::block::registry::block_index(name),
        Some(stat::ValueRegistry::Item) => photon_data::item::registry::index(name),
        Some(stat::ValueRegistry::EntityType) => {
            photon_data::entity_type::id(name).map(|id| id.0 as usize)
        }
        None => None,
    };
    id.map(|id| id as i32)
        .ok_or_else(|| format!("unknown stat value {name}"))
}

fn vanilla_registries() -> photon_game::registries::Registries {
    photon_game::registries::Registries::vanilla(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.data/minecraft/assets/jar/data"),
    )
}

/// A stack's tooltip as the screen asks for it: with a player, in the default language.
fn default_tooltip<'a>(
    lang: &'a photon_game::text::Lang,
    registries: &'a photon_game::registries::Registries,
) -> impl FnMut(&photon_world::ClientItemStackState, bool) -> Vec<Vec<photon_game::text::Run>> + 'a
{
    move |stack, _| {
        photon_game::tooltip::lines(
            stack,
            true,
            photon_game::tooltip::Flags::default(),
            registries,
            photon_game::tooltip::Context::EMPTY,
        )
        .iter()
        .map(|line| lang.runs(line, &photon_game::text::Style::default()))
        .collect()
    }
}

fn int_at(value: &Value, index: usize) -> i32 {
    value[index].as_i64().unwrap_or(0) as i32
}

/// The counter a row lists its stats as.
fn counter_from(row: &Value) -> Result<HashMap<StatKey, i32>, String> {
    let mut counter = HashMap::new();
    for stat in row["stats"].as_array().ok_or("stats")? {
        let stat_type = stat[0].as_str().ok_or("stat type")?;
        let type_id = stat::type_names()
            .iter()
            .position(|candidate| *candidate == stat_type)
            .ok_or("stat type")? as i32;
        let key = StatKey {
            stat_type: type_id,
            value: value_id(stat_type, stat[1].as_str().ok_or("stat value")?)?,
        };
        counter.insert(key, stat[2].as_i64().ok_or("stat count")? as i32);
    }
    Ok(counter)
}

/// The screen drawn once, with no items, and the log of what was drawn.
fn draw_screen(
    ctx: &GuiContext,
    lang: &photon_game::text::Lang,
    (screen, mouse): (&photon_gui::screen::stats::StatsScreen, (i32, i32)),
    now: u64,
) -> Vec<photon_render::gui::DrawOp> {
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    let registries = vanilla_registries();
    let mut tooltip = default_tooltip(lang, &registries);
    let mut item = no_items;
    let mut player = |_: (i32, i32, i32, i32), _: f32, _: f32, _: (f32, f32)| {
        photon_render::gui::GuiFigure::default()
    };
    let mut entity =
        |_: EntityId, _: (i32, i32, i32, i32), _: f32, _: f32, _: (f32, f32)| Vec::new();
    let (book, tags) = (
        photon_world::RecipeBook::default(),
        photon_world::block::tags::ItemTags::default(),
    );
    let mut draw = photon_gui::screen::ScreenDraw {
        viewer: photon_gui::screen::PlayerView::default(),
        frame: &mut frame,
        atlas: &ctx.atlas,
        font: &ctx.font,
        item: &mut item,
        player: &mut player,
        entity: &mut entity,
        tooltip: &mut tooltip,
        lang,
        book: &book,
        tags: &tags,
        control_held: false,
        partial: 0.0,
        widget_tooltips: true,
    };
    screen.draw(&mut draw, mouse, now);
    frame.log.take().unwrap_or_default()
}

/// `stats_screen`: the screen over a counter, loading or loaded, after its steps.
pub(super) fn stats_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_gui::screen::stats::StatsScreen;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let lang = photon_game::text::Lang::load(&ctx.root);
    let counter = counter_from(row)?;
    let size = (int("width"), int("height"));
    let mut screen = StatsScreen::new(&lang, size, true, &counter);
    let mouse = (int_at(&row["mouse"], 0), int_at(&row["mouse"], 1));
    let mut now = row["now"].as_u64().unwrap_or(0);
    if row["loaded"].as_bool().unwrap_or(false) {
        screen.on_stats_updated(&lang, &counter);
    }
    for step in row["steps"].as_array().ok_or("steps")? {
        let at = (
            step["x"].as_f64().unwrap_or(0.0) as i32,
            step["y"].as_f64().unwrap_or(0.0) as i32,
        );
        match step["do"].as_str().ok_or("step")? {
            "click" => {
                screen.clicked(at);
            }
            "key" => {
                screen.key_pressed(
                    step["key"].as_i64().ok_or("key")? as i32,
                    step["modifiers"].as_i64().ok_or("modifiers")? as i32,
                );
            }
            "scroll" => screen.scrolled(at, step["delta"].as_f64().ok_or("delta")?),
            // A frame drawn and thrown away: a tooltip's wait begins in one.
            "frame" => {
                draw_screen(ctx, &lang, (&screen, mouse), now);
            }
            "wait" => now += step["ms"].as_u64().ok_or("ms")?,
            "resize" => screen.resize((
                step["width"].as_i64().unwrap_or(0) as i32,
                step["height"].as_i64().unwrap_or(0) as i32,
            )),
            _ => return Err("unknown step".into()),
        }
    }
    let log = draw_screen(ctx, &lang, (&screen, mouse), now);
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&log),
    }))
}

/// One advancement from its JSON (the shape the oracle builds the game's from).
pub(super) fn advancement_from(
    json: &Value,
) -> Result<photon_game::advancements::Advancement, String> {
    use photon_game::advancements::{Advancement, AdvancementType, DisplayInfo};
    use photon_game::text::Component;
    let display = match json.get("display").filter(|display| !display.is_null()) {
        None => None,
        Some(shown) => Some(DisplayInfo {
            title: Component::literal(shown["title"].as_str().ok_or("title")?),
            description: Component::literal(shown["description"].as_str().ok_or("description")?),
            icon: (
                crate::case_ids::item(1),
                1,
                photon_game::item_components::Patch {
                    added: Vec::new(),
                    removed: Vec::new(),
                },
            ),
            frame: AdvancementType::from_ordinal(shown["frame"].as_i64().ok_or("frame")? as i32)
                .ok_or("frame")?,
            show_toast: shown["show_toast"].as_bool().ok_or("show_toast")?,
            hidden: shown["hidden"].as_bool().ok_or("hidden")?,
            background: shown
                .get("background")
                .and_then(Value::as_str)
                .map(str::to_owned),
            x: crate::f32_field(shown, "x")?,
            y: crate::f32_field(shown, "y")?,
        }),
    };
    let requirements = json["requirements"]
        .as_array()
        .ok_or("requirements")?
        .iter()
        .map(|group| {
            group
                .as_array()
                .map(|names| {
                    names
                        .iter()
                        .filter_map(|name| name.as_str().map(str::to_owned))
                        .collect()
                })
                .ok_or("group")
        })
        .collect::<Result<Vec<Vec<String>>, _>>()?;
    Ok(Advancement {
        id: json["id"].as_str().ok_or("id")?.to_owned(),
        parent: json["parent"].as_str().map(str::to_owned),
        display,
        requirements,
        sends_telemetry_event: false,
    })
}

/// An update from its JSON.
pub(super) fn update_from(
    step: &Value,
) -> Result<photon_game::advancements::UpdateAdvancements, String> {
    let strings = |key: &str| -> Result<Vec<String>, String> {
        step[key]
            .as_array()
            .ok_or_else(|| key.to_owned())?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| key.to_owned())
            })
            .collect()
    };
    let added = step["added"]
        .as_array()
        .ok_or("added")?
        .iter()
        .map(advancement_from)
        .collect::<Result<Vec<_>, _>>()?;
    let progress = step["progress"]
        .as_array()
        .ok_or("progress")?
        .iter()
        .map(|entry| {
            let criteria = entry["criteria"]
                .as_array()
                .ok_or("criteria")?
                .iter()
                .map(|pair| Ok((pair[0].as_str().ok_or("name")?.to_owned(), pair[1].as_i64())))
                .collect::<Result<Vec<_>, String>>()?;
            Ok((entry["id"].as_str().ok_or("id")?.to_owned(), criteria))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(photon_game::advancements::UpdateAdvancements {
        reset: step["reset"].as_bool().ok_or("reset")?,
        added,
        removed: strings("removed")?,
        progress,
        show_advancements: step["show"].as_bool().ok_or("show")?,
    })
}

/// `client_advancements`: the client's copy over a script of updates and tab choices.
pub(super) fn client_advancements_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_game::advancements::ClientAdvancements;
    let lang = photon_game::text::Lang::load(&ctx.root);
    let mut advancements = ClientAdvancements::default();
    let mut states = Vec::new();
    for step in row["steps"].as_array().ok_or("steps")? {
        let mut toasts = Vec::new();
        if let Some(tab) = step.get("tab") {
            // The tab is looked up in the tree first: an id it lacks is no tab.
            let known = tab.as_str().filter(|id| advancements.get(id).is_some());
            advancements.select_tab(known);
        } else {
            toasts = advancements
                .update(&update_from(step)?)
                .toasts
                .into_iter()
                .map(|(id, _)| id)
                .collect();
        }
        toasts.sort();
        states.push(advancements_state(&advancements, &lang, toasts));
    }
    Ok(crate::hex::without_nulls(
        json!({"label": row["label"], "states": states}),
    ))
}

/// What the client's copy holds after a step, as the oracle reports it.
fn advancements_state(
    advancements: &photon_game::advancements::ClientAdvancements,
    lang: &photon_game::text::Lang,
    toasts: Vec<String>,
) -> Value {
    let mut nodes: Vec<&photon_game::advancements::Advancement> =
        advancements.roots().chain(advancements.tasks()).collect();
    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    let node_list: Vec<Value> = nodes
        .iter()
        .map(|node| {
            let mut children: Vec<&str> = advancements
                .child_ids(&node.id)
                .iter()
                .map(String::as_str)
                .collect();
            children.sort_unstable();
            json!({"id": node.id, "parent": node.parent, "children": children})
        })
        .collect();
    let progress_list: Vec<Value> = advancements
        .progress_ids()
        .into_iter()
        .filter_map(|id| progress_state(advancements, lang, id))
        .collect();
    json!({
        "roots": advancements.roots().map(|root| root.id.clone()).collect::<Vec<_>>(),
        "tasks": advancements.tasks().map(|task| task.id.clone()).collect::<Vec<_>>(),
        "nodes": node_list,
        "progress": progress_list,
        "selected": advancements.selected_tab(),
        "toasts": toasts,
    })
}

/// One advancement's progress: the percent, its text, and the criteria done and left.
fn progress_state(
    advancements: &photon_game::advancements::ClientAdvancements,
    lang: &photon_game::text::Lang,
    id: &str,
) -> Option<Value> {
    use photon_game::text::Component;
    let progress = advancements.progress(id)?;
    let text = progress.progress_counts().map(|(done, total)| {
        lang.plain(&Component::translatable_with(
            "advancements.progress",
            vec![
                Component::literal(done.to_string()),
                Component::literal(total.to_string()),
            ],
        ))
    });
    let (completed, remaining): (Vec<_>, Vec<_>) =
        progress.criteria().into_iter().partition(|(_, done)| *done);
    let names = |list: Vec<(&str, bool)>| -> Vec<String> {
        list.into_iter().map(|(name, _)| name.to_owned()).collect()
    };
    Some(json!({
        "id": id,
        "percent": format!("{:08x}", progress.percent().to_bits()),
        "text": text,
        "done": progress.is_done(),
        "started": progress.has_progress(),
        "remaining": names(remaining),
        "completed": names(completed),
    }))
}
