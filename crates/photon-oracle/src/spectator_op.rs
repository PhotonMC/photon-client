//! The spectator menu's paging and selection, stepped as the parity rows say.

use photon_game::spectator::{Action, Candidate, Sources, SpectatorGui, SpectatorMenu};
use photon_game::text::{Component, Content};
use serde_json::{Value, json};

/// A component's translation key, or its plain text.
fn key(component: &Component) -> String {
    match &component.content {
        Content::Literal(text) => text.clone(),
        Content::Translatable { key, .. } => key.clone(),
    }
}

fn hex(row: &Value, key: &str) -> Result<u64, String> {
    row.get(key)
        .and_then(Value::as_str)
        .and_then(|text| u64::from_str_radix(text, 16).ok())
        .ok_or_else(|| format!("a player has a hex {key}"))
}

fn candidates(row: &Value) -> Result<Vec<Candidate>, String> {
    let players = row
        .get("players")
        .and_then(Value::as_array)
        .ok_or("a row has players")?;
    players
        .iter()
        .map(|spec| {
            Ok(Candidate {
                id: (u128::from(hex(spec, "hi")?) << 64) | u128::from(hex(spec, "lo")?),
                name: spec["name"]
                    .as_str()
                    .ok_or("a player has a name")?
                    .to_owned(),
                game_mode: spec["mode"].as_i64().ok_or("a player has a mode")? as i32,
            })
        })
        .collect()
}

fn describe(menu: &SpectatorMenu, teleport: Option<&str>) -> Value {
    let slots: Vec<Value> = menu
        .items()
        .iter()
        .map(|item| json!({"name": key(&item.name()), "enabled": item.is_enabled()}))
        .collect();
    let mut step = json!({
        "selected": menu.selected_slot(),
        "prompt": menu.prompt(),
        "items": slots,
    });
    if let Some(name) = teleport {
        step["teleport"] = json!(name);
    }
    step
}

pub(super) fn spectator_menu(label: &str, row: &Value) -> Result<Value, String> {
    let players = candidates(row)?;
    let sources = Sources {
        players: players.clone(),
        teams: Vec::new(),
    };
    let mut gui = SpectatorGui::default();
    // The first press opens the menu; the reference's rows start with it open.
    gui.on_hotbar_selected(0, 0, &sources);
    let steps = row
        .get("steps")
        .and_then(Value::as_array)
        .ok_or("a row has steps")?;
    let mut out = Vec::new();
    for step in steps {
        let step = step.as_str().ok_or("a step is a string")?;
        let (kind, number) = step.split_once(':').ok_or("a step is kind:number")?;
        let number: i32 = number.parse().map_err(|_| "a step number")?;
        let mut name = None;
        if kind == "select" {
            if let Some(Action::Teleport(id)) = gui.on_hotbar_selected(number, 0, &sources) {
                name = players.iter().find(|p| p.id == id).map(|p| p.name.clone());
            }
        } else {
            gui.on_mouse_scrolled(number, 0);
        }
        let menu = gui.menu().ok_or("the menu stays open")?;
        out.push(describe(menu, name.as_deref()));
    }
    Ok(json!({"label": label, "outcome": "stepped", "steps": out}))
}
