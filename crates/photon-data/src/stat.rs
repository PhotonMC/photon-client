//! The statistics' registries: the stat types and the custom statistics with their ids (their
//! places in the lists, which is what the wire carries) and how each custom value is written.

use crate::tables::registry_names::{CUSTOM_STAT_FORMATTERS, CUSTOM_STATS, STAT_TYPES};

/// A statistic as the wire names it: the stat type's id, then the id of the value in the registry
/// that type counts (`Stat.STREAM_CODEC`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatKey {
    pub stat_type: i32,
    pub value: i32,
}

/// What a stat type counts: the registry its values are in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueRegistry {
    Block,
    Item,
    EntityType,
    Custom,
}

/// How a statistic's number is written (`StatFormatter`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Formatter {
    Default,
    DivideByTen,
    Distance,
    Time,
}

/// The id of the stat type with this name.
pub fn type_id(name: &str) -> Option<i32> {
    STAT_TYPES
        .iter()
        .position(|own| *own == name)
        .map(|id| id as i32)
}

/// The name of the stat type with this id (`minecraft:mined`).
pub fn type_name(id: i32) -> Option<&'static str> {
    entry(&STAT_TYPES, id)
}

/// The registry a stat type's values come from (`Stats`).
pub fn value_registry(type_id: i32) -> Option<ValueRegistry> {
    Some(match type_name(type_id)? {
        "minecraft:mined" => ValueRegistry::Block,
        "minecraft:crafted"
        | "minecraft:used"
        | "minecraft:broken"
        | "minecraft:picked_up"
        | "minecraft:dropped" => ValueRegistry::Item,
        "minecraft:killed" | "minecraft:killed_by" => ValueRegistry::EntityType,
        "minecraft:custom" => ValueRegistry::Custom,
        _ => return None,
    })
}

/// The name of the custom statistic with this id (`minecraft:play_time`).
pub fn custom_name(id: i32) -> Option<&'static str> {
    entry(&CUSTOM_STATS, id)
}

/// The formatter of the custom statistic with this id.
pub fn custom_formatter(id: i32) -> Option<Formatter> {
    Some(match entry(&CUSTOM_STAT_FORMATTERS, id)? {
        "default" => Formatter::Default,
        "divide_by_ten" => Formatter::DivideByTen,
        "distance" => Formatter::Distance,
        "time" => Formatter::Time,
        _ => return None,
    })
}

/// Every stat type's name, in id order.
pub fn type_names() -> &'static [&'static str] {
    &STAT_TYPES
}

/// Every custom statistic's name, in id order.
pub fn custom_names() -> &'static [&'static str] {
    &CUSTOM_STATS
}

/// Every custom statistic's formatter name, in id order.
pub fn custom_formatter_names() -> &'static [&'static str] {
    &CUSTOM_STAT_FORMATTERS
}

fn entry(table: &'static [&'static str], id: i32) -> Option<&'static str> {
    usize::try_from(id)
        .ok()
        .and_then(|index| table.get(index))
        .copied()
}
