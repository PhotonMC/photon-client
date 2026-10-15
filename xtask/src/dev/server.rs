//! `cargo xtask server`: set up and start the offline test server.

use std::fs::{self, File};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use super::args::{Spec, parse};
use super::java;
use super::layout::{RCON_PASSWORD, RCON_PORT, SERVER_PORT, runtime, server_dir};
use super::rcon;
use crate::parity::pins::Pins;

/// The settings the tools rely on: offline, peaceful, seeded, and reachable
/// by RCON so scripts can set the scene.
fn properties() -> Vec<(&'static str, String)> {
    let fixed = [
        ("online-mode", "false"),
        ("level-name", "walk"),
        ("level-seed", "776"),
        ("gamemode", "survival"),
        ("force-gamemode", "true"),
        ("difficulty", "peaceful"),
        ("spawn-monsters", "false"),
        ("spawn-protection", "0"),
        ("view-distance", "32"),
        ("simulation-distance", "6"),
        ("pause-when-empty-seconds", "60"),
        ("enable-rcon", "true"),
        ("broadcast-rcon-to-ops", "true"),
        ("motd", "photon test server"),
    ];
    let mut all = vec![("server-port", SERVER_PORT.to_string())];
    all.extend(fixed.iter().map(|(key, value)| (*key, (*value).to_owned())));
    all.push(("rcon.port", RCON_PORT.to_string()));
    all.push(("rcon.password", RCON_PASSWORD.to_owned()));
    all
}

/// Writes the settings over whatever is in `server.properties`; the rest of
/// the file is left alone.
fn write_properties(directory: &Path) -> Result<(), String> {
    let path = directory.join("server.properties");
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let mut wanted = properties();
    let mut lines = Vec::new();
    for line in existing.lines() {
        let key = if line.starts_with('#') {
            None
        } else {
            line.split_once('=').map(|(key, _)| key)
        };
        match key.and_then(|key| wanted.iter().position(|(name, _)| *name == key)) {
            Some(index) => {
                let (name, value) = wanted.remove(index);
                lines.push(format!("{name}={value}"));
            }
            None => lines.push(line.to_owned()),
        }
    }
    lines.extend(
        wanted
            .into_iter()
            .map(|(name, value)| format!("{name}={value}")),
    );
    fs::write(&path, lines.join("\n") + "\n")
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn check_eula(directory: &Path, accepted: bool) -> Result<(), String> {
    let path = directory.join("eula.txt");
    if fs::read_to_string(&path).is_ok_and(|text| text.contains("eula=true")) {
        return Ok(());
    }
    if !accepted {
        return Err(
            "The server needs the Minecraft EULA (https://aka.ms/MinecraftEULA) accepted. \
                    Read it, then run again with --accept-eula."
                .to_owned(),
        );
    }
    fs::write(&path, "eula=true\n").map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(unix)]
fn detach(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(windows)]
fn detach(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
}

fn listening(port: u16) -> bool {
    TcpStream::connect_timeout(&([127, 0, 0, 1], port).into(), Duration::from_secs(1)).is_ok()
}

/// Waits until both the game port and RCON, which opens a little later, take connections.
fn wait_ready(child: &mut Child, directory: &Path) -> Result<(), String> {
    let mut waiting = vec![SERVER_PORT, RCON_PORT];
    for _ in 0..180 {
        if child
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return Err("the server exited; see the log".to_owned());
        }
        if listening(waiting[0]) {
            waiting.remove(0);
        } else {
            sleep(Duration::from_secs(1));
        }
        if waiting.is_empty() {
            println!("ready: game on 127.0.0.1:{SERVER_PORT}, RCON on {RCON_PORT}");
            return Ok(());
        }
    }
    println!(
        "the server is not ready yet; see {}",
        directory.join("server.log").display()
    );
    Ok(())
}

fn start(
    java: &Path,
    jar: &Path,
    directory: &Path,
    memory: &str,
    background: bool,
) -> Result<(), String> {
    let mut command = Command::new(java);
    command
        .arg(format!("-Xmx{memory}"))
        .arg("-jar")
        .arg(jar)
        .arg("nogui")
        .current_dir(directory);
    if !background {
        println!("starting the server on port {SERVER_PORT} (stop with `stop` or Ctrl+C)");
        return command
            .status()
            .map(drop)
            .map_err(|error| error.to_string());
    }
    let log_path = directory.join("server.log");
    let log =
        File::create(&log_path).map_err(|error| format!("{}: {error}", log_path.display()))?;
    command
        .stdin(Stdio::null())
        .stdout(log.try_clone().map_err(|error| error.to_string())?)
        .stderr(log);
    detach(&mut command);
    let mut child = command
        .spawn()
        .map_err(|error| format!("cannot start {}: {error}", java.display()))?;
    println!(
        "server started in the background (pid {}), log: {}",
        child.id(),
        log_path.display()
    );
    wait_ready(&mut child, directory)
}

const SPEC: Spec = Spec {
    flags: &["--accept-eula", "--background", "--stop"],
    options: &[("--dir", 1), ("--memory", 1), ("--java", 1)],
};

pub fn server(args: &[String]) -> Result<(), String> {
    let parsed = parse(args, &SPEC)?;
    if parsed.flag("--stop") {
        println!("{}", rcon::send(&["stop".to_owned()])?.join("\n"));
        return Ok(());
    }
    let pins = Pins::load()?;
    let jar = runtime().join(format!("server-{}.jar", pins.version));
    if !jar.is_file() {
        return Err(format!(
            "{} is missing: run `cargo xtask setup` first",
            jar.display()
        ));
    }
    let directory = parsed.value("--dir").map_or_else(server_dir, PathBuf::from);
    fs::create_dir_all(&directory).map_err(|error| format!("{}: {error}", directory.display()))?;
    check_eula(&directory, parsed.flag("--accept-eula"))?;
    write_properties(&directory)?;
    let java = java::find(parsed.value("--java"), pins.java_major)?;
    let memory = parsed.value("--memory").unwrap_or("2G");
    start(&java, &jar, &directory, memory, parsed.flag("--background"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tool_settings_replace_old_values_and_the_rest_is_kept() {
        let dir = std::env::temp_dir().join(format!("photon-server-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        fs::write(
            dir.join("server.properties"),
            "# comment\nmotd=mine\nview-distance=8\nmax-players=5\n",
        )
        .expect("write");
        write_properties(&dir).expect("merge");
        let text = fs::read_to_string(dir.join("server.properties")).expect("read");
        assert!(
            text.starts_with(
                "# comment\nmotd=photon test server\nview-distance=32\nmax-players=5\n"
            )
        );
        assert!(text.contains("server-port=25600\n") && text.contains("rcon.password=walk\n"));
        assert_eq!(text.matches("motd=").count(), 1);
        fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn the_eula_needs_consent() {
        let dir = std::env::temp_dir().join(format!("photon-eula-{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        assert!(check_eula(&dir, false).is_err());
        assert!(check_eula(&dir, true).is_ok());
        assert!(check_eula(&dir, false).is_ok());
        fs::remove_dir_all(&dir).expect("cleanup");
    }
}
