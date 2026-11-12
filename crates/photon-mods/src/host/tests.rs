use photon_api::{Event, Owner};

use super::*;

const MS: Duration = Duration::from_secs(1);
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
    let mut loaded = host.load(MOD, EMPTY.as_bytes(), &[], MS).unwrap();
    assert!(loaded.tick().is_ok());
    assert_eq!(loaded.owner(), MOD);
}

#[test]
fn an_import_that_was_not_granted_is_refused() {
    let host = Host::new().unwrap();
    let refused = host.load(MOD, NEEDS_REGISTRY.as_bytes(), &[], MS);
    assert!(matches!(refused, Err(HostError::Load(why)) if why.contains("registry-read")));
    let granted = host.load(MOD, NEEDS_REGISTRY.as_bytes(), &[("registry-read", 7)], MS);
    assert!(granted.is_ok());
}

#[test]
fn an_endless_loop_is_stopped_by_the_clock() {
    let host = Host::new().unwrap();
    let mut loaded = host
        .load(MOD, ENDLESS.as_bytes(), &[], Duration::from_millis(20))
        .unwrap();
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
        .map(|_| host.load(MOD, EMPTY.as_bytes(), &[], MS).unwrap())
        .collect();
    for loaded in &mut all {
        loaded.tick().unwrap();
    }
}

#[test]
fn unloading_a_mod_takes_its_handlers_with_it() {
    let host = Host::new().unwrap();
    let mut ticks: Event<()> = Event::default();
    let loaded = host.load(MOD, EMPTY.as_bytes(), &[], MS).unwrap();
    ticks.register(loaded.owner(), |_| {});
    let owner = loaded.owner();
    drop(loaded);
    assert_eq!(ticks.remove_owner(owner), 1);
    assert!(ticks.is_empty());
}

#[test]
fn a_dropped_mod_gives_its_slot_back() {
    let host = Host::with_slots(1).unwrap();
    let first = host.load(MOD, EMPTY.as_bytes(), &[], MS).unwrap();
    assert!(host.load(MOD, EMPTY.as_bytes(), &[], MS).is_err());
    drop(first);
    assert!(host.load(MOD, EMPTY.as_bytes(), &[], MS).is_ok());
}

#[test]
fn code_that_runs_while_loading_is_not_cut_off_at_once() {
    let host = Host::new().unwrap();
    let busy = r#"(component
  (core module $m (func $f (local i32) (loop (local.set 0 (i32.add (local.get 0) (i32.const 1))) (br_if 0 (i32.lt_u (local.get 0) (i32.const 1000000))))) (start $f) (func (export "tick")))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;
    std::thread::sleep(Duration::from_millis(20));
    assert!(host.load(MOD, busy.as_bytes(), &[], MS).is_ok());
}

const HUD_MOD: &str = r#"(component
  (import "hud-text" (func $hud (param "x" s32) (param "y" s32) (param "colour" u32) (param "text" string)))
  (core module $mem (memory (export "mem") 1) (data (i32.const 0) "Cash: 100"))
  (core instance $m (instantiate $mem))
  (core func $hud_l (canon lower (func $hud) (memory (core memory $m "mem"))))
  (core module $code
    (import "h" "hud" (func (param i32 i32 i32 i32 i32)))
    (func (export "tick") (call 0 (i32.const 4) (i32.const 6) (i32.const 0xFFAA00) (i32.const 0) (i32.const 9))))
  (core instance $i (instantiate $code (with "h" (instance (export "hud" (func $hud_l))))))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

const SOUND_MOD: &str = r#"(component
  (import "play-sound" (func $snd (param "name" string) (param "volume" f32) (param "pitch" f32)))
  (core module $mem (memory (export "mem") 1) (data (i32.const 0) "mymod:cash_register"))
  (core instance $m (instantiate $mem))
  (core func $snd_l (canon lower (func $snd) (memory (core memory $m "mem"))))
  (core module $code
    (import "h" "snd" (func (param i32 i32 f32 f32)))
    (func (export "tick") (call 0 (i32.const 0) (i32.const 19) (f32.const 5.0) (f32.const 1.0))))
  (core instance $i (instantiate $code (with "h" (instance (export "snd" (func $snd_l))))))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

