//! Reading a case of a parity capture.

use std::io::Read;

use serde_json::Value;

use crate::process;

/// The rows of a case's result in the capture of `family` for `version`.
pub fn rows(version: &str, family: &str, case_id: &str) -> Result<Vec<Value>, String> {
    let path = process::root().join(format!(".data/parity/{version}/{family}.jsonl.gz"));
    let file = std::fs::File::open(&path).map_err(|error| {
        format!(
            "{}: {error} (run `cargo xtask parity` first)",
            path.display()
        )
    })?;
    let mut text = String::new();
    flate2::read::GzDecoder::new(file)
        .read_to_string(&mut text)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    for line in text.lines() {
        let case: Value = serde_json::from_str(line).map_err(|error| error.to_string())?;
        let case = case.get("record").unwrap_or(&case);
        if case["case_id"] == case_id {
            return case["result"]["rows"]
                .as_array()
                .cloned()
                .ok_or_else(|| format!("{case_id}: no rows"));
        }
    }
    Err(format!("no {case_id} case in {}", path.display()))
}
