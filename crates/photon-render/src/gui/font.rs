//! The default font, as the reference builds it from its font definition.
//!
//! `font/default.json` lists providers in order, some by reference to other
//! definitions; a codepoint is drawn by the first provider that has it. A
//! space provider gives advances with nothing to draw. A bitmap provider cuts
//! a PNG into a grid of cells, one per codepoint of its `chars` rows: each
//! glyph is as wide as its rightmost column with any alpha, scaled to the
//! provider's height, and advances one pixel past that. The unifont fallback
//! is not read yet, so codepoints only it has are skipped.

use std::collections::HashMap;
use std::path::Path;

use photon_core::identifier::Identifier;
use photon_core::json::{JsonObject, JsonValue};
use photon_core::resources::{FolderResources, Resources};

use crate::texture::png::DecodedImage;

/// One bitmap glyph: where its cell is in its provider's image, and how it
/// is placed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BitmapGlyph {
    /// Index into [`Font::images`].
    pub image: usize,
    /// The cell's corner and size in the image's pixels.
    pub cell: [u32; 4],
    /// Pixels to GUI units: the provider's height over the cell height.
    pub scale: f32,
    pub ascent: i32,
    pub advance: f32,
}

/// What a codepoint is drawn with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Glyph {
    /// Only an advance.
    Space(f32),
    Bitmap(BitmapGlyph),
}

impl Glyph {
    /// `GlyphInfo.getAdvance(bold)`: bold adds a unit.
    pub fn advance(&self, bold: bool) -> f32 {
        let advance = match self {
            Self::Space(advance) => *advance,
            Self::Bitmap(glyph) => glyph.advance,
        };
        if bold { advance + 1.0 } else { advance }
    }
}

/// The default font's glyphs and the images they are cut from.
#[derive(Debug, Clone, Default)]
pub struct Font {
    /// Each bitmap provider's image, by its texture name (`font/ascii`).
    pub images: Vec<(String, DecodedImage)>,
    glyphs: HashMap<u32, Glyph>,
    /// `FontSet.glyphsByWidth`: the codepoints by their advance rounded up, in codepoint order.
    by_width: HashMap<i32, Vec<u32>>,
}

impl Font {
    /// Read `minecraft:font/default.json` and what it refers to.
    pub fn load(assets_root: &Path) -> Option<Self> {
        Self::load_from(&FolderResources::new(assets_root))
    }

    /// The same, from the resource packs.
    pub fn load_from(resources: &dyn Resources) -> Option<Self> {
        Self::load_font(resources, "minecraft:default")
    }

    /// The `minecraft:alt` font the enchanting table writes its names in.
    pub fn load_alt(assets_root: &Path) -> Option<Self> {
        Self::load_alt_from(&FolderResources::new(assets_root))
    }

    /// The same, from the resource packs.
    pub fn load_alt_from(resources: &dyn Resources) -> Option<Self> {
        Self::load_font(resources, "minecraft:alt")
    }

    fn load_font(resources: &dyn Resources, id: &str) -> Option<Self> {
        let mut font = Self {
            images: Vec::new(),
            glyphs: HashMap::new(),
            by_width: HashMap::new(),
        };
        font.load_definition(resources, id)?;
        font.index_widths();
        Some(font)
    }

    /// `FontManager.loadResourceStack`: the definition of every pack, the providers of a higher
    /// pack coming before those of a lower one, and a pack's own in the order of its file.
    fn load_definition(&mut self, root: &dyn Resources, id: &str) -> Option<()> {
        let id = Identifier::parse(&if id.contains(':') {
            id.to_owned()
        } else {
            format!("minecraft:{id}")
        })
        .ok()?;
        let file =
            Identifier::parse(&format!("{}:font/{}.json", id.namespace(), id.path())).ok()?;
        let stack = root.read_stack(&file);
        if stack.is_empty() {
            return None;
        }
        for bytes in stack.iter().rev() {
            let text = String::from_utf8(bytes.clone()).ok()?;
            self.load_providers(root, &text)?;
        }
        Some(())
    }

    fn load_providers(&mut self, root: &dyn Resources, text: &str) -> Option<()> {
        let JsonValue::Object(definition) = photon_core::json::parse(text).ok()? else {
            return None;
        };
        for provider in definition.array("providers")? {
            let JsonValue::Object(provider) = provider else {
                return None;
            };
            // The options' defaults: the font is not forced uniform, and
            // Japanese glyph variants are off.
            if let Some(filter) = provider.object("filter")
                && filter
                    .entries()
                    .any(|(_, wanted)| !matches!(wanted, JsonValue::Bool(false)))
            {
                continue;
            }
            match provider.text("type")?.as_str() {
                "reference" => self.load_definition(root, &provider.text("id")?)?,
                "space" => {
                    for (text, advance) in provider.object("advances")?.entries() {
                        let codepoint = *codepoints(text).first()?;
                        let advance = advance.as_f32()?;
                        self.glyphs
                            .entry(codepoint)
                            .or_insert(Glyph::Space(advance));
                    }
                }
                "bitmap" => self.load_bitmap(root, provider)?,
                _ => {}
            }
        }
        Some(())
    }

