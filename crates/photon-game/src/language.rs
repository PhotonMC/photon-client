//! The languages the client can show (`LanguageManager`, `LanguageInfo`):
//! the list the resource packs name, and a language's texts read over
//! `en_us`, the one every key is looked up in first.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use photon_core::json::{JsonValue, read_object};

use crate::text::Lang;

/// `LanguageInfo`: how a language is named, in itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageInfo {
    pub region: String,
    pub name: String,
    pub bidirectional: bool,
}

impl LanguageInfo {
    /// `toComponent`'s text: "Name (Region)".
    pub fn label(&self) -> String {
        format!("{} ({})", self.name, self.region)
    }
}

/// The local asset folder as the languages see it: the jar's `en_us` and the
/// other languages the asset index names, kept under their hashes.
#[derive(Debug, Clone)]
pub struct LanguageSource {
    /// `assets` of the jar, where `en_us` is.
    jar_assets: PathBuf,
    /// The asset folder with `indexes` and `objects`.
    assets: PathBuf,
    /// The index's files by name, as hashes.
    index: BTreeMap<String, String>,
}

impl LanguageSource {
    /// The asset folder around the jar's `assets`, if it has an index.
    pub fn open(jar_assets: &Path) -> Option<Self> {
        let assets = jar_assets.parent()?.parent()?.to_path_buf();
        let mut indexes: Vec<PathBuf> = std::fs::read_dir(assets.join("indexes"))
            .ok()?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "json")
            })
            .collect();
        indexes.sort();
        let objects = read_object(indexes.first()?)?;
        let JsonValue::Object(objects) = objects.get_str("objects")? else {
            return None;
        };
        let text = |value: &[u16]| String::from_utf16_lossy(value);
        let index = objects
            .entries()
            .filter_map(|(name, entry)| {
                let JsonValue::Object(entry) = entry else {
                    return None;
                };
                let JsonValue::String(hash) = entry.get_str("hash")? else {
                    return None;
                };
                Some((text(name), text(hash)))
            })
            .collect();
        Some(Self {
            jar_assets: jar_assets.to_path_buf(),
            assets,
            index,
        })
    }

    fn object(&self, name: &str) -> Option<PathBuf> {
        let hash = self.index.get(name)?;
        Some(self.assets.join("objects").join(&hash[..2]).join(hash))
    }

    /// `LanguageManager.getLanguages`: the languages the packs name, by
    /// code (the `language` of the asset pack's `pack.mcmeta`).
    pub fn languages(&self) -> BTreeMap<String, LanguageInfo> {
        let mut languages = BTreeMap::new();
        let Some(meta) = self
            .object("pack.mcmeta")
            .and_then(|path| read_object(&path))
        else {
            return languages;
        };
        let Some(JsonValue::Object(listed)) = meta.get_str("language") else {
            return languages;
        };
        let text = |value: &[u16]| String::from_utf16_lossy(value);
        for (code, info) in listed.entries() {
            let JsonValue::Object(info) = info else {
                continue;
            };
            let field = |key: &str| match info.get_str(key) {
                Some(JsonValue::String(value)) if !value.is_empty() => Some(text(value)),
                _ => None,
            };
            if let (Some(name), Some(region)) = (field("name"), field("region")) {
                languages.insert(
                    text(code),
                    LanguageInfo {
                        region,
                        name,
                        bidirectional: matches!(
                            info.get_str("bidirectional"),
                            Some(JsonValue::Bool(true))
                        ),
                    },
                );
            }
        }
        languages
    }

    /// The same language read from the resource packs, whose files for it add to the jar's and
    /// the index's (which the vanilla pack of `resources` holds).
    pub fn lang_from(&self, code: &str, resources: &dyn photon_core::resources::Resources) -> Lang {
        let codes = if code == "en_us" {
            vec!["en_us"]
        } else {
            vec!["en_us", code]
        };
        Lang::load_stack_from(resources, &codes, &self.jar_assets)
    }

    /// A language of a code: `en_us`, with the texts of the code over it
    /// (`ClientLanguage.loadFrom` with the stack `en_us`, code).
    pub fn lang(&self, code: &str) -> Lang {
        let mut texts = Lang::load_raw(&self.jar_assets).into_texts();
        if code != "en_us"
            && let Some(path) = self.object(&format!("minecraft/lang/{code}.json"))
            && let Ok(text) = std::fs::read_to_string(path)
        {
            texts.extend(Lang::parse_texts(&text));
        }
        // `ClientLanguage.loadFrom`: over every file, once.
        crate::text::apply_deprecated(&self.jar_assets, &mut texts);
        Lang::new(texts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> Option<LanguageSource> {
        LanguageSource::open(Path::new(&photon_core::content_root::assets().ok()?))
    }

    #[test]
    fn the_packs_name_the_languages_and_a_language_reads_over_english() {
        let Some(source) = source() else { return };
        let languages = source.languages();
        assert_eq!(languages["en_us"].label(), "English (US)");
        assert_eq!(
            languages["tr_tr"].label(),
            "T\u{fc}rk\u{e7}e (T\u{fc}rkiye)"
        );
        assert!(languages["ar_sa"].bidirectional);
        let turkish = source.lang("tr_tr");
        let english = source.lang("en_us");
        assert_ne!(turkish.text("gui.done"), english.text("gui.done"));
        assert_eq!(english.text("gui.done"), "Done");
    }
}
