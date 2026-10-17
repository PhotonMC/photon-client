//! The beam of a beacon.

use serde_json::{Value, json};

use super::pose::entity;
use crate::bits::f32_bits;
use crate::line::float_text;

/// The real `BeaconRenderer` for a beam of one stretch and of several, at several times of its
/// clock and scales of its radius.
pub fn beacon_beam() -> Vec<Value> {
    let beam = |label: String, sections: &[(i64, i64)], time: f64, scale: f64| {
        let described: Vec<Value> =
            sections.iter().map(|(color, height)| json!({"color": color, "height": height})).collect();
        json!({"label": label, "sections": described, "animation_time": f32_bits(time), "radius_scale": f32_bits(scale)})
    };
    let white = 0xFFF9_FFFE_u32 as i32 as i64;
    let red = 0xFFB0_2E26_u32 as i32 as i64;
    let blue = 0xFF3C_44AA_u32 as i32 as i64;
    let mut rows = Vec::new();
    for time in [0.0, 0.25, 3.5, 17.75, 39.9] {
        rows.push(beam(format!("one stretch at {}", float_text(time)), &[(white, 5)], time, 1.0));
    }
    rows.push(beam("two stretches".to_owned(), &[(white, 2), (red, 7)], 12.5, 1.0));
    rows.push(beam("three stretches".to_owned(), &[(white, 1), (red, 3), (blue, 40)], 5.0, 1.0));
    for scale in [1.0, 1.7, 4.25] {
        rows.push(beam(format!("scaled {}", float_text(scale)), &[(red, 9), (blue, 3)], 20.25, scale));
    }
    vec![entity("beacon-beam", "beacon_beam", rows)]
}

/// The real `TheEndGatewayRenderer` for a beam of several heights, colours, scales and times of
/// its clock.
pub fn gateway_beam() -> Vec<Value> {
    let magenta = 0xFFC7_4EBD_u32 as i32 as i64;
    let purple = 0xFF89_32B8_u32 as i32 as i64;
    let beam = |label: &str, (scale, time): (f64, f64), height: i64, color: i64| json!({"label": label, "scale": f32_bits(scale), "animation_time": f32_bits(time), "height": height, "color": color});
    let rows = vec![
        beam("spawning, just begun", (0.0157, 0.0), 4, magenta),
        beam("spawning, high", (0.9511, 17.25), 301, magenta),
        beam("spawning, full", (1.0, 39.5), 319, magenta),
        beam("cooling, rising", (0.5, 3.75), 25, purple),
        beam("cooling, nearly done", (0.1564, 28.0), 7, purple),
        beam("cooling, tall", (0.9877, 12.5), 49, purple),
    ];
    vec![entity("gateway-beam", "gateway_beam", rows)]
}
