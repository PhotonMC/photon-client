//! `cargo xtask package`: the release archive of this platform. It holds the binary and the
//! licence and readme files, and nothing else; the archive is listed afterwards and the command
//! fails if the list holds anything outside that set or anything from `.data/`, where the
//! original game content lives (ADR 0007).

use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::process;

/// The files an archive may hold besides the binary.
const EXTRAS: &[&str] = &["LICENSE-APACHE", "LICENSE-MIT", "README.md"];

fn binary_name() -> &'static str {
    if cfg!(windows) {
        "photon.exe"
    } else {
        "photon"
    }
}

/// Whether an archive entry (`<folder>/<file>`) is one the release may hold.
fn entry_allowed(entry: &str) -> bool {
    let entry = entry.trim_end_matches('/');
    let parts: Vec<&str> = entry.split(['/', '\\']).collect();
    if entry.starts_with('/') || parts.iter().any(|part| matches!(*part, ".." | ".data")) {
        return false;
    }
    match parts.as_slice() {
        [_folder] => true,
        [_folder, file] => *file == binary_name() || EXTRAS.contains(file),
        _ => false,
    }
}

/// The entries that must not be in the archive; empty when it is clean.
fn forbidden(listing: &[String]) -> Vec<&String> {
    listing
        .iter()
        .filter(|entry| !entry_allowed(entry))
        .collect()
}

/// The host triple, as `rustc -vV` names it.
fn host_triple() -> Result<String, String> {
    let output = process::output("rustc", &["-vV"])?;
    output
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::to_owned)
        .ok_or_else(|| "rustc -vV names no host".to_owned())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn run(args: &[String]) -> Result<(), String> {
    if !args.iter().all(|arg| arg == "--no-build") {
        return Err("usage: cargo xtask package [--no-build]".to_owned());
    }
    if args.is_empty() {
        process::run("cargo", &["build", "--release", "-p", "photon"])?;
    }
    let root = process::root();
    let name = format!("photon-{}-{}", env!("CARGO_PKG_VERSION"), host_triple()?);
    let out = root.join("target").join("package");
    let folder = out.join(&name);
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).map_err(|error| error.to_string())?;
    let binary = root.join("target").join("release").join(binary_name());
    copy(&binary, &folder.join(binary_name()))?;
    for extra in EXTRAS {
        copy(&root.join(extra), &folder.join(extra))?;
    }
    let archive = out.join(format!("{name}.tar.gz"));
    let (archive_text, out_text) = (path_text(&archive)?, path_text(&out)?);
    process::run("tar", &["-czf", &archive_text, "-C", &out_text, &name])?;
    let listing: Vec<String> = process::output("tar", &["-tzf", &archive_text])?
        .lines()
        .map(str::to_owned)
        .collect();
    let bad = forbidden(&listing);
    if !bad.is_empty() {
        let _ = fs::remove_file(&archive);
        return Err(format!(
            "the archive holds files outside the release set: {}",
            bad.iter()
                .map(|entry| entry.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let bytes = fs::read(&archive).map_err(|error| error.to_string())?;
    let sum = hex(&Sha256::digest(&bytes));
    let sums = out.join(format!("{name}.tar.gz.sha256"));
    fs::write(&sums, format!("{sum}  {name}.tar.gz\n")).map_err(|error| error.to_string())?;
    println!(
        "{} ({} files, sha256 {sum})",
        archive.display(),
        listing.len()
    );
    Ok(())
}

fn path_text(path: &Path) -> Result<String, String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("{} is not text", path.display()))
}

fn copy(from: &Path, to: &Path) -> Result<(), String> {
    fs::copy(from, to)
        .map(drop)
        .map_err(|error| format!("{}: {error}", from.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listing(entries: &[&str]) -> Vec<String> {
        entries.iter().map(|entry| (*entry).to_owned()).collect()
    }

    #[test]
    fn the_release_set_passes() {
        let binary = format!("p-1/{}", binary_name());
        let entries = listing(&["p-1/", &binary, "p-1/LICENSE-MIT", "p-1/README.md"]);
        assert!(forbidden(&entries).is_empty());
    }

    #[test]
    fn anything_else_is_named() {
        let binary = format!("p-1/{}", binary_name());
        let entries = listing(&[
            &binary,
            "p-1/.data/minecraft/client.jar",
            "p-1/assets/sound.ogg",
            "p-1/../escape",
            "/etc/passwd",
            "p-1/notes.txt",
        ]);
        assert_eq!(forbidden(&entries).len(), 5);
    }
}
