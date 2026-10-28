//! Rewriting the text between a pair of HTML comment markers.

/// `text` with the lines between `<!-- name:start -->` and `<!-- name:end -->` replaced by `body`.
pub fn replace(text: &str, name: &str, body: &str) -> Result<String, String> {
    let start = format!("<!-- {name}:start -->");
    let end = format!("<!-- {name}:end -->");
    let from = text
        .find(&start)
        .ok_or(format!("README.md has no `{start}` marker"))?
        + start.len();
    let to = text
        .find(&end)
        .ok_or(format!("README.md has no `{end}` marker"))?;
    if to < from {
        return Err(format!("`{end}` comes before `{start}`"));
    }
    let body = body.trim_end();
    Ok(format!("{}\n{body}\n{}", &text[..from], &text[to..]))
}

#[cfg(test)]
mod tests {
    use super::replace;

    const TEXT: &str = "before\n<!-- x:start -->\nold\n<!-- x:end -->\nafter\n";

    #[test]
    fn replaces_only_the_block() {
        let out = replace(TEXT, "x", "new\nlines\n").unwrap();
        assert_eq!(
            out,
            "before\n<!-- x:start -->\nnew\nlines\n<!-- x:end -->\nafter\n"
        );
    }

    #[test]
    fn is_idempotent() {
        let once = replace(TEXT, "x", "new").unwrap();
        assert_eq!(replace(&once, "x", "new").unwrap(), once);
    }

    #[test]
    fn missing_or_reversed_markers_fail() {
        assert!(replace("nothing", "x", "b").is_err());
        assert!(replace("<!-- x:end -->\n<!-- x:start -->", "x", "b").is_err());
    }
}
