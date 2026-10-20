//! `debug-profile.json`: the profile the debug screen last used, or every status set when none is
//! (`DebugScreenEntryList.load` and `save`).

use photon_core::json::{JsonValue, parse};

use super::list::{EntryList, Profile, Status};

fn text_of(units: &[u16]) -> String {
    String::from_utf16_lossy(units)
}

/// `DebugScreenEntryList.load`: the list a file's text describes. None when it cannot be read as
/// one, which the reference answers by resetting to the default profile and saving it.
pub fn parse_profile(text: &str) -> Option<EntryList> {
    let JsonValue::Object(root) = parse(text).ok()? else {
        return None;
    };
    let profile = match root.get_str("profile") {
        None => None,
        Some(JsonValue::String(name)) => Some(Profile::from_name(&text_of(name))?),
        Some(_) => return None,
    };
    let mut custom = Vec::new();
    match root.get_str("custom") {
        None => {}
        Some(JsonValue::Object(map)) => {
            for (key, value) in map.entries() {
                let JsonValue::String(name) = value else {
                    return None;
                };
                let key = text_of(key);
                let id = key.strip_prefix("minecraft:").unwrap_or(&key).to_owned();
                custom.push((id, Status::from_name(&text_of(name))?));
            }
        }
        Some(_) => return None,
    }
    Some(EntryList::from_saved(profile, custom))
}

/// `DebugScreenEntryList.save`: the profile, or the custom statuses, and the data version.
pub fn profile_text(list: &EntryList, data_version: i32) -> String {
    let (profile, statuses) = list.saved();
    let body = if let Some(profile) = profile {
        format!("\"profile\":\"{}\"", profile.name())
    } else {
        let fields: Vec<String> = statuses
            .iter()
            .map(|(id, status)| {
                let namespace = if id.contains(':') { "" } else { "minecraft:" };
                format!("\"{namespace}{id}\":\"{}\"", status.name())
            })
            .collect();
        format!("\"custom\":{{{}}}", fields.join(","))
    };
    format!("{{{body},\"DataVersion\":{data_version}}}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_saved_profile_reads_back() {
        let mut list = EntryList::default();
        assert_eq!(
            profile_text(&list, 4903),
            r#"{"profile":"default","DataVersion":4903}"#
        );
        list.set_status("fps", Status::Never);
        let text = profile_text(&list, 4903);
        assert!(text.starts_with(r#"{"custom":{"minecraft:3d_crosshair":"inOverlay""#));
        let back = parse_profile(&text).expect("a profile");
        assert_eq!(back.status("fps"), Status::Never);
        assert_eq!(back.status("tps"), Status::InOverlay);
        assert!(!back.is_using_profile(Profile::Default));
    }

    #[test]
    fn a_file_that_cannot_be_read_gives_nothing() {
        assert!(parse_profile("{\"profile\":\"bogus\"}").is_none());
        assert!(parse_profile("[]").is_none());
        let empty = parse_profile("{}").expect("neither field");
        assert_eq!(empty.status("fps"), Status::Never);
    }
}
