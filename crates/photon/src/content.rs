//! The game's own assets of an installed Photon (ADR 0007): found in the content cache at start,
//! and downloaded by `photon install` once the stored account is shown to own the game. A
//! development run names its assets with `PHOTON_ASSETS` and never comes here.

use photon_content::{Cache, Progress, Stage};
use photon_net::signin::{self, ClientId, Hosts};

/// Point the readers of the assets at the installed game, unless `PHOTON_ASSETS` already does.
pub(crate) fn use_installed() {
    if photon_core::content_root::assets().is_ok() {
        return;
    }
    let installed = Cache::user_level()
        .ok()
        .and_then(|cache| photon_content::installed(&cache));
    match installed {
        Some(game) => {
            let root = game.jar_assets.to_string_lossy().into_owned();
            photon_core::content_root::set_installed_assets(root);
        }
        None => log::info!(
            "the game's assets are not installed; run `photon sign-in`, then `photon install`"
        ),
    }
}

/// `photon install`: download the game into the user-level cache, as the stored account.
pub(crate) fn run_install() -> Result<(), Box<dyn std::error::Error>> {
    let cache = Cache::user_level()?;
    let host = crate::profile_login::session_host();
    let store = crate::keyring_store::os_store();
    let resumed = signin::resume(
        &Hosts::default(),
        &ClientId::from_environment(),
        (&host, store.as_ref()),
    )?;
    let Some(session) = resumed else {
        return Err("nobody is signed in; run `photon sign-in` first".into());
    };
    let mut last = None;
    let game = photon_content::install(&session.ownership, &cache, &mut |progress| {
        if let Some(line) = progress_line(progress, &mut last) {
            println!("{line}");
        }
    })?;
    println!(
        "Installed in {} ({} files fetched)",
        game.root.display(),
        game.downloaded
    );
    Ok(())
}

/// What to print for a step of the download: each stage once when it starts, the assets by tenths.
fn progress_line(progress: Progress, last: &mut Option<(Stage, u64)>) -> Option<String> {
    let Progress { stage, done, total } = progress;
    let tenth = (done * 10).checked_div(total).unwrap_or(0);
    let key = (stage, if stage == Stage::Assets { tenth } else { 0 });
    if *last == Some(key) {
        return None;
    }
    *last = Some(key);
    Some(match stage {
        Stage::Metadata => "Reading the version information".to_owned(),
        Stage::Jar => "Downloading the game".to_owned(),
        Stage::Assets => format!("Downloading assets: {done} of {total}"),
        Stage::Extract => "Unpacking the game".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stage_is_reported_once_and_the_assets_by_tenths() {
        let mut last = None;
        let mut lines = Vec::new();
        let mut step = |stage, done, total| {
            lines.extend(progress_line(Progress { stage, done, total }, &mut last));
        };
        step(Stage::Metadata, 0, 1);
        step(Stage::Metadata, 1, 1);
        step(Stage::Jar, 0, 5);
        step(Stage::Jar, 5, 5);
        for done in 0..=100 {
            step(Stage::Assets, done, 100);
        }
        step(Stage::Extract, 0, 1);
        assert_eq!(lines.len(), 1 + 1 + 11 + 1);
        assert_eq!(lines[2], "Downloading assets: 0 of 100");
        assert_eq!(lines[12], "Downloading assets: 100 of 100");
    }
}
