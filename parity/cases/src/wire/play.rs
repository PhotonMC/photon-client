//! Play packets that turn the player, move its vehicle or power a projectile, and what the
//! rotation and look-at packets leave the player's view as.

use serde_json::{Value, json};

use super::codec::{flag, varint_hex};
use crate::bits::{f32_bits, f64_bits, hex};
use crate::case::case;

fn f32_hex(value: f32) -> String {
    hex(&value.to_be_bytes())
}

fn f64_hex(value: f64) -> String {
    hex(&value.to_be_bytes())
}

fn rotation(y: f32, relative_y: &str, x: f32, relative_x: &str) -> String {
    format!("{}{relative_y}{}{relative_x}", f32_hex(y), f32_hex(x))
}

fn look_at(anchor: i64, at: [f64; 3], entity: Option<(i64, i64)>) -> String {
    let point: String = at.iter().map(|value| f64_hex(*value)).collect();
    let mut out = format!("{}{point}{}", varint_hex(anchor), flag(entity.is_some()));
    if let Some((id, to)) = entity {
        out += &format!("{}{}", varint_hex(id), varint_hex(to));
    }
    out
}

fn vehicle(at: [f64; 3], y: f32, x: f32) -> String {
    let point: String = at.iter().map(|value| f64_hex(*value)).collect();
    format!("{point}{}{}", f32_hex(y), f32_hex(x))
}

fn power(entity: i64, value: f64) -> String {
    format!("{}{}", varint_hex(entity), f64_hex(value))
}

/// The packet with these bytes, whole, then cut short by a byte and with one byte too many.
fn framed(packet: &str, label: &str, hex: String) -> Vec<Value> {
    let rows = [
        (label.to_owned(), hex.clone()),
        (format!("{label}-short"), hex[..hex.len() - 2].to_owned()),
        (format!("{label}-long"), format!("{hex}00")),
    ];
    rows.into_iter()
        .map(|(label, hex)| {
            case(format!("game/{packet}/{label}"), "wire", "play_packet", json!({"packet": packet, "hex": hex}))
        })
        .collect()
}

fn packets() -> Vec<Value> {
    let mut cases = Vec::new();
    for (label, hex) in [
        ("absolute", rotation(90.0, "00", -30.5, "00")),
        ("relative-yaw", rotation(-45.0, "01", 10.0, "00")),
        ("relative-pitch", rotation(12.0, "00", 95.0, "01")),
        ("both-relative", rotation(720.5, "01", -100.0, "01")),
        ("flag-two", rotation(1.0, "02", 2.0, "ff")),
    ] {
        cases.extend(framed("player_rotation", label, hex));
    }
    for (label, hex) in [
        ("feet", look_at(0, [1.5, 64.0, -3.25], None)),
        ("eyes", look_at(1, [0.0, 70.0, 0.0], None)),
        ("entity-feet", look_at(1, [4.0, 5.0, 6.0], Some((300, 0)))),
        ("entity-eyes", look_at(0, [-4.0, 5.0, 6.0], Some((1, 1)))),
        ("bad-from-anchor", look_at(2, [0.0; 3], None)),
        ("bad-to-anchor", look_at(0, [0.0; 3], Some((1, 2)))),
        ("negative-anchor", look_at(-1, [0.0; 3], None)),
        ("flag-two", format!("{}{}02{}{}", varint_hex(0), f64_hex(0.0).repeat(3), varint_hex(7), varint_hex(1))),
    ] {
        cases.extend(framed("player_look_at", label, hex));
    }
    for (label, hex) in
        [("plain", vehicle([10.5, 64.0, -20.25], 180.0, -5.0)), ("far", vehicle([3.0e7, -64.0, -3.0e7], -720.0, 90.0))]
    {
        cases.extend(framed("move_vehicle", label, hex));
    }
    for (label, hex) in [("plain", power(12, 0.1)), ("zero", power(1, 0.0)), ("big-id", power(2_097_151, -2.5))] {
        cases.extend(framed("projectile_power", label, hex));
    }
    cases
}

/// A label, the view the player has, and the packet's yaw, relative yaw, pitch, relative pitch.
type RotateRow = (&'static str, (f32, f32), (f32, bool, f32, bool));

/// A label, the player's feet, its eye height, from the eyes, and the point.
type LookRow = (&'static str, [f64; 3], f32, bool, [f64; 3]);

fn rotate_rows() -> Vec<Value> {
    // current yaw, pitch; the packet's yaw, relative, pitch, relative.
    let rows: [RotateRow; 8] = [
        ("absolute", (10.0, 5.0), (90.0, false, -30.0, false)),
        ("relative-yaw", (350.0, 5.0), (30.0, true, 0.0, false)),
        ("relative-pitch-clamped", (0.0, 80.0), (0.0, false, 30.0, true)),
        ("pitch-over-90", (0.0, 0.0), (0.0, false, 120.0, false)),
        ("pitch-wraps-at-360", (0.0, 0.0), (0.0, false, 400.0, false)),
        ("pitch-negative-wrap", (0.0, 0.0), (0.0, false, -400.0, false)),
        ("both-relative", (-170.0, -45.0), (-30.0, true, -60.0, true)),
        ("yaw-unwrapped", (0.0, 0.0), (1000.5, false, 0.0, false)),
    ];
    rows.into_iter()
        .map(|(label, current, (y, relative_y, x, relative_x))| {
            case(
                format!("game/rotate-player/{label}"),
                "wire",
                "rotate_player",
                json!({
                    "current_y": f32_bits(current.0), "current_x": f32_bits(current.1),
                    "y_rot": f32_bits(y), "relative_y": relative_y, "x_rot": f32_bits(x), "relative_x": relative_x,
                }),
            )
        })
        .collect()
}

fn look_rows() -> Vec<Value> {
    // the player's feet and eye height; the anchor; the point.
    let rows: [LookRow; 9] = [
        ("level-east", [0.0, 64.0, 0.0], 1.62, true, [10.0, 65.62, 0.0]),
        ("level-south", [0.0, 64.0, 0.0], 1.62, true, [0.0, 65.62, 10.0]),
        ("level-west", [0.0, 64.0, 0.0], 1.62, true, [-10.0, 65.62, 0.0]),
        ("level-north", [0.0, 64.0, 0.0], 1.62, true, [0.0, 65.62, -10.0]),
        ("down-and-away", [1.5, 70.0, -2.5], 1.62, true, [8.25, 60.0, 14.5]),
        ("straight-up", [0.0, 64.0, 0.0], 1.62, true, [0.0, 100.0, 0.0]),
        ("straight-down-feet", [0.0, 64.0, 0.0], 1.62, false, [0.0, 10.0, 0.0]),
        ("same-point", [3.0, 64.0, 3.0], 1.62, false, [3.0, 64.0, 3.0]),
        ("far-eyes", [-100.5, 20.0, 300.25], 1.62, true, [-97.0, 21.0, 296.0]),
    ];
    rows.into_iter()
        .map(|(label, from, eye_height, eyes, at)| {
            case(
                format!("game/look-at/{label}"),
                "wire",
                "look_at",
                json!({
                    "x": f64_bits(from[0]), "y": f64_bits(from[1]), "z": f64_bits(from[2]),
                    "eye_height": f32_bits(eye_height), "eyes": eyes,
                    "at_x": f64_bits(at[0]), "at_y": f64_bits(at[1]), "at_z": f64_bits(at[2]),
                }),
            )
        })
        .collect()
}

pub fn cases() -> Vec<Value> {
    let mut cases = packets();
    cases.extend(rotate_rows());
    cases.extend(look_rows());
    cases
}
