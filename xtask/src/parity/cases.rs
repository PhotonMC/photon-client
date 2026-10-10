//! Parity cases and their content-addressed keys.
//!
//! Ported families come from the `parity-cases` crate; the rest are still
//! the Python modules in `tools/behavior/cases/` until they move too.
//! `--compare-generators` checks a port against the Python output byte for
//! byte, so no stored capture is invalidated.

use std::collections::{BTreeMap, BTreeSet};
use std::process::Command;

use super::digest;
use crate::process;

pub struct Case {
    pub family: String,
    pub id: String,
    /// The JSONL line exactly as the oracle and the adapter receive it.
    pub line: String,
    /// `hash(environment, family sources, line)`: what a capture is stored by.
    pub key: String,
}

const GENERATOR: &str = "\
import json, sys
sys.dont_write_bytecode = True
sys.path.insert(0, 'tools/behavior')
from cases import build_cases
for case in build_cases(sys.argv[1:]):
    sys.stdout.write(json.dumps(case, sort_keys=True, separators=(',', ':'), allow_nan=False) + '\\n')
";

/// Generates the cases of `families`; `family_digest` gives each family's
/// part of the key, `environment` the shared part.
pub fn generate(
    families: &[String],
    environment: &str,
    family_digest: &dyn Fn(&str) -> Result<String, String>,
) -> Result<Vec<Case>, String> {
    let mut sorted = families.to_vec();
    sorted.sort();
    let unported: Vec<String> = sorted
        .iter()
        .filter(|family| parity_cases::lines(family).is_none())
        .cloned()
        .collect();
    let mut python = python_lines(&unported)?;
    let mut cases = Vec::new();
    let mut ids = BTreeSet::new();
    for family in &sorted {
        let lines = match parity_cases::lines(family) {
            Some(lines) => lines,
            None => python.remove(family).unwrap_or_default(),
        };
        let family_key = family_digest(family)?;
        for line in lines {
            let case = case(family, &family_key, environment, line)?;
            if !ids.insert(case.id.clone()) {
                return Err(format!("duplicate case id {}", case.id));
            }
            cases.push(case);
        }
    }
    if cases.is_empty() {
        return Err("no cases were generated".to_owned());
    }
    Ok(cases)
}

fn case(family: &str, family_key: &str, environment: &str, line: String) -> Result<Case, String> {
    let value: serde_json::Value =
        serde_json::from_str(&line).map_err(|error| format!("case: {error}"))?;
    let id = value["case_id"]
        .as_str()
        .ok_or(format!("{family} case without case_id"))?
        .to_owned();
    if value["family"].as_str() != Some(family) {
        return Err(format!("{id} is not in family {family}"));
    }
    let key = digest(format!("{environment}\n{family_key}\n{line}").as_bytes());
    Ok(Case {
        family: family.to_owned(),
        id,
        line,
        key,
    })
}

/// The lines of the families still generated in Python, by family.
fn python_lines(families: &[String]) -> Result<BTreeMap<String, Vec<String>>, String> {
    let mut lines: BTreeMap<String, Vec<String>> = BTreeMap::new();
    if families.is_empty() {
        return Ok(lines);
    }
    let output = Command::new("python3")
        .arg("-c")
        .arg(GENERATOR)
        .args(families)
        .current_dir(process::root())
        .output()
        .map_err(|error| format!("python3: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "case generation failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let text = String::from_utf8(output.stdout).map_err(|_| "case generator printed non-UTF-8")?;
    for line in text.lines() {
        let value: serde_json::Value =
            serde_json::from_str(line).map_err(|error| format!("case: {error}"))?;
        let family = value["family"].as_str().ok_or("case without family")?;
        lines
            .entry(family.to_owned())
            .or_default()
            .push(line.to_owned());
    }
    Ok(lines)
}

/// Checks that each ported family's lines equal the Python generator's,
/// byte for byte (so no capture is invalidated by the port).
pub fn compare_generators(families: &[String]) -> Result<(), String> {
    let ported: Vec<String> = families
        .iter()
        .filter(|family| parity_cases::lines(family).is_some())
        .cloned()
        .collect();
    let mut python = python_lines(&ported)?;
    let mut differing = Vec::new();
    for family in &ported {
        let rust = parity_cases::lines(family).unwrap_or_default();
        let old = python.remove(family).unwrap_or_default();
        match rust.iter().zip(&old).position(|(new, old)| new != old) {
            None if rust.len() == old.len() => println!("{family}: {} lines identical", rust.len()),
            None => differing.push(format!(
                "{family}: {} lines, Python {}",
                rust.len(),
                old.len()
            )),
            Some(index) => {
                let (new, old) = (&rust[index], &old[index]);
                let at = new
                    .bytes()
                    .zip(old.bytes())
                    .take_while(|(a, b)| a == b)
                    .count();
                differing.push(format!(
                    "{family} line {index} byte {at}:\n  rust   {}\n  python {}",
                    clip(new, at),
                    clip(old, at)
                ));
            }
        }
    }
    if differing.is_empty() {
        return Ok(());
    }
    Err(differing.join("\n"))
}

fn clip(line: &str, at: usize) -> &str {
    let start = at.saturating_sub(80);
    line.get(start..line.len().min(at + 120)).unwrap_or(line)
}
