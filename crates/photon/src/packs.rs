//! The server resource packs on the client's side (`Minecraft.getDownloadedPackSource`): the one
//! source the connections share, the setting each server in the list keeps for it, and what the
//! window does when the packs ask for a reload.

use std::sync::OnceLock;

use photon_game::servers::{PackStatus, ServerList};
use photon_net::server_packs::PackSource;

/// The source of the window's connections, which asks the player.
pub(crate) fn shared() -> PackSource {
    static SHARED: OnceLock<PackSource> = OnceLock::new();
    SHARED.get_or_init(PackSource::offline).clone()
}

/// A source for a connection nobody is there to ask for (scripts and probes): packs are declined
/// unless `PHOTON_SERVER_PACKS` says `enabled`.
pub(crate) fn unattended() -> PackSource {
    let source = PackSource::default();
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

/// Answer the reload the packs asked for, if one is due. The assets are not reloaded yet, so
/// the packs are reported as ones that could not be put to use.
pub(crate) fn serve_reload(source: &PackSource) {
    if source.take_reload().is_some() {
        log::warn!("server packs cannot be loaded: the assets are not reloaded yet");
        let recovery = source.reload_failed(false);
        if recovery.is_empty() {
            source.reload_succeeded();
        } else {
            source.reload_failed(true);
        }
    }
}
