//! `fs.mod_data` through the guest's imports: WAT mods that read and write their store.

use super::*;
use crate::{MAX_WRITES_PER_TICK, data_path};

/// A mod whose tick is `body`. The key "n" is at offset 0 of memory, and 3000 is a scratch byte.
/// Imports are core functions 0 (`data-get`: key ptr, key len, return area), 1 (`data-set`: key
/// ptr, key len, value ptr, value len, answer) and 2 (`data-delete`: key ptr, key len, answer).
fn guest(body: &str) -> String {
    format!(
        r#"(component
  (import "data-get" (func $get (param "key" string) (result (option (list u8)))))
  (import "data-set" (func $set (param "key" string) (param "value" (list u8)) (result u32)))
  (import "data-delete" (func $del (param "key" string) (result bool)))
  (core module $libc
    (memory (export "mem") 2)
    (global $bump (mut i32) (i32.const 8192))
    (func (export "realloc") (param i32 i32 i32 i32) (result i32)
      (local $p i32)
      (local.set $p (i32.and (i32.add (global.get $bump) (i32.sub (local.get 2) (i32.const 1)))
                             (i32.sub (i32.const 0) (local.get 2))))
      (global.set $bump (i32.add (local.get $p) (local.get 3)))
      (local.get $p))
    (data (i32.const 0) "n"))
  (core instance $libc (instantiate $libc))
  (core func $get_l (canon lower (func $get) (memory (core memory $libc "mem")) (realloc (core func $libc "realloc"))))
  (core func $set_l (canon lower (func $set) (memory (core memory $libc "mem")) (realloc (core func $libc "realloc"))))
  (core func $del_l (canon lower (func $del) (memory (core memory $libc "mem"))))
  (core module $code
    (import "h" "get" (func $get (param i32 i32 i32)))
    (import "h" "set" (func $set (param i32 i32 i32 i32) (result i32)))
    (import "h" "del" (func $del (param i32 i32) (result i32)))
    (import "libc" "mem" (memory 2))
    (func (export "tick") (local $i i32) (local $ptr i32)
      {body}))
  (core instance $i (instantiate $code
    (with "h" (instance (export "get" (func $get_l)) (export "set" (func $set_l)) (export "del" (func $del_l))))
    (with "libc" (instance (export "mem" (memory $libc "mem"))))))
  (func (export "tick") (canon lift (core func $i "tick"))))"#
    )
}

/// Adds one to the byte stored under "n" (0 if there is none).
const COUNT: &str = r#"
      (call $get (i32.const 0) (i32.const 1) (i32.const 2048))
      (if (i32.eq (i32.load8_u (i32.const 2048)) (i32.const 1))
        (then (i32.store8 (i32.const 3000) (i32.add (i32.load8_u (i32.load (i32.const 2052))) (i32.const 1))))
        (else (i32.store8 (i32.const 3000) (i32.const 1))))
      (drop (call $set (i32.const 0) (i32.const 1) (i32.const 3000) (i32.const 1)))"#;

