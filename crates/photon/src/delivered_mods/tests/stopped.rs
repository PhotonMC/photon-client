//! A mod that is unloaded while running is reported to the server and not loaded again.

use super::*;

/// Ticks fine once, then traps.
const TRAPS_SECOND_TICK: &str = r#"(component
  (core module $m
    (global $n (mut i32) (i32.const 0))
    (func (export "tick")
      (global.set $n (i32.add (global.get $n) (i32.const 1)))
      (if (i32.ge_u (global.get $n) (i32.const 2)) (then unreachable))))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

const ENDLESS: &str = r#"(component
  (core module $m (func (export "tick") (loop br 0)))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

fn load(dir: &std::path::Path, id: &str, code: &str) -> Manifest {
    let bytes = code.as_bytes();
    ModCache::new(dir.join("cache").join("mods"))
        .put(&mods::hash_of(bytes), bytes)
        .expect("cache");
    manifest_of(id, bytes, r#""events.tick""#)
}

#[test]
fn a_mod_that_traps_is_named_stopped_in_one_hello_and_is_not_loaded_again() {
    let dir = scratch("stopped-trap");
    let mut mods = ModRuntime::default();
    mods.hello("P");
    let manifest = load(&dir, "bad", TRAPS_SECOND_TICK);
    mods.on_manifest(&manifest, &dir);
    settle_all_pending(&mut mods);
    mods.hello("P");
    mods.tick();
    assert!(mods.hello("P").is_none());
    mods.tick();
    assert!(mods.runtime.ids().is_empty());
    assert_eq!(
        text(mods.hello("P")),
        r#"{"v":1,"client":"P","loaded":[],"refused":[{"id":"bad","reason":"stopped"}]}"#
    );
    assert!(mods.hello("P").is_none());
    // The same manifest again does not bring it back; the hello still names it.
    mods.on_manifest(&manifest, &dir);
    assert!(mods.runtime.ids().is_empty());
    assert_eq!(
        text(mods.hello("P")),
        r#"{"v":1,"client":"P","loaded":[],"refused":[{"id":"bad","reason":"stopped"}]}"#
    );
    // A reconnect starts clean.
    mods.restart();
    mods.on_manifest(&manifest, &dir);
    settle_all_pending(&mut mods);
    assert_eq!(mods.runtime.ids(), ["bad"]);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn two_mods_stopped_in_one_tick_make_one_hello_and_an_endless_loop_counts() {
    let dir = scratch("stopped-both");
    let mut mods = ModRuntime::default();
    mods.hello("P");
    let (a, b) = (TRAPS_SECOND_TICK.as_bytes(), ENDLESS.as_bytes());
    for bytes in [a, b] {
        ModCache::new(dir.join("cache").join("mods"))
            .put(&mods::hash_of(bytes), bytes)
            .expect("cache");
    }
    let (ha, hb) = (mods::hash_of(a), mods::hash_of(b));
    let text_manifest = format!(
        r#"{{"v":1,"mods":[{{"id":"a","hash":"{ha}","size":{},"url":"http://127.0.0.1:9/a","caps":["events.tick"]}},{{"id":"b","hash":"{hb}","size":{},"url":"http://127.0.0.1:9/b","caps":["events.tick"]}}]}}"#,
        a.len(),
        b.len()
    );
    let manifest = Manifest::parse(text_manifest.as_bytes()).expect("manifest");
    mods.on_manifest(&manifest, &dir);
    settle_all_pending(&mut mods);
    mods.hello("P");
    mods.tick();
    mods.tick();
    assert!(mods.runtime.ids().is_empty());
    let hello = text(mods.hello("P"));
    assert!(
        hello.contains(r#"{"id":"a","reason":"stopped"}"#),
        "{hello}"
    );
    assert!(
        hello.contains(r#"{"id":"b","reason":"stopped"}"#),
        "{hello}"
    );
    assert!(mods.hello("P").is_none());
    let _ = std::fs::remove_dir_all(dir);
}
