use super::Run;

/// `ChatFormatting.stripFormatting` of each run's text: a section sign and the code after it.
pub(super) fn stripped_runs(runs: &[Run]) -> Vec<Run> {
    runs.iter()
        .map(|run| Run {
            text: strip_formatting(&run.text),
            style: run.style.clone(),
        })
        .collect()
}

/// `(?i)\u{a7}[0-9A-FK-OR]` removed.
fn strip_formatting(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        let code = chars.peek().copied().map(|next| next.to_ascii_lowercase());
        if c == '\u{a7}'
            && code.is_some_and(|code| matches!(code, '0'..='9' | 'a'..='f' | 'k'..='o' | 'r'))
        {
            chars.next();
        } else {
            out.push(c);
        }
    }
    out
}
