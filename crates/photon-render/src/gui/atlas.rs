//! The GUI's sprites packed into one texture: the reference's `gui/sprites`
//! folders, the whole textures screens blit parts of, and the font's images.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::texture::atlas::{AtlasSprite, SpriteEntry, Stitcher};
use crate::texture::png::DecodedImage;
use photon_core::identifier::Identifier;

/// The sprite folders the HUD and the screens draw from, under
/// `textures/gui/sprites`.
const SPRITE_FOLDERS: [&str; 16] = [
    "advancements",
    "statistics",
    "social_interactions",
    "gamemode_switcher",
    "boss_bar",
    "server_list",
    "pause_menu",
    "friends",
    "hud",
    "container",
    "recipe_book",
    "tooltip",
    "icon",
    "widget",
    "toast",
    "dialog",
];

/// Whole textures under `textures/gui` that screens blit parts of, packed as
/// `texture/<name>`.
const WHOLE_TEXTURES: [&str; 53] = [
    "tab_header_background",
    "advancements/window",
    "advancements/backgrounds/adventure",
    "advancements/backgrounds/end",
    "advancements/backgrounds/husbandry",
    "advancements/backgrounds/nether",
    "advancements/backgrounds/stone",
    "menu_background",
    "inworld_menu_background",
    "menu_list_background",
    "inworld_menu_list_background",
    "header_separator",
    "footer_separator",
    "inworld_header_separator",
    "inworld_footer_separator",
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
    "container/brewing_stand",
    "container/beacon",
    "container/crafter",
    "container/villager",
    "container/gamemode_switcher",
    "container/horse",
    "container/nautilus",
    "book",
    "container/shulker_box",
    "container/creative_inventory/tab_items",
    "container/creative_inventory/tab_item_search",
    "container/creative_inventory/tab_inventory",
    "recipe_book",
    "title/minecraft",
    "title/edition",
    "title/background/panorama_overlay",
    "textures/misc/unknown_server",
    "textures/misc/credits_vignette",
    "textures/misc/powder_snow_outline",
    "textures/misc/vignette",
    "textures/misc/nausea",
    "textures/misc/pumpkinblur",
    "textures/misc/spyglass_scope",
];

/// The names of the PNG textures in a folder (`creeper` of the banner patterns), sorted.
fn png_names(folder: &Path) -> Vec<String> {
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

/// Every PNG of a folder, stitched under `minecraft:<prefix>/<name>`.
fn register_folder(
    stitcher: &mut Stitcher,
    images: &mut BTreeMap<String, DecodedImage>,
    folder: &Path,
    prefix: &str,
) -> Option<()> {
    for name in png_names(folder) {
        let path = folder.join(format!("{name}.png"));
        let image = crate::texture::png::decode_rgba(&std::fs::read(path).ok()?).ok()?;
        let key = format!("minecraft:{prefix}/{name}");
        stitcher.register_sprite(SpriteEntry {
            width: image.width as i32,
            height: image.height as i32,
            name: Identifier::parse(&key).ok()?,
        });
        images.insert(key, image);
    }
    Some(())
}

/// The whole textures screens blit parts of (`WHOLE_TEXTURES`), and the default skins whose
/// faces the social screen draws, stitched under `minecraft:texture/<name>`.
fn register_whole_textures(
    stitcher: &mut Stitcher,
    images: &mut BTreeMap<String, DecodedImage>,
    assets_root: &Path,
) -> Option<BTreeSet<String>> {
    let mut blurred = BTreeSet::new();
    let skins =
        crate::entity::player_renderer::skin_textures().map(|name| format!("textures/{name}"));
    let wholes = WHOLE_TEXTURES
        .iter()
        .map(|name| (*name).to_owned())
        .chain(skins);
    for name in wholes {
        // A name with its folder (`textures/misc/...`) is outside `textures/gui`.
        let path = if name.starts_with("textures/") {
            assets_root.join(format!("minecraft/{name}.png"))
        } else {
            assets_root.join(format!("minecraft/textures/gui/{name}.png"))
        };
        let mut image = crate::texture::png::decode_rgba(&std::fs::read(&path).ok()?).ok()?;
        let key = format!("minecraft:texture/{name}");
        // A blurred texture is sampled linearly and clamped at its edge: a border of its own
        // edge texels around it keeps the filter off its neighbours.
        if has_blur(&path) {
            image = with_edge_border(&image);
            blurred.insert(key.clone());
        }
        stitcher.register_sprite(SpriteEntry {
            width: image.width as i32,
            height: image.height as i32,
            name: Identifier::parse(&key).ok()?,
        });
        images.insert(key, image);
    }
    Some(blurred)
}

/// Where a stitched image's own texels are in the atlas: inside the border a blurred
/// texture carries.
fn placed_sprite(
    placement: &crate::texture::atlas::SpritePlacement,
    (image, blurred): (&DecodedImage, bool),
    (atlas_width, atlas_height): (i32, i32),
) -> AtlasSprite {
    let border = i32::from(blurred);
    let mut inner = placement.clone();
    inner.x += border;
    inner.y += border;
    AtlasSprite::new(
        &inner,
        image.width as i32 - 2 * border,
        image.height as i32 - 2 * border,
        atlas_width,
        atlas_height,
    )
}

/// Whether a texture's metadata file asks for it to be sampled smoothly (`blur`).
fn has_blur(texture: &Path) -> bool {
    let mut metadata = texture.as_os_str().to_owned();
    metadata.push(".mcmeta");
    photon_core::json::read_object(Path::new(&metadata))
        .and_then(|file| {
            file.object("texture")
                .and_then(crate::texture::metadata::TextureMetadata::parse)
        })
        .is_some_and(|section| section.blur)
}

/// The image with one more texel all round, each a copy of the nearest edge texel.
fn with_edge_border(image: &DecodedImage) -> DecodedImage {
    let (width, height) = (image.width + 2, image.height + 2);
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        let row = y.saturating_sub(1).min(image.height - 1);
        for x in 0..width {
            let column = x.saturating_sub(1).min(image.width - 1);
            let at = ((row * image.width + column) * 4) as usize;
            rgba.extend_from_slice(&image.rgba[at..at + 4]);
        }
    }
    DecodedImage {
        width,
        height,
        rgba,
    }
}

