//! `xtask/lint.toml`: the architecture the lint enforces.

use std::collections::BTreeMap;
use std::fs;

use serde::Deserialize;

use crate::process;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Package name to the workspace packages it may depend on.
    pub layers: BTreeMap<String, Vec<String>>,
    /// Packages that are programs, where printing to the terminal is output.
    pub binary_crates: Vec<String>,
    /// Path prefixes where `unsafe` code is expected.
    pub unsafe_allowed: Vec<String>,
    /// Path prefixes that may hold non-ASCII text (translations, test data).
    pub non_ascii_allowed: Vec<String>,
    /// Path prefixes and fragments where `Version` values may be compared (ADR 0013).
    pub version_allowed: Vec<String>,
    /// Path prefixes and fragments that may name `Version::NEWEST`: where a version is selected,
    /// and tests.
    pub newest_allowed: Vec<String>,
}

pub fn load() -> Result<Config, String> {
    let path = process::root().join("xtask/lint.toml");
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    toml::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
}

pub fn allowed(prefixes: &[String], path: &str) -> bool {
    prefixes.iter().any(|prefix| path.starts_with(prefix))
}
