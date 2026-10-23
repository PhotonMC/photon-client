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

/// Answer the reload the packs asked for, if one is due. `reload` puts the packs it is given (the
/// top of the stack first) in use and says whether that went through. A failure is answered with
/// the recovery the packs ask for, the ones that were in use before.
pub(crate) fn serve_reload(source: &PackSource, reload: &mut dyn FnMut(&LoadedPacks) -> bool) {
    let Some(packs) = source.take_reload() else {
        return;
    };
    if reload(&packs) {
        source.reload_succeeded();
        return;
    }
    let recovery = source.reload_failed(false);
    if reload(&recovery) {
        source.reload_succeeded();
    } else {
        source.reload_failed(true);
    }
}

/// A reload that cannot be done: the assets are fixed once a game has started, so the packs
/// are reported as ones that could not be put to use.
pub(crate) fn cannot_reload(_: &[(photon_core::binary::Uuid, std::path::PathBuf)]) -> bool {
    log::warn!("server packs cannot be loaded while a game is running");
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_reloaded_when_no_reload_is_due() {
        let mut called = false;
        serve_reload(&PackSource::offline(), &mut |_| {
            called = true;
            true
        });
        assert!(!called);
    }
}
