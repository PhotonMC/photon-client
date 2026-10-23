//! What the client keeps of the server packs between connections (`DownloadedPackSource`): the
//! state machine, where its statuses go, the downloads on a thread of their own, and the
//! question the player is asked.
//!
//! The source is a handle that can be cloned and shared: a connection's thread, the window and
//! the download thread all hold it. Everything the state machine decides is carried out here:
//! statuses are sent to the server through the connection's [`PackSink`], downloads go to the
//! [`PackFetcher`], and a reload of the assets is left for the window to take
//! ([`PackSource::take_reload`]).

use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};

use photon_core::binary::Uuid;
use photon_game::servers::PackStatus;
use photon_game::text::Component;
use photon_protocol::play::{PackAction, PackRequest};

use super::manager::{DownloadRequest, Effect, PromptStatus, ServerPackManager, parse_sha1};

/// Packs to fetch, and for whom.
type DownloadBatch = (Vec<DownloadRequest>, PackUser);

type SendStatus = Arc<dyn Fn(Uuid, PackAction) + Send + Sync>;

/// Where the statuses of the packs go: the connection the server is on.
#[derive(Clone)]
pub struct PackSink {
    send: SendStatus,
    /// Ends the connection, for a sink whose owner is waiting on it.
    abort: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl PackSink {
    pub fn new(send: impl Fn(Uuid, PackAction) + Send + Sync + 'static) -> Self {
        Self {
            send: Arc::new(send),
            abort: None,
        }
    }

    /// The same sink, which ends the connection with `abort` when the player's answer must.
    #[must_use]
    pub fn with_abort(mut self, abort: impl Fn() + Send + Sync + 'static) -> Self {
        self.abort = Some(Arc::new(abort));
        self
    }
}

/// Fetches a pack into the cache and names the file it is in. A failure is the reason, for the
/// log; the server is only told that the download failed.
pub trait PackFetcher: Send + Sync {
    fn fetch(&self, request: &DownloadRequest, user: &PackUser) -> Result<PathBuf, String>;
}

/// Who the client is, for the servers the packs are fetched from (`X-Minecraft-Username` and
/// `X-Minecraft-UUID`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackUser {
    pub name: String,
    pub id: Uuid,
}

/// A fetcher for a client that cannot download: every pack fails.
struct NoFetcher;

impl PackFetcher for NoFetcher {
    fn fetch(&self, _request: &DownloadRequest, _user: &PackUser) -> Result<PathBuf, String> {
        Err("this client cannot download packs".to_owned())
    }
}

/// What the player is asked about (`PackConfirmScreen`): the packs waiting for the answer, and
/// whether the latest of them is required, with the text it brought.
#[derive(Debug, Clone, PartialEq)]
pub struct PackPrompt {
    pub requests: Vec<PackRequest>,
    pub required: bool,
    pub text: Option<Component>,
}

/// What follows from the player's answer.
#[derive(Debug, Clone, PartialEq)]
pub struct PromptOutcome {
    /// A required pack was refused: the connection ends with this reason.
    pub disconnect: Option<Component>,
    /// The setting to keep for the server (`ServerData.setResourcePackStatus`), when the answer
    /// changes it.
    pub remember: Option<PackStatus>,
}

struct Inner {
    manager: ServerPackManager,
    sink: Option<PackSink>,
    /// The server's setting: what is done with a pack without asking.
    setting: PackStatus,
    /// Whether anyone is there to answer a question; without, a pack that would ask is declined.
    attended: bool,
    prompt: Option<PackPrompt>,
    prompt_changed: bool,
    reload: bool,
    disconnect: Option<Component>,
    fetcher: Arc<dyn PackFetcher>,
    remember: Option<Arc<dyn Fn(PackStatus) + Send + Sync>>,
    user: PackUser,
    downloads: Option<Sender<DownloadBatch>>,
}

struct Shared {
    inner: Mutex<Inner>,
    /// Held while statuses are sent.
    sending: Mutex<()>,
}

/// `DownloadedPackSource`, shared.
#[derive(Clone)]
pub struct PackSource {
    shared: Arc<Shared>,
}

impl std::fmt::Debug for PackSource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("PackSource")
    }
}

impl PartialEq for PackSource {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.shared, &other.shared)
    }
}

impl Default for PackSource {
    /// A source nobody is there to answer for, that cannot download.
    fn default() -> Self {
        Self::new(Arc::new(NoFetcher), false)
    }
}

