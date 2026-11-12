//! `cargo xtask gen [--check]`: the reference tables of `photon-data/src/tables/`, written from
//! the local reference, and the WIT of the mod API of `photon-api`.
//!
//! A generator reads cases of the parity captures (the Java answer, in
//! `.data/parity/<version>/<family>.jsonl.gz`): the newest version's for the main table, and
//! the older versions' where a table keeps what differs from the newest. It returns the text of
//! one table. The files carry
//! an `@generated` header and are never edited by hand. `--check` regenerates every table and
//! fails on a difference, so a table that drifted from its generator is caught; the parity
//! families keep comparing the tables with the reference.

mod block_older;
mod blocks;
pub(crate) mod capture;
mod component_sets;
mod compost;
mod entity_models;
mod mod_wit;
mod text;

use std::fs;

use crate::process;

/// The versions a generator reads captures of.
pub struct Versions {
    /// The newest version, whose capture the main tables are written from.
    pub newest: String,
    /// The other supported versions that have a reference, oldest first.
    pub older: Vec<String>,
}

/// One table: where it is written and how its text is made.
struct Table {
    /// Path under the repository root.
    path: &'static str,
    /// The text of the file, from the captures of `versions`.
    render: fn(&Versions) -> Result<String, String>,
}

const TABLES: &[Table] = &[
    Table {
        path: "crates/photon-data/src/tables/block/blocks.rs",
        render: blocks::render,
    },
    Table {
        path: "crates/photon-data/src/tables/block_older.rs",
        render: block_older::render,
    },
    Table {
        path: "crates/photon-data/src/tables/compost.rs",
        render: compost::render,
    },
    Table {
        path: "crates/photon-data/src/tables/item/component_sets_older.rs",
        render: component_sets::render,
    },
    Table {
        path: "crates/photon-api/wit/photon-mod.wit",
        render: mod_wit::render,
    },
    Table {
        path: "crates/photon-data/src/tables/entity_models.rs",
        render: entity_models::render,
    },
    Table {
        path: "crates/photon-data/src/tables/entity_models_older.rs",
        render: entity_models::render_older,
    },
];

pub fn run(args: &[String]) -> Result<(), String> {
    let mut check = false;
    for arg in args {
        match arg.as_str() {
            "--check" => check = true,
            other => return Err(format!("unknown gen option `{other}`")),
        }
    }
    let newest = crate::parity::pins::newest_version()?;
    let older = crate::parity::pins::versions()
        .into_iter()
        .filter(|version| *version != newest)
        .collect();
    let versions = Versions { newest, older };
    let mut stale = Vec::new();
    for table in TABLES {
        let text = (table.render)(&versions)?;
        let path = process::root().join(table.path);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
        }
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
