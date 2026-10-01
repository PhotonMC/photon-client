//! The GUI's sprites packed into one texture: the reference's `gui/sprites`
//! folders, the whole textures screens blit parts of, and the font's images.

use std::collections::BTreeMap;
use std::path::Path;

use crate::atlas::{AtlasSprite, SpriteEntry, Stitcher};
use crate::png::DecodedImage;
use voxel_core::identifier::Identifier;

/// The sprite folders the HUD and the screens draw from, under
/// `textures/gui/sprites`.
const SPRITE_FOLDERS: [&str; 9] = [
    "boss_bar",
    "pause_menu",
    "friends",
    "hud",
    "container",
    "recipe_book",
    "tooltip",
    "icon",
    "widget",
];

/// Whole textures under `textures/gui` that screens blit parts of, packed as
/// `texture/<name>`.
const WHOLE_TEXTURES: [&str; 22] = [
    "menu_background",
    "inworld_menu_background",
    "container/inventory",
    "container/generic_54",
    "container/dispenser",
    "container/hopper",
    "container/crafting_table",
    "container/furnace",
    "container/blast_furnace",
    "container/smoker",
    "container/stonecutter",
    "container/grindstone",
    "container/cartography_table",
    "container/loom",
    "container/smithing",
    "container/anvil",
    "container/enchanting_table",
    "container/shulker_box",
    "container/creative_inventory/tab_items",
    "container/creative_inventory/tab_item_search",
    "container/creative_inventory/tab_inventory",
    "recipe_book",
];

/// The names of the banner pattern textures (`creeper`), without `banner`
/// the base and `banner_base` that are no patterns.
fn banner_texture_names(assets_root: &Path) -> Vec<String> {
    let folder = assets_root.join("minecraft/textures/entity/banner");
    let mut names: Vec<String> = std::fs::read_dir(folder)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let file = entry.ok()?.file_name().into_string().ok()?;
            file.strip_suffix(".png").map(str::to_owned)
        })
        .collect();
    names.sort();
    names
}

/// Every PNG under a sprite folder, as sprite names (`hud/heart/full`).
fn sprite_names(root: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let base = root.join("minecraft/textures/gui/sprites");
    let mut pending: Vec<std::path::PathBuf> = SPRITE_FOLDERS
        .iter()
        .map(|folder| base.join(folder))
        .collect();
    while let Some(folder) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "png")
                && let Ok(relative) = path.strip_prefix(&base)
            {
                let name = relative.with_extension("");
                names.push(name.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    names.sort();
    names
}

/// The GUI sprites packed into one texture.
#[derive(Debug, Clone)]
pub struct GuiAtlas {
    sprites: BTreeMap<String, [f32; 4]>,
}

impl GuiAtlas {
    /// Read the HUD's sprites from an asset folder and pack them: where each
    /// went, and the texture they make, which only the GPU needs.
    pub fn load(
        assets_root: &Path,
        fonts: &[&crate::gui::font::Font],
    ) -> Option<(Self, DecodedImage)> {
        let mut images = BTreeMap::new();
        let mut stitcher = Stitcher::new(4096, 4096, 0, 0);
        // The font's images go in whole, and glyphs are cut from them.
        for (name, image) in fonts.iter().flat_map(|font| &font.images) {
            stitcher.register_sprite(SpriteEntry {
                width: image.width as i32,
                height: image.height as i32,
                name: Identifier::parse(&format!("minecraft:{name}")).ok()?,
            });
            images.insert(format!("minecraft:{name}"), image.clone());
        }
        for name in sprite_names(assets_root) {
            let path = assets_root.join(format!("minecraft/textures/gui/sprites/{name}.png"));
            let image = crate::png::decode_rgba(&std::fs::read(path).ok()?).ok()?;
            stitcher.register_sprite(SpriteEntry {
                width: image.width as i32,
                height: image.height as i32,
                name: Identifier::parse(&format!("minecraft:{name}")).ok()?,
            });
            images.insert(format!("minecraft:{name}"), image);
        }
        for name in WHOLE_TEXTURES {
            let path = assets_root.join(format!("minecraft/textures/gui/{name}.png"));
            let image = crate::png::decode_rgba(&std::fs::read(path).ok()?).ok()?;
            let key = format!("minecraft:texture/{name}");
            stitcher.register_sprite(SpriteEntry {
                width: image.width as i32,
                height: image.height as i32,
                name: Identifier::parse(&key).ok()?,
            });
            images.insert(key, image);
        }
        // The banner patterns' textures, which the loom's buttons show a
        // piece of.
        for name in banner_texture_names(assets_root) {
            let path = assets_root.join(format!("minecraft/textures/entity/banner/{name}.png"));
            let image = crate::png::decode_rgba(&std::fs::read(path).ok()?).ok()?;
            let key = format!("minecraft:banner/{name}");
            stitcher.register_sprite(SpriteEntry {
                width: image.width as i32,
                height: image.height as i32,
                name: Identifier::parse(&key).ok()?,
            });
            images.insert(key, image);
        }
        // One white texel, which fills are drawn with.
        stitcher.register_sprite(SpriteEntry {
            width: 1,
            height: 1,
            name: Identifier::parse("minecraft:white").ok()?,
        });
        images.insert(
            "minecraft:white".to_owned(),
            DecodedImage {
                width: 1,
                height: 1,
                rgba: vec![255; 4],
            },
        );
        stitcher.stitch().ok()?;
        let width = stitcher.width().max(1) as u32;
        let height = stitcher.height().max(1) as u32;
        let mut atlas = DecodedImage {
            width,
            height,
            rgba: vec![0; (width * height * 4) as usize],
        };
        let mut sprites = BTreeMap::new();
        for placement in stitcher.placements() {
            let key = placement.name.as_string();
            let image = &images[&key];
            let left = (placement.x + placement.padding) as u32;
            let top = (placement.y + placement.padding) as u32;
            for row in 0..image.height {
                let from = (row * image.width * 4) as usize;
                let to = (((top + row) * width + left) * 4) as usize;
                atlas.rgba[to..to + image.width as usize * 4]
                    .copy_from_slice(&image.rgba[from..from + image.width as usize * 4]);
            }
            let sprite = AtlasSprite::new(
                &placement,
                image.width as i32,
                image.height as i32,
                width as i32,
                height as i32,
            );
            sprites.insert(key, [sprite.u0, sprite.v0, sprite.u1, sprite.v1]);
        }
        Some((Self { sprites }, atlas))
    }

    pub(crate) fn sprite(&self, name: &str) -> [f32; 4] {
        self.sprites
            .get(&format!("minecraft:{name}"))
            .copied()
            .unwrap_or([0.0; 4])
    }
}
