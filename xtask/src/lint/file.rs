//! All per-file rules for one Rust source file.

use std::collections::BTreeMap;
use std::fs;

use super::config::{self, Config};
use super::{Finding, Rule, functions, items, names, source};
use crate::process;
use crate::workspace::{self, Package};

/// Paths that do not hold code meant to run as part of a library.
const NON_LIBRARY: &[&str] = &[
    "/src/main.rs",
    "/src/bin/",
    "/examples/",
    "/tests/",
    "/benches/",
    "/build.rs",
];

/// Packages whose registry names are test data or tooling: the oracle, dev probes, parity cases
/// and the xtask.
const NOT_PRODUCTION: &[&str] = &["photon-oracle", "photon-dev", "parity-cases", "xtask"];

pub fn check(path: &str, config: &Config, packages: &[Package]) -> Result<Vec<Finding>, String> {
    let text = fs::read_to_string(process::root().join(path))
        .map_err(|error| format!("{path}: {error}"))?;
    if let Some(findings) = generated_or_table(path, &text) {
        return Ok(findings);
    }
    let package = workspace::owner(packages, path);
    let mut findings = Vec::new();
    if let Some(package) = package {
        let relative = &path[package.dir.len() + 1..];
        if let Some(problem) = names::problem(relative) {
            findings.push(Finding::new(
                Rule::FileName,
                path.to_owned(),
                format!("{path} ({problem})"),
                1,
            ));
        }
    }
    let file = syn::parse_file(&text).map_err(|error| {
        let at = error.span().start();
        format!("{path}:{}:{}: cannot parse: {error}", at.line, at.column)
    })?;
    let code = source::code_lines(&text);
    let facts = items::collect(&file);
    file_lines(path, &code, &facts, &mut findings);
    function_rules(path, &file, &code, &mut findings);
    let library = package.is_some_and(|package| !config.binary_crates.contains(&package.name))
        && !NON_LIBRARY.iter().any(|part| path.contains(part));
    let non_ascii = if config::allowed(&config.non_ascii_allowed, path) {
        Vec::new()
    } else {
        source::non_ascii_lines(&text)
    };
    let unsafes = if config::allowed(&config.unsafe_allowed, path) {
        &[][..]
    } else {
        &facts.unsafes[..]
    };
    let prints = if library { &facts.prints[..] } else { &[][..] };
    count(path, Rule::NonAscii, &non_ascii, &mut findings);
    count(
        path,
        Rule::LimitSuppression,
        &facts.suppressions,
        &mut findings,
    );
    count(path, Rule::Unsafe, unsafes, &mut findings);
    count(path, Rule::Print, prints, &mut findings);
    count(
        path,
        Rule::RegistryLiteral,
        &facts.registry_literals,
        &mut findings,
    );
    registry_names(path, &facts, package, &mut findings);
    version_rules(path, (&text, &facts), (config, package), &mut findings);
    Ok(findings)
}

/// The `"minecraft:` literals of a production file, counted for its crate.
fn registry_names(
    path: &str,
    facts: &items::Facts,
    package: Option<&Package>,
    findings: &mut Vec<Finding>,
) {
    let Some(package) = package else {
        return;
    };
    // Unlike `NON_LIBRARY`, programs count: `src/main.rs` and `src/bin` are production code.
    let tests = path.contains("/tests/")
        || path.contains("/examples/")
        || path.contains("/benches/")
        || path.ends_with("/build.rs")
        || path.ends_with("/tests.rs")
        || path.ends_with("_tests.rs");
    if NOT_PRODUCTION.contains(&package.name.as_str()) || tests {
        return;
    }
    if let Some(first) = facts.registry_names.first() {
        findings.push(Finding {
            value: facts.registry_names.len() as u64,
            ..Finding::new(
                Rule::RegistryName,
                package.name.clone(),
                path.to_owned(),
                *first,
            )
        });
    }
}

/// The rules of the multi-version content model (ADR 0013).
fn version_rules(
    path: &str,
    source: (&str, &items::Facts),
    context: (&Config, Option<&Package>),
    findings: &mut Vec<Finding>,
) {
    let ((text, facts), (config, package)) = (source, context);
    count(path, Rule::RawIdWrap, &facts.raw_id_wraps, findings);
    if !version_allowed(config, path) {
        count(
            path,
            Rule::VersionCompare,
            &facts.version_compares,
            findings,
        );
    }
    wire_types(path, text, package, findings);
    if !path.starts_with("xtask/") && !config::allowed(&config.newest_allowed, path) {
        let lines: Vec<usize> = text
            .lines()
            .enumerate()
            .filter(|(_, line)| {
                !line.trim_start().starts_with("//") && line.contains("Version::NEWEST")
            })
            .map(|(index, _)| index + 1)
            .collect();
        count(path, Rule::NewestVersion, &lines, findings);
    }
}

