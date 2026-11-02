//! The tag a tag query answer holds, written as text the way the game writes it: `Tag.toString`
//! and the plain text of the pretty printer, whose keys come in the game's hash map order.

use serde_json::{Value, json};

use super::codec::varint_hex;
use crate::bits::hex;
use crate::case::case;

/// A tag: its type id and its payload.
type Tag = (u8, Vec<u8>);

fn byte(value: i8) -> Tag {
    (1, vec![value as u8])
}

fn short(value: i16) -> Tag {
    (2, value.to_be_bytes().to_vec())
}

fn int(value: i32) -> Tag {
    (3, value.to_be_bytes().to_vec())
}

fn long(value: i64) -> Tag {
    (4, value.to_be_bytes().to_vec())
}

fn float(value: f32) -> Tag {
    (5, value.to_be_bytes().to_vec())
}

fn double(value: f64) -> Tag {
    (6, value.to_be_bytes().to_vec())
}

fn text(value: &str) -> Tag {
    let mut out = (value.len() as u16).to_be_bytes().to_vec();
    out.extend_from_slice(value.as_bytes());
    (8, out)
}

fn byte_array(values: &[i8]) -> Tag {
    let mut out = (values.len() as i32).to_be_bytes().to_vec();
    out.extend(values.iter().map(|value| *value as u8));
    (7, out)
}

fn int_array(values: &[i32]) -> Tag {
    let mut out = (values.len() as i32).to_be_bytes().to_vec();
    values.iter().for_each(|value| out.extend_from_slice(&value.to_be_bytes()));
    (11, out)
}

fn long_array(values: &[i64]) -> Tag {
    let mut out = (values.len() as i32).to_be_bytes().to_vec();
    values.iter().for_each(|value| out.extend_from_slice(&value.to_be_bytes()));
    (12, out)
}

fn list(items: &[Tag]) -> Tag {
    let kind = items.first().map_or(0, |item| item.0);
    let mut out = vec![kind];
    out.extend_from_slice(&(items.len() as i32).to_be_bytes());
    items.iter().for_each(|item| out.extend_from_slice(&item.1));
    (9, out)
}

fn compound(entries: &[(&str, Tag)]) -> Tag {
    let mut out = Vec::new();
    for (key, (kind, payload)) in entries {
        out.push(*kind);
        out.extend_from_slice(&(key.len() as u16).to_be_bytes());
        out.extend_from_slice(key.as_bytes());
        out.extend_from_slice(payload);
    }
    out.push(0);
    (10, out)
}

/// A tag query answer for a transaction, the tag nameless as it is on the wire.
fn answer(transaction: i64, tag: Option<&Tag>) -> String {
    let mut out = varint_hex(transaction);
    match tag {
        Some((kind, payload)) => {
            out += &hex(&[*kind]);
            out += &hex(payload);
        }
        None => out += "00",
    }
    out
}

fn nested(depth: usize) -> Tag {
    let mut tag = compound(&[("end", int(1))]);
    for _ in 0..depth {
        tag = compound(&[("in", tag)]);
    }
    tag
}

fn value_rows() -> Vec<(&'static str, Option<Tag>)> {
    let many: Vec<(String, Tag)> = (0..30).map(|index| (format!("key{index}"), int(index))).collect();
    let many: Vec<(&str, Tag)> = many.iter().map(|(key, tag)| (key.as_str(), tag.clone())).collect();
    vec![
        ("none", None),
        ("empty", Some(compound(&[]))),
        (
            "scalars",
            Some(compound(&[
                ("b", byte(-5)),
                ("s", short(300)),
                ("i", int(-70000)),
                ("l", long(1 << 40)),
                ("f", float(1.5)),
                ("d", double(0.1)),
            ])),
        ),
        (
            "floats",
            Some(compound(&[
                ("a", float(1.0e10)),
                ("b", float(-0.0)),
                ("c", float(f32::NAN)),
                ("d", double(1.0e-5)),
                ("e", double(f64::INFINITY)),
                ("f", float(1.0e-3)),
                ("g", double(123456789.0)),
            ])),
        ),
        (
            "entity",
            Some(compound(&[
                ("UUID", int_array(&[1, 2, 3, 4])),
                ("Pos", list(&[double(1.0), double(64.0), double(-3.5)])),
                ("id", text("minecraft:pig")),
                ("Health", float(10.0)),
                ("Air", short(300)),
                ("OnGround", byte(1)),
                ("Tags", list(&[text("a"), text("b")])),
                ("CustomName", text("\"Bob\"")),
                ("Motion", list(&[double(0.0), double(0.0), double(0.0)])),
                ("Rotation", list(&[float(90.0), float(0.0)])),
                ("FallDistance", double(0.0)),
                ("Fire", short(-1)),
                ("Invulnerable", byte(0)),
                ("PortalCooldown", int(0)),
                ("Saddle", compound(&[("id", text("minecraft:saddle")), ("count", int(1))])),
            ])),
        ),
        ("many-keys", Some(compound(&many))),
    ]
}

