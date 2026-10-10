//! The fields every case shares.

use serde_json::{Value, json};

/// A case of `family` running `operation`, with `fields` added.
pub fn case(id: impl Into<String>, family: &str, operation: &str, fields: Value) -> Value {
    let mut case = json!({"case_id": id.into(), "family": family, "operation": operation});
    merge(&mut case, fields);
    case
}

/// Adds the fields of the object `extra` to the object `target`.
pub fn merge(target: &mut Value, extra: Value) {
    if let (Value::Object(target), Value::Object(extra)) = (target, extra) {
        target.extend(extra);
    }
}
