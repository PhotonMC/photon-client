//! Rows built from a table of defaults and the fields a row changes, the
//! way most entity scenarios differ from a standing pose by a few values.

use serde_json::{Map, Value, json};

use crate::bits::f32_bits;

/// A field value: floats travel as `f32` bit patterns.
#[derive(Clone, Copy, Debug)]
pub enum Field {
    Float(f64),
    Bool(bool),
    Int(i64),
    Text(&'static str),
}

impl From<f64> for Field {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<bool> for Field {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i32> for Field {
    fn from(value: i32) -> Self {
        Self::Int(i64::from(value))
    }
}

impl From<i64> for Field {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<&'static str> for Field {
    fn from(value: &'static str) -> Self {
        Self::Text(value)
    }
}

impl Field {
    /// The JSON of this value where `default` is the field's default: an
    /// integer given for a float field is still a float.
    fn json(self, default: Option<Field>) -> Value {
        match (self, default) {
            (Self::Int(value), Some(Self::Float(_))) => json!(f32_bits(value as f64)),
            (Self::Float(value), _) => json!(f32_bits(value)),
            (Self::Bool(value), _) => json!(value),
            (Self::Int(value), _) => json!(value),
            (Self::Text(value), _) => json!(value),
        }
    }
}

/// `(field, value)` pairs.
pub type Fields<'a> = &'a [(&'static str, Field)];

/// `[(stringify!(key), Field::from(value)), ...]`.
macro_rules! fields {
    ($($key:ident = $value:expr),* $(,)?) => {
        &[$((stringify!($key), $crate::entity::pose::Field::from($value))),*]
    };
}
pub(crate) use fields;

/// Every default, replaced where `set` names it, plus the fields of `set`
/// that have no default.
pub fn merged(defaults: Fields, set: Fields) -> Map<String, Value> {
    let mut map = Map::new();
    for (key, default) in defaults {
        let value = set.iter().find(|(name, _)| name == key).map_or(*default, |(_, value)| *value);
        map.insert((*key).to_owned(), value.json(Some(*default)));
    }
    for (key, value) in set {
        if !defaults.iter().any(|(name, _)| name == key) {
            map.insert((*key).to_owned(), value.json(None));
        }
    }
    map
}

/// A row labelled `<layer>-<label>` for a model `layer`.
pub fn layer_row(layer: &str, label: &str, defaults: Fields, set: Fields) -> Value {
    let mut map = merged(defaults, set);
    map.insert("label".to_owned(), json!(format!("{layer}-{label}")));
    map.insert("layer".to_owned(), json!(layer));
    Value::Object(map)
}

/// A row labelled `label`.
pub fn row(label: &str, defaults: Fields, set: Fields) -> Value {
    let mut map = merged(defaults, set);
    map.insert("label".to_owned(), json!(label));
    Value::Object(map)
}

/// The usual look and walk fields, all zero.
pub const LOOK_WALK: [(&str, Field); 4] = [
    ("head_y_rot", Field::Float(0.0)),
    ("x_rot", Field::Float(0.0)),
    ("walk_position", Field::Float(0.0)),
    ("walk_speed", Field::Float(0.0)),
];

/// `0.5` for a baby's layer, else `1.0`.
pub fn age_scale(layer: &str) -> f64 {
    if layer.contains("_baby") { 0.5 } else { 1.0 }
}

/// A case of the entity family.
pub fn entity(label: &str, operation: &str, rows: Vec<Value>) -> Value {
    crate::case::case(format!("entity/{label}"), "entity", operation, json!({ "rows": rows }))
}

/// A case whose oracle reads a whole table: one `all` row.
pub fn table(label: &str, operation: &str) -> Value {
    entity(label, operation, vec![json!({"label": "all"})])
}

/// Rows labelled `<prefix>-<label>` for the model `layer`.
pub fn prefixed(prefix: &str, layer: &str, defaults: Fields, poses: &[(&str, Fields)]) -> Vec<Value> {
    poses
        .iter()
        .map(|(label, set)| {
            let mut row = row(&format!("{prefix}-{label}"), defaults, set);
            row["layer"] = json!(layer);
            row
        })
        .collect()
}
