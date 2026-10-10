//! `cargo xtask parity`: the Java/Rust behaviour comparison.
//!
//! Every case is keyed by its content and environment. Java outputs for
//! keys the store lacks are captured automatically (each family in its own
//! JVM, run twice, families in parallel); then the Rust adapter answers every
//! selected case and the answers must equal the captures exactly. Nothing is
//! written for a passing run; a failing run leaves a report under
//! `.data/parity/failures/`.

mod capture;
mod cases;
mod java;
mod pins;
mod records;
mod store;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::process;
use cases::Case;

/// Every parity family.
pub const FAMILIES: &[&str] = &[
    "atlas",
    "binary",
    "block",
    "entity",
    "gui",
    "image",
    "json",
    "math",
    "model",
    "movement",
    "nbt",
    "resources",
    "sound",
    "timer",
    "utf8",
    "window",
    "wire",
];
const ADAPTER_TIMEOUT: Duration = Duration::from_secs(600);
const KEPT_FAILURES: usize = 5;

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// `cargo xtask parity [--family <name>]... [--recapture]`
pub fn run(args: &[String]) -> Result<(), String> {
    let mut recapture = false;
    let mut families = Vec::new();
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--recapture" => recapture = true,
            "--family" => families.push(rest.next().ok_or("--family needs a name")?.clone()),
            other => return Err(format!("unknown parity option `{other}`")),
        }
    }
    if families.is_empty() {
        families = FAMILIES.iter().map(|family| (*family).to_owned()).collect();
    }
    compare(&families, recapture)
}

/// Whether the local Java reference needed for parity is present.
pub fn reference_present() -> bool {
    runtime().is_dir()
}

fn runtime() -> PathBuf {
    process::root().join(".data/minecraft/runtime")
}

fn assets() -> PathBuf {
    process::root().join(".data/minecraft/assets/jar/assets")
}

/// Captures what is missing, then compares the adapter on `families`.
pub fn compare(families: &[String], recapture: bool) -> Result<(), String> {
    let started = Instant::now();
    let pins = pins::Pins::load()?;
    let classpath = pins.classpath(&runtime())?;
    let jdk = java::Jdk::find(pins.java_major)?;
    let environment = environment(&pins, &jdk);
    let cases = cases::generate(families, &environment, &|family| pins.family_digest(family))?;
    let mut store = store::Store::open(&pins.version);
    store.load(families)?;
    let missing: Vec<&Case> = cases
        .iter()
        .filter(|case| recapture || store.get(&case.key).is_none())
        .collect();
    if !missing.is_empty() {
        println!("parity: capturing {} cases from Java", missing.len());
        let joined = join_paths(&classpath);
        let classes = jdk.compile(&pins.all_sources()?, &joined)?;
        let oracle = capture::Oracle {
            jdk: &jdk,
            classes: &classes,
            classpath: &joined,
            assets: &assets(),
        };
        for (family, key, record) in capture::capture(&oracle, &missing)? {
            store.insert(&family, key, record);
        }
    }
    store.save(&keys_in_use(&cases))?;
    let failures = adapter_failures(&cases, &store)?;
    let seconds = started.elapsed().as_secs_f32();
    if failures.is_empty() {
        println!("parity: {} cases passed in {seconds:.1}s", cases.len());
        return Ok(());
    }
    let report = write_report(&cases, &failures)?;
    for (case, difference) in failures.iter().take(10) {
        eprintln!("  {}: {difference}", case.id);
    }
    Err(format!(
        "parity: {} of {} cases differ ({seconds:.1}s); report in {}",
        failures.len(),
        cases.len(),
        report.display()
    ))
}

/// What every key shares: the game, the libraries, the JDK and the machine.
fn environment(pins: &pins::Pins, jdk: &java::Jdk) -> String {
    let libraries: BTreeMap<String, &str> = pins
        .library_keys()
        .into_iter()
        .filter_map(|key| {
            pins.libraries
                .get(&key)
                .map(|library| (key, library.sha1.as_str()))
        })
        .collect();
    json!({
        "schema": 2, "version": pins.version, "protocol": pins.protocol, "client": pins.client_sha1,
        "libraries": libraries, "jdk": jdk.versions, "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH, "jvm_flags": java::JVM_FLAGS,
    })
    .to_string()
}

fn join_paths(paths: &[PathBuf]) -> String {
    let separator = if cfg!(windows) { ";" } else { ":" };
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(separator)
}

fn keys_in_use(cases: &[Case]) -> BTreeMap<String, Vec<String>> {
    let mut in_use: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for case in cases {
        let keys = in_use.entry(case.family.clone()).or_default();
        if !keys.contains(&case.key) {
            keys.push(case.key.clone());
        }
    }
    in_use
}

/// Runs the adapter on every case and returns those whose answer differs.
fn adapter_failures<'a>(
    cases: &'a [Case],
    store: &store::Store,
) -> Result<Vec<(&'a Case, String)>, String> {
    process::run(
        "cargo",
        &[
            "build",
            "--quiet",
            "--release",
            "--package",
            "photon-oracle",
        ],
    )?;
    let mut command =
        std::process::Command::new(process::root().join("target/release/photon-oracle"));
    command.env("PHOTON_ASSETS", assets());
    let input: String = cases
        .iter()
        .map(|case| format!("{}\n", case.line))
        .collect();
    let raw = java::run_with_timeout(command, &input, ADAPTER_TIMEOUT)
        .map_err(|error| format!("Rust adapter: {error}"))?;
    let all: Vec<&Case> = cases.iter().collect();
    let answers = records::parse(&raw, &all)?;
    let mut failures = Vec::new();
    for (case, answer) in cases.iter().zip(&answers) {
        let expected = store
            .get(&case.key)
            .ok_or_else(|| format!("no capture for {}", case.id))?;
        if let Some(difference) =
            records::first_difference(&expected["result"], &answer["result"], "result")
        {
            failures.push((case, difference));
        }
    }
    Ok(failures)
}

/// Writes the failing cases with their inputs and differences; keeps the
/// newest few reports only.
fn write_report(cases: &[Case], failures: &[(&Case, String)]) -> Result<PathBuf, String> {
    let base = process::root().join(".data/parity/failures");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let dir = base.join(stamp.to_string());
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let rows: Vec<Value> = failures
        .iter()
        .map(|(case, difference)| {
            let input: Value = serde_json::from_str(&case.line).unwrap_or(Value::Null);
            json!({"case_id": case.id, "family": case.family, "difference": difference, "input": input})
        })
        .collect();
    let summary = json!({"cases": cases.len(), "failures": rows});
    let text = serde_json::to_string_pretty(&summary).map_err(|error| error.to_string())?;
    std::fs::write(dir.join("failures.json"), text).map_err(|error| error.to_string())?;
    prune(&base);
    Ok(dir)
}

fn prune(base: &std::path::Path) {
    let Ok(entries) = std::fs::read_dir(base) else {
        return;
    };
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    dirs.sort();
    let excess = dirs.len().saturating_sub(KEPT_FAILURES);
    for old in dirs.into_iter().take(excess) {
        let _ = std::fs::remove_dir_all(old);
    }
}
