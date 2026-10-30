//! The named performance routes of `cargo xtask bench`: the settings the frame-time targets are
//! measured at, a world pinned to the same weather and time for every run, and the pig scene.

use std::time::Duration;

use serde_json::Value;

use super::args::Parsed;
use super::bench::STILL_SCRIPT;
use super::rcon;

/// The routes, in the order `--route perf` runs them.
pub const ROUTES: [&str; 3] = ["perf-still", "perf-run", "perf-pigs"];

/// What every named route is measured at.
pub const SIZE: [&str; 2] = ["1280", "720"];
pub const RENDER_DISTANCE: u32 = 32;

/// Seconds the client stands before the measured part starts: at render distance 32 the server
/// is still sending chunks for tens of seconds after the join, and the measured frames are of
/// the full view.
pub const LEAD_IN_SECONDS: u32 = 60;

/// The lead-in script: the client stands still and tells the frame limiter every second that it
/// is in use (`keep_awake` changes nothing on screen), because a client that gets no input for
/// a minute drops to the reference's 30 frames per second idle limit.
pub fn lead_in() -> String {
    "keep_awake=20;".repeat(LEAD_IN_SECONDS as usize)
}

/// The spectator's run: out and around in four legs, without `jump`, which climbs in spectator
/// and would take the route off the terrain.
pub const RUN_SCRIPT: &str = "none=200;forward,sprint=300;turn:90;forward,sprint=100;turn:90;\
                              forward,sprint=300;turn:90;forward,sprint=100;none=100";

/// The pigs: a square grid of this many on a side, this far apart, around the world's origin.
const PIGS_PER_SIDE: i32 = 40;
const PIG_SPACING: f64 = 1.5;
/// The chunks the grid stands in, forceloaded so the pigs exist while the player is far away.
const FORCELOAD: &str = "forceload add -40 -40 40 40";
/// The commands that clear the pigs with nothing left behind: the loot table is emptied when
/// they are summoned, so killing them drops no items.
const PIG_NBT: &str = "{NoAI:1b,PersistenceRequired:1b,DeathLootTable:\"minecraft:empty\"}";

/// The script of a named route: the lead-in, then the route itself.
pub fn script(route: &str) -> String {
    let route_script = if route == "perf-run" {
        RUN_SCRIPT
    } else {
        STILL_SCRIPT
    };
    format!("{}{route_script}", lead_in())
}

/// What a named route insists on: its own value, whatever was asked for.
pub fn forced(named: bool, asked: Option<&[String]>, own: &[String]) -> Vec<String> {
    match asked {
        Some(asked) if !named => asked.to_vec(),
        _ => own.to_vec(),
    }
}

pub fn render_distance(named: bool, parsed: &Parsed) -> Result<Option<u32>, String> {
    if named {
        return Ok(Some(RENDER_DISTANCE));
    }
    parsed
        .value("--render-distance")
        .map(|_| parsed.number("--render-distance", 0))
        .transpose()
}

/// The lead-in is the warmup of a named route; the others take `--warmup`.
pub fn warmup(named: bool, parsed: &Parsed) -> Result<f64, String> {
    if named {
        return Ok(f64::from(LEAD_IN_SECONDS) + 10.0);
    }
    parsed.number("--warmup", 10.0)
}

/// A run that failed leaves the world as it found it.
pub fn cleanup_on_error<T>(route: &str, run: Result<T, String>) -> Result<T, String> {
    if is_named(route) && run.is_err() {
        let _ = clear_pigs();
        let _ = restore_world();
    }
    run
}

/// After the runs of a named route: the world put back, and the reports checked.
pub fn finish(route: &str, reports: &[Value]) -> Result<(), String> {
    if !is_named(route) {
        return Ok(());
    }
    clear_pigs()?;
    restore_world()?;
    reports
        .iter()
        .try_for_each(|report| check_report(route, report))
}

pub fn is_named(route: &str) -> bool {
    ROUTES.contains(&route)
}

/// Whether the route is one of the named ones, or `perf` for all of them.
pub fn is_perf(route: &str) -> bool {
    route == "perf" || is_named(route)
}

/// The world as every named route finds it: clear weather at noon that stays so, new players in
/// spectator mode (so the route is repeatable and joins in the mode before the first frame), and
/// no mob drops.
fn pin_world() -> Result<(), String> {
    send(&[
        "gamerule advance_time false",
        "gamerule advance_weather false",
        "gamerule mob_drops false",
        "defaultgamemode spectator",
        "weather clear 1000000",
        "time set noon",
    ])
}

/// What `pin_world` changed, put back to the test server's usual values.
pub fn restore_world() -> Result<(), String> {
    send(&[
        "gamerule advance_time true",
        "gamerule advance_weather true",
        "gamerule mob_drops true",
        "defaultgamemode survival",
    ])
}

fn send(commands: &[&str]) -> Result<(), String> {
    let commands: Vec<String> = commands.iter().map(|&command| command.to_owned()).collect();
    rcon::send(&commands).map(|_| ())
}

