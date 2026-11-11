//! The reference's model layers: the newest version's table, and for each older version the
//! layers its definition differs in and the ids it lacks.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde_json::Value;

use super::{Versions, capture, text};

/// The capture case that lists every layer with its full tree.
const FAMILY: &str = "entity";
const CASE: &str = "entity/model-layers";

/// One layer per id, in order of id (the order a lookup searches).
type Layers = BTreeMap<String, Value>;

fn layers(version: &str) -> Result<Layers, String> {
    let mut out = Layers::new();
    for row in capture::rows(version, FAMILY, CASE)? {
        let id = row["label"]
            .as_str()
            .ok_or("model layer: no label")?
            .to_owned();
        if out.insert(id.clone(), row).is_some() {
            return Err(format!("model layer {id} appears twice in {version}"));
        }
    }
    Ok(out)
}

/// The newest version's table.
pub fn render(versions: &Versions) -> Result<String, String> {
    let layers = layers(&versions.newest)?;
    let mut out = text::header(
        "Every model layer the reference builds, in order of id; the tree of each is as `LayerDefinitions` assembles it.",
        "entity_models",
    );
    out.push_str(APPROX_ALLOW);
    out.push_str(IMPORTS);
    out.push_str("#[rustfmt::skip]\n");
    let _ = writeln!(out, "pub static LAYERS: [ModelLayer; {}] = [", layers.len());
    layer_rows(&mut out, layers.values())?;
    out.push_str("];\n");
    Ok(out)
}

/// What each older version differs in: the layers whose definition is not the newest's (and
/// the ones only it has), and the ids of the newest's layers it does not have.
pub fn render_older(versions: &Versions) -> Result<String, String> {
    let newest = layers(&versions.newest)?;
    let mut out = text::header(
        "What each older version's model layers differ in from the newest version's table.",
        "entity_models",
    );
    out.push_str(
        "#![allow(unused_imports, reason = \"a version with no replaced layer uses none\")]\n",
    );
    out.push_str(APPROX_ALLOW);
    out.push_str("use photon_core::version::Version;\n\n");
    out.push_str(IMPORTS);
    let mut entries = String::new();
    for version in &versions.older {
        let tag = crate::parity::pins::tag(version);
        let older = layers(version)?;
        let (replaced, absent) = difference(&newest, &older);
        out.push_str("#[rustfmt::skip]\n");
        let _ = writeln!(
            out,
            "static REPLACED_{tag}: [ModelLayer; {}] = [",
            replaced.len()
        );
        layer_rows(&mut out, replaced.iter().copied())?;
        out.push_str("];\n\n");
        let _ = writeln!(out, "static ABSENT_{tag}: [&str; {}] = [", absent.len());
        for id in &absent {
            let _ = writeln!(out, "    {},", text::string(id));
        }
        out.push_str("];\n\n");
        let _ = writeln!(
            entries,
            "    (Version::V{tag}, &REPLACED_{tag}, &ABSENT_{tag}),"
        );
    }
    out.push_str(OLDER_DOC);
    let _ = writeln!(
        out,
        "pub static OLDER: &[(Version, &[ModelLayer], &[&str])] = &[\n{entries}];"
    );
    Ok(out)
}

const APPROX_ALLOW: &str = "#![allow(\n    clippy::approx_constant,\n    reason = \"the reference writes 1.5708 and 3.1416 for its angles\"\n)]\n\n";

const IMPORTS: &str =
    "use crate::entity_model::{Face, ModelCube, ModelLayer, ModelPart, PartPose};\n\n";

const OLDER_DOC: &str = "/// For each version older than the newest: the layers it defines differently or alone (in order\n/// of id) and the ids of the newest table's layers it does not have.\n#[rustfmt::skip]\n";

/// The layers of `older` the newest table does not hold as they are, and the ids of the newest
/// table's layers `older` lacks.
fn difference<'a>(newest: &'a Layers, older: &'a Layers) -> (Vec<&'a Value>, Vec<&'a str>) {
    let replaced = older
        .iter()
        .filter(|(id, layer)| newest.get(*id) != Some(*layer))
        .map(|(_, layer)| layer)
        .collect();
    let absent = newest
        .keys()
        .filter(|id| !older.contains_key(*id))
        .map(String::as_str)
        .collect();
    (replaced, absent)
}

