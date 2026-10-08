//! The data-driven registries an item's tooltip reads its texts from:
//! enchantments, painting variants, instruments, jukebox songs, trim
//! materials and patterns and banner patterns. The reference loads them from
//! the data pack the client holds, in the order the server syncs their
//! entries, and an id a component carries is an index into that order.

use std::path::{Path, PathBuf};

use photon_core::json::{self, JsonObject, JsonValue};

/// `Enchantment`, as far as a tooltip reads it.
#[derive(Debug, Clone)]
pub struct EnchantmentEntry {
    pub name: String,
    /// The description, a text component as its JSON.
    pub description: Option<JsonValue>,
    /// In the `curse` tag.
    pub curse: bool,
    pub max_level: i32,
    /// Its place in the `tooltip_order` tag, or -1.
    pub tooltip_order: i32,
}

/// `PaintingVariant`.
#[derive(Debug, Clone)]
pub struct PaintingEntry {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub title: Option<JsonValue>,
    pub author: Option<JsonValue>,
}

/// An entry that is a name and a description: instruments, jukebox songs,
/// trim materials and trim patterns.
#[derive(Debug, Clone)]
pub struct DescribedEntry {
    pub name: String,
    pub description: Option<JsonValue>,
}

/// `BannerPattern`.
#[derive(Debug, Clone)]
pub struct BannerEntry {
    pub name: String,
    pub translation_key: String,
    /// The texture's name under `entity/banner`, without the namespace.
    pub asset_id: String,
}

/// The registries, each in the order its entries were named.
#[derive(Debug, Clone, Default)]
pub struct Registries {
    pub enchantments: Vec<EnchantmentEntry>,
    pub painting_variants: Vec<PaintingEntry>,
    pub instruments: Vec<DescribedEntry>,
    pub jukebox_songs: Vec<DescribedEntry>,
    pub trim_materials: Vec<DescribedEntry>,
    pub trim_patterns: Vec<DescribedEntry>,
    pub banner_patterns: Vec<BannerEntry>,
    /// The chat types' decorations for chat, by registry id.
    pub chat_types: Vec<crate::chat::ChatDecoration>,
}

/// Where a registry entry's file sits in the data pack.
fn entry_path(data_root: &Path, directory: &str, name: &str) -> PathBuf {
    let (namespace, path) = name.split_once(':').unwrap_or(("minecraft", name));
    data_root
        .join(namespace)
        .join(directory)
        .join(format!("{path}.json"))
}

fn read_entry(data_root: &Path, directory: &str, name: &str) -> JsonObject {
    json::read_object(&entry_path(data_root, directory, name)).unwrap_or_default()
}

/// A tag's entries in order, nested tags expanded.
fn tag_values(data_root: &Path, directory: &str, tag: &str) -> Vec<String> {
    let (namespace, path) = tag.split_once(':').unwrap_or(("minecraft", tag));
    let file = data_root
        .join(namespace)
        .join("tags")
        .join(directory)
        .join(format!("{path}.json"));
    let Some(object) = json::read_object(&file) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for value in object.array("values").unwrap_or_default() {
        let entry = match value {
            JsonValue::String(units) => String::from_utf16_lossy(units),
            JsonValue::Object(object) => object.text("id").unwrap_or_default(),
            _ => continue,
        };
        match entry.strip_prefix('#') {
            Some(nested) => out.extend(tag_values(data_root, directory, nested)),
            None => out.push(entry),
        }
    }
    out
}

/// A field of an entry that is a text component, as its JSON.
fn component(object: &JsonObject, key: &str) -> Option<JsonValue> {
    object.get_str(key).cloned()
}

/// The `chat` decoration of a chat type's entry.
fn chat_decoration(entry: &JsonObject) -> crate::chat::ChatDecoration {
    let Some(chat) = entry.object("chat") else {
        return crate::chat::ChatDecoration::default();
    };
    crate::chat::ChatDecoration {
        translation_key: chat.text("translation_key").unwrap_or_default(),
        parameters: chat
            .array("parameters")
            .unwrap_or_default()
            .iter()
            .filter_map(|value| match value {
                JsonValue::String(units) => {
                    crate::chat::ChatParameter::from_name(&String::from_utf16_lossy(units))
                }
                _ => None,
            })
            .collect(),
        style: chat
            .get_str("style")
            .and_then(crate::text::Component::from_json)
            .map(|component| component.style)
            .unwrap_or_default(),
    }
}