const HUD_FLOOD: &str = r#"(component
  (import "hud-text" (func $hud (param "x" s32) (param "y" s32) (param "colour" u32) (param "text" string)))
  (core module $mem (memory (export "mem") 1) (data (i32.const 0) "x"))
  (core instance $m (instantiate $mem))
  (core func $hud_l (canon lower (func $hud) (memory (core memory $m "mem"))))
  (core module $code
    (import "h" "hud" (func (param i32 i32 i32 i32 i32)))
    (func (export "tick") (local i32)
      (loop
        (call 0 (i32.const 0) (local.get 0) (i32.const 0) (i32.const 0) (i32.const 1))
        (local.set 0 (i32.add (local.get 0) (i32.const 1)))
        (br_if 0 (i32.lt_u (local.get 0) (i32.const 200))))))
  (core instance $i (instantiate $code (with "h" (instance (export "hud" (func $hud_l))))))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

fn with_caps(host: &Host, wat: &str, caps: Caps) -> Result<Mod, HostError> {
    host.load_with(MOD, wat.as_bytes(), (&[], caps), MS)
}

#[test]
fn a_mod_draws_hud_text_only_when_granted() {
    let host = Host::new().unwrap();
    let denied = with_caps(&host, HUD_MOD, Caps::default());
    assert!(matches!(denied, Err(HostError::Load(why)) if why.contains("hud-text")));
    let mut loaded = with_caps(
        &host,
        HUD_MOD,
        Caps {
            hud_text: true,
            sound: false,
            ..Caps::default()
        },
    )
    .unwrap();
    loaded.tick().unwrap();
    let (effects, dropped) = loaded.take_effects();
    assert_eq!(dropped, 0);
    assert_eq!(
        effects,
        [Effect::HudText {
            x: 4,
            y: 6,
            colour: 0xFFAA00,
            text: "Cash: 100".to_owned()
        }]
    );
    // The next tick starts empty.
    assert!(loaded.take_effects().0.is_empty());
}

#[test]
fn a_sound_is_clamped_and_needs_its_own_grant() {
    let host = Host::new().unwrap();
    let hud_only = Caps {
        hud_text: true,
        sound: false,
        ..Caps::default()
    };
    assert!(with_caps(&host, SOUND_MOD, hud_only).is_err());
    let mut loaded = with_caps(
        &host,
        SOUND_MOD,
        Caps {
            hud_text: false,
            sound: true,
            ..Caps::default()
        },
    )
    .unwrap();
    loaded.tick().unwrap();
    let (effects, _) = loaded.take_effects();
    assert_eq!(
        effects,
        [Effect::Sound {
            name: "mymod:cash_register".to_owned(),
            volume: 1.0,
            pitch: 1.0
        }]
    );
}

#[test]
fn a_flood_of_effects_is_capped_and_counted() {
    let host = Host::new().unwrap();
    let mut loaded = with_caps(
        &host,
        HUD_FLOOD,
        Caps {
            hud_text: true,
            sound: false,
            ..Caps::default()
        },
    )
    .unwrap();
    loaded.tick().unwrap();
    let (effects, dropped) = loaded.take_effects();
    assert_eq!(effects.len(), MAX_EFFECTS);
    assert_eq!(dropped as usize, 200 - MAX_EFFECTS);
}

