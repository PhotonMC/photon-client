//! The `custom_model_data` properties an item model decides from a stack's lists.

use serde_json::{Value, json};

use super::gui;
use crate::bits::f32_bits;
use crate::components::{CUSTOM_MODEL_DATA, custom_model_data, patch};

/// A stack of `item` whose patch holds the given `custom_model_data`.
fn row(label: &str, extra: Value, item: &str, data: Option<String>) -> Value {
    let mut row = json!({"label": label, "item": item, "components": []});
    if let Some(data) = data {
        row["patch"] = json!(patch(&[(CUSTOM_MODEL_DATA, data)]));
    }
    for (key, value) in extra.as_object().into_iter().flatten() {
        row[key] = value.clone();
    }
    row
}

/// The stacks the properties are asked of: lists of every length, and none.
fn stacks() -> Vec<(&'static str, Option<String>)> {
    let floats = [f32_bits(1.5), f32_bits(-2.0), "7fc00000".to_owned(), f32_bits(3.0e38)];
    let floats: Vec<&str> = floats.iter().map(String::as_str).collect();
    vec![
        ("none", None),
        ("empty", Some(custom_model_data(&[], &[], &[], &[]))),
        (
            "full",
            Some(custom_model_data(
                &floats,
                &[true, false, true],
                &["a", "", "caf\u{e9}", "\u{1f600}x"],
                &[0xff00ff, -1, 0x123456],
            )),
        ),
        ("floats-only", Some(custom_model_data(&floats[..1], &[], &[], &[]))),
        ("flags-only", Some(custom_model_data(&[], &[false], &[], &[]))),
        ("strings-only", Some(custom_model_data(&[], &[], &["x", "y"], &[]))),
        ("colors-only", Some(custom_model_data(&[], &[], &[], &[7]))),
    ]
}

/// One row for each stack and each index (the property's own default included).
fn rows_for(property: impl Fn(Option<i64>) -> Value, key: &str) -> Vec<Value> {
    let mut rows = Vec::new();
    for (name, data) in stacks() {
        for index in [None, Some(0), Some(1), Some(2), Some(3), Some(4), Some(40)] {
            let label = format!("{name}-{}", index.map_or("default".to_owned(), |index| index.to_string()));
            rows.push(row(&label, json!({key: property(index)}), "minecraft:stone", data.clone()));
        }
    }
    rows
}

/// The flag condition: set, unset and out of range.
pub fn conditions() -> Vec<Value> {
    rows_for(|index| with_index(json!({"property": "minecraft:custom_model_data"}), index), "property")
}

/// The float range: the value at the index, or zero.
pub fn numbers() -> Vec<Value> {
    rows_for(|index| with_index(json!({"property": "minecraft:custom_model_data"}), index), "property")
}

/// The string select: the value at the index, or none.
pub fn selects() -> Vec<Value> {
    let mut rows = Vec::new();
    for (name, data) in stacks() {
        for index in [0, 1, 2, 3, 4, 40] {
            rows.push(row(&format!("{name}-{index}"), json!({"custom_index": index}), "minecraft:stone", data.clone()));
        }
    }
    rows
}

fn with_index(mut property: Value, index: Option<i64>) -> Value {
    if let Some(index) = index {
        property["index"] = json!(index);
    }
    property
}

/// The colour tint: the colour at the index, or the default.
pub fn tints() -> Vec<Value> {
    let mut rows = Vec::new();
    for (name, data) in stacks() {
        for index in [None, Some(0), Some(1), Some(2), Some(3), Some(40)] {
            let mut tint = json!({"type": "minecraft:custom_model_data", "default": 0x00ab_cdef});
            if let Some(index) = index {
                tint["index"] = json!(index);
            }
            let label = format!("{name}-{}", index.map_or("default".to_owned(), |index| index.to_string()));
            rows.push(row(&label, json!({"tint": tint}), "minecraft:stone", data.clone()));
        }
    }
    rows
}

/// The cases of the `custom_model_data` properties.
pub fn cases() -> Vec<Value> {
    vec![
        gui("item-custom-condition", "item_condition", conditions()),
        gui("item-custom-number", "item_number", numbers()),
        gui("item-custom-select", "item_select", selects()),
        gui("item-custom-tint", "item_tint", tints()),
    ]
}
