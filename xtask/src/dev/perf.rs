//! The named performance routes of `cargo xtask bench`: the settings the frame-time targets are
//! measured at, a world pinned to the same weather and time for every run, and the pig scene.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use serde_json::Value;

use super::args::Parsed;
use super::bench::STILL_SCRIPT;
use super::busy;
use super::rcon;

/// The routes, in the order `--route perf` runs them.
pub const ROUTES: [&str; 4] = ["perf-still", "perf-run", "perf-pigs", "perf-busy"];

/// What every named route is measured at.
pub const SIZE: [&str; 2] = ["1280", "720"];
pub const RENDER_DISTANCE: u32 = 32;
/// Where the still and pig routes hold the camera, as the position and rotation of a `tp`: on
/// the summit the world spawns at, over the middle of the pig grid, looking east. The grid is
/// built on the surface there (about y 198), so a camera lower down would never see a pig.
pub const STILL_SPOT: &str = "0.5 198.5 0.5 -90 0";

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
pub const RUN_SCRIPT: &str = "forward,sprint=100;turn:90;forward,sprint=100;turn:90;\
                              forward,sprint=100;turn:90;forward,sprint=100;turn:90;\
                              forward,sprint=100;turn:90;forward,sprint=100;turn:90;\
                              forward,sprint=100;turn:90;forward,sprint=100;turn:90";

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

