//! `cargo xtask progress`: the README's computed progress block.
//!
//! The block between the `progress` markers holds numbers that are fixed for a given commit, so
//! `--check` can compare it with a fresh computation (CI and the pre-push hook do). Numbers only:
//! nothing derived from the reference leaves `.data/` in any other form.

mod classes;
mod local;
mod markers;
mod milestones;
mod reference;

use std::fs;

use crate::process;

const BLOCK: &str = "progress";
const MILESTONES: &str = "milestones";

fn readme() -> std::path::PathBuf {
    process::root().join("README.md")
}

/// The markdown table of the metrics.
fn block() -> Result<String, String> {
    let mut rows = Vec::new();
    rows.extend(reference::rows()?);
    rows.extend(local::rows()?);
    let mut out = String::from("| Measure | Now |\n| --- | --- |\n");
    for (label, value) in rows {
        out.push_str(&format!("| {label} | {value} |\n"));
    }
    Ok(out)
}

/// The staleness check for the pre-push hook: skipped without the decompiled reference, which
/// the numbers need.
pub fn check_if_possible() -> Result<(), String> {
    if !process::root().join(".data/minecraft/source").is_dir() {
        println!("progress: no decompiled reference, skipped");
        return Ok(());
    }
    run(&["--check".to_owned()])
}

/// `cargo xtask progress [--check]`
pub fn run(args: &[String]) -> Result<(), String> {
    let check = match args {
        [] => false,
        [flag] if flag == "--check" => true,
        _ => return Err("usage: cargo xtask progress [--check]".to_owned()),
    };
    let path = readme();
    let text = fs::read_to_string(&path).map_err(|error| format!("README.md: {error}"))?;
    let fresh = block()?;
    let mut updated = markers::replace(&text, BLOCK, &fresh)?;
    if !check {
        updated = markers::replace(&updated, MILESTONES, &milestones::table()?)?;
    }
    if updated == text {
        println!("progress: README block is current");
        return Ok(());
    }
    if check {
        for line in fresh.lines().filter(|line| !text.contains(line)) {
            eprintln!("  computed, not in the README: {line}");
        }
        return Err("progress: the README block is stale; run `cargo xtask progress`".to_owned());
    }
    fs::write(&path, updated).map_err(|error| format!("README.md: {error}"))?;
    println!("progress: README block rewritten");
    Ok(())
}
