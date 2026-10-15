//! Finding a `java` of the pinned major version.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn executable() -> &'static str {
    if cfg!(windows) { "java.exe" } else { "java" }
}

fn candidates(explicit: Option<&str>) -> Vec<PathBuf> {
    let mut found = Vec::new();
    if let Some(path) = explicit {
        found.push(PathBuf::from(path));
    }
    if let Some(home) = env::var_os("JAVA_HOME") {
        found.push(PathBuf::from(home).join("bin").join(executable()));
    }
    if let Some(path) = env::var_os("PATH") {
        found.extend(env::split_paths(&path).map(|dir| dir.join(executable())));
    }
    found
}

/// The first Java major version printed by `java -version`.
fn major_of(java: &PathBuf) -> Option<u32> {
    let output = Command::new(java).arg("-version").output().ok()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    text.replace('"', " ")
        .split_whitespace()
        .filter_map(|token| token.split('.').next()?.parse::<u32>().ok())
        .find(|major| *major >= 8)
}

/// A `java` of `major`: `--java`, then `JAVA_HOME`, then `PATH`.
pub fn find(explicit: Option<&str>, major: u32) -> Result<PathBuf, String> {
    candidates(explicit)
        .into_iter()
        .filter(|candidate| candidate.is_file())
        .find(|candidate| major_of(candidate) == Some(major))
        .ok_or(format!(
            "Java {major} not found: install it, or pass --java PATH or set JAVA_HOME"
        ))
}
