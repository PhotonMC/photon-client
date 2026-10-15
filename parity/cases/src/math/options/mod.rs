//! The `options` cases: text events, enum and numeric settings, key mappings and keyboard input.

mod enum_options;
mod key_mappings;
mod keyboard_input;
mod text_events;
mod value_ranges;

use serde_json::Value;

pub fn cases() -> Vec<Value> {
    [text_events::cases(), value_ranges::cases(), enum_options::cases(), key_mappings::cases(), keyboard_input::cases()]
        .concat()
}
