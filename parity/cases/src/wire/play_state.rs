//! Play packets that carry particles, an explosion or where the player spawns. Each layout of a
//! packet is a case of the version that has it: 26.2 and 26.3 write these packets differently.

use serde_json::{Value, json};

use super::codec::{flag, string_frame, varint_hex};
use crate::bits::{f32_bits, f64_bits, hex};
use crate::case::case;

fn f32_hex(value: f32) -> String {
    hex(&value.to_be_bytes())
}

fn f64_hex(value: f64) -> String {
    hex(&value.to_be_bytes())
}

fn int_hex(value: i32) -> String {
    hex(&value.to_be_bytes())
}

/// The packet's bytes whole, cut short by a byte and with one byte too many, for one version.
fn framed(version: &str, packet: &str, label: &str, bytes: String) -> Vec<Value> {
    let rows = [
        (label.to_owned(), bytes.clone()),
        (format!("{label}-short"), bytes[..bytes.len().saturating_sub(2)].to_owned()),
        (format!("{label}-long"), format!("{bytes}00")),
    ];
    rows.into_iter()
        .map(|(label, bytes)| {
            let mut row = case(
                format!("game/{packet}-{version}/{label}"),
                "wire",
                "play_packet",
                json!({"packet": packet, "hex": bytes}),
            );
            row["version"] = json!(version);
            row
        })
        .collect()
}

/// The particle types the rows use, by id (the ids before 26.3's additions are the same in
/// both versions).
const ANGRY_VILLAGER: i64 = 0;
const CLOUD: i64 = 11;
const CRIT: i64 = 13;
const EXPLOSION: i64 = 30;
const FLAME: i64 = 39;

