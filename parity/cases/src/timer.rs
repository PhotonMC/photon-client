//! The `timer` family: `DeltaTracker` traces of game and real time.

use serde_json::{Value, json};

use crate::bits::f32_bits;
use crate::line::float_text;

struct Rate {
    tps: f64,
    target: f64,
}

impl Default for Rate {
    fn default() -> Self {
        Self {
            tps: 20.0,
            target: 50.0,
        }
    }
}

fn trace(label: &str, steps: Vec<Value>, start: u64, rate: &Rate) -> Value {
    json!({
        "case_id": format!("timer/{label}"), "family": "timer", "operation": "trace",
        "start_ms": start.to_string(), "tps": f32_bits(rate.tps),
        "target_mspt": f32_bits(rate.target), "steps": steps,
    })
}

fn time(action: &str, ms: u64) -> Value {
    json!({"action": action, "ms": ms.to_string()})
}

fn flag(action: &str, value: bool) -> Value {
    json!({"action": action, "value": value})
}

fn target(value: f64) -> Value {
    json!({"action": "target", "value": f32_bits(value)})
}

/// An integral rate prints as an integer in the label, as written.
fn rate_label(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        float_text(value)
    }
}

pub fn cases() -> Vec<Value> {
    let rate = Rate::default();
    let mut cases = Vec::new();
    for delta in [0, 1, 49, 50, 51, 1000] {
        let steps = vec![time("game", delta), time("real", delta)];
        cases.push(trace(&format!("boundary-{delta}"), steps, 0, &rate));
    }
    let residual = [17, 34, 49, 50, 51, 99, 100, 100, 101];
    let steps = residual.iter().map(|ms| time("game", *ms)).collect();
    cases.push(trace("residual", steps, 0, &rate));
    for (tps, target) in [
        (10.0, 100.0),
        (20.0, 25.0),
        (30.0, 1000.0 / 30.0),
        (60.0, 1000.0 / 60.0),
    ] {
        let steps = [1, 17, 33, 50, 100, 1000]
            .iter()
            .flat_map(|ms| [time("game", *ms), time("real", *ms)])
            .collect();
        let label = format!("rate-{}-{}", rate_label(tps), rate_label(target));
        cases.push(trace(&label, steps, 0, &Rate { tps, target }));
    }
    cases.extend(state_cases(&rate));
    let start = (1 << 53) + 1;
    let steps = vec![
        time("game", start + 49),
        time("real", start + 50),
        time("game", start + 50),
        time("game", start + 51),
    ];
    cases.push(trace("large-clock", steps, start, &rate));
    cases
}

fn state_cases(rate: &Rate) -> Vec<Value> {
    let mut cases = Vec::new();
    let steps = vec![
        time("game", 25),
        target(100.0),
        time("game", 100),
        target(25.0),
    ];
    let steps = [steps, vec![time("game", 125), time("real", 125)]].concat();
    cases.push(trace("rate-change", steps, 0, rate));
    let steps = vec![
        time("game", 25),
        flag("pause", true),
        time("game", 50),
        flag("pause", true),
        time("real", 75),
        time("game", 75),
        flag("pause", false),
        time("game", 100),
        flag("pause", false),
    ];
    cases.push(trace("pause-residual", steps, 0, rate));
    for pause in [false, true] {
        for freeze in [false, true] {
            let steps = vec![
                time("game", 25),
                flag("pause", pause),
                flag("freeze", freeze),
                time("game", 75),
                time("real", 100),
                flag("freeze", false),
                flag("pause", false),
                time("game", 100),
            ];
            let label = format!("pause-{}-freeze-{}", title(pause), title(freeze));
            cases.push(trace(&label, steps, 0, rate));
        }
    }
    for delta in [349, 350, 351, 5000] {
        let steps = vec![time("real", delta), time("real", delta + 1)];
        cases.push(trace(&format!("real-limit-{delta}"), steps, 0, rate));
    }
    cases
}

/// `True` / `False`, as the labels spell them.
fn title(flag: bool) -> &'static str {
    if flag { "True" } else { "False" }
}
