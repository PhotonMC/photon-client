//! Where the developer commands keep things, and what they run.

use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use crate::process;

pub const SERVER_PORT: u16 = 25600;
pub const RCON_PORT: u16 = 25601;
pub const RCON_PASSWORD: &str = "walk";

pub fn data() -> PathBuf {
    process::root().join(".data")
}

pub fn minecraft() -> PathBuf {
    data().join("minecraft")
}

pub fn runtime() -> PathBuf {
    minecraft().join("runtime")
}

pub fn assets() -> PathBuf {
    minecraft().join("assets")
}

pub fn server_dir() -> PathBuf {
    data().join("render").join("walk")
}

pub fn default_address() -> String {
    format!("127.0.0.1:{SERVER_PORT}")
}

/// The unpacked client resources, or the reason they are missing.
pub fn jar_assets() -> Result<PathBuf, String> {
    let path = assets().join("jar").join("assets");
    if path.is_dir() {
        Ok(path)
    } else {
        Err(format!(
            "{} is missing: run `cargo xtask setup` first",
            path.display()
        ))
    }
}

pub fn client_binary() -> PathBuf {
    let name = if cfg!(windows) {
        "photon.exe"
    } else {
        "photon"
    };
    process::root().join("target").join("release").join(name)
}

/// The environment the client runs in: the assets, and the Vulkan loader on macOS.
pub fn client_environment() -> Result<Vec<(String, String)>, String> {
    let mut environment = vec![(
        "PHOTON_ASSETS".to_owned(),
        jar_assets()?.display().to_string(),
    )];
    if cfg!(target_os = "macos") && std::path::Path::new("/usr/local/lib").is_dir() {
        environment.push(("DYLD_LIBRARY_PATH".to_owned(), "/usr/local/lib".to_owned()));
    }
    Ok(environment)
}

/// `cargo build --release` of the client (or of one package).
pub fn build_client(package: Option<&str>) -> Result<(), String> {
    match package {
        Some(name) => process::run("cargo", &["build", "--release", "-p", name]),
        None => process::run("cargo", &["build", "--release"]),
    }
}

/// Runs `command` to the end and returns its exit status with stdout and
/// stderr merged in the order they were written.
pub fn run_merged(mut command: Command) -> Result<(std::process::ExitStatus, String), String> {
    let (mut reader, writer) = std::io::pipe().map_err(|error| error.to_string())?;
    command
        .stdout(writer.try_clone().map_err(|error| error.to_string())?)
        .stderr(writer);
    let mut child = command
        .spawn()
        .map_err(|error| format!("cannot start the client: {error}"))?;
    drop(command);
    let mut text = String::new();
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    text.push_str(&String::from_utf8_lossy(&bytes));
    let status = child.wait().map_err(|error| error.to_string())?;
    Ok((status, text))
}
