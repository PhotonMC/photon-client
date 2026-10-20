//! `debug_entries`: the debug screen's entries, the profiles, and an entry list put through
//! operations with the file it saves.

use photon_gui::hud::debug::{
    ENTRIES, EntryList, Profile, Status, listed_entries, parse_profile, profile_text,
};
use serde_json::{Map, Value, json};

fn status_of(name: &str) -> Result<Status, String> {
    match name {
        "ALWAYS_ON" => Ok(Status::AlwaysOn),
        "IN_OVERLAY" => Ok(Status::InOverlay),
        "NEVER" => Ok(Status::Never),
        other => Err(format!("unknown status {other}")),
    }
}

fn profile_of(name: &str) -> Result<Profile, String> {
    Profile::from_name(&name.to_lowercase()).ok_or_else(|| format!("unknown profile {name}"))
}

/// The entries registered and the profiles, as the screen reads them.
fn registry() -> (Value, Value) {
    let mut entries: Vec<_> = ENTRIES.iter().collect();
    entries.sort_by_key(|entry| entry.id);
    let registry = entries
        .into_iter()
        .map(|entry| {
            json!({
                "id": entry.id,
                "sort_key": format!("{:08x}", entry.category.sort_key().to_bits()),
                "category": entry.category.translation_key(),
                "allowed": true,
                "allowed_reduced": entry.allowed_when_reduced,
            })
        })
        .collect();
    let profiles: Map<String, Value> = Profile::ALL
        .into_iter()
        .map(|profile| {
            let mut statuses: Vec<_> = profile.statuses().to_vec();
            statuses.sort_by_key(|(id, _)| *id);
            let map: Map<String, Value> = statuses
                .into_iter()
                .map(|(id, status)| (id.to_owned(), json!(status.name())))
                .collect();
            (profile.name().to_owned(), Value::Object(map))
        })
        .collect();
    (Value::Array(registry), Value::Object(profiles))
}

/// What a list and the file it saved look like after an operation.
fn state(list: &EntryList, reduced: bool, file: Option<&str>) -> Value {
    let statuses: Map<String, Value> = {
        let mut ids: Vec<_> = ENTRIES.iter().map(|entry| entry.id).collect();
        ids.sort_unstable();
        ids.into_iter()
            .map(|id| (id.to_owned(), json!(list.status(id).name())))
            .collect()
    };
    let mut state = json!({
        "statuses": statuses,
        "enabled": list.enabled(reduced),
        "using_default": list.is_using_profile(Profile::Default),
        "using_performance": list.is_using_profile(Profile::Performance),
    });
    if let Some(text) = file {
        state["file"] = serde_json::from_str(text).unwrap_or(Value::Null);
    }
    state
}

pub(super) fn debug_entries_case(row: &Value) -> Result<Value, String> {
    let mut out = json!({"label": row["label"]});
    if row.get("registry").is_some() {
        let (registry, profiles) = registry();
        out["registry"] = registry;
        out["profiles"] = profiles;
        // The screen lists them by group, then by id.
        debug_assert!(listed_entries().len() == ENTRIES.len());
    }
    if let Some(ops) = row["ops"].as_array() {
        out["states"] = run_ops(ops)?;
    }
    Ok(out)
}

/// An entry list and the file it saved, put through `ops`: the state after each.
fn run_ops(ops: &[Value]) -> Result<Value, String> {
    let mut list = EntryList::default();
    let mut reduced = false;
    let mut file: Option<String> = None;
    let mut states = vec![state(&list, reduced, file.as_deref())];
    for op in ops {
        let result = apply_op(op, &mut list, &mut reduced, &mut file)?;
        if list.take_changed() {
            file = Some(profile_text(&list, photon_game::options::DATA_VERSION));
        }
        let mut after = state(&list, reduced, file.as_deref());
        if let Some(result) = result {
            after["result"] = json!(result);
        }
        states.push(after);
    }
    Ok(Value::Array(states))
}

/// One operation; a toggle's answer.
fn apply_op(
    op: &Value,
    list: &mut EntryList,
    reduced: &mut bool,
    file: &mut Option<String>,
) -> Result<Option<bool>, String> {
    let id = op["id"].as_str().unwrap_or("");
    match op["op"].as_str().unwrap_or("") {
        "set" => list.set_status(id, status_of(op["status"].as_str().unwrap_or(""))?),
        "toggle" => return Ok(Some(list.toggle_status(id))),
        "profile" => {
            list.load_profile(profile_of(op["profile"].as_str().unwrap_or(""))?);
            *file = Some(profile_text(list, photon_game::options::DATA_VERSION));
        }
        "overlay" => list.set_overlay_visible(op["visible"].as_bool().unwrap_or(false)),
        "reduced" => *reduced = op["on"].as_bool().unwrap_or(false),
        "write" => write_file(op["text"].as_str().unwrap_or(""), list, file),
        other => return Err(format!("unknown debug entries op {other}")),
    }
    Ok(None)
}

/// The file is written and read: a file that cannot be read gives the default profile, saved.
fn write_file(text: &str, list: &mut EntryList, file: &mut Option<String>) {
    let overlay = list.is_overlay_visible();
    *file = Some(text.to_owned());
    *list = parse_profile(text).unwrap_or_else(|| {
        let fresh = EntryList::default();
        *file = Some(profile_text(&fresh, photon_game::options::DATA_VERSION));
        fresh
    });
    list.set_overlay_visible(overlay);
}
