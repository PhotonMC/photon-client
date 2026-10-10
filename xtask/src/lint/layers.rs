//! Crate dependency direction.

use super::config::Config;
use super::{Finding, Rule};
use crate::workspace::Package;

/// A finding for every package missing from the layer table and every
/// workspace dependency the table does not allow.
pub fn check(packages: &[Package], config: &Config) -> Vec<Finding> {
    let mut findings = Vec::new();
    for package in packages {
        let manifest = format!("{}/Cargo.toml", package.dir);
        let Some(allowed) = config.layers.get(&package.name) else {
            findings.push(Finding::new(
                Rule::Layer,
                format!("{} (not in the layer table)", package.name),
                manifest,
                1,
            ));
            continue;
        };
        for dep in package.deps.iter().filter(|dep| !allowed.contains(dep)) {
            findings.push(Finding::new(
                Rule::Layer,
                format!("{} -> {dep}", package.name),
                manifest.clone(),
                1,
            ));
        }
    }
    findings
}
