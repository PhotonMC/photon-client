//! Metrics that need only the repository, not the reference.

use crate::lint::baseline;
use crate::parity::FAMILIES;

/// Entries the lint baseline held when it was introduced (commit `dde9af555`).
const BASELINE_AT_INTRODUCTION: usize = 1234;

/// Thousands separators, as the README writes numbers.
pub fn group(number: usize) -> String {
    let digits = number.to_string();
    let mut out = String::new();
    for (at, digit) in digits.chars().enumerate() {
        if at > 0 && (digits.len() - at).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// Parity cases and families, and the lint baseline.
pub fn rows() -> Result<Vec<(String, String)>, String> {
    let mut cases = 0;
    for family in FAMILIES {
        cases += parity_cases::lines(family)
            .ok_or(format!("no case generator for family {family}"))?
            .len();
    }
    let entries: usize = baseline::load()?.values().map(|rule| rule.len()).sum();
    Ok(vec![
        (
            "Parity cases".to_owned(),
            format!("{} in {} families", group(cases), FAMILIES.len()),
        ),
        (
            "Lint baseline entries".to_owned(),
            format!(
                "{} (at introduction: {})",
                group(entries),
                group(BASELINE_AT_INTRODUCTION)
            ),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::group;

    #[test]
    fn groups_thousands() {
        assert_eq!(group(0), "0");
        assert_eq!(group(970), "970");
        assert_eq!(group(1234), "1,234");
        assert_eq!(group(4600), "4,600");
        assert_eq!(group(1234567), "1,234,567");
    }
}
