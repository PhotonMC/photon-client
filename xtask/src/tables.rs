//! `cargo xtask gen [--check]`: the reference tables of `photon-data/src/tables/`, written from
//! the local reference.
//!
//! A generator reads one case of the newest version's parity capture (the Java answer, in
//! `.data/parity/<version>/<family>.jsonl.gz`) and returns the text of one table. The files carry
//! an `@generated` header and are never edited by hand. `--check` regenerates every table and
//! fails on a difference, so a table that drifted from its generator is caught; the parity
//! families keep comparing the tables with the reference.

mod capture;
mod compost;
mod text;

use std::fs;

use crate::process;

/// One table: where it is written and how its text is made.
struct Table {
    /// Path under the repository root.
    path: &'static str,
    /// The text of the file, from the capture of `version`.
    render: fn(&str) -> Result<String, String>,
}

const TABLES: &[Table] = &[Table {
    path: "crates/photon-data/src/tables/compost.rs",
    render: compost::render,
}];

pub fn run(args: &[String]) -> Result<(), String> {
    let mut check = false;
    for arg in args {
        match arg.as_str() {
            "--check" => check = true,
            other => return Err(format!("unknown gen option `{other}`")),
        }
    }
    let version = crate::parity::pins::newest_version()?;
    let mut stale = Vec::new();
    for table in TABLES {
        let text = (table.render)(&version)?;
        let path = process::root().join(table.path);
        let current = fs::read_to_string(&path).unwrap_or_default();
        if current == text {
            continue;
        }
        if check {
            stale.push(table.path);
        } else {
            fs::write(&path, text).map_err(|error| format!("{}: {error}", path.display()))?;
            println!("gen: wrote {}", table.path);
        }
    }
    if stale.is_empty() {
        println!("gen: {} tables, all current", TABLES.len());
        return Ok(());
    }
    Err(format!(
        "gen: these tables differ from their generator (run `cargo xtask gen`):\n  {}",
        stale.join("\n  ")
    ))
}
