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

/// `stats_screen`: the screen over a counter, loading or loaded, after its steps.
pub(super) fn stats_screen_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    use photon_gui::screen::stats::StatsScreen;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let lang = photon_game::text::Lang::load(&ctx.root);
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
    let size = (int("width"), int("height"));
    let mut screen = StatsScreen::new(&lang, size, true, &counter);
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
            "scroll" => screen.scrolled(at, step["delta"].as_f64().ok_or("delta")?),
            _ => return Err("unknown step".into()),
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
    screen.draw(
        &mut frame,
        (&ctx.atlas, &ctx.font),
        mouse,
        row["now"].as_u64().unwrap_or(0),
    );
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}
