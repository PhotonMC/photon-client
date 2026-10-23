//! The server resource packs' state machine (`ServerPackManager`): which packs the server pushed,
//! whether the player agreed to them, which are downloaded and active, and what the server is
//! told about each.
//!
//! The machine does no work itself. Every change that needs the outside (a status for the server,
//! a download, a reload of the assets) is left as an [`Effect`] for its owner to carry out, and
//! the owner reports the outcome back ([`ServerPackManager::downloaded`],
//! [`ServerPackManager::reload_succeeded`], [`ServerPackManager::reload_failed`]).

use std::path::PathBuf;

use photon_core::binary::Uuid;
use photon_protocol::play::PackAction;

/// `ServerPackManager.PackPromptStatus`: what the player has said about server packs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PromptStatus {
    #[default]
    Pending,
    Allowed,
    Declined,
}

/// What `PackLoadFeedback` reports on the way: the pack was taken or it arrived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Update {
    Accepted,
    Downloaded,
}

/// What `PackLoadFeedback` reports at the end of a pack's life.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinalResult {
    Declined,
    Applied,
    Discarded,
    DownloadFailed,
    ActivationFailed,
}

impl Update {
    /// The status the server is sent.
    pub fn action(self) -> PackAction {
        match self {
            Self::Accepted => PackAction::Accepted,
            Self::Downloaded => PackAction::Downloaded,
        }
    }
}

impl FinalResult {
    /// The status the server is sent.
    pub fn action(self) -> PackAction {
        match self {
            Self::Applied => PackAction::SuccessfullyLoaded,
            Self::DownloadFailed => PackAction::FailedDownload,
            Self::Declined => PackAction::Declined,
            Self::Discarded => PackAction::Discarded,
            Self::ActivationFailed => PackAction::FailedReload,
        }
    }
}

/// One pack to fetch: its address and the SHA-1 the server named (as 40 lowercase hex digits),
/// when it named a valid one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadRequest {
    pub id: Uuid,
    /// Which push this is, to name it when the outcome is reported.
    pub ticket: u64,
    pub url: String,
    pub hash: Option<String>,
}

/// What the manager asks its owner to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Tell the server how a pack is coming along.
    Update(Uuid, Update),
    /// Tell the server how a pack ended.
    Final(Uuid, FinalResult),
    /// Fetch these packs; each outcome comes back through [`ServerPackManager::downloaded`].
    Download(Vec<DownloadRequest>),
    /// Reload the assets with exactly these packs (top of the stack first), then report through
    /// [`ServerPackManager::reload_succeeded`] or [`ServerPackManager::reload_failed`].
    Reload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Activation {
    Inactive,
    Pending,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fetch {
    Requested,
    Pending,
    Done,
}

/// `ServerPackManager.RemovalReason`; the server hears of all but the two it caused itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Removal {
    DownloadFailed,
    ActivationFailed,
    Declined,
    Discarded,
    ServerRemoved,
    ServerReplaced,
}

impl Removal {
    fn response(self) -> Option<FinalResult> {
        match self {
            Self::DownloadFailed => Some(FinalResult::DownloadFailed),
            Self::ActivationFailed => Some(FinalResult::ActivationFailed),
            Self::Declined => Some(FinalResult::Declined),
            Self::Discarded => Some(FinalResult::Discarded),
            Self::ServerRemoved | Self::ServerReplaced => None,
        }
    }
}

#[derive(Debug)]
struct Pack {
    /// Which push this is: a pack pushed again under the same id is another one.
    serial: u64,
    id: Uuid,
    url: String,
    hash: Option<String>,
    path: Option<PathBuf>,
    removal: Option<Removal>,
    fetch: Fetch,
    activation: Activation,
    accepted: bool,
}

impl Pack {
    fn new(serial: u64, id: Uuid, url: String, hash: Option<String>) -> Self {
        Self {
            serial,
            id,
            url,
            hash,
            path: None,
            removal: None,
            fetch: Fetch::Requested,
            activation: Activation::Inactive,
            accepted: false,
        }
    }

