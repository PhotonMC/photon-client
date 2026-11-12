//! The `photon:mods` payload a server sends and the `photon:hello` payload a client answers with.

use photon_core::json::{JsonObject, JsonValue, parse_object};

use super::MAX_MOD_BYTES;
use super::sync::Refused;
use crate::account::is_this_machine;

/// The channel a server names its mods on.
pub const MODS_CHANNEL: &str = "photon:mods";
/// The channel a Photon client announces itself on, once per join.
pub const HELLO_CHANNEL: &str = "photon:hello";

const MAX_PAYLOAD: usize = 64 << 10;
const MAX_MODS: usize = 64;
const MAX_CAPS: usize = 16;

/// Why a manifest was not taken.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    TooBig,
    NotJson,
    /// A manifest of a version this client does not know.
    Version,
    /// An entry that is wrong, with what is wrong with it.
    Entry(&'static str),
}

/// One mod the server wants loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModEntry {
    /// The mod's name, which is also what it is unloaded by: lower-case letters, digits, `_`, `-`.
    pub id: String,
    /// SHA-256 of the file, 64 lower-case hex digits.
    pub hash: String,
    pub size: u64,
    /// Where the file is fetched from: `https`, or this machine.
    pub url: String,
    /// The capabilities the mod asks for, such as `hud.text`.
    pub caps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Manifest {
    pub mods: Vec<ModEntry>,
}

impl Manifest {
    /// The manifest in a `photon:mods` payload.
    pub fn parse(data: &[u8]) -> Result<Self, ManifestError> {
        if data.len() > MAX_PAYLOAD {
            return Err(ManifestError::TooBig);
        }
        let text = std::str::from_utf8(data).map_err(|_| ManifestError::NotJson)?;
        let object = parse_object(text).ok_or(ManifestError::NotJson)?;
        if object.i32("v") != Some(1) {
            return Err(ManifestError::Version);
        }
        let items = object.array("mods").ok_or(ManifestError::Entry("mods"))?;
        if items.len() > MAX_MODS {
            return Err(ManifestError::Entry("too many mods"));
        }
        let mut mods: Vec<ModEntry> = Vec::new();
        for item in items {
            let JsonValue::Object(item) = item else {
                return Err(ManifestError::Entry("mod"));
            };
            let entry = entry(item)?;
            if mods.iter().any(|other| other.id == entry.id) {
                return Err(ManifestError::Entry("duplicate id"));
            }
            mods.push(entry);
        }
        Ok(Self { mods })
    }
}

