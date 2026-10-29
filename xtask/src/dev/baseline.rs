//! The stored bench baseline: what `cargo xtask bench --save` keeps of a run and what
//! `--compare` checks a later run against. The file is machine-specific and lives under `.data/`.

use std::fs;
use std::path::Path;

use serde_json::{Map, Value, json};

/// How far a p99 may rise above the baseline, in percent, before `--compare` fails.
pub const DEFAULT_THRESHOLD_PERCENT: f64 = 10.0;

fn work(report: &Value, name: &str) -> Option<f64> {
    report.pointer(&format!("/work_ms/{name}"))?.as_f64()
}

/// The run in the middle when the runs are ordered by their p99 work time, so one noisy run
/// of several decides neither the baseline nor the comparison.
pub fn representative(reports: &[Value]) -> Option<&Value> {
    let mut ordered: Vec<&Value> = reports.iter().collect();
    ordered.sort_by(|a, b| {
        work(a, "p99")
            .unwrap_or(f64::NAN)
            .total_cmp(&work(b, "p99").unwrap_or(f64::NAN))
    });
    ordered.get(ordered.len() / 2).copied()
}

/// What is kept of a report: the work time's p50, p99 and max, and each part's maximum.
pub fn distill(route: &str, report: &Value) -> Result<Value, String> {
    let mut times = Map::new();
    for name in ["p50", "p99", "max"] {
        let value =
            work(report, name).ok_or_else(|| format!("the report has no work_ms/{name}"))?;
        times.insert(name.to_owned(), json!(value));
    }
    let mut parts = Map::new();
    for (name, stats) in report["parts_ms"].as_object().into_iter().flatten() {
        if let Some(max) = stats["max"].as_f64() {
            parts.insert(name.clone(), json!(max));
        }
    }
    Ok(json!({"route": route, "work_ms": times, "parts_ms": parts}))
}

pub fn save(path: &Path, route: &str, report: &Value) -> Result<(), String> {
    let text = serde_json::to_string_pretty(&distill(route, report)?)
        .map_err(|error| error.to_string())?;
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    }
    fs::write(path, text + "\n").map_err(|error| format!("{}: {error}", path.display()))
}

pub fn load(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| {
        format!(
            "{}: {error}; run `cargo xtask bench --save` first",
            path.display()
        )
    })?;
    serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
}

/// The lines that compare a run with the baseline, and whether its p99 stayed within the
/// threshold. Parts are listed for where a regression sits, but only the p99 fails the run.
pub fn compare(baseline: &Value, report: &Value, threshold: f64) -> (Vec<String>, bool) {
    let mut lines = Vec::new();
    let mut within = true;
    for name in ["p50", "p99", "max"] {
        let (Some(old), Some(new)) = (baseline["work_ms"][name].as_f64(), work(report, name))
        else {
            lines.push(format!("work_ms {name}: not in both"));
            within = false;
            continue;
        };
        let change = if old == 0.0 {
            0.0
        } else {
            (new - old) / old * 100.0
        };
        let failed = name == "p99" && change > threshold;
        within &= !failed;
        let verdict = if failed { " REGRESSED" } else { "" };
        lines.push(format!(
            "work_ms {name}: {old:.2} -> {new:.2} ({change:+.1}%){verdict}"
        ));
    }
    for (name, old) in baseline["parts_ms"].as_object().into_iter().flatten() {
        if let (Some(old), Some(new)) = (old.as_f64(), report["parts_ms"][name]["max"].as_f64()) {
            lines.push(format!("part {name} max: {old:.2} -> {new:.2}"));
        }
    }
    (lines, within)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(p99: f64) -> Value {
        json!({
            "work_ms": {"p50": 4.0, "p95": 5.0, "p99": p99, "max": 9.0},
            "parts_ms": {"mesh": {"max": 2.5}, "light": {"max": 1.0}},
        })
    }

    #[test]
    fn a_baseline_keeps_the_work_times_and_each_parts_maximum() {
        let kept = distill("run", &report(6.0)).unwrap();
        assert_eq!(kept["route"], "run");
        assert_eq!(kept["work_ms"], json!({"p50": 4.0, "p99": 6.0, "max": 9.0}));
        assert_eq!(kept["parts_ms"], json!({"mesh": 2.5, "light": 1.0}));
        assert!(distill("run", &json!({})).is_err());
    }

    #[test]
    fn a_p99_past_the_threshold_fails_and_one_within_it_passes() {
        let baseline = distill("run", &report(10.0)).unwrap();
        let (lines, within) = compare(&baseline, &report(10.9), 10.0);
        assert!(within, "{lines:?}");
        let (lines, within) = compare(&baseline, &report(11.5), 10.0);
        assert!(!within);
        assert!(
            lines
                .iter()
                .any(|line| line.contains("p99") && line.contains("REGRESSED"))
        );
    }

    #[test]
    fn a_faster_run_passes_and_lists_the_parts() {
        let baseline = distill("run", &report(10.0)).unwrap();
        let (lines, within) = compare(&baseline, &report(8.0), 10.0);
        assert!(within);
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("part mesh max: 2.50 -> 2.50"))
        );
    }

    #[test]
    fn the_middle_run_by_p99_stands_for_several() {
        let runs = [report(9.0), report(30.0), report(7.0)];
        assert_eq!(work(representative(&runs).unwrap(), "p99"), Some(9.0));
        assert!(representative(&[]).is_none());
    }
}
