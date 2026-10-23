use photon_core::identifier::Identifier;
use photon_core::resources::Resources;

use super::*;

impl Lang {
    /// A language of given texts.
    pub fn new(texts: impl IntoIterator<Item = (String, String)>) -> Self {
        Self {
            texts: texts.into_iter().collect(),
        }
    }

    /// The default language from `assets/minecraft/lang/en_us.json`, with
    /// the deprecated keys removed and renamed (`Language.loadDefault`).
    pub fn load(assets_root: &std::path::Path) -> Self {
        let folder = photon_core::resources::FolderResources::new(assets_root);
        Self::load_from(&folder, assets_root)
    }

    /// The default language as the resource packs have it.
    pub fn load_from(resources: &dyn Resources, jar_assets: &std::path::Path) -> Self {
        Self::load_stack_from(resources, &["en_us"], jar_assets)
    }

    /// `ClientLanguage.loadFrom`: the files of the language codes, the first code first, each
    /// over the ones before it, of every namespace and every pack from the lowest up. The jar's
    /// `deprecated.json` is applied to the whole.
    pub fn load_stack_from(
        resources: &dyn Resources,
        codes: &[&str],
        jar_assets: &std::path::Path,
    ) -> Self {
        let mut texts = HashMap::new();
        for code in codes {
            for namespace in resources.namespaces() {
                let Ok(id) = Identifier::parse(&format!("{namespace}:lang/{code}.json")) else {
                    continue;
                };
                for bytes in resources.read_stack(&id) {
                    if let Ok(text) = String::from_utf8(bytes) {
                        texts.extend(Self::parse_texts(&text));
                    }
                }
            }
        }
        apply_deprecated(jar_assets, &mut texts);
        Self { texts }
    }

    /// `en_us.json` as it is, for a language read over it.
    pub(crate) fn load_raw(assets_root: &std::path::Path) -> Self {
        let Ok(text) = std::fs::read_to_string(assets_root.join("minecraft/lang/en_us.json"))
        else {
            return Self::default();
        };
        Self::new(flat_strings(&text).unwrap_or_default())
    }

    /// The texts of the language, to put others over.
    pub fn into_texts(self) -> HashMap<String, String> {
        self.texts
    }

    /// The texts a language file holds.
    pub fn parse_texts(text: &str) -> Vec<(String, String)> {
        flat_strings(text).unwrap_or_default()
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.texts.get(key).map(String::as_str)
    }

    /// `Language.getOrDefault`: the text, or the key itself.
    pub fn text(&self, key: &str) -> String {
        self.get(key).unwrap_or(key).to_owned()
    }

    /// A component in words: its runs, each in the style it ends up in
    /// under `parent`.
    pub fn runs(&self, component: &Component, parent: &Style) -> Vec<Run> {
        let mut out = Vec::new();
        self.push_runs(component, parent, &mut out);
        out
    }

    pub(super) fn push_runs(&self, component: &Component, parent: &Style, out: &mut Vec<Run>) {
        let style = component.style.apply_to(parent);
        match &component.content {
            Content::Literal(text) => {
                if !text.is_empty() {
                    out.push(Run {
                        text: text.clone(),
                        style: style.clone(),
                    });
                }
            }
            Content::Translatable {
                key,
                fallback,
                args,
            } => {
                let template = self
                    .get(key)
                    .or(fallback.as_deref())
                    .unwrap_or(key)
                    .to_owned();
                self.push_template(&template, args, &style, out);
            }
        }
        for sibling in &component.siblings {
            self.push_runs(sibling, &style, out);
        }
    }

    /// `TranslatableContents.decomposeTemplate`: literal text, and an
    /// argument where `%s`, `%d` or `%n$s` stands.
    pub(super) fn push_template(
        &self,
        template: &str,
        args: &[Component],
        style: &Style,
        out: &mut Vec<Run>,
    ) {
        let mut literal = String::new();
        let mut next_arg = 0;
        let chars: Vec<char> = template.chars().collect();
        let mut at = 0;
        let flush = |literal: &mut String, out: &mut Vec<Run>| {
            if !literal.is_empty() {
                out.push(Run {
                    text: std::mem::take(literal),
                    style: style.clone(),
                });
            }
        };
        while at < chars.len() {
            if chars[at] != '%' {
                literal.push(chars[at]);
                at += 1;
                continue;
            }
            // `%(?:(\d+)\$)?([A-Za-z%]|$)`
            let mut end = at + 1;
            let digits_start = end;
            while end < chars.len() && chars[end].is_ascii_digit() {
                end += 1;
            }
            let position = if end > digits_start && chars.get(end) == Some(&'$') {
                let number: String = chars[digits_start..end].iter().collect();
                end += 1;
                number.parse::<usize>().ok()
            } else {
                end = digits_start;
                None
            };
            match chars.get(end) {
                Some('%') => {
                    literal.push('%');
                    at = end + 1;
                }
                Some(kind) if kind.is_ascii_alphabetic() && matches!(kind, 's' | 'd') => {
                    let index = position.map_or_else(
                        || {
                            let index = next_arg;
                            next_arg += 1;
                            index
                        },
                        |position| position.saturating_sub(1),
                    );
                    flush(&mut literal, out);
                    if let Some(arg) = args.get(index) {
                        self.push_runs(arg, style, out);
                    }
                    at = end + 1;
                }
                _ => {
                    literal.extend(&chars[at..end.min(chars.len())]);
                    if end >= chars.len() {
                        at = chars.len();
                    } else {
                        literal.push(chars[end]);
                        at = end + 1;
                    }
                }
            }
        }
        flush(&mut literal, out);
    }

    /// `Component.getString`: the text without its styles.
    pub fn plain(&self, component: &Component) -> String {
        self.runs(component, &Style::default())
            .into_iter()
            .map(|run| run.text)
            .collect()
    }
}

/// `DeprecatedTranslationsInfo.applyToMap` with the jar's
/// `lang/deprecated.json`: removed keys go, and a renamed key's text moves to
/// its new key (which goes too when the old one is missing).
pub(crate) fn apply_deprecated(assets_root: &std::path::Path, texts: &mut HashMap<String, String>) {
    let Some(info) =
        photon_core::json::read_object(&assets_root.join("minecraft/lang/deprecated.json"))
    else {
        return;
    };
    let strings = |value: &JsonValue| match value {
        JsonValue::String(units) => String::from_utf16(units).ok(),
        _ => None,
    };
    for key in info
        .array("removed")
        .unwrap_or_default()
        .iter()
        .filter_map(strings)
    {
        texts.remove(&key);
    }
    for (from, to) in info
        .object("renamed")
        .into_iter()
        .flat_map(|renamed| renamed.entries())
    {
        let (Ok(from), Some(to)) = (String::from_utf16(from), strings(to)) else {
            continue;
        };
        match texts.remove(&from) {
            Some(text) => texts.insert(to, text),
            None => texts.remove(&to),
        };
    }
}
