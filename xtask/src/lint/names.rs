//! File and directory naming rules.

/// The longest allowed file stem or directory name.
pub const MAX_NAME: usize = 24;

/// Names that say nothing about what the module holds.
const VAGUE: &[&str] = &[
    "utils", "util", "misc", "helpers", "helper", "common", "stuff",
];
/// Stems with a fixed meaning in Cargo or Rust.
const RESERVED: &[&str] = &["mod", "lib", "main", "build"];
/// Directories that group code by kind rather than by topic.
const ROOTS: &[&str] = &["src", "tests", "examples", "benches", "bin"];

/// Why `path` (relative to its package directory) breaks a naming rule.
pub fn problem(relative: &str) -> Option<String> {
    let parts: Vec<&str> = relative.split('/').collect();
    let mut parent: Option<&str> = None;
    for (index, part) in parts.iter().enumerate() {
        let name = if index + 1 == parts.len() {
            part.strip_suffix(".rs").unwrap_or(part)
        } else {
            part
        };
        if let Some(problem) = name_problem(name, parent) {
            return Some(format!("`{part}`: {problem}"));
        }
        parent = (!ROOTS.contains(&name)).then_some(name);
    }
    None
}

fn name_problem(name: &str, parent: Option<&str>) -> Option<String> {
    if RESERVED.contains(&name) || ROOTS.contains(&name) {
        return None;
    }
    let snake = name.starts_with(|c: char| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !snake {
        return Some("use snake_case ASCII".to_owned());
    }
    if name.len() > MAX_NAME {
        return Some(format!("longer than {MAX_NAME} characters"));
    }
    if VAGUE.contains(&name) {
        return Some("vague name; name the module after what it holds".to_owned());
    }
    match parent {
        Some(parent) if name.starts_with(&format!("{parent}_")) => {
            Some(format!("repeats its directory `{parent}`"))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::problem;

    #[test]
    fn accepts_plain_module_paths() {
        assert_eq!(problem("src/gui/screen/pause.rs"), None);
        assert_eq!(problem("src/lib.rs"), None);
        assert_eq!(problem("tests/protocol.rs"), None);
    }

    #[test]
    fn rejects_case_length_vagueness_and_stutter() {
        assert!(problem("src/Gui.rs").is_some());
        assert!(problem("src/a_very_long_module_name_here.rs").is_some());
        assert!(problem("src/net/utils.rs").is_some());
        assert!(problem("src/gui/gui_button.rs").is_some());
    }
}
