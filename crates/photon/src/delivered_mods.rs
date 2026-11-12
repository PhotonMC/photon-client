//! Mods the connected server delivers (ADR 0024): the manifest it sends is fetched (or read from
//! the cache) and compiled on a thread of its own, each mod is loaded with only the capabilities
//! this client grants, ticked with the client, and unloaded when the server drops it or the
//! connection starts over. What to fetch, what to load and what to tell the server is decided by
//! [`ModSync`], which does no I/O; this holds the threads and the runtime.

use std::sync::mpsc::{Receiver, Sender, channel};

use photon_mods::{Compiled, Runtime};
use photon_net::mods::{self, Admission, Manifest, ModCache, ModEntry, ModSync, Reason};

/// What the fetch thread made of one file: compiled code, or why not.
type Fetched = (String, Result<Compiled, Reason>);

/// The mods of the connection.
#[derive(Default)]
pub(crate) struct ModRuntime {
    pub(crate) runtime: Runtime,
    sync: ModSync,
    /// Where the fetch threads answer; made with the first manifest.
    answers: Option<(Sender<Fetched>, Receiver<Fetched>)>,
}

/// Whether this client lets `entry` load.
fn admit(entry: &ModEntry) -> Admission {
    if photon_mods::grant(&entry.caps).is_some() {
        return Admission::Load;
    }
    log::warn!("mod {} asks for {:?}: refused", entry.id, entry.caps);
    Admission::Refuse(Reason::Capability)
}

impl ModRuntime {
    /// A new manifest replaces the mods: those it no longer names, or names with another hash,
    /// are unloaded; the others it adds are fetched off the game's thread.
    pub(crate) fn on_manifest(&mut self, manifest: &Manifest, cache_dir: std::path::PathBuf) {
        let runtime = &self.runtime;
        let plan = self
            .sync
            .manifest(manifest, admit, |id, hash| runtime.has(id, hash));
        let named: Vec<(&str, &str)> = plan
            .keep
            .iter()
            .map(|(id, hash)| (id.as_str(), hash.as_str()))
            .collect();
        self.runtime.keep_only(&named);
        if !plan.fetch.is_empty() {
            self.start_fetching(plan.fetch, cache_dir);
        }
    }

    fn start_fetching(&mut self, wanted: Vec<ModEntry>, cache_dir: std::path::PathBuf) {
        let host = match self.runtime.host() {
            Ok(host) => host,
            Err(error) => {
                log::warn!("mods not started: {error}");
                self.settle_all(&wanted, Reason::Load);
                return;
            }
        };
        let (sender, _) = self.answers.get_or_insert_with(channel);
        let sender = sender.clone();
        let for_failure = wanted.clone();
        let spawned = std::thread::Builder::new()
            .name("Mod Fetch".to_owned())
            .spawn(move || {
                let cache = ModCache::new(cache_dir);
                for entry in wanted {
                    // Fetching and compiling both happen here, off the game's thread.
                    let result = mods::ensure(&cache, &entry)
                        .map_err(|error| {
                            log::warn!("mod {} not fetched: {error:?}", entry.id);
                            error.reason()
                        })
                        .and_then(|bytes| {
                            host.compile(&bytes).map_err(|error| {
                                log::warn!("mod {} not compiled: {error}", entry.id);
                                Reason::Load
                            })
                        });
                    if sender.send((entry.hash, result)).is_err() {
                        return;
                    }
                }
            });
        if spawned.is_err() {
            self.settle_all(&for_failure, Reason::Fetch);
        }
    }

    fn settle_all(&mut self, entries: &[ModEntry], reason: Reason) {
        for entry in entries {
            self.settle(&entry.hash, Err(reason));
        }
    }

    /// One file's answer: the mods of the current manifest that wait for it are loaded or
    /// refused; an answer nobody waits for any more does nothing.
    fn settle(&mut self, hash: &str, result: Result<Compiled, Reason>) {
        let (compiled, outcome) = match result {
            Ok(compiled) => (Some(compiled), Ok(())),
            Err(reason) => (None, Err(reason)),
        };
        for entry in self.sync.fetched(hash, outcome) {
            let Some(compiled) = &compiled else { continue };
            let loaded =
                self.runtime
                    .load_compiled((&entry.id, &entry.hash), &entry.caps, compiled);
            if let Err(error) = loaded {
                log::warn!("mod {} not loaded: {error}", entry.id);
                self.sync.load_failed(&entry.id, Reason::Load);
            }
        }
    }

    /// Loads what the fetch threads have finished.
    pub(crate) fn load_fetched(&mut self) {
        let Some((_, receiver)) = self.answers.as_ref() else {
            return;
        };
        let done: Vec<Fetched> = std::iter::from_fn(|| receiver.try_recv().ok()).collect();
        for (hash, result) in done {
            self.settle(&hash, result);
        }
    }

    /// The `photon:hello` payload if one is due: who this is, which mods loaded and which were
    /// refused.
    pub(crate) fn hello(&mut self, client: &str) -> Option<Vec<u8>> {
        self.sync
            .take_hello_due()
            .then(|| mods::hello(client, &self.runtime.ids(), self.sync.refused()))
    }

    /// The connection starts over (a configuration phase again, or a transfer to another
    /// server): the mods of the old one go, answers still on their way are dropped, and the join
    /// hello is due again.
    pub(crate) fn restart(&mut self) {
        self.runtime.keep_only(&[]);
        self.answers = None;
        self.sync.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(caps: &str) -> Manifest {
        let hash = "ab".repeat(32);
        let text = format!(
            r#"{{"v":1,"mods":[{{"id":"m","hash":"{hash}","size":5,"url":"https://h/m.wasm","caps":[{caps}]}}]}}"#
        );
        Manifest::parse(text.as_bytes()).expect("manifest")
    }

    fn text(payload: Option<Vec<u8>>) -> String {
        String::from_utf8(payload.expect("a hello")).expect("utf8")
    }

    #[test]
    fn the_first_hello_is_the_join_hello_and_is_sent_once() {
        let mut mods = ModRuntime::default();
        assert_eq!(
            text(mods.hello("Photon")),
            r#"{"v":1,"client":"Photon","loaded":[]}"#
        );
        assert!(mods.hello("Photon").is_none());
    }

    #[test]
    fn a_mod_that_asks_for_too_much_is_named_in_the_next_hello() {
        let mut mods = ModRuntime::default();
        mods.hello("P");
        mods.on_manifest(&manifest(r#""fs.mod_data""#), std::env::temp_dir());
        assert_eq!(
            text(mods.hello("P")),
            r#"{"v":1,"client":"P","loaded":[],"refused":[{"id":"m","reason":"capability"}]}"#
        );
    }

    #[test]
    fn a_restarted_connection_forgets_refusals_and_says_hello_again() {
        let mut mods = ModRuntime::default();
        mods.hello("P");
        mods.on_manifest(&manifest(r#""fs.mod_data""#), std::env::temp_dir());
        mods.restart();
        assert_eq!(text(mods.hello("P")), r#"{"v":1,"client":"P","loaded":[]}"#);
    }
}
