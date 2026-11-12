//! What a connection does with the server's mod lists, without any I/O: which entries to fetch,
//! which results still matter, which mods were refused and why, and when the client owes the
//! server a `photon:hello`. The app feeds it events and does what it answers.

use super::manifest::{Manifest, ModEntry};

/// Why a mod was not loaded: a short fixed word the hello carries to the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// It asks for a capability this client does not grant.
    Capability,
    /// The bytes are not the hash the manifest named.
    Hash,
    /// The bytes are not the size the manifest named.
    Size,
    /// The file could not be fetched or kept.
    Fetch,
    /// The code is not valid, ran wrong when loading, or the host has no room.
    Load,
    /// The player has not allowed a capability it needs (or refused it).
    Consent,
}

impl Reason {
    /// The word in the hello.
    pub fn word(self) -> &'static str {
        match self {
            Self::Capability => "capability",
            Self::Hash => "hash",
            Self::Size => "size",
            Self::Fetch => "fetch",
            Self::Load => "load",
            Self::Consent => "consent",
        }
    }
}

/// A mod the client will not run, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    pub id: String,
    pub reason: Reason,
}

/// What the client decides about one manifest entry before fetching it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Admission {
    Load,
    Refuse(Reason),
    /// The mod needs the player's answer about these capabilities first. It is held back, not
    /// fetched; while no answer is given the hello names it as refused for `consent`.
    Ask(Vec<String>),
}

/// A question for the player: a mod waits for an answer about these capabilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub id: String,
    pub hash: String,
    pub caps: Vec<String>,
}

/// What the app does for a manifest.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Plan {
    /// The `(id, hash)` pairs the manifest names: every loaded mod not in it is unloaded.
    pub keep: Vec<(String, String)>,
    /// The files to fetch, one entry per hash, none already being fetched.
    pub fetch: Vec<ModEntry>,
}

/// The state of one connection's mods.
#[derive(Debug)]
pub struct ModSync {
    /// Entries of the latest manifest that are not loaded or refused yet.
    pending: Vec<ModEntry>,
    /// Hashes whose fetch was started and has not answered; a later manifest does not start it
    /// again.
    in_flight: Vec<String>,
    refused: Vec<Refused>,
    /// Entries of the latest manifest that wait for the player, with what they ask about.
    held: Vec<(ModEntry, Vec<String>)>,
    hello_due: bool,
}

impl Default for ModSync {
    fn default() -> Self {
        Self::new()
    }
}

impl ModSync {
    /// A fresh connection: the join hello is due, with nothing loaded.
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            in_flight: Vec::new(),
            refused: Vec::new(),
            held: Vec::new(),
            hello_due: true,
        }
    }

    /// The server sent `manifest`. `admit` decides each entry that is not loaded already; a
    /// refused one is named in the next hello. The hello is due at once if nothing has to be
    /// fetched, else when the last fetch has answered.
    pub fn manifest(
        &mut self,
        manifest: &Manifest,
        admit: impl Fn(&ModEntry) -> Admission,
        is_loaded: impl Fn(&str, &str) -> bool,
    ) -> Plan {
        let keep = manifest
            .mods
            .iter()
            .map(|entry| (entry.id.clone(), entry.hash.clone()))
            .collect();
        self.pending.clear();
        self.refused.clear();
        self.held.clear();
        let mut fetch: Vec<ModEntry> = Vec::new();
        for entry in &manifest.mods {
            if is_loaded(&entry.id, &entry.hash) {
                continue;
            }
            match admit(entry) {
                Admission::Load => {}
                Admission::Refuse(reason) => {
                    self.refused.push(Refused {
                        id: entry.id.clone(),
                        reason,
                    });
                    continue;
                }
                Admission::Ask(caps) => {
                    self.refused.push(Refused {
                        id: entry.id.clone(),
                        reason: Reason::Consent,
                    });
                    self.held.push((entry.clone(), caps));
                    continue;
                }
            }
            if !self.in_flight.contains(&entry.hash) {
                self.in_flight.push(entry.hash.clone());
                fetch.push(entry.clone());
            }
            self.pending.push(entry.clone());
        }
        self.hello_due |= self.pending.is_empty();
        Plan { keep, fetch }
    }

    /// The fetch of `hash` answered. The entries of the latest manifest that wait for it are
    /// returned to be loaded (on `Ok`) or refused; an answer for a hash no current entry waits
    /// for, because a newer manifest dropped or changed the mod, loads nothing.
    pub fn fetched(&mut self, hash: &str, outcome: Result<(), Reason>) -> Vec<ModEntry> {
        self.in_flight.retain(|h| h != hash);
        let (waiting, rest): (Vec<ModEntry>, Vec<ModEntry>) = std::mem::take(&mut self.pending)
            .into_iter()
            .partition(|entry| entry.hash == hash);
        self.pending = rest;
        if waiting.is_empty() {
            return waiting;
        }
        self.hello_due |= self.pending.is_empty();
        match outcome {
            Ok(()) => waiting,
            Err(reason) => {
                self.refused
                    .extend(waiting.into_iter().map(|entry| Refused {
                        id: entry.id,
                        reason,
                    }));
                Vec::new()
            }
        }
    }

    /// The questions the player has not answered yet, for the latest manifest.
    pub fn questions(&self) -> Vec<Question> {
        self.held
            .iter()
            .map(|(entry, caps)| Question {
                id: entry.id.clone(),
                hash: entry.hash.clone(),
                caps: caps.clone(),
            })
            .collect()
    }

    /// The player answered the question of mod `id`. A refusal leaves the mod refused for
    /// `consent`; a grant returns the file to fetch (none if it is being fetched already or
    /// the question is gone), and the mod is loaded when it arrives like any other.
    pub fn answer(&mut self, id: &str, granted: bool) -> Vec<ModEntry> {
        let Some(index) = self.held.iter().position(|(entry, _)| entry.id == id) else {
            return Vec::new();
        };
        let (entry, _) = self.held.remove(index);
        if !granted {
            self.hello_due = true;
            return Vec::new();
        }
        self.refused.retain(|r| r.id != id);
        let fetch = !self.in_flight.contains(&entry.hash);
        if fetch {
            self.in_flight.push(entry.hash.clone());
        }
        self.pending.push(entry.clone());
        if fetch { vec![entry] } else { Vec::new() }
    }

    /// A mod returned by [`ModSync::fetched`] could not be loaded.
    pub fn load_failed(&mut self, id: &str, reason: Reason) {
        self.refused.push(Refused {
            id: id.to_owned(),
            reason,
        });
        self.hello_due = true;
    }

    /// Whether a hello is owed now; asking takes the debt.
    pub fn take_hello_due(&mut self) -> bool {
        std::mem::take(&mut self.hello_due)
    }

    /// The mods the client will not run, as of the latest manifest.
    pub fn refused(&self) -> &[Refused] {
        &self.refused
    }

    /// How many entries of the latest manifest still wait for a fetch.
    pub fn waiting(&self) -> usize {
        self.pending.len()
    }

    /// The connection starts over (a reconfiguration or a transfer): as new.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

#[cfg(test)]
mod tests;
