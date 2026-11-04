use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use photon_api::Owner;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, InstanceAllocationStrategy, PoolingAllocationConfig, Store};

/// How often the clock thread advances the epoch; a mod is stopped within a few of these.
const EPOCH_STEP: Duration = Duration::from_millis(1);
/// Most mods the host keeps at once (ADR 0006 plans for 300).
const MOD_SLOTS: u32 = 400;
/// Each mod's linear memory may grow to this many bytes.
const MEMORY_BUDGET: usize = 16 << 20;

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

/// The engine every mod runs in, and the thread that keeps its clock.
pub struct Host {
    engine: Engine,
    stop: Arc<AtomicBool>,
    clock: Option<JoinHandle<()>>,
}

impl Host {
    pub fn new() -> Result<Self, HostError> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.epoch_interruption(true);
        let mut pool = PoolingAllocationConfig::default();
        pool.total_component_instances(MOD_SLOTS);
        pool.total_core_instances(MOD_SLOTS);
        pool.total_memories(MOD_SLOTS);
        pool.total_tables(MOD_SLOTS);
        pool.max_memory_size(MEMORY_BUDGET);
        config.allocation_strategy(InstanceAllocationStrategy::Pooling(pool));
        let engine = Engine::new(&config).map_err(|e| HostError::Engine(e.to_string()))?;
        let stop = Arc::new(AtomicBool::new(false));
        let clock = {
            let (engine, stop) = (engine.clone(), Arc::clone(&stop));
            std::thread::Builder::new()
                .name("Mod Clock".to_owned())
                .spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        std::thread::sleep(EPOCH_STEP);
                        engine.increment_epoch();
                    }
                })
                .map_err(|e| HostError::Engine(e.to_string()))?
        };
        Ok(Self {
            engine,
            stop,
            clock: Some(clock),
        })
    }

    /// Loads `wasm` (binary or text) as `owner`. `grants` names each import the mod may call, as
    /// `(import name, answer)`; an import the mod asks for beyond those fails here.
    pub fn load(
        &self,
        owner: Owner,
        wasm: &[u8],
        grants: &[(&str, u32)],
        ticks_allowed: u64,
    ) -> Result<Mod, HostError> {
        let load = |e: wasmtime::Error| HostError::Load(e.to_string());
        let component = Component::new(&self.engine, wasm).map_err(load)?;
        let mut linker: Linker<()> = Linker::new(&self.engine);
        for &(name, answer) in grants {
            linker
                .root()
                .func_wrap(name, move |_, ()| Ok((answer,)))
                .map_err(load)?;
        }
        let mut store = Store::new(&self.engine, ());
        let instance = linker.instantiate(&mut store, &component).map_err(load)?;
        let tick = instance
            .get_typed_func::<(), ()>(&mut store, "tick")
            .map_err(load)?;
        Ok(Mod {
            owner,
            store,
            tick,
            ticks_allowed,
        })
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(clock) = self.clock.take() {
            let _ = clock.join();
        }
    }
}

/// One loaded mod: its own store, so nothing it holds outlives it.
pub struct Mod {
    owner: Owner,
    store: Store<()>,
    tick: wasmtime::component::TypedFunc<(), ()>,
    ticks_allowed: u64,
}

impl Mod {
    pub fn owner(&self) -> Owner {
        self.owner
    }

    /// Runs the mod's `tick`; a call that outlasts its allowance (in clock steps) is stopped.
    pub fn tick(&mut self) -> Result<(), HostError> {
        self.store.set_epoch_deadline(self.ticks_allowed);
        self.tick
            .call(&mut self.store, ())
            .map_err(|e| HostError::Call(trap_name(&e)))
    }
}

/// `interrupt` for a call the clock stopped, else the error's own words.
fn trap_name(error: &wasmtime::Error) -> String {
    match error.downcast_ref::<wasmtime::Trap>() {
        Some(wasmtime::Trap::Interrupt) => "interrupt".to_owned(),
        _ => error.to_string(),
    }
}

#[cfg(test)]
mod tests;
