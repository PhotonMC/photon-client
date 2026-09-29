//! The in-game HUD, laid out the way the reference's HUD lays it out.
//!
//! Everything is in GUI units: the window's pixels divided by the GUI scale,
//! which by default is the largest whole number that keeps the GUI at least
//! 320 by 240. Sprites come from the GUI atlas, the reference's
//! `gui/sprites` folder, drawn at the size the HUD asks for.

use std::collections::BTreeMap;
use std::path::Path;

use crate::atlas::{AtlasSprite, SpriteEntry, Stitcher};
use crate::png::DecodedImage;
use voxel_core::identifier::Identifier;
use voxel_core::random::RandomSource;

/// The sprite folders the HUD and the screens draw from, under
/// `textures/gui/sprites`.
const SPRITE_FOLDERS: [&str; 4] = ["hud", "container", "recipe_book", "tooltip"];

/// Whole textures under `textures/gui` that screens blit parts of, packed as
/// `texture/<name>`.
const WHOLE_TEXTURES: [&str; 4] = [
    "container/inventory",
    "container/creative_inventory/tab_items",
    "container/creative_inventory/tab_item_search",
    "container/creative_inventory/tab_inventory",
];

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
    pub image: DecodedImage,
    sprites: BTreeMap<String, [f32; 4]>,
}

impl GuiAtlas {
    /// Read the HUD's sprites from an asset folder and pack them.
    pub fn load(assets_root: &Path, font: &crate::font::Font) -> Option<Self> {
        let mut images = BTreeMap::new();
        let mut stitcher = Stitcher::new(4096, 4096, 0, 0);
        // The font's images go in whole, and glyphs are cut from them.
        for (name, image) in &font.images {
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
        Some(Self {
            image: atlas,
            sprites,
        })
    }

    fn sprite(&self, name: &str) -> [f32; 4] {
        self.sprites
            .get(&format!("minecraft:{name}"))
            .copied()
            .unwrap_or([0.0; 4])
    }
}

/// One GUI vertex, in GUI units.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GuiVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub colour: [f32; 4],
}

/// How a run of GUI triangles blends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiBlend {
    /// Source alpha over the destination.
    Translucent,
    /// The crosshair's inversion of what is behind it.
    Invert,
    /// Items in slots, whose run counts in `GuiFrame::items`.
    Items,
    /// The player's model in a screen, whose run counts in
    /// `GuiFrame::items` and which samples the player's skin.
    Entity,
}

/// One frame of GUI: triangles and the runs they blend in.
#[derive(Debug, Default)]
pub struct GuiFrame {
    pub vertices: Vec<GuiVertex>,
    pub batches: Vec<(GuiBlend, u32, u32)>,
    pub items: Vec<crate::gui_items::GuiItemVertex>,
}

impl GuiFrame {
    /// `GuiGraphics.fillGradient`: a rectangle from one colour at the top to
    /// another at the bottom (`0xAARRGGBB`).
    pub fn fill_gradient(
        &mut self,
        atlas: &GuiAtlas,
        (x0, y0, x1, y1): (i32, i32, i32, i32),
        top: u32,
        bottom: u32,
    ) {
        let [u0, v0, u1, v1] = atlas.sprite("white");
        let (u, v) = ((u0 + u1) / 2.0, (v0 + v1) / 2.0);
        let unpack =
            |colour: u32| [16, 8, 0, 24].map(|shift| ((colour >> shift) & 0xFF) as f32 / 255.0);
        let corner = |px: i32, py: i32, colour: u32| GuiVertex {
            position: [px as f32, py as f32],
            uv: [u, v],
            colour: unpack(colour),
        };
        let corners = [
            corner(x0, y0, top),
            corner(x0, y1, bottom),
            corner(x1, y1, bottom),
            corner(x1, y0, top),
        ];
        let first = self.vertices.len() as u32;
        for index in [0, 1, 2, 0, 2, 3] {
            self.vertices.push(corners[index]);
        }
        match self.batches.last_mut() {
            Some((GuiBlend::Translucent, _, count)) => *count += 6,
            _ => self.batches.push((GuiBlend::Translucent, first, 6)),
        }
    }

    /// `GuiGraphics.fill` in one colour.
    pub fn fill(&mut self, atlas: &GuiAtlas, rect: (i32, i32, i32, i32), colour: u32) {
        self.fill_gradient(atlas, rect, colour, colour);
    }

