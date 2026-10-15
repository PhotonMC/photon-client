//! The values the `math` cases share: vectors, boxes and block positions as
//! the oracle reads them.

use serde_json::{Map, Value, json};

use crate::bits::{f32_bits, f64_bits};

/// An `{x, y, z}` vector of `f64` bit patterns.
pub fn vector_bits(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>) -> Value {
    json!({"x": f64_bits(x), "y": f64_bits(y), "z": f64_bits(z)})
}

/// An `{x, y, z}` vector of `f32` bit patterns.
pub fn vector_bits_f32(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>) -> Value {
    json!({"x": f32_bits(x), "y": f32_bits(y), "z": f32_bits(z)})
}

/// An `{x, y}` vector of `f32` bit patterns.
pub fn vector2_bits(x: impl Into<f64>, y: impl Into<f64>) -> Value {
    json!({"x": f32_bits(x), "y": f32_bits(y)})
}

/// A box of `f64` bit patterns.
pub fn box_bits(
    min_x: impl Into<f64>,
    min_y: impl Into<f64>,
    min_z: impl Into<f64>,
    max_x: impl Into<f64>,
    max_y: impl Into<f64>,
    max_z: impl Into<f64>,
) -> Value {
    json!({
        "min_x": f64_bits(min_x), "min_y": f64_bits(min_y), "min_z": f64_bits(min_z),
        "max_x": f64_bits(max_x), "max_y": f64_bits(max_y), "max_z": f64_bits(max_z),
    })
}

/// A point of `f64` bit patterns.
pub fn point(x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>) -> Value {
    vector_bits(x, y, z)
}

/// A block position, its coordinates as text.
pub fn block_position(x: i64, y: i64, z: i64) -> Value {
    json!({"x": x.to_string(), "y": y.to_string(), "z": z.to_string()})
}

/// The unit cube at the origin.
pub fn unit() -> Value {
    box_bits(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
}

/// The fields of each object in order, later ones replacing earlier ones.
pub fn merged(parts: Vec<Value>) -> Value {
    let mut out = Map::new();
    for part in parts {
        if let Value::Object(fields) = part {
            out.extend(fields);
        }
    }
    Value::Object(out)
}

/// A number the oracle reads as bits, or bits already written out.
#[derive(Clone, Debug)]
pub enum Bits {
    Raw(String),
    Num(f64),
}

/// Bits already written out.
pub fn raw(text: &str) -> Bits {
    Bits::Raw(text.to_owned())
}

/// A number to be written as bits.
pub fn num(value: f64) -> Bits {
    Bits::Num(value)
}

impl From<f64> for Bits {
    fn from(value: f64) -> Self {
        Self::Num(value)
    }
}

impl From<i32> for Bits {
    fn from(value: i32) -> Self {
        Self::Num(f64::from(value))
    }
}

impl From<&str> for Bits {
    fn from(value: &str) -> Self {
        Self::Raw(value.to_owned())
    }
}

impl Bits {
    /// The `f32` bits: raw text is kept as it is.
    pub fn f32(&self) -> String {
        match self {
            Self::Raw(text) => text.clone(),
            Self::Num(value) => f32_bits(*value),
        }
    }

    /// The `f64` bits: raw text is kept as it is.
    pub fn f64(&self) -> String {
        match self {
            Self::Raw(text) => text.clone(),
            Self::Num(value) => f64_bits(*value),
        }
    }
}

/// `value` as bits of the width named, raw bits as they are.
pub fn interpolation_bits(value: impl Into<Bits>, width: &str) -> String {
    let bits: Bits = value.into();
    if width == "f32" { bits.f32() } else { bits.f64() }
}

/// An attribute modifier: its id, amount and operation.
pub fn attribute_modifier(identifier: &str, amount: impl Into<f64>, operation: i64) -> Value {
    json!({"id": identifier, "amount": f64_bits(amount), "operation": operation})
}

/// A box of raw values.
pub fn raw_box(
    min_x: impl Into<Value>,
    min_y: impl Into<Value>,
    min_z: impl Into<Value>,
    max_x: impl Into<Value>,
    max_y: impl Into<Value>,
    max_z: impl Into<Value>,
) -> Value {
    json!({
        "min_x": min_x.into(), "min_y": min_y.into(), "min_z": min_z.into(),
        "max_x": max_x.into(), "max_y": max_y.into(), "max_z": max_z.into(),
    })
}

/// The bits of `value` and the two beside them.
pub fn neighbors(value: f64, width: &str) -> Vec<String> {
    let (bits, digits) =
        if width == "f32" { (i128::from((value as f32).to_bits()), 8) } else { (i128::from(value.to_bits()), 16) };
    [-1, 0, 1].iter().map(|offset| format!("{:0digits$x}", bits + offset)).collect()
}

/// The bits of `value` in the width named (`"f32"` or `"f64"`).
pub fn float_bits(value: impl Into<f64>, width: &str) -> String {
    if width == "f32" { f32_bits(value) } else { f64_bits(value) }
}

/// The value of `key` in a table of named entries.
pub fn lookup<T: Clone>(entries: &[(&str, T)], key: &str) -> T {
    entries.iter().find(|(name, _)| *name == key).map(|(_, value)| value.clone()).expect("a named entry")
}

/// A position in a block search: its status and the positions beside it.
pub fn traversal_node(position: (i64, i64, i64), status: &str, neighbors: Vec<(i64, i64, i64)>) -> Value {
    let neighbors: Vec<Value> =
        neighbors.iter().map(|(x, y, z)| json!({"x": x.to_string(), "y": y.to_string(), "z": z.to_string()})).collect();
    json!({
        "position": {"x": position.0.to_string(), "y": position.1.to_string(), "z": position.2.to_string()},
        "status": status, "neighbors": neighbors,
    })
}

/// The `f32` nearest 1e-5, as an `f64`: the threshold the reference compares
/// squared lengths with.
pub fn threshold() -> f64 {
    f64::from(1e-5_f32)
}
