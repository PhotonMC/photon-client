//! Which crates a set of changed files can affect.

use std::collections::BTreeSet;

use crate::git;
use crate::workspace::{self, Package};

/// Files outside any crate that change how every crate builds or behaves.
const GLOBAL_FILES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "clippy.toml",
    "rust-toolchain.toml",
    ".cargo/config.toml",
];

/// The packages owning `files`, plus every package that depends on them
/// (including through dev-dependencies, since their tests can break).
pub fn crates_for(packages: &[Package], files: &[String]) -> BTreeSet<String> {
    if files
        .iter()
        .any(|file| GLOBAL_FILES.contains(&file.as_str()))
    {
        return packages
            .iter()
            .map(|package| package.name.clone())
            .collect();
    }
    let mut affected: BTreeSet<String> = files
        .iter()
        .filter_map(|file| workspace::owner(packages, file))
        .map(|package| package.name.clone())
        .collect();
    loop {
        let before = affected.len();
        for package in packages {
            let uses_affected = package
                .deps
                .iter()
                .chain(&package.dev_deps)
                .any(|dep| affected.contains(dep));
            if uses_affected {
                affected.insert(package.name.clone());
            }
        }
        if affected.len() == before {
            return affected;
        }
    }
}

/// `cargo xtask affected [--base <ref>]`
pub fn print(args: &[String]) -> Result<(), String> {
    let base = match args {
        [] => git::push_base()?,
        [flag, base] if flag == "--base" => base.clone(),
        _ => return Err("usage: cargo xtask affected [--base <ref>]".to_owned()),
    };
    let packages = workspace::packages()?;
    for name in crates_for(&packages, &git::changed_since(&base)?) {
        println!("{name}");
    }
    Ok(())
}