/// What a particles packet carries besides its particle.
struct Body {
    flags: [&'static str; 2],
    at: [f64; 3],
    dist: [f32; 3],
}

fn body(body: &Body) -> String {
    let at: String = body.at.iter().map(|value| f64_hex(*value)).collect();
    let dist: String = body.dist.iter().map(|value| f32_hex(*value)).collect();
    format!("{}{}{at}{dist}", body.flags[0], body.flags[1])
}

const NEAR: Body = Body { flags: ["00", "00"], at: [1.5, 64.0, -3.25], dist: [0.5, 0.25, 1.0] };

/// 26.3's packet: the particle first, three speeds, the count a var int, then the randomization.
fn newer_particles(particle: i64, body: &Body, speeds: [f32; 3], count: i64, randomization: i64) -> String {
    let speeds: String = speeds.iter().map(|value| f32_hex(*value)).collect();
    format!("{}{}{speeds}{}{}", varint_hex(particle), self::body(body), varint_hex(count), varint_hex(randomization))
}

/// 26.2's packet: one speed and a four-byte count, then the particle.
fn older_particles(particle: i64, body: &Body, speed: f32, count: i32) -> String {
    format!("{}{}{}{}", self::body(body), f32_hex(speed), int_hex(count), varint_hex(particle))
}

fn particles() -> Vec<Value> {
    let mut cases = Vec::new();
    let flagged = Body { flags: ["01", "01"], ..NEAR };
    let flag_two = Body { flags: ["02", "ff"], ..NEAR };
    let newer = [
        ("one", newer_particles(FLAME, &NEAR, [0.5, 1.5, -2.0], 0, 0)),
        ("default", newer_particles(CRIT, &NEAR, [0.1, 0.2, 0.3], 5, 0)),
        ("alternative", newer_particles(CLOUD, &NEAR, [0.1, 0.2, 0.3], 3, 1)),
        ("alternative-with-speed", newer_particles(CLOUD, &NEAR, [1.0, 2.0, 3.0], 4, 2)),
        ("randomization-out-of-range", newer_particles(CRIT, &NEAR, [0.1, 0.2, 0.3], 2, 7)),
        ("randomization-negative", newer_particles(CRIT, &NEAR, [0.1, 0.2, 0.3], 2, -1)),
        ("negative-count", newer_particles(FLAME, &NEAR, [0.1, 0.1, 0.1], -3, 0)),
        ("long-count", newer_particles(FLAME, &NEAR, [0.0, 0.0, 0.0], 300, 1)),
        ("flags", newer_particles(ANGRY_VILLAGER, &flagged, [0.1, 0.1, 0.1], 1, 0)),
        ("flags-above-one", newer_particles(ANGRY_VILLAGER, &flag_two, [0.1, 0.1, 0.1], 1, 0)),
        ("unknown-particle", newer_particles(9999, &NEAR, [0.1, 0.1, 0.1], 1, 0)),
    ];
    for (label, bytes) in newer {
        cases.extend(framed("26.3", "level_particles", label, bytes));
    }
    let older = [
        ("one", older_particles(FLAME, &NEAR, 2.0, 0)),
        ("count", older_particles(CRIT, &NEAR, 0.25, 5)),
        ("negative-count", older_particles(FLAME, &NEAR, 0.25, -3)),
        ("big-count", older_particles(FLAME, &NEAR, 0.0, 100_000)),
        ("flags", older_particles(ANGRY_VILLAGER, &flagged, 1.0, 1)),
        ("flags-above-one", older_particles(ANGRY_VILLAGER, &flag_two, 1.0, 1)),
        ("unknown-particle", older_particles(9999, &NEAR, 1.0, 1)),
    ];
    for (label, bytes) in older {
        cases.extend(framed("26.2", "level_particles", label, bytes));
    }
    cases
}

/// An explosion packet without its trailing sound flag (26.2's); 26.3 appends `play_sound`.
fn explosion(knockback: Option<[f64; 3]>, kinds: &[(i64, f32, f32, i64)]) -> String {
    let mut out = format!("{}{}{}", f64_hex(2.5), f64_hex(64.0), f64_hex(-1.5));
    out += &format!("{}{}", f32_hex(4.0), int_hex(12));
    match knockback {
        Some(push) => {
            out += flag(true);
            out.extend(push.iter().map(|value| f64_hex(*value)));
        }
        None => out += flag(false),
    }
    // The particle at the centre, and the sound as a holder of a registered event.
    out += &varint_hex(EXPLOSION);
    out += &varint_hex(101);
    out += &varint_hex(kinds.len() as i64);
    for (particle, scaling, speed, weight) in kinds {
        out += &format!("{}{}{}{}", varint_hex(*particle), f32_hex(*scaling), f32_hex(*speed), varint_hex(*weight));
    }
    out
}

fn explosions() -> Vec<Value> {
    let mut cases = Vec::new();
    let rows = [
        ("plain", explosion(None, &[])),
        ("knockback", explosion(Some([0.5, 0.25, -0.125]), &[])),
        ("block-particles", explosion(None, &[(CLOUD, 0.5, 1.0, 1), (CRIT, 1.0, 2.0, 3)])),
    ];
    for (label, bytes) in &rows {
        cases.extend(framed("26.2", "explode", label, bytes.clone()));
        cases.extend(framed("26.3", "explode", &format!("{label}-sound"), format!("{bytes}{}", flag(true))));
        cases.extend(framed("26.3", "explode", &format!("{label}-silent"), format!("{bytes}{}", flag(false))));
    }
    cases.extend(framed("26.3", "explode", "flag-above-one", format!("{}02", rows[0].1)));
    cases
}

/// A spawn info: the dimension type, the dimension and seed, then `modes` (the bytes of the game
/// modes), the flags, a death place if given, the portal cooldown and the sea level.
fn spawn(modes: &str, death: Option<(&str, i64)>) -> String {
    let mut out =
        format!("{}{}{}", varint_hex(1), string_frame("the_nether"), hex(&0x0102_0304_0506_0708_i64.to_be_bytes()));
    out += &format!("{modes}{}{}", flag(true), flag(false));
    match death {
        Some((dimension, pos)) => {
            out += &format!("{}{}{}", flag(true), string_frame(dimension), hex(&pos.to_be_bytes()));
        }
        None => out += flag(false),
    }
    out + &varint_hex(42) + &varint_hex(32)
}

fn bytes_of(game: u8, previous: u8) -> String {
    hex(&[game, previous])
}

fn vars_of(game: i64, previous: i64) -> String {
    varint_hex(game) + &varint_hex(previous)
}

fn spawns() -> Vec<Value> {
    let mut cases = Vec::new();
    let death = Some(("the_end", -5_000_000_000));
    let older = [
        ("survival-none", bytes_of(0, 0xff), None),
        ("creative-after-survival", bytes_of(1, 0), None),
        ("spectator-after-adventure", bytes_of(3, 2), death),
        ("game-out-of-range", bytes_of(4, 0xff), None),
        ("game-negative", bytes_of(0xff, 1), None),
        ("previous-out-of-range", bytes_of(1, 4), None),
        ("previous-negative", bytes_of(1, 0xfe), None),
    ];
    for (label, modes, death) in older {
        let bytes = format!("{}00", spawn(&modes, death));
        cases.extend(framed("26.2", "respawn", label, bytes));
    }
    let newer = [
        ("survival-none", vars_of(0, 0), None),
        ("creative-after-survival", vars_of(1, 1), None),
        ("spectator-after-adventure", vars_of(3, 3), death),
        ("creative-after-creative", vars_of(1, 2), None),
        ("game-out-of-range", vars_of(4, 0), None),
        ("game-wide", vars_of(257, 0), None),
        ("game-negative", vars_of(-1, 0), None),
        ("previous-out-of-range", vars_of(1, 5), None),
        ("previous-wide", vars_of(1, 300), None),
        ("previous-negative", vars_of(1, -4), None),
    ];
    for (label, modes, death) in newer {
        let bytes = format!("{}03", spawn(&modes, death));
        cases.extend(framed("26.3", "respawn", label, bytes));
    }
    cases
}

/// A login packet around a spawn info: the player, the levels and the limits before it, the
/// flags after.
fn login(spawn: &str) -> String {
    let levels = format!("{}{}{}", varint_hex(2), string_frame("the_nether"), string_frame("overworld"));
    format!(
        "{}{}{levels}{}{}{}{}{}{}{spawn}{}{}",
        int_hex(17),
        flag(true),
        varint_hex(20),
        varint_hex(5),
        varint_hex(6),
        flag(false),
        flag(true),
        flag(false),
        flag(false),
        flag(true),
    )
}

fn logins() -> Vec<Value> {
    let mut cases = Vec::new();
    let death = Some(("overworld", 0x1234_5678));
    let older = login(&spawn(&bytes_of(1, 0), death));
    cases.extend(framed("26.2", "login", "creative-after-survival", older));
    let none = login(&spawn(&bytes_of(0, 0xff), None));
    cases.extend(framed("26.2", "login", "survival-none", none));
    let newer = login(&spawn(&vars_of(1, 1), death));
    cases.extend(framed("26.3", "login", "creative-after-survival", newer));
    let none = login(&spawn(&vars_of(0, 0), None));
    cases.extend(framed("26.3", "login", "survival-none", none));
    cases
}

/// A particle batch and an explosion in the packet's own units, for a handler row.
pub fn cases() -> Vec<Value> {
    let _ = (f32_bits(0.0), f64_bits(0.0));
    let mut cases = particles();
    cases.extend(explosions());
    cases.extend(spawns());
    cases.extend(logins());
    cases
}
