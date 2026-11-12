use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use photon_api::{Owner, PlayerView, WorldView};

use crate::data::ModData;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, InstanceAllocationStrategy, PoolingAllocationConfig, Store};

/// How often the clock thread advances the epoch; an allowance is counted in these.
const EPOCH_STEP: Duration = Duration::from_millis(1);
/// Most mods the host keeps at once (ADR 0006 plans for 300).
const MOD_SLOTS: u32 = 400;
/// Each mod's linear memory may grow to this many bytes.
const MEMORY_BUDGET: usize = 16 << 20;
/// Table elements one mod may hold (a mod's function table is a few thousand at most).
const TABLE_ELEMENTS: usize = 20_000;
/// Most memories, tables and core instances one component may make.
const PER_COMPONENT: u32 = 4;
const CORE_INSTANCES: u32 = 16;
/// Core instances the pool holds per mod slot: a component is usually a few (main, shims).
const CORE_PER_MOD: u32 = 4;
/// Largest an instance's own bookkeeping may be, in bytes. It grows with the functions a module
/// takes the address of (32 bytes each), so a large real module needs room; 1 MiB is wasmtime's
/// default, stated here so it cannot change under us.
const CORE_INSTANCE_SIZE: usize = 1 << 20;
const COMPONENT_INSTANCE_SIZE: usize = 1 << 20;
/// The native stack a mod's calls may use, so deep recursion traps instead of eating the game's.
const WASM_STACK: usize = 512 << 10;

#[derive(Debug)]
pub enum HostError {
    /// The engine could not be set up.
    Engine(String),
    /// The component is not valid, or asks for something it was not granted.
    Load(String),
    /// A call into the mod failed or was stopped for running too long.
    Call(String),
}

impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Engine(why) => write!(f, "mod engine: {why}"),
            Self::Load(why) => write!(f, "mod load: {why}"),
            Self::Call(why) => write!(f, "mod call: {why}"),
        }
    }
}

impl std::error::Error for HostError {}

/// What a mod asked the client to do during a tick. The client applies these on its own thread;
/// the mod never touches the screen or the sound engine.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Text at a place on the screen, in GUI units, with an `0xRRGGBB` colour.
    HudText {
        x: i32,
        y: i32,
        colour: u32,
        text: String,
    },
    /// A sound by its identifier.
    Sound {
        name: String,
        volume: f32,
        pitch: f32,
    },
}

/// Most effects one tick may leave; the rest are counted and dropped.
pub const MAX_EFFECTS: usize = 64;
/// Longest text or sound name, in bytes.
pub const MAX_TEXT: usize = 256;

/// The imports a mod may be given beyond its tick, one per capability of ADR 0006.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Caps {
    /// `events.tick`: the mod's `tick` is called each client tick. Without it the host never
    /// calls the mod after it is loaded, and its `tick` export is not needed. With it the export
    /// is required, and a component without one is refused at load.
    pub tick: bool,
    /// `hud.text`: draw text on the screen.
    pub hud_text: bool,
    /// `sound.play`: play a sound.
    pub sound: bool,
    /// `game.time`: read the level's tick count and day time.
    pub game_time: bool,
    /// `player.state`: read the local player's place, health and look.
    pub player: bool,
    /// `fs.mod_data`: a private key-value store. The player must have allowed it, and the mod
    /// must be given its [`ModData`] when it is instantiated.
    pub mod_data: bool,
}

/// What a mod's store holds: the effects of the tick that is running.
#[derive(Default)]
pub struct ModState {
    pub(crate) data: Option<ModData>,
    pub(crate) world: WorldView,
    pub(crate) player: PlayerView,
    effects: Vec<Effect>,
    dropped: u32,
}

impl ModState {
    pub(crate) fn push(&mut self, effect: Effect) {
        if self.effects.len() < MAX_EFFECTS {
            self.effects.push(effect);
        } else {
            self.dropped += 1;
        }
    }
}

