//! Capturing Java outputs for cases the store does not have yet.
//!
//! Each family runs in its own JVM, twice, and the families run in parallel.
//! A capture is kept only when both runs agree.

use std::collections::BTreeMap;
use std::path::Path;

use rayon::prelude::*;
use serde_json::Value;

use super::cases::Case;
use super::java::Jdk;
use super::records::{first_difference, parse};

/// One oracle run: the family, which of its two runs, and the records.
type RunOutput = (String, usize, Vec<Value>);

pub struct Oracle<'a> {
    pub jdk: &'a Jdk,
    pub classes: &'a Path,
    pub classpath: &'a str,
    pub assets: &'a Path,
}

/// Captures `cases` and returns `(family, key, record)` for each.
pub fn capture(oracle: &Oracle, cases: &[&Case]) -> Result<Vec<(String, String, Value)>, String> {
    let mut by_family: BTreeMap<&str, Vec<&Case>> = BTreeMap::new();
    for case in cases {
        by_family.entry(&case.family).or_default().push(case);
    }
    let jobs: Vec<(&str, usize)> = by_family
        .keys()
        .flat_map(|family| [(*family, 0), (*family, 1)])
        .collect();
    let threads =
        std::thread::available_parallelism().map_or(2, |count| (count.get() / 2).clamp(1, 8));
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .map_err(|error| error.to_string())?;
    let outputs: Vec<Result<RunOutput, String>> = pool.install(|| {
        jobs.par_iter()
            .map(|(family, run)| {
                let batch = &by_family[family];
                let input: String = batch
                    .iter()
                    .map(|case| format!("{}\n", case.line))
                    .collect();
                let raw = oracle
                    .jdk
                    .run(oracle.classes, oracle.classpath, &input, oracle.assets)
                    .map_err(|error| format!("Java oracle, family {family}: {error}"))?;
                Ok(((*family).to_owned(), *run, parse(&raw, batch)?))
            })
            .collect()
    });
    let mut runs: BTreeMap<(String, usize), Vec<Value>> = BTreeMap::new();
    for output in outputs {
        let (family, run, records) = output?;
        runs.insert((family, run), records);
    }
    let mut captured = Vec::new();
    for (family, batch) in by_family {
        let first = &runs[&(family.to_owned(), 0)];
        let second = &runs[&(family.to_owned(), 1)];
        for ((case, one), two) in batch.iter().zip(first).zip(second) {
            if let Some(difference) = first_difference(&one["result"], &two["result"], "result") {
                return Err(format!(
                    "Java is not repeatable for {}: {difference}",
                    case.id
                ));
            }
            captured.push((case.family.clone(), case.key.clone(), one.clone()));
        }
    }
    Ok(captured)
}
