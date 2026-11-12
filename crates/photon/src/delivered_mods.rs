//! Mods the connected server delivers (ADR 0024): the manifest it sends is fetched (or read from
//! the cache) and compiled on a thread of its own, each mod is loaded with only the capabilities
//! this client grants, ticked with the client, and unloaded when the server drops it or the
//! connection starts over. What to fetch, what to load and what to tell the server is decided by
//! [`ModSync`], which does no I/O; this holds the threads and the runtime.

use std::sync::mpsc::{Receiver, Sender, channel};

use photon_mods::{Compiled, Runtime};
use photon_net::mods::{
    self, Consent, Manifest, ModCache, ModEntry, ModSync, Question, Reason, decide,
};

/// What the fetch thread made of one file: compiled code, or why not.
type Fetched = (String, Result<Compiled, Reason>);

/// The mods of the connection.
#[derive(Default)]
pub(crate) struct ModRuntime {
    pub(crate) runtime: Runtime,
    sync: ModSync,
    /// Where the fetch threads answer; made with the first manifest.
    answers: Option<(Sender<Fetched>, Receiver<Fetched>)>,
    /// What the player allowed, read from the game directory with the first manifest.
    consent: Option<(Consent, std::path::PathBuf)>,
    /// Who the mods come from, the key of what the player allowed. Empty while the app does not
    /// know it, and then nothing is remembered and every question is asked.
    server: String,
}

/// Whether the client grants `capability` without asking the player.
fn is_local(capability: &str) -> bool {
    photon_mods::GRANTED.contains(&capability)
}

/// The capabilities of `entry` the host links imports for: those the client grants itself. The
/// ones the player was asked about (`fs.mod_data`) have no import yet, so they link nothing.
fn linked(entry: &ModEntry) -> Vec<String> {
    let local = entry.caps.iter().filter(|cap| is_local(cap));
    local.cloned().collect()
}

impl ModRuntime {
    /// A new manifest replaces the mods: those it no longer names, or names with another hash,
    /// are unloaded; the others it adds are fetched off the game's thread.
    pub(crate) fn on_manifest(&mut self, manifest: &Manifest, game_dir: &std::path::Path) {
        let (consent, _) = self
            .consent
            .get_or_insert_with(|| (Consent::load(game_dir), game_dir.to_owned()));
        let (runtime, server) = (&self.runtime, self.server.as_str());
        let plan = self.sync.manifest(
            manifest,
            |entry| {
                let admission = decide(consent, server, entry, is_local);
                if admission != mods::Admission::Load {
                    log::warn!("mod {} asks for {:?}: {admission:?}", entry.id, entry.caps);
                }
                admission
            },
            |id, hash| runtime.has(id, hash),
        );
        let named: Vec<(&str, &str)> = plan
            .keep
            .iter()
            .map(|(id, hash)| (id.as_str(), hash.as_str()))
            .collect();
        self.runtime.keep_only(&named);
        if !plan.fetch.is_empty() {
            self.start_fetching(plan.fetch, game_dir.join("cache").join("mods"));
        }
    }

    /// The server the mods come from, as the session knows it; the key of the player's answers.
    /// Without one (none known) nothing is remembered and every question is asked again.
    pub(crate) fn set_server(&mut self, server: Option<&str>) {
        let server = server.unwrap_or_default();
        if self.server != server {
            server.clone_into(&mut self.server);
        }
    }