/// The pooling allocator's limits: `slots` mods at once, each within fixed bounds. A load past the
/// last slot, or a mod that asks for more than a bound, fails with an error, nothing else.
fn pool_limits(slots: u32) -> PoolingAllocationConfig {
    let mut pool = PoolingAllocationConfig::default();
    pool.total_component_instances(slots);
    pool.total_core_instances(slots.saturating_mul(CORE_PER_MOD));
    pool.total_memories(slots);
    pool.total_tables(slots.saturating_mul(2));
    pool.max_memory_size(MEMORY_BUDGET);
    pool.table_elements(TABLE_ELEMENTS);
    pool.max_memories_per_component(PER_COMPONENT);
    pool.max_tables_per_component(PER_COMPONENT);
    pool.max_memories_per_module(PER_COMPONENT);
    pool.max_tables_per_module(PER_COMPONENT);
    pool.max_core_instances_per_component(CORE_INSTANCES);
    pool.max_core_instance_size(CORE_INSTANCE_SIZE);
    pool.max_component_instance_size(COMPONENT_INSTANCE_SIZE);
    pool
}

/// The engine every mod runs in, and the thread that keeps its clock.
pub struct Host {
    engine: Engine,
    stop: Arc<AtomicBool>,
    /// Started by the first load, so a client without mods never wakes for it.
    clock: Mutex<Option<JoinHandle<()>>>,
}

impl Host {
    pub fn new() -> Result<Self, HostError> {
        Self::with_slots(MOD_SLOTS)
    }

    /// A host that keeps at most `slots` mods at once.
    pub(crate) fn with_slots(slots: u32) -> Result<Self, HostError> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.epoch_interruption(true);
        // Without these the pool reserves several GiB of address space per slot (about 1.6 TB
        // for 400 slots, measured); with them 400 slots reserve about 6.6 GB.
        config.memory_reservation(MEMORY_BUDGET as u64);
        config.memory_guard_size(64 << 10);
        config.max_wasm_stack(WASM_STACK);
        config.allocation_strategy(InstanceAllocationStrategy::Pooling(pool_limits(slots)));
        let engine = Engine::new(&config).map_err(|e| HostError::Engine(e.to_string()))?;
        Ok(Self {
            engine,
            stop: Arc::new(AtomicBool::new(false)),
            clock: Mutex::new(None),
        })
    }

    fn ensure_clock(&self) -> Result<(), HostError> {
        let mut clock = self.clock.lock().unwrap_or_else(|e| e.into_inner());
        if clock.is_some() {
            return Ok(());
        }
        let (engine, stop) = (self.engine.clone(), Arc::clone(&self.stop));
        let handle = std::thread::Builder::new()
            .name("Mod Clock".to_owned())
            .spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    std::thread::sleep(EPOCH_STEP);
                    engine.increment_epoch();
                }
            })
            .map_err(|e| HostError::Engine(e.to_string()))?;
        *clock = Some(handle);
        Ok(())
    }

    /// Loads `wasm` (binary or text) as `owner`. `grants` names each import the mod may call, as
    /// `(import name, answer)`; an import the mod asks for beyond those fails here.
    pub fn load(
        &self,
        owner: Owner,
        wasm: &[u8],
        grants: &[(&str, u32)],
        allowance: Duration,
    ) -> Result<Mod, HostError> {
        self.load_with(owner, wasm, (grants, Caps::default()), allowance)
    }

    /// [`Host::load`] with the capabilities of [`Caps`] granted as well.
    pub fn load_with(
        &self,
        owner: Owner,
        wasm: &[u8],
        grants: (&[(&str, u32)], Caps),
        allowance: Duration,
    ) -> Result<Mod, HostError> {
        let compiled = self.compile(wasm)?;
        self.instantiate(owner, &compiled, grants, allowance)
    }

    /// Compiles `wasm` (binary or text). This is the slow part of a load, and needs no mod slot
    /// and no clock, so the app runs it on a thread of its own and instantiates on the game's.
    pub fn compile(&self, wasm: &[u8]) -> Result<Compiled, HostError> {
        Component::new(&self.engine, wasm)
            .map(|component| Compiled { component })
            .map_err(|e| HostError::Load(e.to_string()))
    }

    /// Instantiates a compiled mod as `owner`, linking only the imports it was granted. Quick,
    /// but it takes a slot of the pool and runs the mod's start function.
    pub fn instantiate(
        &self,
        owner: Owner,
        compiled: &Compiled,
        grants: (&[(&str, u32)], Caps),
        allowance: Duration,
    ) -> Result<Mod, HostError> {
        self.instantiate_with_data(owner, compiled, grants, None, allowance)
    }

    /// [`Host::instantiate`] for a mod that may keep data: `data` is what its `fs.mod_data`
    /// imports read and write, which the caller saves.
    pub fn instantiate_with_data(
        &self,
        owner: Owner,
        compiled: &Compiled,
        (grants, caps): (&[(&str, u32)], Caps),
        data: Option<ModData>,
        allowance: Duration,
    ) -> Result<Mod, HostError> {
        self.ensure_clock()?;
        let ticks_allowed = (allowance.as_millis() as u64).max(1);
        let load = |e: wasmtime::Error| HostError::Load(e.to_string());
        let mut linker: Linker<ModState> = Linker::new(&self.engine);
        for &(name, answer) in grants {
            linker
                .root()
                .func_wrap(name, move |_, ()| Ok((answer,)))
                .map_err(load)?;
        }
        crate::link::link(&mut linker, caps).map_err(load)?;
        let mut store = Store::new(
            &self.engine,
            ModState {
                data,
                ..ModState::default()
            },
        );
        // The clock is already running: a start function gets the same allowance as a tick.
        // The deadline is set before the instance exists, never after.
        store.set_epoch_deadline(ticks_allowed);
        let instance = linker
            .instantiate(&mut store, &compiled.component)
            .map_err(load)?;
        // The export is required exactly when the mod asked for `events.tick`; otherwise it is
        // used if it happens to be there and never needed.
        let tick = instance.get_typed_func::<(), ()>(&mut store, "tick");
        let tick = if caps.tick {
            Some(tick.map_err(load)?)
        } else {
            tick.ok()
        };
        Ok(Mod {
            owner,
            store,
            tick,
            ticks_allowed,
        })
    }
}

