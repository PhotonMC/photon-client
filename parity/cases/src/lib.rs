//! Parity case definitions: original synthetic inputs for every family.
//!
//! A case only describes an input. Expected values always come from running
//! the Java reference on it, never from this crate or from Rust output.

mod atlas;
mod binary;
mod bits;
mod block;
mod case;
mod json;
mod line;
mod nbt;
mod sound;
mod timer;
mod utf8;
mod window;

use serde_json::Value;

/// The cases of `family` as JSONL lines, or `None` when its generator is
/// not in this crate.
pub fn lines(family: &str) -> Option<Vec<String>> {
    let cases = match family {
        "atlas" => atlas::cases(),
        "binary" => binary::cases(),
        "block" => block::cases(),
        "json" => json::cases(),
        "nbt" => nbt::cases(),
        "sound" => sound::cases(),
        "timer" => timer::cases(),
        "utf8" => utf8::cases(),
        "window" => window::cases(),
        _ => return None,
    };
    Some(
        cases
            .into_iter()
            .map(with_schema)
            .map(|case| line::render(&case))
            .collect(),
    )
}

fn with_schema(mut case: Value) -> Value {
    if let Value::Object(map) = &mut case {
        map.insert("schema_version".to_owned(), Value::from(1));
    }
    case
}
