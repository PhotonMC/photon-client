use super::*;

pub(super) fn tab_list_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let root = &ctx.root;
    let font = &ctx.font;
    let atlas = &ctx.atlas;
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    use photon_gui::hud::tab_list::{self, HealthStates, TabScores};
    let runs = |key: &str| {
        row[key]
            .as_str()
            .map(photon_gui::chat::plain_runs)
            .unwrap_or_default()
    };
    let board = scoreboard_from(row)?;
    let lang = photon_game::text::Lang::load(root);
    let displays: Vec<Option<photon_game::text::Component>> = row["players"]
        .as_array()
        .ok_or("players")?
        .iter()
        .map(|player| {
            player["display"]
                .as_str()
                .map(photon_game::text::Component::literal)
        })
        .collect();
    let players = row["players"].as_array().ok_or("players")?;
    let (entries, objective) = tab_list::entries(
        players
            .iter()
            .zip(&displays)
            .map(|(player, display)| row_player(player, display.as_ref())),
        &board,
        &lang,
    );
    let mut health = HealthStates::default();
    let no_faces = std::collections::HashMap::new();
    let mut scores = TabScores::plain(objective, &mut health, &lang);
    scores.heads = row["online_mode"]
        .as_bool()
        .unwrap_or(false)
        .then_some(&no_faces);
    tab_list::draw(
        &mut frame,
        (atlas, font),
        int("width"),
        &entries,
        (&runs("header"), &runs("footer")),
        scores,
    );

    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

/// The id a tab list row gives a player (all zeros where it gives none).
fn row_uuid(player: &Value) -> photon_core::binary::Uuid {
    let digits: String = player["id"]
        .as_str()
        .unwrap_or_default()
        .chars()
        .filter(|c| *c != '-')
        .collect();
    let value = u128::from_str_radix(&digits, 16).unwrap_or(0);
    photon_core::binary::Uuid::new((value >> 64) as u64, value as u64)
}

/// A player of a tab list row.
fn row_player<'a>(
    player: &'a Value,
    display_name: Option<&'a photon_game::text::Component>,
) -> photon_gui::hud::tab_list::TabPlayer<'a> {
    photon_gui::hud::tab_list::TabPlayer {
        id: row_uuid(player),
        hat: player["hat"].as_bool().unwrap_or(true),
        flip: player["flip"].as_bool().unwrap_or(false),
        name: player["name"].as_str().unwrap_or_default(),
        display_name,
        game_mode: player["mode"].as_i64().unwrap_or(0) as i32,
        latency: player["latency"].as_i64().unwrap_or(0) as i32,
        list_order: player["order"].as_i64().unwrap_or(0) as i32,
    }
}
