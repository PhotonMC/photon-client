//! `player.state`: the reads follow the player of each tick and need the capability.

use super::*;

const LOOK: &str = r#"(component
  (import "player-x" (func $x (result f64)))
  (import "player-health" (func $hp (result f32)))
  (import "player-yaw" (func $yaw (result f32)))
  (import "hud-text" (func $hud (param "x" s32) (param "y" s32) (param "colour" u32) (param "text" string)))
  (core module $mem (memory (export "mem") 1) (data (i32.const 0) "P"))
  (core instance $m (instantiate $mem))
  (core func $x_l (canon lower (func $x)))
  (core func $hp_l (canon lower (func $hp)))
  (core func $yaw_l (canon lower (func $yaw)))
  (core func $hud_l (canon lower (func $hud) (memory (core memory $m "mem"))))
  (core module $code
    (import "h" "x" (func (result f64)))
    (import "h" "hp" (func (result f32)))
    (import "h" "yaw" (func (result f32)))
    (import "h" "hud" (func (param i32 i32 i32 i32 i32)))
    (func (export "tick")
      (call 3 (i32.trunc_f64_s (call 0)) (i32.trunc_f32_s (call 1)) (i32.trunc_f32_s (call 2)) (i32.const 0) (i32.const 1))))
  (core instance $i (instantiate $code (with "h" (instance (export "x" (func $x_l)) (export "hp" (func $hp_l)) (export "yaw" (func $yaw_l)) (export "hud" (func $hud_l))))))
  (func (export "tick") (canon lift (core func $i "tick"))))"#;

fn caps(list: &[&str]) -> Vec<String> {
    list.iter().map(|cap| (*cap).to_owned()).collect()
}

#[test]
fn the_player_reads_follow_each_tick_and_are_read_only_values() {
    let mut runtime = Runtime::default();
    let asked = caps(&["events.tick", "hud.text", "player.state"]);
    runtime
        .load(("look", "h"), &asked, LOOK.as_bytes())
        .expect("look");
    runtime.set_player(PlayerView {
        x: 12.7,
        health: 18.0,
        yaw: 90.0,
        ..PlayerView::default()
    });
    runtime.tick();
    let line = &runtime.hud[0];
    assert_eq!((line.x, line.y, line.colour), (12, 18, 90));
    runtime.set_player(PlayerView {
        x: -3.2,
        health: 4.0,
        yaw: 10.0,
        ..PlayerView::default()
    });
    runtime.tick();
    let line = &runtime.hud[0];
    assert_eq!((line.x, line.y, line.colour), (-3, 4, 10));
}

#[test]
fn a_mod_that_did_not_ask_for_the_player_does_not_load() {
    let mut runtime = Runtime::default();
    let asked = caps(&["events.tick", "hud.text"]);
    assert!(
        runtime
            .load(("look", "h"), &asked, LOOK.as_bytes())
            .is_err()
    );
}
