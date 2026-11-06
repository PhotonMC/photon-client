//! Parity cases and their content-addressed keys.
//!
//! Every family's cases come from the `parity-cases` crate. A case is keyed
//! by its exact line, so a change to a generator invalidates exactly the
//! captures of the cases it changes.

use std::collections::BTreeSet;

use super::digest;

pub struct Case {
    pub family: String,
    pub id: String,
    /// The JSONL line exactly as the oracle and the adapter receive it.
    pub line: String,
    /// `hash(environment, family sources, line)`: what a capture is stored by.
    pub key: String,
}

/// Generates the cases of `families`; `family_digest` gives each family's
/// part of the key, `environment` the shared part.
pub fn generate(
    families: &[String],
    environment: &str,
    family_digest: &dyn Fn(&str) -> Result<String, String>,
    version: Option<&str>,
) -> Result<Vec<Case>, String> {
    let mut sorted = families.to_vec();
    sorted.sort();
    if let Some(version) = version {
        parity_cases::set_version(version);
    }
    let translation = version.and_then(super::translate::Translation::load);
    let mut cases = Vec::new();
    let mut ids = BTreeSet::new();
    for family in &sorted {
        let lines =
            parity_cases::lines(family).ok_or(format!("no case generator for family {family}"))?;
        let family_key = family_digest(family)?;
        for line in lines {
            if !applies_to(&line, version) {
                continue;
            }
            let line = match &translation {
                Some(translation) => translation.line(line),
                None => line,
            };
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

/// Whether a case belongs to the version under test: a case naming a `version` is captured from
/// that version's jar only.
fn applies_to(line: &str, version: Option<&str>) -> bool {
    let named = serde_json::from_str::<serde_json::Value>(line)
        .ok()
        .and_then(|value| value["version"].as_str().map(str::to_owned));
    match (named, version) {
        (Some(named), Some(version)) => named == version,
        _ => true,
    }
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
