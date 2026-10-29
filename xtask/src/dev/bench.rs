//! `cargo xtask bench`: a fixed scripted run with a frame timing report.

use std::fs;
use std::net::{TcpStream, ToSocketAddrs};
use std::process::Command;
use std::time::{Duration, SystemTime};

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::args::{Parsed, Spec, parse};
use super::baseline;
use super::layout::{
    build_client, client_binary, client_environment, data, default_address, jar_assets, run_merged,
};
use crate::process;

/// Standing still and turning through the four directions, so every run draws
/// the same world from the same place. 40 seconds of ticks; the first seconds
/// (`PHOTON_BENCH_WARMUP`) are left out while the sections mesh.
const STILL_SCRIPT: &str =
    "none=200;turn:90;none=100;turn:90;none=100;turn:90;none=100;turn:90;none=300";
/// Sprinting out and turning, so chunks arrive, sections mesh and the
/// occlusion flood reruns while frames are measured: where frame spikes show.
const RUN_SCRIPT: &str = "none=200;forward,sprint,jump=300;turn:90;forward,sprint,jump=100;turn:90;\
                          forward,sprint,jump=300;turn:90;forward,sprint,jump=100;none=100";
/// Diving into the ground and flying level through the rock, turning at each leg: the
/// spectator's camera sits inside blocks, where the occlusion flood has smart culling off.
const DIVE_SCRIPT: &str = "none=200;pitch:60;forward,sprint=80;pitch:-60;forward,sprint=200;turn:90;\
                           forward,sprint=200;turn:90;forward,sprint=200;turn:90;forward,sprint=200;none=100";
/// The `void` route stands where nothing is built, as a skyblock map does: the spectator is
/// taken far above the terrain, so the frame costs what the engine costs when the world is empty.
const VOID_HEIGHT: i32 = 300;
/// A user of its own, so its saved position is the world's spawn and stays there.
/// The `run` route uses a new user per run instead (see [`fresh_user`]).
const STILL_USER: &str = "Bench";
/// The largest p50 change between consecutive runs that still counts as quiet.
const SPREAD_LIMIT_PERCENT: f64 = 5.0;

struct Options {
    address: String,
    name: String,
    repeat: u32,
    warmup: f64,
    size: Vec<String>,
    route: String,
    vsync_off: bool,
    fps_limit: Option<u32>,
    render_distance: Option<u32>,
    validation: bool,
    /// Keep the run as the baseline, or check it against the baseline.
    save: bool,
    compare: bool,
    threshold: f64,
}

const SPEC: Spec = Spec {
    flags: &["--no-build", "--validation", "--save", "--compare"],
    options: &[
        ("--name", 1),
        ("--repeat", 1),
        ("--warmup", 1),
        ("--size", 2),
        ("--route", 1),
        ("--vsync", 1),
        ("--fps-limit", 1),
        ("--render-distance", 1),
        ("--threshold", 1),
    ],
};

fn options(parsed: &Parsed) -> Result<Options, String> {
    if parsed.flag("--save") && parsed.flag("--compare") {
        return Err("--save and --compare are one or the other".to_owned());
    }
    let route = parsed.value("--route").unwrap_or("still").to_owned();
    if !["still", "run", "dive", "void"].contains(&route.as_str()) {
        return Err("--route is still, run, dive or void".to_owned());
    }
    let vsync = parsed.value("--vsync").unwrap_or("on");
    if !["on", "off"].contains(&vsync) {
        return Err("--vsync is on or off".to_owned());
    }
    let size = parsed.group("--size").map_or_else(
        || vec!["1280".to_owned(), "720".to_owned()],
        <[String]>::to_vec,
    );
    Ok(Options {
        address: parsed.address(&default_address()),
        name: parsed.value("--name").unwrap_or("bench").to_owned(),
        repeat: parsed.number("--repeat", 1)?,
        warmup: parsed.number("--warmup", 10.0)?,
        size,
        route,
        vsync_off: vsync == "off",
        fps_limit: parsed
            .value("--fps-limit")
            .map(|_| parsed.number("--fps-limit", 0))
            .transpose()?,
        render_distance: parsed
            .value("--render-distance")
            .map(|_| parsed.number("--render-distance", 0))
            .transpose()?,
        validation: parsed.flag("--validation"),
        save: parsed.flag("--save"),
        compare: parsed.flag("--compare"),
        threshold: parsed.number("--threshold", baseline::DEFAULT_THRESHOLD_PERCENT)?,
    })
}

