//! The key each mapping is bound to (`KeyMapping.key`), as `options.txt` keeps it in `key_` lines.

use voxel_data::keys::{InputKey, key_name, key_of, mappings};

use super::Options;

/// The `options.txt` name of a mapping's line.
fn line_name(mapping: &str) -> String {
    format!("key_{mapping}")
}

/// Every mapping bound to its default key.
pub(super) fn default_bindings() -> Vec<InputKey> {
    mappings()
        .map(|mapping| key_of(mapping.default_key).unwrap_or(InputKey::Keysym(-1)))
        .collect()
}

impl Options {
    /// The key a mapping (`key.forward`) is bound to.
    pub fn key_binding(&self, mapping: &str) -> Option<InputKey> {
        mappings()
            .position(|row| row.name == mapping)
            .and_then(|index| self.key_bindings.get(index).copied())
    }

    /// `KeyMapping.setKey`.
    pub fn set_key_binding(&mut self, mapping: &str, key: InputKey) {
        if let Some(index) = mappings().position(|row| row.name == mapping) {
            self.key_bindings[index] = key;
        }
    }

    /// The mappings a key is bound to, in `keyMappings` order.
    pub fn mappings_of(&self, key: InputKey) -> impl Iterator<Item = &'static str> + '_ {
        mappings()
            .zip(&self.key_bindings)
            .filter(move |(_, bound)| **bound == key)
            .map(|(row, _)| row.name)
    }

    /// `processOptions` for the `key_` lines: a name it cannot read leaves the mapping as it was.
    pub(super) fn read_key_bindings(&mut self) {
        for (index, row) in mappings().enumerate() {
            let name = line_name(row.name);
            let value = self.lines.iter().find(|(known, _)| *known == name);
            if let Some(key) = value.and_then(|(_, value)| key_of(value)) {
                self.key_bindings[index] = key;
            }
        }
    }

    /// The value a `key_` line is written with (`KeyMapping.saveString`); none for another line.
    pub(super) fn key_binding_line(&self, line: &str) -> Option<String> {
        let mapping = line.strip_prefix("key_")?;
        self.key_binding(mapping).map(key_name)
    }

    /// The `key_` lines the file did not have, to add.
    pub(super) fn missing_key_lines(&self) -> Vec<(String, String)> {
        mappings()
            .zip(&self.key_bindings)
            .map(|(row, key)| (line_name(row.name), key_name(*key)))
            .filter(|(name, _)| !self.lines.iter().any(|(known, _)| known == name))
            .collect()
    }
}
