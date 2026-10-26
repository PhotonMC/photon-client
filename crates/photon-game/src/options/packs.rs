//! The resource packs `options.txt` selects.

use super::*;

impl Options {
    /// `resourcePacks`: the selected packs' ids, the lowest first. The line is kept as the file
    /// has it, so it is read where it is used.
    #[must_use]
    pub fn resource_packs(&self) -> Vec<String> {
        self.list("resourcePacks")
    }

    /// `incompatibleResourcePacks`: the packs the player kept although they are not compatible.
    #[must_use]
    pub fn incompatible_resource_packs(&self) -> Vec<String> {
        self.list("incompatibleResourcePacks")
    }

    /// `updateResourcePacks`: both lists replaced. A line the file lacks goes where the reference
    /// writes it, before `lastServer`. Whether the selection changed.
    pub fn set_resource_packs(&mut self, packs: &[String], incompatible: &[String]) -> bool {
        let changed = self.resource_packs() != packs;
        self.set_list("resourcePacks", packs, "lastServer");
        self.set_list("incompatibleResourcePacks", incompatible, "lastServer");
        changed
    }

    /// `highContrast`: the High Contrast pack is in use. The line is kept as the file has it.
    #[must_use]
    pub fn high_contrast(&self) -> bool {
        self.lines
            .iter()
            .any(|(name, value)| name == "highContrast" && value == "true")
    }

    /// Sets `highContrast`, written before `highContrastBlockOutline` as the reference does.
    pub fn set_high_contrast(&mut self, on: bool) {
        self.set_line("highContrast", on.to_string(), "highContrastBlockOutline");
    }

    fn set_list(&mut self, key: &str, items: &[String], before: &str) {
        self.set_line(key, json_list(items), before);
    }

    fn set_line(&mut self, key: &str, value: String, before: &str) {
        if let Some(line) = self.lines.iter_mut().find(|(name, _)| name == key) {
            line.1 = value;
            return;
        }
        let at = self
            .lines
            .iter()
            .position(|(name, _)| name == before || name == "lang")
            .unwrap_or(self.lines.len());
        self.lines.insert(at, (key.to_owned(), value));
    }

    fn list(&self, key: &str) -> Vec<String> {
        self.lines
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| string_list(value))
            .unwrap_or_default()
    }
}

/// `GSON::toJson` of a list of strings: compact, with the characters Gson keeps out of HTML
/// written as escapes.
fn json_list(items: &[String]) -> String {
    let quoted: Vec<String> = items
        .iter()
        .map(|item| format!("\"{}\"", item.chars().map(escaped).collect::<String>()))
        .collect();
    format!("[{}]", quoted.join(","))
}

/// A character as Gson writes it inside a string.
fn escaped(c: char) -> String {
    match c {
        '"' => "\\\"".to_owned(),
        '\\' => "\\\\".to_owned(),
        '\n' => "\\n".to_owned(),
        '\r' => "\\r".to_owned(),
        '\t' => "\\t".to_owned(),
        '\u{8}' => "\\b".to_owned(),
        '\u{c}' => "\\f".to_owned(),
        c if needs_unicode_escape(c) => format!("\\u{:04x}", c as u32),
        c => c.to_string(),
    }
}

fn needs_unicode_escape(c: char) -> bool {
    matches!(c, '<' | '>' | '&' | '=' | '\'' | '\u{2028}' | '\u{2029}') || (c as u32) < 0x20
}

/// `Options.readListOfStrings`: the strings of a JSON list; anything else is an empty list.
fn string_list(text: &str) -> Vec<String> {
    let Some(inner) = text
        .trim()
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    else {
        return Vec::new();
    };
    let mut items = Vec::new();
    let mut rest = inner.trim_start();
    while rest.starts_with('"') {
        let mut end = 1;
        let bytes = rest.as_bytes();
        while end < bytes.len() && bytes[end] != b'"' {
            end += if bytes[end] == b'\\' { 2 } else { 1 };
        }
        let Some(quoted) = rest.get(..=end) else {
            return Vec::new();
        };
        let Some(item) = json_string(quoted) else {
            return Vec::new();
        };
        items.push(item);
        rest = rest[end + 1..]
            .trim_start()
            .trim_start_matches(',')
            .trim_start();
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lists_are_written_as_gson_does() {
        assert_eq!(json_list(&[]), "[]");
        assert_eq!(
            json_list(&["vanilla".to_owned(), "file/a \"b\"'.zip".to_owned()]),
            r#"["vanilla","file/a \"b\"\u0027.zip"]"#
        );
    }

    #[test]
    fn the_selected_packs_read_as_a_json_list_of_strings() {
        assert_eq!(
            string_list(r#"["vanilla", "file/a \"b\".zip"]"#),
            ["vanilla", "file/a \"b\".zip"]
        );
        assert!(string_list("oops").is_empty());
        assert!(string_list("[]").is_empty());
        assert!(string_list(r#"["open"#).is_empty());
    }

    #[test]
    fn high_contrast_is_a_line_before_the_block_outline() {
        let mut options = Options::default();
        options
            .lines
            .push(("highContrastBlockOutline".to_owned(), "false".to_owned()));
        assert!(!options.high_contrast());
        options.set_high_contrast(true);
        assert!(options.high_contrast());
        assert_eq!(options.lines[0].0, "highContrast");
        options.set_high_contrast(false);
        assert!(!options.high_contrast());
        assert_eq!(options.lines.len(), 2);
    }
}
