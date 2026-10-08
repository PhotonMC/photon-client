//! The key mappings (`KeyMapping`) and the names of keys (`InputConstants`).

use crate::generated::key_mappings::{KEY_CATEGORIES, KEY_MAPPINGS, NAMED_KEYS};

/// One key mapping of `Options.keyMappings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyMappingRow {
    /// `key.forward`, as `options.txt` names it after `key_`.
    pub name: &'static str,
    /// `minecraft:movement`.
    pub category: &'static str,
    /// The default key's name (`key.keyboard.w`).
    pub default_key: &'static str,
    /// Where it sorts among its category's (`KeyMapping.order`).
    pub order: i32,
}

/// What a key is: a keyboard key by its GLFW code, a scancode, or a mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputKey {
    Keysym(i32),
    Scancode(i32),
    Mouse(i32),
}

/// Every key mapping, in `Options.keyMappings` order.
pub fn mappings() -> impl Iterator<Item = KeyMappingRow> {
    KEY_MAPPINGS
        .iter()
        .map(|&(name, category, default_key, order)| KeyMappingRow {
            name,
            category,
            default_key,
            order,
        })
}

/// `KeyMapping.Category.SORT_ORDER`.
pub fn categories() -> &'static [&'static str] {
    &KEY_CATEGORIES
}

/// The type names and prefixes of `InputConstants.Type`.
const TYPES: [(&str, &str); 3] = [
    ("KEYSYM", "key.keyboard"),
    ("SCANCODE", "scancode"),
    ("MOUSE", "key.mouse"),
];

fn type_of(key: InputKey) -> (&'static str, &'static str, i32) {
    match key {
        InputKey::Keysym(code) => (TYPES[0].0, TYPES[0].1, code),
        InputKey::Scancode(code) => (TYPES[1].0, TYPES[1].1, code),
        InputKey::Mouse(code) => (TYPES[2].0, TYPES[2].1, code),
    }
}

/// `InputConstants.Key.getName`: a named key's name, or the type's prefix and the number (a mouse
/// button counted from one).
pub fn key_name(key: InputKey) -> String {
    let (kind, prefix, code) = type_of(key);
    if let Some((_, _, name)) = NAMED_KEYS
        .iter()
        .find(|(named, number, _)| *named == kind && *number == code)
    {
        return (*name).to_owned();
    }
    let shown = if matches!(key, InputKey::Mouse(_)) {
        code + 1
    } else {
        code
    };
    format!("{prefix}.{shown}")
}

/// `InputConstants.getKey(String)`: the key a name names; none for a name it cannot read.
pub fn key_of(name: &str) -> Option<InputKey> {
    if let Some(&(kind, code, _)) = NAMED_KEYS.iter().find(|(_, _, named)| *named == name) {
        return Some(match kind {
            "KEYSYM" => InputKey::Keysym(code),
            "SCANCODE" => InputKey::Scancode(code),
            _ => InputKey::Mouse(code),
        });
    }
    for (kind, prefix) in TYPES {
        if let Some(rest) = name.strip_prefix(prefix) {
            let number: i32 = rest.strip_prefix('.')?.parse().ok()?;
            return Some(match kind {
                "KEYSYM" => InputKey::Keysym(number),
                "SCANCODE" => InputKey::Scancode(number),
                _ => InputKey::Mouse(number - 1),
            });
        }
    }
    None
}

/// Every named key: its type, code and name.
pub fn named_keys() -> &'static [(&'static str, i32, &'static str)] {
    &NAMED_KEYS
}
