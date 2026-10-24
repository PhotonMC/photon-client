//! Which sprites the GUI atlas holds: the sources of `atlases/gui.json` of every pack, as full
//! ids (`namespace:prefix + path under the source`).

use super::{DecodedImage, Resources};
use photon_core::identifier::Identifier;
use photon_core::json::{JsonObject, JsonValue};
use std::collections::BTreeMap;

/// One entry of an atlas definition's `sources`. The `filter`, `unstable` and
/// `paletted_permutations` types are not read.
#[derive(Debug, PartialEq)]
enum Source {
    /// `minecraft:directory`: every PNG under `textures/<source>`, named `<prefix><path>`.
    Directory { prefix: String, source: String },
    /// `minecraft:single`: one texture, named `sprite` or else by its own path.
    Single {
        resource: String,
        sprite: Option<String>,
    },
}

fn parse_sources(atlas: &JsonObject) -> Vec<Source> {
    let entries = atlas.array("sources").unwrap_or_default();
    entries
        .iter()
        .filter_map(|entry| {
            let JsonValue::Object(entry) = entry else {
                return None;
            };
            match entry.text("type")?.as_str() {
                "minecraft:directory" => Some(Source::Directory {
                    prefix: entry.text("prefix")?,
                    source: entry.text("source")?,
                }),
                "minecraft:single" => Some(Source::Single {
                    resource: entry.text("resource")?,
                    sprite: entry.text("sprite"),
                }),
                _ => None,
            }
        })
        .collect()
}

/// The sources of `atlases/gui.json` in every pack, the lowest pack's first (`AtlasLoader`). A
/// stack with no such file holds the game's own two: the GUI sprites and the effect icons.
fn sources(resources: &dyn Resources) -> Vec<Source> {
    let Ok(file) = Identifier::parse("minecraft:atlases/gui.json") else {
        return Vec::new();
    };
    let found: Vec<Source> = resources
        .read_stack(&file)
        .iter()
        .filter_map(|bytes| std::str::from_utf8(bytes).ok())
        .filter_map(photon_core::json::parse_object)
        .flat_map(|atlas| parse_sources(&atlas))
        .collect();
    if !found.is_empty() {
        return found;
    }
    vec![
        Source::Directory {
            prefix: String::new(),
            source: "gui/sprites".to_owned(),
        },
        Source::Directory {
            prefix: "mob_effect/".to_owned(),
            source: "mob_effect".to_owned(),
        },
    ]
}

/// A sprite of the atlas: its id and the texture file it is cut from.
pub(super) struct Sprite {
    pub(super) id: String,
    file: Identifier,
}

/// The ids of the sprites `atlases/gui.json` lists, sorted.
pub fn sprite_ids(resources: &dyn Resources) -> Vec<String> {
    sprites(resources)
        .into_iter()
        .map(|sprite| sprite.id)
        .collect()
}

/// Every sprite, sorted by id; a later source names a sprite again over an earlier one
/// (`SpriteSourceList`).
pub(super) fn sprites(resources: &dyn Resources) -> Vec<Sprite> {
    let mut found: BTreeMap<String, Identifier> = BTreeMap::new();
    for source in sources(resources) {
        match source {
            Source::Directory { prefix, source } => {
                let root = format!("textures/{source}");
                for file in resources.list(&root, ".png") {
                    let below = file
                        .path()
                        .strip_prefix(&root)
                        .and_then(|rest| rest.strip_prefix('/'))
                        .and_then(|rest| rest.strip_suffix(".png"));
                    if let Some(name) = below {
                        found.insert(format!("{}:{prefix}{name}", file.namespace()), file);
                    }
                }
            }
            Source::Single { resource, sprite } => {
                let Ok(named) = Identifier::parse(&resource) else {
                    continue;
                };
                let name = sprite.unwrap_or(resource);
                let Ok(sprite_id) = Identifier::parse(&name) else {
                    continue;
                };
                let Ok(file) = Identifier::parse(&format!(
                    "{}:textures/{}.png",
                    named.namespace(),
                    named.path()
                )) else {
                    continue;
                };
                if resources.read(&file).is_some() {
                    found.insert(
                        format!("{}:{}", sprite_id.namespace(), sprite_id.path()),
                        file,
                    );
                }
            }
        }
    }
    found
        .into_iter()
        .map(|(id, file)| Sprite { id, file })
        .collect()
}

/// The picture of a sprite.
pub(super) fn read_sprite(resources: &dyn Resources, sprite: &Sprite) -> Option<DecodedImage> {
    crate::texture::png::decode_rgba(&resources.read(&sprite.file)?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Files by id, and the `atlases/gui.json` of each pack, the lowest first.
    struct Files(Vec<&'static str>, Vec<&'static str>);

    impl Resources for Files {
        fn read(&self, id: &Identifier) -> Option<Vec<u8>> {
            self.0
                .iter()
                .any(|file| *file == format!("{}:{}", id.namespace(), id.path()))
                .then(|| vec![0])
        }
        fn read_stack(&self, _: &Identifier) -> Vec<Vec<u8>> {
            self.1.iter().map(|text| text.as_bytes().to_vec()).collect()
        }
        fn list(&self, directory: &str, extension: &str) -> Vec<Identifier> {
            self.0
                .iter()
                .filter_map(|id| Identifier::parse(id).ok())
                .filter(|id| id.path().starts_with(&format!("{directory}/")))
                .filter(|id| id.path().ends_with(extension))
                .collect()
        }
        fn namespaces(&self) -> Vec<String> {
            Vec::new()
        }
    }

    fn ids(files: &Files) -> Vec<String> {
        sprites(files).into_iter().map(|sprite| sprite.id).collect()
    }

    #[test]
    fn without_a_definition_the_gui_sprites_and_the_effect_icons_are_listed() {
        let files = Files(
            vec![
                "minecraft:textures/gui/sprites/hud/heart/full.png",
                "minecraft:textures/gui/sprites/spectator/close.png",
                "foo:textures/gui/sprites/other/y.png",
                "minecraft:textures/mob_effect/speed.png",
                "minecraft:textures/block/stone.png",
            ],
            Vec::new(),
        );
        assert_eq!(
            ids(&files),
            [
                "foo:other/y",
                "minecraft:hud/heart/full",
                "minecraft:mob_effect/speed",
                "minecraft:spectator/close"
            ]
        );
    }

    #[test]
    fn the_definitions_of_every_pack_are_joined() {
        let files = Files(
            vec![
                "minecraft:textures/gui/sprites/hud/heart/full.png",
                "minecraft:textures/custom/deep/a.png",
                "bar:textures/custom/b.png",
                "minecraft:textures/misc/one.png",
            ],
            vec![
                r#"{"sources":[{"type":"minecraft:directory","prefix":"","source":"gui/sprites"}]}"#,
                r#"{"sources":[
                    {"type":"minecraft:directory","prefix":"extra/","source":"custom"},
                    {"type":"minecraft:single","resource":"minecraft:misc/one","sprite":"minecraft:solo"},
                    {"type":"minecraft:filter","pattern":{}}]}"#,
            ],
        );
        assert_eq!(
            ids(&files),
            [
                "bar:extra/b",
                "minecraft:extra/deep/a",
                "minecraft:hud/heart/full",
                "minecraft:solo"
            ]
        );
    }
}