    /// The questions the player has not answered: mods held back until they are.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the screen that asks does not exist yet")
    )]
    pub(crate) fn questions(&self) -> Vec<Question> {
        self.sync.questions()
    }

    /// The player's answer about mod `id`: remembered for this server and mod hash, then the mod
    /// is fetched and loaded (a grant) or stays refused (a refusal).
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the screen that asks does not exist yet")
    )]
    pub(crate) fn answer(&mut self, id: &str, granted: bool) {
        let Some(question) = self.sync.questions().into_iter().find(|q| q.id == id) else {
            return;
        };
        let Some((consent, dir)) = self.consent.as_mut() else {
            return;
        };
        for cap in &question.caps {
            consent.answer(&self.server, &question.hash, cap, granted);
        }
        if let Err(error) = consent.save(dir) {
            log::warn!("the answer about mod {id} was not saved: {error}");
        }
        let cache_dir = dir.join("cache").join("mods");
        let fetch = self.sync.answer(id, granted);
        if !fetch.is_empty() {
            self.start_fetching(fetch, cache_dir);
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
                    .load_compiled((&entry.id, &entry.hash), &linked(&entry), compiled);
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
    use std::path::PathBuf;

    use super::*;

    fn manifest(caps: &str) -> Manifest {
        let hash = "ab".repeat(32);
        // Nothing listens on port 9 of this machine, so a fetch fails at once.
        let text = format!(
            r#"{{"v":1,"mods":[{{"id":"m","hash":"{hash}","size":5,"url":"http://127.0.0.1:9/m.wasm","caps":[{caps}]}}]}}"#
        );
        Manifest::parse(text.as_bytes()).expect("manifest")
    }

    fn scratch(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("photon-mods-app-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    fn on_server() -> ModRuntime {
        ModRuntime {
            server: "play.example.org:25565".to_owned(),
            ..ModRuntime::default()
        }
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
    fn a_mod_that_asks_for_an_unknown_capability_is_named_in_the_next_hello() {
        let mut mods = ModRuntime::default();
        mods.hello("P");
        mods.on_manifest(&manifest(r#""net.connect""#), &scratch("unknown"));
        assert_eq!(
            text(mods.hello("P")),
            r#"{"v":1,"client":"P","loaded":[],"refused":[{"id":"m","reason":"capability"}]}"#
        );
        assert!(mods.questions().is_empty());
    }

    #[test]
    fn a_mod_that_needs_an_answer_waits_and_is_refused_for_consent_meanwhile() {
        let dir = scratch("waits");
        let mut mods = ModRuntime::default();
        mods.hello("P");
        mods.on_manifest(&manifest(r#""hud.text","fs.mod_data""#), &dir);
        assert_eq!(
            text(mods.hello("P")),
            r#"{"v":1,"client":"P","loaded":[],"refused":[{"id":"m","reason":"consent"}]}"#
        );
        let questions = mods.questions();
        assert_eq!(questions.len(), 1);
        assert_eq!(questions[0].caps, ["fs.mod_data"]);
        // Nothing was fetched or loaded: the file the manifest names is not in the cache.
        assert!(!dir.join("cache").exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_refusal_is_remembered_for_the_server_and_refuses_next_time() {
        let dir = scratch("refused");
        let mut mods = on_server();
        mods.on_manifest(&manifest(r#""fs.mod_data""#), &dir);
        mods.hello("P");
        mods.answer("m", false);
        assert!(mods.questions().is_empty());
        assert_eq!(
            text(mods.hello("P")),
            r#"{"v":1,"client":"P","loaded":[],"refused":[{"id":"m","reason":"consent"}]}"#
        );
        // A later join reads the file and does not ask again.
        let mut later = on_server();
        later.on_manifest(&manifest(r#""fs.mod_data""#), &dir);
        assert!(later.questions().is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_grant_resumes_the_mod_and_a_failed_fetch_is_then_reported() {
        let dir = scratch("granted");
        let mut mods = on_server();
        mods.hello("P");
        mods.on_manifest(&manifest(r#""fs.mod_data""#), &dir);
        mods.hello("P");
        mods.answer("m", true);
        assert!(mods.questions().is_empty());
        // The fetch goes to a closed port, fails, and the hello then says why.
        let started = std::time::Instant::now();
        let hello = loop {
            mods.load_fetched();
            if let Some(payload) = mods.hello("P") {
                break text(Some(payload));
            }
            assert!(started.elapsed() < std::time::Duration::from_secs(30));
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        assert_eq!(
            hello,
            r#"{"v":1,"client":"P","loaded":[],"refused":[{"id":"m","reason":"fetch"}]}"#
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_grant_is_kept_for_the_server_the_session_names() {
        let dir = scratch("keyed");
        let mut mods = ModRuntime::default();
        mods.set_server(Some("play.example.org:25565"));
        mods.on_manifest(&manifest(r#""fs.mod_data""#), &dir);
        mods.answer("m", true);
        let kept = Consent::load(&dir);
        assert!(
            kept.answers("play.example.org:25565", &"ab".repeat(32))
                .is_some()
        );
        assert!(
            kept.answers("other.example.org:25565", &"ab".repeat(32))
                .is_none()
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn an_unknown_server_remembers_no_answer() {
        let dir = scratch("nobody");
        let mut mods = ModRuntime::default();
        mods.on_manifest(&manifest(r#""fs.mod_data""#), &dir);
        mods.answer("m", true);
        assert_eq!(Consent::load(&dir), Consent::default());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_restarted_connection_forgets_refusals_and_says_hello_again() {
        let mut mods = ModRuntime::default();
        mods.hello("P");
        mods.on_manifest(&manifest(r#""net.connect""#), &scratch("restart"));
        mods.restart();
        assert_eq!(text(mods.hello("P")), r#"{"v":1,"client":"P","loaded":[]}"#);
    }
}