#[test]
fn compiling_on_another_thread_then_instantiating_here() {
    fn shareable<T: Send + Sync>() {}
    shareable::<Host>();
    shareable::<Compiled>();
    let host = std::sync::Arc::new(Host::new().unwrap());
    let worker = std::sync::Arc::clone(&host);
    let compiled = std::thread::spawn(move || worker.compile(NEEDS_REGISTRY.as_bytes()))
        .join()
        .unwrap()
        .expect("compile");
    // Linking happens at instantiation: the grant decides, and one compile serves many mods.
    assert!(matches!(
        host.instantiate(MOD, &compiled, (&[], Caps::default()), MS),
        Err(HostError::Load(why)) if why.contains("registry-read")
    ));
    let grants: &[(&str, u32)] = &[("registry-read", 1)];
    let mut first = host
        .instantiate(MOD, &compiled, (grants, Caps::default()), MS)
        .expect("first");
    let mut second = host
        .instantiate(MOD, &compiled, (grants, Caps::default()), MS)
        .expect("second");
    assert!(first.tick().is_ok() && second.tick().is_ok());
}

#[test]
fn bad_code_fails_at_compile_not_at_instantiate() {
    let host = Host::new().unwrap();
    assert!(matches!(
        host.compile(b"not a component"),
        Err(HostError::Load(_))
    ));
}

#[test]
fn an_endless_start_function_is_stopped_when_instantiating_a_compile() {
    const ENDLESS_START: &str = r#"(component
  (core module $m (func $s (loop br 0)) (start $s) (func (export "tick")))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;
    let host = Host::new().unwrap();
    let compiled = host.compile(ENDLESS_START.as_bytes()).expect("compile");
    let started = std::time::Instant::now();
    let result = host.instantiate(
        MOD,
        &compiled,
        (&[], Caps::default()),
        Duration::from_millis(20),
    );
    assert!(result.is_err());
    assert!(started.elapsed() < Duration::from_secs(5));
}

const TABLE_GROW: &str = r#"(component
  (core module $m
    (table 1 funcref)
    (func (export "tick")
      ;; a modest growth is allowed, a huge one must be refused with -1, not trap or allocate
      (if (i32.eq (table.grow (ref.null func) (i32.const 100)) (i32.const -1)) (then unreachable))
      (if (i32.ne (table.grow (ref.null func) (i32.const 1000000)) (i32.const -1)) (then unreachable))))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

const BIG_TABLE: &str = r#"(component
  (core module $m (table 1000000 funcref) (func (export "tick")))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

const RECURSES: &str = r#"(component
  (core module $m
    (func $down (param i32) (result i32) (call $down (i32.add (local.get 0) (i32.const 1))))
    (func (export "tick") (drop (call $down (i32.const 0)))))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

#[test]
fn a_table_cannot_grow_past_the_limit() {
    let host = Host::new().unwrap();
    let mut loaded = host.load(MOD, TABLE_GROW.as_bytes(), &[], MS).unwrap();
    loaded
        .tick()
        .expect("growth past the limit is refused, not fatal");
    assert!(host.load(MOD, BIG_TABLE.as_bytes(), &[], MS).is_err());
}

#[test]
fn runaway_recursion_traps_on_the_wasm_stack() {
    let host = Host::new().unwrap();
    let mut loaded = host.load(MOD, RECURSES.as_bytes(), &[], MS).unwrap();
    let Err(HostError::Call(why)) = loaded.tick() else {
        panic!("recursion returned");
    };
    assert!(why.contains("stack"), "{why}");
    // The host and its other mods are fine.
    let mut other = host.load(MOD, EMPTY.as_bytes(), &[], MS).unwrap();
    assert!(other.tick().is_ok());
}

#[test]
fn a_load_past_the_pool_is_an_error_not_a_panic() {
    let host = Host::with_slots(2).unwrap();
    let _a = host.load(MOD, EMPTY.as_bytes(), &[], MS).unwrap();
    let _b = host.load(MOD, EMPTY.as_bytes(), &[], MS).unwrap();
    let Err(HostError::Load(why)) = host.load(MOD, EMPTY.as_bytes(), &[], MS) else {
        panic!("a third mod fitted");
    };
    assert!(why.contains("limit"), "{why}");
}