    /// `GuiGraphics.blitSprite` of a whole sprite.
    pub fn sprite(&mut self, atlas: &GuiAtlas, name: &str, x: i32, y: i32, w: i32, h: i32) {
        self.blit(atlas, GuiBlend::Translucent, name, x, y, w, h);
    }

    /// `GuiGraphics.blit` of a region of a whole texture: `(u, v)` and the
    /// drawn size in texels of a texture `texture_size` big.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_texture(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        texture_size: (i32, i32),
        uv: (i32, i32),
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) {
        self.blit_part(
            atlas,
            GuiBlend::Translucent,
            &format!("texture/{name}"),
            texture_size,
            uv,
            x,
            y,
            w,
            h,
        );
    }

    /// `GuiGraphics.item` and `itemDecorations`: an item in the slot at
    /// `(x, y)`, then its count (or the given text) in the given colour when
    /// it is not one.
    #[allow(clippy::too_many_arguments)]
    pub fn item_with_count(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::font::Font,
        draw: &mut ItemDrawer<'_>,
        stack: &voxel_world::ClientItemStackState,
        count: i32,
        text: Option<(&str, u32)>,
        x: i32,
        y: i32,
    ) {
        self.item(draw, stack, x, y);
        let count_text = count.to_string();
        let shown = match text {
            Some(text) => Some(text),
            None if count != 1 => Some((count_text.as_str(), 0xFFFF_FFFF)),
            None => None,
        };
        if let Some((text, colour)) = shown {
            self.text_with_shadow(
                atlas,
                font,
                text,
                x + 19 - 2 - font.width(text),
                y + 6 + 3,
                colour,
            );
        }
    }

    /// `GuiGraphics.entity`: an entity's triangles, in GUI units with depth
    /// towards the viewer, textured from the skin.
    pub fn entity(&mut self, vertices: &[crate::gui_items::GuiItemVertex]) {
        if vertices.is_empty() {
            return;
        }
        let first = self.items.len() as u32;
        self.items.extend_from_slice(vertices);
        self.batches
            .push((GuiBlend::Entity, first, vertices.len() as u32));
    }

    /// `GuiGraphics.item`: one item's triangles into the slot at `(x, y)`.
    pub fn item(
        &mut self,
        draw: &mut ItemDrawer<'_>,
        stack: &voxel_world::ClientItemStackState,
        x: i32,
        y: i32,
    ) {
        let first = self.items.len() as u32;
        draw(stack, x as f32, y as f32, &mut self.items);
        let added = self.items.len() as u32 - first;
        if added == 0 {
            return;
        }
        match self.batches.last_mut() {
            Some((GuiBlend::Items, _, count)) => *count += added,
            _ => self.batches.push((GuiBlend::Items, first, added)),
        }
    }

    /// `GuiGraphics.blitSprite` of a nine-slice sprite: the corners as they
    /// are, the edges and the middle tiled (or stretched, when the sprite
    /// says so) over what is left. `tex` is the sprite's size in units.
    #[allow(clippy::too_many_arguments)]
    pub fn nine_slice(
        &mut self,
        atlas: &GuiAtlas,
        name: &str,
        tex: (i32, i32),
        border: i32,
        stretch_inner: bool,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) {
        let blend = GuiBlend::Translucent;
        let (inner_w, inner_h) = (tex.0 - 2 * border, tex.1 - 2 * border);
        let (mid_w, mid_h) = (w - 2 * border, h - 2 * border);
        // One segment: its source origin and size, and where it goes.
        let mut segment = |source: (i32, i32, i32, i32), dest: (i32, i32, i32, i32)| {
            let (sx, sy, sw, sh) = source;
            let (dx, dy, dw, dh) = dest;
            if dw <= 0 || dh <= 0 {
                return;
            }
            if stretch_inner || (sw == dw && sh == dh) {
                self.blit_region(atlas, blend, name, tex, (sx, sy, sw, sh), dest);
                return;
            }
            // Tiled: whole tiles, then the part of one that is left.
            let mut ty = 0;
            while ty < dh {
                let piece_h = sh.min(dh - ty);
                let mut tx = 0;
                while tx < dw {
                    let piece_w = sw.min(dw - tx);
                    self.blit_region(
                        atlas,
                        blend,
                        name,
                        tex,
                        (sx, sy, piece_w, piece_h),
                        (dx + tx, dy + ty, piece_w, piece_h),
                    );
                    tx += sw;
                }
                ty += sh;
            }
        };
        let (right, bottom) = (x + w - border, y + h - border);
        // Corners.
        segment((0, 0, border, border), (x, y, border, border));
        segment(
            (tex.0 - border, 0, border, border),
            (right, y, border, border),
        );
        segment(
            (0, tex.1 - border, border, border),
            (x, bottom, border, border),
        );
        segment(
            (tex.0 - border, tex.1 - border, border, border),
            (right, bottom, border, border),
        );
        // Edges.
        segment((border, 0, inner_w, border), (x + border, y, mid_w, border));
        segment(
            (border, tex.1 - border, inner_w, border),
            (x + border, bottom, mid_w, border),
        );
        segment((0, border, border, inner_h), (x, y + border, border, mid_h));
        segment(
            (tex.0 - border, border, border, inner_h),
            (right, y + border, border, mid_h),
        );
        // The middle.
        segment(
            (border, border, inner_w, inner_h),
            (x + border, y + border, mid_w, mid_h),
        );
    }

    /// `TooltipRenderUtil` and `GuiGraphics.tooltip`: a box of text lines
    /// beside the mouse. Lines are runs of styled text, each 10 units tall
    /// with the first followed by two more; a single line is two units
    /// shorter. The box goes where the mouse's tooltip position puts it in a
    /// screen `screen` units big.
    #[allow(clippy::too_many_arguments)]
    pub fn tooltip(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::font::Font,
        lines: &[Vec<voxel_game::text::Run>],
        mouse: (i32, i32),
        screen: (i32, i32),
    ) {
        if lines.is_empty() {
            return;
        }
        let width = lines
            .iter()
            .map(|line| font.width_of_runs(line))
            .max()
            .unwrap_or(0);
        let height = lines.len() as i32 * 10 - if lines.len() == 1 { 2 } else { 0 };
        // `DefaultTooltipPositioner`.
        let (mut x, mut y) = (mouse.0 + 12, mouse.1 - 12);
        if x + width > screen.0 {
            x = (x - 24 - width).max(4);
        }
        if y + height + 3 > screen.1 {
            y = screen.1 - (height + 3);
        }
        let (x0, y0) = (x - 3 - 9, y - 3 - 9);
        let (padded_w, padded_h) = (width + 6 + 18, height + 6 + 18);
        self.nine_slice(
            atlas,
            "tooltip/background",
            (100, 100),
            9,
            false,
            x0,
            y0,
            padded_w,
            padded_h,
        );
        self.nine_slice(
            atlas,
            "tooltip/frame",
            (100, 100),
            10,
            true,
            x0,
            y0,
            padded_w,
            padded_h,
        );
        let mut line_y = y;
        for (index, line) in lines.iter().enumerate() {
            self.styled_text(atlas, font, line, x, line_y, 0xFFFF_FFFF, true);
            line_y += 10 + if index == 0 { 2 } else { 0 };
        }
    }

    /// `GuiGraphics.blitSprite` of a whole sprite at a size.
    #[allow(clippy::too_many_arguments)]
    fn blit(
        &mut self,
        atlas: &GuiAtlas,
        blend: GuiBlend,
        name: &str,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) {
        self.blit_part(atlas, blend, name, (w, h), (0, 0), x, y, w, h);
    }

    /// `GuiGraphics.text` without a shadow: each glyph of the default font
    /// at its place, in one colour (`0xAARRGGBB`).
    pub fn text(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::font::Font,
        text: &str,
        x: i32,
        y: i32,
        colour: u32,
    ) {
        self.glyphs(atlas, font, text, x, y, colour, false);
    }

    /// `GuiGraphics.text` with its shadow: each glyph's copy a unit right and
    /// down in a quarter of its colour, then the glyph.
    pub fn text_with_shadow(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::font::Font,
        text: &str,
        x: i32,
        y: i32,
        colour: u32,
    ) {
        self.glyphs(atlas, font, text, x, y, colour, true);
    }

    #[allow(clippy::too_many_arguments)]
    fn glyphs(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::font::Font,
        text: &str,
        x: i32,
        y: i32,
        colour: u32,
        shadow: bool,
    ) {
        let runs = [voxel_game::text::Run {
            text: text.to_owned(),
            style: voxel_game::text::Style::default(),
        }];
        self.styled_text(atlas, font, &runs, x, y, colour, shadow);
    }

    /// `GuiGraphics.text` of styled text: each run in its colour, with the
    /// bold glyphs drawn twice a unit apart and the italic ones sheared.
    /// Underlines, strikethrough and obfuscation are not drawn.
    #[allow(clippy::too_many_arguments)]
    pub fn styled_text(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::font::Font,
        runs: &[voxel_game::text::Run],
        x: i32,
        y: i32,
        default_colour: u32,
        shadow: bool,
    ) {
        let unpack =
            |colour: u32| [16, 8, 0, 24].map(|shift| ((colour >> shift) & 0xFF) as f32 / 255.0);
        let mut pen = x as f32;
        for run in runs {
            let colour = run
                .style
                .colour
                .map_or(default_colour, |rgb| (default_colour & 0xFF00_0000) | rgb);
            // `ARGB.scaleRGB(colour, 0.25)`.
            let shadow_colour = [16, 8, 0]
                .map(|shift| ((((colour >> shift) & 0xFF) as f32 * 0.25) as u32) << shift)
                .into_iter()
                .fold(colour & 0xFF00_0000, |sum, channel| sum | channel);
            let (shadow_colour, colour) = (unpack(shadow_colour), unpack(colour));
            let bold = run.style.bold == Some(true);
            let italic = run.style.italic == Some(true);
            for codepoint in run.text.chars() {
                let Some(glyph) = font.glyph(codepoint) else {
                    continue;
                };
                if let crate::font::Glyph::Bitmap(bitmap) = glyph {
                    let (name, image) = &font.images[bitmap.image];
                    let [u0, v0, u1, v1] = atlas.sprite(name);
                    let [cx, cy, cw, ch] = bitmap.cell;
                    let su = |pixel: u32| u0 + (u1 - u0) * pixel as f32 / image.width as f32;
                    let sv = |pixel: u32| v0 + (v1 - v0) * pixel as f32 / image.height as f32;
                    // `GlyphBitmap`: the cell from its left edge, its top at
                    // seven units less the ascent.
                    let up = 7.0 - bitmap.ascent as f32;
                    let down = up + ch as f32 * bitmap.scale;
                    let (shear_top, shear_bottom) = if italic {
                        (1.0 - 0.25 * up, 1.0 - 0.25 * down)
                    } else {
                        (0.0, 0.0)
                    };
                    let thickness = if bold { 0.1 } else { 0.0 };
                    // `(offset, bold shift, colour)` for each pass: the
                    // shadow's, then the glyph's; a bold glyph twice.
                    let mut passes: Vec<(f32, f32, [f32; 4])> = Vec::new();
                    if shadow {
                        passes.push((1.0, 0.0, shadow_colour));
                        if bold {
                            passes.push((1.0, 1.0, shadow_colour));
                        }
                    }
                    passes.push((0.0, 0.0, colour));
                    if bold {
                        passes.push((0.0, 1.0, colour));
                    }
                    for (offset, shift, colour) in passes {
                        let left = pen + offset + shift;
                        let right = left + cw as f32 * bitmap.scale;
                        let top = y as f32 + up + offset;
                        let bottom = y as f32 + down + offset;
                        let corner = |px: f32, py: f32, uv: [f32; 2]| GuiVertex {
                            position: [px, py],
                            uv,
                            colour,
                        };
                        let corners = [
                            corner(
                                left + shear_top - thickness,
                                top - thickness,
                                [su(cx), sv(cy)],
                            ),
                            corner(
                                left + shear_bottom - thickness,
                                bottom + thickness,
                                [su(cx), sv(cy + ch)],
                            ),
                            corner(
                                right + shear_bottom + thickness,
                                bottom + thickness,
                                [su(cx + cw), sv(cy + ch)],
                            ),
                            corner(
                                right + shear_top + thickness,
                                top - thickness,
                                [su(cx + cw), sv(cy)],
                            ),
                        ];
                        let first = self.vertices.len() as u32;
                        for index in [0, 1, 2, 0, 2, 3] {
                            self.vertices.push(corners[index]);
                        }
                        match self.batches.last_mut() {
                            Some((GuiBlend::Translucent, _, count)) => *count += 6,
                            _ => self.batches.push((GuiBlend::Translucent, first, 6)),
                        }
                    }
                }
                pen += glyph.advance(bold);
            }
        }
    }

    /// `GuiGraphics.blitSprite` of part of a sprite: the sprite is taken as
    /// `size` units, and the part from `(u, v)` as big as the drawn area.
    #[allow(clippy::too_many_arguments)]
    fn blit_part(
        &mut self,
        atlas: &GuiAtlas,
        blend: GuiBlend,
        name: &str,
        size: (i32, i32),
        uv: (i32, i32),
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) {
        self.blit_region(atlas, blend, name, size, (uv.0, uv.1, w, h), (x, y, w, h));
    }

    /// A region of a sprite, `(u, v, width, height)` of a sprite `size`
    /// units big, stretched to a rectangle `(x, y, w, h)`.
    #[allow(clippy::too_many_arguments)]
    fn blit_region(
        &mut self,
        atlas: &GuiAtlas,
        blend: GuiBlend,
        name: &str,
        size: (i32, i32),
        (u, v, uw, vh): (i32, i32, i32, i32),
        (x, y, w, h): (i32, i32, i32, i32),
    ) {
        if w == 0 || h == 0 || uw == 0 || vh == 0 {
            return;
        }
        let [u0, v0, u1, v1] = atlas.sprite(name);
        let su = |offset: i32| u0 + (u1 - u0) * offset as f32 / size.0 as f32;
        let sv = |offset: i32| v0 + (v1 - v0) * offset as f32 / size.1 as f32;
        let (left, right) = (su(u), su(u + uw));
        let (top, bottom) = (sv(v), sv(v + vh));
        let corner = |px: i32, py: i32, uv: [f32; 2]| GuiVertex {
            position: [px as f32, py as f32],
            uv,
            colour: [1.0; 4],
        };
        let corners = [
            corner(x, y, [left, top]),
            corner(x, y + h, [left, bottom]),
            corner(x + w, y + h, [right, bottom]),
            corner(x + w, y, [right, top]),
        ];
        let first = self.vertices.len() as u32;
        for index in [0, 1, 2, 0, 2, 3] {
            self.vertices.push(corners[index]);
        }
        match self.batches.last_mut() {
            Some((last, _, count)) if *last == blend => *count += 6,
            _ => self.batches.push((blend, first, 6)),
        }
    }
}