impl Registries {
    /// The registries from the data pack at `data_root`, each entry named by
    /// `names(registry)` in the order given (the server's sync order).
    pub fn load(data_root: &Path, names: impl Fn(&str) -> Vec<String>) -> Self {
        let curse = tag_values(data_root, "enchantment", "minecraft:curse");
        let order = tag_values(data_root, "enchantment", "minecraft:tooltip_order");
        let enchantments = names("minecraft:enchantment")
            .into_iter()
            .map(|name| {
                let entry = read_entry(data_root, "enchantment", &name);
                EnchantmentEntry {
                    description: component(&entry, "description"),
                    curse: curse.contains(&name),
                    max_level: entry.i32("max_level").unwrap_or(1),
                    tooltip_order: order
                        .iter()
                        .position(|ordered| *ordered == name)
                        .map_or(-1, |place| place as i32),
                    name,
                }
            })
            .collect();
        let painting_variants = names("minecraft:painting_variant")
            .into_iter()
            .map(|name| {
                let entry = read_entry(data_root, "painting_variant", &name);
                PaintingEntry {
                    width: entry.i32("width").unwrap_or(1),
                    height: entry.i32("height").unwrap_or(1),
                    title: component(&entry, "title"),
                    author: component(&entry, "author"),
                    name,
                }
            })
            .collect();
        let described = |registry: &str, directory: &str| -> Vec<DescribedEntry> {
            names(registry)
                .into_iter()
                .map(|name| DescribedEntry {
                    description: component(&read_entry(data_root, directory, &name), "description"),
                    name,
                })
                .collect()
        };
        let banner_patterns = names("minecraft:banner_pattern")
            .into_iter()
            .map(|name| {
                let entry = read_entry(data_root, "banner_pattern", &name);
                BannerEntry {
                    translation_key: entry.text("translation_key").unwrap_or_default(),
                    asset_id: entry
                        .text("asset_id")
                        .unwrap_or_default()
                        .trim_start_matches("minecraft:")
                        .to_owned(),
                    name,
                }
            })
            .collect();
        let chat_types = names("minecraft:chat_type")
            .into_iter()
            .map(|name| chat_decoration(&read_entry(data_root, "chat_type", &name)))
            .collect();
        Self {
            chat_types,
            enchantments,
            painting_variants,
            instruments: described("minecraft:instrument", "instrument"),
            jukebox_songs: described("minecraft:jukebox_song", "jukebox_song"),
            trim_materials: described("minecraft:trim_material", "trim_material"),
            trim_patterns: described("minecraft:trim_pattern", "trim_pattern"),
            banner_patterns,
        }
    }

    /// The registries with every entry the data pack holds, by name: the
    /// order a vanilla server syncs them in.
    pub fn vanilla(data_root: &Path) -> Self {
        Self::load(data_root, |registry| {
            let directory = registry.strip_prefix("minecraft:").unwrap_or(registry);
            let folder = data_root.join("minecraft").join(directory);
            let mut names: Vec<String> = std::fs::read_dir(folder)
                .into_iter()
                .flatten()
                .filter_map(|entry| {
                    let file = entry.ok()?.file_name().into_string().ok()?;
                    Some(format!("minecraft:{}", file.strip_suffix(".json")?))
                })
                .collect();
            names.sort();
            names
        })
    }
}

/// The vanilla registries the tests read tooltips and creative stacks with:
/// the data pack next to the assets `PHOTON_ASSETS` names, or empty ones.
#[cfg(test)]
pub(crate) fn for_tests() -> Registries {
    let assets = PathBuf::from(std::env::var("PHOTON_ASSETS").unwrap_or_default());
    Registries::vanilla(&assets.with_file_name("data"))
}
