//! The updater as the client runs it: one background service for the whole process, its changes
//! logged and fired as events while a game is played, and the restart that applies a stored
//! release ([`crate::self_update`] swaps it in when the new process starts).

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

use photon_api::{ClientEvents, UpdateChanged, UpdateState};
use photon_update::{
    CHECK_INTERVAL, Config, PLATFORM, RELEASE_HOST, RELEASE_KEYS, Service, Settings,
};

static SERVICE: OnceLock<Mutex<Service>> = OnceLock::new();
static RESTART: AtomicBool = AtomicBool::new(false);

/// The folder the updater keeps its files in, beside the content cache.
pub(crate) fn update_folder() -> Option<PathBuf> {
    let cache = photon_content::Cache::user_level().ok()?;
    Some(cache.root().parent()?.join("update"))
}

/// What the service needs: the update folder's `settings.txt` (`auto_check`, `auto_download` and
/// `channel` lines; a missing file means the defaults and a wrong one is logged and ignored).
fn config(folder: &std::path::Path, settings: Option<Settings>) -> Config {
    let settings =
        settings.unwrap_or_else(
            || match std::fs::read_to_string(folder.join("settings.txt")) {
                Ok(text) => Settings::parse(&text).unwrap_or_else(|problem| {
                    log::warn!("update settings ignored: {problem}");
                    Settings::default()
                }),
                Err(_) => Settings::default(),
            },
        );
    Config {
        host: RELEASE_HOST.to_owned(),
        keys: RELEASE_KEYS.to_vec(),
        platform: PLATFORM.to_owned(),
        current: env!("CARGO_PKG_VERSION").to_owned(),
        pending: folder.join("pending"),
        settings,
        interval: CHECK_INTERVAL,
    }
}

/// Starts the service once, with the settings file's choices.
pub(crate) fn start() {
    let Some(folder) = update_folder() else {
        return;
    };
    let _ = SERVICE.set(Mutex::new(Service::start(config(&folder, None))));
}

/// What `photon update` learns from one state: `Some` when the run is over.
fn report(state: &UpdateState, checked: bool) -> Option<Result<(), String>> {
    match state {
        UpdateState::Available {
            version,
            notes,
            size,
        } => println!("update {version} ({size} bytes) is available: {notes}"),
        UpdateState::Downloading { done, total } if done == total => {
            println!("downloaded {total} bytes");
        }
        UpdateState::Ready { version } => {
            println!("update {version} is stored and applies at the next start");
            return Some(Ok(()));
        }
        UpdateState::Idle if checked => {
            println!("photon {} is up to date", env!("CARGO_PKG_VERSION"));
            return Some(Ok(()));
        }
        UpdateState::Failed { reason } => return Some(Err(reason.clone())),
        _ => {}
    }
    None
}

/// `photon update`: asks the host once, downloads what it finds and prints each state.
pub(crate) fn run_update() -> Result<(), Box<dyn std::error::Error>> {
    let folder = update_folder().ok_or("no user data folder")?;
    let manual = Settings {
        auto_check: false,
        auto_download: false,
    };
    let service = Service::start(config(&folder, Some(manual)));
    service.check_now();
    let end = std::time::Instant::now() + std::time::Duration::from_secs(600);
    let (mut asked, mut checked) = (false, false);
    while std::time::Instant::now() < end {
        for state in service.poll() {
            checked |= state == UpdateState::Checking;
            if let Some(done) = report(&state, checked) {
                return done.map_err(Into::into);
            }
            if matches!(state, UpdateState::Available { .. })
                && !std::mem::replace(&mut asked, true)
            {
                service.download();
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    Err("the update did not finish in ten minutes".into())
}

fn service() -> Option<MutexGuard<'static, Service>> {
    SERVICE.get().map(|service| {
        service
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    })
}

/// The changes of state since the last call, logged. Called from the loop that owns the window.
pub(crate) fn poll() -> Vec<UpdateState> {
    let changes = service().map(|service| service.poll()).unwrap_or_default();
    for state in &changes {
        match state {
            UpdateState::Available { version, .. } => log::info!("update {version} is available"),
            UpdateState::Ready { version } => {
                log::info!("update {version} is ready and applies at the next start");
            }
            UpdateState::Failed { reason } => log::info!("update check failed: {reason}"),
            UpdateState::Idle | UpdateState::Checking | UpdateState::Downloading { .. } => {}
        }
    }
    changes
}

/// [`poll`], with each change fired to the handlers of `events`.
pub(crate) fn fire(events: &mut ClientEvents) {
    for state in poll() {
        events.update_changed.fire(&UpdateChanged(state));
    }
}

/// Ends the game cleanly (the window closes, the connection with it) and starts this program again
/// with the same arguments, which applies a stored release before anything else. Options are
/// written when they change, so nothing is left to save.
pub(crate) fn restart_into_update() {
    RESTART.store(true, Ordering::Relaxed);
}

/// Whether [`restart_into_update`] was called: the loops stop and `main` starts again.
pub(crate) fn restart_requested() -> bool {
    RESTART.load(Ordering::Relaxed)
}
