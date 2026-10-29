//! `cargo xtask lint`: deterministic structure rules for the whole tree.
//!
//! The rules complement clippy with limits clippy cannot express (code lines
//! per file, crate layering, naming) or expresses only as suppressible
//! warnings. Violations that predate a rule live in the baseline and may
//! only shrink.

pub(crate) mod baseline;
mod config;
mod file;
mod functions;
mod items;
mod layers;
mod names;
pub mod source;

use std::collections::BTreeMap;
use std::time::Instant;

use rayon::prelude::*;

use crate::{git, workspace};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
    FileLines,
    FnComplexity,
    FnLength,
    FnParams,
    FnNesting,
    FileName,
    NonAscii,
    LimitSuppression,
    Unsafe,
    Print,
    RegistryLiteral,
    RawIdWrap,
    VersionCompare,
    WireType,
    NewestVersion,
    RegistryName,
    Layer,
    Generated,
}

impl Rule {
    pub fn name(self) -> &'static str {
        const NAMES: [&str; 18] = [
            "file-lines",
            "fn-complexity",
            "fn-length",
            "fn-params",
            "fn-nesting",
            "file-name",
            "non-ascii",
            "limit-suppression",
            "unsafe",
            "print",
            "registry-literal",
            "raw-id-wrap",
            "version-compare",
            "wire-type",
            "newest-version",
            "registry-name",
            "layer",
            "generated",
        ];
        NAMES[self as usize]
    }

    /// The highest value that is not a violation.
    pub fn limit(self) -> u64 {
        match self {
            Self::FileLines => 500,
            Self::FnComplexity => 10,
            Self::FnLength => 60,
            Self::FnParams => 6,
            Self::FnNesting => 4,
            _ => 0,
        }
    }

    /// The hints of the rules that keep ids and versions apart.
    fn id_hint(self) -> &'static str {
        match self {
            Self::RegistryLiteral => {
                "ids compared with number literals; ids differ per game version, resolve them by name"
            }
            Self::RegistryName => {
                "\"minecraft:\" literals in production code, per crate; compare typed registry constants instead of names"
            }
            Self::RawIdWrap => {
                "a canonical id wrapped around a raw read; read a wire id type and translate it"
            }
            Self::VersionCompare => {
                "Version compared outside the version modules listed in xtask/lint.toml"
            }
            Self::NewestVersion => {
                "Version::NEWEST outside the selection points listed in xtask/lint.toml; take the session's registries"
            }
            _ => "wire id types outside photon-protocol and photon-net",
        }
    }

    /// The hints of the rules about where code may come from and depend on.
    fn source_hint(self) -> &'static str {
        match self {
            Self::Generated => {
                "generated sources; tables are kept by hand (ADR 0013), do not add a generator"
            }
            _ => "crate dependency not allowed by the layer table in xtask/lint.toml",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::FileLines => {
                "code lines (no comments, blanks or inline tests); split the file by responsibility"
            }
            Self::FnComplexity => "branches + 1; extract decisions into named functions or tables",
            Self::FnLength => "code lines in the function; extract steps",
            Self::FnParams => "parameters; group related ones into a struct",
            Self::FnNesting => "nested control flow; return early or extract the inner block",
            Self::FileName => {
                "snake_case, at most 24 characters, specific, not repeating its directory"
            }
            Self::NonAscii => {
                "lines with non-ASCII text; source is English ASCII (use \\u{..} escapes)"
            }
            Self::LimitSuppression => {
                "suppressed clippy limits; the xtask limits replace them, fix the code"
            }
            Self::Unsafe => "unsafe outside the modules listed in xtask/lint.toml",
            Self::Print => "terminal printing in a library; use `log`",
            Self::RegistryLiteral
            | Self::RegistryName
            | Self::RawIdWrap
            | Self::VersionCompare
            | Self::WireType
            | Self::NewestVersion => self.id_hint(),
            Self::Layer | Self::Generated => self.source_hint(),
        }
    }
}

pub struct Finding {
    pub rule: Rule,
    /// Stable identity in the baseline: a path, `path::function` or a crate edge.
    pub key: String,
    pub path: String,
    pub line: usize,
    pub value: u64,
}

impl Finding {
    pub fn new(rule: Rule, key: String, path: String, line: usize) -> Self {
        Self {
            rule,
            key,
            path,
            line,
            value: 1,
        }
    }
}

