//! The main block table (`tables/block/blocks.rs`): every block of the newest version's registry,
//! in its order, written from the capture of the block declarations.
//!
//! Each field has one source. The class, the state properties and their defaults, the declared
//! floats and flags, the push reaction, the instrument, the offset type, the item and the fluid
//! are the capture's; the light and the state rules are its per-state values, and the oracle
//! tells the view blocking rule apart by the predicate the block names (the near plane test, or
//! the one every block starts with). The span of versions a block exists in comes from diffing
//! the captures; a block older than every capture takes its first version from
//! `block_spans_uncaptured.toml`, which is interim until a capture of that older version exists.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::Deserialize;
use serde_json::Value;

use super::block_older::{blocks, reaction, rule};
use super::{Versions, text};

/// The blocks whose first version no capture reaches back to.
const UNCAPTURED: &str = include_str!("block_spans_uncaptured.toml");

#[derive(Deserialize)]
struct Uncaptured {
    first_version: BTreeMap<String, String>,
}

/// A captured version's blocks, by name.
type Captured = BTreeMap<String, Value>;

/// The `declared` value `key` of a block, as the text of its float bits.
fn bits<'a>(block: &'a Value, key: &str) -> Result<&'a str, String> {
    block["declared"][key]
        .as_str()
        .ok_or_else(|| format!("{}: no {key}", block["name"]))
}

fn flag(block: &Value, key: &str) -> Result<bool, String> {
    block["declared"][key]
        .as_bool()
        .ok_or_else(|| format!("{}: no {key}", block["name"]))
}

/// A float of the block as a literal, or `None` where it is `default` (given as its bits).
fn float(block: &Value, key: &str, default: &str) -> Result<Option<String>, String> {
    let value = bits(block, key)?;
    if value == default {
        return Ok(None);
    }
    text::f32_from_bits(value).map(Some)
}

/// The builder calls for the floats a block declares.
fn float_calls(block: &Value, calls: &mut Vec<String>) -> Result<(), String> {
    let destroy = float(block, "destroyTime", "00000000")?;
    let resistance = float(block, "explosionResistance", "00000000")?;
    if destroy.is_some() || resistance.is_some() {
        calls.push(format!(
            "strength({}, {})",
            text::f32_from_bits(bits(block, "destroyTime")?)?,
            text::f32_from_bits(bits(block, "explosionResistance")?)?
        ));
    }
    for (key, call, default) in [
        ("friction", "friction", "3f19999a"),
        ("speedFactor", "speed_factor", "3f800000"),
        ("jumpFactor", "jump_factor", "3f800000"),
        ("bounceRestitution", "bounce_restitution", "00000000"),
        (
            "fallDistanceReduction",
            "fall_distance_reduction",
            "00000000",
        ),
    ] {
        if block["declared"].get(key).is_none() {
            continue;
        }
        if let Some(value) = float(block, key, default)? {
            calls.push(format!("{call}({value})"));
        }
    }
    Ok(())
}

/// The builder calls for the flags a block declares. `noCollision` also clears occlusion, so a
/// block that collides not and occludes is not written by the builders.
fn flag_calls(block: &Value, calls: &mut Vec<String>) -> Result<(), String> {
    let collides = flag(block, "hasCollision")?;
    let occludes = flag(block, "canOcclude")?;
    match (collides, occludes) {
        (true, true) => {}
        (true, false) => calls.push("no_occlusion()".to_owned()),
        (false, false) => calls.push("no_collision()".to_owned()),
        (false, true) => return Err(format!("{}: no collision and occludes", block["name"])),
    }
    for (key, call, on) in [
        (
            "requiresCorrectToolForDrops",
            "requires_correct_tool_for_drops",
            true,
        ),
        ("isRandomlyTicking", "random_ticks", true),
        ("isAir", "air", true),
        ("ignitedByLava", "ignited_by_lava", true),
        ("liquid", "liquid", true),
        ("forceSolidOff", "force_solid_off", true),
        ("forceSolidOn", "force_solid_on", true),
        ("spawnTerrainParticles", "no_terrain_particles", false),
        ("replaceable", "replaceable", true),
        ("dynamicShape", "dynamic_shape", true),
    ] {
        if flag(block, key)? == on {
            calls.push(format!("{call}()"));
        }
    }
    Ok(())
}

