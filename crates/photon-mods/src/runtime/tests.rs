use super::*;

const HUD: &str = r#"(component
  (import "hud-text" (func $hud (param "x" s32) (param "y" s32) (param "colour" u32) (param "text" string)))
  (import "play-sound" (func $snd (param "name" string) (param "volume" f32) (param "pitch" f32)))
  (core module $mem (memory (export "mem") 1) (data (i32.const 0) "Cash"))
  (core instance $m (instantiate $mem))
  (core func $hud_l (canon lower (func $hud) (memory (core memory $m "mem"))))
  (core func $snd_l (canon lower (func $snd) (memory (core memory $m "mem"))))
  (core module $code
    (import "h" "hud" (func (param i32 i32 i32 i32 i32)))
    (import "h" "snd" (func (param i32 i32 f32 f32)))
    (func (export "tick")
      (call 0 (i32.const 1) (i32.const 2) (i32.const 0xFFFFFF) (i32.const 0) (i32.const 4))
      (call 1 (i32.const 0) (i32.const 4) (f32.const 1.0) (f32.const 1.0))))
  (core instance $i (instantiate $code (with "h" (instance (export "hud" (func $hud_l)) (export "snd" (func $snd_l))))))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

const TRAPS: &str = r#"(component
  (core module $m (func (export "tick") unreachable))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

fn caps(list: &[&str]) -> Vec<String> {
    list.iter().map(|cap| (*cap).to_owned()).collect()
}

#[test]
fn only_the_harmless_capabilities_are_granted() {
    assert!(grant(&caps(&["hud.text", "sound.play", "events.tick"])).is_some());
    assert_eq!(
        grant(&caps(&["hud.text"])).map(|c| (c.tick, c.hud_text, c.sound)),
        Some((false, true, false))
    );
    assert!(grant(&caps(&["hud.text", "fs.write"])).is_none());
    assert!(grant(&caps(&["net.connect"])).is_none());
}

#[test]
fn a_mod_with_a_forbidden_capability_does_not_load() {
    let mut runtime = Runtime::default();
    let refused = runtime.load(("evil", "h1"), &caps(&["fs.write"]), HUD.as_bytes());
    assert!(refused.is_err());
    assert!(runtime.ids().is_empty());
}

#[test]
fn a_mod_may_not_use_what_it_did_not_ask_for() {
    let mut runtime = Runtime::default();
    // The mod imports the sound too, but only asked for the HUD.
    let result = runtime.load(("sneaky", "h"), &caps(&["hud.text"]), HUD.as_bytes());
    assert!(result.is_err());
}

#[test]
fn a_loaded_mod_ticks_into_the_hud_and_the_sounds() {
    let mut runtime = Runtime::default();
    runtime
        .load(
            ("tycoon", "h1"),
            &caps(&["events.tick", "hud.text", "sound.play"]),
            HUD.as_bytes(),
        )
        .expect("load");
    let sounds = runtime.tick();
    assert_eq!(
        runtime.hud,
        [HudLine {
            x: 1,
            y: 2,
            colour: 0xFFFFFF,
            text: "Cash".into()
        }]
    );
    assert_eq!(sounds.len(), 1);
    // The HUD is the last tick's, not a growing list.
    runtime.tick();
    assert_eq!(runtime.hud.len(), 1);
}

#[test]
fn an_update_replaces_the_mod_and_a_dropped_mod_goes() {
    let mut runtime = Runtime::default();
    let both = caps(&["events.tick", "hud.text", "sound.play"]);
    runtime.load(("a", "h1"), &both, HUD.as_bytes()).expect("a");
    runtime.load(("b", "h1"), &both, HUD.as_bytes()).expect("b");
    assert!(runtime.has("a", "h1"));
    // The server now lists a with a new hash, and no b.
    runtime.keep_only(&[("a", "h2")]);
    assert!(runtime.ids().is_empty());
    runtime
        .load(("a", "h2"), &both, HUD.as_bytes())
        .expect("a2");
    assert!(runtime.has("a", "h2") && !runtime.has("a", "h1"));
    runtime
        .load(("a", "h3"), &both, HUD.as_bytes())
        .expect("a3 replaces");
    assert_eq!(runtime.ids(), ["a"]);
}

#[test]
fn a_mod_that_traps_is_unloaded_and_the_others_keep_running() {
    let mut runtime = Runtime::default();
    runtime
        .load(("bad", "h"), &caps(&["events.tick"]), TRAPS.as_bytes())
        .expect("bad");
    runtime
        .load(
            ("good", "h"),
            &caps(&["events.tick", "hud.text", "sound.play"]),
            HUD.as_bytes(),
        )
        .expect("good");
    runtime.tick();
    assert_eq!(runtime.ids(), ["good"]);
    assert_eq!(runtime.hud.len(), 1);
}

#[test]
fn a_mod_ticks_only_if_it_asked_for_events_tick() {
    let mut runtime = Runtime::default();
    let draw = ["hud.text", "sound.play"];
    runtime
        .load(("quiet", "h"), &caps(&draw), HUD.as_bytes())
        .expect("quiet");
    runtime.tick();
    assert!(runtime.hud.is_empty(), "no events.tick, no tick");
    assert_eq!(runtime.ids(), ["quiet"]);
    runtime
        .load(
            ("loud", "h"),
            &caps(&["events.tick", "hud.text", "sound.play"]),
            HUD.as_bytes(),
        )
        .expect("loud");
    runtime.tick();
    assert_eq!(runtime.hud.len(), 1);
}

