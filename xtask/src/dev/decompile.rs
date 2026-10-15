//! `setup --decompile`: readable sources of the client with Vineflower.

use std::fs::{self, File};
use std::process::{Command, Stdio};

use super::download::fetch;
use super::java;
use super::layout::{minecraft, runtime};
use crate::parity::pins::Pins;

const VINEFLOWER_VERSION: &str = "1.12.0";

fn vineflower_url() -> String {
    format!(
        "https://repo1.maven.org/maven2/org/vineflower/vineflower/{VINEFLOWER_VERSION}/vineflower-{VINEFLOWER_VERSION}.jar"
    )
}

/// The client jar to `.data/minecraft/source`, libraries resolved, as the
/// reference sources were made; JOML gets its own tree.
pub fn run(pins: &Pins, java_path: Option<&str>) -> Result<(), String> {
    let java = java::find(java_path, pins.java_major)?;
    let tool = runtime()
        .join("tools")
        .join(format!("vineflower-{VINEFLOWER_VERSION}.jar"));
    fetch(&vineflower_url(), &tool, None)?;
    let source = minecraft().join("source");
    if source.exists() {
        fs::remove_dir_all(&source).map_err(|error| format!("{}: {error}", source.display()))?;
    }
    fs::create_dir_all(&source).map_err(|error| error.to_string())?;
    let mut libraries: Vec<_> = fs::read_dir(runtime().join("libs"))
        .map_err(|error| error.to_string())?
        .filter_map(|entry| Some(entry.ok()?.path()))
        .filter(|path| path.extension().is_some_and(|extension| extension == "jar"))
        .collect();
    libraries.sort();
    println!("decompiling with Vineflower {VINEFLOWER_VERSION} (this takes minutes)");
    let threads = std::thread::available_parallelism().map_or(4, usize::from);
    let log = File::create(minecraft().join("decompile.log")).map_err(|error| error.to_string())?;
    let status = Command::new(&java)
        .args(["-Xmx10G", "-jar"])
        .arg(&tool)
        .arg(format!("--thread-count={threads}"))
        .args([
            "--indent-string=    ",
            "--include-runtime=current",
            "--decompile-generics=true",
        ])
        .args(["--skip-extra-files=true", "--dump-bytecode-on-error=true"])
        .args(
            libraries
                .iter()
                .map(|library| format!("-e={}", library.display())),
        )
        .arg(runtime().join(format!("client-{}.jar", pins.version)))
        .arg(&source)
        .stdout(log.try_clone().map_err(|error| error.to_string())?)
        .stderr(log)
        .status()
        .map_err(|error| format!("cannot start {}: {error}", java.display()))?;
    if !status.success() {
        return Err(format!(
            "Vineflower failed ({status}); see .data/minecraft/decompile.log"
        ));
    }
    decompile_joml(&java, &tool, &libraries)
}

fn decompile_joml(
    java: &std::path::Path,
    tool: &std::path::Path,
    libraries: &[std::path::PathBuf],
) -> Result<(), String> {
    let joml = libraries.iter().find(|library| {
        library
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with("joml-"))
    });
    let Some(joml) = joml else { return Ok(()) };
    let target = minecraft().join("joml-source");
    let _ = fs::remove_dir_all(&target);
    fs::create_dir_all(&target).map_err(|error| error.to_string())?;
    let status = Command::new(java)
        .arg("-jar")
        .arg(tool)
        .args(["--indent-string=    ", "--skip-extra-files=true"])
        .arg(joml)
        .arg(&target)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Vineflower failed on JOML ({status})"))
    }
}
