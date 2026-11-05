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
pub(crate) mod pins;
mod records;
mod report;
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
    "bidi",
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
    "registry",
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

/// `cargo xtask parity [--version <v>] [--family <name>]... [--recapture]`, or
/// `parity report [--missing]`. The version defaults to the newest that has a reference.
pub fn run(args: &[String]) -> Result<(), String> {
    if args.first().map(String::as_str) == Some("report") {
        let all: Vec<String> = FAMILIES.iter().map(|family| (*family).to_owned()).collect();
        return report::run(&all, &args[1..]);
    }
    let mut recapture = false;
    let mut print_cases = false;
    let mut families = Vec::new();
    let mut version = None;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--recapture" => recapture = true,
            "--print-cases" => print_cases = true,
            "--version" => version = Some(rest.next().ok_or("--version needs a name")?.clone()),
            "--family" => families.push(rest.next().ok_or("--family needs a name")?.clone()),
            other => return Err(format!("unknown parity option `{other}`")),
        }
    }
    if families.is_empty() {
        families = FAMILIES.iter().map(|family| (*family).to_owned()).collect();
    }
    if print_cases {
        for case in cases::generate(&families, "", &|_| Ok(String::new()))? {
            println!("{}", case.line);
        }
        return Ok(());
    }
    compare(&families, recapture, version.as_deref())
}

/// The families whose cases or oracle sources `changed` touches: a file named for one family
/// (`parity/cases/src/<family>`, `tests/behavior/java/**/<Family>*Oracle.java`) runs that family,
/// anything shared runs all of them, and the coverage table (data only) runs none.
pub fn families_for(changed: &[String]) -> Vec<String> {
    let all = || {
        FAMILIES
            .iter()
            .map(|family| (*family).to_owned())
            .collect::<Vec<_>>()
    };
    let mut chosen = std::collections::BTreeSet::new();
    for path in changed {
        let family = if let Some(rest) = path.strip_prefix("parity/cases/src/") {
            if rest == "coverage.rs" {
                continue;
            }
            let stem = rest.split(['/', '.']).next().unwrap_or("");
            let stem = if stem == "block_placement" {
                "block"
            } else {
                stem
            };
            FAMILIES.iter().find(|family| **family == stem)
        } else if let Some(name) = path.strip_prefix("tests/behavior/java/") {
            let file = name.rsplit('/').next().unwrap_or("").to_lowercase();
            FAMILIES
                .iter()
                .find(|family| file.starts_with(*family) && file.ends_with("oracle.java"))
        } else if path.starts_with("tests/behavior/") || path.starts_with("parity/cases/") {
            None
        } else {
            continue;
        };
        match family {
            Some(family) => {
                chosen.insert((*family).to_owned());
            }
            None => return all(),
        }
    }
    chosen.into_iter().collect()
}

/// Every version that has a reference, oldest first.
pub fn versions() -> Vec<String> {
    pins::versions()
}

