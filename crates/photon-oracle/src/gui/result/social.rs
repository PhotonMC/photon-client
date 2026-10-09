use super::*;

use photon_core::binary::Uuid;
use photon_game::text::Component;
use photon_render::gui::screen::social::{
    Page, Roster, SocialEvent, SocialInteractionsScreen, SocialPlayer,
};

/// A `UUID.toString` text as an id.
fn uuid(text: &str) -> Result<Uuid, String> {
    let digits: String = text.chars().filter(|c| *c != '-').collect();
    let value = u128::from_str_radix(&digits, 16).map_err(|_| format!("a uuid: {text}"))?;
    Ok(Uuid::new((value >> 64) as u64, value as u64))
}

fn players(list: &Value) -> Result<Vec<SocialPlayer>, String> {
    list.as_array()
        .ok_or("players")?
        .iter()
        .map(|player| {
            Ok(SocialPlayer {
                id: uuid(player["id"].as_str().ok_or("id")?)?,
                name: player["name"].as_str().unwrap_or_default().to_owned(),
                recent_messages: player["recent"].as_bool().unwrap_or(false),
                draft_report: player["draft"].as_bool().unwrap_or(false),
            })
        })
        .collect()
}

/// `social_interactions_screen`: the player list over a server of the row's players, some
/// hidden or gone, on one of its pages, after a search was typed.
pub(super) fn social_interactions_screen_case(
    ctx: &mut GuiContext,
    row: &Value,
) -> Result<Value, String> {
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let lang = photon_game::text::Lang::load(&ctx.root);
    let roster = Roster {
        server: row["server"].as_str().unwrap_or_default().to_owned(),
        online: players(&row["players"])?,
        gone: players(&row["gone"])?,
        hidden: row["hidden"]
            .as_array()
            .ok_or("hidden")?
            .iter()
            .map(|id| uuid(id.as_str().unwrap_or_default()))
            .collect::<Result<_, _>>()?,
        local: uuid(row["local"].as_str().ok_or("local")?)?,
        reporting: row["reporting"].as_bool().unwrap_or(false),
    };
    let page = match row["page"].as_str() {
        Some("hidden") => Page::Hidden,
        Some("blocked") => Page::Blocked,
        _ => Page::All,
    };
    let mut screen = SocialInteractionsScreen::new(
        &ctx.font,
        &lang,
        (int("width"), int("height")),
        roster,
        page,
    );
    screen.type_search(row["search"].as_str().unwrap_or_default(), &ctx.font);
    let mouse = (
        row["mouse"][0].as_i64().unwrap_or(0) as i32,
        row["mouse"][1].as_i64().unwrap_or(0) as i32,
    );
    // The frames before the one recorded: the blocking hint shows only in the first.
    for _ in 1..row["frames"].as_u64().unwrap_or(1) {
        screen.draw(&mut GuiFrame::default(), &ctx.atlas, &ctx.font, mouse);
    }
    // The steps, each followed by a frame, and what the screen asked of the game.
    let mut events = Vec::new();
    let mut chat = Vec::new();
    for step in row["steps"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        let at = (
            step["x"].as_f64().unwrap_or(0.0),
            step["y"].as_f64().unwrap_or(0.0),
        );
        let dy = step["dy"].as_f64().unwrap_or(0.0);
        let asked = match step["do"].as_str().ok_or("do")? {
            "click" => screen.clicked(at, &ctx.font),
            "release" => {
                screen.released();
                None
            }
            "drag" => {
                screen.dragged(at, dy);
                None
            }
            "scroll" => {
                screen.scrolled(at, dy);
                None
            }
            "type" => {
                screen.type_search(step["text"].as_str().unwrap_or_default(), &ctx.font);
                None
            }
            "key" => {
                let code = step["key"].as_i64().ok_or("key")?;
                let key = match code {
                    80 => glfw::Key::P,
                    256 => glfw::Key::Escape,
                    259 => glfw::Key::Backspace,
                    _ => return Err(format!("a key: {code}")),
                };
                screen.key_pressed(
                    key,
                    (false, false),
                    code == 80,
                    &ctx.font,
                    &mut crate::gui::NoClipboard,
                )
            }
            other => return Err(format!("a step: {other}")),
        };
        match asked {
            Some(SocialEvent::Close) => events.push("close".to_owned()),
            Some(SocialEvent::BlockingHelp) => events.push("link".to_owned()),
            Some(SocialEvent::Hidden { name, hidden, .. }) => {
                let key = if hidden {
                    "gui.socialInteractions.hidden_in_chat"
                } else {
                    "gui.socialInteractions.shown_in_chat"
                };
                let message = Component::translatable_with(key, vec![Component::literal(name)]);
                let text: String = lang
                    .runs(&message, &photon_game::text::Style::default())
                    .iter()
                    .map(|run| run.text.as_str())
                    .collect();
                chat.push(format!("chat:{text}"));
            }
            Some(SocialEvent::Report(_)) | None => {}
        }
        screen.draw(&mut GuiFrame::default(), &ctx.atlas, &ctx.font, mouse);
    }
    events.extend(chat);
    let mut frame = GuiFrame {
        log: Some(Vec::new()),
        ..GuiFrame::default()
    };
    screen.draw(&mut frame, &ctx.atlas, &ctx.font, mouse);
    Ok(json!({
        "label": row["label"],
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
        "events": events,
    }))
}