/// The chunks around the origin the server generates once, so a run at this render distance
/// finds them on disk and measures the client, not the server's world generation: a square of
/// this many chunks on a side, taken in tiles the server loads one command at a time.
const GENERATED_SIDE: i32 = 72;
const TILE: i32 = 8;

/// Generates the chunks a render distance 32 view stands in. A marker keeps it to once per world.
fn generate_world() -> Result<(), String> {
    let marker = super::layout::data()
        .join("runs")
        .join("bench")
        .join("generated-rd32");
    if marker.exists() {
        return Ok(());
    }
    println!("generating the chunks around the origin once ({GENERATED_SIDE}x{GENERATED_SIDE})");
    let start = -(GENERATED_SIDE / 2);
    for tile_z in (start..start + GENERATED_SIDE).step_by(TILE as usize) {
        let mut commands = Vec::new();
        for tile_x in (start..start + GENERATED_SIDE).step_by(TILE as usize) {
            let (x0, z0) = (tile_x * 16, tile_z * 16);
            let (x1, z1) = (x0 + TILE * 16 - 1, z0 + TILE * 16 - 1);
            commands.push(format!("forceload add {x0} {z0} {x1} {z1}"));
            commands.push(format!("forceload remove {x0} {z0} {x1} {z1}"));
        }
        rcon::send(&commands)?;
    }
    std::fs::write(&marker, "").map_err(|error| format!("{}: {error}", marker.display()))
}

/// The scene a route is measured in, made before each run.
pub fn prepare(route: &str) -> Result<(), String> {
    generate_world()?;
    pin_world()?;
    clear_pigs()?;
    if route != "perf-pigs" {
        return Ok(());
    }
    let half = f64::from(PIGS_PER_SIDE - 1) * PIG_SPACING / 2.0;
    let mut commands = vec![FORCELOAD.to_owned()];
    for row in 0..PIGS_PER_SIDE {
        for column in 0..PIGS_PER_SIDE {
            let x = f64::from(column) * PIG_SPACING - half;
            let z = f64::from(row) * PIG_SPACING - half;
            commands.push(format!(
                "execute positioned {x} 100 {z} positioned over world_surface run summon \
                 minecraft:pig ~ ~ ~ {PIG_NBT}"
            ));
        }
    }
    rcon::send(&commands)?;
    // The summons are done when the server answers; the chunks they stand in follow.
    std::thread::sleep(Duration::from_secs(2));
    Ok(())
}

/// Takes the pigs and any item entity away, and checks that nothing of them is left: a later run
/// would otherwise draw a different scene.
pub fn clear_pigs() -> Result<(), String> {
    send(&[
        "kill @e[type=minecraft:pig]",
        "kill @e[type=minecraft:item]",
        "forceload remove all",
    ])?;
    let left = rcon::send(&[
        "execute if entity @e[type=minecraft:pig]".to_owned(),
        "execute if entity @e[type=minecraft:item]".to_owned(),
    ])?;
    if left.iter().any(|reply| !reply.contains("failed")) {
        return Err(format!("pigs or items are left in the world: {left:?}"));
    }
    Ok(())
}

/// The checks that say a report is of the scene the route names: the render distance filled
/// with chunks, and the pigs present in the pig scene only.
pub fn check_report(route: &str, report: &Value) -> Result<(), String> {
    let side = 2 * RENDER_DISTANCE + 1;
    let wanted = u64::from(side * side) * 85 / 100;
    let chunks = report["loaded_chunks"].as_u64().unwrap_or(0);
    // The run leaves the generated square's centre, so only the routes that stand are checked.
    if route != "perf-run" && chunks < wanted {
        return Err(format!(
            "{route}: {chunks} chunks loaded, fewer than {wanted}: the view is not render \
             distance {RENDER_DISTANCE}"
        ));
    }
    let entities = report["entities"].as_u64().unwrap_or(0);
    let pigs = u64::try_from(PIGS_PER_SIDE * PIGS_PER_SIDE).unwrap_or(0);
    if (route == "perf-pigs") != (entities >= pigs) {
        return Err(format!(
            "{route}: {entities} entities, but the scene should {} the {pigs} pigs",
            if route == "perf-pigs" {
                "have"
            } else {
                "not have"
            }
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_report_needs_the_render_distance_in_chunks() {
        let full = json!({"loaded_chunks": 3725, "entities": 10});
        assert!(check_report("perf-still", &full).is_ok());
        let capped = json!({"loaded_chunks": 1573, "entities": 10});
        assert!(check_report("perf-still", &capped).is_err());
    }

    #[test]
    fn the_pigs_are_in_the_pig_scene_only() {
        let pigs = json!({"loaded_chunks": 4225, "entities": 1700});
        assert!(check_report("perf-pigs", &pigs).is_ok());
        assert!(check_report("perf-still", &pigs).is_err());
        let none = json!({"loaded_chunks": 4225, "entities": 10});
        assert!(check_report("perf-pigs", &none).is_err());
    }
}
