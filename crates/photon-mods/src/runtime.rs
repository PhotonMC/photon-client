//! The mods of one connection: which are loaded, what each may do, and what they ask for each
//! tick. Fetching and the connection are the app's business; this holds no network.

use std::sync::Arc;
use std::time::{Duration, Instant};

use photon_api::Owner;

use crate::{Caps, Compiled, Effect, Host, HostError, Mod, WorldView};

/// What a tick of one mod may take before it is stopped.
const ALLOWANCE: Duration = Duration::from_millis(20);

/// What all mods together may take of one client tick, before the rest wait for the next. Each
/// mod is also held to [`ALLOWANCE`], so one mod can overrun this by up to that much.
pub const TICK_BUDGET: Duration = Duration::from_millis(4);

/// Capabilities a server may ask for that this client grants without asking the player: they
/// touch nothing outside the game's own screen and sound. Anything else (files, network, the
/// player's data) is refused until the client can ask the player.
pub const GRANTED: &[&str] = &["events.tick", "hud.text", "sound.play", "game.time"];

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
    /// The text it drew in the last tick it ran.
    hud: Vec<HudLine>,
    module: Mod,
}

#[derive(Default)]
pub struct Runtime {
    host: Option<Arc<Host>>,
    loaded: Vec<Loaded>,
    /// Index of the mod that runs first in the next tick (round-robin under the budget).
    cursor: usize,
    /// What mods that asked for `game.time` read in the next tick.
    world: WorldView,
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
            game_time: asked.iter().any(|cap| cap == "game.time"),
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

    /// The host every mod of this runtime runs in, made at the first call. The app hands a clone
    /// to its fetch thread, which compiles with it; the engine is shared, so a compile there
    /// can be instantiated here.
    pub fn host(&mut self) -> Result<Arc<Host>, HostError> {
        if let Some(host) = &self.host {
            return Ok(Arc::clone(host));
        }
        let host = Arc::new(Host::new()?);
        self.host = Some(Arc::clone(&host));
        Ok(host)
    }

    /// Compiles and loads one mod with the capabilities it asked for, if all are granted.
    pub fn load(
        &mut self,
        (id, hash): (&str, &str),
        asked: &[String],
        bytes: &[u8],
    ) -> Result<(), HostError> {
        grant(asked)
            .ok_or_else(|| HostError::Load(format!("{id} asks for more than it may have")))?;
        let compiled = self.host()?.compile(bytes)?;
        self.load_compiled((id, hash), asked, &compiled)
    }

    /// Instantiates a mod that was compiled on another thread. Any mod of the same id goes
    /// first, so its slot is free for the new one.
    pub fn load_compiled(
        &mut self,
        (id, hash): (&str, &str),
        asked: &[String],
        compiled: &Compiled,
    ) -> Result<(), HostError> {
        let caps = grant(asked)
            .ok_or_else(|| HostError::Load(format!("{id} asks for more than it may have")))?;
        let owner = Owner::dynamic(id).ok_or_else(|| HostError::Load("too many mods".into()))?;
        self.loaded.retain(|m| m.id != id);
        let module = self
            .host()?
            .instantiate(owner, compiled, (&[], caps), ALLOWANCE)?;
        self.loaded.push(Loaded {
            id: id.to_owned(),
            hash: hash.to_owned(),
            ticks: caps.tick,
            hud: Vec::new(),
            module,
        });
        Ok(())
    }

    /// One client tick of every mod that asked for `events.tick`, within [`TICK_BUDGET`]. A mod
    /// that fails is unloaded. The text lands in [`Runtime::hud`]; the sounds are returned for
    /// the app to play.
    pub fn tick(&mut self) -> Vec<Effect> {
        self.tick_within(TICK_BUDGET).sounds
    }

    /// [`Runtime::tick`] with an explicit total budget. Mods run in turn from where the last
    /// tick stopped; once the budget is spent the rest wait and are first next tick, so no mod
    /// starves. At least one mod runs per tick, and a mod that overruns its own allowance is
    /// stopped and unloaded as ever. A mod that waited keeps the text it last drew.
    pub fn tick_within(&mut self, budget: Duration) -> TickReport {
        let started = Instant::now();
        let count = self.loaded.len();
        let mut report = TickReport::default();
        let mut failed = Vec::new();
        let mut next = 0;
        for step in 0..count {
            let index = (self.cursor + step) % count;
            if !self.loaded[index].ticks {
                continue;
            }
            if report.ran + failed.len() > 0 && started.elapsed() >= budget {
                report.deferred = (step..count)
                    .filter(|s| self.loaded[(self.cursor + s) % count].ticks)
                    .count();
                next = index;
                break;
            }
            if run_one(&mut self.loaded[index], self.world, &mut report.sounds) {
                report.ran += 1;
            } else {
                failed.push(index);
            }
            next = (index + 1) % count;
        }
        // Dropped mods shift the ones behind them, and the cursor with them.
        let before = failed.iter().filter(|&&index| index < next).count();
        let mut position = 0;
        self.loaded.retain(|_| {
            position += 1;
            !failed.contains(&(position - 1))
        });
        self.cursor = next.saturating_sub(before);
        self.hud = self.loaded.iter().flat_map(|m| m.hud.clone()).collect();
        report
    }

    /// The level as the next tick shows it to mods.
    pub fn set_world(&mut self, world: WorldView) {
        self.world = world;
    }

    /// The id of the mod that runs first in the next tick.
    pub fn next_up(&self) -> Option<&str> {
        self.loaded.get(self.cursor).map(|m| m.id.as_str())
    }
}

/// What one tick did.
#[derive(Debug, Default)]
pub struct TickReport {
    pub sounds: Vec<Effect>,
    /// Mods that ran to the end of their tick.
    pub ran: usize,
    /// Mods that had to wait for the next tick because the budget was spent.
    pub deferred: usize,
}

/// Runs one mod's tick; false if it failed and must be unloaded.
fn run_one(m: &mut Loaded, world: WorldView, sounds: &mut Vec<Effect>) -> bool {
    m.module.set_world(world);
    if let Err(error) = m.module.tick() {
        log::warn!("mod {} stopped: {error}", m.id);
        return false;
    }
    let (effects, dropped) = m.module.take_effects();
    if dropped > 0 {
        log::warn!("mod {} left {dropped} effects over the limit", m.id);
    }
    m.hud.clear();
    for effect in effects {
        match effect {
            Effect::HudText { x, y, colour, text } => m.hud.push(HudLine { x, y, colour, text }),
            sound @ Effect::Sound { .. } => sounds.push(sound),
        }
    }
    true
}

#[cfg(test)]
mod tests;
