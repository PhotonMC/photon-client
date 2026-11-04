use photon_api::{Event, Owner};

use super::*;

const MOD: Owner = Owner::new("test-mod");

const EMPTY: &str = r#"(component
  (core module $m (func (export "tick")))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

const NEEDS_REGISTRY: &str = r#"(component
  (import "registry-read" (func $r (result u32)))
  (core func $rc (canon lower (func $r)))
  (core module $m (import "h" "r" (func (result i32))) (func (export "tick") call 0 drop))
  (core instance $i (instantiate $m (with "h" (instance (export "r" (func $rc))))))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

const ENDLESS: &str = r#"(component
  (core module $m (func (export "tick") (loop br 0)))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

#[test]
fn an_empty_tick_returns() {
    let host = Host::new().unwrap();
    let mut loaded = host.load(MOD, EMPTY.as_bytes(), &[], 1000).unwrap();
    assert!(loaded.tick().is_ok());
    assert_eq!(loaded.owner(), MOD);
}

#[test]
fn an_import_that_was_not_granted_is_refused() {
    let host = Host::new().unwrap();
    let refused = host.load(MOD, NEEDS_REGISTRY.as_bytes(), &[], 1000);
    assert!(matches!(refused, Err(HostError::Load(why)) if why.contains("registry-read")));
    let granted = host.load(
        MOD,
        NEEDS_REGISTRY.as_bytes(),
        &[("registry-read", 7)],
        1000,
    );
    assert!(granted.is_ok());
}

#[test]
fn an_endless_loop_is_stopped_by_the_clock() {
    let host = Host::new().unwrap();
    let mut loaded = host.load(MOD, ENDLESS.as_bytes(), &[], 20).unwrap();
    let started = std::time::Instant::now();
    let Err(HostError::Call(why)) = loaded.tick() else {
        panic!("the loop returned");
    };
    assert_eq!(why, "interrupt");
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn many_mods_fit_at_once() {
    let host = Host::new().unwrap();
    let mut all: Vec<Mod> = (0..300)
        .map(|_| host.load(MOD, EMPTY.as_bytes(), &[], 1000).unwrap())
        .collect();
    for loaded in &mut all {
        loaded.tick().unwrap();
    }
}

#[test]
fn unloading_a_mod_takes_its_handlers_with_it() {
    let host = Host::new().unwrap();
    let mut ticks: Event<()> = Event::default();
    let loaded = host.load(MOD, EMPTY.as_bytes(), &[], 1000).unwrap();
    ticks.register(loaded.owner(), |_| {});
    let owner = loaded.owner();
    drop(loaded);
    assert_eq!(ticks.remove_owner(owner), 1);
    assert!(ticks.is_empty());
}

#[test]
fn a_dropped_mod_gives_its_slot_back() {
    let host = Host::with_slots(1).unwrap();
    let first = host.load(MOD, EMPTY.as_bytes(), &[], 1000).unwrap();
    assert!(host.load(MOD, EMPTY.as_bytes(), &[], 1000).is_err());
    drop(first);
    assert!(host.load(MOD, EMPTY.as_bytes(), &[], 1000).is_ok());
}

#[test]
fn code_that_runs_while_loading_is_not_cut_off_at_once() {
    let host = Host::new().unwrap();
    let busy = r#"(component
  (core module $m (func $f (local i32) (loop (local.set 0 (i32.add (local.get 0) (i32.const 1))) (br_if 0 (i32.lt_u (local.get 0) (i32.const 1000000))))) (start $f) (func (export "tick")))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;
    std::thread::sleep(Duration::from_millis(20));
    assert!(host.load(MOD, busy.as_bytes(), &[], 1000).is_ok());
}