fn caps(list: &[&str]) -> Vec<String> {
    list.iter().map(|cap| (*cap).to_owned()).collect()
}

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("photon-mod-guest-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

const ASKED: &[&str] = &["events.tick", "fs.mod_data"];

/// Loads `wat` as `id` with a store at `path`, and runs `ticks` ticks.
fn run(runtime: &mut Runtime, (id, hash): (&str, &str), wat: &str, path: &std::path::Path) {
    let compiled = runtime.host().unwrap().compile(wat.as_bytes()).unwrap();
    runtime
        .load_stored((id, hash), &caps(ASKED), &compiled, Some(path.to_owned()))
        .unwrap();
}

fn stored(path: &std::path::Path, key: &str) -> Option<Vec<u8>> {
    ModData::load(path).get(key)
}

#[test]
fn data_survives_ticks_a_restart_and_an_update_of_the_mod() {
    let dir = scratch("survive");
    let path = data_path(&dir, "play.example.org:25565", "counter");
    let mut runtime = Runtime::default();
    run(&mut runtime, ("counter", "h1"), &guest(COUNT), &path);
    for _ in 0..3 {
        runtime.tick();
    }
    drop(runtime);
    assert_eq!(stored(&path, "n"), Some(vec![3]));
    // A new connection: a new runtime reads the file and goes on.
    let mut again = Runtime::default();
    run(&mut again, ("counter", "h1"), &guest(COUNT), &path);
    again.tick();
    // The server updated the mod (another hash, the same id): the data stays.
    run(&mut again, ("counter", "h2"), &guest(COUNT), &path);
    again.tick();
    drop(again);
    assert_eq!(stored(&path, "n"), Some(vec![5]));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn mods_and_servers_do_not_see_each_others_data() {
    let dir = scratch("isolated");
    let mut runtime = Runtime::default();
    let paths = [
        data_path(&dir, "a.example:1", "one"),
        data_path(&dir, "a.example:1", "two"),
        data_path(&dir, "b.example:1", "one"),
    ];
    run(&mut runtime, ("one", "h"), &guest(COUNT), &paths[0]);
    run(&mut runtime, ("two", "h"), &guest(COUNT), &paths[1]);
    runtime.tick();
    runtime.tick();
    // The same id on another server starts empty, and ticks once.
    run(&mut runtime, ("one", "h"), &guest(COUNT), &paths[2]);
    runtime.tick();
    drop(runtime);
    assert_eq!(stored(&paths[0], "n"), Some(vec![2]));
    assert_eq!(stored(&paths[1], "n"), Some(vec![3]));
    assert_eq!(stored(&paths[2], "n"), Some(vec![1]));
    let _ = std::fs::remove_dir_all(dir);
}

/// Stores the answer of the call `call` under "n".
fn answer_of(call: &str) -> String {
    format!(
        r#"(i32.store8 (i32.const 3000) {call})
      (drop (call $set (i32.const 0) (i32.const 1) (i32.const 3000) (i32.const 1)))"#
    )
}

#[test]
fn limits_are_enforced_at_the_imports() {
    let dir = scratch("limits");
    let path = data_path(&dir, "s:1", "m");
    let mut runtime = Runtime::default();
    // A 65 byte key, a value of 64 KiB and a byte, and the 17th write of a tick.
    let long_key =
        answer_of("(call $set (i32.const 0) (i32.const 65) (i32.const 3000) (i32.const 1))");
    let big_value =
        answer_of("(call $set (i32.const 0) (i32.const 1) (i32.const 0) (i32.const 65537))");
    // One write more than a tick may make, each storing the number of the write.
    let many = format!(
        r#"(local.set $i (i32.const 0))
      (loop $again
        (i32.store8 (i32.const 3000) (local.get $i))
        (drop (call $set (i32.const 0) (i32.const 1) (i32.const 3000) (i32.const 1)))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br_if $again (i32.lt_u (local.get $i) (i32.const {}))))"#,
        MAX_WRITES_PER_TICK + 1
    );
    for (body, expected) in [(long_key, 1u8), (big_value, 2)] {
        run(&mut runtime, ("m", "h"), &guest(&body), &path);
        runtime.tick();
        runtime.save_data();
        drop(std::mem::take(&mut runtime));
        assert_eq!(stored(&path, "n"), Some(vec![expected]));
    }
    // The last write of the tick is refused, so the one before it is what stays.
    run(&mut runtime, ("m", "h"), &guest(&many), &path);
    runtime.tick();
    drop(runtime);
    assert_eq!(
        stored(&path, "n"),
        Some(vec![MAX_WRITES_PER_TICK as u8 - 1])
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_mod_without_a_store_or_without_asking_does_not_load() {
    let mut runtime = Runtime::default();
    let compiled = runtime
        .host()
        .unwrap()
        .compile(guest(COUNT).as_bytes())
        .unwrap();
    // The player did not allow it: no store, and the capability is not granted.
    let refused = runtime.load_stored(("m", "h"), &caps(ASKED), &compiled, None);
    assert!(refused.is_err());
    // A store, but the mod never asked for the capability: the imports are not linked.
    let dir = scratch("unasked");
    let path = data_path(&dir, "s:1", "m");
    let unasked = runtime.load_stored(("m", "h"), &caps(&["events.tick"]), &compiled, Some(path));
    assert!(unasked.is_err());
    assert!(runtime.ids().is_empty());
}
