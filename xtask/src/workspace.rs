//! The workspace's own packages and the dependency edges between them.

use std::fs;

use crate::process;

pub struct Package {
    pub name: String,
    /// Directory relative to the repository root, without a trailing slash.
    pub dir: String,
    /// Workspace packages this one links into its library or binaries.
    pub deps: Vec<String>,
    /// Workspace packages only its tests, examples or benches use.
    pub dev_deps: Vec<String>,
}

/// The directories whose subdirectories are workspace members.
const MEMBER_DIRS: &[&str] = &["crates", "parity"];

/// Reads every workspace member manifest.
pub fn packages() -> Result<Vec<Package>, String> {
    let root = process::root();
    let mut dirs = Vec::new();
    for parent in MEMBER_DIRS {
        let entries = fs::read_dir(root.join(parent))
            .map_err(|error| format!("cannot list {parent}/: {error}"))?;
        dirs.extend(
            entries
                .filter_map(Result::ok)
                .filter(|entry| entry.path().join("Cargo.toml").is_file())
                .map(|entry| format!("{parent}/{}", entry.file_name().to_string_lossy())),
        );
    }
    dirs.push("xtask".to_owned());
    dirs.sort();
    let manifests = dirs
        .into_iter()
        .map(|dir| {
            let path = root.join(&dir).join("Cargo.toml");
            let text = fs::read_to_string(&path)
                .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
            let table: toml::Table = text
                .parse()
                .map_err(|error| format!("{}: {error}", path.display()))?;
            Ok((dir, table))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let names: Vec<String> = manifests.iter().map(|(_, table)| name_of(table)).collect();
    Ok(manifests
        .into_iter()
        .map(|(dir, table)| Package {
            name: name_of(&table),
            deps: internal(&table, &["dependencies", "build-dependencies"], &names),
            dev_deps: internal(&table, &["dev-dependencies"], &names),
            dir,
        })
        .collect())
}

/// The package that owns a repository-relative path, if any.
pub fn owner<'a>(packages: &'a [Package], path: &str) -> Option<&'a Package> {
    packages
        .iter()
        .find(|package| path.starts_with(&format!("{}/", package.dir)))
}

fn name_of(table: &toml::Table) -> String {
    table
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn internal(table: &toml::Table, sections: &[&str], names: &[String]) -> Vec<String> {
    let mut deps: Vec<String> = sections
        .iter()
        .filter_map(|section| table.get(*section).and_then(toml::Value::as_table))
        .flat_map(|deps| deps.keys())
        .filter(|key| names.contains(key))
        .cloned()
        .collect();
    deps.sort();
    deps.dedup();
    deps
}
