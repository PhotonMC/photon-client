//! The issue counts per milestone, from the GitHub API through the `gh` CLI.
//!
//! These counts change whenever anyone closes an issue, so they sit in their own README block that
//! `cargo xtask progress` rewrites and `--check` leaves alone.

use std::process::Command;

const REPOSITORY: &str = "PhotonMC/photon-client";
const FIELDS: &str = r#".[] | [.number, .title, .html_url, .open_issues, .closed_issues] | @tsv"#;

/// One milestone's row: the title as a link, and its closed and total issues.
fn row(line: &str) -> Result<(u64, String), String> {
    let fields: Vec<&str> = line.split('\t').collect();
    let [number, title, url, open, closed] = fields[..] else {
        return Err(format!("milestone line has {} fields, not 5", fields.len()));
    };
    let count = |text: &str| {
        text.parse::<u64>()
            .map_err(|error| format!("{text}: {error}"))
    };
    let (open, closed) = (count(open)?, count(closed)?);
    Ok((
        count(number)?,
        format!("| [{title}]({url}) | {closed} / {} |", open + closed),
    ))
}

/// The markdown table of every milestone, in the order of their numbers.
pub fn table() -> Result<String, String> {
    let output = Command::new("gh")
        .args([
            "api",
            "--paginate",
            &format!("repos/{REPOSITORY}/milestones?state=all&per_page=100"),
            "--jq",
            FIELDS,
        ])
        .output()
        .map_err(|error| format!("cannot start `gh`: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "`gh api` failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let mut rows = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        rows.push(row(line)?);
    }
    rows.sort();
    let mut out = String::from("| Milestone | Closed / issues |\n| --- | --- |\n");
    for (_, text) in rows {
        out.push_str(&text);
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::row;

    #[test]
    fn formats_a_milestone() {
        let line = "8\tM4 Entities\thttps://example.test/milestone/8\t1\t5";
        assert_eq!(
            row(line).unwrap(),
            (
                8,
                "| [M4 Entities](https://example.test/milestone/8) | 5 / 6 |".to_owned()
            )
        );
    }

    #[test]
    fn rejects_a_short_line() {
        assert!(row("8\tonly").is_err());
    }
}