fn layer_rows<'a>(out: &mut String, layers: impl Iterator<Item = &'a Value>) -> Result<(), String> {
    for layer in layers {
        let id = layer["label"].as_str().ok_or("model layer: no label")?;
        let open = format!(
            "ModelLayer::of({}, {}, {}, ",
            text::string(id),
            layer["texture_width"],
            layer["texture_height"]
        );
        part(out, &layer["root"], 1, &open, "),").map_err(|error| format!("{id}: {error}"))?;
    }
    Ok(())
}

/// A part and its subtree, indented `depth` levels. `open` stands before the part on its first
/// line and `close` after its last.
fn part(
    out: &mut String,
    node: &Value,
    depth: usize,
    open: &str,
    close: &str,
) -> Result<(), String> {
    let indent = "    ".repeat(depth);
    let name = node["name"].as_str().ok_or("part: no name")?;
    let cubes = node["cubes"].as_array().ok_or("part: no cubes")?;
    let cubes = cubes.iter().map(cube).collect::<Result<Vec<_>, _>>()?;
    let _ = write!(
        out,
        "{indent}{open}ModelPart::of({}, {}, &[{}], &[",
        text::string(name),
        pose(&node["pose"])?,
        cubes.join(", ")
    );
    let children = node["children"].as_array().ok_or("part: no children")?;
    if children.is_empty() {
        let _ = writeln!(out, "]){close}");
        return Ok(());
    }
    out.push('\n');
    for child in children {
        part(out, child, depth + 1, "", ",")?;
    }
    let _ = writeln!(out, "{indent}]){close}");
    Ok(())
}

/// The words of a list of float bit patterns.
fn words(value: &Value, count: usize) -> Result<Vec<u32>, String> {
    let list = value
        .as_array()
        .filter(|list| list.len() == count)
        .ok_or_else(|| format!("expected {count} floats"))?;
    list.iter()
        .map(|item| {
            let hex = item.as_str().ok_or("a float is not a string")?;
            u32::from_str_radix(hex, 16).map_err(|error| format!("{hex}: {error}"))
        })
        .collect()
}

const ZERO: u32 = 0;
const ONE: u32 = 0x3f80_0000;

/// A vector literal: `[1.0, 2.0, 3.0]`.
fn vector(bits: &[u32]) -> Result<String, String> {
    let items = bits
        .iter()
        .map(|bits| text::f32_literal(*bits))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(format!("[{}]", items.join(", ")))
}

fn all_are(bits: &[u32], value: u32) -> bool {
    bits.iter().all(|bits| *bits == value)
}

/// A `PartPose` from position, rotation and scale, by the shortest constructor that makes it.
fn pose(value: &Value) -> Result<String, String> {
    let bits = words(value, 9)?;
    let (position, rotation, scale) = (&bits[0..3], &bits[3..6], &bits[6..9]);
    if !all_are(scale, ONE) {
        let parts = [vector(position)?, vector(rotation)?, vector(scale)?];
        return Ok(format!("PartPose::new({})", parts.join(", ")));
    }
    Ok(match (all_are(position, ZERO), all_are(rotation, ZERO)) {
        (true, true) => "PartPose::ZERO".to_owned(),
        (false, true) => format!("PartPose::offset({})", vector(position)?),
        (true, false) => format!("PartPose::rotation({})", vector(rotation)?),
        (false, false) => format!(
            "PartPose::offset_and_rotation({}, {})",
            vector(position)?,
            vector(rotation)?
        ),
    })
}

/// The faces a cube shows when it names none.
const ALL_FACES: [&str; 6] = ["down", "east", "north", "south", "up", "west"];

/// A `Face` variant from its name.
fn face(name: &str) -> Result<&'static str, String> {
    Ok(match name {
        "down" => "Face::Down",
        "east" => "Face::East",
        "north" => "Face::North",
        "south" => "Face::South",
        "up" => "Face::Up",
        "west" => "Face::West",
        other => return Err(format!("no face {other}")),
    })
}

