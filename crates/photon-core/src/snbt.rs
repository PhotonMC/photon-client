//! Tags written as text: `Tag.toString` (`StringTagVisitor`) and the plain text of
//! `NbtUtils.toPrettyComponent` (`TextComponentTagVisitor` with no indentation).

use crate::identifier::java_string_hash;
use crate::java_float::{java_double, java_float};
use crate::nbt::{Field, FieldValue};

/// `StringTag.quoteAndEscape`: the quote that needs no escaping is chosen, double quotes unless
/// the text holds one.
pub fn quote_and_escape(input: &str) -> String {
    let mut body = String::new();
    let mut quote = None;
    for c in input.chars() {
        if c == '"' || c == '\'' {
            let chosen = *quote.get_or_insert(if c == '"' { '\'' } else { '"' });
            if chosen == c {
                body.push('\\');
            }
            body.push(c);
        } else {
            push_escaped(c, &mut body);
        }
    }
    let quote = quote.unwrap_or('"');
    format!("{quote}{body}{quote}")
}

fn push_escaped(c: char, body: &mut String) {
    match c {
        '\\' => body.push_str("\\\\"),
        '\u{8}' => body.push_str("\\b"),
        '\t' => body.push_str("\\t"),
        '\n' => body.push_str("\\n"),
        '\u{c}' => body.push_str("\\f"),
        '\r' => body.push_str("\\r"),
        c if c < ' ' => body.push_str(&format!("\\x{:02x}", c as u32)),
        c => body.push(c),
    }
}

/// The text of a tag that holds a single value, or `None` for a container.
fn scalar(value: &FieldValue) -> Option<String> {
    Some(match value {
        FieldValue::Byte(v) => format!("{v}b"),
        FieldValue::Short(v) => format!("{v}s"),
        FieldValue::Int(v) => v.to_string(),
        FieldValue::Long(v) => format!("{v}L"),
        FieldValue::Float(v) => format!("{}f", java_float(*v)),
        FieldValue::Double(v) => format!("{}d", java_double(*v)),
        FieldValue::Str(v) => quote_and_escape(v),
        _ => return None,
    })
}

/// `String.compareTo`: by UTF-16 code unit.
fn utf16_order(a: &str, b: &str) -> std::cmp::Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

fn is_unquoted_key(key: &str) -> bool {
    let mut chars = key.chars();
    let first = |c: char| c.is_ascii_alphabetic() || c == '.' || c == '_';
    let rest = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '+' | '-');
    chars.next().is_some_and(first)
        && chars.all(rest)
        && !key.eq_ignore_ascii_case("true")
        && !key.eq_ignore_ascii_case("false")
}

/// `Tag.toString`: keys sorted, numbers with their type suffix.
pub fn compact(value: &FieldValue) -> String {
    let mut out = String::new();
    write_compact(value, &mut out);
    out
}

fn join<T>(items: &[T], out: &mut String, mut each: impl FnMut(&T, &mut String)) {
    for (index, item) in items.iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        each(item, out);
    }
}

fn write_compact(value: &FieldValue, out: &mut String) {
    if let Some(text) = scalar(value) {
        out.push_str(&text);
        return;
    }
    match value {
        FieldValue::ByteArray(items) => {
            out.push_str("[B;");
            join(items, out, |v, out| out.push_str(&format!("{v}B")));
            out.push(']');
        }
        FieldValue::IntArray(items) => {
            out.push_str("[I;");
            join(items, out, |v, out| out.push_str(&v.to_string()));
            out.push(']');
        }
        FieldValue::LongArray(items) => {
            out.push_str("[L;");
            join(items, out, |v, out| out.push_str(&format!("{v}L")));
            out.push(']');
        }
        FieldValue::List(items) => {
            out.push('[');
            join(items, out, write_compact);
            out.push(']');
        }
        FieldValue::Compound(fields) => write_compact_compound(fields, out),
        _ => {}
    }
}

fn write_compact_compound(fields: &[Field], out: &mut String) {
    let mut sorted: Vec<&Field> = fields.iter().collect();
    sorted.sort_by(|a, b| utf16_order(&a.key, &b.key));
    out.push('{');
    join(&sorted, out, |field, out| {
        if is_unquoted_key(&field.key) {
            out.push_str(&field.key);
        } else {
            out.push_str(&quote_and_escape(&field.key));
        }
        out.push(':');
        write_compact(&field.value, out);
    });
    out.push('}');
}

