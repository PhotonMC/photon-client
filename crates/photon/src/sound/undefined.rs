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
        if !engine.knows_event(event) && self.0.insert(event.to_owned()) {
            log::warn!("no sound definition for the event {event}");
        }
    }
}