/// Which hearts the player's health is drawn with: `Hud.HeartType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeartType {
    Container,
    Normal,
    Poisoned,
    Withered,
    Absorbing,
    Frozen,
}

impl HeartType {
    /// `HeartType.forPlayer`: poison, then wither, then being frozen through.
    pub fn for_player(poisoned: bool, withered: bool, fully_frozen: bool) -> Self {
        if poisoned {
            Self::Poisoned
        } else if withered {
            Self::Withered
        } else if fully_frozen {
            Self::Frozen
        } else {
            Self::Normal
        }
    }

    /// `HeartType.getSprite`.
    fn sprite(self, hardcore: bool, half: bool, blinking: bool) -> String {
        let (prefix, container) = match self {
            Self::Container => ("container", true),
            Self::Normal => ("", false),
            Self::Poisoned => ("poisoned_", false),
            Self::Withered => ("withered_", false),
            Self::Absorbing => ("absorbing_", false),
            Self::Frozen => ("frozen_", false),
        };
        let blink = if blinking { "_blinking" } else { "" };
        if container {
            let hardcore = if hardcore { "_hardcore" } else { "" };
            return format!("hud/heart/container{hardcore}{blink}");
        }
        let hardcore = if hardcore { "hardcore_" } else { "" };
        let size = if half { "half" } else { "full" };
        format!("hud/heart/{prefix}{hardcore}{size}{blink}")
    }
}