/// A compound's fields in the order a `HashMap` made by reading them in this order lists them
/// (`CompoundTag` keeps its entries in one): by bucket of the spread hash, in the order read
/// within a bucket. The table starts at 16 buckets and doubles when more than three quarters
/// are full.
pub fn hash_map_order(fields: &[Field]) -> Vec<&Field> {
    let mut buckets = 16_usize;
    while fields.len() > buckets / 4 * 3 {
        buckets *= 2;
    }
    let bucket = |key: &str| {
        let hash = java_string_hash(key) as u32;
        ((hash ^ (hash >> 16)) as usize) & (buckets - 1)
    };
    let mut ordered: Vec<&Field> = fields.iter().collect();
    ordered.sort_by_key(|field| bucket(&field.key));
    ordered
}

/// `toPrettyComponent` of a compound the entries `removed` were taken out of after it was read.
pub fn pretty_compound_without(fields: &[Field], removed: &[&str]) -> String {
    let mut out = String::new();
    write_compound(fields, removed, 0, &mut out);
    out
}

const MAX_DEPTH: usize = 64;
const MAX_LENGTH: usize = 128;

/// `NbtUtils.toPrettyComponent(tag).getString()`: the text with no styling, keys in the order of
/// the game's hash map.
pub fn pretty(value: &FieldValue) -> String {
    let mut out = String::new();
    write_pretty(value, 0, &mut out);
    out
}

fn is_simple(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '+' | '-'))
}

fn write_numbers<T>(prefix: &str, items: &[T], out: &mut String, show: impl Fn(&T) -> String) {
    out.push('[');
    out.push_str(prefix);
    out.push(';');
    for (index, item) in items.iter().enumerate().take(MAX_LENGTH) {
        out.push(' ');
        out.push_str(&show(item));
        if index != items.len() - 1 {
            out.push(',');
        }
    }
    if items.len() > MAX_LENGTH {
        out.push_str("<...>");
    }
    out.push(']');
}

fn write_pretty(value: &FieldValue, depth: usize, out: &mut String) {
    if let Some(text) = scalar(value) {
        out.push_str(&text);
        return;
    }
    match value {
        FieldValue::ByteArray(items) => write_numbers("B", items, out, |v| format!("{v}b")),
        FieldValue::IntArray(items) => write_numbers("I", items, out, ToString::to_string),
        FieldValue::LongArray(items) => write_numbers("L", items, out, |v| format!("{v}L")),
        FieldValue::List(items) => write_list(items, depth, out),
        FieldValue::Compound(fields) => write_compound(fields, &[], depth, out),
        _ => {}
    }
}

fn write_list(items: &[FieldValue], depth: usize, out: &mut String) {
    if items.is_empty() {
        out.push_str("[]");
    } else if depth >= MAX_DEPTH {
        out.push_str("[<...>]");
    } else {
        // Wrapped or not, with no indentation the elements are told apart by a comma and a
        // space. A list that wraps has fewer than eight elements, so no cap applies.
        out.push('[');
        for (index, item) in items.iter().enumerate() {
            if index != 0 {
                out.push_str(", ");
            }
            write_pretty(item, depth + 1, out);
        }
        out.push(']');
    }
}

fn write_compound(fields: &[Field], removed: &[&str], depth: usize, out: &mut String) {
    // Entries taken out of a map leave its table as large as it was.
    let ordered: Vec<&Field> = hash_map_order(fields)
        .into_iter()
        .filter(|field| !removed.contains(&field.key.as_str()))
        .collect();
    if ordered.is_empty() {
        out.push_str("{}");
    } else if depth >= MAX_DEPTH {
        out.push_str("{<...>}");
    } else {
        out.push('{');
        for (index, field) in ordered.iter().enumerate() {
            if is_simple(&field.key) {
                out.push_str(&field.key);
            } else {
                out.push_str(&quote_and_escape(&field.key));
            }
            out.push_str(": ");
            write_pretty(&field.value, depth + 1, out);
            if index + 1 != ordered.len() {
                out.push_str(", ");
            }
        }
        out.push('}');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(key: &str, value: i32) -> Field {
        Field {
            key: key.to_owned(),
            value: FieldValue::Int(value),
        }
    }

    #[test]
    fn quoting_picks_the_delimiter_from_the_first_quote() {
        assert_eq!(quote_and_escape("plain"), "\"plain\"");
        assert_eq!(quote_and_escape("a\"b"), "'a\"b'");
        // The first quote met decides the delimiter; a later one of the same kind is escaped.
        assert_eq!(quote_and_escape("a\"b'c"), "'a\"b\\'c'");
    }

    #[test]
    fn hash_order_follows_buckets_then_insertion() {
        // "a" and "b" hash to distinct low buckets; the order is by bucket, not insertion.
        let fields = [field("b", 1), field("a", 2)];
        let keys: Vec<&str> = hash_map_order(&fields)
            .iter()
            .map(|f| f.key.as_str())
            .collect();
        assert_eq!(keys, ["a", "b"]);
    }
}