/// One white texel, which fills are drawn with.
fn register_white(
    stitcher: &mut Stitcher,
    images: &mut BTreeMap<String, DecodedImage>,
) -> Option<()> {
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
    Some(())
}

/// How many server icons the atlas holds at once.
const FAVICON_SLOTS: usize = 32;
/// `FaviconTexture`'s width and height.
const FAVICON_SIZE: u32 = 64;
/// The name the reserved icon slots are stitched under.
const FAVICON_SLOT_PREFIX: &str = "minecraft:favicon_slot/";

/// A place in the atlas for one server icon, which a texture name can claim.
#[derive(Debug, Clone)]
struct FaviconSlot {
    /// The slot's corner in texels.
    origin: (u32, u32),
    uv: [f32; 4],
    /// The sprite key it is claimed under.
    key: Option<String>,
}

/// Room for the icons servers send, filled while the game runs.
fn register_favicon_slots(
    stitcher: &mut Stitcher,
    images: &mut BTreeMap<String, DecodedImage>,
) -> Option<()> {
    for slot in 0..FAVICON_SLOTS {
        let key = format!("{FAVICON_SLOT_PREFIX}{slot}");
        stitcher.register_sprite(SpriteEntry {
            width: FAVICON_SIZE as i32,
            height: FAVICON_SIZE as i32,
            name: Identifier::parse(&key).ok()?,
        });
        images.insert(
            key,
            DecodedImage {
                width: FAVICON_SIZE,
                height: FAVICON_SIZE,
                rgba: vec![0; (FAVICON_SIZE * FAVICON_SIZE * 4) as usize],
            },
        );
    }
    Some(())
}