/// A cube: the constructor, then a builder call for each field that is not the default.
fn cube(value: &Value) -> Result<String, String> {
    let mut out = format!(
        "ModelCube::of({}, {}, {})",
        vector(&words(&value["uv"], 2)?)?,
        vector(&words(&value["origin"], 3)?)?,
        vector(&words(&value["dimensions"], 3)?)?
    );
    let grow = words(&value["grow"], 3)?;
    if !all_are(&grow, ZERO) {
        let _ = write!(out, ".grow({})", vector(&grow)?);
    }
    if value["mirror"].as_bool().ok_or("cube: no mirror")? {
        out.push_str(".mirror()");
    }
    let uv_scale = words(&value["uv_scale"], 2)?;
    if !all_are(&uv_scale, ONE) {
        let _ = write!(out, ".uv_scale({})", vector(&uv_scale)?);
    }
    let faces: Vec<&str> = value["faces"]
        .as_array()
        .ok_or("cube: no faces")?
        .iter()
        .map(|name| name.as_str().ok_or("cube: a face is not a string"))
        .collect::<Result<_, _>>()?;
    if faces != ALL_FACES {
        let names = faces
            .iter()
            .map(|name| face(name))
            .collect::<Result<Vec<_>, _>>()?;
        let _ = write!(out, ".faces(&[{}])", names.join(", "));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn leaf(name: &str, pose: Value) -> Value {
        json!({"name": name, "pose": pose, "cubes": [], "children": []})
    }

    fn zero_pose() -> Value {
        json!([
            "00000000", "00000000", "00000000", "00000000", "00000000", "00000000", "3f800000",
            "3f800000", "3f800000"
        ])
    }

    #[test]
    fn a_pose_uses_the_shortest_constructor() {
        assert_eq!(pose(&zero_pose()).as_deref(), Ok("PartPose::ZERO"));
        let mut moved = zero_pose();
        moved[1] = json!("3f800000");
        assert_eq!(
            pose(&moved).as_deref(),
            Ok("PartPose::offset([0.0, 1.0, 0.0])")
        );
        let mut scaled = zero_pose();
        scaled[7] = json!("40000000");
        assert!(pose(&scaled).unwrap().starts_with("PartPose::new("));
    }

    #[test]
    fn a_cube_writes_only_what_is_not_the_default() {
        let zero = json!(["00000000", "00000000", "00000000"]);
        let cube_value = json!({
            "uv": ["41200000", "00000000"], "origin": zero, "dimensions": ["3f800000", "3f800000", "3f800000"],
            "grow": zero, "mirror": true, "uv_scale": ["3f800000", "3f800000"], "faces": ["north"],
        });
        assert_eq!(
            cube(&cube_value).as_deref(),
            Ok(
                "ModelCube::of([10.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]).mirror().faces(&[Face::North])"
            )
        );
    }

    #[test]
    fn a_part_nests_its_children() {
        let mut root = leaf("root", zero_pose());
        root["children"] = json!([leaf("head", zero_pose())]);
        let mut out = String::new();
        part(&mut out, &root, 1, "ModelLayer::of(\"a#b\", 8, 8, ", "),").unwrap();
        assert_eq!(
            out,
            "    ModelLayer::of(\"a#b\", 8, 8, ModelPart::of(\"root\", PartPose::ZERO, &[], &[\n        ModelPart::of(\"head\", PartPose::ZERO, &[], &[]),\n    ])),\n"
        );
    }

    #[test]
    fn the_older_table_keeps_what_differs() {
        let layer = |tag: i64| json!({"label": "x", "tag": tag});
        let mut newest = Layers::new();
        newest.insert("a".into(), layer(1));
        newest.insert("b".into(), layer(1));
        newest.insert("c".into(), layer(1));
        let mut older = Layers::new();
        older.insert("a".into(), layer(1));
        older.insert("b".into(), layer(2));
        older.insert("z".into(), layer(1));
        let (replaced, absent) = difference(&newest, &older);
        assert_eq!(replaced, vec![&layer(2), &layer(1)]);
        assert_eq!(absent, vec!["c"]);
    }
}
