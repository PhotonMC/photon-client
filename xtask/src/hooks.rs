//! Git hook installation and the commit message rules.

use std::fs;

use crate::process;

/// Conventional Commits types this repository uses.
const TYPES: &[&str] = &[
    "feat", "fix", "refactor", "perf", "docs", "test", "build", "ci", "chore", "style", "revert",
];
const MAX_SUBJECT: usize = 72;
/// Trailers and footers that attribute a commit to a tool instead of its author.
const ATTRIBUTION: &[&str] = &["co-authored-by:", "generated with", "claude-session:"];

/// `cargo xtask install-hooks`
pub fn install() -> Result<(), String> {
    process::run("git", &["config", "core.hooksPath", ".githooks"])?;
    println!("git hooks: using .githooks/ (pre-commit, commit-msg, pre-push)");
    Ok(())
}

/// `cargo xtask commit-msg <file>`
pub fn commit_msg(args: &[String]) -> Result<(), String> {
    let [path] = args else {
        return Err("usage: cargo xtask commit-msg <file>".to_owned());
    };
    let text = fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
    let problems = message_problems(&message_body(&text));
    if problems.is_empty() {
        return Ok(());
    }
    Err(format!(
        "commit message rejected:\n  - {}\nformat: `type(scope): imperative English summary`, types: {}",
        problems.join("\n  - "),
        TYPES.join(", ")
    ))
}

/// The message as git will store it: comments and the verbose diff removed.
fn message_body(text: &str) -> String {
    text.lines()
        .take_while(|line| !line.starts_with("# ------------------------ >8"))
        .filter(|line| !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_owned()
}

fn message_problems(message: &str) -> Vec<String> {
    let subject = message.lines().next().unwrap_or_default();
    let generated = ["Merge ", "Revert \"", "fixup! ", "squash! ", "amend! "];
    if generated.iter().any(|prefix| subject.starts_with(prefix)) {
        return Vec::new();
    }
    let mut problems = Vec::new();
    if !conventional(subject) {
        problems.push(format!("subject `{subject}` is not `type(scope): summary`"));
    }
    if subject.len() > MAX_SUBJECT {
        problems.push(format!("subject is longer than {MAX_SUBJECT} characters"));
    }
    if !message.is_ascii() {
        problems.push("message is not English ASCII text".to_owned());
    }
    let attributed = message.lines().any(|line| {
        let line = line.trim().to_ascii_lowercase();
        ATTRIBUTION.iter().any(|marker| line.contains(marker))
    });
    if attributed {
        problems.push("remove tool attribution trailers".to_owned());
    }
    problems
}

fn conventional(subject: &str) -> bool {
    let Some((head, summary)) = subject.split_once(": ") else {
        return false;
    };
    let head = head.strip_suffix('!').unwrap_or(head);
    let (kind, scope) = match head.split_once('(') {
        Some((kind, rest)) => (kind, rest.strip_suffix(')')),
        None => (head, Some("x")),
    };
    let scope_ok = scope.is_some_and(|scope| {
        !scope.is_empty()
            && scope
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "-_/.,".contains(c))
    });
    TYPES.contains(&kind) && scope_ok && summary.starts_with(|c: char| c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_conventional_english_messages() {
        assert!(message_problems("feat(render): add region buffers").is_empty());
        assert!(message_problems("fix: handle empty chunk\n\nLonger body.").is_empty());
        assert!(message_problems("Merge branch 'x'").is_empty());
    }

    #[test]
    fn rejects_format_language_and_attribution() {
        assert_eq!(message_problems("added stuff").len(), 1);
        assert_eq!(message_problems("feat: s\u{fc}r\u{fc}m ekle").len(), 1);
        assert_eq!(
            message_problems("feat: x\n\nCo-Authored-By: A <a@b>").len(),
            1
        );
        assert_eq!(message_problems("feat(Gui): x").len(), 1);
    }

    #[test]
    fn strips_comments_and_verbose_diff() {
        let text = "feat: x\n# comment\n# ------------------------ >8 ------------------------\ndiff \u{fc}";
        assert_eq!(message_body(text), "feat: x");
    }
}
