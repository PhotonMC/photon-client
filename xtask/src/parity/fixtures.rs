//! Files the Rust adapter reads that a version's captures produce: the reference's whole default
//! component patch of each item, which the `item_components` case reads with this port and writes
//! back. They stay under `.data/` with the captures they come from. The newest version's is the
//! tracked `tests/behavior/fixtures/item-default-patches.txt`.

use std::fmt::Write as _;
use std::path::PathBuf;

use crate::process;
use crate::tables::capture;

/// Where the fixtures of every version are written.
pub(super) fn dir() -> PathBuf {
    process::root().join(".data/parity/fixtures")
}

/// Writes `version`'s fixtures from its capture, when it has one.
pub(super) fn prepare(version: &str) -> Result<(), String> {
    let Ok(rows) = capture::rows(version, "block", "block/item-components") else {
        return Ok(());
    };
    let items = rows
        .first()
        .and_then(|row| row["items"].as_array())
        .ok_or("item_components: no items")?;
    let mut text = String::new();
    for item in items {
        let name = item["name"].as_str().ok_or("item_components: no name")?;
        let wire = item["wire"].as_str().ok_or("item_components: no wire")?;
        let _ = writeln!(text, "{name} {wire}");
    }
    let folder = dir().join(version);
    std::fs::create_dir_all(&folder).map_err(|error| format!("{}: {error}", folder.display()))?;
    let path = folder.join("item-default-patches.txt");
    if std::fs::read_to_string(&path).is_ok_and(|have| have == text) {
        return Ok(());
    }
    std::fs::write(&path, text).map_err(|error| format!("{}: {error}", path.display()))
}
