//! An entity taken flies to its taker over three ticks (the square of the
//! time), following where the taker is.

use serde_json::{Value, json};

use super::scene::partials;
use crate::bits::{f32_bits, f64_bits};
use crate::case::case;

type Target = (f64, f64, f64, f64);

fn pickup_row(label: &str, item: (f64, f64, f64), targets: &[Target], camera: (f64, f64, f64)) -> Value {
    let targets: Vec<Value> = targets
        .iter()
        .map(|(x, y, z, eye)| json!({"x": f64_bits(*x), "y": f64_bits(*y), "z": f64_bits(*z), "eye": f32_bits(*eye)}))
        .collect();
    let mut shown = partials().iter().map(|bits| (*bits).to_owned()).collect::<Vec<_>>();
    shown.extend([f32_bits(0.3), f32_bits(0.875)]);
    json!({
        "label": label, "blocks": [], "chunk_radius": 1,
        "x": f64_bits(item.0), "y": f64_bits(item.1), "z": f64_bits(item.2),
        "camera_x": f64_bits(camera.0), "camera_y": f64_bits(camera.1), "camera_z": f64_bits(camera.2),
        "targets": targets, "partials": shown,
    })
}

/// An entity taken flies to its taker over three ticks (the square of the
/// time), following where the taker is: one standing, one walking off, one
/// crouching and turning back; ticked past its end.
pub fn item_pickup() -> Value {
    let standing = [(0.5, 64.0, 0.5, 1.62); 5];
    let walking: Vec<Target> = (0..5)
        .map(|tick| {
            let tick = f64::from(tick);
            (2.3 + 0.21 * tick, 64.0, -1.7 + 0.13 * tick, 1.62)
        })
        .collect();
    let crouching = [
        (-3.1, 63.5, 2.9, 1.62),
        (-3.0, 63.5, 2.8, 1.27),
        (-3.2, 63.25, 2.7, 1.27),
        (-3.3, 63.0, 2.75, 1.27),
        (-3.3, 63.0, 2.75, 1.62),
    ];
    let rows = vec![
        pickup_row("standing", (1.25, 64.0, 0.75), &standing, (0.0, 65.0, 0.0)),
        pickup_row("walking-off", (3.0, 64.5, -2.0), &walking, (1.5, 65.62, -1.0)),
        pickup_row("crouching", (-1.6, 63.0, 1.9), &crouching, (-3.2, 64.8, 2.8)),
    ];
    case("movement/item-pickup", "movement", "item_pickup", json!({ "rows": rows }))
}