/// The builder calls for the push reaction, instrument and offset a block declares.
fn kind_calls(block: &Value, calls: &mut Vec<String>) -> Result<(), String> {
    let reaction = reaction(block["declared"]["pushReaction"].as_str().unwrap_or(""))?;
    if reaction != "Normal" {
        calls.push(format!("push_reaction(PushReaction::{reaction})"));
    }
    let instrument = block["declared"]["instrument"]
        .as_str()
        .ok_or("instrument")?;
    if instrument != "harp" {
        calls.push(format!("instrument({})", text::string(instrument)));
    }
    match block["declared"]["offsetType"].as_str().unwrap_or("") {
        "NONE" => {}
        "XZ" => calls.push("offset_type(OffsetType::Xz)".to_owned()),
        "XYZ" => calls.push("offset_type(OffsetType::Xyz)".to_owned()),
        other => return Err(format!("{}: offset type {other}", block["name"])),
    }
    Ok(())
}

/// `Properties.lightLevel`: one level for every state, or one per state.
fn light(levels: &Value) -> Result<Option<String>, String> {
    let levels: Vec<u64> = levels
        .as_array()
        .ok_or("light levels")?
        .iter()
        .map(|level| level.as_u64().ok_or("a light level"))
        .collect::<Result<_, _>>()?;
    let first = *levels.first().ok_or("no states")?;
    Ok(if levels.iter().all(|level| *level == first) {
        (first != 0).then(|| format!("Emission::Constant({first})"))
    } else {
        let list: Vec<String> = levels.iter().map(u64::to_string).collect();
        Some(format!("Emission::States(&[{}])", list.join(", ")))
    })
}

/// The builder calls for the light and the state rules a block lists.
fn rule_calls(block: &Value, calls: &mut Vec<String>) -> Result<(), String> {
    let per_state = &block["per_state"];
    if let Some(levels) = per_state.get("lightEmission")
        && let Some(emission) = light(levels)?
    {
        calls.push(format!("light_level({emission})"));
    }
    for (key, call) in [
        ("emissiveRendering", "emissive_rendering"),
        ("isSuffocating", "is_suffocating"),
        ("isRedstoneConductor", "is_redstone_conductor"),
    ] {
        if let Some(values) = per_state.get(key) {
            calls.push(format!("{call}({})", rule(values)?));
        }
    }
    let declared = &block["declared"];
    if declared["viewBlockingNearPlane"] == true {
        calls.push("is_view_blocking(Rule::NearPlane)".to_owned());
    } else if declared["viewBlockingDefault"] != true {
        let values = per_state.get("isViewBlocking").ok_or("view blocking")?;
        calls.push(format!("is_view_blocking({})", rule(values)?));
    }
    Ok(())
}

/// The variant of `Fluid` a fluid name stands for.
fn fluid(name: &str) -> Result<&'static str, String> {
    Ok(match name {
        "minecraft:empty" => "Empty",
        "minecraft:water" => "Water",
        "minecraft:flowing_water" => "FlowingWater",
        "minecraft:lava" => "Lava",
        "minecraft:flowing_lava" => "FlowingLava",
        other => return Err(format!("fluid {other}")),
    })
}

/// The builder calls for the state properties, the item and the fluid.
fn identity_calls(block: &Value, calls: &mut Vec<String>) -> Result<(), String> {
    let name = block["name"].as_str().ok_or("a block name")?;
    let fields = block["properties"].as_array().ok_or("properties")?;
    if !fields.is_empty() {
        let statics: Vec<String> = fields
            .iter()
            .map(|field| match field.as_str() {
                Some(field) if !field.contains('.') => Ok(format!("&{field}")),
                other => Err(format!("{name}: property field {other:?} has no static")),
            })
            .collect::<Result<_, _>>()?;
        let defaults: Vec<String> = block["default"]
            .as_array()
            .ok_or("defaults")?
            .iter()
            .map(|value| value.as_str().map(text::string).ok_or("a default"))
            .collect::<Result<_, _>>()?;
        calls.push(format!(
            "properties(&[{}], &[{}])",
            statics.join(", "),
            defaults.join(", ")
        ));
    }
    if let Some(name) = block.get("fluid").and_then(Value::as_str) {
        calls.push(format!("fluid(Fluid::{})", fluid(name)?));
    }
    let item = block["item"].as_str().ok_or("item")?;
    if item != name {
        calls.push(format!("item({})", text::string(item)));
    }
    Ok(())
}

