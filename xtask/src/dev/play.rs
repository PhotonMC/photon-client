//! `cargo xtask play`: build and start the client on its title screen, or with `--direct` joined
//! to the server at once.

use std::process::Command;

use super::args::{Spec, parse};
use super::layout::{build_client, client_binary, client_environment, default_address};
use crate::process;

const SPEC: Spec = Spec {
    flags: &["--fullscreen", "--profile", "--no-build", "--direct"],
    options: &[("--user", 1), ("--size", 2), ("--camera", 1)],
};

pub fn play(args: &[String]) -> Result<(), String> {
    let parsed = parse(args, &SPEC)?;
    if !parsed.flag("--no-build") {
        build_client(None)?;
    }
    let mut command = Command::new(client_binary());
    command
        .current_dir(process::root())
        .envs(client_environment()?);
    if let Some(user) = parsed.value("--user") {
        command.env("PHOTON_USER", user);
        command.env("PHOTON_OFFLINE", "1");
    }
    if parsed.flag("--fullscreen") {
        command.env("PHOTON_FULLSCREEN", "1");
    }
    if let Some(camera) = parsed.value("--camera") {
        if !["back", "front"].contains(&camera) {
            return Err("--camera is back or front".to_owned());
        }
        command.env("PHOTON_CAMERA", camera);
    }
    if parsed.flag("--profile") {
        command.env("PHOTON_PROFILE", "1");
    }
    // The menu takes the address only for a scripted join; the player picks a server there.
    let mode = if parsed.flag("--direct") {
        "play"
    } else {
        "menu"
    };
    command.arg(mode).arg(parsed.address(&default_address()));
    command.args(parsed.group("--size").unwrap_or(&[]));
    let status = command
        .status()
        .map_err(|error| format!("cannot start the client: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("the client exited with {status}"))
    }
}
