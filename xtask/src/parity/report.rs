//! `cargo xtask parity report`: what the generated cases cover, per family and operation, joined with
//! the Java class each operation exercises (`parity_cases::coverage`). No Java is needed.

use std::collections::BTreeMap;

use super::cases;

/// Cases per `(family, operation)`.
type Counts = BTreeMap<String, BTreeMap<String, usize>>;

/// `cargo xtask parity report [--missing]`: the coverage per family, and with `--missing` each
/// operation that has no Java class recorded yet.
pub fn run(families: &[String], args: &[String]) -> Result<(), String> {
    let missing = args.iter().any(|arg| arg == "--missing");
    if let Some(other) = args.iter().find(|arg| *arg != "--missing") {
        return Err(format!("unknown report option `{other}`"));
    }
    let mut counts = Counts::new();
    for case in cases::generate(families, "", &|_| Ok(String::new()))? {
        let value: serde_json::Value =
            serde_json::from_str(&case.line).map_err(|error| error.to_string())?;
        let operation = value["operation"].as_str().unwrap_or("?").to_owned();
        *counts
            .entry(case.family)
            .or_default()
            .entry(operation)
            .or_default() += 1;
    }
    print_report(&counts, missing);
    Ok(())
}

fn java_class(family: &str, operation: &str) -> Option<&'static str> {
    parity_cases::coverage::COVERAGE
        .iter()
        .chain(
            parity_cases::coverage::INFERRED
                .iter()
                .flat_map(|part| part.iter()),
        )
        .find(|(f, o, _)| *f == family && *o == operation)
        .map(|(_, _, class)| *class)
}

/// The evidence class of a family (docs/parity/README.md): the registry family only compares
/// constants, every other family compares the Java client's behaviour with Rust's.
fn evidence_class(family: &str) -> &'static str {
    if family == "registry" {
        "constants"
    } else {
        "client-parity"
    }
}

fn print_report(counts: &Counts, missing: bool) {
    println!(
        "{:<10} {:<13} {:>7} {:>11} {:>10}",
        "family", "evidence", "cases", "operations", "with class"
    );
    let (mut cases, mut operations, mut known) = (0, 0, 0);
    for (family, ops) in counts {
        let with_class = ops
            .keys()
            .filter(|op| java_class(family, op).is_some())
            .count();
        let total: usize = ops.values().sum();
        println!(
            "{family:<10} {:<13} {total:>7} {:>11} {with_class:>10}",
            evidence_class(family),
            ops.len()
        );
        (cases, operations, known) = (cases + total, operations + ops.len(), known + with_class);
    }
    println!(
        "{:<10} {:<13} {cases:>7} {operations:>11} {known:>10}",
        "all", ""
    );
    if missing {
        for (family, ops) in counts {
            for (operation, count) in ops {
                if java_class(family, operation).is_none() {
                    println!("missing: {family}/{operation} ({count} cases)");
                }
            }
        }
    }
}
