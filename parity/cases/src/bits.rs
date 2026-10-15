//! Floats travel as their IEEE bit patterns, so no decimal rounding sits
//! between a case and the oracle.

use serde_json::{Value, json};

/// The big-endian hex bits of `value` rounded to `f32`.
pub fn f32_bits(value: impl Into<f64>) -> String {
    format!("{:08x}", (value.into() as f32).to_bits())
}

/// The big-endian hex bits of `value`.
pub fn f64_bits(value: impl Into<f64>) -> String {
    format!("{:016x}", value.into().to_bits())
}

/// An `{x, y, z}` vector of `f64` bit patterns.
pub fn vector_bits(x: f64, y: f64, z: f64) -> Value {
    json!({"x": f64_bits(x), "y": f64_bits(y), "z": f64_bits(z)})
}

/// `bytes` as lowercase hex.
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// `value` as a protocol VarInt (its low 32 bits, unsigned).
pub fn varint(value: i64) -> Vec<u8> {
    let mut value = (value as u64) & 0xffff_ffff;
    let mut out = Vec::new();
    loop {
        let part = (value & 0x7f) as u8;
        value >>= 7;
        out.push(part | if value != 0 { 0x80 } else { 0 });
        if value == 0 {
            return out;
        }
    }
}
