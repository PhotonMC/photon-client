//! Mods on the client's tick event: a loaded mod is registered under its owner, so unloading it
//! is one `remove_owner`. A mod that fails is unloaded after the tick, not left to fail again.

use std::sync::{Arc, Mutex};

use photon_api::{ClientEvents, Owner, TickEnd};

use crate::Mod;

/// The mods listening to the tick, and the ones that failed since the last look.
#[derive(Default, Clone)]
pub struct TickMods {
    failed: Arc<Mutex<Vec<Owner>>>,
}

impl TickMods {
    /// `loaded` runs at the end of every client tick until it is unloaded.
    pub fn add(&self, events: &mut ClientEvents, mut loaded: Mod) {
        let (owner, failed) = (loaded.owner(), Arc::clone(&self.failed));
        events.tick_end.register(owner, move |_: &TickEnd| {
            if let Err(error) = loaded.tick() {
                log::warn!("mod {} failed: {error}", owner.name());
                failed.lock().unwrap_or_else(|e| e.into_inner()).push(owner);
            }
        });
    }

    /// Unloads `owner`'s handlers, which drops its store and frees its slot.
    pub fn unload(events: &mut ClientEvents, owner: Owner) -> usize {
        events.tick_end.remove_owner(owner)
    }

    /// Unloads every mod that failed in a tick that has run; the owners that were removed.
    pub fn unload_failed(&self, events: &mut ClientEvents) -> Vec<Owner> {
        let failed = std::mem::take(&mut *self.failed.lock().unwrap_or_else(|e| e.into_inner()));
        for owner in &failed {
            Self::unload(events, *owner);
        }
        failed
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::Host;

    const GOOD: Owner = Owner::new("good");
    const BAD: Owner = Owner::new("bad");
    const EMPTY: &str = r#"(component
  (core module $m (func (export "tick")))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;
    const TRAPS: &str = r#"(component
  (core module $m (func (export "tick") unreachable))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

    fn load(host: &Host, owner: Owner, wat: &str) -> Mod {
        host.load(owner, wat.as_bytes(), &[], Duration::from_millis(50))
            .unwrap()
    }

    #[test]
    fn a_mod_that_traps_is_unloaded_after_the_tick_and_its_slot_is_free() {
        let host = Host::with_slots(2).unwrap();
        let (mut events, mods) = (ClientEvents::default(), TickMods::default());
        mods.add(&mut events, load(&host, GOOD, EMPTY));
        mods.add(&mut events, load(&host, BAD, TRAPS));
        assert!(
            host.load(
                Owner::new("x"),
                EMPTY.as_bytes(),
                &[],
                Duration::from_millis(5)
            )
            .is_err()
        );
        events.tick_end.fire(&TickEnd);
        assert_eq!(mods.unload_failed(&mut events), vec![BAD]);
        assert!(mods.unload_failed(&mut events).is_empty());
        // The good mod still runs; the bad one's slot is back.
        events.tick_end.fire(&TickEnd);
        assert!(mods.unload_failed(&mut events).is_empty());
        let _again = load(&host, Owner::new("again"), EMPTY);
    }

    #[test]
    fn unloading_by_owner_drops_the_mod() {
        let host = Host::with_slots(1).unwrap();
        let (mut events, mods) = (ClientEvents::default(), TickMods::default());
        mods.add(&mut events, load(&host, GOOD, EMPTY));
        assert_eq!(TickMods::unload(&mut events, GOOD), 1);
        assert!(events.tick_end.is_empty());
        let _next = load(&host, BAD, EMPTY);
    }
}