fn version_allowed(config: &Config, path: &str) -> bool {
    path.contains("/versions/") || config::allowed(&config.version_allowed, path)
}

/// Wire id types belong to the packet boundary: the protocol and the connection.
fn wire_types(path: &str, text: &str, package: Option<&Package>, findings: &mut Vec<Finding>) {
    const OWNERS: &[&str] = &["photon-protocol", "photon-net", "xtask"];
    if package.is_none_or(|package| OWNERS.contains(&package.name.as_str())) {
        return;
    }
    let lines: Vec<usize> = text
        .lines()
        .enumerate()
        .filter(|(_, line)| {
            ["WireItemId", "WireBlockStateId", "WireEntityTypeId"]
                .iter()
                .any(|name| line.contains(name))
        })
        .map(|(index, _)| index + 1)
        .collect();
    count(path, Rule::WireType, &lines, findings);
}

/// A table file is exempt from the other rules, a source that claims to be generated is a finding.
fn generated_or_table(path: &str, text: &str) -> Option<Vec<Finding>> {
    if let Some(finding) = generated_source(path, text) {
        return Some(vec![finding]);
    }
    path.starts_with("crates/photon-data/src/tables/")
        .then(Vec::new)
}

/// Sources that claim to be generated, or sit in a `generated` directory. The tables are kept by
/// hand ([ADR 0013](../../../docs/adr/0013-multi-version-content-model.md)); the big ones in
/// `photon-data/src/tables/` are only exempt from the size and style rules.
fn generated_source(path: &str, text: &str) -> Option<Finding> {
    let marked = path.contains("/generated/")
        || text
            .lines()
            .take(5)
            .any(|line| line.contains("@generated") || line.contains("This file is generated"));
    marked.then(|| Finding::new(Rule::Generated, path.to_owned(), path.to_owned(), 1))
}

fn file_lines(path: &str, code: &[bool], facts: &items::Facts, findings: &mut Vec<Finding>) {
    let in_tests = |line: usize| {
        facts
            .test_modules
            .iter()
            .any(|(start, end)| (*start..=*end).contains(&line))
    };
    let lines = code
        .iter()
        .enumerate()
        .filter(|(index, code)| **code && !in_tests(index + 1))
        .count() as u64;
    if lines > Rule::FileLines.limit() {
        findings.push(Finding {
            value: lines,
            ..Finding::new(Rule::FileLines, path.to_owned(), path.to_owned(), 1)
        });
    }
}

fn function_rules(path: &str, file: &syn::File, code: &[bool], findings: &mut Vec<Finding>) {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for measured in functions::measure(file, code) {
        let occurrence = seen.entry(measured.name.clone()).or_default();
        *occurrence += 1;
        let key = match *occurrence {
            1 => format!("{path}::{}", measured.name),
            n => format!("{path}::{}#{n}", measured.name),
        };
        let checks = [
            (Rule::FnComplexity, measured.complexity),
            (Rule::FnLength, measured.length),
            (Rule::FnParams, measured.params),
            (Rule::FnNesting, measured.nesting),
        ];
        for (rule, value) in checks {
            if value > rule.limit() {
                findings.push(Finding {
                    value,
                    ..Finding::new(rule, key.clone(), path.to_owned(), measured.line)
                });
            }
        }
    }
}

/// One finding whose value is how many lines break a zero-tolerance rule.
fn count(path: &str, rule: Rule, lines: &[usize], findings: &mut Vec<Finding>) {
    if let Some(first) = lines.first() {
        findings.push(Finding {
            value: lines.len() as u64,
            ..Finding::new(rule, path.to_owned(), path.to_owned(), *first)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_generated_marker_or_directory_is_a_finding() {
        assert!(generated_source("crates/a/src/generated/x.rs", "fn f() {}").is_some());
        assert!(generated_source("crates/a/src/x.rs", "// @generated\nfn f() {}").is_some());
        assert!(generated_source("crates/a/src/x.rs", "fn f() {}").is_none());
    }

    #[test]
    fn the_hand_kept_tables_are_exempt_from_the_other_rules() {
        let table = "crates/photon-data/src/tables/block.rs";
        assert_eq!(
            generated_or_table(table, "const A: u8 = 1;").map(|f| f.len()),
            Some(0)
        );
        assert!(generated_or_table("crates/a/src/x.rs", "fn f() {}").is_none());
    }
}