fn structure_rows() -> Vec<(&'static str, Option<Tag>)> {
    vec![
        (
            "arrays",
            Some(compound(&[
                ("bytes", byte_array(&[1, -2, 3])),
                ("ints", int_array(&[5, -6])),
                ("longs", long_array(&[7, -8])),
                ("none", byte_array(&[])),
            ])),
        ),
        (
            "lists",
            Some(compound(&[
                ("empty", list(&[])),
                ("numbers", list(&[int(1), int(2)])),
                ("long-numbers", list(&(0..10).map(int).collect::<Vec<_>>())),
                ("strings", list(&[text("x"), text("y")])),
                ("compounds", list(&[compound(&[("a", int(1))]), compound(&[("b", int(2))])])),
                ("nested", list(&[list(&[int(1)]), list(&[int(2)])])),
            ])),
        ),
        (
            "strings",
            Some(compound(&[
                ("plain", text("hello")),
                ("double", text("say \"hi\"")),
                ("single", text("it's")),
                ("both", text("it's \"x\"")),
                ("slash", text("a\\b")),
                ("control", text("t\tn\nr\rb\u{8}f\u{c}\u{1}")),
                ("empty", text("")),
                ("unicode", text("\u{e9}\u{4e2d}")),
            ])),
        ),
        (
            "keys",
            Some(compound(&[
                ("true", int(1)),
                ("False", int(2)),
                ("_a.b", int(3)),
                ("1abc", int(4)),
                ("a b", int(5)),
                ("", int(6)),
                ("a+b-c", int(7)),
                ("quote\"d", int(8)),
                ("it's", int(9)),
                ("a:b", int(10)),
            ])),
        ),
        ("list-deep", Some(compound(&[("l", list(&(0..200).map(|_| compound(&[])).collect::<Vec<_>>()))]))),
        (
            "big-arrays",
            Some(compound(&[
                ("bytes", byte_array(&(0..200).map(|v| v as i8).collect::<Vec<_>>())),
                ("ints", int_array(&(0..130).collect::<Vec<_>>())),
            ])),
        ),
    ]
}

fn deep_rows() -> Vec<(&'static str, Option<Tag>)> {
    vec![("deep-63", Some(nested(63))), ("deep-70", Some(nested(70)))]
}

pub fn cases() -> Vec<Value> {
    let mut cases = Vec::new();
    for (label, tag) in value_rows().into_iter().chain(structure_rows()).chain(deep_rows()) {
        cases.push(case(
            format!("game/tag_query/{label}"),
            "wire",
            "play_packet",
            json!({"packet": "tag_query", "hex": answer(7, tag.as_ref())}),
        ));
    }
    let scalar = answer(0, Some(&int(1)));
    cases.push(case(
        "game/tag_query/not-a-compound".to_owned(),
        "wire",
        "play_packet",
        json!({"packet": "tag_query", "hex": scalar}),
    ));
    cases.push(case(
        "game/tag_query/trailing".to_owned(),
        "wire",
        "play_packet",
        json!({"packet": "tag_query", "hex": format!("{}00", answer(1, None))}),
    ));
    cases
}
