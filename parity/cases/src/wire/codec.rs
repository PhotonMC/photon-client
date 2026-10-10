//! Hex encodings of protocol fields, written independently of both
//! implementations.

use crate::bits::{hex, varint};

pub fn varint_hex(value: i64) -> String {
    hex(&varint(value))
}

/// `value` as UTF-16BE hex.
pub fn utf16_hex(value: &str) -> String {
    value.encode_utf16().map(|unit| format!("{unit:04x}")).collect()
}

/// A protocol string: VarInt length, then UTF-8.
pub fn string_frame(value: &str) -> String {
    format!("{}{}", varint_hex(value.len() as i64), hex(value.as_bytes()))
}

/// `value` as a big-endian two's complement integer of `width` bytes.
pub fn fixed_hex(value: i64, width: usize) -> String {
    hex(&value.to_be_bytes()[8 - width..])
}

pub fn flag(value: bool) -> &'static str {
    if value { "01" } else { "00" }
}

pub fn feature_frame(values: &[&str]) -> String {
    let strings: String = values.iter().map(|value| string_frame(value)).collect();
    format!("{}{strings}", varint_hex(values.len() as i64))
}

/// One tag registry of `(tag, values)` entries.
pub fn tags_frame(registry: &str, entries: &[(&str, &[i64])]) -> String {
    let mut out = format!("{}{}{}", varint_hex(1), string_frame(registry), varint_hex(entries.len() as i64));
    for (tag, values) in entries {
        out.push_str(&string_frame(tag));
        out.push_str(&varint_hex(values.len() as i64));
        out.extend(values.iter().map(|value| varint_hex(*value)));
    }
    out
}