/// What the HUD shows, as the client knows it this frame.
#[derive(Debug, Clone)]
pub struct HudInput {
    /// The camera is in the player's eyes: the crosshair shows.
    pub first_person: bool,
    pub spectator: bool,
    /// Survival or adventure: `canHurtPlayer` and `hasExperience`.
    pub survival: bool,
    pub selected_slot: i32,
    pub health: f32,
    pub max_health: f32,
    pub absorption: f32,
    /// Ticks of damage immunity left.
    pub invulnerable_time: i32,
    pub heart_type: HeartType,
    pub regeneration: bool,
    pub hardcore: bool,
    pub armor: i32,
    pub food: i32,
    pub saturation: f32,
    pub hunger: bool,
    pub air_supply: i32,
    pub max_air_supply: i32,
    pub eye_in_water: bool,
    pub experience_progress: f32,
    pub experience_level: i32,
    /// `getAttackStrengthScale(0)`.
    pub attack_strength: f32,
    /// The hotbar's stacks and the off hand's.
    pub hotbar: [Option<voxel_world::ClientItemStackState>; 9],
    pub offhand: Option<voxel_world::ClientItemStackState>,
}

/// Draws an item's triangles into the 16 by 16 slot at a GUI position.
pub type ItemDrawer<'a> = dyn FnMut(&voxel_world::ClientItemStackState, f32, f32, &mut Vec<crate::gui_items::GuiItemVertex>)
    + 'a;