fn server_listening(address: &str) -> bool {
    address
        .to_socket_addrs()
        .ok()
        .and_then(|mut addresses| addresses.next())
        .is_some_and(|socket| TcpStream::connect_timeout(&socket, Duration::from_secs(2)).is_ok())
}

fn number(report: &Value, pointer: &str) -> Option<f64> {
    report.pointer(pointer)?.as_f64()
}

fn stat(report: &Value, section: &str, name: &str) -> f64 {
    number(report, &format!("/{section}/{name}")).unwrap_or(f64::NAN)
}

fn triple(report: &Value, section: &str) -> String {
    format!(
        "{:.2}/{:.2}/{:.2}",
        stat(report, section, "p50"),
        stat(report, section, "p95"),
        stat(report, section, "p99")
    )
}

fn gpu_text(report: &Value) -> String {
    let present = report["gpu_ms"]
        .as_object()
        .is_some_and(|gpu| !gpu.is_empty());
    if !present {
        return " gpu=unavailable".to_owned();
    }
    let resolved = report["gpu_ms"]["pass_breakdown_resolved"]
        .as_bool()
        .unwrap_or(false);
    let passes = if resolved {
        ""
    } else {
        " (no per-pass breakdown on this driver)"
    };
    format!(" gpu_scene={}{passes}", triple(report, "gpu_ms/scene"))
}

fn parts_text(report: &Value) -> String {
    let Some(parts) = report["parts_ms"].as_object() else {
        return String::new();
    };
    let mut heaviest: Vec<(&String, f64)> = parts
        .iter()
        .map(|(name, stats)| (name, stats["max"].as_f64().unwrap_or(0.0)))
        .collect();
    heaviest.sort_by(|a, b| b.1.total_cmp(&a.1));
    heaviest.truncate(3);
    if heaviest.is_empty() {
        return String::new();
    }
    let listed: Vec<String> = heaviest
        .iter()
        .map(|(name, max)| format!("{name}:{max:.1}"))
        .collect();
    format!(" max_parts={}", listed.join(","))
}

/// One line: the frame's work and the GPU's scene time, as p50/p95/p99 milliseconds.
fn summary(report: &Value) -> String {
    format!(
        "frames={} work_ms={} max={:.2} work_fps={:.0} wall_fps={:.1}{}{}",
        report["frames"],
        triple(report, "work_ms"),
        stat(report, "work_ms", "max"),
        number(report, "/work_fps").unwrap_or(f64::NAN),
        number(report, "/wall_fps").unwrap_or(f64::NAN),
        gpu_text(report),
        parts_text(report)
    ) + &pacing_text(report)
}

/// The worst one-second window, the intervals past 1/240 s and 1/230 s, and the blinks.
fn pacing_text(report: &Value) -> String {
    let pacing = &report["pacing"];
    if pacing.is_null() {
        return String::new();
    }
    format!(
        " worst_second_fps={} longest_ms={} over240={}/{} over230={} blinks={}",
        pacing["worst_second_fps"],
        pacing["longest_interval_ms"],
        pacing["intervals_over_240"],
        pacing["intervals"],
        pacing["intervals_over_230"],
        report["blinks"]
    )
}

/// A name no earlier run used, so its saved position is the world's spawn.
fn fresh_user() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_nanos());
    let digest = Sha256::digest(format!("{now}{}", std::process::id()));
    format!(
        "Run{}",
        digest
            .iter()
            .take(6)
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    )
}

/// Puts the bench's player in spectator mode once it has joined: the command is repeated for
/// the seconds the join may take, and an unknown player is no error for the server.
fn spectate(user: &str, high: bool) {
    let mut commands = vec![format!("gamemode spectator {user}")];
    if high {
        commands.push(format!("tp {user} 17 {VOID_HEIGHT} -35"));
    }
    std::thread::spawn(move || {
        for _ in 0..40 {
            std::thread::sleep(Duration::from_millis(500));
            for command in &commands {
                let _ = super::rcon::send(std::slice::from_ref(command));
            }
        }
    });
}

