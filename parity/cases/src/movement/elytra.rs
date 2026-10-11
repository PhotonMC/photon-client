//! The elytra state cases.

use serde_json::{Value, json};

use crate::bits::f64_bits;
use crate::case::case;

fn tick(glide: bool, sneak: bool, motion: (f64, f64, f64)) -> Value {
    json!({
        "fall_flying": glide, "crouching": sneak, "x": f64_bits(motion.0), "y": f64_bits(motion.1),
        "z": f64_bits(motion.2),
    })
}

const STILL: (f64, f64, f64) = (0.0, 0.0, 0.0);

/// A glide at a motion for `count` ticks.
fn glide(count: usize, motion: (f64, f64, f64)) -> Vec<Value> {
    vec![tick(true, false, motion); count]
}

fn scenario(label: &str, ticks: Vec<Value>) -> Value {
    json!({"label": label, "ticks": ticks})
}

/// The real `ElytraAnimationState` of a real remote player ticked through
/// scenarios: standing, sneaking, gliding level, diving steeply and
/// shallowly, rising, a glide that ends, sneaking while gliding, and no
/// motion.
pub fn elytra_state() -> Value {
    let standing = |count: usize| vec![tick(false, false, STILL); count];
    let sneaking = |count: usize| vec![tick(false, true, STILL); count];
    let varying = |step: fn(f64) -> (f64, f64, f64), count: usize| -> Vec<Value> {
        (0..count).map(|i| tick(true, false, step(i as f64))).collect()
    };
    let scenarios = vec![
        scenario("standing", standing(12)),
        scenario("sneaking", sneaking(12)),
        scenario("sneak-then-stand", [sneaking(8), standing(8)].concat()),
        scenario("glide-level", glide(14, (0.0, 0.0, 1.2))),
        scenario("glide-dive-steep", glide(14, (0.0, -1.5, 0.3))),
        scenario("glide-dive-shallow", glide(14, (0.0, -0.2, 1.5))),
        scenario("glide-rising", glide(14, (0.0, 0.8, 1.0))),
        scenario("glide-no-motion", glide(8, STILL)),
        scenario("glide-tiny-motion", glide(8, (0.0, -1.0e-6, 0.0))),
        scenario("glide-ends", [glide(6, (0.0, -0.7, 1.0)), standing(10)].concat()),
        scenario("glide-sneaking", vec![tick(true, true, (0.5, -0.5, 0.5)); 10]),
        scenario("glide-varying", varying(|i| (0.1 * i, -0.1 * i, 1.0 - 0.05 * i), 16)),
        scenario("glide-turning-down", varying(|i| (0.0, -0.05 * i, 1.0), 16)),
    ];
    case("movement/elytra-state", "movement", "elytra_state", json!({"rows": scenarios}))
}
