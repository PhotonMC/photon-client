//! Parity cases and their content-addressed keys.
//!
//! The generators are still the Python modules in `tools/behavior/cases/`;
//! they move to Rust family by family.

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
    let mut digests = std::collections::BTreeMap::new();
    let mut cases = Vec::new();
    for line in text.lines() {
        let value: serde_json::Value =
            serde_json::from_str(line).map_err(|error| format!("case: {error}"))?;
        let field = |name: &str| {
            value[name]
                .as_str()
                .map(str::to_owned)
                .ok_or(format!("case without {name}"))
        };
        let family = field("family")?;
        let id = field("case_id")?;
        if !digests.contains_key(&family) {
            digests.insert(family.clone(), family_digest(&family)?);
        }
        let key = digest(format!("{environment}\n{}\n{line}", digests[&family]).as_bytes());
        cases.push(Case {
            family,
            id,
            line: line.to_owned(),
            key,
        });
    }
    Ok(cases)
}
