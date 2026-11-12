//! What the player has allowed a server's mods to do (ADR 0024 addendum): capabilities that touch
//! the player's machine or data are asked about, naming the server, the mod, its hash and the
//! capability, and the answer is remembered per server and mod hash. This holds the answers and
//! decides a manifest entry from them; asking is the app's business.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use photon_core::json::{JsonObject, JsonValue, parse_object};

use super::manifest::ModEntry;
use super::sync::{Admission, Reason};
use crate::account::json_string;

/// The file under the game directory.
pub const CONSENT_FILE: &str = "mod_consent.json";
/// Most (server, hash) records kept; the oldest go first.
const MAX_RECORDS: usize = 1024;
/// Most characters of a server name kept.
const MAX_SERVER: usize = 256;
static PARTS: AtomicU64 = AtomicU64::new(0);

/// Whether `capability` touches the player's machine or data, so it needs the player's answer.
/// Capabilities that are neither this nor local to the game are not known to this client.
pub fn needs_consent(capability: &str) -> bool {
    capability == "fs.mod_data"
}

/// What the player said about one mod (by hash) on one server.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Record {
    pub server: String,
    pub hash: String,
    pub granted: Vec<String>,
    pub denied: Vec<String>,
}

/// The remembered answers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Consent {
    records: Vec<Record>,
}

impl Consent {
    /// The answers in `<game directory>/mod_consent.json`; none when the file is missing or not
    /// ours (a damaged file asks again, it never grants).
    pub fn load(game_directory: &Path) -> Self {
        std::fs::read_to_string(game_directory.join(CONSENT_FILE))
            .map(|text| Self::from_json(&text))
            .unwrap_or_default()
    }

    /// Writes the answers as one file replaced in one step, so a crash leaves the old file or
    /// the new one, and two clients on one directory never write the same temporary file.
    pub fn save(&self, game_directory: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(game_directory)?;
        let unique = PARTS.fetch_add(1, Ordering::Relaxed);
        let part: PathBuf = game_directory.join(format!(
            "{CONSENT_FILE}.{}.{unique}.part",
            std::process::id()
        ));
        std::fs::write(&part, self.to_json())?;
        std::fs::rename(&part, game_directory.join(CONSENT_FILE)).inspect_err(|_| {
            let _ = std::fs::remove_file(&part);
        })
    }

    pub fn from_json(text: &str) -> Self {
        let object = parse_object(text).filter(|object| object.i32("v") == Some(1));
        let records = object
            .as_ref()
            .and_then(|object| object.array("entries"))
            .unwrap_or_default()
            .iter()
            .filter_map(|item| match item {
                JsonValue::Object(object) => record(object),
                _ => None,
            })
            .take(MAX_RECORDS)
            .collect();
        Self { records }
    }

    pub fn to_json(&self) -> String {
        let list = |caps: &[String]| {
            let items: Vec<String> = caps.iter().map(|cap| json_string(cap)).collect();
            format!("[{}]", items.join(","))
        };
        let entries: Vec<String> = self
            .records
            .iter()
            .map(|r| {
                format!(
                    "{{\"server\":{},\"hash\":{},\"granted\":{},\"denied\":{}}}",
                    json_string(&r.server),
                    json_string(&r.hash),
                    list(&r.granted),
                    list(&r.denied)
                )
            })
            .collect();
        format!("{{\"v\":1,\"entries\":[{}]}}\n", entries.join(","))
    }

    /// The answers for this mod hash on this server. An empty server name is not a server and
    /// remembers nothing.
    pub fn answers(&self, server: &str, hash: &str) -> Option<&Record> {
        if server.is_empty() {
            return None;
        }
        self.records
            .iter()
            .find(|r| r.server == server && r.hash == hash)
    }

    /// Remembers that the player allowed or refused `capability` for this mod hash on this
    /// server; the last answer for a capability wins. Nothing is kept for an empty server name
    /// or a capability that needs no answer.
    pub fn answer(&mut self, server: &str, hash: &str, capability: &str, granted: bool) {
        if server.is_empty() || !needs_consent(capability) {
            return;
        }
        let index = self
            .records
            .iter()
            .position(|r| r.server == server && r.hash == hash)
            .unwrap_or_else(|| {
                self.records.push(Record {
                    server: server.chars().take(MAX_SERVER).collect(),
                    hash: hash.to_owned(),
                    ..Record::default()
                });
                self.records.len() - 1
            });
        let record = &mut self.records[index];
        record.granted.retain(|cap| cap != capability);
        record.denied.retain(|cap| cap != capability);
        let side = if granted {
            &mut record.granted
        } else {
            &mut record.denied
        };
        side.push(capability.to_owned());
        if self.records.len() > MAX_RECORDS {
            self.records.remove(0);
        }
    }
}

fn record(object: &JsonObject) -> Option<Record> {
    let strings = |key: &str| -> Vec<String> {
        object
            .array(key)
            .unwrap_or_default()
            .iter()
            .filter_map(|item| match item {
                JsonValue::String(units) => String::from_utf16(units).ok(),
                _ => None,
            })
            .filter(|cap| needs_consent(cap))
            .collect()
    };
    Some(Record {
        server: object.text("server").filter(|s| !s.is_empty())?,
        hash: object.text("hash").filter(|h| h.len() == 64)?,
        granted: strings("granted"),
        denied: strings("denied"),
    })
}

/// What to do with a manifest entry for the server `server`. A capability this client does not
/// know refuses the mod; one the player refused for this hash refuses it too; one that needs
/// an answer not given yet asks (and the mod waits); else the mod loads. `is_local` says which
/// capabilities the client grants without asking.
pub fn decide(
    consent: &Consent,
    server: &str,
    entry: &ModEntry,
    is_local: impl Fn(&str) -> bool,
) -> Admission {
    if entry
        .caps
        .iter()
        .any(|cap| !is_local(cap) && !needs_consent(cap))
    {
        return Admission::Refuse(Reason::Capability);
    }
    let known = consent.answers(server, &entry.hash);
    let said = |cap: &String, side: fn(&Record) -> &Vec<String>| {
        known.is_some_and(|record| side(record).contains(cap))
    };
    let needing = entry.caps.iter().filter(|cap| needs_consent(cap));
    if needing.clone().any(|cap| said(cap, |r| &r.denied)) {
        return Admission::Refuse(Reason::Consent);
    }
    let unanswered: Vec<String> = needing
        .filter(|cap| !said(cap, |r| &r.granted))
        .cloned()
        .collect();
    if unanswered.is_empty() {
        Admission::Load
    } else {
        Admission::Ask(unanswered)
    }
}

#[cfg(test)]
mod tests;
