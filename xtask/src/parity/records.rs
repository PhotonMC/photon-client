//! Reading oracle and adapter output, and finding the first difference.
//!
//! Comparison is exact in type and value: an integer never equals a float,
//! object keys must match as sets, list order matters.

use serde_json::Value;

use super::cases::Case;

/// Parses JSONL output that must answer `cases` one by one, in order.
pub fn parse(raw: &str, cases: &[&Case]) -> Result<Vec<Value>, String> {
    let lines: Vec<&str> = raw.lines().collect();
    if lines.len() != cases.len() {
        return Err(format!(
            "expected {} records, received {}",
            cases.len(),
            lines.len()
        ));
    }
    lines
        .iter()
        .zip(cases)
        .map(|(line, case)| {
            let record: Value = serde_json::from_str(line)
                .map_err(|error| format!("invalid record for {}: {error}", case.id))?;
            if well_formed(&record, &case.id) {
                Ok(record)
            } else {
                Err(format!(
                    "malformed, reordered or missing record for {}",
                    case.id
                ))
            }
        })
        .collect()
}

fn well_formed(record: &Value, case_id: &str) -> bool {
    let Some(object) = record.as_object() else {
        return false;
    };
    let known = ["schema_version", "case_id", "result", "diagnostics"];
    object.get("schema_version").and_then(Value::as_u64) == Some(1)
        && object.get("case_id").and_then(Value::as_str) == Some(case_id)
        && object.get("result").is_some_and(Value::is_object)
        && object.get("diagnostics").is_none_or(Value::is_object)
        && object.keys().all(|key| known.contains(&key.as_str()))
}

/// The path and values of the first difference, or `None` when equal.
pub fn first_difference(expected: &Value, actual: &Value, path: &str) -> Option<String> {
    match (expected, actual) {
        (Value::Object(left), Value::Object(right)) => object_difference(left, right, path),
        (Value::Array(left), Value::Array(right)) => array_difference(left, right, path),
        (Value::Number(left), Value::Number(right)) if same_number(left, right) => None,
        _ if expected == actual && !matches!(expected, Value::Number(_)) => None,
        _ => Some(format!(
            "{path}: expected {}, actual {}",
            clip(expected),
            clip(actual)
        )),
    }
}

fn object_difference(
    left: &serde_json::Map<String, Value>,
    right: &serde_json::Map<String, Value>,
    path: &str,
) -> Option<String> {
    let mut keys: Vec<&String> = left.keys().collect();
    keys.sort();
    if left.len() != right.len() || keys.iter().any(|key| !right.contains_key(*key)) {
        let mut other: Vec<&String> = right.keys().collect();
        other.sort();
        return Some(format!("{path}.keys: expected {keys:?}, actual {other:?}"));
    }
    keys.iter()
        .find_map(|key| first_difference(&left[*key], &right[*key], &format!("{path}.{key}")))
}

fn array_difference(left: &[Value], right: &[Value], path: &str) -> Option<String> {
    if left.len() != right.len() {
        return Some(format!(
            "{path}.length: expected {}, actual {}",
            left.len(),
            right.len()
        ));
    }
    left.iter()
        .zip(right)
        .enumerate()
        .find_map(|(index, (one, two))| first_difference(one, two, &format!("{path}[{index}]")))
}

/// Java writes `1` and `1.0` differently; so does Rust, and they must agree.
fn same_number(left: &serde_json::Number, right: &serde_json::Number) -> bool {
    left.is_f64() == right.is_f64() && left == right
}

fn clip(value: &Value) -> String {
    let text = value.to_string();
    if text.len() > 300 {
        format!("{}...", &text[..300])
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn integers_and_floats_differ() {
        assert!(first_difference(&json!({"a": 1}), &json!({"a": 1.0}), "result").is_some());
        assert!(first_difference(&json!({"a": [1, 2]}), &json!({"a": [1, 2]}), "result").is_none());
    }

    #[test]
    fn reports_the_first_path() {
        let found = first_difference(
            &json!({"a": {"b": [1, 2]}}),
            &json!({"a": {"b": [1, 3]}}),
            "result",
        );
        assert_eq!(
            found.as_deref(),
            Some("result.a.b[1]: expected 2, actual 3")
        );
        let keys = first_difference(&json!({"a": 1}), &json!({"b": 1}), "result");
        assert!(keys.is_some_and(|text| text.starts_with("result.keys")));
    }
}
