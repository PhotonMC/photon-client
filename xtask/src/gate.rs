//! The commit gates: the full `check` and the two git-hook subsets.

use std::collections::BTreeSet;

use crate::{affected, git, lint, parity, process, workspace};

/// Sources whose change invalidates the saved Java reference outputs.
const REFERENCE_SOURCES: &[&str] = &["tests/behavior/", "tools/behavior/"];

/// `cargo xtask check [--no-parity]`: everything CI runs, plus parity.
pub fn check(args: &[String]) -> Result<(), String> {
    let with_parity = !args.iter().any(|arg| arg == "--no-parity");
    let mut failed = Vec::new();
    let mut step = |label: &str, result: Result<(), String>| {
        if let Err(message) = result {
            eprintln!("{label}: {message}");
            failed.push(label.to_owned());
        }
    };
    step("fmt", process::run("cargo", &["fmt", "--all", "--check"]));
    step("lint", lint::run(&[]));
    step("clippy", clippy(&[]));
    step(
        "test",
        process::run(
            "cargo",
            &["test", "--release", "--workspace", "--all-targets"],
        ),
    );
    for (tool, args) in [
        ("cargo-deny", &["deny", "check"][..]),
        ("cargo-machete", &["machete"][..]),
    ] {
        if process::available(tool) {
            step(tool, process::run("cargo", args));
        } else {
            println!("{tool}: not installed, skipped");
        }
    }
    if with_parity && parity::reference_present() {
        let families: Vec<String> = parity::FAMILIES.iter().map(|f| (*f).to_owned()).collect();
        step("parity", parity::compare(false, &families));
    } else if with_parity {
        println!("parity: no local Java reference (.data/minecraft/runtime), skipped");
    }
    if failed.is_empty() {
        println!("CHECK OK");
        Ok(())
    } else {
        Err(format!("CHECK FAILED: {}", failed.join(", ")))
    }
}

/// Fast checks on what is about to be committed.
pub fn pre_commit() -> Result<(), String> {
    let staged = git::staged()?;
    if staged.is_empty() {
        return Ok(());
    }
    let rust: Vec<&str> = staged
        .iter()
        .filter(|path| path.ends_with(".rs") && process::root().join(path).is_file())
        .map(String::as_str)
        .collect();
    if !rust.is_empty() {
        let mut args = vec!["--check", "--edition", "2024"];
        args.extend(&rust);
        process::run("rustfmt", &args).map_err(|_| "rustfmt: run `cargo fmt --all`".to_owned())?;
    }
    lint::run(&["--quiet".to_owned()])?;
    let crates = affected::crates_for(&workspace::packages()?, &staged);
    if crates.is_empty() {
        return Ok(());
    }
    clippy(&package_args(&crates))
}

/// Tests and parity for the commits about to be pushed.
pub fn pre_push() -> Result<(), String> {
    let changed = git::changed_for_push()?;
    let crates = affected::crates_for(&workspace::packages()?, &changed);
    if !crates.is_empty() {
        let mut args = vec![
            "test".to_owned(),
            "--release".to_owned(),
            "--all-targets".to_owned(),
        ];
        args.extend(package_args(&crates));
        process::run(
            "cargo",
            &args.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
    }
    let reference_changed = changed.iter().any(|path| {
        REFERENCE_SOURCES
            .iter()
            .any(|prefix| path.starts_with(prefix))
    });
    let engine_changed = crates.iter().any(|name| name != "xtask");
    if !(reference_changed || engine_changed) {
        return Ok(());
    }
    if !parity::reference_present() {
        println!("parity: no local Java reference, skipped");
        return Ok(());
    }
    let families: Vec<String> = parity::FAMILIES.iter().map(|f| (*f).to_owned()).collect();
    parity::compare(reference_changed, &families)
}

fn clippy(packages: &[String]) -> Result<(), String> {
    let mut args = vec!["clippy"];
    if packages.is_empty() {
        args.push("--workspace");
    }
    args.extend(packages.iter().map(String::as_str));
    args.extend(["--all-targets", "--", "-D", "warnings"]);
    process::run("cargo", &args)
}

fn package_args(crates: &BTreeSet<String>) -> Vec<String> {
    crates
        .iter()
        .flat_map(|name| ["--package".to_owned(), name.clone()])
        .collect()
}
