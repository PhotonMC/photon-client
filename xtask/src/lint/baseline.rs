//! `xtask/lint-baseline.toml`: violations that predate a rule.
//!
//! Each entry caps a known violation at its recorded value. The cap only
//! ever moves down: the lint lowers it when the code improves and drops it
//! when the violation is gone, so the codebase ratchets toward the limits.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use crate::process;

/// Rule name to (violation key to the highest accepted value).
pub type Baseline = BTreeMap<String, BTreeMap<String, u64>>;

const HEADER: &str = "\
# Violations that predate the lint rules (see `cargo xtask lint`).
# Entries only shrink: the lint lowers or removes them as code improves.
# Never raise a value or add an entry by hand; fix the code instead.

";

fn path() -> PathBuf {
    process::root().join("xtask/lint-baseline.toml")
}

pub fn exists() -> bool {
    path().is_file()
}

pub fn load() -> Result<Baseline, String> {
    if !exists() {
        return Ok(Baseline::new());
    }
    let text = fs::read_to_string(path()).map_err(|error| format!("lint baseline: {error}"))?;
    toml::from_str(&text).map_err(|error| format!("lint baseline: {error}"))
}

pub fn save(baseline: &Baseline) -> Result<(), String> {
    let body = toml::to_string(baseline).map_err(|error| format!("lint baseline: {error}"))?;
    fs::write(path(), format!("{HEADER}{body}")).map_err(|error| format!("lint baseline: {error}"))
}