/// The GUI sprites packed into one texture.
#[derive(Debug, Clone)]
pub struct GuiAtlas {
    sprites: BTreeMap<String, [f32; 4]>,
    favicons: Vec<FaviconSlot>,
    /// The whole textures sampled smoothly (`blur` in their metadata), by key.
    blurred: BTreeSet<String>,
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
            let image = crate::texture::png::decode_rgba(&std::fs::read(path).ok()?).ok()?;
            stitcher.register_sprite(SpriteEntry {
                width: image.width as i32,
                height: image.height as i32,
                name: Identifier::parse(&format!("minecraft:{name}")).ok()?,
            });
            images.insert(format!("minecraft:{name}"), image);
        }
        let blurred = register_whole_textures(&mut stitcher, &mut images, assets_root)?;
        // The boards the sign editing screens show, one per wood, and the
        // banner patterns' textures, which the loom's buttons show a piece of.
        let textures = assets_root.join("minecraft/textures");
        for (folder, prefix) in [
            ("gui/signs", "texture/signs"),
            ("gui/hanging_signs", "texture/hanging_signs"),
            ("entity/banner", "banner"),
        ] {
            register_folder(&mut stitcher, &mut images, &textures.join(folder), prefix)?;
        }
        register_favicon_slots(&mut stitcher, &mut images)?;
        register_white(&mut stitcher, &mut images)?;
        stitcher.stitch().ok()?;
        let width = stitcher.width().max(1) as u32;
        let height = stitcher.height().max(1) as u32;
        let mut atlas = DecodedImage {
            width,
            height,
            rgba: vec![0; (width * height * 4) as usize],
        };
        let mut sprites = BTreeMap::new();
        let mut favicons = Vec::new();
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
            let sprite = placed_sprite(
                &placement,
                (image, blurred.contains(&key)),
                (width as i32, height as i32),
            );
            let uv = [sprite.u0, sprite.v0, sprite.u1, sprite.v1];
            if key.starts_with(FAVICON_SLOT_PREFIX) {
                favicons.push(FaviconSlot {
                    origin: (left, top),
                    uv,
                    key: None,
                });
            } else {
                sprites.insert(key, uv);
            }
        }
        Some((
            Self {
                sprites,
                favicons,
                blurred,
            },
            atlas,
        ))
    }

    /// Every icon texture released (`FaviconTexture.close` of the entries of a list that is gone).
    pub fn clear_favicons(&mut self) {
        for slot in &mut self.favicons {
            if let Some(key) = slot.key.take() {
                self.sprites.remove(&key);
            }
        }
    }

    /// `FaviconTexture.upload` and `clear` of the texture `name`: put a 64 by 64 `icon` where the
    /// texture is drawn from, or take the texture out. Whether `atlas`, the image `load` made,
    /// changed and is to be sent to the GPU again. An icon with no slot left is not shown.
    pub fn set_favicon(
        &mut self,
        atlas: &mut DecodedImage,
        name: &str,
        icon: Option<&DecodedImage>,
    ) -> bool {
        let key = format!("minecraft:texture/{name}");
        let claimed = self
            .favicons
            .iter()
            .position(|slot| slot.key.as_deref() == Some(&key));
        let Some(icon) = icon else {
            let Some(index) = claimed else {
                return false;
            };
            self.favicons[index].key = None;
            self.sprites.remove(&key);
            return false;
        };
        let Some(index) =
            claimed.or_else(|| self.favicons.iter().position(|slot| slot.key.is_none()))
        else {
            return false;
        };
        let slot = &mut self.favicons[index];
        slot.key = Some(key.clone());
        self.sprites.insert(key, slot.uv);
        let row_bytes = (FAVICON_SIZE * 4) as usize;
        for row in 0..FAVICON_SIZE {
            let from = row as usize * row_bytes;
            let to = (((slot.origin.1 + row) * atlas.width + slot.origin.0) * 4) as usize;
            atlas.rgba[to..to + row_bytes].copy_from_slice(&icon.rgba[from..from + row_bytes]);
        }
        true
    }

    /// The blend a draw of `name` takes: the smooth-sampling variant for a blurred texture.
    pub(crate) fn blend_of(&self, name: &str, blend: super::GuiBlend) -> super::GuiBlend {
        if !self.blurred.contains(&format!("minecraft:{name}")) {
            return blend;
        }
        match blend {
            super::GuiBlend::Translucent => super::GuiBlend::TranslucentBlur,
            super::GuiBlend::Vignette => super::GuiBlend::VignetteBlur,
            other => other,
        }
    }

    pub(crate) fn sprite(&self, name: &str) -> [f32; 4] {
        self.sprites
            .get(&format!("minecraft:{name}"))
            .copied()
            .unwrap_or([0.0; 4])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_edge_border_repeats_the_nearest_texels() {
        let image = DecodedImage {
            width: 2,
            height: 1,
            rgba: vec![1, 2, 3, 4, 5, 6, 7, 8],
        };
        let padded = with_edge_border(&image);
        assert_eq!((padded.width, padded.height), (4, 3));
        let row = [[1, 2, 3, 4], [1, 2, 3, 4], [5, 6, 7, 8], [5, 6, 7, 8]].concat();
        assert_eq!(padded.rgba, [row.clone(), row.clone(), row].concat());
    }
}