/// Draws the player's model into a box (`x0, y0, x1, y1`) at a size and an
/// offset (`InventoryScreen.extractEntityInInventoryFollowsMouse`), turned
/// towards the mouse, as triangles for [`GuiFrame::entity`].
pub type PlayerDrawer<'a> = dyn FnMut((i32, i32, i32, i32), f32, f32, (f32, f32)) -> Vec<crate::gui_items::GuiItemVertex>
    + 'a;

/// An item's tooltip as lines of styled text: for a stack, and whether it is
/// on one of the creative inventory's own slots.
pub type TooltipLines<'a> =
    dyn FnMut(&voxel_world::ClientItemStackState, bool) -> Vec<Vec<voxel_game::text::Run>> + 'a;

/// The HUD's own state across frames: its tick count, the health it last
/// showed, and when the hearts blink.
#[derive(Debug)]
pub struct Hud {
    tick_count: i32,
    last_health: i32,
    display_health: i32,
    last_health_time: u128,
    health_blink_time: i64,
    random: RandomSource,
}

impl Default for Hud {
    fn default() -> Self {
        Self {
            tick_count: 0,
            last_health: 0,
            display_health: 0,
            last_health_time: 0,
            health_blink_time: 0,
            random: RandomSource::create(0),
        }
    }
}

