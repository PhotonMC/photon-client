//! The start of a release the client downloaded earlier: before any window opens, a staged
//! release is checked again and swapped in, and the new binary is started with the same arguments.

use std::path::PathBuf;

/// Applies the staged release, if there is one and the build has a release key. On success this
/// does not return on Unix (the new binary replaces this process); elsewhere it exits with the new
/// process's status. Any problem is logged and the current install keeps running.
pub fn apply_pending() {
    if photon_update::RELEASE_KEYS.is_empty() {
        return;
    }
    let Some(pending) = pending_folder() else {
        return;
    };
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let (Some(install), Some(binary)) = (exe.parent(), exe.file_name().and_then(|n| n.to_str()))
    else {
        return;
    };
    let current = env!("CARGO_PKG_VERSION");
    match photon_update::apply(
        install,
        &pending,
        photon_update::RELEASE_KEYS,
        (current, binary),
    ) {
        Ok(None) => {}
        Ok(Some(applied)) => {
            log::info!("updated to {}", applied.version);
            restart(&applied.binary);
        }
        Err(error) => log::warn!("the staged update was not applied: {error:?}"),
    }
}

/// `<user data>/photon/update/pending`, beside the content cache.
fn pending_folder() -> Option<PathBuf> {
    let cache = photon_content::Cache::user_level().ok()?;
    Some(cache.root().parent()?.join("update").join("pending"))
}

fn restart(binary: &std::path::Path) {
    let mut command = std::process::Command::new(binary);
    command.args(std::env::args_os().skip(1));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let error = command.exec();
        log::warn!("the updated binary did not start: {error}");
    }
    #[cfg(not(unix))]
    match command.status() {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(error) => log::warn!("the updated binary did not start: {error}"),
    }
}
