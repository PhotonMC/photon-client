//! The data-driven registries an item's tooltip reads its texts from:
//! enchantments, painting variants, instruments, jukebox songs, trim
//! materials and patterns and banner patterns. The reference loads them from
//! the data pack the client holds, in the order the server syncs their
//! entries, and an id a component carries is an index into that order.

use std::path::{Path, PathBuf};

use voxel_core::json::{self, JsonObject, JsonValue};

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
            .map(|name| BannerEntry {
                translation_key: read_entry(data_root, "banner_pattern", &name)
                    .text("translation_key")
                    .unwrap_or_default(),
                name,
            })
            .collect();
        Self {
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