/// `changed` without the files that belong to a reference other than the newest (its pins and its
/// oracle classes), which only CI runs; those files come back second.
pub fn split_older(changed: Vec<String>) -> (Vec<String>, Vec<String>) {
    let newest = pins::newest_version().ok();
    let mut versions = pins::versions();
    versions.retain(|version| Some(version) != newest.as_ref());
    let own = |path: &str, version: &str| {
        path == format!("tests/behavior/reference/{version}.json")
            || path.starts_with(&format!("tests/behavior/java/v{}/", pins::tag(version)))
    };
    changed
        .into_iter()
        .partition(|path| !versions.iter().any(|version| own(path, version)))
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
pub fn compare(families: &[String], recapture: bool, version: Option<&str>) -> Result<(), String> {
    let started = Instant::now();
    let pins = match version {
        Some(version) => pins::Pins::load(version)?,
        None => pins::Pins::load_newest()?,
    };
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
    let stamp = pass_stamp(&cases)?;
    if !recapture && passed_before(&pins.version, &stamp) {
        println!(
            "parity: {} cases unchanged since the last pass, skipped",
            cases.len()
        );
        return Ok(());
    }
    let failures = adapter_failures(&cases, &store)?;
    let seconds = started.elapsed().as_secs_f32();
    if failures.is_empty() {
        println!("parity: {} cases passed in {seconds:.1}s", cases.len());
        remember_pass(&pins.version, &stamp);
        return Ok(());
    }
    let report = write_report(&pins.version, &cases, &failures)?;
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
fn write_report(
    version: &str,
    cases: &[Case],
    failures: &[(&Case, String)],
) -> Result<PathBuf, String> {
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
    let summary = json!({"version": version, "cases": cases.len(), "failures": rows});
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

/// What a pass depends on: every case key (the case lines, the oracle sources, the game and the
/// JDK) and the Rust adapter binary, built first so that a code change shows in it.
fn pass_stamp(cases: &[Case]) -> Result<String, String> {
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
    let binary = std::fs::read(process::root().join("target/release/photon-oracle"))
        .map_err(|error| format!("Rust adapter: {error}"))?;
    let keys: String = cases.iter().map(|case| format!("{}\n", case.key)).collect();
    Ok(digest(format!("{}\n{keys}", digest(&binary)).as_bytes()))
}

fn pass_file(version: &str) -> PathBuf {
    process::root()
        .join(".data/parity")
        .join(version)
        .join("passed")
}

/// Whether this exact stamp passed last time (any other run overwrites the file).
fn passed_before(version: &str, stamp: &str) -> bool {
    std::fs::read_to_string(pass_file(version))
        .is_ok_and(|saved| saved.lines().any(|line| line == stamp))
}

/// Keeps the stamps of the latest passes, so that a run of one family does not forget the full one.
fn remember_pass(version: &str, stamp: &str) {
    const KEPT: usize = 8;
    let mut stamps: Vec<String> = std::fs::read_to_string(pass_file(version))
        .map(|saved| saved.lines().map(str::to_owned).collect())
        .unwrap_or_default();
    stamps.retain(|saved| saved != stamp);
    stamps.push(stamp.to_owned());
    let skip = stamps.len().saturating_sub(KEPT);
    let _ = std::fs::write(pass_file(version), stamps[skip..].join("\n"));
}

#[cfg(test)]
mod tests {
    use super::families_for;

    fn files(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|path| (*path).to_owned()).collect()
    }

    #[test]
    fn a_family_file_runs_that_family_only() {
        let changed = files(&[
            "parity/cases/src/movement/particles/crack_rows.rs",
            "tests/behavior/java/v26_2/MovementParticleOracle.java",
            "parity/cases/src/block_placement/walls.rs",
        ]);
        assert_eq!(families_for(&changed), ["block", "movement"]);
    }

    #[test]
    fn shared_reference_files_run_every_family() {
        for shared in [
            "tests/behavior/java/common/Oracle.java",
            "tests/behavior/reference/26.2.json",
            "parity/cases/src/bits.rs",
        ] {
            assert_eq!(
                families_for(&files(&[shared])).len(),
                super::FAMILIES.len(),
                "{shared}"
            );
        }
    }

    #[test]
    fn a_version_is_spelled_with_an_underscore_in_oracle_files() {
        assert_eq!(super::pins::tag("26.2"), "26_2");
    }

    #[test]
    fn the_newest_supported_version_has_a_reference() {
        assert_eq!(
            super::pins::newest_version().as_deref(),
            Ok(photon_core::version::Version::NEWEST.name())
        );
    }

    #[test]
    fn the_only_reference_is_never_older() {
        let changed = files(&["tests/behavior/reference/26.2.json"]);
        let (own, older) = super::split_older(changed.clone());
        assert_eq!((own, older.len()), (changed, 0));
    }

    #[test]
    fn the_coverage_table_and_other_files_run_nothing() {
        assert!(
            families_for(&files(&[
                "parity/cases/src/coverage.rs",
                "docs/parity/README.md"
            ]))
            .is_empty()
        );
    }
}
