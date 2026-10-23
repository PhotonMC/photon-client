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

    fn list(&self, key: &str) -> Vec<String> {
        self.lines
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| string_list(value))
            .unwrap_or_default()
    }
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
    fn the_selected_packs_read_as_a_json_list_of_strings() {
        assert_eq!(
            string_list(r#"["vanilla", "file/a \"b\".zip"]"#),
            ["vanilla", "file/a \"b\".zip"]
        );
        assert!(string_list("oops").is_empty());
        assert!(string_list("[]").is_empty());
        assert!(string_list(r#"["open"#).is_empty());
    }
}
