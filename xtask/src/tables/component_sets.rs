//! What each older version's items differ in from the newest version's table of default component
//! types: components an older version's items have that the newest table lacks, and components
//! of types the version registers that its items lack.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde_json::Value;

use super::{Versions, capture, text};

const FAMILY: &str = "block";

/// Each item's component type names, in the order of the version's items.
type Sets = Vec<(String, BTreeSet<String>)>;

/// The component type names of the version's registry, by wire id.
fn type_names(version: &str) -> Result<Vec<String>, String> {
    let rows = capture::rows(version, FAMILY, "block/item-declarations")?;
    rows.first()
        .and_then(|row| row["component_types"].as_array())
        .ok_or_else(|| format!("{version}: item declarations list no component types"))?
        .iter()
        .map(|name| {
            name.as_str()
                .map(str::to_owned)
                .ok_or("a component type name".to_owned())
        })
        .collect()
}

fn sets(version: &str) -> Result<Sets, String> {
    let names = type_names(version)?;
    let rows = capture::rows(version, FAMILY, "block/item-components")?;
    let items = rows
        .first()
        .and_then(|row| row["items"].as_array())
        .ok_or_else(|| format!("{version}: no item components"))?;
    items.iter().map(|item| one(item, &names)).collect()
}

fn one(item: &Value, names: &[String]) -> Result<(String, BTreeSet<String>), String> {
    let name = item["name"].as_str().ok_or("an item name")?.to_owned();
    let types = item["types"].as_array().ok_or("an item's types")?;
    let mut set = BTreeSet::new();
    for id in types {
        let id = usize::try_from(id.as_u64().ok_or("a type id")?).map_err(|e| e.to_string())?;
        set.insert(
            names
                .get(id)
                .ok_or("a type id outside the registry")?
                .clone(),
        );
    }
    Ok((name, set))
}

/// The items that have each component in `want` and not in `other` (by item name).
fn differing(
    want: &BTreeMap<&str, &BTreeSet<String>>,
    other: &BTreeMap<&str, &BTreeSet<String>>,
) -> BTreeMap<String, Vec<String>> {
    let mut found: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (item, set) in want {
        let Some(their) = other.get(item) else {
            continue;
        };
        for component in set.iter().filter(|component| !their.contains(*component)) {
            found
                .entry(component.clone())
                .or_default()
                .push((*item).to_owned());
        }
    }
    found
}

pub fn render(versions: &Versions) -> Result<String, String> {
    let newest = sets(&versions.newest)?;
    let newest_by: BTreeMap<&str, &BTreeSet<String>> = newest
        .iter()
        .map(|(name, set)| (name.as_str(), set))
        .collect();
    let mut out = text::header(
        "What each older version's items differ in from the newest version's default component types.",
        "component_sets",
    );
    out.push_str("use photon_core::version::Version;\n\nuse crate::versions::component_sets::{Items, SetChange};\n\n");
    let mut entries = String::new();
    for version in &versions.older {
        let tag = crate::parity::pins::tag(version);
        let older = sets(version)?;
        let registered: BTreeSet<String> = type_names(version)?.into_iter().collect();
        let older_by: BTreeMap<&str, &BTreeSet<String>> = older
            .iter()
            .map(|(name, set)| (name.as_str(), set))
            .collect();
        if let Some((item, _)) = older
            .iter()
            .find(|(name, _)| !newest_by.contains_key(name.as_str()))
        {
            return Err(format!(
                "{item} is in {version} and not in the newest version"
            ));
        }
        let added = differing(&older_by, &newest_by);
        let mut removed = differing(&newest_by, &older_by);
        removed.retain(|component, _| registered.contains(component));
        write_changes(&mut out, &format!("ADDED_{tag}"), &added, older.len())?;
        write_changes(&mut out, &format!("REMOVED_{tag}"), &removed, older.len())?;
        let _ = writeln!(
            entries,
            "    (Version::V{tag}, &ADDED_{tag}, &REMOVED_{tag}),"
        );
    }
    out.push_str("/// For each version older than the newest: the components its items have that the newest table's\n/// sets lack, and the components of types it registers that its items lack.\n#[rustfmt::skip]\n");
    let _ = writeln!(
        out,
        "pub static OLDER: &[(Version, &[SetChange], &[SetChange])] = &[\n{entries}];"
    );
    Ok(out)
}

fn write_changes(
    out: &mut String,
    name: &str,
    changes: &BTreeMap<String, Vec<String>>,
    all: usize,
) -> Result<(), String> {
    let _ = writeln!(
        out,
        "#[rustfmt::skip]\nstatic {name}: [SetChange; {}] = [",
        changes.len()
    );
    for (component, items) in changes {
        let which = if items.len() == all {
            "Items::All".to_owned()
        } else {
            let list: Vec<String> = items.iter().map(|item| text::string(item)).collect();
            format!("Items::Only(&[{}])", list.join(", "))
        };
        let _ = writeln!(
            out,
            "    SetChange {{ component: {}, items: {which} }},",
            text::string(component)
        );
    }
    out.push_str("];\n\n");
    Ok(())
}