#[test]
fn a_mod_that_traps_without_events_tick_is_never_called() {
    let mut runtime = Runtime::default();
    runtime
        .load(("bad", "h"), &[], TRAPS.as_bytes())
        .expect("bad");
    runtime.tick();
    assert_eq!(runtime.ids(), ["bad"]);
}

#[test]
fn an_unknown_capability_refuses_the_mod_but_not_the_others() {
    let mut runtime = Runtime::default();
    let all = caps(&["events.tick", "hud.text", "sound.play"]);
    runtime.load(("ok", "h"), &all, HUD.as_bytes()).expect("ok");
    assert!(
        runtime
            .load(
                ("odd", "h"),
                &caps(&["events.tick", "world.read"]),
                HUD.as_bytes()
            )
            .is_err()
    );
    assert_eq!(runtime.ids(), ["ok"]);
}

const ENDLESS: &str = r#"(component
  (core module $m (func (export "tick") (loop br 0)))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

fn tick_cap() -> Vec<String> {
    caps(&["events.tick", "hud.text", "sound.play"])
}

#[test]
fn a_spent_budget_defers_the_rest_and_they_go_first_next_tick() {
    let mut runtime = Runtime::default();
    for id in ["a", "b", "c"] {
        runtime
            .load((id, "h"), &tick_cap(), HUD.as_bytes())
            .expect(id);
    }
    // A zero budget lets exactly one mod run per tick, in turn.
    let mut order = Vec::new();
    for _ in 0..6 {
        order.push(runtime.next_up().unwrap().to_owned());
        let report = runtime.tick_within(Duration::ZERO);
        assert_eq!((report.ran, report.deferred), (1, 2));
    }
    assert_eq!(order, ["a", "b", "c", "a", "b", "c"]);
    // A generous budget runs everyone.
    let report = runtime.tick_within(Duration::from_secs(60));
    assert_eq!((report.ran, report.deferred), (3, 0));
    assert_eq!(report.sounds.len(), 3);
    assert_eq!(runtime.hud.len(), 3);
}

#[test]
fn a_mod_that_waits_keeps_its_text() {
    let mut runtime = Runtime::default();
    for id in ["a", "b"] {
        runtime
            .load((id, "h"), &tick_cap(), HUD.as_bytes())
            .expect(id);
    }
    runtime.tick_within(Duration::from_secs(60));
    assert_eq!(runtime.hud.len(), 2);
    runtime.tick_within(Duration::ZERO);
    assert_eq!(runtime.hud.len(), 2, "the mod that waited did not blink");
}

#[test]
fn slow_mods_cannot_stall_a_tick_and_do_not_starve_the_cheap_ones() {
    let mut runtime = Runtime::default();
    let tick_only = caps(&["events.tick"]);
    for n in 0..5 {
        let id = format!("slow{n}");
        runtime
            .load((&id, "h"), &tick_only, ENDLESS.as_bytes())
            .expect("slow");
    }
    runtime
        .load(("cheap", "h"), &tick_cap(), HUD.as_bytes())
        .expect("cheap");
    let mut ticks = 0;
    while runtime.ids().iter().any(|id| id.starts_with("slow")) {
        let before = std::time::Instant::now();
        runtime.tick_within(Duration::from_millis(4));
        // One overrunning mod is cut at its own allowance; the others wait.
        assert!(before.elapsed() < Duration::from_secs(5));
        ticks += 1;
        assert!(ticks <= 6, "every slow mod is unloaded after its one run");
    }
    assert_eq!(runtime.ids(), ["cheap"]);
    runtime.tick_within(Duration::from_millis(4));
    assert_eq!(runtime.hud.len(), 1);
}

#[test]
fn a_failing_mod_does_not_make_the_cursor_skip_the_next_one() {
    let mut runtime = Runtime::default();
    runtime
        .load(("bad", "h"), &caps(&["events.tick"]), TRAPS.as_bytes())
        .expect("bad");
    runtime
        .load(("good", "h"), &tick_cap(), HUD.as_bytes())
        .expect("good");
    let report = runtime.tick_within(Duration::ZERO);
    assert_eq!(report.ran, 0);
    assert_eq!(runtime.ids(), ["good"]);
    assert_eq!(runtime.next_up(), Some("good"));
    assert_eq!(runtime.tick_within(Duration::ZERO).ran, 1);
}

const RECURSES: &str = r#"(component
  (core module $m
    (func $down (param i32) (result i32) (call $down (i32.add (local.get 0) (i32.const 1))))
    (func (export "tick") (drop (call $down (i32.const 0)))))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

#[test]
fn a_mod_that_recurses_forever_is_unloaded_alone() {
    let mut runtime = Runtime::default();
    let tick_only = caps(&["events.tick"]);
    runtime
        .load(("deep", "h"), &tick_only, RECURSES.as_bytes())
        .expect("deep");
    runtime
        .load(("good", "h"), &tick_cap(), HUD.as_bytes())
        .expect("good");
    runtime.tick_within(Duration::from_secs(60));
    assert_eq!(runtime.ids(), ["good"]);
    assert_eq!(runtime.hud.len(), 1);
}