/// The Rust text of one block, with the span of versions it exists in when that is not all.
fn line(block: &Value, since: Option<&str>) -> Result<String, String> {
    let name = block["name"].as_str().ok_or("a block name")?;
    let class = block["class"].as_str().ok_or("a block class")?;
    let mut calls = Vec::new();
    identity_calls(block, &mut calls)?;
    float_calls(block, &mut calls)?;
    flag_calls(block, &mut calls)?;
    kind_calls(block, &mut calls)?;
    rule_calls(block, &mut calls)?;
    if let Some(version) = since {
        calls.push(format!(
            "since(Version::V{})",
            crate::parity::pins::tag(version)
        ));
    }
    let mut out = format!("    Block::of({}, BlockClass::{class})", text::string(name));
    for call in calls {
        let _ = write!(out, ".{call}");
    }
    out.push(',');
    Ok(out)
}

/// The captures to diff, oldest first, the newest last.
fn captures(versions: &Versions) -> Result<Vec<(String, Captured)>, String> {
    let mut all = Vec::new();
    for version in versions.older.iter().chain([&versions.newest]) {
        all.push((version.clone(), blocks(version)?));
    }
    Ok(all)
}

/// The first version that has `name`: the first capture that lists it, or, for a block the oldest
/// capture lists, the interim input's version where it has one.
fn first_version(
    name: &str,
    captures: &[(String, Captured)],
    uncaptured: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let present: Vec<bool> = captures
        .iter()
        .map(|(_, blocks)| blocks.contains_key(name))
        .collect();
    let first = present
        .iter()
        .position(|here| *here)
        .ok_or_else(|| format!("{name} is in no capture"))?;
    if present[first..].iter().any(|here| !*here) {
        return Err(format!(
            "{name} left a version; the table has no `until` yet"
        ));
    }
    let oldest = &captures[0].0;
    match (first, uncaptured.get(name)) {
        (0, None) => Ok(None),
        (0, Some(version)) if version == oldest => Ok(Some(version.clone())),
        (0, Some(version)) => Err(format!(
            "{name}: block_spans_uncaptured.toml names {version}, and only {oldest}, the oldest capture, can be named"
        )),
        (_, None) => Ok(Some(captures[first].0.clone())),
        (_, Some(_)) => Err(format!(
            "{name}: block_spans_uncaptured.toml contradicts the capture, which lacks the block in {oldest}"
        )),
    }
}

/// The properties of a block must be the same in every captured version: a property a version
/// lacks needs `property_since`, which no block uses yet.
fn same_properties(name: &str, captures: &[(String, Captured)]) -> Result<(), String> {
    let newest = &captures[captures.len() - 1].1[name];
    for (version, blocks) in captures {
        let Some(block) = blocks.get(name) else {
            continue;
        };
        if block["properties"] != newest["properties"] || block["default"] != newest["default"] {
            return Err(format!(
                "{name}: its properties differ in {version}; the generator has to write property_since for it"
            ));
        }
    }
    Ok(())
}

pub fn render(versions: &Versions) -> Result<String, String> {
    let captures = captures(versions)?;
    let newest = &versions.newest;
    let rows = super::capture::rows(newest, "block", "block/declarations")?;
    let list = rows
        .first()
        .and_then(|row| row["blocks"].as_array())
        .ok_or("no block declarations")?;
    let uncaptured: Uncaptured = toml::from_str(UNCAPTURED).map_err(|error| error.to_string())?;
    let names: BTreeSet<&str> = list.iter().filter_map(|b| b["name"].as_str()).collect();
    if let Some(stray) = uncaptured
        .first_version
        .keys()
        .find(|name| !names.contains(name.as_str()))
    {
        return Err(format!(
            "block_spans_uncaptured.toml: {stray} is not a block"
        ));
    }
    let mut body = String::new();
    for block in list {
        let name = block["name"].as_str().ok_or("a block name")?;
        same_properties(name, &captures)?;
        let since = first_version(name, &captures, &uncaptured.first_version)?;
        let _ = writeln!(body, "{}", line(block, since.as_deref())?);
    }
    let mut out = text::header(
        "Every block, in registry order, as the reference's `Blocks` declares it.",
        "blocks",
    );
    out.push_str(PRELUDE);
    let _ = write!(
        out,
        "#[rustfmt::skip]\npub static BLOCKS: [Block; {}] = [\n{body}];\n",
        list.len()
    );
    Ok(out)
}

const PRELUDE: &str = "use crate::block::classes::BlockClass;
use crate::block::definition::{Block, Emission, Fluid, OffsetType, PushReaction, Rule};
use crate::block::properties::*;
use photon_core::version::Version;

pub use super::ids::{id, index_of};

";
