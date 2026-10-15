//! `cargo xtask shot`: a scripted client run that saves its first frame.

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use super::args::{Spec, parse};
use super::layout::{client_binary, client_environment, data, default_address};
use super::png::ppm_to_png;
use super::rcon;
use crate::process;

const SPEC: Spec = Spec {
    flags: &[],
    options: &[
        ("--user", 1),
        ("--address", 1),
        ("--rcon", 1),
        ("--wait", 1),
    ],
};

/// Removes the files of an earlier shot with the same name.
fn clear_stale(directory: &Path, name: &str) -> Result<(), String> {
    let prefix = format!("{name}.");
    for entry in fs::read_dir(directory)
        .map_err(|error| error.to_string())?
        .flatten()
    {
        if entry.file_name().to_string_lossy().starts_with(&prefix) {
            fs::remove_file(entry.path()).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn clip(reply: &str) -> String {
    reply.chars().take(80).collect()
}

/// Runs the client with a script, sends RCON commands once joined, and saves the first frame.
pub fn shot(args: &[String]) -> Result<(), String> {
    let parsed = parse(args, &SPEC)?;
    let [name, script] = parsed.positional.as_slice() else {
        return Err("usage: cargo xtask shot NAME \"SCRIPT\" [--user U] [--address A] [--rcon CMD]... [--wait S]".to_owned());
    };
    let directory = data().join("runs").join("shots");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    clear_stale(&directory, name)?;
    let prefix = directory.join(name);
    let log_path = PathBuf::from(format!("{}.log", prefix.display()));
    let log =
        File::create(&log_path).map_err(|error| format!("{}: {error}", log_path.display()))?;
    let address = parsed
        .value("--address")
        .map_or_else(default_address, str::to_owned);
    let mut child = Command::new(client_binary())
        .current_dir(process::root())
        .envs(client_environment()?)
        .env("PHOTON_USER", parsed.value("--user").unwrap_or("Miner"))
        .env("PHOTON_SCRIPT", script)
        .env("PHOTON_CAPTURE_PPM", &prefix)
        .env("PHOTON_CAPTURE_FRAMES", "1")
        .args(["play", &address, "854", "480"])
        .stdout(log.try_clone().map_err(|error| error.to_string())?)
        .stderr(log)
        .stdin(Stdio::null())
        .spawn()
        .map_err(|error| format!("cannot start the client: {error}"))?;
    let commands: Vec<String> = parsed
        .values("--rcon")
        .into_iter()
        .map(str::to_owned)
        .collect();
    if !commands.is_empty() {
        sleep(Duration::from_secs_f64(parsed.number("--wait", 9.0)?));
        for (command, reply) in commands.iter().zip(rcon::send(&commands)?) {
            println!("{command} -> {}", clip(&reply));
        }
    }
    child.wait().map_err(|error| error.to_string())?;
    let frame = PathBuf::from(format!("{}.000.ppm", prefix.display()));
    let png = PathBuf::from(format!("{}.png", prefix.display()));
    if frame.exists() {
        ppm_to_png(&frame, &png)?;
        println!("frame: {}", png.display());
    } else {
        println!("frame: {}", frame.display());
    }
    let text = fs::read_to_string(&log_path).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    for line in &lines[lines.len().saturating_sub(2)..] {
        println!("{line}");
    }
    Ok(())
}