impl PackSource {
    /// A source that downloads with `fetcher`; `attended` says whether the player can be asked.
    pub fn new(fetcher: Arc<dyn PackFetcher>, attended: bool) -> Self {
        Self {
            shared: Arc::new(Shared {
                sending: Mutex::new(()),
                inner: Mutex::new(Inner {
                    manager: ServerPackManager::new(PromptStatus::Pending),
                    sink: None,
                    setting: PackStatus::Prompt,
                    attended,
                    prompt: None,
                    prompt_changed: false,
                    reload: false,
                    disconnect: None,
                    fetcher,
                    remember: None,
                    user: PackUser {
                        name: "Player".to_owned(),
                        id: Uuid::new(0, 1),
                    },
                    downloads: None,
                }),
            }),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.shared
            .inner
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Carry out what the machine decided: the statuses are sent first and in the order the
    /// machine gave them, outside the lock the rest of the client waits on, and only then do
    /// the downloads start (so that a pack is never reported as downloaded before accepted).
    fn settle(&self, mut inner: MutexGuard<'_, Inner>) {
        let mut statuses = Vec::new();
        let mut batches = Vec::new();
        for effect in inner.manager.take_effects() {
            match effect {
                Effect::Update(id, update) => statuses.push((id, update.action())),
                Effect::Final(id, result) => statuses.push((id, result.action())),
                Effect::Download(requests) => batches.push((requests, inner.user.clone())),
                Effect::Reload => inner.reload = true,
            }
        }
        let sink = inner.sink.clone();
        let downloads = if batches.is_empty() {
            None
        } else {
            Some(inner.download_queue(&self.shared))
        };
        // Whoever settles next waits for this to be sent, which keeps the statuses in order.
        let sending = self
            .shared
            .sending
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        drop(inner);
        for (id, action) in statuses {
            log::debug!("pack {} changed status to {action:?}", id.to_string_java());
            if let Some(sink) = &sink {
                (sink.send)(id, action);
            }
        }
        for batch in batches {
            if let Some(downloads) = &downloads {
                let _ = downloads.send(batch);
            }
        }
        drop(sending);
    }

    /// A source that asks the player but cannot download, until a fetcher is given.
    pub fn offline() -> Self {
        Self::new(Arc::new(NoFetcher), true)
    }

    /// What is done with the setting the player's answer changes (the server list keeps it).
    pub fn set_remember(&self, remember: impl Fn(PackStatus) + Send + Sync + 'static) {
        self.lock().remember = Some(Arc::new(remember));
    }

    /// The setting of the server the next connection goes to (`ServerData.getResourcePackStatus`).
    pub fn set_setting(&self, setting: PackStatus) {
        self.lock().setting = setting;
    }

    /// Who the client is to the servers it fetches packs from.
    pub fn set_user(&self, user: PackUser) {
        self.lock().user = user;
    }

    /// `configureForServerControl`: a connection starts, with the server's setting. A pack that
    /// was kept from before stays.
    pub fn configure_for_server(&self, sink: PackSink) {
        let mut inner = self.lock();
        inner.sink = Some(sink);
        inner.disconnect = None;
        match inner.setting {
            PackStatus::Enabled => inner.manager.allow(),
            PackStatus::Disabled => inner.manager.reject(),
            PackStatus::Prompt => inner.manager.reset_prompt(),
        }
        self.settle(inner);
    }

    /// The connection moved to another phase (or was made again): statuses go there now.
    pub fn set_sink(&self, sink: PackSink) {
        self.lock().sink = Some(sink);
    }

    /// `handleResourcePackPush`.
    pub fn push(&self, request: PackRequest) {
        let mut inner = self.lock();
        if !request.has_valid_url() {
            let sink = inner.sink.clone();
            drop(inner);
            if let Some(sink) = sink {
                (sink.send)(request.id, PackAction::InvalidUrl);
            }
            return;
        }
        // A setting of "disabled" still asks about a pack the server requires.
        let ask = inner.setting == PackStatus::Prompt
            || (request.required && inner.setting == PackStatus::Disabled);
        if !ask {
            inner.push_request(&request);
        } else if inner.attended {
            inner.add_to_prompt(request);
        } else {
            inner.manager.reject();
            inner.push_request(&request);
        }
        self.settle(inner);
    }

    /// `handleResourcePackPop`: one pack, or all of them.
    pub fn pop(&self, id: Option<Uuid>) {
        let mut inner = self.lock();
        match id {
            Some(id) => inner.manager.pop_pack(id),
            None => inner.manager.pop_all(),
        }
        self.settle(inner);
    }

    /// The question to ask, when it is new or changed since it was last taken.
    pub fn take_prompt(&self) -> Option<PackPrompt> {
        let mut inner = self.lock();
        if std::mem::take(&mut inner.prompt_changed) {
            inner.prompt.clone()
        } else {
            None
        }
    }

    /// The player answered the question (`PackConfirmScreen`'s callback).
    pub fn answer_prompt(&self, accepted: bool) -> PromptOutcome {
        let mut inner = self.lock();
        let Some(prompt) = inner.prompt.take() else {
            return PromptOutcome {
                disconnect: None,
                remember: None,
            };
        };
        let mut outcome = PromptOutcome {
            disconnect: None,
            remember: None,
        };
        if accepted {
            inner.setting = PackStatus::Enabled;
            outcome.remember = Some(PackStatus::Enabled);
            inner.manager.allow();
        } else {
            inner.manager.reject();
            if prompt.required {
                let reason =
                    Component::translatable("multiplayer.requiredTexturePrompt.disconnect");
                inner.disconnect = Some(reason.clone());
                outcome.disconnect = Some(reason);
            } else {
                inner.setting = PackStatus::Disabled;
                outcome.remember = Some(PackStatus::Disabled);
            }
        }
        for request in &prompt.requests {
            inner.push_request(request);
        }
        let abort = outcome
            .disconnect
            .as_ref()
            .and_then(|_| inner.sink.as_ref()?.abort.clone());
        let remember = inner.remember.clone().zip(outcome.remember);
        self.settle(inner);
        if let Some((remember, setting)) = remember {
            remember(setting);
        }
        if let Some(abort) = abort {
            abort();
        }
        outcome
    }

    /// The reason the connection was ended on the player's answer, once.
    pub fn take_disconnect(&self) -> Option<Component> {
        self.lock().disconnect.take()
    }

    /// Whether no pack is being asked about, fetched or loaded.
    pub fn is_idle(&self) -> bool {
        let inner = self.lock();
        inner.prompt.is_none() && inner.manager.is_idle()
    }

    /// The setting for the server, as the player last made it.
    pub fn setting(&self) -> PackStatus {
        self.lock().setting
    }

    /// `cleanupAfterDisconnect`: the connection is over, so every pack is taken off.
    pub fn cleanup_after_disconnect(&self) {
        let mut inner = self.lock();
        inner.manager.pop_all();
        inner.sink = None;
        inner.prompt = None;
        inner.prompt_changed = false;
        self.settle(inner);
        self.lock().manager.reset_prompt();
    }

    /// A reload of the assets is due: the packs to load, top of the stack first. The window
    /// answers with [`Self::reload_succeeded`] or [`Self::reload_failed`].
    pub fn take_reload(&self) -> Option<Vec<(Uuid, PathBuf)>> {
        let mut inner = self.lock();
        std::mem::take(&mut inner.reload).then(|| inner.manager.packs_to_load())
    }

    /// `onReloadSuccess`.
    pub fn reload_succeeded(&self) {
        let mut inner = self.lock();
        inner.manager.reload_succeeded();
        self.settle(inner);
    }

    /// `Callbacks.onFailure`: the packs to load again as a recovery (empty after a failed
    /// recovery).
    pub fn reload_failed(&self, recovery: bool) -> Vec<(Uuid, PathBuf)> {
        let mut inner = self.lock();
        inner.manager.reload_failed(recovery);
        let packs = if recovery {
            Vec::new()
        } else {
            inner.manager.packs_to_load()
        };
        self.settle(inner);
        packs
    }

    fn downloaded(&self, files: Vec<(u64, PathBuf)>, failed: &[u64]) {
        let mut inner = self.lock();
        inner.manager.downloaded(files, failed);
        self.settle(inner);
    }
}

impl Inner {
    /// The way to the download thread, which is started the first time.
    fn download_queue(&mut self, shared: &Arc<Shared>) -> Sender<DownloadBatch> {
        if let Some(downloads) = &self.downloads {
            return downloads.clone();
        }
        let (send, receive) = mpsc::channel();
        let source = Arc::downgrade(shared);
        let fetcher = Arc::clone(&self.fetcher);
        let spawned = std::thread::Builder::new()
            .name("Download-Queue".to_owned())
            .spawn(move || download_until_closed(&receive, &source, fetcher.as_ref()));
        if spawned.is_err() {
            log::error!("no thread to download packs on");
        }
        self.downloads = Some(send.clone());
        send
    }

    /// `ServerPackManager.pushPack` for a pack the server asked for.
    fn push_request(&mut self, request: &PackRequest) {
        self.manager
            .push_pack(request.id, request.url.clone(), parse_sha1(&request.hash));
    }

    /// `addOrUpdatePackPrompt`: a question already up takes the pack in, and the latest pack's
    /// wording.
    fn add_to_prompt(&mut self, request: PackRequest) {
        let prompt = self.prompt.get_or_insert_with(|| PackPrompt {
            requests: Vec::new(),
            required: false,
            text: None,
        });
        prompt.required = request.required;
        prompt.text.clone_from(&request.prompt);
        prompt.requests.push(request);
        self.prompt_changed = true;
    }
}

/// The download thread: batches come in order, each pack one after the other, and the outcome
/// of a batch goes back whole.
fn download_until_closed(
    batches: &mpsc::Receiver<DownloadBatch>,
    source: &Weak<Shared>,
    fetcher: &dyn PackFetcher,
) {
    while let Ok((batch, user)) = batches.recv() {
        let mut files = Vec::new();
        let mut failed = Vec::new();
        for request in &batch {
            match fetcher.fetch(request, &user) {
                Ok(path) => files.push((request.ticket, path)),
                Err(reason) => {
                    log::error!("failed to download {}: {reason}", request.url);
                    failed.push(request.ticket);
                }
            }
        }
        let Some(shared) = source.upgrade() else {
            return;
        };
        PackSource { shared }.downloaded(files, &failed);
    }
}