/// A mod's code, compiled for one [`Host`] and not yet running. It can cross threads, and one
/// compile can be instantiated again.
#[derive(Clone)]
pub struct Compiled {
    component: Component,
}

impl Drop for Host {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        let clock = self
            .clock
            .get_mut()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        if let Some(clock) = clock {
            let _ = clock.join();
        }
    }
}

/// One loaded mod: its own store, so nothing it holds outlives it.
pub struct Mod {
    owner: Owner,
    store: Store<ModState>,
    tick: Option<wasmtime::component::TypedFunc<(), ()>>,
    ticks_allowed: u64,
}

impl Mod {
    pub fn owner(&self) -> Owner {
        self.owner
    }

    /// What the last tick asked for, taken: the next tick starts empty. The number is how many
    /// effects went over [`MAX_EFFECTS`] and were dropped.
    pub fn take_effects(&mut self) -> (Vec<Effect>, u32) {
        let state = self.store.data_mut();
        (
            std::mem::take(&mut state.effects),
            std::mem::take(&mut state.dropped),
        )
    }

    /// What the mod reads of the level during its next tick.
    pub fn set_world(&mut self, world: WorldView) {
        self.store.data_mut().world = world;
    }

    /// What the mod reads of the local player during its next tick.
    pub fn set_player(&mut self, player: PlayerView) {
        self.store.data_mut().player = player;
    }

    /// Runs the mod's `tick`; a call that outlasts its allowance is stopped. A mod without the
    /// export (it did not ask for `events.tick`) does nothing.
    pub fn tick(&mut self) -> Result<(), HostError> {
        let Some(tick) = self.tick else {
            return Ok(());
        };
        if let Some(data) = &self.store.data().data {
            data.new_tick();
        }
        self.store.set_epoch_deadline(self.ticks_allowed);
        tick.call(&mut self.store, ())
            .map_err(|e| HostError::Call(trap_name(&e)))
    }
}

/// `interrupt` for a call the clock stopped, `stack overflow` for runaway recursion, else the
/// error's own words.
fn trap_name(error: &wasmtime::Error) -> String {
    match error.downcast_ref::<wasmtime::Trap>() {
        Some(wasmtime::Trap::Interrupt) => "interrupt".to_owned(),
        Some(wasmtime::Trap::StackOverflow) => "stack overflow".to_owned(),
        _ => error.to_string(),
    }
}

#[cfg(test)]
mod tests;
