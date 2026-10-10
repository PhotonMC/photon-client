//! Parity case definitions: original synthetic inputs for every family.
//!
//! A case only describes an input. Expected values always come from running
//! the Java reference on it, never from this crate or from Rust output.

mod atlas;
mod binary;
mod bits;
mod block;
mod case;
mod image;
mod json;
mod line;
mod model;
mod nbt;
mod resources;
mod sound;
mod timer;
mod utf8;
mod window;
mod wire;

use serde_json::Value;

type Generator = fn() -> Vec<Value>;

/// Each ported family and its generator.
const GENERATORS: &[(&str, Generator)] = &[
    ("atlas", atlas::cases),
    ("binary", binary::cases),
    ("block", block::cases),
    ("image", image::cases),
    ("json", json::cases),
    ("model", model::cases),
    ("nbt", nbt::cases),
    ("resources", resources::cases),
    ("sound", sound::cases),
    ("timer", timer::cases),
    ("utf8", utf8::cases),
    ("window", window::cases),
    ("wire", wire::cases),
];

/// The cases of `family` as JSONL lines, or `None` when its generator is
/// not in this crate.
pub fn lines(family: &str) -> Option<Vec<String>> {
    let (_, generate) = GENERATORS.iter().find(|(name, _)| *name == family)?;
    let lines = generate()
        .into_iter()
        .map(with_schema)
        .map(|case| line::render(&case))
        .collect();
    Some(lines)
}

fn with_schema(mut case: Value) -> Value {
    if let Value::Object(map) = &mut case {
        map.insert("schema_version".to_owned(), Value::from(1));
    }
    case
}