impl Hud {
    /// `Hud.tick`: the HUD's tick count moves on.
    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
    }

    /// `Hud.extractRenderState` for what this client has: the crosshair,
    /// then the hotbar and its decorations.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::font::Font,
        width: i32,
        height: i32,
        input: &HudInput,
        now_millis: u128,
        draw_item: &mut ItemDrawer<'_>,
    ) -> GuiFrame {
        let mut frame = GuiFrame::default();
        self.crosshair(&mut frame, atlas, width, height, input);
        let t = GuiBlend::Translucent;
        let centre = width / 2;
        // `extractItemHotbar`; a spectator's hotbar is its own menu.
        if !input.spectator {
            frame.blit(atlas, t, "hud/hotbar", centre - 91, height - 22, 182, 22);
            frame.blit(
                atlas,
                t,
                "hud/hotbar_selection",
                centre - 91 - 1 + input.selected_slot * 20,
                height - 22 - 1,
                24,
                23,
            );
            // The off hand is on the left of a right-handed player, which
            // is the default main arm.
            if input.offhand.is_some() {
                frame.blit(
                    atlas,
                    t,
                    "hud/hotbar_offhand_left",
                    centre - 91 - 29,
                    height - 23,
                    29,
                    24,
                );
            }
            // `extractSlot` for each hotbar slot, then the off hand: the
            // item, then its count when it is not one.
            let y = height - 16 - 3;
            let slots = (0..9)
                .map(|i| (&input.hotbar[i], centre - 90 + i as i32 * 20 + 2))
                .chain(std::iter::once((&input.offhand, centre - 91 - 26)));
            for (stack, x) in slots {
                let Some(stack) = stack else {
                    continue;
                };
                frame.item(draw_item, stack, x, y);
                if stack.item_count() != 1 {
                    let text = stack.item_count().to_string();
                    frame.text_with_shadow(
                        atlas,
                        font,
                        &text,
                        x + 19 - 2 - font.width(&text),
                        y + 6 + 3,
                        0xFFFF_FFFF,
                    );
                }
            }
        }
        if input.survival {
            self.player_health(&mut frame, atlas, width, height, input, now_millis);
            // `ExperienceBar.extractBackground`: every level needs some
            // experience, so the bar always shows.
            let left = (width - 182) / 2;
            let top = height - 24 - 5;
            let progress = (input.experience_progress * 183.0) as i32;
            frame.blit(atlas, t, "hud/experience_bar_background", left, top, 182, 5);
            if progress > 0 {
                frame.blit_part(
                    atlas,
                    t,
                    "hud/experience_bar_progress",
                    (182, 5),
                    (0, 0),
                    left,
                    top,
                    progress,
                    5,
                );
            }
            // `ContextualBar.extractExperienceLevel`: the level in green,
            // outlined by four black copies a unit off.
            if input.experience_level > 0 {
                let text = input.experience_level.to_string();
                let x = (width - font.width(&text)) / 2;
                let y = height - 24 - 9 - 2;
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    frame.text(atlas, font, &text, x + dx, y + dy, 0xFF00_0000);
                }
                frame.text(atlas, font, &text, x, y, 0xFF80_FF20);
            }
        }
        frame
    }

    /// `Hud.extractCrosshair` with the attack indicator under it, as the
    /// default option places it. Nothing is picked as an entity yet, so the
    /// full indicator never shows.
    fn crosshair(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        width: i32,
        height: i32,
        input: &HudInput,
    ) {
        if !input.first_person || input.spectator {
            return;
        }
        let invert = GuiBlend::Invert;
        frame.blit(
            atlas,
            invert,
            "hud/crosshair",
            (width - 15) / 2,
            (height - 15) / 2,
            15,
            15,
        );
        if input.attack_strength < 1.0 {
            let y = height / 2 - 7 + 16;
            let x = width / 2 - 8;
            let progress = (input.attack_strength * 17.0) as i32;
            frame.blit(
                atlas,
                invert,
                "hud/crosshair_attack_indicator_background",
                x,
                y,
                16,
                4,
            );
            frame.blit_part(
                atlas,
                invert,
                "hud/crosshair_attack_indicator_progress",
                (16, 4),
                (0, 0),
                x,
                y,
                progress,
                4,
            );
        }
    }

    /// `Hud.extractPlayerHealth`: armour, hearts, food and air.
    fn player_health(
        &mut self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        width: i32,
        height: i32,
        input: &HudInput,
        now_millis: u128,
    ) {
        let t = GuiBlend::Translucent;
        let current = input.health.ceil() as i32;
        let tick = i64::from(self.tick_count);
        let blink = self.health_blink_time > tick && (self.health_blink_time - tick) / 3 % 2 == 1;
        if current < self.last_health && input.invulnerable_time > 0 {
            self.last_health_time = now_millis;
            self.health_blink_time = tick + 20;
        } else if current > self.last_health && input.invulnerable_time > 0 {
            self.last_health_time = now_millis;
            self.health_blink_time = tick + 10;
        }
        if now_millis.saturating_sub(self.last_health_time) > 1000 {
            self.display_health = current;
            self.last_health_time = now_millis;
        }
        self.last_health = current;
        let old_health = self.display_health;
        // The tick count times the constant is an int product.
        self.random
            .set_seed(i64::from(self.tick_count.wrapping_mul(312_871)));
        let x_left = width / 2 - 91;
        let x_right = width / 2 + 91;
        let y_base = height - 39;
        let max_health = input.max_health.max(old_health.max(current) as f32);
        let absorption = input.absorption.ceil() as i32;
        let rows = ((max_health + absorption as f32) / 2.0 / 10.0).ceil() as i32;
        let row_height = (10 - (rows - 2)).max(3);
        let mut y_air = y_base - 10;
        let heart_offset_index = if input.regeneration {
            self.tick_count % (max_health + 5.0).ceil() as i32
        } else {
            -1
        };

        // `extractArmor`.
        if input.armor > 0 {
            let y = y_base - (rows - 1) * row_height - 10;
            for i in 0..10 {
                let x = x_left + i * 8;
                if i * 2 + 1 < input.armor {
                    frame.blit(atlas, t, "hud/armor_full", x, y, 9, 9);
                }
                if i * 2 + 1 == input.armor {
                    frame.blit(atlas, t, "hud/armor_half", x, y, 9, 9);
                }
                if i * 2 + 1 > input.armor {
                    frame.blit(atlas, t, "hud/armor_empty", x, y, 9, 9);
                }
            }
        }

        // `extractHearts`.
        let health_containers = (f64::from(max_health) / 2.0).ceil() as i32;
        let absorption_containers = (f64::from(absorption) / 2.0).ceil() as i32;
        let max_health_halves = health_containers * 2;
        let hardcore = input.hardcore;
        let heart =
            |frame: &mut GuiFrame, kind: HeartType, x: i32, y: i32, blinking: bool, half: bool| {
                frame.blit(atlas, t, &kind.sprite(hardcore, half, blinking), x, y, 9, 9);
            };
        for index in (0..health_containers + absorption_containers).rev() {
            let row = index / 10;
            let column = index % 10;
            let x = x_left + column * 8;
            let mut y = y_base - row * row_height;
            if current + absorption <= 4 {
                y += self.random.next_int_bound(2).unwrap_or(0);
            }
            if index < health_containers && index == heart_offset_index {
                y -= 2;
            }
            heart(frame, HeartType::Container, x, y, blink, false);
            let halves = index * 2;
            if index >= health_containers {
                let absorption_halves = halves - max_health_halves;
                if absorption_halves < absorption {
                    let half = absorption_halves + 1 == absorption;
                    let kind = if input.heart_type == HeartType::Withered {
                        HeartType::Withered
                    } else {
                        HeartType::Absorbing
                    };
                    heart(frame, kind, x, y, false, half);
                }
            }
            if blink && halves < old_health {
                heart(
                    frame,
                    input.heart_type,
                    x,
                    y,
                    true,
                    halves + 1 == old_health,
                );
            }
            if halves < current {
                heart(frame, input.heart_type, x, y, false, halves + 1 == current);
            }
        }

        // `extractFood`, with no vehicle's hearts in its place.
        let (empty, half, full) = if input.hunger {
            (
                "hud/food_empty_hunger",
                "hud/food_half_hunger",
                "hud/food_full_hunger",
            )
        } else {
            ("hud/food_empty", "hud/food_half", "hud/food_full")
        };
        for i in 0..10 {
            let mut y = y_base;
            if input.saturation <= 0.0 && self.tick_count % (input.food * 3 + 1) == 0 {
                y += self.random.next_int_bound(3).unwrap_or(1) - 1;
            }
            let x = x_right - i * 8 - 9;
            frame.blit(atlas, t, empty, x, y, 9, 9);
            if i * 2 + 1 < input.food {
                frame.blit(atlas, t, full, x, y, 9, 9);
            }
            if i * 2 + 1 == input.food {
                frame.blit(atlas, t, half, x, y, 9, 9);
            }
        }
        y_air -= 10;

        // `extractAirBubbles`: with no vehicle, the row sits one line up
        // from where the vehicle's rows would start.
        let max_air = input.max_air_supply;
        let air = input.air_supply.clamp(0, max_air);
        let under_water = input.eye_in_water;
        if under_water || air < max_air {
            y_air += 10;
            let bubble =
                |offset: i32| (((air + offset) * 10) as f32 / max_air as f32).ceil() as i32;
            let full_bubbles = bubble(-2);
            let popping = bubble(0);
            let delay = if air != 0 && under_water { 1 } else { 0 };
            let empty_bubbles = 10 - bubble(delay);
            let is_popping = full_bubbles != popping;
            for index in 1..=10 {
                let x = x_right - (index - 1) * 8 - 9;
                if index <= full_bubbles {
                    frame.blit(atlas, t, "hud/air", x, y_air, 9, 9);
                } else if is_popping && index == popping && under_water {
                    frame.blit(atlas, t, "hud/air_bursting", x, y_air, 9, 9);
                } else if index > 10 - empty_bubbles {
                    let wobble = if empty_bubbles == 10 && self.tick_count % 2 == 0 {
                        self.random.next_int_bound(2).unwrap_or(0)
                    } else {
                        0
                    };
                    frame.blit(atlas, t, "hud/air_empty", x, y_air + wobble, 9, 9);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heart_sprites_follow_the_reference_names() {
        assert_eq!(
            HeartType::Container.sprite(false, false, true),
            "hud/heart/container_blinking"
        );
        assert_eq!(
            HeartType::Container.sprite(true, false, false),
            "hud/heart/container_hardcore"
        );
        assert_eq!(
            HeartType::Normal.sprite(false, true, false),
            "hud/heart/half"
        );
        assert_eq!(
            HeartType::Poisoned.sprite(true, false, true),
            "hud/heart/poisoned_hardcore_full_blinking"
        );
    }
}
