//! One case as a JSONL line: sorted keys, no spaces, ASCII only.
//!
//! The format is fixed because a capture is stored under a hash of the line;
//! a byte that changes means every case of the family is captured again.
//! Floats are written in their shortest round-trip form, switching to an
//! exponent below 1e-4 and from 1e16 up (`1e-05`, `1.5e+16`).

use std::fmt::Write;

use serde_json::{Map, Number, Value};

/// `value` as one line.
pub fn render(value: &Value) -> String {
    let mut out = String::new();
    write_value(&mut out, value);
    out
}

fn write_value(out: &mut String, value: &Value) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(flag) => out.push_str(if *flag { "true" } else { "false" }),
        Value::Number(number) => write_number(out, number),
        Value::String(text) => write_string(out, text),
        Value::Array(items) => write_array(out, items),
        Value::Object(map) => write_object(out, map),
    }
}

fn write_array(out: &mut String, items: &[Value]) {
    out.push('[');
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        write_value(out, item);
    }
    out.push(']');
}

fn write_object(out: &mut String, map: &Map<String, Value>) {
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    out.push('{');
    for (index, key) in keys.into_iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        write_string(out, key);
        out.push(':');
        write_value(out, &map[key]);
    }
    out.push('}');
}

fn write_number(out: &mut String, number: &Number) {
    match number.as_f64() {
        Some(float) if number.is_f64() => out.push_str(&float_text(float)),
        _ => out.push_str(&number.to_string()),
    }
}

/// The shortest text that reads back as `value`.
pub fn float_text(value: f64) -> String {
    assert!(value.is_finite(), "case values must be finite: {value}");
    let scientific = format!("{:e}", value.abs());
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let exponent: i32 = exponent.parse().unwrap_or(0);
    let digits = mantissa.replace('.', "");
    let sign = if value.is_sign_negative() { "-" } else { "" };
    if (-4..16).contains(&exponent) {
        return format!("{sign}{}", fixed(&digits, exponent));
    }
    let exponent_sign = if exponent < 0 { '-' } else { '+' };
    format!("{sign}{mantissa}e{exponent_sign}{:02}", exponent.abs())
}

fn fixed(digits: &str, exponent: i32) -> String {
    if exponent < 0 {
        let zeros = "0".repeat(exponent.unsigned_abs() as usize - 1);
        return format!("0.{zeros}{digits}");
    }
    let point = exponent as usize + 1;
    if digits.len() <= point {
        return format!("{digits}{}.0", "0".repeat(point - digits.len()));
    }
    format!("{}.{}", &digits[..point], &digits[point..])
}

/// The characters with a short escape.
const SHORT_ESCAPES: &[(char, &str)] = &[
    ('"', "\\\""),
    ('\\', "\\\\"),
    ('\n', "\\n"),
    ('\r', "\\r"),
    ('\t', "\\t"),
    ('\u{8}', "\\b"),
    ('\u{c}', "\\f"),
];

fn write_string(out: &mut String, text: &str) {
    out.push('"');
    for ch in text.chars() {
        if let Some((_, escape)) = SHORT_ESCAPES.iter().find(|(short, _)| *short == ch) {
            out.push_str(escape);
        } else if (' '..='~').contains(&ch) {
            out.push(ch);
        } else {
            let mut units = [0; 2];
            for unit in ch.encode_utf16(&mut units) {
                let _ = write!(out, "\\u{unit:04x}");
            }
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floats_switch_to_exponents_at_the_edges() {
        let cases = [
            (0.0, "0.0"),
            (-0.0, "-0.0"),
            (1.0, "1.0"),
            (0.1, "0.1"),
            (1e-4, "0.0001"),
            (1e-5, "1e-05"),
            (1.5e-7, "1.5e-07"),
            (123.456, "123.456"),
            (1e15, "1000000000000000.0"),
            (1e16, "1e+16"),
            (1.5e300, "1.5e+300"),
            (1000.0 / 30.0, "33.333333333333336"),
        ];
        for (value, text) in cases {
            assert_eq!(float_text(value), text);
        }
    }

    #[test]
    fn strings_escape_everything_outside_printable_ascii() {
        let value = Value::String("a\"\\\n\u{1}\u{7f}\u{e7}\u{1f600}".to_owned());
        assert_eq!(render(&value), r#""a\"\\\n\u0001\u007f\u00e7\ud83d\ude00""#);
    }
}
