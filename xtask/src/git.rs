//! The few git queries the gates need.

use crate::process;

/// Every tracked file, plus untracked files that are not ignored.
pub fn files() -> Result<Vec<String>, String> {
    let text = process::output(
        "git",
        &["ls-files", "--cached", "--others", "--exclude-standard"],
    )?;
    let root = process::root();
    Ok(text
        .lines()
        .filter(|path| root.join(path).is_file())
        .map(str::to_owned)
        .collect())
}

/// Files in the index that differ from `HEAD` (added, copied, modified, renamed).
pub fn staged() -> Result<Vec<String>, String> {
    lines(&["diff", "--cached", "--name-only", "--diff-filter=ACMR"])
}

/// Files changed between `base` and `HEAD`.
pub fn changed_since(base: &str) -> Result<Vec<String>, String> {
    lines(&["diff", "--name-only", base, "HEAD"])
}

/// The ref a push compares against: the upstream if set, else the remote default.
pub fn push_base() -> Result<String, String> {
    let candidates = ["@{upstream}", "origin/main", "origin/master"];
    for candidate in candidates {
        if let Ok(base) = process::output("git", &["merge-base", "HEAD", candidate]) {
            return Ok(base.trim().to_owned());
        }
    }
    Err("no upstream, origin/main or origin/master to compare the push with".to_owned())
}

fn lines(args: &[&str]) -> Result<Vec<String>, String> {
    Ok(process::output("git", args)?
        .lines()
        .map(str::to_owned)
        .collect())
}
