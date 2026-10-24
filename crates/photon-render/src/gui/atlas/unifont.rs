//! Room in the atlas for unifont glyphs, filled as text needs them. The reference stitches each
//! glyph into a 256 by 256 page of its own when it is first drawn; here the atlas holds a block
//! of 16 by 16 cells, and a glyph takes the next free one. When every cell is taken the glyphs
//! are forgotten and the cells start over.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::gui::font::{Font, UnihexGlyph};
use crate::texture::atlas::{SpriteEntry, Stitcher};
use crate::texture::png::DecodedImage;
use photon_core::identifier::Identifier;

/// How many glyphs the atlas holds at once.
const CELLS: usize = 2048;
/// A cell's width and height: a unifont glyph is 16 rows tall, and none drawn is wider.
const SIZE: u32 = 16;
/// The name the reserved cells are stitched under.
pub(super) const PREFIX: &str = "minecraft:unifont_cell/";

/// Reserve the cells with the stitcher.
pub(super) fn register(
    stitcher: &mut Stitcher,
    images: &mut std::collections::BTreeMap<String, DecodedImage>,
) -> Option<()> {
    for cell in 0..CELLS {
        let key = format!("{PREFIX}{cell}");
        stitcher.register_sprite(SpriteEntry {
            width: SIZE as i32,
            height: SIZE as i32,
            name: Identifier::parse(&key).ok()?,
        });
        images.insert(
            key,
            DecodedImage {
                width: SIZE,
                height: SIZE,
                rgba: vec![0; (SIZE * SIZE * 4) as usize],
            },
        );
    }
    Some(())
}

/// The glyph cells of an atlas, shared by its clones.
#[derive(Debug, Clone, Default)]
pub(super) struct GlyphCells(Arc<Mutex<Cells>>);

#[derive(Debug, Default)]
struct Cells {
    /// Each cell's corner in texels, in the order they are handed out.
    origins: Vec<(u32, u32)>,
    atlas: (u32, u32),
    next: usize,
    /// Where each glyph is: its cell and width, or none for one with nothing to draw.
    placed: HashMap<(u8, char), Option<(usize, u32)>>,
    /// The cells written since the atlas image was last updated: cell, width, RGBA rows.
    pending: Vec<(usize, u32, Vec<u8>)>,
}

impl GlyphCells {
    /// Remember where the stitcher put a cell.
    pub(super) fn add_cell(&self, origin: (u32, u32), atlas: (u32, u32)) {
        if let Ok(mut cells) = self.0.lock() {
            cells.origins.push(origin);
            cells.atlas = atlas;
        }
    }

    /// The texture rectangle `[u0, v0, u1, v1]` of a glyph, put into a cell the first time it is
    /// asked for; none if it has nothing to draw. The reference insets it by a hundredth of a
    /// texel (`FontTexture.add`).
    pub(super) fn uv(&self, font: &Font, codepoint: char, glyph: &UnihexGlyph) -> Option<[f32; 4]> {
        let mut cells = self.0.lock().ok()?;
        let key = (glyph.source, codepoint);
        let (cell, width) = match cells.placed.get(&key) {
            Some(placed) => (*placed)?,
            None => cells.place(key, font.unihex_pixels(codepoint, glyph)?)?,
        };
        let (x, y) = cells.origins[cell];
        let (w, h) = (cells.atlas.0 as f32, cells.atlas.1 as f32);
        let nudge = 0.01;
        Some([
            (x as f32 + nudge) / w,
            (y as f32 + nudge) / h,
            (x as f32 - nudge + width as f32) / w,
            (y as f32 - nudge + SIZE as f32) / h,
        ])
    }

    /// Write the glyphs put into cells since the last call into `image`, the atlas as the GPU
    /// has it; whether any were.
    pub(super) fn apply(&self, image: &mut DecodedImage) -> bool {
        let Ok(mut cells) = self.0.lock() else {
            return false;
        };
        let pending = std::mem::take(&mut cells.pending);
        for (cell, width, rgba) in &pending {
            let (x, y) = cells.origins[*cell];
            let row_bytes = *width as usize * 4;
            for row in 0..SIZE {
                let to = (((y + row) * image.width + x) * 4) as usize;
                let from = row as usize * row_bytes;
                // A cell is cleared first: the glyph before it may have been wider.
                image.rgba[to..to + SIZE as usize * 4].fill(0);
                image.rgba[to..to + row_bytes].copy_from_slice(&rgba[from..from + row_bytes]);
            }
        }
        !pending.is_empty()
    }
}

impl Cells {
    /// Take the next cell for `key`, whose bitmap is `pixels`.
    fn place(&mut self, key: (u8, char), pixels: Vec<u8>) -> Option<(usize, u32)> {
        let width = (pixels.len() / (SIZE as usize * 4)) as u32;
        // Nothing to draw, or more than a cell holds.
        if width > SIZE || pixels.iter().all(|&byte| byte == 0) {
            self.placed.insert(key, None);
            return None;
        }
        if self.next >= self.origins.len() {
            self.placed.clear();
            self.next = 0;
        }
        let cell = self.next;
        self.next += 1;
        self.placed.insert(key, Some((cell, width)));
        self.pending.push((cell, width, pixels));
        Some((cell, width))
    }
}