/// The client run that writes its report to `path`.
fn client_command(
    options: &Options,
    path: &std::path::Path,
    user: String,
    script: &str,
) -> Result<Command, String> {
    let mut command = Command::new(client_binary());
    command
        .current_dir(process::root())
        .envs(client_environment()?)
        .env("PHOTON_USER", user)
        .env("PHOTON_SCRIPT", script)
        .env("PHOTON_BENCH", path)
        .env("PHOTON_BENCH_WARMUP", options.warmup.to_string())
        .env("PHOTON_VSYNC", if options.vsync_off { "0" } else { "1" });
    if options.validation {
        command.env("PHOTON_VALIDATION", "1");
    }
    if let Some(distance) = options.render_distance {
        command.env("PHOTON_RENDER_DISTANCE", distance.to_string());
    }
    if let Some(limit) = options.fps_limit {
        command.env("PHOTON_MAX_FPS", limit.to_string());
    }
    command
        .arg("play")
        .arg(&options.address)
        .args(&options.size);
    Ok(command)
}

fn run_once(options: &Options, number: u32) -> Result<Value, String> {
    let directory = data().join("runs").join("bench");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let path = directory.join(format!("{}-{number}.json", options.name));
    let _ = fs::remove_file(&path);
    let (user, script) = if options.route == "run" {
        (fresh_user(), RUN_SCRIPT)
    } else if options.route == "dive" {
        (fresh_user(), DIVE_SCRIPT)
    } else if options.route == "void" {
        (fresh_user(), STILL_SCRIPT)
    } else {
        (STILL_USER.to_owned(), STILL_SCRIPT)
    };
    // A spectator flies through trees and water, so the run measures the engine, not the terrain.
    spectate(&user, options.route == "void");
    let command = client_command(options, &path, user, script)?;
    let (status, output) = run_merged(command)?;
    if !status.success() || !path.exists() {
        let lines: Vec<&str> = output.lines().collect();
        println!("{}", lines[lines.len().saturating_sub(15)..].join("\n"));
        return Err(format!(
            "bench run {number} failed ({status}); no report at {}",
            path.display()
        ));
    }
    let log = path.with_extension("log");
    fs::write(&log, &output).map_err(|error| format!("{}: {error}", log.display()))?;
    let text = fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let report: Value =
        serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
    println!("run {number}: {}", summary(&report));
    if options.validation {
        let findings = &report["validation"];
        println!(
            "validation: {} errors, {} warnings (see {})",
            findings["errors"],
            findings["warnings"],
            log.display()
        );
    }
    Ok(report)
}