/// The p50 change between runs that still counts as quiet: the run route meets the chunks the
/// server sends at slightly different moments, so its GPU and meshing spans move more.
pub fn spread_limit(route: &str, usual: f64) -> f64 {
    if route == "perf-run" {
        usual * 2.0
    } else {
        usual
    }
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
        "gamerule spawn_mobs false",
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
        "gamerule spawn_mobs true",
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
const GENERATED_SIDE: i32 = 88;
const TILE: i32 = 8;

/// Generates the chunks a render distance 32 view stands in. A marker keeps it to once per world.
fn generate_world() -> Result<(), String> {
    let marker = super::layout::data()
        .join("runs")
        .join("bench")
        .join(format!("generated-{GENERATED_SIDE}"));
    if marker.exists() {
        return Ok(());
    }
    println!("generating the chunks around the origin once ({GENERATED_SIDE}x{GENERATED_SIDE})");
    let start = -(GENERATED_SIDE / 2);
    for tile_z in (start..start + GENERATED_SIDE).step_by(TILE as usize) {
        for tile_x in (start..start + GENERATED_SIDE).step_by(TILE as usize) {
            let (x0, z0) = (tile_x * 16, tile_z * 16);
            let (x1, z1) = (x0 + TILE * 16 - 1, z0 + TILE * 16 - 1);
            send(&[&format!("forceload add {x0} {z0} {x1} {z1}")])?;
            // World generation places the animals of a biome, which would stand in every
            // scene; the server finds an entity once its chunk's entities have loaded.
            std::thread::sleep(Duration::from_millis(800));
            send(&[
                "kill @e[type=!minecraft:player]",
                &format!("forceload remove {x0} {z0} {x1} {z1}"),
            ])?;
        }
    }
    std::fs::write(&marker, "").map_err(|error| format!("{}: {error}", marker.display()))
}

/// The scene of a run while it lasts: a thread keeps every entity near the player but the
/// player (and, in the pig scene, the pigs) killed, because the generated world is full of
/// animals the server loads as the player comes near; and in the pig scene it summons the pigs
/// shortly before the measured part starts. Dropping the scene stops the thread.
pub struct Scene {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Drop for Scene {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// The world pinned and emptied, and the thread that keeps the scene as the route names it.
pub fn start(route: &str) -> Result<Scene, String> {
    generate_world()?;
    pin_world()?;
    clear_pigs()?;
    let route = route.to_owned();
    let stop = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&stop);
    let thread = std::thread::spawn(move || {
        let begun = Instant::now();
        let mut summoned = false;
        while !flag.load(Ordering::Relaxed) {
            if !summoned && begun.elapsed() > SUMMON_AT {
                summoned = summon_scene(&route);
            }
            let _ = send(&[&kill_command(&route)]);
            if route == "perf-busy" && summoned {
                let _ = send(&busy::PARTICLES);
            }
            std::thread::sleep(Duration::from_secs(if route == "perf-busy" {
                1
            } else {
                2
            }));
        }
    });
    Ok(Scene {
        stop,
        thread: Some(thread),
    })
}

/// Whether the summoning of the route's own scene went through; a route without one is done.
fn summon_scene(route: &str) -> bool {
    match route {
        "perf-pigs" => summon_pigs().is_ok(),
        "perf-busy" => rcon::send(&busy::summon()).is_ok(),
        _ => true,
    }
}

/// What the scene thread kills every round: everything near the player but the route's own scene.
fn kill_command(route: &str) -> String {
    match route {
        "perf-pigs" => "kill @e[type=!minecraft:player,type=!minecraft:pig]".to_owned(),
        "perf-busy" => format!("kill @e[type=!minecraft:player,tag=!{}]", busy::TAG),
        _ => "kill @e[type=!minecraft:player]".to_owned(),
    }
}

/// When, after the run starts, the pigs are summoned: before the measured part, after the
/// client has joined and its chunks have arrived.
const SUMMON_AT: Duration = Duration::from_secs(LEAD_IN_SECONDS as u64 - 15);

fn summon_pigs() -> Result<(), String> {
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
    rcon::send(&commands).map(|_| ())
}

/// Takes the pigs and any item entity away, and checks that nothing of them is left: a later run
/// would otherwise draw a different scene. The chunks they stand in are loaded first, because
/// the server only finds the entities of loaded chunks.
pub fn clear_pigs() -> Result<(), String> {
    send(&[FORCELOAD, busy::FORCELOAD])?;
    std::thread::sleep(Duration::from_secs(2));
    let cleared = rcon::send(&busy::clear())?;
    if let Some(refused) = cleared.iter().find(|reply| reply.contains("Too many")) {
        return Err(format!("the busy scene could not be cleared: {refused}"));
    }
    let mut left = Vec::new();
    for _ in 0..5 {
        send(&[
            "kill @e[type=minecraft:pig]",
            "kill @e[type=minecraft:item]",
        ])?;
        left = rcon::send(&[
            "execute if entity @e[type=minecraft:pig]".to_owned(),
            "execute if entity @e[type=minecraft:item]".to_owned(),
        ])?;
        if left.iter().all(|reply| reply.contains("failed")) {
            return send(&["forceload remove all"]);
        }
    }
    Err(format!("pigs or items are left in the world: {left:?}"))
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
    if route == "perf-busy" {
        return check_busy(report, entities);
    }
    // A few pigs are lost to the summon (a spot over water, a collision): most must be there.
    let pigs = u64::try_from(PIGS_PER_SIDE * PIGS_PER_SIDE).unwrap_or(0) * 9 / 10;
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
    // The pigs only count when the camera sees a good part of them: a run that stands below or
    // beside the grid draws almost none and measures an empty scene.
    let drawn = report["drawn_entities"].as_f64().unwrap_or(0.0);
    if route == "perf-pigs" && drawn < MIN_DRAWN_PIGS {
        return Err(format!(
            "{route}: the frames draw {drawn:.0} entities on average, fewer than \
             {MIN_DRAWN_PIGS}: the pigs are not in view"
        ));
    }
    // Animals left on disk by an earlier run load as the player arrives and cost the GPU about
    // twice what the empty scene does, so a world that still holds them is not a baseline.
    if route != "perf-pigs" && entities > STRAY_ENTITIES {
        return Err(format!(
            "{route}: {entities} entities are left in the world, more than {STRAY_ENTITIES}"
        ));
    }
    Ok(())
}

/// The busy scene: most of its entities are there and the camera sees a good part of them.
fn check_busy(report: &Value, entities: u64) -> Result<(), String> {
    let wanted = u64::try_from(busy::expected_entities()).unwrap_or(0) * 9 / 10;
    let drawn = report["drawn_entities"].as_f64().unwrap_or(0.0);
    if entities < wanted || drawn < MIN_DRAWN_BUSY {
        return Err(format!(
            "perf-busy: {entities} entities (at least {wanted} wanted), {drawn:.0} drawn per \
             frame (at least {MIN_DRAWN_BUSY} wanted)"
        ));
    }
    Ok(())
}

/// The entities of the busy scene the camera must see on average.
const MIN_DRAWN_BUSY: f64 = 100.0;

/// The pigs the camera must see on average for a run to be the pig scene.
const MIN_DRAWN_PIGS: f64 = 300.0;

/// Entities a route without pigs may still see: a few players' worth of strays at most.
const STRAY_ENTITIES: u64 = 50;

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
        let pigs = json!({"loaded_chunks": 4225, "entities": 1700, "drawn_entities": 600.0});
        assert!(check_report("perf-pigs", &pigs).is_ok());
        assert!(check_report("perf-still", &pigs).is_err());
        let none = json!({"loaded_chunks": 4225, "entities": 10});
        assert!(check_report("perf-pigs", &none).is_err());
    }

    #[test]
    fn pigs_the_view_leaves_out_are_no_pig_scene() {
        let unseen = json!({"loaded_chunks": 4225, "entities": 1700, "drawn_entities": 0.0});
        assert!(check_report("perf-pigs", &unseen).is_err());
    }

    #[test]
    fn strays_left_in_the_world_fail_a_route_without_pigs() {
        let strays = json!({"loaded_chunks": 4225, "entities": 778});
        assert!(check_report("perf-still", &strays).is_err());
        assert!(check_report("perf-run", &strays).is_err());
    }
}