    /// `BitmapProvider.Definition.load`.
    fn load_bitmap(&mut self, root: &dyn Resources, provider: &JsonObject) -> Option<()> {
        let file = provider.text("file")?;
        let file = file.as_str();
        let (namespace, path) = file.split_once(':').unwrap_or(("minecraft", file));
        let id = Identifier::parse(&format!("{namespace}:textures/{path}")).ok()?;
        let image = crate::texture::png::decode_rgba(&root.read(&id)?).ok()?;
        let height = provider.i32("height").unwrap_or(8);
        let ascent = provider.i32("ascent")?;
        let rows: Vec<Vec<u32>> = provider
            .array("chars")?
            .iter()
            .map(|row| match row {
                JsonValue::String(units) => Some(codepoints(units)),
                _ => None,
            })
            .collect::<Option<_>>()?;
        let columns = rows.first()?.len() as u32;
        let cell_width = image.width / columns;
        let cell_height = image.height / rows.len() as u32;
        let scale = height as f32 / cell_height as f32;
        let index = self.images.len();
        for (slot_y, row) in rows.iter().enumerate() {
            for (slot_x, &codepoint) in row.iter().enumerate() {
                if codepoint == 0 {
                    continue;
                }
                let (x, y) = (slot_x as u32 * cell_width, slot_y as u32 * cell_height);
                // `getActualGlyphWidth`: the rightmost column with alpha.
                let width = (0..cell_width)
                    .rev()
                    .find(|column| {
                        (0..cell_height).any(|row| {
                            let pixel = ((y + row) * image.width + x + column) as usize;
                            image.rgba[pixel * 4 + 3] != 0
                        })
                    })
                    .map_or(0, |column| column + 1);
                let advance = (0.5 + width as f32 * scale) as i32 + 1;
                self.glyphs
                    .entry(codepoint)
                    .or_insert(Glyph::Bitmap(BitmapGlyph {
                        image: index,
                        cell: [x, y, cell_width, cell_height],
                        scale,
                        ascent,
                        advance: advance as f32,
                    }));
            }
        }
        self.images
            .push((path.trim_end_matches(".png").to_owned(), image));
        Some(())
    }

    fn index_widths(&mut self) {
        let mut codepoints: Vec<u32> = self.glyphs.keys().copied().collect();
        codepoints.sort_unstable();
        for codepoint in codepoints {
            let width = self.glyphs[&codepoint].advance(false).ceil() as i32;
            self.by_width.entry(width).or_default().push(codepoint);
        }
    }

    /// `FontSet.getRandomGlyph`: a glyph as wide as `width`, picked by `roll`; none if there is
    /// none.
    pub fn random_glyph(&self, width: i32, roll: u32) -> Option<Glyph> {
        let same = self.by_width.get(&width).filter(|same| !same.is_empty())?;
        self.glyphs.get(&same[roll as usize % same.len()]).copied()
    }

    pub fn glyph(&self, codepoint: char) -> Option<Glyph> {
        self.glyphs.get(&u32::from(codepoint)).copied()
    }

    /// `Font.width`: the advances added up, rounded up.
    pub fn width(&self, text: &str) -> i32 {
        text.chars()
            .filter_map(|codepoint| self.glyph(codepoint))
            .map(|glyph| glyph.advance(false))
            .sum::<f32>()
            .ceil() as i32
    }

    /// `Font.width` of styled text: bold glyphs advance a unit more.
    pub fn width_of_text_runs(&self, runs: &[crate::gui::TextRun]) -> i32 {
        runs.iter()
            .flat_map(|run| {
                let bold = run.bold;
                run.text
                    .chars()
                    .filter_map(|codepoint| self.glyph(codepoint))
                    .map(move |glyph| glyph.advance(bold))
            })
            .sum::<f32>()
            .ceil() as i32
    }
}

/// `String.codePoints`: UTF-16 units to codepoints, pairs joined and a lone
/// surrogate kept as itself.
fn codepoints(units: &[u16]) -> Vec<u32> {
    char::decode_utf16(units.iter().copied())
        .map(|decoded| match decoded {
            Ok(character) => u32::from(character),
            Err(error) => u32::from(error.unpaired_surrogate()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_surrogate_pair_is_one_codepoint() {
        assert_eq!(codepoints(&[0xD83D, 0xDE00, 0x41]), vec![0x1F600, 0x41]);
        assert_eq!(codepoints(&[0xD800]), vec![0xD800]);
    }
}
