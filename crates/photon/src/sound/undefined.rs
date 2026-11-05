//! Sound events the server asked for that no sound definition has.

use std::collections::HashSet;

use photon_game::sounds::engine::SoundEngine;

use super::MixerChannels;

/// The events already warned about, so a server that keeps asking for one logs it once.
#[derive(Default)]
pub(super) struct Warned(HashSet<String>);

impl Warned {
    /// Log the first request for an event the definitions lack.
    pub(super) fn check(&mut self, engine: &SoundEngine<MixerChannels>, event: &str) {
        if self.first_unknown(engine.knows_event(event), event) {
            log::warn!("no sound definition for the event {event}");
        }
    }

    /// Whether this is the first request for an event the definitions lack.
    fn first_unknown(&mut self, known: bool, event: &str) -> bool {
        !known && self.0.insert(event.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_event_is_reported_once() {
        let mut warned = Warned::default();
        assert!(warned.first_unknown(false, "pack:cash"));
        assert!(!warned.first_unknown(false, "pack:cash"));
        assert!(warned.first_unknown(false, "pack:other"));
    }

    #[test]
    fn a_known_event_is_never_reported() {
        let mut warned = Warned::default();
        assert!(!warned.first_unknown(true, "minecraft:entity.pig.ambient"));
    }
}
