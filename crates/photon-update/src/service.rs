//! The updater as a service without a user interface: a worker thread checks the release host at
//! start and every few hours, downloads and stores what it finds when the settings allow it, and
//! reports each change of state. The owner reads the changes with [`Service::poll`] on its own
//! thread, so no event handler ever runs on the worker.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use photon_api::{UpdateForServer, UpdateState};

use crate::{Error, Offer, download, offer, staged};

/// How often the host is asked while the client runs.
pub const CHECK_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

/// What the owner chose; a file of `key=value` lines (`auto_check`, `auto_download`, `channel`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub auto_check: bool,
    pub auto_download: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_check: true,
            auto_download: true,
        }
    }
}

impl Settings {
    /// The settings in `text`. Blank lines and `#` comments are skipped; an unknown key, a value
    /// that is not `true` or `false`, or a channel other than `stable` is an error naming the line,
    /// so a typo is never taken for a choice.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut settings = Self::default();
        for (number, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let at = number + 1;
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("line {at}: expected key=value"))?;
            let (key, value) = (key.trim(), value.trim());
            let flag = || {
                value
                    .parse::<bool>()
                    .map_err(|_| format!("line {at}: {key} must be true or false"))
            };
            match key {
                "auto_check" => settings.auto_check = flag()?,
                "auto_download" => settings.auto_download = flag()?,
                "channel" if value == "stable" => {}
                "channel" => return Err(format!("line {at}: the only channel is stable")),
                _ => return Err(format!("line {at}: unknown setting {key}")),
            }
        }
        Ok(settings)
    }
}

/// Where the service looks and keeps things.
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub keys: Vec<[u8; 32]>,
    pub platform: String,
    pub current: String,
    pub pending: PathBuf,
    pub settings: Settings,
    pub interval: Duration,
}

/// A release the service knows of, with the protocols it joins servers with.
#[derive(Debug, Clone)]
struct Known {
    version: String,
    protocols: Vec<u32>,
}

#[derive(Default)]
struct Shared {
    state: UpdateState,
    offered: Option<Known>,
    ready: Option<Known>,
}

enum Command {
    Check,
    Download,
    Stop,
}

/// The running updater.
pub struct Service {
    shared: Arc<Mutex<Shared>>,
    commands: Sender<Command>,
    changes: Receiver<UpdateState>,
    worker: Option<thread::JoinHandle<()>>,
}

impl Service {
    /// Starts the worker. With the settings' `auto_check` and a key it checks at once and then
    /// every `interval`; without a key it never touches the network.
    pub fn start(config: Config) -> Self {
        let shared = Arc::new(Mutex::new(Shared::default()));
        let (commands, inbox) = mpsc::channel();
        let (outbox, changes) = mpsc::channel();
        let mut worker = Worker {
            config,
            shared: Arc::clone(&shared),
            outbox,
            offer: None,
        };
        worker.note_staged();
        let handle = thread::Builder::new()
            .name("update".to_owned())
            .spawn(move || worker.run(&inbox))
            .ok();
        Self {
            shared,
            commands,
            changes,
            worker: handle,
        }
    }

    /// Where the updater stands now.
    pub fn state(&self) -> UpdateState {
        lock(&self.shared).state.clone()
    }

    /// The changes of state since the last call, oldest first.
    pub fn poll(&self) -> Vec<UpdateState> {
        self.changes.try_iter().collect()
    }

    /// Asks the host now, whatever the settings say.
    pub fn check_now(&self) {
        let _ = self.commands.send(Command::Check);
    }

    /// Downloads the available release (asking the host first when none is known).
    pub fn download(&self) {
        let _ = self.commands.send(Command::Download);
    }

