//! What each older version's blocks declare differently from the newest version's table: the
//! blast resistance, forced solidity and push reaction a block declares, and the view blocking
//! rule of every block the version declares one for. The captures of the block declarations are
//! compared; any other difference is an error, so a new kind of difference is not lost.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde_json::Value;

use super::{Versions, capture, text};

const FAMILY: &str = "block";

/// A block's declarations as a capture lists them.
type Blocks = BTreeMap<String, Value>;

fn blocks(version: &str) -> Result<Blocks, String> {
    let rows = capture::rows(version, FAMILY, "block/declarations")?;
    let list = rows
        .first()
        .and_then(|row| row["blocks"].as_array())
        .ok_or_else(|| format!("{version}: no block declarations"))?;
    list.iter()
        .map(|block| {
            let name = block["name"].as_str().ok_or("a block name")?;
            Ok((name.to_owned(), block.clone()))
        })
        .collect()
}

/// The variant of the push reaction a capture names, in the names of either generation: 26.3
/// renamed all five.
fn reaction(name: &str) -> Result<&'static str, String> {
    Ok(match name {
        "NORMAL" | "PUSH_PULL" => "Normal",
        "DESTROY" | "POPPED" => "Destroy",
        "BLOCK" | "IMMOVEABLE" => "Block",
        "IGNORE" | "IGNORE_ENTITY" => "Ignore",
        "PUSH_ONLY" | "PUSH" => "PushOnly",
        other => return Err(format!("push reaction {other}")),
    })
}

/// The rule that gives the listed value of every state.
fn rule(values: &Value) -> Result<String, String> {
    let values: Vec<bool> = values
        .as_array()
        .ok_or("view blocking values")?
        .iter()
        .map(|value| value.as_bool().ok_or("a view blocking value".to_owned()))
        .collect::<Result<_, _>>()?;
    Ok(if values.iter().all(|value| *value) {
        "Rule::Always".to_owned()
    } else if values.iter().all(|value| !*value) {
        "Rule::Never".to_owned()
    } else {
        let list: Vec<String> = values.iter().map(bool::to_string).collect();
        format!("Rule::States(&[{}])", list.join(", "))
    })
}

/// The declared fields that differ, as the Rust text of each patch field.
fn declared(
    name: &str,
    older: &Value,
    newest: &Value,
) -> Result<Vec<(&'static str, String)>, String> {
    let (older, newest) = (&older["declared"], &newest["declared"]);
    let mut fields = Vec::new();
    for (key, value) in older.as_object().ok_or("declared fields")? {
        let other = &newest[key];
        let same = match key.as_str() {
            "pushReaction" => {
                reaction(value.as_str().unwrap_or(""))? == reaction(other.as_str().unwrap_or(""))?
            }
            _ => value == other,
        };
        if same {
            continue;
        }
        let field = match key.as_str() {
            "explosionResistance" => (
                "explosion_resistance",
                format!(
                    "Some({})",
                    text::f32_from_bits(value.as_str().ok_or("bits")?)?
                ),
            ),
            "forceSolidOn" => (
                "force_solid_on",
                format!("Some({})", value.as_bool().ok_or("bool")?),
            ),
            "pushReaction" => (
                "push_reaction",
                format!(
                    "Some(PushReaction::{})",
                    reaction(value.as_str().ok_or("name")?)?
                ),
            ),
            other => return Err(format!("{name}: {other} differs, and no patch carries it")),
        };
        fields.push(field);
    }
    Ok(fields)
}

/// The view blocking rule an older version lists for the block, where it lists one.
fn view_blocking(name: &str, older: &Value, newest: &Value) -> Result<Option<String>, String> {
    let (older, newest) = (&older["per_state"], &newest["per_state"]);
    for (key, value) in older.as_object().ok_or("per_state")? {
        if key != "isViewBlocking" && newest[key] != *value {
            return Err(format!(
                "{name}: {key} differs per state, and no patch carries it"
            ));
        }
    }
    older.get("isViewBlocking").map(rule).transpose()
}

fn patches(older: &Blocks, newest: &Blocks) -> Result<String, String> {
    let mut out = String::new();
    for (name, block) in older {
        let other = newest
            .get(name)
            .ok_or_else(|| format!("{name} is in an older version and not in the newest"))?;
        for key in ["properties", "default", "item"] {
            if block[key] != other[key] {
                return Err(format!("{name}: {key} differs"));
            }
        }
        let mut fields = declared(name, block, other)?;
        if let Some(rule) = view_blocking(name, block, other)? {
            fields.push(("view_blocking", format!("Some({rule})")));
        }
        if fields.is_empty() {
            continue;
        }
        let _ = write!(out, "    BlockPatch {{ name: {}", text::string(name));
        for key in [
            "explosion_resistance",
            "force_solid_on",
            "push_reaction",
            "view_blocking",
        ] {
            let value = fields
                .iter()
                .find(|(field, _)| *field == key)
                .map_or("None", |(_, v)| v);
            let _ = write!(out, ", {key}: {value}");
        }
        out.push_str(" },\n");
    }
    Ok(out)
}

pub fn render(versions: &Versions) -> Result<String, String> {
    let newest = blocks(&versions.newest)?;
    let mut out = text::header(
        "What each older version's blocks declare differently from the newest version's table.",
        "block_older",
    );
    out.push_str("use photon_core::version::Version;\n\nuse crate::block::definition::{PushReaction, Rule};\nuse crate::versions::block_patches::BlockPatch;\n\n");
    let mut entries = String::new();
    for version in &versions.older {
        let tag = crate::parity::pins::tag(version);
        let body = patches(&blocks(version)?, &newest)?;
        let count = body.lines().count();
        let _ = write!(
            out,
            "#[rustfmt::skip]\nstatic PATCHES_{tag}: [BlockPatch; {count}] = [\n{body}];\n\n"
        );
        let _ = writeln!(entries, "    (Version::V{tag}, &PATCHES_{tag}),");
    }
    out.push_str("/// For each version older than the newest: the blocks it declares differently.\n#[rustfmt::skip]\n");
    let _ = writeln!(
        out,
        "pub static OLDER: &[(Version, &[BlockPatch])] = &[\n{entries}];"
    );
    Ok(out)
}