fn entry(object: &JsonObject) -> Result<ModEntry, ManifestError> {
    let id = object.text("id").filter(|id| valid_id(id));
    let hash = object.text("hash").filter(|hash| valid_hash(hash));
    let url = object.text("url").filter(|url| valid_url(url));
    let size = object
        .get_str("size")
        .and_then(JsonValue::as_i64)
        .and_then(|size| u64::try_from(size).ok())
        .filter(|size| (1..=MAX_MOD_BYTES).contains(size));
    let (Some(id), Some(hash), Some(url), Some(size)) = (id, hash, url, size) else {
        return Err(ManifestError::Entry("id, hash, url or size"));
    };
    let caps = object.array("caps").unwrap_or_default();
    if caps.len() > MAX_CAPS {
        return Err(ManifestError::Entry("too many capabilities"));
    }
    let caps = caps
        .iter()
        .map(|cap| match cap {
            JsonValue::String(units) => String::from_utf16(units)
                .ok()
                .filter(|cap| valid_capability(cap)),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(ManifestError::Entry("capability"))?;
    Ok(ModEntry {
        id,
        hash,
        size,
        url,
        caps,
    })
}

fn valid_id(id: &str) -> bool {
    (1..=32).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

fn valid_hash(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn valid_url(url: &str) -> bool {
    url.len() <= 2048 && (url.starts_with("https://") || is_this_machine(url))
}

fn valid_capability(cap: &str) -> bool {
    (1..=32).contains(&cap.len())
        && cap
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b == b'.' || b == b'_')
}

/// The `photon:hello` payload: the client says it is Photon, which payload version it speaks and
/// which mods it has loaded (ids), so a server can gate on it and on what loaded. `refused`
/// names each mod the client did not run with a short fixed reason; the field is left out when
/// nothing was refused.
pub fn hello(client: &str, loaded: &[&str], refused: &[Refused]) -> Vec<u8> {
    let list: Vec<String> = loaded.iter().map(|id| format!("\"{id}\"")).collect();
    let client: String = client
        .chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ')
        .filter(|c| *c != '"' && *c != '\\')
        .take(64)
        .collect();
    let refused = refused_json(refused);
    format!(
        "{{\"v\":1,\"client\":\"{client}\",\"loaded\":[{}]{refused}}}",
        list.join(",")
    )
    .into_bytes()
}

/// `,"refused":[...]`, or nothing when no mod was refused.
fn refused_json(refused: &[Refused]) -> String {
    if refused.is_empty() {
        return String::new();
    }
    let items: Vec<String> = refused
        .iter()
        .take(MAX_MODS)
        .map(|r| {
            let (id, reason) = (&r.id, r.reason.word());
            format!("{{\"id\":\"{id}\",\"reason\":\"{reason}\"}}")
        })
        .collect();
    format!(",\"refused\":[{}]", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH: &str = "ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12";

    fn one(extra: &str) -> String {
        format!(
            r#"{{"v":1,"mods":[{{"id":"tycoon","hash":"{HASH}","size":1200,"url":"https://cdn.example/t.wasm"{extra}}}]}}"#
        )
    }

    #[test]
    fn a_manifest_is_read() {
        let manifest = Manifest::parse(one(r#","caps":["hud.text","sound.play"]"#).as_bytes())
            .expect("manifest");
        let entry = &manifest.mods[0];
        assert_eq!((entry.id.as_str(), entry.size), ("tycoon", 1200));
        assert_eq!(entry.caps, ["hud.text", "sound.play"]);
    }

    #[test]
    fn a_wrong_manifest_is_refused_whole() {
        let bad = |text: String| Manifest::parse(text.as_bytes()).is_err();
        assert!(bad(one("").replace("\"v\":1", "\"v\":2")));
        assert!(bad(one("").replace("tycoon", "Bad Id")));
        assert!(bad(one("").replace(HASH, "abc")));
        assert!(bad(one("").replace(":1200", ":0")));
        assert!(bad(one("").replace(":1200", ":99999999999")));
        assert!(bad(
            one("").replace("https://cdn.example", "http://cdn.example")
        ));
        assert!(bad(one(r#","caps":["HUD"]"#)));
        assert!(bad("not json".to_owned()));
        assert!(bad(r#"{"v":1}"#.to_owned()));
    }

    #[test]
    fn this_machine_may_serve_over_plain_http_and_ids_must_differ() {
        let local = one("").replace("https://cdn.example", "http://127.0.0.1:9000");
        assert!(Manifest::parse(local.as_bytes()).is_ok());
        let entry = format!(r#"{{"id":"a","hash":"{HASH}","size":5,"url":"https://h/a"}}"#);
        let twice = format!(r#"{{"v":1,"mods":[{entry},{entry}]}}"#);
        assert_eq!(
            Manifest::parse(twice.as_bytes()),
            Err(ManifestError::Entry("duplicate id"))
        );
    }

    #[test]
    fn the_hello_names_the_client_and_what_loaded() {
        let text = String::from_utf8(hello("Photon 0.1 \"x\"", &["a", "b"], &[])).expect("utf8");
        assert_eq!(
            text,
            r#"{"v":1,"client":"Photon 0.1 x","loaded":["a","b"]}"#
        );
        assert!(parse_object(&text).is_some());
    }

    #[test]
    fn refused_mods_are_named_with_a_reason() {
        use super::super::sync::Reason;
        let refused = [
            Refused {
                id: "odd".to_owned(),
                reason: Reason::Capability,
            },
            Refused {
                id: "big".to_owned(),
                reason: Reason::Size,
            },
        ];
        let text = String::from_utf8(hello("Photon", &["a"], &refused)).expect("utf8");
        assert_eq!(
            text,
            r#"{"v":1,"client":"Photon","loaded":["a"],"refused":[{"id":"odd","reason":"capability"},{"id":"big","reason":"size"}]}"#
        );
        let object = parse_object(&text).expect("json");
        assert_eq!(object.array("refused").map(|list| list.len()), Some(2));
    }
}