    fn remove(&mut self, reason: Removal) {
        self.removal.get_or_insert(reason);
    }

    fn removed(&self) -> bool {
        self.removal.is_some()
    }

    /// Whether the pack should be on: taken, fetched and not removed.
    fn wanted(&self) -> bool {
        self.accepted && self.fetch == Fetch::Done && !self.removed()
    }
}

/// The reload in flight: the packs it loads, and the ones it takes off.
#[derive(Debug, Default)]
struct Reload {
    load: Vec<u64>,
    unload: Vec<u64>,
}

/// `ServerPackManager`.
#[derive(Debug, Default)]
pub struct ServerPackManager {
    prompt: PromptStatus,
    packs: Vec<Pack>,
    pushed: u64,
    reload: Option<Reload>,
    effects: Vec<Effect>,
    dirty: bool,
}

/// `SHA1` of the pack format check: forty hex digits of either case, lowercased.
pub fn parse_sha1(hash: &str) -> Option<String> {
    (hash.len() == 40 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then(|| hash.to_ascii_lowercase())
}

impl ServerPackManager {
    pub fn new(prompt: PromptStatus) -> Self {
        Self {
            prompt,
            ..Self::default()
        }
    }

    /// Whether every pack known is on and in place, none on its way.
    pub fn is_idle(&self) -> bool {
        self.packs
            .iter()
            .all(|pack| pack.activation == Activation::Active)
    }

    pub fn prompt_status(&self) -> PromptStatus {
        self.prompt
    }

    /// What is left to do since the last call. The machine is brought to rest first.
    pub fn take_effects(&mut self) -> Vec<Effect> {
        self.settle();
        std::mem::take(&mut self.effects)
    }

    /// The packs a reload loads, top of the stack first (`PackReloadConfig.Callbacks.packsToLoad`
    /// lists them bottom first and the loader reverses the list).
    pub fn packs_to_load(&self) -> Vec<(Uuid, PathBuf)> {
        let Some(reload) = &self.reload else {
            return Vec::new();
        };
        reload
            .load
            .iter()
            .rev()
            .filter_map(|serial| {
                let pack = self.packs.iter().find(|pack| pack.serial == *serial)?;
                Some((pack.id, pack.path.clone()?))
            })
            .collect()
    }

    fn settle(&mut self) {
        while std::mem::take(&mut self.dirty) {
            self.tick();
        }
    }

    /// `pushPack`: a pack the server sent.
    pub fn push_pack(&mut self, id: Uuid, url: String, hash: Option<String>) {
        if self.prompt == PromptStatus::Declined {
            self.effects.push(Effect::Final(id, FinalResult::Declined));
            return;
        }
        for pack in &mut self.packs {
            if pack.id == id {
                pack.remove(Removal::ServerReplaced);
            }
        }
        self.pushed += 1;
        self.packs.push(Pack::new(self.pushed, id, url, hash));
        if self.prompt == PromptStatus::Allowed {
            let index = self.packs.len() - 1;
            self.accept(index);
        }
        self.dirty = true;
    }

    fn accept(&mut self, index: usize) {
        let pack = &mut self.packs[index];
        pack.accepted = true;
        self.effects.push(Effect::Update(pack.id, Update::Accepted));
    }

    /// `popPack`.
    pub fn pop_pack(&mut self, id: Uuid) {
        if let Some(pack) = self
            .packs
            .iter_mut()
            .find(|pack| !pack.removed() && pack.id == id)
        {
            pack.remove(Removal::ServerRemoved);
            self.dirty = true;
        }
    }

    /// `popAll`.
    pub fn pop_all(&mut self) {
        for pack in &mut self.packs {
            pack.remove(Removal::ServerRemoved);
        }
        self.dirty = true;
    }

    /// `allowServerPacks`: the player agreed; the packs waiting are taken.
    pub fn allow(&mut self) {
        self.prompt = PromptStatus::Allowed;
        for index in 0..self.packs.len() {
            let pack = &self.packs[index];
            if !pack.accepted && !pack.removed() {
                self.accept(index);
            }
        }
        self.dirty = true;
    }

    /// `rejectServerPacks`: the player refused; the packs not yet taken are dropped.
    pub fn reject(&mut self) {
        self.prompt = PromptStatus::Declined;
        for pack in &mut self.packs {
            if !pack.accepted {
                pack.remove(Removal::Declined);
            }
        }
        self.dirty = true;
    }

    /// `resetPromptStatus`.
    pub fn reset_prompt(&mut self) {
        self.prompt = PromptStatus::Pending;
    }

    /// `tick`: start downloads that are due, reload when nothing is in flight, and let go of the
    /// packs that are finished.
    fn tick(&mut self) {
        if !self.update_downloads() {
            self.trigger_reload_if_needed();
        }
        self.cleanup_removed();
    }

    fn cleanup_removed(&mut self) {
        let mut kept = Vec::with_capacity(self.packs.len());
        for pack in std::mem::take(&mut self.packs) {
            let finished = pack.activation == Activation::Inactive && pack.removal.is_some();
            if !finished {
                kept.push(pack);
            } else if let Some(result) = pack.removal.and_then(Removal::response) {
                self.effects.push(Effect::Final(pack.id, result));
            }
        }
        self.packs = kept;
    }

    /// `onDownload`: how a batch of requested packs went; `files` has the path of each that
    /// arrived and `failed` the tickets of those that did not.
    pub fn downloaded(&mut self, files: Vec<(u64, PathBuf)>, failed: &[u64]) {
        if !failed.is_empty() {
            for pack in &mut self.packs {
                if pack.activation != Activation::Active {
                    pack.remove(if failed.contains(&pack.serial) {
                        Removal::DownloadFailed
                    } else {
                        Removal::Discarded
                    });
                }
            }
        }
        for (ticket, path) in files {
            for pack in self.packs.iter_mut().filter(|pack| pack.serial == ticket) {
                pack.fetch = Fetch::Done;
                pack.path = Some(path.clone());
                if !pack.removed() {
                    self.effects
                        .push(Effect::Update(pack.id, Update::Downloaded));
                }
            }
        }
        self.dirty = true;
    }

    /// `updateDownloads`: whether downloads are still going.
    fn update_downloads(&mut self) -> bool {
        let mut requests = Vec::new();
        let mut in_progress = false;
        for pack in &mut self.packs {
            if pack.removed() || !pack.accepted {
                continue;
            }
            if pack.fetch != Fetch::Done {
                in_progress = true;
            }
            if pack.fetch == Fetch::Requested {
                pack.fetch = Fetch::Pending;
                requests.push(DownloadRequest {
                    id: pack.id,
                    ticket: pack.serial,
                    url: pack.url.clone(),
                    hash: pack.hash.clone(),
                });
            }
        }
        if !requests.is_empty() {
            self.effects.push(Effect::Download(requests));
        }
        in_progress
    }

    /// `triggerReloadIfNeeded`: the packs that are to come on or go off, when there are any.
    fn plan_reload(&self) -> Option<Reload> {
        let mut plan = Reload::default();
        let mut needs_reload = false;
        for pack in &self.packs {
            if pack.activation == Activation::Pending {
                return None;
            }
            let wanted = pack.wanted();
            if wanted && pack.activation == Activation::Inactive {
                plan.load.push(pack.serial);
                needs_reload = true;
            }
            if pack.activation == Activation::Active {
                needs_reload |= !wanted;
                if wanted {
                    plan.load.push(pack.serial);
                } else {
                    plan.unload.push(pack.serial);
                }
            }
        }
        needs_reload.then_some(plan)
    }

    fn trigger_reload_if_needed(&mut self) {
        let Some(plan) = self.plan_reload() else {
            return;
        };
        for pack in &mut self.packs {
            if (plan.load.contains(&pack.serial) && pack.activation != Activation::Active)
                || plan.unload.contains(&pack.serial)
            {
                pack.activation = Activation::Pending;
            }
        }
        self.reload = Some(plan);
        self.effects.push(Effect::Reload);
    }

    /// `Callbacks.onSuccess`: the reload went through.
    pub fn reload_succeeded(&mut self) {
        let Some(reload) = self.reload.take() else {
            return;
        };
        for serial in &reload.load {
            for pack in self.packs.iter_mut().filter(|pack| pack.serial == *serial) {
                pack.activation = Activation::Active;
                if pack.removal.is_none() {
                    self.effects
                        .push(Effect::Final(pack.id, FinalResult::Applied));
                }
            }
        }
        for serial in &reload.unload {
            for pack in self.packs.iter_mut().filter(|pack| pack.serial == *serial) {
                pack.activation = Activation::Inactive;
            }
        }
        self.dirty = true;
    }

    /// `Callbacks.onFailure`: the reload did not go through. The first failure drops the packs
    /// that were to be loaded and asks for a reload with the ones that were already active (the
    /// recovery, whose own failure is reported with `recovery`).
    pub fn reload_failed(&mut self, recovery: bool) {
        if recovery {
            self.reload = None;
            for pack in &mut self.packs {
                if pack.activation == Activation::Pending {
                    pack.activation = Activation::Inactive;
                }
            }
            return;
        }
        let mut still_active = Vec::new();
        for pack in &mut self.packs {
            match pack.activation {
                Activation::Inactive => pack.remove(Removal::Discarded),
                Activation::Pending => {
                    pack.activation = Activation::Inactive;
                    pack.remove(Removal::ActivationFailed);
                }
                Activation::Active => still_active.push(pack.serial),
            }
        }
        if let Some(reload) = &mut self.reload {
            reload.load = still_active;
        }
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(n: u64) -> Uuid {
        Uuid::new(0, n)
    }

    fn request(n: u64, ticket: u64) -> DownloadRequest {
        DownloadRequest {
            id: id(n),
            ticket,
            url: format!("http://example.test/{n}.zip"),
            hash: None,
        }
    }

    fn push(manager: &mut ServerPackManager, n: u64) {
        manager.push_pack(id(n), format!("http://example.test/{n}.zip"), None);
    }

    /// The pack pushed as `ticket` arrives, under the cache path of `n`.
    fn arrive(manager: &mut ServerPackManager, (n, ticket): (u64, u64)) -> Vec<Effect> {
        manager.downloaded(vec![(ticket, PathBuf::from(format!("/cache/{n}")))], &[]);
        manager.take_effects()
    }

    /// A pack taken, fetched and loaded; the manager is left at rest.
    fn applied(manager: &mut ServerPackManager, (n, ticket): (u64, u64)) {
        push(manager, n);
        manager.take_effects();
        arrive(manager, (n, ticket));
        manager.reload_succeeded();
        manager.take_effects();
    }

    #[test]
    fn a_pack_waits_for_the_player_while_the_prompt_is_pending() {
        let mut manager = ServerPackManager::new(PromptStatus::Pending);
        push(&mut manager, 1);
        assert_eq!(manager.take_effects(), vec![]);
        manager.allow();
        assert_eq!(
            manager.take_effects(),
            vec![
                Effect::Update(id(1), Update::Accepted),
                Effect::Download(vec![request(1, 1)])
            ]
        );
    }

    #[test]
    fn an_allowed_pack_goes_accepted_downloaded_loaded() {
        let mut manager = ServerPackManager::new(PromptStatus::Allowed);
        push(&mut manager, 1);
        assert_eq!(
            manager.take_effects(),
            vec![
                Effect::Update(id(1), Update::Accepted),
                Effect::Download(vec![request(1, 1)])
            ]
        );
        assert_eq!(
            arrive(&mut manager, (1, 1)),
            vec![Effect::Update(id(1), Update::Downloaded), Effect::Reload]
        );
        assert_eq!(
            manager.packs_to_load(),
            vec![(id(1), PathBuf::from("/cache/1"))]
        );
        manager.reload_succeeded();
        assert_eq!(
            manager.take_effects(),
            vec![Effect::Final(id(1), FinalResult::Applied)]
        );
    }

    #[test]
    fn a_pack_pushed_after_a_refusal_is_declined_at_once() {
        let mut manager = ServerPackManager::new(PromptStatus::Declined);
        push(&mut manager, 1);
        assert_eq!(
            manager.take_effects(),
            vec![Effect::Final(id(1), FinalResult::Declined)]
        );
    }

    #[test]
    fn refusing_declines_the_packs_that_were_waiting() {
        let mut manager = ServerPackManager::new(PromptStatus::Pending);
        push(&mut manager, 1);
        push(&mut manager, 2);
        manager.reject();
        assert_eq!(
            manager.take_effects(),
            vec![
                Effect::Final(id(1), FinalResult::Declined),
                Effect::Final(id(2), FinalResult::Declined)
            ]
        );
    }

    #[test]
    fn a_failed_download_is_reported_and_the_others_are_discarded() {
        let mut manager = ServerPackManager::new(PromptStatus::Allowed);
        push(&mut manager, 1);
        push(&mut manager, 2);
        manager.take_effects();
        manager.downloaded(vec![], &[2]);
        assert_eq!(
            manager.take_effects(),
            vec![
                Effect::Final(id(1), FinalResult::Discarded),
                Effect::Final(id(2), FinalResult::DownloadFailed)
            ]
        );
    }

    #[test]
    fn a_pack_that_cannot_be_loaded_is_reported_and_its_recovery_keeps_the_active_ones() {
        let mut manager = ServerPackManager::new(PromptStatus::Allowed);
        applied(&mut manager, (1, 1));
        push(&mut manager, 2);
        manager.take_effects();
        assert_eq!(
            arrive(&mut manager, (2, 2)),
            vec![Effect::Update(id(2), Update::Downloaded), Effect::Reload]
        );
        assert_eq!(manager.packs_to_load().len(), 2);
        manager.reload_failed(false);
        assert_eq!(
            manager.take_effects(),
            vec![Effect::Final(id(2), FinalResult::ActivationFailed)]
        );
        assert_eq!(
            manager.packs_to_load(),
            vec![(id(1), PathBuf::from("/cache/1"))]
        );
        manager.reload_succeeded();
        assert_eq!(
            manager.take_effects(),
            vec![Effect::Final(id(1), FinalResult::Applied)]
        );
    }

    #[test]
    fn popping_an_active_pack_reloads_without_a_report() {
        let mut manager = ServerPackManager::new(PromptStatus::Allowed);
        applied(&mut manager, (1, 1));
        manager.pop_pack(id(1));
        assert_eq!(manager.take_effects(), vec![Effect::Reload]);
        assert_eq!(manager.packs_to_load(), vec![]);
        manager.reload_succeeded();
        assert_eq!(manager.take_effects(), vec![]);
        // Nothing is left to pop.
        manager.pop_all();
        assert_eq!(manager.take_effects(), vec![]);
    }

    #[test]
    fn a_pack_pushed_again_replaces_the_first_silently() {
        let mut manager = ServerPackManager::new(PromptStatus::Allowed);
        applied(&mut manager, (1, 1));
        push(&mut manager, 1);
        manager.take_effects();
        arrive(&mut manager, (1, 2));
        manager.reload_succeeded();
        // The old copy was taken off and the new one put on: only the new one is reported.
        assert_eq!(
            manager.take_effects(),
            vec![Effect::Final(id(1), FinalResult::Applied)]
        );
    }

    #[test]
    fn only_forty_hex_digits_are_a_hash() {
        assert_eq!(
            parse_sha1("ABCDEF0123456789abcdef0123456789ABCDEF01").as_deref(),
            Some("abcdef0123456789abcdef0123456789abcdef01")
        );
        assert_eq!(parse_sha1("abc"), None);
        assert_eq!(parse_sha1(&"g".repeat(40)), None);
    }
}