    /// Whether a release the service knows of joins servers of `protocol`: the stored one first,
    /// then the offered one. A manifest that names no protocols says nothing, so it never counts.
    pub fn update_for_protocol(&self, protocol: u32) -> UpdateForServer {
        let shared = lock(&self.shared);
        let supports = |known: &Option<Known>| {
            known
                .as_ref()
                .filter(|known| known.protocols.contains(&protocol))
                .map(|known| known.version.clone())
        };
        if let Some(version) = supports(&shared.ready) {
            return UpdateForServer::Staged { version };
        }
        match supports(&shared.offered) {
            Some(version) => UpdateForServer::Available { version },
            None => UpdateForServer::None,
        }
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Stop);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn lock(shared: &Mutex<Shared>) -> std::sync::MutexGuard<'_, Shared> {
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

struct Worker {
    config: Config,
    shared: Arc<Mutex<Shared>>,
    outbox: Sender<UpdateState>,
    offer: Option<Offer>,
}

impl Worker {
    fn run(&mut self, inbox: &Receiver<Command>) {
        if self.config.settings.auto_check && !self.config.keys.is_empty() {
            self.check(self.config.settings.auto_download);
        }
        loop {
            let next = if self.config.settings.auto_check {
                inbox.recv_timeout(self.config.interval)
            } else {
                inbox.recv().map_err(|_| RecvTimeoutError::Disconnected)
            };
            match next {
                Ok(Command::Check) => self.check(false),
                Ok(Command::Download) => self.fetch(),
                Ok(Command::Stop) | Err(RecvTimeoutError::Disconnected) => return,
                Err(RecvTimeoutError::Timeout) => {
                    if !self.config.keys.is_empty() {
                        self.check(self.config.settings.auto_download);
                    }
                }
            }
        }
    }

    /// A release stored by an earlier run is ready, and says so before anything is asked.
    fn note_staged(&mut self) {
        if let Ok(stored) = staged(&self.config.pending, &self.config.keys) {
            let known = Known {
                version: stored.release.version.clone(),
                protocols: stored.release.protocols.clone(),
            };
            let mut shared = lock(&self.shared);
            shared.ready = Some(known);
            shared.state = UpdateState::Ready {
                version: stored.release.version,
            };
        }
    }

    fn set(&self, state: UpdateState) {
        lock(&self.shared).state = state.clone();
        let _ = self.outbox.send(state);
    }

    fn fail(&self, error: &Error) {
        let reason = match error {
            Error::NoKey => "this build has no release key".to_owned(),
            Error::NotFound => "the host has no release for this system".to_owned(),
            other => format!("{other:?}"),
        };
        self.set(UpdateState::Failed { reason });
    }

    /// Asks the host; downloads at once when `then_download` and a newer release is there.
    fn check(&mut self, then_download: bool) {
        self.set(UpdateState::Checking);
        let found = offer(
            &self.config.host,
            &self.config.keys,
            &self.config.platform,
            &self.config.current,
        );
        match found {
            Ok(Some(found)) => {
                let release = &found.release;
                lock(&self.shared).offered = Some(Known {
                    version: release.version.clone(),
                    protocols: release.protocols.clone(),
                });
                self.set(UpdateState::Available {
                    version: release.version.clone(),
                    notes: release.notes.clone(),
                    size: release.size,
                });
                self.offer = Some(found);
                if then_download {
                    self.fetch();
                }
            }
            Ok(None) => {
                let mut shared = lock(&self.shared);
                shared.offered = None;
                drop(shared);
                self.offer = None;
                self.settle();
            }
            Err(error) => self.fail(&error),
        }
    }

    /// Up to date: Ready if a release is stored, otherwise Idle.
    fn settle(&self) {
        let ready = lock(&self.shared).ready.clone();
        self.set(match ready {
            Some(known) => UpdateState::Ready {
                version: known.version,
            },
            None => UpdateState::Idle,
        });
    }

    fn fetch(&mut self) {
        if self.offer.is_none() {
            self.check(false);
        }
        let Some(found) = self.offer.take() else {
            return;
        };
        let total = found.release.size;
        self.set(UpdateState::Downloading { done: 0, total });
        let outbox = self.outbox.clone();
        let shared = Arc::clone(&self.shared);
        let mut last = 0;
        let result = download(
            &self.config.host,
            &self.config.keys,
            &found,
            (&self.config.current, &self.config.pending),
            &mut |done, total| {
                // About a hundred reports per download, never one per chunk.
                if done == total || done >= last + total / 100 {
                    last = done;
                    let state = UpdateState::Downloading { done, total };
                    lock(&shared).state = state.clone();
                    let _ = outbox.send(state);
                }
            },
        );
        match result {
            Ok(stored) => {
                let known = Known {
                    version: stored.release.version.clone(),
                    protocols: stored.release.protocols.clone(),
                };
                let mut shared = lock(&self.shared);
                shared.offered = None;
                shared.ready = Some(known);
                drop(shared);
                self.set(UpdateState::Ready {
                    version: stored.release.version,
                });
            }
            Err(error) => {
                self.offer = Some(found);
                self.fail(&error);
            }
        }
    }
}
