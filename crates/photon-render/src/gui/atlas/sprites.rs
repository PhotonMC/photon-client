//! Which sprites the GUI atlas holds: the sources of `atlases/gui.json` of every pack, as full
//! ids (`namespace:prefix + path under the source`).

use super::{DecodedImage, Resources};
use photon_assets::IdentifierPattern;
use photon_core::identifier::Identifier;
use photon_core::json::{JsonObject, JsonValue};
use std::collections::BTreeMap;

/// One entry of an atlas definition's `sources`. The `paletted_permutations` type is not read.
enum Source {
    /// `minecraft:directory`: every PNG under `textures/<source>`, named `<prefix><path>`.
    Directory { prefix: String, source: String },
    /// `minecraft:single`: one texture, named `sprite` or else by its own path.
    Single {
        resource: String,
        sprite: Option<String>,
    },
    /// `minecraft:filter`: the sprites listed so far whose id matches are dropped.
    Filter(IdentifierPattern),
    /// `minecraft:unstitch`: named regions cut from one texture.
    Unstitch {
        resource: String,
        regions: Vec<Region>,
        divisors: (f64, f64),
    },
}

/// A region of an unstitched texture, in units of the texture's size over the divisors.
struct Region {
    sprite: String,
    rect: [f64; 4],
}

fn parse_unstitch(entry: &JsonObject) -> Option<Source> {
    let regions: Vec<Region> = entry
        .array("regions")?
        .iter()
        .map(|region| {
            let JsonValue::Object(region) = region else {
                return None;
            };
            Some(Region {
                sprite: region.text("sprite")?,
                rect: [
                    region.f64("x")?,
                    region.f64("y")?,
                    region.f64("width")?,
                    region.f64("height")?,
                ],
            })
        })
        .collect::<Option<_>>()?;
    (!regions.is_empty()).then_some(Source::Unstitch {
        resource: entry.text("resource")?,
        regions,
        divisors: (
            entry.f64("divisor_x").unwrap_or(1.0),
            entry.f64("divisor_y").unwrap_or(1.0),
        ),
    })
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
                "minecraft:filter" => {
                    IdentifierPattern::from_json(entry.object("pattern")?).map(Source::Filter)
                }
                "minecraft:unstitch" => parse_unstitch(entry),
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

/// Where a sprite's picture is: a whole texture, or a region cut from one.
#[derive(Clone)]
enum Origin {
    File(Identifier),
    Region {
        file: Identifier,
        rect: [f64; 4],
        divisors: (f64, f64),
    },
}

/// A sprite of the atlas: its id and where its picture is.
pub(super) struct Sprite {
    pub(super) id: String,
    origin: Origin,
}

/// The ids of the sprites `atlases/gui.json` lists, sorted.
pub fn sprite_ids(resources: &dyn Resources) -> Vec<String> {
    sprites(resources)
        .into_iter()
        .map(|sprite| sprite.id)
        .collect()
}

/// `TEXTURE_ID_CONVERTER.idToFile`: the texture file of a resource name.
fn texture_file(resource: &str) -> Option<Identifier> {
    let named = Identifier::parse(resource).ok()?;
    Identifier::parse(&format!(
        "{}:textures/{}.png",
        named.namespace(),
        named.path()
    ))
    .ok()
}

/// A sprite id as `namespace:path`.
fn full_id(name: &str) -> Option<String> {
    let id = Identifier::parse(name).ok()?;
    Some(format!("{}:{}", id.namespace(), id.path()))
}

/// `SingleFile`: one texture, named `sprite` or else by its own resource name; nothing if the
/// texture is missing.
fn add_single(
    resources: &dyn Resources,
    found: &mut BTreeMap<String, Origin>,
    resource: &str,
    sprite: Option<String>,
) {
    let name = sprite.as_deref().unwrap_or(resource);
    if let (Some(file), Some(id)) = (texture_file(resource), full_id(name))
        && resources.read(&file).is_some()
    {
        found.insert(id, Origin::File(file));
    }
}

/// `Unstitcher`: each region is a sprite; nothing if the texture is missing.
fn add_regions(
    resources: &dyn Resources,
    found: &mut BTreeMap<String, Origin>,
    (resource, regions): (&str, Vec<Region>),
    divisors: (f64, f64),
) {
    let Some(file) = texture_file(resource).filter(|file| resources.read(file).is_some()) else {
        return;
    };
    for region in regions {
        if let Some(id) = full_id(&region.sprite) {
            let origin = Origin::Region {
                file: file.clone(),
                rect: region.rect,
                divisors,
            };
            found.insert(id, origin);
        }
    }
}

/// Every sprite, sorted by id; a later source names a sprite again over an earlier one and a
/// filter drops the ids it matches (`SpriteSourceList`).
pub(super) fn sprites(resources: &dyn Resources) -> Vec<Sprite> {
    let mut found: BTreeMap<String, Origin> = BTreeMap::new();
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
                        found.insert(
                            format!("{}:{prefix}{name}", file.namespace()),
                            Origin::File(file),
                        );
                    }
                }
            }
            Source::Single { resource, sprite } => {
                add_single(resources, &mut found, &resource, sprite);
            }
            Source::Filter(pattern) => found.retain(|id, _| {
                let (namespace, path) = id.split_once(':').unwrap_or(("minecraft", id));
                !pattern.matches(namespace, path)
            }),
            Source::Unstitch {
                resource,
                regions,
                divisors,
            } => add_regions(resources, &mut found, (&resource, regions), divisors),
        }
    }
    found
        .into_iter()
        .map(|(id, origin)| Sprite { id, origin })
        .collect()
}

