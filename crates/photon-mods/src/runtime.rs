//! The mods of one connection: which are loaded, what each may do, and what they ask for each
//! tick. Fetching and the connection are the app's business; this holds no network.

use std::time::Duration;

use photon_api::Owner;

use crate::{Caps, Effect, Host, HostError, Mod};

/// What a tick of one mod may take before it is stopped.
const ALLOWANCE: Duration = Duration::from_millis(20);

/// Capabilities a server may ask for that this client grants without asking the player: they
/// touch nothing outside the game's own screen and sound. Anything else (files, network, the
/// player's data) is refused until the client can ask the player.
pub const GRANTED: &[&str] = &["events.tick", "hud.text", "sound.play"];

/// Text a mod asked to have drawn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudLine {
    pub x: i32,
    pub y: i32,
    pub colour: u32,
    pub text: String,
}

struct Loaded {
    id: String,
    hash: String,
    /// Whether the mod asked for `events.tick`; one that did not is never ticked.
    ticks: bool,
    module: Mod,
}

#[derive(Default)]
pub struct Runtime {
    host: Option<Host>,
    loaded: Vec<Loaded>,
    /// The text the mods asked for in the last tick.
    pub hud: Vec<HudLine>,
}

/// The capabilities `asked` names, if every one of them is granted.
pub fn grant(asked: &[String]) -> Option<Caps> {
    asked
        .iter()
        .all(|cap| GRANTED.contains(&cap.as_str()))
        .then(|| Caps {
            tick: asked.iter().any(|cap| cap == "events.tick"),
            hud_text: asked.iter().any(|cap| cap == "hud.text"),
            sound: asked.iter().any(|cap| cap == "sound.play"),
        })
}

impl Runtime {
    /// Unloads every mod that is not in `wanted` (id and hash): a mod the server dropped, or
    /// updated, goes, and its slot is free for the new one.
    pub fn keep_only(&mut self, wanted: &[(&str, &str)]) {
        self.loaded
            .retain(|m| wanted.contains(&(m.id.as_str(), m.hash.as_str())));
    }

    /// Whether the mod with this id and hash is loaded.
    pub fn has(&self, id: &str, hash: &str) -> bool {
        self.loaded.iter().any(|m| m.id == id && m.hash == hash)
    }

    /// The ids of the loaded mods.
    pub fn ids(&self) -> Vec<&str> {
        self.loaded.iter().map(|m| m.id.as_str()).collect()
    }

    /// Loads one mod with the capabilities it asked for, if all are granted.
    pub fn load(
        &mut self,
        (id, hash): (&str, &str),
        asked: &[String],
        bytes: &[u8],
    ) -> Result<(), HostError> {
        let caps = grant(asked)
            .ok_or_else(|| HostError::Load(format!("{id} asks for more than it may have")))?;
        let owner = Owner::dynamic(id).ok_or_else(|| HostError::Load("too many mods".into()))?;
        let host = match self.host.take() {
            Some(host) => host,
            None => Host::new()?,
        };
        let loaded = host.load_with(owner, bytes, (&[], caps), ALLOWANCE);
        self.host = Some(host);
        self.loaded.retain(|m| m.id != id);
        self.loaded.push(Loaded {
            id: id.to_owned(),
            hash: hash.to_owned(),
            ticks: caps.tick,
            module: loaded?,
        });
        Ok(())
    }

    /// One client tick of every mod that asked for `events.tick`. A mod that fails is unloaded. The text lands in
    /// [`Runtime::hud`]; the sounds are returned for the app to play.
    pub fn tick(&mut self) -> Vec<Effect> {
        let mut sounds = Vec::new();
        self.hud.clear();
        let hud = &mut self.hud;
        self.loaded.retain_mut(|m| {
            if !m.ticks {
                return true;
            }
            if let Err(error) = m.module.tick() {
                log::warn!("mod {} stopped: {error}", m.id);
                return false;
            }
            let (effects, dropped) = m.module.take_effects();
            if dropped > 0 {
                log::warn!("mod {} left {dropped} effects over the limit", m.id);
            }
            for effect in effects {
                match effect {
                    Effect::HudText { x, y, colour, text } => {
                        hud.push(HudLine { x, y, colour, text });
                    }
                    sound @ Effect::Sound { .. } => sounds.push(sound),
                }
            }
            true
        });
        sounds
    }
}

#[cfg(test)]
mod tests;
