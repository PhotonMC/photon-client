//! The server resource packs on the client's side (`Minecraft.getDownloadedPackSource`): the one
//! source the connections share, the setting each server in the list keeps for it, and what the
//! window does when the packs ask for a reload.

use std::sync::OnceLock;

use photon_game::servers::{PackStatus, ServerList};
use photon_net::server_packs::{HttpFetcher, PackSource};

/// The source of the window's connections, which asks the player and downloads into the
/// `downloads` folder of the game directory (`Minecraft.downloadedPackSource`).
pub(crate) fn shared() -> PackSource {
    static SHARED: OnceLock<PackSource> = OnceLock::new();
    SHARED.get_or_init(downloading).clone()
}

fn downloading() -> PackSource {
    match HttpFetcher::new(crate::controls::game_directory().join("downloads")) {
        Ok(fetcher) => PackSource::new(std::sync::Arc::new(fetcher), true),
        Err(error) => {
            log::error!("failed to open the pack download folder: {error}");
            PackSource::offline()
        }
    }
}

/// A source for a connection nobody is there to ask for (scripts and probes): packs are declined
/// unless `PHOTON_SERVER_PACKS` says `enabled`.
pub(crate) fn unattended() -> PackSource {
    let source = match HttpFetcher::new(crate::controls::game_directory().join("downloads")) {
        Ok(fetcher) => PackSource::new(std::sync::Arc::new(fetcher), false),
        Err(_) => PackSource::default(),
    };
    source.set_setting(setting_from_environment().unwrap_or(PackStatus::Disabled));
    source
}

fn setting_from_environment() -> Option<PackStatus> {
    match std::env::var("PHOTON_SERVER_PACKS").ok()?.as_str() {
        "enabled" => Some(PackStatus::Enabled),
        "disabled" => Some(PackStatus::Disabled),
        "prompt" => Some(PackStatus::Prompt),
        _ => None,
    }
}

/// The next connection goes to the server `ip` of the list (none for one that is not in it):
/// its setting decides what is done with a pack, and the player's answer is kept in the list.
pub(crate) fn prepare(source: &PackSource, ip: Option<&str>) {
    let directory = crate::controls::game_directory();
    let listed = ip.and_then(|ip| ServerList::load(&directory).find(ip).map(|s| s.pack_status));
    source.set_setting(
        listed
            .or_else(setting_from_environment)
            .unwrap_or(PackStatus::Prompt),
    );
    let ip = ip.map(str::to_owned);
    source.set_remember(move |setting| {
        let Some(ip) = ip.as_deref() else {
            return;
        };
        let mut list = ServerList::load(&directory);
        let server = list
            .servers
            .iter_mut()
            .chain(&mut list.hidden)
            .find(|server| server.ip == ip);
        if let Some(server) = server {
            server.pack_status = setting;
            if let Err(error) = list.save(&directory) {
                log::warn!("the server list was not saved: {error}");
            }
        }
    });
}

/// The packs a reload loads: their ids and where they are kept.
pub(crate) type LoadedPacks = [(photon_core::binary::Uuid, std::path::PathBuf)];

/// Which try of a reload an answer is for.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Attempt {
    /// The packs the reload asked for.
    First,
    /// The packs that were in use, after the first failed.
    Recovery,
}

/// Tell the packs how an attempt went. A failed first attempt is answered with the recovery the
/// packs ask for, the ones that were in use before, which the caller then loads as the second
/// attempt.
pub(crate) fn answer_attempt(
    source: &PackSource,
    attempt: Attempt,
    loaded: bool,
) -> Option<Vec<(photon_core::binary::Uuid, std::path::PathBuf)>> {
    match (attempt, loaded) {
        (_, true) => source.reload_succeeded(),
        (Attempt::First, false) => return Some(source.reload_failed(false)),
        (Attempt::Recovery, false) => {
            source.reload_failed(true);
        }
    }
    None
}

/// Answer the reload the packs asked for, if one is due, with `reload` loading the packs it is
/// given (the top of the stack first) and saying whether that went through.
pub(crate) fn serve_reload(source: &PackSource, reload: &mut dyn FnMut(&LoadedPacks) -> bool) {
    let Some(packs) = source.take_reload() else {
        return;
    };
    if let Some(recovery) = answer_attempt(source, Attempt::First, reload(&packs)) {
        answer_attempt(source, Attempt::Recovery, reload(&recovery));
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    use photon_core::binary::Uuid;
    use photon_net::server_packs::{DownloadRequest, PackFetcher, PackSink, PackUser};
    use photon_protocol::play::PackRequest;

    use super::*;

    struct Fetch;

    impl PackFetcher for Fetch {
        fn fetch(&self, _: &DownloadRequest, _: &PackUser) -> Result<PathBuf, String> {
            Ok(PathBuf::from("/cache/pack"))
        }
    }

    /// A source with one pack pushed and downloaded, and the statuses it sends.
    fn pushed() -> (PackSource, Arc<Mutex<Vec<String>>>) {
        let source = PackSource::new(Arc::new(Fetch), true);
        source.set_setting(PackStatus::Enabled);
        let seen = Arc::new(Mutex::new(Vec::new()));
        let record = Arc::clone(&seen);
        source.configure_for_server(PackSink::new(move |_, action| {
            record.lock().unwrap().push(format!("{action:?}"));
        }));
        source.push(PackRequest {
            id: Uuid::new(1, 2),
            url: "https://example.com/pack.zip".to_owned(),
            hash: "0123456789abcdef0123456789abcdef01234567".to_owned(),
            required: false,
            prompt: None,
        });
        (source, seen)
    }

    /// Serve the reload once it is due, with the packs it asked for.
    fn serve(source: &PackSource, reload: &mut dyn FnMut(&LoadedPacks) -> bool) {
        let started = Instant::now();
        let mut served = false;
        while !served {
            serve_reload(source, &mut |packs| {
                served = true;
                reload(packs)
            });
            assert!(started.elapsed() < Duration::from_secs(5), "no reload");
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[test]
    fn nothing_is_reloaded_when_no_reload_is_due() {
        let mut called = false;
        serve_reload(&PackSource::offline(), &mut |_| {
            called = true;
            true
        });
        assert!(!called);
    }

    #[test]
    fn a_pack_that_loads_is_reported_loaded() {
        let (source, seen) = pushed();
        let mut loads = 0;
        serve(&source, &mut |packs| {
            loads += 1;
            assert_eq!(packs.len(), 1);
            true
        });
        assert_eq!(loads, 1);
        assert_eq!(
            *seen.lock().unwrap(),
            ["Accepted", "Downloaded", "SuccessfullyLoaded"]
        );
    }

    #[test]
    fn a_pack_that_fails_is_reported_and_the_old_packs_come_back() {
        let (source, seen) = pushed();
        let mut loads = Vec::new();
        serve(&source, &mut |packs| {
            loads.push(packs.len());
            loads.len() > 1
        });
        assert_eq!(loads, [1, 0]);
        assert_eq!(
            *seen.lock().unwrap(),
            ["Accepted", "Downloaded", "FailedReload"]
        );
        assert!(source.is_idle());
    }
}