/// The picture of a sprite.
pub(super) fn read_sprite(resources: &dyn Resources, sprite: &Sprite) -> Option<DecodedImage> {
    match &sprite.origin {
        Origin::File(file) => crate::texture::png::decode_rgba(&resources.read(file)?).ok(),
        Origin::Region {
            file,
            rect,
            divisors,
        } => {
            let whole = crate::texture::png::decode_rgba(&resources.read(file)?).ok()?;
            unstitched(&whole, *rect, *divisors)
        }
    }
}

/// `Unstitcher.RegionInstance.get`: the region of `whole`, its numbers scaled by the picture's
/// size over the divisors and floored; none if it reaches outside the picture.
fn unstitched(
    whole: &DecodedImage,
    [x, y, width, height]: [f64; 4],
    (x_divisor, y_divisor): (f64, f64),
) -> Option<DecodedImage> {
    let x_scale = f64::from(whole.width) / x_divisor;
    let y_scale = f64::from(whole.height) / y_divisor;
    let [x, y, width, height] = [x * x_scale, y * y_scale, width * x_scale, height * y_scale]
        .map(|value| value.floor() as i64);
    let inside = x >= 0
        && y >= 0
        && width >= 0
        && height >= 0
        && x + width <= i64::from(whole.width)
        && y + height <= i64::from(whole.height);
    inside.then(|| {
        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
        for row in y..y + height {
            let start = ((row * i64::from(whole.width) + x) * 4) as usize;
            rgba.extend_from_slice(&whole.rgba[start..start + (width * 4) as usize]);
        }
        DecodedImage {
            width: width as u32,
            height: height as u32,
            rgba,
        }
    })
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
                    {"type":"minecraft:filter","pattern":{"path":"deep"}}]}"#,
            ],
        );
        assert_eq!(
            ids(&files),
            ["bar:extra/b", "minecraft:hud/heart/full", "minecraft:solo"]
        );
    }

    #[test]
    fn a_filter_only_drops_what_came_before_it() {
        let files = Files(
            vec!["minecraft:textures/a/x.png", "minecraft:textures/b/y.png"],
            vec![
                r#"{"sources":[
                    {"type":"minecraft:directory","prefix":"","source":"a"},
                    {"type":"minecraft:filter","pattern":{"namespace":"^minecraft$","path":"^x$"}},
                    {"type":"minecraft:directory","prefix":"","source":"b"},
                    {"type":"minecraft:directory","prefix":"","source":"a"}]}"#,
            ],
        );
        assert_eq!(ids(&files), ["minecraft:x", "minecraft:y"]);
    }

    #[test]
    fn a_region_is_cut_from_the_picture() {
        // 4 by 4, each pixel's red is its index.
        let whole = DecodedImage {
            width: 4,
            height: 4,
            rgba: (0..16).flat_map(|i| [i, 0, 0, 255]).collect(),
        };
        // Half-size regions with divisors of 2: the region at (1, 1) is 2 by 2 pixels from (2, 2).
        let part = unstitched(&whole, [1.0, 1.0, 1.0, 1.0], (2.0, 2.0)).unwrap();
        assert_eq!((part.width, part.height), (2, 2));
        let reds: Vec<u8> = part.rgba.chunks(4).map(|pixel| pixel[0]).collect();
        assert_eq!(reds, [10, 11, 14, 15]);
        assert!(unstitched(&whole, [1.5, 0.0, 1.0, 1.0], (2.0, 2.0)).is_none());
    }
}
