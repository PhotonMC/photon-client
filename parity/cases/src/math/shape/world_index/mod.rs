//! The `shape/world_index` cases.

mod indexed_entities;
mod pushable_contexts;
mod pushable_tags;
mod section_index_updates;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [indexed_entities::cases(), pushable_contexts::cases(), pushable_tags::cases(), section_index_updates::cases()]
        .concat()
}
