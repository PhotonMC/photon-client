//! Mods the connected server delivers (ADR 0024): the manifest it sends is fetched (or read from
//! the cache), each mod is loaded with only the capabilities this client grants, ticked with the
//! client, and unloaded when the server drops it or the connection ends.

use std::sync::mpsc::{Receiver, channel};

use photon_mods::{Compiled, Runtime};
use photon_net::mods::{self, Manifest, ModCache, ModEntry};

/// What the fetch thread made of one entry: compiled code, or why not.
type Fetched = (ModEntry, std::result::Result<Compiled, String>);

/// The mods of the connection.
#[derive(Default)]
pub(crate) struct ModRuntime {
    pub(crate) runtime: Runtime,
    fetching: Option<Receiver<Fetched>>,
    /// How many fetches of the last manifest have not answered yet.
    pub(crate) waiting: usize,
    pub(crate) said_hello: bool,
}

impl ModRuntime {
    /// A new manifest replaces the mods: those it no longer names, or names with another hash,
    /// are unloaded; the others it adds are fetched off the game's thread.
    pub(crate) fn on_manifest(&mut self, manifest: Manifest, cache_dir: std::path::PathBuf) {
        let named: Vec<(&str, &str)> = manifest
            .mods
            .iter()
            .map(|entry| (entry.id.as_str(), entry.hash.as_str()))
            .collect();
        self.runtime.keep_only(&named);
        let wanted: Vec<ModEntry> = manifest
            .mods
            .iter()
            .filter(|entry| !self.runtime.has(&entry.id, &entry.hash))
            .filter(|entry| {
                let allowed = photon_mods::grant(&entry.caps).is_some();
                if !allowed {
                    log::warn!("mod {} asks for {:?}: refused", entry.id, entry.caps);
                }
                allowed
            })
            .cloned()
            .collect();
        self.waiting = wanted.len();
        self.fetching = None;
        if wanted.is_empty() {
            return;
        }
        let host = match self.runtime.host() {
            Ok(host) => host,
            Err(error) => {
                log::warn!("mods not started: {error}");
                self.waiting = 0;
                return;
            }
        };
        let (sender, receiver) = channel();
        let spawned = std::thread::Builder::new()
            .name("Mod Fetch".to_owned())
            .spawn(move || {
                let cache = ModCache::new(cache_dir);
                for entry in wanted {
                    // Fetching and compiling both happen here, off the game's thread.
                    let result = mods::ensure(&cache, &entry)
                        .map_err(|error| format!("not fetched: {error:?}"))
                        .and_then(|bytes| {
                            host.compile(&bytes)
                                .map_err(|error| format!("not compiled: {error}"))
                        });
                    if sender.send((entry, result)).is_err() {
                        return;
                    }
                }
            });
        if spawned.is_err() {
            self.waiting = 0;
        }
        self.fetching = Some(receiver);
    }

    /// Loads what the fetch thread has finished; whether the last one of a manifest came in.
    pub(crate) fn load_fetched(&mut self) -> bool {
        let Some(receiver) = self.fetching.as_ref() else {
            return false;
        };
        let done: Vec<Fetched> = std::iter::from_fn(|| receiver.try_recv().ok()).collect();
        let arrived = !done.is_empty();
        for (entry, result) in done {
            self.waiting = self.waiting.saturating_sub(1);
            match result {
                Ok(compiled) => {
                    let loaded = self.runtime.load_compiled(
                        (&entry.id, &entry.hash),
                        &entry.caps,
                        &compiled,
                    );
                    if let Err(error) = loaded {
                        log::warn!("mod {} not loaded: {error}", entry.id);
                    }
                }
                Err(error) => log::warn!("mod {} {error}", entry.id),
            }
        }
        arrived && self.waiting == 0
    }
}
