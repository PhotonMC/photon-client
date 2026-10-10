//! Floats travel as their IEEE bit patterns, so no decimal rounding sits
//! between a case and the oracle.

use serde_json::{Value, json};

/// The big-endian hex bits of `value` rounded to `f32`.
pub fn f32_bits(value: f64) -> String {
    format!("{:08x}", (value as f32).to_bits())
}

/// The big-endian hex bits of `value`.
pub fn f64_bits(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

/// An `{x, y, z}` vector of `f64` bit patterns.
pub fn vector_bits(x: f64, y: f64, z: f64) -> Value {
    json!({"x": f64_bits(x), "y": f64_bits(y), "z": f64_bits(z)})
}

/// `bytes` as lowercase hex.
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