/// The frame is pinned at the display's refresh time under FIFO, so its p50
/// alone proves little: the waits, the meshing span and the GPU's scene time
/// are compared too.
fn measures(report: &Value) -> [(&'static str, Option<f64>); 4] {
    let gpu_present = report["gpu_ms"]
        .as_object()
        .is_some_and(|gpu| !gpu.is_empty());
    [
        ("work_ms", number(report, "/work_ms/p50")),
        ("fence_wait", number(report, "/cpu_ms/fence_wait/p50")),
        ("mesh", number(report, "/cpu_ms/mesh/p50")),
        (
            "gpu_scene",
            if gpu_present {
                number(report, "/gpu_ms/scene/p50")
            } else {
                None
            },
        ),
    ]
}

/// Whether consecutive runs agree within the limit; prints each comparison.
fn runs_agree(reports: &[Value]) -> bool {
    let mut agree = true;
    for pair in reports.windows(2) {
        let mut changes = Vec::new();
        for ((name, old), (_, new)) in measures(&pair[0]).into_iter().zip(measures(&pair[1])) {
            if let (Some(old), Some(new)) = (old, new)
                && old != 0.0
            {
                changes.push((name, (new - old).abs() / old * 100.0));
            }
        }
        let listed: Vec<String> = changes
            .iter()
            .map(|(name, change)| format!("{name} {change:.1}%"))
            .collect();
        println!("p50 change between consecutive runs: {}", listed.join(", "));
        agree = agree
            && changes
                .iter()
                .all(|(_, change)| *change < SPREAD_LIMIT_PERCENT);
    }
    agree
}

/// Where the baseline of a route is kept.
fn baseline_path(route: &str) -> std::path::PathBuf {
    data()
        .join("runs")
        .join("bench")
        .join(format!("baseline-{route}.json"))
}

/// `--save` keeps the middle run as the baseline; `--compare` fails when its p99 is past the threshold.
fn keep_or_check(options: &Options, reports: &[Value]) -> Result<(), String> {
    let Some(report) =
        baseline::representative(reports).filter(|_| options.save || options.compare)
    else {
        return Ok(());
    };
    let path = baseline_path(&options.route);
    if options.save {
        baseline::save(&path, &options.route, report)?;
        println!("baseline saved to {}", path.display());
        return Ok(());
    }
    let (lines, within) = baseline::compare(&baseline::load(&path)?, report, options.threshold);
    println!("{}", lines.join("\n"));
    if within {
        Ok(())
    } else {
        Err(format!(
            "the p99 work time is more than {}% above the baseline",
            options.threshold
        ))
    }
}

pub fn bench(args: &[String]) -> Result<(), String> {
    let parsed = parse(args, &SPEC)?;
    let options = options(&parsed)?;
    if !server_listening(&options.address) {
        return Err(format!(
            "no server on {}: run `cargo xtask server --accept-eula --background`",
            options.address
        ));
    }
    jar_assets()?;
    if !parsed.flag("--no-build") {
        build_client(Some("photon"))?;
    }
    if options.repeat == 0 {
        return Err("--repeat is at least 1".to_owned());
    }
    let mut reports = Vec::new();
    for number in 1..=options.repeat {
        reports.push(run_once(&options, number)?);
    }
    let first = &reports[0];
    println!(
        "{} {}x{} present={} render_distance={} chunks={} entities={}",
        first["device"].as_str().unwrap_or("?"),
        first["extent"][0],
        first["extent"][1],
        first["present_mode"].as_str().unwrap_or("?"),
        first["render_distance"],
        first["loaded_chunks"],
        first["entities"]
    );
    if runs_agree(&reports) {
        keep_or_check(&options, &reports)
    } else {
        Err(
            "a p50 moved 5% or more between runs: the machine was not quiet enough to compare"
                .to_owned(),
        )
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn report(work_p50: f64, gpu: bool) -> Value {
        let gpu_ms = if gpu {
            json!({"scene": {"p50": 3.0, "p95": 4.0, "p99": 5.0}, "pass_breakdown_resolved": false})
        } else {
            json!({})
        };
        json!({
            "frames": 1200,
            "work_ms": {"p50": work_p50, "p95": 9.5, "p99": 12.0, "max": 20.25},
            "work_fps": 123.4,
            "wall_fps": 59.96,
            "gpu_ms": gpu_ms,
            "cpu_ms": {"fence_wait": {"p50": 1.0}, "mesh": {"p50": 2.0}},
            "parts_ms": {"light": {"max": 1.5}, "mesh": {"max": 7.25}, "sky": {"max": 0.5}, "fog": {"max": 3.0}},
        })
    }

    #[test]
    fn the_summary_names_the_work_gpu_and_heaviest_parts() {
        assert_eq!(
            summary(&report(8.0, true)),
            "frames=1200 work_ms=8.00/9.50/12.00 max=20.25 work_fps=123 wall_fps=60.0 \
             gpu_scene=3.00/4.00/5.00 (no per-pass breakdown on this driver) max_parts=mesh:7.2,fog:3.0,light:1.5"
        );
        assert!(summary(&report(8.0, false)).contains(" gpu=unavailable"));
    }

    #[test]
    fn runs_within_five_percent_agree() {
        assert!(runs_agree(&[report(8.0, true), report(8.3, true)]));
        assert!(!runs_agree(&[report(8.0, true), report(8.5, true)]));
    }

    #[test]
    fn a_missing_gpu_section_is_left_out_of_the_comparison() {
        assert!(runs_agree(&[report(8.0, false), report(8.0, true)]));
    }
}