/// `cargo xtask lint [--quiet] [--init-baseline]`
pub fn run(args: &[String]) -> Result<(), String> {
    let quiet = args.iter().any(|arg| arg == "--quiet");
    let init = args.iter().any(|arg| arg == "--init-baseline");
    let started = Instant::now();
    let config = config::load()?;
    let packages = workspace::packages()?;
    let paths: Vec<String> = git::files()?
        .into_iter()
        .filter(|path| path.ends_with(".rs"))
        .collect();
    let mut findings: Vec<Finding> = paths
        .par_iter()
        .map(|path| file::check(path, &config, &packages))
        .collect::<Result<Vec<_>, String>>()?
        .into_iter()
        .flatten()
        .collect();
    findings = per_crate(findings, &packages);
    findings.extend(layers::check(&packages, &config));
    findings.sort_by(|a, b| (a.rule, &a.path, a.line).cmp(&(b.rule, &b.path, b.line)));

    if init {
        return init_baseline(&findings);
    }
    let old = baseline::load()?;
    let (errors, tightened) = judge(&findings, &old);
    if tightened != old {
        baseline::save(&tightened)?;
        println!("lint: baseline tightened; commit xtask/lint-baseline.toml");
    }
    for finding in &errors {
        report(finding, &old);
    }
    if !quiet || !errors.is_empty() {
        println!(
            "lint: {} files, {} known violations, {} errors ({} ms)",
            paths.len(),
            findings.len() - errors.len(),
            errors.len(),
            started.elapsed().as_millis()
        );
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err("lint failed".to_owned())
    }
}

/// Folds the per-file `registry-name` findings into one finding per crate: the cap is on the
/// crate, so a name moving between its files does not count as growth.
fn per_crate(findings: Vec<Finding>, packages: &[workspace::Package]) -> Vec<Finding> {
    let mut crates: BTreeMap<String, Finding> = BTreeMap::new();
    let mut rest = Vec::new();
    for finding in findings {
        if finding.rule != Rule::RegistryName {
            rest.push(finding);
        } else if let Some(total) = crates.get_mut(&finding.key) {
            total.value += finding.value;
        } else {
            // The cap is on the crate, so the finding points at its directory, not at one file.
            let dir = packages
                .iter()
                .find(|package| package.name == finding.key)
                .map_or(finding.path.clone(), |package| package.dir.clone());
            crates.insert(
                finding.key.clone(),
                Finding {
                    path: dir,
                    line: 1,
                    ..finding
                },
            );
        }
    }
    rest.extend(crates.into_values());
    rest
}

/// Splits findings into errors (new or worse than the baseline) and returns
/// the baseline lowered to the current values.
fn judge<'a>(
    findings: &'a [Finding],
    old: &baseline::Baseline,
) -> (Vec<&'a Finding>, baseline::Baseline) {
    let mut errors = Vec::new();
    let mut tightened = baseline::Baseline::new();
    for finding in findings {
        let known = old
            .get(finding.rule.name())
            .and_then(|entries| entries.get(&finding.key))
            .copied();
        if known.is_none_or(|cap| finding.value > cap) {
            errors.push(finding);
        }
        // A known entry keeps its cap while it is exceeded, so fixing the
        // code later finds it again; otherwise it drops to the current value.
        if let Some(cap) = known {
            tightened
                .entry(finding.rule.name().to_owned())
                .or_default()
                .insert(finding.key.clone(), finding.value.min(cap));
        }
    }
    (errors, tightened)
}

fn report(finding: &Finding, old: &baseline::Baseline) {
    let rule = finding.rule;
    let cap = old
        .get(rule.name())
        .and_then(|entries| entries.get(&finding.key))
        .map_or(String::new(), |cap| format!(", baseline {cap}"));
    eprintln!(
        "{}:{}: {} `{}` = {} (limit {}{cap}): {}",
        finding.path,
        finding.line,
        rule.name(),
        finding.key,
        finding.value,
        rule.limit(),
        rule.hint()
    );
}

fn init_baseline(findings: &[Finding]) -> Result<(), String> {
    if baseline::exists() {
        return Err("lint: the baseline already exists; it may only shrink".to_owned());
    }
    let mut fresh = baseline::Baseline::new();
    for finding in findings {
        let entry = fresh
            .entry(finding.rule.name().to_owned())
            .or_default()
            .entry(finding.key.clone())
            .or_insert(0);
        *entry = (*entry).max(finding.value);
    }
    baseline::save(&fresh)?;
    println!("lint: baseline written with {} violations", findings.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(value: u64) -> Finding {
        Finding {
            value,
            ..Finding::new(Rule::FnLength, "f".to_owned(), "f.rs".to_owned(), 1)
        }
    }

    #[test]
    fn an_exceeded_cap_is_kept_and_a_lower_value_tightens_it() {
        let mut old = baseline::Baseline::new();
        old.entry("fn-length".to_owned())
            .or_default()
            .insert("f".to_owned(), 80);
        let over = [finding(90)];
        let (errors, kept) = judge(&over, &old);
        assert_eq!(errors.len(), 1);
        assert_eq!(kept["fn-length"]["f"], 80);
        let under = [finding(70)];
        let (errors, tightened) = judge(&under, &old);
        assert!(errors.is_empty());
        assert_eq!(tightened["fn-length"]["f"], 70);
    }
}
