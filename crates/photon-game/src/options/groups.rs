//! The options that come in groups, dispatched to the group that holds them.

use super::*;

impl Options {
    /// `read` for the main hand and the skin's parts.
    /// The options that come in groups: the skin's and the quality options.
    pub(super) fn read_group(&mut self, key: Key, json: &Json) {
        match key {
            Key::Quality(key) => self.quality.read(key, json),
            Key::Online(key) => self.online.read(key, json),
            Key::Play(key) => self.play.read(key, json),
            _ => self.read_skin(key, json),
        }
    }

    pub(super) fn write_group(&self, key: Key) -> String {
        match key {
            Key::Quality(key) => self.quality.write(key),
            Key::Online(key) => self.online.write(key),
            Key::Play(key) => self.play.write(key),
            Key::ModelPart(bit) => self.is_model_part_enabled(bit).to_string(),
            _ => String::new(),
        }
    }
}
