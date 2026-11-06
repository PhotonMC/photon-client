//! The same-process version switch check ([ADR 0013](../../../docs/adr/0013-multi-version-content-model.md)):
//! the adapter answers the cases of several versions in one process, switching between them (and
//! back) the way the client does between play sessions, and every answer must equal the one a
//! fresh process gives for that version.

use crate::process;

use super::{ADAPTER_TIMEOUT, assets, cases, java};

/// The adapter's answers to `input`, one line each, started as the version `first`.
fn answers(first: &str, input: &str) -> Result<Vec<String>, String> {
    let mut command =
        std::process::Command::new(process::root().join("target/release/photon-oracle"));
    command.env("PHOTON_ASSETS", assets(first));
    command.env("PHOTON_ORACLE_VERSION", first);
    let raw = java::run_with_timeout(command, input, ADAPTER_TIMEOUT)
        .map_err(|error| format!("Rust adapter: {error}"))?;
    Ok(raw.lines().map(str::to_owned).collect())
}

/// What each version's cases are and what a fresh process answers to them.
struct Fresh {
    lines: Vec<Vec<String>>,
    answers: Vec<Vec<String>>,
}

fn fresh_runs(families: &[String], versions: &[String]) -> Result<Fresh, String> {
    let mut fresh = Fresh {
        lines: Vec::new(),
        answers: Vec::new(),
    };
    for version in versions {
        let cases = cases::generate(families, "", &|_| Ok(String::new()), Some(version))?;
        let own: Vec<String> = cases.into_iter().map(|case| case.line).collect();
        fresh.answers.push(answers(version, &own.join("\n"))?);
        fresh.lines.push(own);
    }
    Ok(fresh)
}

/// The cases of each version in `order`, a switch line before each batch after the first.
fn switched_input(lines: &[Vec<String>], versions: &[String], order: &[usize]) -> String {
    let mut input = String::new();
    for (position, &which) in order.iter().enumerate() {
        if position > 0 {
            input.push_str(&format!("{{\"switch_version\":\"{}\"}}\n", versions[which]));
        }
        input.push_str(&lines[which].join("\n"));
        input.push('\n');
    }
    input
}

/// Checks `families` for `versions` (at least two): each version fresh, then all of them in one
/// process, twice around.
pub fn check(families: &[String], versions: &[String]) -> Result<(), String> {
    if versions.len() < 2 {
        return Err("--switch needs two versions or more (26.2,26.3)".to_owned());
    }
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
    let fresh = fresh_runs(families, versions)?;
    // 0, 1, .., n-1, 0, 1, .., n-1: every version is switched to, and back to.
    let order: Vec<usize> = (0..2).flat_map(|_| 0..versions.len()).collect();
    let input = switched_input(&fresh.lines, versions, &order);
    let switched = answers(&versions[order[0]], &input)?;
    let mut rest = switched.as_slice();
    let mut differing = 0;
    for &which in &order {
        let (now, later) = rest.split_at(fresh.lines[which].len().min(rest.len()));
        rest = later;
        if now.len() != fresh.answers[which].len() {
            return Err(format!(
                "{}: answers missing after a switch",
                versions[which]
            ));
        }
        for (index, (a, b)) in now.iter().zip(&fresh.answers[which]).enumerate() {
            if a != b {
                differing += 1;
                eprintln!("  {} case {index}: differs after a switch", versions[which]);
            }
        }
    }
    if differing > 0 {
        return Err(format!(
            "switch: {differing} answers differ from a fresh run"
        ));
    }
    let total: usize = order.iter().map(|&which| fresh.lines[which].len()).sum();
    println!(
        "switch: {total} answers over {} versions equal fresh runs",
        versions.len()
    );
    Ok(())
}
